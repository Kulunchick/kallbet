"""Walk-forward evaluation in the spirit of `signal-lab/4`.

- Evaluated: every in-scope Match (tier S/A, not a qualifier) with an anchor, Event by Event.
- Fitted for an Event E: only Matches of other Events that are complete (known end) and ended
  before E began; never a qualifier/RMR Event. Out-of-scope rows (tier B, older Events) may train
  when `train_extra` is set. Holdout Matches are not in the table at all.
- Too little training data (< `min_train` Matches) → the anchor itself, as the lab's combiner.
- Hyperparameters: chosen on the training Events only, by an inner walk-forward split (the
  latest ~30% of training Events validate a fit on the training Events that ended before them).
- Metrics on the paired Matches: accuracy, mean log-loss (clipped 1e-6), Brier; differences
  model − anchor with the Event-level paired bootstrap (2000 resamples, 90%, seed 33).
"""
from __future__ import annotations

import math
import time
from dataclasses import dataclass

import numpy as np
import pandas as pd

from . import stats
from .models import MODELS


@dataclass
class EventInfo:
    id: int
    title: str
    begins: int
    ended: int | None
    tier: str | None
    qualifier: bool
    in_scope: bool

    def trains(self, evaluated: "EventInfo") -> bool:
        return self.id != evaluated.id and self.ended is not None and self.ended < evaluated.begins


def event_infos(matches: pd.DataFrame) -> dict[int, EventInfo]:
    out = {}
    for eid, g in matches.groupby("event_id", sort=False):
        r = g.iloc[0]
        ended = r["event_ended"]
        out[int(eid)] = EventInfo(int(eid), r["event"], int(g["event_begins"].min()),
                                  None if pd.isna(ended) else int(ended), r["tier"],
                                  bool(r["is_qualifier"]), bool(g["in_scope"].any()))
    return out


def _ll(p, y):
    return float(stats.log_loss(p, y).mean())


def walk_forward(matches: pd.DataFrame, X: pd.DataFrame, kinds: dict[str, str], model: str,
                 train_extra: bool = True, min_train: int = 150, seed: int = 33, log=print) -> pd.DataFrame:
    """Per evaluated Match: event_id, team_a_won, anchor, variant, has_market, tier, fallback."""
    cols = list(kinds)
    anti = np.array([kinds[c] == "anti" for c in cols])
    feats = X.loc[matches["match_id"], cols].to_numpy(dtype=float)
    y_all = matches["team_a_won"].to_numpy(dtype=float)
    events = event_infos(matches)
    ev_of = matches["event_id"].to_numpy()
    trainable = ~matches["is_qualifier"].to_numpy()
    if not train_extra:
        trainable &= matches["in_scope"].to_numpy()
    evaluated = (matches["in_scope"] & matches["anchor_p"].notna()).to_numpy()
    cls = MODELS[model]
    order = sorted({int(e) for e in ev_of[evaluated]}, key=lambda e: (events[e].begins, e))
    rows = []
    importance = np.zeros(len(cols))
    chosen, chosen_at = None, 0
    for eid in order:
        E = events[eid]
        train_events = [e for e in events.values() if e.trains(E)]
        tr_ids = {e.id for e in train_events}
        tr = trainable & np.isin(ev_of, list(tr_ids))
        te = evaluated & (ev_of == eid)
        anchor = matches["anchor_p"].to_numpy()[te].astype(float)
        if tr.sum() < min_train:
            variant, fallback, params = anchor, True, None
        else:
            if chosen is None or tr.sum() >= 1.25 * chosen_at:
                chosen = select(cls, train_events, ev_of, tr, feats, y_all, anti, min_train, seed)
                chosen_at = tr.sum()
            params = chosen
            m = cls(anti, seed).fit(feats[tr], y_all[tr], params)
            variant, fallback = m.predict(feats[te]), False
            if hasattr(m, "importance"):
                imp = m.importance()
                importance += te.sum() * imp / max(imp.sum(), 1e-12)  # share of gain, weighted by Matches
        sub = matches[te]
        rows.append(pd.DataFrame({
            "match_id": sub["match_id"].to_numpy(), "event_id": eid, "event": E.title,
            "tier": sub["tier"].to_numpy(), "team_a_won": y_all[te].astype(bool),
            "anchor": anchor, "variant": variant, "has_market": sub["market_p"].notna().to_numpy(),
            "fallback": fallback, "train_matches": int(tr.sum()), "params": str(params),
        }))
    out = pd.concat(rows, ignore_index=True)
    if importance.sum() > 0:
        out.attrs["importance"] = dict(zip(cols, importance / importance.sum()))
    return out


def select(cls, train_events, ev_of, tr, feats, y, anti, min_train, seed):
    """Inner walk-forward: validate on the latest ~30% of the training Events, fitting on the
    training Events that ended before the first of them began. Lowest validation log-loss wins."""
    grid = cls.grid
    evs = sorted(train_events, key=lambda e: (e.begins, e.id))
    n_val = max(3, math.ceil(0.3 * len(evs)))
    if len(evs) <= n_val:
        return grid[0]
    val_events = evs[-n_val:]
    first = min(e.begins for e in val_events)
    inner = [e for e in evs if e.ended is not None and e.ended < first and e not in val_events]
    itr = tr & np.isin(ev_of, [e.id for e in inner])
    iva = tr & np.isin(ev_of, [e.id for e in val_events])
    if itr.sum() < min_train or iva.sum() < 50:
        return grid[0]
    scores = []
    for params in grid:
        m = cls(anti, seed).fit(feats[itr], y[itr], params)
        scores.append(_ll(m.predict(feats[iva]), y[iva].astype(bool)))
    return grid[int(np.argmin(scores))]


def summarize(pred: pd.DataFrame, resamples: int = 2000, level: float = 0.9, seed: int = 33) -> dict:
    y = pred["team_a_won"].to_numpy()
    a, v = pred["anchor"].to_numpy(float), pred["variant"].to_numpy(float)
    ev = pred["event_id"].to_numpy()
    out = {"matches": len(pred), "events": int(pred["event_id"].nunique()),
           "fallback_matches": int(pred["fallback"].sum())}
    for name, fn in (("accuracy", stats.correct), ("log_loss", stats.log_loss), ("brier", stats.brier)):
        sa, sv = fn(a, y), fn(v, y)
        b = stats.event_bootstrap(ev, sv - sa, resamples, level, seed)
        out[name] = {"anchor": float(sa.mean()), "model": float(sv.mean()), "diff": b["mean"],
                     "low": b["low"], "high": b["high"]}
    mk = pred["has_market"].to_numpy()
    out["market_subset"] = {"matches": int(mk.sum()),
                            "log_loss_diff": float((stats.log_loss(v[mk], y[mk]) - stats.log_loss(a[mk], y[mk])).mean())
                            if mk.any() else None,
                            "accuracy_diff": float((stats.correct(v[mk], y[mk]) - stats.correct(a[mk], y[mk])).mean())
                            if mk.any() else None}
    return out


def table(results: dict[str, dict]) -> str:
    head = ("| Model | Matches | Accuracy | Δ acc [90%] | Log-loss | Δ log-loss [90%] | Brier | Δ Brier [90%] "
            "| Market n, Δ acc, Δ ll |")
    lines = [head, "|" + "---|" * 9]
    first = next(iter(results.values()))
    lines.append(f"| anchor | {first['matches']} | {first['accuracy']['anchor']:.4f} | — | "
                 f"{first['log_loss']['anchor']:.4f} | — | {first['brier']['anchor']:.4f} | — | — |")
    for name, r in results.items():
        acc, ll, br, mk = r["accuracy"], r["log_loss"], r["brier"], r["market_subset"]
        lines.append(
            f"| {name} | {r['matches']} | {acc['model']:.4f} | {100*acc['diff']:+.1f} pp [{100*acc['low']:+.1f}, {100*acc['high']:+.1f}] "
            f"| {ll['model']:.4f} | {ll['diff']:+.4f} [{ll['low']:+.4f}, {ll['high']:+.4f}] "
            f"| {br['model']:.4f} | {br['diff']:+.4f} [{br['low']:+.4f}, {br['high']:+.4f}] "
            f"| {mk['matches']}, {100*(mk['accuracy_diff'] or 0):+.1f} pp, {(mk['log_loss_diff'] or 0):+.4f} |")
    return "\n".join(lines)
