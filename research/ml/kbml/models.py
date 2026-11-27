"""Three learned models behind one interface: `fit(X, y, params)` then `predict(X)` → p(team A wins).

Every model is trained on both orientations of each Match (anti columns negated, label flipped)
and predicts p = (f(x) + 1 − f(swap x)) / 2, so p(A beats B) = 1 − p(B beats A) exactly and the
arbitrary PandaScore order of the two teams carries no signal.

Missing values: the feature groups already fill gaps with a neutral value plus a `has_*` flag.
"""
from __future__ import annotations

import warnings

import numpy as np


def swap(X: np.ndarray, anti: np.ndarray) -> np.ndarray:
    Xs = X.copy()
    Xs[:, anti] = -Xs[:, anti]
    return Xs


def augment(X, y, anti):
    return np.vstack([X, swap(X, anti)]), np.concatenate([y, 1.0 - y])


class Model:
    name = ""
    grid: list[dict] = []

    def __init__(self, anti: np.ndarray, seed: int = 33):
        self.anti = anti
        self.seed = seed

    def fit(self, X, y, params):
        Xa, ya = augment(np.asarray(X, float), np.asarray(y, float), self.anti)
        self._fit(Xa, ya, params)
        return self

    def predict(self, X):
        X = np.asarray(X, float)
        p = self._predict(X)
        q = self._predict(swap(X, self.anti))
        return np.clip((p + 1.0 - q) / 2.0, 1e-6, 1 - 1e-6)

    def _fit(self, X, y, params):
        raise NotImplementedError

    def _predict(self, X):
        raise NotImplementedError


class _Scaled(Model):
    def _scale_fit(self, X):
        self.mu = X.mean(axis=0)
        self.mu[self.anti] = 0.0  # keep antisymmetry exact (augmented mean is 0 anyway)
        self.sd = X.std(axis=0)
        self.sd[self.sd < 1e-9] = 1.0
        return (X - self.mu) / self.sd

    def _scale(self, X):
        return (X - self.mu) / self.sd


class LogReg(_Scaled):
    """L2 logistic regression on standardised features."""
    name = "logreg"
    grid = [{"C": 0.01}, {"C": 0.1}, {"C": 1.0}]

    def _fit(self, X, y, params):
        from sklearn.linear_model import LogisticRegression
        self.m = LogisticRegression(C=params["C"], max_iter=2000)
        self.m.fit(self._scale_fit(X), y.astype(int))

    def _predict(self, X):
        return self.m.predict_proba(self._scale(X))[:, 1]


class Lgbm(Model):
    """LightGBM, small trees, strong regularisation (a few thousand Matches)."""
    name = "lgbm"
    grid = [
        {"num_leaves": 4, "n_estimators": 150, "learning_rate": 0.03, "min_child_samples": 40},
        {"num_leaves": 8, "n_estimators": 150, "learning_rate": 0.03, "min_child_samples": 40},
        {"num_leaves": 4, "n_estimators": 400, "learning_rate": 0.03, "min_child_samples": 40},
    ]

    def _fit(self, X, y, params):
        import lightgbm as lgb
        self.m = lgb.LGBMClassifier(subsample=0.8, subsample_freq=1, colsample_bytree=0.8, reg_lambda=1.0,
                                    random_state=self.seed, verbose=-1, n_jobs=4, **params)
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            self.m.fit(X, y.astype(int))

    def _predict(self, X):
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            return self.m.predict_proba(X)[:, 1]


class Mlp(_Scaled):
    """A small torch MLP (one hidden layer, dropout, weight decay), full-batch Adam, seeded."""
    name = "mlp"
    grid = [{"hidden": 16, "weight_decay": 1e-2, "epochs": 200},
            {"hidden": 16, "weight_decay": 1e-3, "epochs": 200},
            {"hidden": 32, "weight_decay": 1e-2, "epochs": 400}]

    def _fit(self, X, y, params):
        import torch
        torch.manual_seed(self.seed)
        torch.set_num_threads(4)
        Xt = torch.tensor(self._scale_fit(X), dtype=torch.float32)
        yt = torch.tensor(y, dtype=torch.float32)
        h = params["hidden"]
        self.net = torch.nn.Sequential(torch.nn.Linear(X.shape[1], h), torch.nn.ReLU(), torch.nn.Dropout(0.1),
                                       torch.nn.Linear(h, 1))
        opt = torch.optim.Adam(self.net.parameters(), lr=1e-2, weight_decay=params["weight_decay"])
        loss_fn = torch.nn.BCEWithLogitsLoss()
        self.net.train()
        for _ in range(params["epochs"]):
            opt.zero_grad()
            loss = loss_fn(self.net(Xt).squeeze(1), yt)
            loss.backward()
            opt.step()
        self.net.eval()

    def _predict(self, X):
        import torch
        with torch.no_grad():
            z = self.net(torch.tensor(self._scale(X), dtype=torch.float32)).squeeze(1)
            return torch.sigmoid(z).numpy().astype(float)


MODELS = {c.name: c for c in (LogReg, Lgbm, Mlp)}
