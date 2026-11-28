"""A data-driven opponent model from the real League's human picks (torch; CPU or GPU).

P(member j picks the anchor favourite of Match k) = σ(α + β·x_k + u_j + v_k), x_k = logit of the
anchor favourite's probability (≥ 0), u_j ~ N(0, τ²) the member's own lean (heterogeneity), v_k ~
N(0, σ²) a shared per-Match swing (herding: members deviate together). α, β, τ: maximum marginal
likelihood over u (Gauss-Hermite, members factor). σ: method of moments on the mean pairwise
agreement between members on the same Match, given α, β and each member's posterior mean lean.
Intervals: Match bootstrap (stats.draws, 2000 resamples, seed 33), all resamples fitted at once as
one batch (block-diagonal Newton).
"""
from __future__ import annotations

import math

import numpy as np
import torch

from . import stats

GH = np.polynomial.hermite.hermgauss(24)


def _gh(device):
    z, w = GH
    return (torch.tensor(z, dtype=torch.float64, device=device),
            torch.tensor(np.log(w / math.sqrt(math.pi)), dtype=torch.float64, device=device))


class Data:
    def __init__(self, x, y, member, match, device="cpu"):
        self.device = device
        t = lambda a, dt=torch.float64: torch.as_tensor(np.asarray(a), dtype=dt, device=device)
        self.x, self.y = t(x), t(y)
        self.member = t(member, torch.long)
        self.match = t(match, torch.long)
        self.J = int(self.member.max()) + 1
        self.K = int(self.match.max()) + 1
        self.onehot = torch.nn.functional.one_hot(self.member, self.J).to(torch.float64)  # (N, J)


def neg_loglik(theta, data: Data, weights):
    """theta (B, 3) = (α, β, τ); weights (B, N). Returns (B,) negative marginal log-likelihoods."""
    z, lw = _gh(data.device)
    a, b, tau = theta[:, 0:1, None], theta[:, 1:2, None], theta[:, 2:3, None].abs()
    eta = a + b * data.x[None, None, :] + math.sqrt(2) * tau * z[None, :, None]  # (B, Q, N)
    ll = data.y * torch.nn.functional.logsigmoid(eta) + (1 - data.y) * torch.nn.functional.logsigmoid(-eta)
    per = (ll * weights[:, None, :]) @ data.onehot  # (B, Q, J)
    return -torch.logsumexp(per + lw[None, :, None], dim=1).sum(dim=1)


def fit(data: Data, weights, iters=60):
    """Batched damped Newton; returns theta (B, 3) with τ ≥ 0."""
    B = weights.shape[0]
    ybar = float(data.y.mean())
    theta = torch.tensor([[math.log(ybar / (1 - ybar)), 0.0, 0.5]], dtype=torch.float64,
                         device=data.device).repeat(B, 1)
    lam = torch.full((B,), 1e-3, dtype=torch.float64, device=data.device)
    eye = torch.eye(3, dtype=torch.float64, device=data.device)
    for _ in range(iters):
        th = theta.clone().requires_grad_(True)
        f = neg_loglik(th, data, weights)
        g = torch.autograd.grad(f.sum(), th, create_graph=True)[0]
        H = torch.stack([torch.autograd.grad(g[:, i].sum(), th, retain_graph=True)[0] for i in range(3)], dim=1)
        g, H, f = g.detach(), H.detach(), f.detach()
        step = torch.linalg.solve(H + lam[:, None, None] * eye, g[..., None])[..., 0]
        step = step.clamp(-1.0, 1.0)
        new = theta - step
        fn = neg_loglik(new, data, weights)
        ok = fn <= f + 1e-12
        theta = torch.where(ok[:, None], new, theta)
        lam = torch.where(ok, lam / 3, lam * 10).clamp(1e-8, 1e8)
    theta[:, 2] = theta[:, 2].abs()
    return theta


def posterior_lean(theta, data: Data, weights):
    """Posterior mean of each member's lean u_j: (B, J)."""
    z, lw = _gh(data.device)
    a, b, tau = theta[:, 0:1, None], theta[:, 1:2, None], theta[:, 2:3, None]
    u = math.sqrt(2) * tau * z[None, :, None]  # (B, Q, 1)
    eta = a + b * data.x[None, None, :] + u
    ll = data.y * torch.nn.functional.logsigmoid(eta) + (1 - data.y) * torch.nn.functional.logsigmoid(-eta)
    per = (ll * weights[:, None, :]) @ data.onehot + lw[None, :, None]  # (B, Q, J)
    post = torch.softmax(per, dim=1)
    return (post * u).sum(dim=1)


def _pairs(data: Data):
    """All pairs of picks on the same Match: (i, l) index tensors."""
    m = data.match.cpu().numpy()
    ii, ll = [], []
    for k in np.unique(m):
        idx = np.flatnonzero(m == k)
        for p in range(len(idx)):
            for q in range(p + 1, len(idx)):
                ii.append(idx[p])
                ll.append(idx[q])
    return torch.tensor(ii, device=data.device), torch.tensor(ll, device=data.device)


def match_sigma(theta, lean, data: Data, weights, iters=40, hi=6.0):
    """σ ≥ 0 matching the observed mean pairwise agreement (pairs weighted by their Match's
    resample count); 0 if agreement is below what independent members would show."""
    i, l = _pairs(data)
    wpair = weights[:, i]  # (B, P): both picks share the Match, so one Match weight
    agree = (data.y[i] == data.y[l]).to(torch.float64)
    obs = (wpair * agree).sum(1) / wpair.sum(1)
    z, lw = _gh(data.device)
    w = lw.exp()
    a, b = theta[:, 0:1], theta[:, 1:2]
    base_i = a + b * data.x[i][None] + lean[:, data.member[i]]
    base_l = a + b * data.x[l][None] + lean[:, data.member[l]]

    def model(sig):  # sig (B,)
        v = math.sqrt(2) * sig[:, None, None] * z[None, :, None]  # (B, Q, 1)
        pi, pl = torch.sigmoid(base_i[:, None] + v), torch.sigmoid(base_l[:, None] + v)
        ag = (pi * pl + (1 - pi) * (1 - pl) * 1.0)
        ag = (ag * w[None, :, None]).sum(1)  # (B, P)
        return (wpair * ag).sum(1) / wpair.sum(1)

    lo_s = torch.zeros_like(obs)
    hi_s = torch.full_like(obs, hi)
    for _ in range(iters):
        mid = (lo_s + hi_s) / 2
        up = model(mid) < obs
        lo_s = torch.where(up, mid, lo_s)
        hi_s = torch.where(up, hi_s, mid)
    sig = (lo_s + hi_s) / 2
    return torch.where(model(torch.zeros_like(obs)) >= obs, torch.zeros_like(obs), sig), obs


def estimate(x, y, member, match, device="cpu", resamples=2000, seed=33, chunk=250):
    """Point estimate and Match-bootstrap draws of (α, β, τ, σ). Returns dict of numpy arrays."""
    data = Data(x, y, member, match, device)
    ones = torch.ones((1, len(data.y)), dtype=torch.float64, device=device)
    th = fit(data, ones)
    lean = posterior_lean(th, data, ones)
    sig, obs = match_sigma(th, lean, data, ones)
    point = np.r_[th[0].cpu().numpy(), sig.cpu().numpy()]
    d = stats.draws(data.K, resamples, seed)  # (R, K) Match indices
    counts = np.zeros((resamples, data.K))
    for r in range(resamples):
        counts[r] = np.bincount(d[r], minlength=data.K)
    boots = []
    for s in range(0, resamples, chunk):
        W = torch.tensor(counts[s:s + chunk], dtype=torch.float64, device=device)[:, data.match]
        t = fit(data, W)
        ln = posterior_lean(t, data, W)
        sg, _ = match_sigma(t, ln, data, W)
        boots.append(np.c_[t.cpu().numpy(), sg.cpu().numpy()])
        del W, t, ln, sg
    if device != "cpu":
        torch.cuda.empty_cache()
    return {"point": point, "boot": np.vstack(boots), "observed_agreement": float(obs[0]),
            "lean": lean[0].cpu().numpy()}


def follow_prob(theta, x, n=24):
    """Mean P(pick the anchor favourite) over members and Matches' swings: E σ(α+βx+N(0,τ²+σ²))."""
    a, b, tau, sig = theta
    s = math.sqrt(tau ** 2 + sig ** 2)
    z, w = np.polynomial.hermite.hermgauss(n)
    x = np.asarray(x, float)[..., None]
    return (w / math.sqrt(math.pi) * (1 / (1 + np.exp(-(a + b * x + math.sqrt(2) * s * z))))).sum(-1)
