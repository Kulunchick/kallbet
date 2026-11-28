"""The Swiss simulation of kbml.pickem_check.simulate, vectorised over runs with torch (CUDA when present).

Every run advances round by round in tensors: Buchholz-and-seed priority within each (wins, losses)
bucket, the sidecar's rematch-avoiding pairing tables, BO1/BO3 by bucket, and Match winners drawn
from the model's pairwise probabilities. Uniform draws are consumed in the same order as the numpy
version (one per run per Match: opening Matches in order, then each round's buckets by ascending
wins and their pairs in order), so feeding both the same uniforms gives identical outcomes
(tests/test_swiss_gpu.py). Outcome codes: 0 = 3–0, 1 = 0–3, 2 = 3–1/3–2, 3 = eliminated otherwise.
"""

import numpy as np
import torch

from .pickem_check import TABLES

_TABLES = {n: torch.tensor(t, dtype=torch.long) for n, t in TABLES.items()}


def device(name="auto"):
    if name == "auto":
        return torch.device("cuda" if torch.cuda.is_available() else "cpu")
    return torch.device(name)


def simulate(stage, values, runs, dev, uniform, eta=None):
    """Outcome codes (runs, 16) as a uint8 tensor on `dev`; `uniform(n)` returns n float64 draws on `dev`.
    `eta` (runs, 16), optional: per-run team shocks added in logit to all of a team's Matches."""
    p = torch.zeros((2, 16, 16), dtype=torch.float64, device=dev)
    a, b = np.triu_indices(16, 1)
    v = torch.tensor(np.array(values, dtype=np.float64).reshape(2, -1), device=dev)
    p[:, a, b] = v
    p[:, b, a] = 1 - v
    if eta is not None:
        lg = torch.logit(p.clamp(1e-9, 1 - 1e-9))
    seed = torch.as_tensor(np.arange(16) if stage.published else stage.seed, dtype=torch.int64, device=dev)
    wins = torch.zeros((runs, 16), dtype=torch.int64, device=dev)
    losses = torch.zeros_like(wins)
    played = torch.zeros((runs, 16), dtype=torch.int64, device=dev)
    opponents = torch.zeros((4, runs, 16), dtype=torch.int64, device=dev)
    row = torch.arange(runs, device=dev)
    ones = torch.ones(runs, dtype=torch.int64, device=dev)

    def play(x, y, best_of, rnd):
        if eta is None:
            win = uniform(runs) < p[int(best_of == 3), x, y]
        else:
            q = torch.sigmoid(lg[int(best_of == 3), x, y] + eta[row, x] - eta[row, y])
            win = uniform(runs) < q
        wins.index_put_((row, torch.where(win, x, y)), ones, accumulate=True)
        losses.index_put_((row, torch.where(win, y, x)), ones, accumulate=True)
        played[row, x] |= (1 << y)
        played[row, y] |= (1 << x)
        if rnd < 4:
            opponents[rnd, row, x] = y
            opponents[rnd, row, y] = x

    for x, y in stage.opening:
        play(torch.full((runs,), x, device=dev), torch.full((runs,), y, device=dev), stage.formats[0, 0], 0)
    for r in range(1, 5):
        diff = wins - losses
        buchholz = sum(torch.gather(diff, 1, opponents[k]) for k in range(r))
        priority = seed - 16 * buchholz
        pending = []
        for w in range(max(0, r - 2), min(2, r) + 1):
            l = r - w
            mask = (wins == w) & (losses == l)
            n = int(mask[0].sum())
            pool = torch.argsort(torch.where(mask, priority, torch.full_like(priority, 10000)), dim=1)[:, :n]
            table = _TABLES[n].to(dev)
            choice = torch.zeros(runs, dtype=torch.long, device=dev)
            unresolved = torch.ones(runs, dtype=torch.bool, device=dev)
            for k in range(len(table)):
                pairs = pool[:, table[k]]  # (runs, n/2, 2)
                hit = torch.gather(played, 1, pairs[:, :, 0]) & (1 << pairs[:, :, 1])
                good = unresolved & ~(hit != 0).any(dim=1)
                choice = torch.where(good, k, choice)
                unresolved &= ~good
                if k % 8 == 7 and not bool(unresolved.any()):
                    break
            pending.append((pool.gather(1, table[choice].reshape(runs, -1)).reshape(runs, n // 2, 2),
                            stage.formats[w, l]))
        for pairs, fmt in pending:
            for i in range(pairs.shape[1]):
                play(pairs[:, i, 0], pairs[:, i, 1], fmt, r)
    out = torch.full((runs, 16), 3, dtype=torch.uint8, device=dev)
    out[(wins == 3) & (losses > 0)] = 2
    out[(losses == 3) & (wins == 0)] = 1
    out[(wins == 3) & (losses == 0)] = 0
    return out


def simulate_many(stage, values, runs, dev, batch=2_000_000, seed=33, sd=None):
    """`runs` outcomes as a numpy uint8 array; batch b draws from torch.Generator seeded (seed, b).
    `sd` (16,): per-team standard deviation of the logit shock; its normals come from a second
    generator, so the uniforms are the same with or without shocks."""
    sd_t = None if sd is None else torch.as_tensor(np.asarray(sd, dtype=np.float64), device=dev)
    out = np.empty((runs, 16), dtype=np.uint8)
    for start in range(0, runs, batch):
        n = min(batch, runs - start)
        gen = torch.Generator(device=dev)
        gen.manual_seed(seed * 1_000_003 + start // batch)
        eta = None
        if sd_t is not None:
            g2 = torch.Generator(device=dev)
            g2.manual_seed(seed * 1_000_003 + start // batch + 500_000_000)
            eta = torch.randn((n, 16), generator=g2, dtype=torch.float64, device=dev) * sd_t
        out[start:start + n] = simulate(
            stage, values, n, dev,
            lambda k: torch.rand(k, generator=gen, dtype=torch.float64, device=dev), eta,
        ).cpu().numpy()
    if dev.type == "cuda":
        torch.cuda.synchronize()
    return out
