"""Signal A: two-stage market fusion (phase 5; the declaration is in SIGNALS.md).

Stage 1: a fundamental model (the base model and feature set without any market-derived column),
walk-forward by Event on the training blocks, predicted out of sample for every evaluated Event and
for every trainable Event that has a priced Match (each Event from a fit on Events that ended before
it began). Stage 2: per evaluated Event, a logistic regression without intercept on
logit(p_fund) and the de-vigged source logits (0 where a source has no price), fitted only on
priced Matches of trainable Events that ended before the Event began. Fewer than `MIN_STAGE2`
stage 2 rows: the Event keeps the base model's prediction.
"""
from __future__ import annotations

import numpy as np
import pandas as pd

from . import evaluate, features
from .market_detail import logit, source_logits
from .models import MODELS

MARKET_COLUMNS = {"logit_anchor", "logit_market", "has_market", "elim_x_anchor", "playoff_x_anchor",
                  "bo1_x_anchor", "bo5_x_anchor"}
TRAIN_BLOCKS = {"scope", "sa_old", "b", "c"}
MIN_STAGE2 = 100


def fundamental_columns(fset: str, kinds: dict[str, str]) -> dict[str, str]:
    cols = [c for g in features.SETS[fset] for c in features.REGISTRY[g].columns]
    return {c: kinds[c] for c in cols if c not in MARKET_COLUMNS}


def stage1(m: pd.DataFrame, X: pd.DataFrame, kinds: dict[str, str], model: str, min_train: int = 150,
           seed: int = 33, log=print) -> pd.Series:
    """Out-of-sample p_fund (index = row position of `m`) for evaluated rows and priced trainable rows."""
    cols = list(kinds)
    anti = np.array([kinds[c] == "anti" for c in cols])
    feats = X.loc[m["match_id"], cols].to_numpy(float)
    y = m["team_a_won"].to_numpy(float)
    events = evaluate.event_infos(m)
    ev = m["event_id"].to_numpy()
    trainable = (~m["is_qualifier"] & m["block"].isin(TRAIN_BLOCKS)).to_numpy()
    evaluated = (m["in_scope"] & m["anchor_p"].notna()).to_numpy()
    priced = m["market_p"].notna().to_numpy()
    want = evaluated | (priced & trainable)
    target = m["in_scope"].to_numpy()
    cls = MODELS[model]
    out = np.full(len(m), np.nan)
    order = sorted({int(e) for e in ev[want]}, key=lambda e: (events[e].begins, e))
    chosen, chosen_at = None, 0
    for i, eid in enumerate(order):
        E = events[eid]
        train_events = [e for e in events.values() if e.trains(E)]
        tr = trainable & np.isin(ev, [e.id for e in train_events])
        te = want & (ev == eid)
        if tr.sum() < min_train:
            continue
        if chosen is None or tr.sum() >= 1.25 * chosen_at:
            chosen = evaluate.select(cls, train_events, ev, tr, feats, y, anti, min_train, seed, None, target)
            chosen_at = tr.sum()
        fit = cls(anti, seed).fit(feats[tr], y[tr], chosen)
        out[te] = fit.predict(feats[te])
        if i % 20 == 0:
            log(f"stage 1 {model}: Event {i + 1}/{len(order)}")
    return pd.Series(out, index=m.index)


def stage2_inputs(m: pd.DataFrame, p_fund: pd.Series, detail: pd.DataFrame, variant: str = "core") -> pd.DataFrame:
    """Stage 2 design matrix (index = rows of `m`). Variants: `core` (declared: power de-vig),
    `proportional`, `shin`, `liquidity`, `movement`, `pin_only`, `fund_market` (one chain logit)."""
    devig = variant if variant in ("proportional", "shin") else "power"
    det = detail.reindex(m["match_id"].to_numpy())
    z = source_logits(det, devig)
    z.index = m.index
    Z = pd.DataFrame(index=m.index)
    Z["z_fund"] = logit(p_fund.to_numpy())
    if variant == "fund_market":
        Z["z_market"] = np.nan_to_num(logit(m["market_p"].fillna(0.5).to_numpy()) * m["market_p"].notna().to_numpy())
        return Z
    Z["z_pin"] = z["z_pin"].fillna(0.0)
    if variant == "pin_only":
        return Z
    Z["z_poly"] = z["z_poly"].fillna(0.0)
    Z["z_kal"] = z["z_kal"].fillna(0.0)
    if variant == "liquidity":
        lim = np.log10(np.fmax(np.fmin(det["pin_lim_a"].to_numpy(float), det["pin_lim_b"].to_numpy(float)), 1.0))
        Z["z_pin_x_limit"] = Z["z_pin"] * np.nan_to_num(lim - 3.0)
        spread = det["kal_spread"].to_numpy(float)
        Z["z_kal_x_tight"] = Z["z_kal"] * np.nan_to_num(np.fmin(0.01 / np.fmax(spread, 0.01), 1.0))
        Z["z_kal_x_vol"] = Z["z_kal"] * np.nan_to_num(np.log1p(det["kal_vol24"].to_numpy(float)) / 10.0)
    if variant == "movement":
        e = source_logits(det, "power", early=True)
        e.index = m.index
        for s in ("pin", "poly", "kal"):
            mv = (z[f"z_{s}"] - e[f"z_{s}"]).fillna(0.0)
            Z[f"move_{s}"] = mv
    return Z


def stage2(m: pd.DataFrame, Z: pd.DataFrame, base: pd.Series, min_rows: int = MIN_STAGE2, C: float = 1.0
           ) -> tuple[pd.Series, dict]:
    """Fusion probability for evaluated rows (index = rows of `m`); `base` (the base model's
    prediction on evaluated rows) where stage 2 cannot be fitted. Returns (p, info)."""
    from sklearn.linear_model import LogisticRegression
    events = evaluate.event_infos(m)
    ev = m["event_id"].to_numpy()
    y = m["team_a_won"].to_numpy(int)
    priced = m["market_p"].notna().to_numpy()
    trainable = (~m["is_qualifier"] & m["block"].isin(TRAIN_BLOCKS)).to_numpy()
    evaluated = (m["in_scope"] & m["anchor_p"].notna()).to_numpy()
    zf = Z["z_fund"].to_numpy()
    ok = ~np.isnan(zf)
    A = Z.to_numpy(float)
    out = pd.Series(np.nan, index=m.index)
    p_fund = 1 / (1 + np.exp(-np.nan_to_num(zf)))
    info = {"fallback_events": [], "coef": {}, "rows": {}}
    for eid in sorted({int(e) for e in ev[evaluated]}, key=lambda e: (events[e].begins, e)):
        E = events[eid]
        te = evaluated & (ev == eid)
        te_priced = te & priced
        # unpriced rows: the stage 1 probability (base where stage 1 had too little training)
        np_rows = te & ~priced
        out[np_rows] = np.where(ok[np_rows], p_fund[np_rows], base[np_rows].to_numpy())
        if not te_priced.any():
            continue
        tr_ids = [e.id for e in events.values() if e.trains(E)]
        tr = priced & trainable & ok & np.isin(ev, tr_ids)
        info["rows"][eid] = int(tr.sum())
        if tr.sum() < min_rows or not ok[te_priced].all():
            out[te_priced] = base[te_priced].to_numpy()
            info["fallback_events"].append(eid)
            continue
        lr = LogisticRegression(C=C, fit_intercept=False, max_iter=5000)
        lr.fit(A[tr], y[tr])
        out[te_priced] = np.clip(lr.predict_proba(A[te_priced])[:, 1], 1e-6, 1 - 1e-6)
        info["coef"][eid] = dict(zip(Z.columns, lr.coef_[0].round(4).tolist()))
    return out, info
