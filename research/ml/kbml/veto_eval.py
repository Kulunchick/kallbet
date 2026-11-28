"""W6: how well do the veto simulators predict the actual map vetoes (VETO.md)?

One chronological pass over every Match with a full 7-step HLTV veto (in-scope and extended
archive). At each Match's cutoff only vetoes whose Match finished strictly before it are released
(`known_finish < cutoff`), so a Match's own veto is never seen. For each Match the pass stores, per
method, the predicted distribution over the ten map names at every veto step:

- next-step mode (`D`): given the acting team and the realised earlier steps; the candidates are the
  active pool minus the maps already removed or picked;
- pre-veto mode (`M`): the earlier steps are unknown; the 'sim' methods integrate exactly over the
  5,040 ordered choices (torch), the baselines read their step frequencies directly.

Methods: `uniform`, `global` (G walk-forward), `freq` (the team's own choice at the step type over its
last N vetoes, (N, α) walk-forward), `eb` (VetoEB's choice weights), `lab` (the Rust lab's veto-sim
weights), and the exploratory `shrunk` (team counts shrunk to the global step distribution).
`pool` is the detected pool (`ActivePool`) or, for the sensitivity runs, the actual seven maps.
"""
from __future__ import annotations

import pickle
from collections import defaultdict, deque
from pathlib import Path

import numpy as np

from . import stats
from .features_signals import ActivePool, DAY

W5_CACHE = Path("D:/Projects/KallBet/target/ml-cache-w5/research-archive-ext")
OUT = Path("D:/Projects/KallBet/target/ml-cache-w5/veto")

MAPS = ["Ancient", "Anubis", "Cache", "Dust2", "Inferno", "Mirage", "Nuke", "Overpass", "Train", "Vertigo"]
MI = {m: i for i, m in enumerate(MAPS)}
V = len(MAPS)
FMT = {"bo1": 0, "bo3": 1, "bo5": 2}
FMT_NAMES = ["bo1", "bo3", "bo5"]
PATTERN = {"bo1": "bbbbbb", "bo3": "bbppbb", "bo5": "bbpppp"}
FLOOR = 1e-3  # probability given to a choice outside the candidate set, under every method

#: baselines' grids (declared in VETO.md)
FREQ_GRID = [(n, a) for n in (3, 5, 8, 12, 20, 40) for a in (0.25, 1.0)]
GLOB_GRID = [100, 300, 1000]
GLOB_ALPHA = 0.5
SHR_GRID = [(n, k) for n in (8, 20) for k in (1.0, 3.0, 8.0)]
#: exploratory baseline: the team's removals (picks) over its last N vetoes of any format, EB's prior
FREQX_GRID = [(n, 0.3) for n in (5, 8, 12, 25, 40)]
EB_WINDOW, EB_PRIOR = 25, 0.3
CHANGE_WINDOW_DAYS = 28


# ------------------------------------------------------------------ parsing

def team_key_of(step_team: str, names: dict, keys: list) -> int | None:
    from .teams import team_key
    k = team_key(step_team or "")
    if k in keys:
        return keys.index(k)
    return names.get(k)


def parse_record(rec: dict) -> dict | None:
    """A full veto of a known format with the six acting teams resolved, else None."""
    from .teams import team_key
    fmt = rec.get("format")
    v = rec.get("veto") or []
    if fmt not in PATTERN or len(v) != 7:
        return None
    keys = list(rec["keys"])
    names = {}
    for side, team in enumerate((rec.get("overall") or [])[:2]):
        names[team_key(team.get("team_name") or "")] = side
    sides, kinds, maps = [], [], []
    for step, want in zip(v[:6], PATTERN[fmt]):
        act = {"removed": "b", "picked": "p"}.get(step.get("action"))
        side = team_key_of(step.get("team"), names, keys)
        if act != want or side is None or step.get("map_name") not in MI:
            return None
        sides.append(side)
        kinds.append(0 if act == "b" else 1)
        maps.append(MI[step["map_name"]])
    last = v[6]
    if last.get("action") != "left_over" or last.get("map_name") not in MI:
        return None
    maps.append(MI[last["map_name"]])
    if len(set(maps)) != 7 or len(set(sides)) != 2 or sides.count(0) != 3:
        return None
    ordn, seen = [], defaultdict(int)
    for s, kd in zip(sides, kinds):
        ordn.append(seen[(s, kd)])
        seen[(s, kd)] += 1
    return {"fmt": FMT[fmt], "keys": keys, "sides": sides, "kinds": kinds, "ordn": ordn, "maps": maps,
            "names": [m for m in (x["map_name"] for x in v)], "started_at": int(rec["started_at"])}


def load_records(cache: Path = W5_CACHE, log=print):
    """Parsed vetoes joined to the Match table, sorted by (cutoff, match_id)."""
    import pandas as pd
    s = pickle.loads((cache / "hltv_build.pkl").read_bytes())
    m, recs = s["matches"], s["records"]
    mm = m.set_index("match_id")
    rows, bad = [], 0
    for mid, kf, rec in zip(recs.match_id, recs.known_finish, recs.rec):
        p = parse_record(rec)
        if p is None:
            bad += 1
            continue
        r = mm.loc[mid]
        p.update(match_id=int(mid), known_finish=None if pd.isna(kf) else int(kf), cutoff=int(r.cutoff),
                 event_id=int(r.event_id), tier=r.tier, block=r.block, in_scope=bool(r.in_scope),
                 scored=bool(r.in_scope and not pd.isna(r.anchor_p)),
                 ev_begins=int(r.event_begins), ev_ended=None if pd.isna(r.event_ended) else int(r.event_ended))
        rows.append(p)
    rows.sort(key=lambda p: (p["cutoff"], p["match_id"]))
    log(f"{len(rows)} parsed vetoes ({bad} unparsed or incomplete), {sum(p['scored'] for p in rows)} scored")
    return rows


# ------------------------------------------------------------------ state of released vetoes

def onehot(idx, n=V):
    v = np.zeros(n)
    v[list(idx)] = 1.0
    return v


class History:
    def __init__(self):
        self.pool = ActivePool()
        self.pool_now = 0
        self.pool_first = True
        self.recent = defaultdict(lambda: deque(maxlen=EB_WINDOW))  # team -> (removed vec, picked vec)
        self.sum_rem = defaultdict(lambda: np.zeros(V))
        self.sum_pick = defaultdict(lambda: np.zeros(V))
        self.n_vetoes = defaultdict(int)
        self.n_fmt = defaultdict(int)       # (team, format) -> vetoes
        self.hist_vec = defaultdict(lambda: ([], []))  # team -> (removed vecs, picked vecs), every veto
        self.seq = defaultdict(list)       # (team, fmt, kind, ordn) -> maps chosen
        self.team_left = defaultdict(list)  # (team, fmt) -> leftover maps
        self.glob = defaultdict(list)      # (fmt, step) -> maps
        self.glob_cnt = defaultdict(lambda: np.zeros(V))
        self.glob_left = defaultdict(list)
        self.glob_left_cnt = defaultdict(lambda: np.zeros(V))
        self.lab_ban = defaultdict(lambda: np.zeros(V))
        self.lab_pick = defaultdict(lambda: np.zeros(V))
        self.lab_ban1 = defaultdict(lambda: np.zeros(V))
        self.latest = {}
        self.latest_any = None

    def release(self, p):
        self.pool.add(p["started_at"], p["names"])
        fmt = p["fmt"]
        for side in (0, 1):
            team = p["keys"][side]
            rem = onehot([mp for s, k, mp in zip(p["sides"], p["kinds"], p["maps"]) if s == side and k == 0])
            pick = onehot([mp for s, k, mp in zip(p["sides"], p["kinds"], p["maps"]) if s == side and k == 1])
            d = self.recent[team]
            if len(d) == d.maxlen:
                old_r, old_p = d[0]
                self.sum_rem[team] -= old_r
                self.sum_pick[team] -= old_p
            d.append((rem, pick))
            self.hist_vec[team][0].append(rem)
            self.hist_vec[team][1].append(pick)
            self.sum_rem[team] += rem
            self.sum_pick[team] += pick
            self.n_vetoes[team] += 1
            self.n_fmt[(team, fmt)] += 1
            self.lab_ban[team] += rem
            self.lab_pick[team] += pick
            if fmt == 0:
                self.lab_ban1[team] += rem
            self.team_left[(team, fmt)].append(p["maps"][6])
        for k, (s, kd, o, mp) in enumerate(zip(p["sides"], p["kinds"], p["ordn"], p["maps"][:6])):
            self.seq[(p["keys"][s], fmt, kd, o)].append(mp)
            self.glob[(fmt, k)].append(mp)
            self.glob_cnt[(fmt, k)][mp] += 1
        self.glob_left[fmt].append(p["maps"][6])
        self.glob_left_cnt[fmt][p["maps"][6]] += 1
        self.latest[fmt] = p["maps"]
        self.latest_any = p["maps"]

    def refresh_pool(self, c):
        if self.pool_first or c - self.pool_now > 7 * DAY:
            self.pool.refresh(c)
            self.pool_now = c
            self.pool_first = False

    def kind_counts(self, team, kd, n):
        v = self.hist_vec[team][kd][-n:]
        return np.sum(v, axis=0) if v else np.zeros(V)

    def counts(self, seq, n):
        a = seq[-n:]
        return np.bincount(a, minlength=V).astype(float) if a else np.zeros(V)


def norm(w, cand):
    w = np.where(cand, w, 0.0)
    s = w.sum()
    return w / s if s > 0 else cand / max(cand.sum(), 1)


# ------------------------------------------------------------------ the pass

def run_pass(R: list[dict], pool_mode: str = "detected", log=print) -> dict:
    n = len(R)
    CF, CG, CS, CX = len(FREQ_GRID), len(GLOB_GRID), len(SHR_GRID), len(FREQX_GRID)
    z = lambda *s: np.zeros(s, dtype=np.float32)
    out = {
        "valid": np.zeros(n, bool), "cand": np.zeros((n, 6, V), bool), "tie": z(n, 6, V),
        "D_uniform": z(n, 6, V), "D_eb": z(n, 6, V), "D_lab": z(n, 6, V),
        "D_freq": z(n, CF, 6, V), "D_global": z(n, CG, 6, V), "D_shrunk": z(n, CS, 6, V), "D_freqx": z(n, CX, 6, V),
        "M_uniform": z(n, 7, V), "M_freq": z(n, CF, 7, V), "M_global": z(n, CG, 7, V),
        "mcand": np.zeros((n, V), bool), "mtie": z(n, 7, V),
        "W_eb": z(n, 2, 2, V), "W_lab": z(n, 2, 2, V), "W_freq": z(n, CF, 2, 2, V),
        "pool_eb": np.zeros((n, 7), np.int64), "pool_lab": np.zeros((n, 7), np.int64),
        "nprior": np.zeros((n, 2), np.int64), "days_since_change": np.full(n, 1e9), "pool_ok": np.zeros(n, bool),
        "lab_pool_ok": np.zeros(n, bool), "nfmt": np.zeros((n, 2), np.int64),
    }
    order_rel = sorted((i for i, p in enumerate(R) if p["known_finish"] is not None),
                       key=lambda i: (R[i]["known_finish"], R[i]["match_id"]))
    H = History()
    ptr = 0
    for i, p in enumerate(R):  # R is sorted by cutoff
        c = p["cutoff"]
        while ptr < len(order_rel) and R[order_rel[ptr]]["known_finish"] < c:
            H.release(R[order_rel[ptr]])
            ptr += 1
        if p["known_finish"] is not None and p["known_finish"] < c:
            raise RuntimeError("a Match finished before its own cutoff")
        H.refresh_pool(c)
        actual = set(p["maps"])
        if pool_mode == "oracle":
            pool_idx = sorted(actual)
        else:
            pool_idx = sorted(MI[x] for x in H.pool.pool)
        if len(pool_idx) != 7:
            continue
        out["valid"][i] = True
        pool_vec = onehot(pool_idx).astype(bool)
        out["mcand"][i] = pool_vec
        out["pool_eb"][i] = pool_idx
        out["pool_ok"][i] = set(pool_idx) == actual
        if H.pool.changes:
            out["days_since_change"][i] = (c - H.pool.changes[-1][0]) / DAY
        fmt = p["fmt"]
        if pool_mode == "oracle":
            lab_pool = actual
        else:
            lab_pool = set((H.latest_any if fmt == 2 else H.latest.get(fmt)) or [])
        lab_pool_idx = sorted(lab_pool) if len(lab_pool) == 7 else pool_idx
        out["pool_lab"][i] = lab_pool_idx
        out["lab_pool_ok"][i] = set(lab_pool_idx) == actual
        lab_vec = onehot(lab_pool_idx).astype(bool)
        teams = p["keys"]
        out["nprior"][i] = [H.n_vetoes[teams[0]], H.n_vetoes[teams[1]]]
        out["nfmt"][i] = [H.n_fmt[(teams[0], fmt)], H.n_fmt[(teams[1], fmt)]]
        used = np.zeros(V, bool)
        for k in range(6):
            side, kd, o = p["sides"][k], p["kinds"][k], p["ordn"][k]
            team = teams[side]
            cand = pool_vec & ~used
            out["cand"][i, k] = cand
            out["tie"][i, k] = H.glob_cnt[(fmt, k)]
            out["D_uniform"][i, k] = norm(np.ones(V), cand)
            w_eb = (H.sum_pick if kd else H.sum_rem)[team] + EB_PRIOR
            out["D_eb"][i, k] = norm(w_eb, cand)
            lab_cand = lab_vec & ~used
            if fmt == 0:
                w_lab = 1.0 + H.lab_ban1[team]
            else:
                w_lab = 1.0 + (H.lab_pick if kd else H.lab_ban)[team]
            out["D_lab"][i, k] = norm(w_lab, lab_cand)
            seq = H.seq[(team, fmt, kd, o)]
            glob_seq = H.glob[(fmt, k)]
            gdist = None
            for j, (nn, a) in enumerate(FREQ_GRID):
                out["D_freq"][i, j, k] = norm(H.counts(seq, nn) + a, cand)
            for j, g in enumerate(GLOB_GRID):
                out["D_global"][i, j, k] = norm(H.counts(glob_seq, g) + GLOB_ALPHA, cand)
            for j, (nn, a) in enumerate(FREQX_GRID):
                out["D_freqx"][i, j, k] = norm(H.kind_counts(team, kd, nn) + a, cand)
            gdist = norm(H.counts(glob_seq, 300) + GLOB_ALPHA, cand)
            for j, (nn, kappa) in enumerate(SHR_GRID):
                out["D_shrunk"][i, j, k] = norm(H.counts(seq, nn) + kappa * gdist, cand)
            # pre-veto (no removal): the pool, whatever was banned before
            out["mtie"][i, k] = H.glob_cnt[(fmt, k)]
            for j, (nn, a) in enumerate(FREQ_GRID):
                out["M_freq"][i, j, k] = norm(H.counts(seq, nn) + a, pool_vec)
            for j, g in enumerate(GLOB_GRID):
                out["M_global"][i, j, k] = norm(H.counts(glob_seq, g) + GLOB_ALPHA, pool_vec)
            out["M_uniform"][i, k] = norm(np.ones(V), pool_vec)
            used[p["maps"][k]] = True
        # leftover / decider (pre-veto only)
        out["mtie"][i, 6] = H.glob_left_cnt[fmt]
        out["M_uniform"][i, 6] = norm(np.ones(V), pool_vec)
        for j, g in enumerate(GLOB_GRID):
            out["M_global"][i, j, 6] = norm(H.counts(H.glob_left[fmt], g) + GLOB_ALPHA, pool_vec)
        for j, (nn, a) in enumerate(FREQ_GRID):
            cl = sum(H.counts(H.team_left[(t, fmt)], nn) + a for t in teams) / 2
            out["M_freq"][i, j, 6] = norm(cl, pool_vec)
        # integrator weights (pool-aligned later): [side, kind]
        for side in (0, 1):
            t = teams[side]
            out["W_eb"][i, side, 0] = H.sum_rem[t] + EB_PRIOR
            out["W_eb"][i, side, 1] = H.sum_pick[t] + EB_PRIOR
            if fmt == 0:
                out["W_lab"][i, side, 0] = 1.0 + H.lab_ban1[t]
            else:
                out["W_lab"][i, side, 0] = 1.0 + H.lab_ban[t]
            out["W_lab"][i, side, 1] = 1.0 + H.lab_pick[t]
            for j, (nn, a) in enumerate(FREQ_GRID):
                for kd in (0, 1):
                    cnt = sum(H.counts(H.seq[(t, fmt, kd, o)], nn) for o in range(3))
                    out["W_freq"][i, j, side, kd] = cnt + a
        if i % 2000 == 0:
            log(f"  pass {i}/{n} ({pool_mode})")
    out["changes"] = [(t, a, d) for t, a, d in H.pool.changes]
    return out


# ------------------------------------------------------------------ exact integration (torch)

_P = None


def perms():
    global _P
    if _P is None:
        import itertools
        _P = np.array(list(itertools.permutations(range(7), 6)), dtype=np.int64)
    return _P


def _set_tables():
    P = perms()
    left = 21 - P.sum(1)
    masks = (1 << P[:, 2]) + (1 << P[:, 3]) + (1 << left)
    uniq = sorted(set(masks.tolist()))
    sid = np.array([uniq.index(m) for m in masks.tolist()])
    tri = P[:, 2] * 49 + P[:, 3] * 7 + left
    return left, sid, len(uniq), tri


def integrate(Wp: np.ndarray, sides: np.ndarray, kinds: np.ndarray, device: str = "cpu", chunk: int = 256) -> dict:
    """Exact veto integration. `Wp`: (n, 2, 2, 7) weights [team side, kind: ban/pick] over the seven
    pool slots; `sides`, `kinds`: (n, 6) per step. Each step draws among the remaining slots in
    proportion to the acting side's weights. Returns `steps` (n, 6, 7), `left` (n, 7) (the seventh map),
    `sets` (n, 35) (maps of steps 3, 4 and the leftover, as a set; meaningful for Bo3), `triple`
    (n, 343) (ordered pick, pick, leftover) and `total` (n,) (sums to 1)."""
    import torch
    P = perms()
    left, sid, nsets, tri = _set_tables()
    dev = torch.device(device)
    Pt = torch.as_tensor(P, device=dev)
    sid_t, tri_t = torch.as_tensor(sid, device=dev), torch.as_tensor(tri, device=dev)
    onehots = [torch.nn.functional.one_hot(Pt[:, k], 7).to(torch.float64) for k in range(6)]
    oneleft = torch.nn.functional.one_hot(torch.as_tensor(left, device=dev), 7).to(torch.float64)
    n = len(Wp)
    res = {"steps": np.zeros((n, 6, 7)), "left": np.zeros((n, 7)), "sets": np.zeros((n, nsets)),
           "triple": np.zeros((n, 343)), "total": np.zeros(n)}
    for a in range(0, n, chunk):
        b = min(n, a + chunk)
        W = torch.as_tensor(Wp[a:b], dtype=torch.float64, device=dev)
        s = torch.as_tensor(sides[a:b], device=dev)
        kd = torch.as_tensor(kinds[a:b], device=dev)
        ar = torch.arange(b - a, device=dev)
        prob = None
        for k in range(6):
            Wk = W[ar, s[:, k], kd[:, k]]  # (m, 7)
            chosen = Wk[:, Pt[:, k]]
            used = Wk[:, Pt[:, :k]].sum(-1) if k else 0.0
            denom = Wk.sum(-1, keepdim=True) - used
            f = chosen / denom
            prob = f if prob is None else prob * f
        res["total"][a:b] = prob.sum(-1).cpu().numpy()
        for k in range(6):
            res["steps"][a:b, k] = (prob @ onehots[k]).cpu().numpy()
        res["left"][a:b] = (prob @ oneleft).cpu().numpy()
        sets = torch.zeros((b - a, nsets), dtype=torch.float64, device=dev)
        sets.index_add_(1, sid_t, prob)
        res["sets"][a:b] = sets.cpu().numpy()
        trip = torch.zeros((b - a, 343), dtype=torch.float64, device=dev)
        trip.index_add_(1, tri_t, prob)
        res["triple"][a:b] = trip.cpu().numpy()
        del prob, f, chosen, sets, trip
    if device != "cpu":
        torch.cuda.empty_cache()
    return res


def set_masks():
    left, sid, nsets, _ = _set_tables()
    P = perms()
    masks = (1 << P[:, 2]) + (1 << P[:, 3]) + (1 << left)
    out = np.zeros(nsets, int)
    out[sid] = masks
    return out


def _ranks(p: np.ndarray, k: int):
    """(n_greater, n_equal_others) of entry k of p."""
    return int((p > p[k] + 1e-15).sum()), int((np.abs(p - p[k]) <= 1e-15).sum()) - 1


def set_metrics(r: dict, pool: np.ndarray, A: np.ndarray, rows: np.ndarray) -> dict:
    """Bo3 set and order metrics of an integrated forecast. `r`: `integrate` output for `rows` (indices
    into A), `pool` (n_ok, 7). Actual set = the two picks and the decider. Ties (the uniform forecast)
    count in expectation: hit_k = P(actual in a random top-k among equals). A choice off the pool: FLOOR
    and a miss; the overlaps count only the played maps in the pool."""
    n = len(rows)
    masks = set_masks()
    out = {k: np.zeros(n) for k in ("p_set", "hit1_set", "hit3_set", "ov_exp", "ov_top", "p_tri", "hit1_tri")}
    for j, i in enumerate(rows):
        played = A[i, [2, 3, 6]]
        slots = [pool_slots(pool[j], [x]) for x in played]
        in_slots = [s[0] for s in slots if s]
        inc = r["steps"][j, 2] + r["steps"][j, 3] + r["left"][j]
        out["ov_exp"][j] = inc[in_slots].sum() if in_slots else 0.0
        sets = r["sets"][j]
        top = np.where(sets >= sets.max() - 1e-15)[0]
        out["ov_top"][j] = np.mean([bin(int(masks[t]) & sum(1 << s for s in in_slots)).count("1") for t in top])
        if len(in_slots) < 3:
            out["p_set"][j] = out["p_tri"][j] = FLOOR
            continue
        want = sum(1 << s for s in in_slots)
        k = int(np.where(masks == want)[0][0])
        out["p_set"][j] = max(sets[k], FLOOR)
        g, e = _ranks(sets, k)
        out["hit1_set"][j] = float(np.clip((1 - g) / (e + 1), 0, 1))
        out["hit3_set"][j] = float(np.clip((3 - g) / (e + 1), 0, 1))
        t = slots[0][0] * 49 + slots[1][0] * 7 + slots[2][0]
        out["p_tri"][j] = max(r["triple"][j, t], FLOOR)
        g, e = _ranks(r["triple"][j], t)
        out["hit1_tri"][j] = float(np.clip((1 - g) / (e + 1), 0, 1))
    return out


def set_index(maps3_slots) -> int:
    """Index of an (unordered) set of three pool slots in `integrate`'s `sets` axis."""
    _, sid, _, _ = _set_tables()
    P = perms()
    left = 21 - P.sum(1)
    want = sum(1 << s for s in maps3_slots)
    masks = (1 << P[:, 2]) + (1 << P[:, 3]) + (1 << left)
    return int(sid[np.argmax(masks == want)])


def pool_slots(pool_idx: np.ndarray, maps) -> list[int] | None:
    """Slots (0..6) in a record's pool of some map indices, None if one is outside the pool."""
    lookup = {int(m): j for j, m in enumerate(pool_idx)}
    if any(int(x) not in lookup for x in maps):
        return None
    return [lookup[int(x)] for x in maps]


def to_pool_weights(W: np.ndarray, pool: np.ndarray) -> np.ndarray:
    """(n, 2, 2, V) weights over maps → (n, 2, 2, 7) over each record's pool slots."""
    n = len(W)
    out = np.empty((n, 2, 2, 7))
    for i in range(n):
        out[i] = W[i][..., pool[i]]
    return out


def pool_to_maps(M: np.ndarray, pool: np.ndarray) -> np.ndarray:
    """(n, ..., 7) distributions over pool slots → (n, ..., V) over the ten map names."""
    out = np.zeros(M.shape[:-1] + (V,))
    for i in range(len(M)):
        out[i][..., pool[i]] = M[i]
    return out


# ------------------------------------------------------------------ scoring

def score(D: np.ndarray, cand: np.ndarray, tie: np.ndarray, actual: np.ndarray) -> dict:
    """D, cand, tie: (..., V); actual: (...) map index. A choice outside `cand` gets FLOOR and a miss."""
    D = D.astype(float)
    a = actual[..., None]
    pa_raw = np.take_along_axis(D, a, -1)[..., 0]
    in_c = np.take_along_axis(cand, a, -1)[..., 0]
    pa = np.where(in_c, np.maximum(pa_raw, FLOOR), FLOOR)
    ta = np.take_along_axis(tie, a, -1)
    idx = np.arange(V)
    better = cand & ((D > pa_raw[..., None]) | ((D == pa_raw[..., None]) & ((tie > ta) | ((tie == ta) & (idx < a)))))
    rank = np.where(in_c, better.sum(-1), 99)
    onehot_a = (idx == a).astype(float)
    bs = ((D - onehot_a) ** 2).sum(-1) + np.where(in_c, 0.0, 1.0)
    return {"p": pa, "rank": rank, "hit1": (rank < 1).astype(float), "hit2": (rank < 2).astype(float),
            "hit3": (rank < 3).astype(float), "ll": -np.log(pa), "bs": bs}


def actual_arrays(R):
    A = np.array([p["maps"] for p in R])
    return A  # (n, 7)


def select_walk_forward(ll: np.ndarray, R, fmts: np.ndarray, valid: np.ndarray) -> np.ndarray:
    """Per record, the config index with the lowest summed log-loss over records of the same format
    whose Event ended strictly before this record's Event began (all tiers). `ll`: (n, C)."""
    n, C = ll.shape
    ev_begins = np.array([p["ev_begins"] for p in R])
    ev_ended = np.array([p["ev_ended"] if p["ev_ended"] is not None else 1 << 62 for p in R])
    ev_id = np.array([p["event_id"] for p in R])
    chosen = np.zeros(n, int)
    cache = {}
    for e in np.unique(ev_id):
        rows = np.where(ev_id == e)[0]
        b = ev_begins[rows[0]]
        for f in np.unique(fmts[rows]):
            prior = (fmts == f) & (ev_ended < b) & (ev_id != e) & valid
            chosen[rows[fmts[rows] == f]] = int(np.argmin(ll[prior].sum(0))) if prior.sum() >= 30 else 0
    return chosen


def build_results(R: list[dict], out: dict, device="cpu", log=print) -> dict:
    """Walk-forward selection, scoring of every method in both modes, integrator-based sims."""
    n = len(R)
    fmts = np.array([p["fmt"] for p in R])
    A = actual_arrays(R)
    valid = out["valid"]
    cand, tie = out["cand"], out["tie"]
    res = {"valid": valid, "fmts": fmts}

    def sc_family(name, key, G):
        sc = [score(out[key][:, j], cand, tie, A[:, :6]) for j in range(G)]
        ll = np.stack([s["ll"].sum(1) for s in sc], 1)
        return sc, ll

    sel = {}
    for name, key, G in (("freq", "D_freq", len(FREQ_GRID)), ("global", "D_global", len(GLOB_GRID)),
                         ("shrunk", "D_shrunk", len(SHR_GRID)),
                         ("freqx", "D_freqx", len(FREQX_GRID))):
        sc, ll = sc_family(name, key, G)
        ll = np.where(valid[:, None], ll, 0.0)
        ch = select_walk_forward(ll, R, fmts, valid)
        sel[name] = ch
        res[f"cfg_{name}"] = ch
        res[f"n_{name}"] = G
        stack = {k: np.stack([s[k] for s in sc], 1) for k in sc[0]}  # (n, G, 6)
        res[f"next_{name}"] = {k: v[np.arange(n), ch] for k, v in stack.items()}
    for name, key in (("uniform", "D_uniform"), ("eb", "D_eb"), ("lab", "D_lab")):
        res[f"next_{name}"] = score(out[key], cand, tie, A[:, :6])
    # lab with its own pool: candidates were already restricted to its pool in the pass; the score
    # above uses the detected `cand` for the floor, so rescore with the lab candidate set
    labc = np.zeros_like(cand)
    for i in range(n):
        lp = out["pool_lab"][i]
        used = np.zeros(V, bool)
        for k in range(6):
            c = np.zeros(V, bool)
            c[lp] = True
            labc[i, k] = c & ~used
            used[A[i, k]] = True
    res["next_lab"] = score(out["D_lab"], labc, tie, A[:, :6])
    res["cand"], res["labc"], res["tie"], res["mtie"], res["mcand"] = cand, labc, tie, out["mtie"], out["mcand"]
    for name, key in (("uniform", "D_uniform"), ("eb", "D_eb"), ("lab", "D_lab")):
        res[f"dist_next_{name}"] = out[key]
    for name, key in (("freq", "D_freq"), ("global", "D_global"), ("shrunk", "D_shrunk"), ("freqx", "D_freqx")):
        res[f"dist_next_{name}"] = out[key][np.arange(n), sel[name]]
    # pre-veto, direct
    mc = np.broadcast_to(out["mcand"][:, None, :], (n, 7, V))
    for name, key, G in (("freq", "M_freq", len(FREQ_GRID)), ("global", "M_global", len(GLOB_GRID))):
        sc = [score(out[key][:, j], mc, out["mtie"], A) for j in range(G)]
        stack = {k: np.stack([s[k] for s in sc], 1) for k in sc[0]}
        res[f"pre_{name}"] = {k: v[np.arange(n), sel[name]] for k, v in stack.items()}
    res["pre_uniform"] = score(out["M_uniform"], mc, out["mtie"], A)
    # pre-veto, integrated: eb, lab, uniform, freq (as weights)
    sides = np.array([p["sides"] for p in R])
    kinds = np.array([p["kinds"] for p in R])
    ok = np.where(valid)[0]
    pools = {"eb": out["pool_eb"], "lab": out["pool_lab"], "uniform": out["pool_eb"], "freq_int": out["pool_eb"]}
    Ws = {"eb": out["W_eb"], "lab": out["W_lab"], "uniform": np.ones_like(out["W_eb"]),
          "freq_int": out["W_freq"][np.arange(n), sel["freq"]]}
    for name in ("eb", "lab", "uniform", "freq_int"):
        Wp = to_pool_weights(Ws[name][ok].astype(float), pools[name][ok])
        r = integrate(Wp, sides[ok], kinds[ok], device)
        assert np.allclose(r["total"], 1.0), name
        bo3 = ok[fmts[ok] == 1]
        pick = np.searchsorted(ok, bo3)
        res["sets"] = res.get("sets", {})
        res["sets"][name] = {"rows": bo3, **set_metrics({k: v[pick] for k, v in r.items()}, pools[name][ok][pick], A, bo3)}
        # step marginals as maps (n_ok, 7, V): steps 0..5 then leftover
        M = np.concatenate([r["steps"], r["left"][:, None]], 1)
        full = np.zeros((n, 7, V))
        full[ok] = pool_to_maps(M, pools[name][ok])
        cmask = np.zeros((n, 7, V), bool)
        for i in ok:
            cmask[i, :, pools[name][i]] = True
        res[f"pre_{name}"] = score(full, cmask, out["mtie"], A)
        res[f"dist_pre_{name}"] = full.astype(np.float32)
        log(f"  integrated {name}: {len(ok)} Matches [{device}]")
    return res


def save(path: Path, **objs):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(pickle.dumps(objs))


def main(argv=None):
    import argparse
    import time
    ap = argparse.ArgumentParser()
    ap.add_argument("--device", default="cuda")
    ap.add_argument("--pool", default="detected,oracle")
    args = ap.parse_args(argv)
    t0 = time.time()
    R = load_records()
    all_res = {}
    for mode in args.pool.split(","):
        out = run_pass(R, mode)
        print(f"pass {mode} [{time.time() - t0:.0f}s]; {len(out['changes'])} pool changes")
        res = build_results(R, out, args.device)
        res["changes"] = out["changes"]
        for k in ("pool_ok", "lab_pool_ok", "days_since_change", "nprior", "nfmt", "valid"):
            res[k] = out[k]
        all_res[mode] = res
        print(f"results {mode} [{time.time() - t0:.0f}s]")
    meta = {"records": [{k: p[k] for k in ("match_id", "event_id", "tier", "block", "in_scope", "scored", "fmt",
                                            "cutoff", "sides", "kinds", "ordn", "maps", "keys")} for p in R]}
    save(OUT / "veto_results.pkl", results=all_res, **meta)
    print(f"saved {OUT / 'veto_results.pkl'} [{time.time() - t0:.0f}s]")


if __name__ == "__main__":
    main()
