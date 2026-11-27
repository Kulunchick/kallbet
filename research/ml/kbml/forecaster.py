"""Pairwise forecaster on the extended archive (phase 4).

`PairwiseForecaster.predict(requests)`: for any two teams, a format and a cutoff, P(team A wins
the series), from a model fitted only on Matches whose result was known strictly before the
cutoff (training rows: in-scope + S/A 2023-24 + tier B + tier C, never a qualifier). A request is a
hypothetical pairing: it has no HLTV page, so the page-specific features (lineups, past-matches and
head-to-head panels, stage note) take the groups' missing-data path; ratings, Elo, form, map pool,
ranks and the weekly ranking are read as of the cutoff from the history like any Match.

Two forecasters (the phase 3 configurations):
- `lgbm-ext`: LightGBM on `p3_all` (best walk-forward log-loss, 0.6198);
- `logreg-ext`: logreg on `p3_rw` (phase 3's declared primary, 0.6235).

Hyperparameters: inner walk-forward on Events completed before the cutoff, validated on in-scope
Matches. One fit per (model, cutoff).
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
import pandas as pd

from . import evaluate, features, hltv_data
from . import timeutil as T
from .run import with_hltv
from .table import load_table

EXT = Path("D:/Projects/KallBet/target/research-archive-ext")
TRAIN_BLOCKS = {"scope", "sa_old", "b", "c"}
CONFIGS = {"lgbm-ext": ("lgbm", "p3_all"), "logreg-ext": ("logreg", "p3_rw")}

#: Columns a request may carry (everything else is missing for a hypothetical pairing).
REQUEST_COLUMNS = ["team_a_key", "team_b_key", "format", "cutoff", "hltv_is_lan", "market_p", "vrs_series_p",
                   "vrs_map_p", "anchor_p", "vrs_points_a", "vrs_points_b", "tier",
                   "hltv_rank_a", "hltv_rank_b", "hltv_rank28_a", "hltv_rank28_b", "hltv_rank84_a", "hltv_rank84_b"]


class PairwiseForecaster:
    def __init__(self, archive: Path | None = EXT, log=print, frames=None):
        """`frames=(history, maps, (pre, records), rankings)` instead of an archive (tests)."""
        self.log = log
        if frames is not None:
            self.history, self.maps, self.hltv, self.rankings = frames
        else:
            m, self.maps, _ = load_table(archive, log=log)
            self.history, self.hltv = with_hltv(m, archive)
            self.rankings = hltv_data.weekly_rankings(archive)
        self.events = evaluate.event_infos(self.history)

    def predict(self, requests: pd.DataFrame, configs=CONFIGS) -> dict[str, np.ndarray]:
        """P(team A wins) per request row and forecaster name. `requests` needs `cutoff`, the two
        team keys and `format`; the other `REQUEST_COLUMNS` are optional (missing = unknown)."""
        req = requests.copy().reset_index(drop=True)
        for c in REQUEST_COLUMNS:
            if c not in req:
                req[c] = None
        req["match_id"] = -(np.arange(len(req)) + 10_000_000)
        req["known_finish"] = pd.array([None] * len(req), dtype="Int64")  # never a result
        req = hltv_data.weekly_columns(req, {}, self.rankings)
        frame = pd.concat([self.history, req], ignore_index=True)
        groups = features.resolve("p3_all")
        X, kinds = features.compute(frame, self.maps, groups, self.hltv)
        h = self.history
        hx = X.loc[h["match_id"]]
        rx = X.loc[req["match_id"]]
        ev = h["event_id"].to_numpy()
        y = h["team_a_won"].to_numpy(float)
        finish = h["known_finish"].astype("float").to_numpy()
        base = (~h["is_qualifier"] & h["block"].isin(TRAIN_BLOCKS)).to_numpy()
        target = h["in_scope"].to_numpy()
        out = {name: np.full(len(req), np.nan) for name in configs}
        for cut in sorted(req["cutoff"].unique()):
            sel = (req["cutoff"] == cut).to_numpy()
            tr = base & (finish < cut)  # NaN (no known finish) is never < cut
            completed = [e for e in self.events.values() if e.ended is not None and e.ended < cut]
            for name, (model, fset) in configs.items():
                cols = [c for g in features.SETS[fset] for c in features.REGISTRY[g].columns]
                anti = np.array([kinds[c] == "anti" for c in cols])
                xt = hx[cols].to_numpy(float)
                cls = evaluate.MODELS[model]
                params = evaluate.select(cls, completed, ev, tr, xt, y, anti, 150, 33, None, target)
                fit = cls(anti, 33).fit(xt[tr], y[tr], params)
                out[name][sel] = fit.predict(rx[cols].to_numpy(float)[sel])
            self.log(f"forecaster: cutoff {T.iso(int(cut))}, {int(tr.sum())} training Matches, {int(sel.sum())} pairings")
        return out


def pickem_hook(archive: Path = EXT):
    """`forecast_all(extra=...)`: adds `lgbm ext` and `logreg ext` to every Stage's forecasts."""
    def extra(future: pd.DataFrame, stages: dict):
        fc = PairwiseForecaster(archive)
        tiers = fc.history.groupby("stage_id")["tier"].first().to_dict()
        req = future.copy()
        req["tier"] = req["stage_id"].map(tiers).fillna("s")
        preds = fc.predict(req)
        out = {}
        for sid in stages:
            mask = (future["stage_id"] == sid).to_numpy()
            out[sid] = {name.replace("-", " "): p[mask] for name, p in preds.items()}
        return out
    return extra
