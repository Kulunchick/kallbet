"""Paired comparisons for phase 5: Δ log-loss, Δ Brier, Δ Brier resolution (Murphy), Δ accuracy.

All differences are variant − base on the same Matches, with the lab's Event bootstrap (whole
Events resampled with the ported RNG, 2000 resamples, 90%, seed 33).

Murphy decomposition: Brier = reliability − resolution + uncertainty, with forecasts binned in ten
equal-width bins. The PandaScore order of the two teams is arbitrary, so each Match enters in both
orientations ((p, y) and (1 − p, 1 − y)): uncertainty is then 0.25 and the decomposition does not
depend on which team is "A". Higher resolution = forecasts that separate winners from losers more.
"""
from __future__ import annotations

import numpy as np

from . import stats

BINS = 10


def murphy(p, y, w=None):
    p = np.asarray(p, float)
    y = np.asarray(y, float)
    w = np.ones(len(p)) if w is None else np.asarray(w, float)
    P = np.concatenate([p, 1 - p])
    Y = np.concatenate([y, 1 - y])
    W = np.concatenate([w, w])
    b = np.minimum((P * BINS).astype(int), BINS - 1)
    n = np.bincount(b, weights=W, minlength=BINS)
    sy = np.bincount(b, weights=W * Y, minlength=BINS)
    sp = np.bincount(b, weights=W * P, minlength=BINS)
    N = W.sum()
    ybar = (W * Y).sum() / N
    nz = n > 0
    rel = float((((sp[nz] / n[nz]) - (sy[nz] / n[nz])) ** 2 * n[nz]).sum() / N)
    res = float((((sy[nz] / n[nz]) - ybar) ** 2 * n[nz]).sum() / N)
    return {"reliability": rel, "resolution": res, "uncertainty": float(ybar * (1 - ybar))}


def _bin_matrix(events, p, y, ids):
    """Per Event × bin: weight n and sums of y and p (both orientations)."""
    k = len(ids)
    idx = np.searchsorted(ids, events)
    P = np.concatenate([p, 1 - p])
    Y = np.concatenate([y, 1 - y])
    I = np.concatenate([idx, idx])
    b = np.minimum((P * BINS).astype(int), BINS - 1)
    flat = I * BINS + b
    n = np.bincount(flat, minlength=k * BINS).reshape(k, BINS)
    sy = np.bincount(flat, weights=Y, minlength=k * BINS).reshape(k, BINS)
    return n.astype(float), sy


def resolution_boot(events, p_var, p_base, y, resamples=2000, level=0.9, seed=33):
    events = np.asarray(events)
    ids = np.unique(events)
    k = len(ids)
    d = stats.draws(k, resamples, seed)
    counts = np.zeros((resamples, k))
    for j in range(k):
        counts[:, j] = (d == j).sum(axis=1)
    out = []
    for p in (p_var, p_base):
        n, sy = _bin_matrix(events, np.asarray(p, float), np.asarray(y, float), ids)
        nb, sb = counts @ n, counts @ sy  # (resamples, bins)
        tot = nb.sum(axis=1, keepdims=True)
        ybar = sb.sum(axis=1, keepdims=True) / tot
        with np.errstate(invalid="ignore", divide="ignore"):
            res = np.nansum(np.where(nb > 0, (sb / nb - ybar) ** 2 * nb, 0.0), axis=1) / tot[:, 0]
        out.append(res)
    diff = np.sort(out[0] - out[1])
    tail = (1 - level) / 2
    point = murphy(p_var, y)["resolution"] - murphy(p_base, y)["resolution"]
    return {"mean": point, "low": stats._quantile(diff, tail), "high": stats._quantile(diff, 1 - tail)}


def compare(p_var, p_base, y, events, resamples=2000, seed=33) -> dict:
    p_var, p_base = np.asarray(p_var, float), np.asarray(p_base, float)
    y = np.asarray(y, bool)
    out = {"matches": int(len(y)), "events": int(len(np.unique(events)))}
    for name, fn in (("log_loss", stats.log_loss), ("brier", stats.brier), ("accuracy", stats.correct)):
        sv, sb = fn(p_var, y), fn(p_base, y)
        b = stats.event_bootstrap(events, sv - sb, resamples, 0.9, seed)
        out[name] = {"base": float(sb.mean()), "variant": float(sv.mean()), "diff": b["mean"],
                     "low": b["low"], "high": b["high"]}
    out["resolution"] = resolution_boot(events, p_var, p_base, y, resamples, 0.9, seed)
    out["resolution"].update(base=murphy(p_base, y)["resolution"], variant=murphy(p_var, y)["resolution"])
    out["reliability"] = {"base": murphy(p_base, y)["reliability"], "variant": murphy(p_var, y)["reliability"]}
    return out


def fmt(d: dict, key: str, scale: float = 1.0, digits: int = 4, unit: str = "") -> str:
    x = d[key]
    f = f"{{:+.{digits}f}}"
    return f"{f.format(x['diff' if 'diff' in x else 'mean'] * scale)}{unit} [{f.format(x['low'] * scale)}, {f.format(x['high'] * scale)}]"
