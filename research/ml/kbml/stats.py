"""`lab/stats.rs` ported: scores, the Event-level paired bootstrap with the lab's RNG
(`SeededRng`: SplitMix64 seeding of xoshiro256++), and the verdict rule."""
from __future__ import annotations

import math

import numpy as np

CLIP = 1e-6
MASK = (1 << 64) - 1


def log_loss(p_a, a_won):
    p = np.where(a_won, p_a, 1.0 - p_a)
    return -np.log(np.clip(p, CLIP, 1.0))


def brier(p_a, a_won):
    p = np.where(a_won, p_a, 1.0 - p_a)
    return (1.0 - p) ** 2


def correct(p_a, a_won):
    return ((np.asarray(p_a) >= 0.5) == np.asarray(a_won)).astype(float)


def _rotl(x, k):
    return ((x << k) | (x >> (64 - k))) & MASK


class SeededRng:
    def __init__(self, seed: int):
        z = seed & MASK
        state = []
        for _ in range(4):
            z = (z + 0x9E3779B97F4A7C15) & MASK
            x = z
            x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & MASK
            x = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & MASK
            state.append(x ^ (x >> 31))
        self.s = state

    def next_u64(self) -> int:
        s = self.s
        result = (_rotl((s[0] + s[3]) & MASK, 23) + s[0]) & MASK
        t = (s[1] << 17) & MASK
        s[2] ^= s[0]
        s[3] ^= s[1]
        s[1] ^= s[2]
        s[0] ^= s[3]
        s[2] ^= t
        s[3] = _rotl(s[3], 45)
        return result


_DRAWS: dict[tuple[int, int, int], np.ndarray] = {}


def draws(k: int, resamples: int, seed: int) -> np.ndarray:
    """The Event indices each resample draws, as `event_bootstrap` draws them: (resamples, k)."""
    key = (k, resamples, seed)
    if key not in _DRAWS:
        rng = SeededRng(seed)
        _DRAWS[key] = np.array([[rng.next_u64() % k for _ in range(k)] for _ in range(resamples)], dtype=np.int64)
    return _DRAWS[key]


def _quantile(sorted_vals: np.ndarray, q: float) -> float:
    h = (len(sorted_vals) - 1) * q
    lo, hi = math.floor(h), math.ceil(h)
    return float(sorted_vals[lo] + (h - math.floor(h)) * (sorted_vals[hi] - sorted_vals[lo]))


def event_bootstrap(events, diffs, resamples: int = 2000, level: float = 0.9, seed: int = 33):
    """Pooled mean, central `level` interval of resampled pooled means, std error; resampling whole
    Events (sorted by id, like the lab's BTreeMap) with replacement. None without a Match."""
    events = np.asarray(events)
    diffs = np.asarray(diffs, dtype=float)
    if len(diffs) == 0 or resamples == 0:
        return None
    ids, inv = np.unique(events, return_inverse=True)
    sums = np.bincount(inv, weights=diffs)
    counts = np.bincount(inv).astype(float)
    k = len(ids)
    d = draws(k, resamples, seed)
    means = sums[d].sum(axis=1) / counts[d].sum(axis=1)
    means.sort()
    avg = means.mean()
    var = ((means - avg) ** 2).mean()
    tail = (1.0 - level) / 2.0
    se = math.sqrt(var)
    return {"mean": float(diffs.mean()), "low": _quantile(means, tail), "high": _quantile(means, 1.0 - tail),
            "std_error": 0.0 if se < 1e-12 else se}
