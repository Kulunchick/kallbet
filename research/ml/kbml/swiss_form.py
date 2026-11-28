"""Phase 5: the Pick'em Swiss replay with correlated tournament form (SWISS.md).

Inputs are the phase 4 cache (`forecaster-pickem.pkl`: the pairwise probabilities of the 28 Stages
for every forecaster) and the Stage objects of the replay. Variants differ in four things:

- the probability source: a logit ensemble w·lgbm-ext + (1 − w)·logreg-ext (w fixed or walk-forward);
- the between-stage update: a per-team logit shift κ·Σ(result − p) over the team's Matches of the
  previous Stage of the same Event (κ walk-forward);
- the correlated form shock η_i ~ N(0, σ_form² + σ_i²), one per team and run, added in logit to all
  of a team's Matches (σ_form walk-forward by the Brier of the ≥5 event; σ_i point in time);
- strategy B regularised, maximise P(≥5) + λ·E[correct] (λ walk-forward).

Everything is point in time: a selection for Stage k looks only at Stages that finished before k's
cutoff, and team features only at results with a known finish before the cutoff.
"""
from __future__ import annotations

import pickle
import time
from collections import defaultdict
from itertools import combinations
from pathlib import Path

import numpy as np
from scipy.special import expit, logit

from . import stats
from .pickem_check import expected_picks, scores
from .teams import team_key

CACHE = Path("D:/Projects/KallBet/target/ml-cache")
DAY = 86_400_000_000
SIGMA_GRID = (0.0, 0.15, 0.30, 0.45)
SIGMA_DEFAULT, SIGMA_MIN_STAGES = 0.30, 5
TAU = 0.3
LAMBDAS = (0.0, 0.02, 0.05)
WEIGHTS = (0.0, 0.25, 0.5, 0.75, 1.0)
KAPPAS = (0.0, 0.1, 0.2, 0.4)
RUNS = 200_000


# ------------------------------------------------------------------ search with the regularised objective

def search_reg(outcomes, starts, lam=0.0):
    """pickem_check.search with the objective P(≥5) + lam·E[correct]; lam = 0 is the same search."""
    n = len(outcomes)
    marg = np.zeros((16, 4))
    for k in range(3):
        marg[:, k] = (outcomes == k).mean(axis=0)

    def packed(bits):
        return np.packbits(bits)

    hits = [[packed(outcomes[:, t] == k) if k != 3 else packed(np.zeros(n, bool)) for k in range(4)] for t in range(16)]
    visited, best = {}, None
    for initial in starts:
        roles = initial.copy()
        while roles.tobytes() not in visited:
            old = scores(outcomes, roles)
            value = int((old >= 5).sum()) + lam * n * float(marg[np.arange(16), roles].sum())
            visited[roles.tobytes()] = value
            if best is None or value > best[0]:
                best = (value, roles.copy())
            s3, s4, s5, s6 = (packed(old == k) for k in (3, 4, 5, 6))
            gain, move = 1e-9 if lam else 0, None
            for a, b in combinations(range(16), 2):
                i, j = roles[a], roles[b]
                if i == j:
                    continue
                add_a, add_b = hits[a][j], hits[b][i]
                rem_a, rem_b = hits[a][i], hits[b][j]
                up = (add_a & ~rem_b) | (add_b & ~rem_a)
                down = (rem_a & ~add_b) | (rem_b & ~add_a)
                better = (s3 & add_a & add_b) | (s4 & up)
                worse = (s6 & rem_a & rem_b) | (s5 & down)
                delta = int(np.bitwise_count(better).sum()) - int(np.bitwise_count(worse).sum())
                if lam:
                    delta += lam * n * float(marg[a, j] + marg[b, i] - marg[a, i] - marg[b, j])
                if delta > gain:
                    gain, move = delta, (a, b)
            if move is None:
                break
            a, b = move
            roles[a], roles[b] = roles[b], roles[a]
    return best[1]


def strategies(stage, outcomes, lams=(0.0,)):
    """`pickem_check.evaluate_strategy`'s search (same starts) on given outcomes, for each lambda:
    {lam: (roles, realised correct, expected correct, simulated P(≥5))}."""
    runs = len(outcomes)
    small = outcomes[: runs // 2]
    a_small, a = expected_picks(small), expected_picks(outcomes)
    gen = np.random.default_rng(33)
    starts = [gen.permutation(a_small) for _ in range(2)]
    out = {}
    for lam in lams:
        b_small = search_reg(small, [a_small, *starts], lam)
        b = search_reg(outcomes, [b_small, a, *starts], lam)
        sim = scores(outcomes, b)
        out[lam] = (b, int(scores(stage.truth, b)), float(sim.mean()), float((sim >= 5).mean()))
    return out


# ------------------------------------------------------------------ inputs

class Inputs:
    def __init__(self, log=print):
        from .forecaster import EXT
        from .run import with_hltv
        from .table import load_table

        new = pickle.loads((CACHE / "forecaster-pickem.pkl").read_bytes())
        self.rows = new["rows"]
        self.pairs = new["pairs"]
        self.stages = build_stages(self.pairs)
        assert set(self.stages) == set(self.pairs)
        self.order = sorted(self.stages, key=lambda i: self.stages[i].cutoff)
        history, _, _ = load_table(EXT, log=lambda *a: None)
        history, (_, records) = with_hltv(history, EXT)
        self.history = history[~history.duplicate].copy()
        self.records = records
        h = self.history
        self.end = {}
        for sid in self.order:
            f = h.loc[h.stage_id == sid, "known_finish"].dropna()
            self.end[sid] = int(f.max()) if len(f) else None
        self.event = h.groupby("stage_id").event_id.first().to_dict()
        self._matches = {}
        for sid in self.order:
            self.matches(sid)
        log(f"inputs: {len(self.order)} Stages, ends known for {sum(v is not None for v in self.end.values())}")

    def matches(self, sid):
        """(i, j, format index, i won) of the Stage's actual Matches, i < j, indexes into stage.teams."""
        if sid not in self._matches:
            stage = self.stages[sid]
            idx = {team_key(t): i for i, t in enumerate(stage.teams)}
            out = []
            rows = self.history[self.history.stage_id == sid]
            for r in rows.itertuples():
                a, b = idx.get(r.team_a_key), idx.get(r.team_b_key)
                if a is None or b is None:
                    continue
                if r.team_a_won is None or r.team_a_won != r.team_a_won:
                    continue
                out.append((a, b, int(r.format == "bo3"), bool(r.team_a_won)))
            self._matches[sid] = out
        return self._matches[sid]

    def earlier(self, sid):
        """Stages whose last result is known strictly before this Stage's cutoff."""
        c = self.stages[sid].cutoff
        return [j for j in self.order if self.end.get(j) is not None and self.end[j] < c]

    def previous(self, sid):
        """The Stage before this one at the same Event (None for a first Stage)."""
        ev = self.event.get(sid)
        c = self.stages[sid].cutoff
        if "Major" not in self.stages[sid].title:
            return None
        cand = [j for j in self.order if self.event.get(j) == ev and ev is not None and self.stages[j].cutoff < c
                and self.end.get(j) is not None and self.end[j] < c]
        return cand[-1] if cand else None

    # --- point-in-time team uncertainty
    def uncertainty(self, sid):
        """(σ_i array, details) for the Stage's 16 teams at its cutoff."""
        stage = self.stages[sid]
        c = stage.cutoff
        h = self.history
        keys = [team_key(t) for t in stage.teams]
        done = h[h.known_finish.notna() & (h.known_finish < c)]
        recent = done[done.known_finish >= c - 365 * DAY]
        rec = self.records
        rec = rec[rec.known_finish.notna() & (rec.known_finish < c)]
        lineups = defaultdict(list)
        for r in rec["rec"]:
            if r["lineups"]:
                for side in (0, 1):
                    lineups[r["keys"][side]].append((r["started_at"], set(r["lineups"][side])))
        out, det = np.zeros(16), []
        for t, k in enumerate(keys):
            mine = recent[(recent.team_a_key == k) | (recent.team_b_key == k)]
            n90 = int((mine.known_finish >= c - 90 * DAY).sum())
            thin = min(1.0, max(0.0, (8 - n90) / 8))
            opp = set(mine.team_a_key) | set(mine.team_b_key)
            met = sum(o in opp for o in keys if o != k)
            isolation = 1 - met / 15
            ls = sorted(lineups.get(k, []), key=lambda x: x[0])
            if ls:
                start, last = ls[-1]
                known = set().union(*[s for s0, s in ls[:-1] if start - 120 * DAY <= s0 < start]) if len(ls) > 1 else set()
                new = len(last - known) if known else 5
                roster = min(1.0, new / 2)
            else:
                roster = 1.0
            u = (thin + roster + isolation) / 3
            out[t] = TAU * u
            det.append((stage.teams[t], n90, round(roster, 2), round(isolation, 2), round(TAU * u, 3)))
        return out, det

    # --- probabilities
    def base(self, sid, w):
        """240 probabilities of the logit ensemble w·lgbm + (1 − w)·logreg for the Stage."""
        p = self.pairs[sid]
        return expit(w * logit(np.clip(p["lgbm ext"], 1e-9, 1 - 1e-9)) + (1 - w) * logit(np.clip(p["logreg ext"], 1e-9, 1 - 1e-9)))

    def shift(self, sid, w, kappa):
        """Between-stage update: per-team logit shifts from the previous Stage of the same Event."""
        delta = np.zeros(16)
        prev = self.previous(sid)
        if kappa == 0 or prev is None:
            return delta
        stage, ps = self.stages[sid], self.stages[prev]
        probs = pair_matrix(self.base(prev, w))
        pidx = {team_key(t): i for i, t in enumerate(ps.teams)}
        sidx = {team_key(t): i for i, t in enumerate(stage.teams)}
        for a, b, f, won in self.matches(prev):
            p = probs[f, a, b]
            for x, y in ((a, p), (b, 1 - p)):
                k = team_key(ps.teams[x])
                if k in sidx:
                    res = (won if x == a else not won) - y
                    delta[sidx[k]] += kappa * res
        return delta

    def probs(self, sid, w, kappa):
        p = self.base(sid, w)
        d = self.shift(sid, w, kappa)
        if not d.any():
            return p
        a, b = np.triu_indices(16, 1)
        z = logit(np.clip(p, 1e-9, 1 - 1e-9)).reshape(2, -1) + (d[a] - d[b])
        return expit(z).reshape(-1)

    def match_loss(self, sid, p):
        m = pair_matrix(p)
        ll = [-np.log(m[f, a, b] if won else 1 - m[f, a, b]) for a, b, f, won in self.matches(sid)]
        return float(np.sum(ll)), len(ll)


def build_stages(pairs):
    """The 28 Stage objects, built as `pickem_data.forecast_all` builds them (manifest, seed from the
    anchor's total), without fitting any model; the Anchor pairs come from the phase 4 cache."""
    from . import archive, paths
    from .pickem_data import FIXTURES, manifest
    from .table import is_qualifier

    stages = {}
    for directory in (paths.ARCHIVE, FIXTURES):
        data = archive.load(directory, with_hltv=False)
        for replay in data.stages:
            if (directory == FIXTURES) != (replay.id == 22378):
                continue
            if not any(m.round == 5 for m in replay.matches) and replay.id != 22378:
                continue
            if is_qualifier(replay.event) or (not replay.complete and replay.id != 22378):
                continue
            stage = manifest(replay, directory)
            if not stage.published:
                a, b = np.triu_indices(16, 1)
                m = np.zeros((16, 16))
                m[a, b] = np.asarray(pairs[stage.id]["Anchor"])[:120]
                m[b, a] = 1 - m[a, b]
                stage.seed = np.argsort(np.lexsort((stage.team_ids, -m.sum(axis=1).round(12))))
            stages[stage.id] = stage
    return stages


def pair_matrix(values):
    out = np.zeros((2, 16, 16))
    a, b = np.triu_indices(16, 1)
    out[:, a, b] = np.asarray(values).reshape(2, -1)
    out[:, b, a] = 1 - out[:, a, b]
    return out


# ------------------------------------------------------------------ evaluation engine

class Engine:
    """Memoised (Stage, weight, κ, σ_form, λ) → realised/expected/predicted, on disk."""

    def __init__(self, inputs: Inputs, runs=RUNS, log=print, cache_name="swiss-form-cache.pkl"):
        import torch  # noqa: F401
        from . import swiss_gpu

        self.i, self.runs, self.log = inputs, runs, log
        self.dev = swiss_gpu.device("auto")
        self.path = CACHE / cache_name
        self.cache = pickle.loads(self.path.read_bytes()) if self.path.exists() else {}
        self.sig = {sid: inputs.uncertainty(sid)[0] for sid in inputs.order}
        self.n_new = 0

    def save(self):
        self.path.write_bytes(pickle.dumps(self.cache))

    def run(self, sid, w, kappa, sf, lam):
        """`sf` None = independent Matches (no shocks)."""
        key = (sid, w, kappa, sf, lam, self.runs)
        if key not in self.cache:
            from . import swiss_gpu

            stage = self.i.stages[sid]
            values = self.i.probs(sid, w, kappa)
            sd = None if sf is None else np.sqrt(sf ** 2 + self.sig[sid] ** 2)
            t0 = time.perf_counter()
            outcomes = swiss_gpu.simulate_many(stage, values, self.runs, self.dev, 1_000_000, 33, sd)
            t1 = time.perf_counter()
            res = strategies(stage, outcomes, LAMBDAS)
            for l, v in res.items():
                self.cache[(sid, w, kappa, sf, l, self.runs)] = v
            self.n_new += 1
            self.log(f"  stage {sid} w={w} k={kappa} sf={sf}: sim {t1 - t0:.1f}s, search {time.perf_counter() - t1:.1f}s")
        return self.cache[key]

    # --- walk-forward choices
    def choose_weight(self, sid):
        prev = self.i.earlier(sid)
        if not prev:
            return 0.5
        loss = []
        for w in WEIGHTS:
            tot = n = 0
            for j in prev:
                s, c = self.i.match_loss(j, self.i.base(j, w))
                tot, n = tot + s, n + c
            loss.append(tot / n)
        return WEIGHTS[int(np.argmin(loss))]

    def choose_kappa(self, sid, w):
        """κ by the Match log-loss of the earlier Stages that have a previous Stage at their Event."""
        elig = [j for j in self.i.earlier(sid) if self.i.previous(j) is not None]
        if not elig:
            return 0.0
        loss = []
        for k in KAPPAS:
            tot = n = 0
            for j in elig:
                s, c = self.i.match_loss(j, self.i.probs(j, w, k))
                tot, n = tot + s, n + c
            loss.append(tot / n)
        return KAPPAS[int(np.argmin(loss))]

    def choose_sigma(self, sid, source):
        """σ_form by the Brier of ≥5 over the earlier Stages (`source(j)` = (w, κ) used at Stage j)."""
        prev = self.i.earlier(sid)
        if len(prev) < SIGMA_MIN_STAGES:
            return SIGMA_DEFAULT
        br = []
        for sf in SIGMA_GRID:
            b = []
            for j in prev:
                w, k = source(j)
                _, correct, _, p5 = self.run(j, w, k, sf, 0.0)
                b.append((p5 - (correct >= 5)) ** 2)
            br.append(np.mean(b))
        return SIGMA_GRID[int(np.argmin(br))]

    def choose_lambda(self, sid, config_at):
        """λ by the earlier Stages' realised share of ≥5, then mean correct, then smaller λ."""
        prev = self.i.earlier(sid)
        if not prev:
            return 0.0
        best = None
        for lam in LAMBDAS:
            hit = cor = 0
            for j in prev:
                w, k, sf = config_at(j)
                _, c, _, _ = self.run(j, w, k, sf, lam)
                hit += c >= 5
                cor += c
            score = (hit, cor)
            if best is None or score > best[0]:
                best = (score, lam)
        return best[1]

    # --- variants
    def variant(self, weight="fixed:0.5", sigma="wf", kappa="off", lam="off"):
        """Rows per Stage for a variant. weight: 'fixed:x' | 'wf'; sigma: None | 'wf' | 'fixed:x';
        kappa: 'off' | 'wf'; lam: 'off' | 'wf'."""
        def w_at(sid):
            return float(weight.split(":")[1]) if weight.startswith("fixed") else self.choose_weight(sid)

        def wk_at(sid):
            w = w_at(sid)
            return w, (0.0 if kappa == "off" else self.choose_kappa(sid, w))

        def sf_at(sid):
            if sigma is None:
                return None
            if sigma.startswith("fixed"):
                return float(sigma.split(":")[1])
            return self.choose_sigma(sid, wk_at)

        def cfg(sid):
            return (*wk_at(sid), sf_at(sid))

        out = {}
        for sid in self.i.order:
            w, k, sf = cfg(sid)
            lam_v = 0.0 if lam == "off" else self.choose_lambda(sid, cfg)
            roles, correct, expected, p5 = self.run(sid, w, k, sf, lam_v)
            out[sid] = {"correct": correct, "expected": expected, "p5": p5, "w": w, "kappa": k, "sf": sf, "lam": lam_v}
        return out


# ------------------------------------------------------------------ summaries

def reference(inputs: Inputs, model, strategy="B"):
    return {r["stage"]: r for r in inputs.rows if r["model"] == model and r["strategy"] == strategy}


def summarise(inputs, rows, base_anchor, base_logreg, stages=None):
    ids = [s for s in inputs.order if stages is None or s in stages]
    hit = np.array([rows[s]["correct"] >= 5 for s in ids], float)
    p5 = np.array([rows[s]["p5"] for s in ids])
    cor = np.array([rows[s]["correct"] for s in ids], float)
    exp = np.array([rows[s]["expected"] for s in ids])
    brier = (p5 - hit) ** 2

    def ref(b):
        h = np.array([b[s]["correct"] >= 5 for s in ids], float)
        return h, (np.array([b[s]["p5"] for s in ids]) - h) ** 2

    ha, ba = ref(base_anchor)
    hl, bl = ref(base_logreg)
    boot = lambda d: stats.event_bootstrap(ids, d, 2000, 0.9, 33)
    return {"n": len(ids), "correct": cor.mean(), "hits": int(hit.sum()), "p5": p5.mean(), "real_minus_pred": boot(hit - p5),
            "brier": brier.mean(), "d_brier_logreg": boot(brier - bl), "d_brier_anchor": boot(brier - ba),
            "d_hits_anchor": boot(hit - ha), "d_hits_logreg": boot(hit - hl), "exp_minus_real": (exp - cor).mean()}
