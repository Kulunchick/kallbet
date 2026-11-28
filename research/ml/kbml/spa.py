"""Hansen's test for superior predictive ability (SPA, Hansen 2005) with an Event block bootstrap,
and the stepwise SPA of Hsu, Hsu and Kuan (2010) to say which configurations survive.

Loss differentials d_k,i = L_benchmark,i − L_k,i (positive: configuration k is better) on the same
Matches. The bootstrap resamples whole Events with replacement (the lab's RNG and draws, 2000
resamples, seed 33), so dependence within an Event is kept. ω_k is the bootstrap standard deviation
of √n·d̄_k. Statistic T = max(0, max_k √n d̄_k / ω_k). Null distributions:
Z*_k = d̄*_k − g(d̄_k), i.e. the centred bootstrap mean d̄*_k − d̄_k plus μ_k = d̄_k − g(d̄_k):
- SPA_c (consistent): g(x) = x·1[√n x/ω_k ≥ −√(2 log log n)] (a clearly worse configuration keeps
  its negative mean and cannot inflate the null);
- SPA_l (lower, g(x) = max(x, 0)) and SPA_u (upper, g(x) = x: White's reality check).
Stepwise: reject every k with √n d̄_k/ω_k above the (1 − α) quantile of the SPA_c maximum over the
not yet rejected set; repeat on the rest until nothing more is rejected.
"""
from __future__ import annotations

import math

import numpy as np

from . import stats


def _boot_means(events, D, resamples=2000, seed=33):
    """(d̄ per k, bootstrap pooled means (resamples, K))."""
    events = np.asarray(events)
    ids, inv = np.unique(events, return_inverse=True)
    k = len(ids)
    sums = np.zeros((k, D.shape[1]))
    np.add.at(sums, inv, D)
    counts = np.bincount(inv).astype(float)
    d = stats.draws(k, resamples, seed)
    C = np.zeros((resamples, k))
    for j in range(k):
        C[:, j] = (d == j).sum(axis=1)
    means = (C @ sums) / (C @ counts)[:, None]
    return D.mean(axis=0), means


def spa(events, losses_bench, losses: dict[str, np.ndarray], resamples=2000, seed=33, alpha=0.10) -> dict:
    names = list(losses)
    D = np.column_stack([np.asarray(losses_bench, float) - np.asarray(losses[k], float) for k in names])
    n = D.shape[0]
    dbar, boot = _boot_means(events, D, resamples, seed)
    centred = boot - dbar  # d̄* − d̄
    omega = np.sqrt(n) * centred.std(axis=0)
    omega[omega < 1e-12] = 1e-12
    t_k = np.sqrt(n) * dbar / omega
    T = max(0.0, float(t_k.max()))
    thr = -math.sqrt(2 * math.log(math.log(n)))
    mu = {"c": np.where(t_k >= thr, 0.0, dbar), "l": np.minimum(dbar, 0.0), "u": np.zeros_like(dbar)}
    pvals = {}
    for kind, m in mu.items():
        Tb = np.maximum(0.0, (np.sqrt(n) * (centred + m) / omega).max(axis=1))
        pvals[kind] = float((Tb >= T).mean())
    # stepwise SPA (consistent recentring)
    alive = np.ones(len(names), bool)
    rejected = []
    while alive.any():
        Tb = np.maximum(0.0, (np.sqrt(n) * (centred[:, alive] + mu["c"][alive]) / omega[alive]).max(axis=1))
        crit = float(np.quantile(Tb, 1 - alpha))
        new = alive & (t_k > crit)
        if not new.any():
            break
        rejected += [names[i] for i in np.where(new)[0]]
        alive &= ~new
    best = int(np.argmax(t_k))
    return {"configurations": len(names), "matches": int(n), "T": T, "p_consistent": pvals["c"],
            "p_lower": pvals["l"], "p_upper": pvals["u"], "best": names[best], "best_t": float(t_k[best]),
            "best_mean_diff": float(dbar[best]), "alpha": alpha, "survivors": rejected,
            "per_config": {k: {"mean_diff": float(dbar[i]), "t": float(t_k[i])} for i, k in enumerate(names)}}


# ---------------------------------------------------------------- phase 5 driver

def _losses(frame, cols, kind):
    y = frame["team_a_won"].to_numpy().astype(bool)
    fn = stats.log_loss if kind == "log_loss" else (lambda p, a: 1.0 - stats.correct(p, a))
    return {c: fn(frame[c].to_numpy(float), y) for c in cols}


def families(cache) -> dict:
    """The three SPA families of SIGNALS.md, each for log-loss and for 0-1 loss (accuracy)."""
    import pandas as pd
    p2 = pd.read_parquet(cache / "spa" / "phase2.parquet")
    p3 = pd.read_parquet(cache / "spa" / "phase3.parquet")
    p5 = pd.read_parquet(cache / "signals" / "preds.parquet")
    key = ["match_id"]
    f = p3.merge(p2.drop(columns=["event_id", "team_a_won", "anchor", "has_market"]), on=key, validate="1:1")
    f = f.merge(p5.drop(columns=["event_id", "team_a_won", "anchor", "has_market"]), on=key, validate="1:1")
    assert len(f) == len(p3) == len(p2) == len(p5)
    c2 = [c for c in p2.columns if c.startswith("p2 ")]
    c3 = [c for c in p3.columns if c.startswith("p3 ")]
    c5 = [c for c in p5.columns if c not in ("match_id", "event_id", "team_a_won", "anchor", "has_market")]
    bases = ("lgbm-ext", "logreg-ext")
    c5_new = [c for c in c5 if c not in bases]  # the two bases are phase 3 configurations already
    y = f["team_a_won"].to_numpy().astype(bool)
    ev = f["event_id"].to_numpy()
    out = {"counts": {"phase2": len(c2), "phase3": len(c3), "phase5": len(c5_new)}}
    for kind in ("log_loss", "accuracy"):
        L = _losses(f, c2 + c3 + c5_new + ["anchor"], kind)
        bench = L.pop("anchor")
        out[f"all_vs_anchor/{kind}"] = spa(ev, bench, L)
        # phase 5 add-ons against their own base, all scored Matches
        Lb = _losses(f, list(bases), kind)
        adds = {c: c for c in c5_new if c.split(" ")[0] in bases and "stage 1" not in c}
        D_bench = {c: Lb[c.split(" ")[0]] for c in adds}
        # SPA needs one benchmark: use differentials directly (benchmark loss − add-on loss)
        Lr = _losses(f, list(adds), kind)
        zero = np.zeros(len(f))
        out[f"addons_vs_base/{kind}"] = spa(ev, zero, {c: Lr[c] - D_bench[c] for c in adds})
        mk = f["has_market"].to_numpy()
        a_cols = [c for c in adds if "+ A[" in c]
        out[f"A_vs_base_market/{kind}"] = spa(ev[mk], zero[mk], {c: (Lr[c] - D_bench[c])[mk] for c in a_cols})
    return out


def main(argv=None):
    import json
    import sys

    from .forecaster import EXT
    from .table import cache_dir
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except AttributeError:
        pass
    cache = cache_dir(EXT)
    res = families(cache)
    (cache / "signals" / "spa.json").write_text(json.dumps(res, indent=1, default=float))
    for k, v in res.items():
        if k == "counts":
            print(v)
            continue
        print(f"{k}: K={v['configurations']} T={v['T']:.2f} p_c={v['p_consistent']:.3f} p_l={v['p_lower']:.3f} "
              f"p_u={v['p_upper']:.3f} best={v['best']} ({v['best_mean_diff']:+.4f}) survivors={len(v['survivors'])}")


if __name__ == "__main__":
    main()
