"""The torch replay equals catchup_dp.simulate with identical opponents; CPU equals GPU."""
import numpy as np
import pytest
import torch

from kbml import catchup_dp as dp, catchup_gpu as cg, opponent_model as om


class ListRng:
    def __init__(self, u):
        self.u = [int(x * (1 << 53)) << 11 for x in u]
        self.i = 0

    def next_u64(self):
        self.i += 1
        return self.u[self.i - 1]


def games(seed, n=30):
    rng = np.random.default_rng(seed)
    out = []
    for i in range(n):
        a = None if i == 7 else float(rng.uniform(0.2, 0.9))
        out.append((i * 10, None if i == 3 else i * 10 + int(rng.integers(5, 40)), a, None, bool(rng.random() < 0.6),
                    {"m": float(rng.uniform(0.3, 0.8))}))
    return out


@pytest.mark.parametrize("size,noise", [(4, 0.1), (12, 0.25)])
def test_replay_equals_python_simulate(size, noise):
    g = games(size)
    n_opp = size - 1
    tabs = dp.build(n_opp, noise, np.array([0.4, 0.6, 0.7]), np.array([1.0, 2.0, 1.0]), 0.0, 0.75, 40)
    mixed = cg.build_mixed(n_opp, [noise], [(0.4, 0, 1.0), (0.6, 0, 2.0), (0.7, 0, 1.0)], 0.0, 0.75, 40)
    assert torch.allclose(mixed.U[5, 0, 1, 1], torch.tensor(np.stack(
        [[tabs.after(5, d - dp.D, s, 1, 1) for s in range(len(tabs.index))] for d in range(2 * dp.D + 1)])))
    ev = cg.event_arrays(g, ["m"], "cpu")
    B = 25
    rs = np.random.default_rng(1)
    picks = np.full((B, n_opp, len(g)), np.nan)
    want = []
    for b in range(B):
        u = rs.random(n_opp * len(g))
        k = 0
        for j, game in enumerate(g):
            if game[2] is None:
                continue
            for o in range(n_opp):
                picks[b, o, j] = float((game[2] >= 0.5) ^ (u[k] < noise))
                k += 1
        want.append(dp.simulate(g, size, noise, ListRng(u), [dp.Plain("m"), dp.Window("m"), dp.DP("m", {None: tabs})]))
    got = cg.replay(ev, torch.tensor(picks), [("m", "plain", None), ("m", "window", None), ("m", "dp", mixed)])
    assert got.tolist() == want


@pytest.mark.skipif(not torch.cuda.is_available(), reason="no CUDA")
def test_cpu_equals_gpu():
    g = games(3)
    out = []
    for dev in ("cpu", "cuda"):
        mixed = cg.build_mixed(11, [0.1, 0.2], [(0.6, 0, 1.0), (0.55, 1, 1.0)], 0.1, 0.7, 40, dev)
        ev = cg.event_arrays(g, ["m"], dev)
        gen = torch.Generator().manual_seed(5)
        pick = cg.fitted_opponents(ev, 200, 11, (1.0, 0.5, 0.5, 0.3), gen, dev)
        eb = torch.zeros(len(g), dtype=torch.long, device=dev)
        out.append(cg.replay(ev, pick, [("m", "plain", None), ("m", "dp", mixed)], eps_bin=eb).cpu())
    assert torch.equal(out[0], out[1])


def test_opponent_model_recovers_parameters():
    rng = np.random.default_rng(2)
    J, K = 30, 200
    x = rng.uniform(0, 2, K)
    u = rng.normal(0, 0.8, J)
    mem = np.repeat(np.arange(J), K)
    mat = np.tile(np.arange(K), J)
    y = rng.random(J * K) < 1 / (1 + np.exp(-(0.3 + 1.2 * x[mat] + u[mem])))
    d = om.Data(x[mat], y.astype(float), mem, mat)
    th = om.fit(d, torch.ones((1, J * K), dtype=torch.float64))[0].numpy()
    assert abs(th[0] - 0.3) < 0.4 and abs(th[1] - 1.2) < 0.25 and abs(th[2] - 0.8) < 0.3
