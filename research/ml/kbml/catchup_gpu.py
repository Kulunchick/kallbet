"""Vectorised League replay in torch (CPU or GPU): many trials of one Event at once.

Same rules as `catchup_dp.simulate` (published standings = results with known finish strictly
before the cutoff and processed earlier; a missing input is a missed pick; ties share first), with
opponents given as a pick tensor so any opponent model can be plugged in. DP tables may use a
different noise level per Match type (`build_mixed`). `tests/test_catchup_gpu.py` checks the replay
against `catchup_dp.simulate` with identical opponent draws, and CPU against GPU.

All random numbers are drawn on the CPU (torch.Generator) and moved to the device, so a run gives
the same result on either.
"""
from __future__ import annotations

import math

import numpy as np
import torch

from . import catchup_dp as dp

F64 = torch.float64


class MixedTables:
    """U[r, e, outcome, ours, d, s] = E[V_{r-1} | outcome, our correctness] for noise bin e."""

    def __init__(self, U, index, n_opp):
        self.U, self.index, self.n_opp = U, index, n_opp
        idx = torch.full((n_opp + 1, n_opp + 1), -1, dtype=torch.long)
        for (c0, c1), s in index.items():
            idx[c0, c1] = s
        self.sidx = idx.to(U.device)


def build_mixed(n_opp, eps_grid, types, w_none, f_none, rmax, device="cpu"):
    """`types`: rows (q, eps bin, weight). Backward recursion as `catchup_dp.build`, but each Match
    type carries its own opponent noise."""
    Ts = [dp.transitions(n_opp, float(e)) for e in eps_grid]
    index = Ts[0][1]
    S, E, Dn = len(index), len(eps_grid), 2 * dp.D + 1
    P = torch.tensor(np.array([[Ts[e][0][o] for o in (0, 1)] for e in range(E)]), dtype=F64, device=device)
    # P[e, o, inc, s, s']
    q = torch.tensor([t[0] for t in types], dtype=F64, device=device)
    eb = torch.tensor([int(t[1]) for t in types], dtype=torch.long, device=device)
    w = torch.tensor([t[2] for t in types], dtype=F64, device=device)
    wsum = float(w.sum()) + w_none
    shift = {k: torch.clamp(torch.arange(Dn, device=device) + k, 0, Dn - 1) for k in (-1, 0, 1)}
    V = torch.zeros((Dn, S), dtype=F64, device=device)
    V[dp.D:] = 1.0
    U_all = [torch.zeros((E, 2, 2, Dn, S), dtype=F64, device=device)]  # r = 0 unused
    for _ in range(rmax):
        U = torch.zeros((E, 2, 2, Dn, S), dtype=F64, device=device)
        for o in (0, 1):
            for ours in (0, 1):
                for inc in (0, 1):
                    # (Dn, S') @ (E, S', S) -> (E, Dn, S)
                    U[:, o, ours] += V[shift[ours - inc]] @ P[:, o, inc].transpose(1, 2)
        U_all.append(U)
        Ut = U[eb]  # (T, 2, 2, Dn, S)
        qq = q[:, None, None]
        cons = qq * Ut[:, 1, 1] + (1 - qq) * Ut[:, 0, 0]
        other = qq * Ut[:, 1, 0] + (1 - qq) * Ut[:, 0, 1]
        Vn = (w[:, None, None] * torch.maximum(cons, other)).sum(0)
        Vn = Vn + w_none * (f_none * V[shift[1]] + (1 - f_none) * V)
        V = Vn / wsum
    return MixedTables(torch.stack(U_all), index, n_opp)


def event_arrays(games, src_names, device):
    """Tensors of one Event's games (tuples as `catchup.events_from`), cutoff order."""
    G = len(games)
    cut = torch.tensor([g[0] for g in games], dtype=F64)
    known = torch.tensor([math.inf if g[1] is None else g[1] for g in games], dtype=F64)
    j = torch.arange(G)
    vis = (j[None, :] < j[:, None]) & (known[None, :] < cut[:, None])  # vis[g, j]
    nan = float("nan")
    anc = torch.tensor([nan if g[2] is None else g[2] for g in games], dtype=F64)
    mkt = torch.tensor([nan if g[3] is None else g[3] for g in games], dtype=F64)
    won = torch.tensor([g[4] for g in games], dtype=torch.bool)
    ps = {s: anc if s == "anchor" else torch.tensor([nan if g[5].get(s) is None else g[5][s] for g in games],
                                                      dtype=F64) for s in src_names}
    to = lambda t: t.to(device)
    return {"G": G, "vis": to(vis), "anchor": to(anc), "market": to(mkt), "won": to(won),
            "p": {k: to(v) for k, v in ps.items()}}


def replay(ev, opp_pick, bots, consensus="anchor", eps_bin=None, return_picks=False):
    """ev: `event_arrays`; opp_pick: (B, N, G) float, 1 = team A, 0 = team B, nan = no pick.
    bots: list of (src, kind, tables) with kind in plain / window / dp. eps_bin: (G,) long, the
    noise bin of each Match for mixed DP tables (zeros if None). Returns places (B, K)."""
    B, N, G = opp_pick.shape
    dev = opp_pick.device
    won = ev["won"].to(F64)
    valid = ~torch.isnan(opp_pick)
    opp_correct = (valid & (opp_pick == won)).to(F64)  # (B, N, G)
    vis = ev["vis"].to(F64)
    opp_vis = opp_correct @ vis.T  # (B, N, G): Points published before each Match
    leader = opp_vis.max(dim=1).values  # (B, G)
    c0 = (opp_vis == leader[:, None]).sum(1)
    c1 = (opp_vis == leader[:, None] - 1).sum(1)
    r_left = G - vis.sum(1)  # (G,)
    cons_p = ev["anchor"] if consensus == "anchor" else ev["market"]
    if eps_bin is None:
        eps_bin = torch.zeros(G, dtype=torch.long, device=dev)
    K = len(bots)
    bot_correct = torch.zeros((B, K, G), dtype=F64, device=dev)
    bot_pick = torch.zeros((B, K, G), dtype=torch.bool, device=dev)
    for g in range(G):
        own = bot_correct[:, :, :g] @ vis[g, :g] if g else torch.zeros((B, K), dtype=F64, device=dev)
        lead = leader[:, g]
        for k, (src, kind, tab) in enumerate(bots):
            p = ev["p"][src][g]
            if torch.isnan(p):
                continue
            fav = bool(p >= 0.5)
            pick = torch.full((B,), fav, dtype=torch.bool, device=dev)
            if kind == "window":
                if p != 0.5 and 0.40 <= float(p) <= 0.60:
                    pick = torch.where(lead - own[:, k] > 0, ~pick, pick)
            elif kind == "dp" and not torch.isnan(cons_p[g]) and r_left[g] >= 1:
                cons_a = bool(cons_p[g] >= 0.5)
                q = float(p) if cons_a else 1 - float(p)
                d = torch.clamp(own[:, k] - lead, -dp.D, dp.D).long() + dp.D
                s = tab.sidx[c0[:, g], c1[:, g]]
                r = min(int(r_left[g]), tab.U.shape[0] - 1)
                U = tab.U[r, int(eps_bin[g])]  # (2, 2, Dn, S)
                cons = q * U[1, 1, d, s] + (1 - q) * U[0, 0, d, s]
                other = q * U[1, 0, d, s] + (1 - q) * U[0, 1, d, s]
                v_fav, v_dog = (cons, other) if cons_a == fav else (other, cons)
                pick = torch.where(v_dog > v_fav + 1e-12, ~pick, pick)
            bot_correct[:, k, g] = (pick == ev["won"][g]).to(F64)
            bot_pick[:, k, g] = pick
    final_opp = opp_correct.sum(-1)  # (B, N)
    final_bot = bot_correct.sum(-1)  # (B, K)
    places = 1 + (final_opp[:, None, :] > final_bot[:, :, None]).sum(-1)
    return (places, bot_pick) if return_picks else places


def fitted_opponents(ev, B, N, theta, gen, device, theta_per_trial=None):
    """Picks (B, N, G) of members following the fitted model σ(α + β·x + u_j + v_k) on the anchor
    favourite. theta = (α, β, τ, σ); theta_per_trial (B, 4) overrides it per trial."""
    G = ev["G"]
    anc = ev["anchor"].cpu()
    fav_a = (anc >= 0.5).to(F64)
    pf = torch.maximum(anc, 1 - anc)
    x = torch.log(pf / (1 - pf))
    th = torch.tensor(theta, dtype=F64).repeat(B, 1) if theta_per_trial is None else torch.as_tensor(theta_per_trial, dtype=F64)
    u = torch.randn((B, N), generator=gen, dtype=F64) * th[:, 2:3]
    v = torch.randn((B, G), generator=gen, dtype=F64) * th[:, 3:4]
    unif = torch.rand((B, N, G), generator=gen, dtype=F64)
    eta = th[:, 0, None, None] + th[:, 1, None, None] * x[None, None, :] + u[:, :, None] + v[:, None, :]
    follow = unif < torch.sigmoid(eta)
    pick = torch.where(follow, fav_a[None, None, :], 1 - fav_a[None, None, :])
    pick = torch.where(torch.isnan(anc)[None, None, :], torch.full_like(pick, float("nan")), pick)
    return pick.to(device)
