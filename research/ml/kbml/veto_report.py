"""Tables of VETO.md from `veto_results.pkl` (written by `kbml.veto_eval`). Everything after the marker
`<!-- tables -->` of VETO.md is regenerated; the declaration and the conclusion above it are hand-written.

    uv run python -m kbml.veto_report
"""
from __future__ import annotations

import pickle
from pathlib import Path

import numpy as np

from . import stats
from .veto_eval import MAPS, OUT, V, FREQ_GRID, GLOB_GRID, SHR_GRID

MARK = "<!-- tables -->"
DOC = Path(__file__).resolve().parents[1] / "VETO.md"
METHODS = ["uniform", "global", "freq", "eb", "lab"]
LABEL = {"uniform": "uniform", "global": "global", "freq": "team freq", "eb": "EB sim", "lab": "lab sim",
         "shrunk": "shrunk (expl.)", "freqx": "freq any format (expl.)", "freq_int": "team freq, integrated"}
STEP_NAMES = ["ban 1", "ban 2", "pick 1", "pick 2", "ban 3", "ban 4"]


class Data:
    def __init__(self, path=OUT / "veto_results.pkl"):
        d = pickle.loads(Path(path).read_bytes())
        self.R, self.all = d["records"], d["results"]
        R = self.R
        self.ev = np.array([r["event_id"] for r in R])
        self.scored = np.array([r["scored"] for r in R])
        self.fmt = np.array([r["fmt"] for r in R])
        self.tier = np.array([r["tier"] for r in R])
        self.block = np.array([r["block"] for r in R])
        self.sides = np.array([r["sides"] for r in R])
        self.cut = np.array([r["cutoff"] for r in R])
        self.A = np.array([r["maps"] for r in R])


def ci(e, v):
    return stats.event_bootstrap(np.asarray(e), np.asarray(v, float))


def pct(b, d=1):
    return f"{b['mean'] * 100:.{d}f} [{b['low'] * 100:.{d}f}, {b['high'] * 100:.{d}f}]"


def num(b, d=3):
    return f"{b['mean']:.{d}f} [{b['low']:.{d}f}, {b['high']:.{d}f}]"


def sgn(b, scale=1.0, d=3, unit=""):
    f = lambda x: f"{x * scale:+.{d}f}"
    return f"{f(b['mean'])}{unit} [{f(b['low'])}, {f(b['high'])}]"


class View:
    """Per-step values of a mode (`detected` or `oracle`) for a Match mask and step list."""

    def __init__(self, D: Data, mode="detected"):
        self.D, self.r = D, D.all[mode]

    def mask(self, fmt=None, scored=True, extra=None):
        m = self.r["valid"].copy()
        if scored:
            m &= self.D.scored
        if fmt is not None:
            m &= self.D.fmt == fmt
        if extra is not None:
            m &= extra
        return m

    def vals(self, method, key, steps, mask, kind="next"):
        v = self.r[f"{kind}_{method}"][key][mask][:, steps]
        e = np.repeat(self.D.ev[mask][:, None], len(steps), 1)
        return v.ravel(), e.ravel()

    def side_info(self, steps, mask, arr):
        """`arr` (n, 2) per team → per step of the acting team, (m, len(steps)) flattened."""
        s = self.D.sides[mask][:, steps]
        a = arr[mask]
        return np.take_along_axis(a, s, 1).ravel()


def row(view, name, key, steps, mask, scale="pct", kind="next"):
    v, e = view.vals(name, key, steps, mask, kind)
    return ci(e, v)


# ------------------------------------------------------------------ tables

def primary(view, md):
    m = view.mask(fmt=1)
    a, e = view.vals("eb", "hit1", [2, 3], m)
    b, _ = view.vals("freq", "hit1", [2, 3], m)
    bd = ci(e, a - b)
    nm = int(m.sum())
    md += ["## Primary result", "",
           f"Bo3 team picks (veto steps 3 and 4), next-step mode, detected pool; {nm} scored Bo3 Matches "
           f"with a full veto ({len(np.unique(view.D.ev[m]))} Events), {2 * nm} pick steps. Top-1 hit rate, 90% Event-bootstrap "
           "interval (2000 resamples, seed 33).", "",
           "| Method | Top-1 hit rate (%) |", "|---|---|"]
    for name in ("uniform", "global", "freq", "eb"):
        md.append(f"| {LABEL[name]} | {pct(row(view, name, 'hit1', [2, 3], m))} |")
    verdict = "interval wholly above 0: the EB simulator beats the frequency rule" if bd["low"] > 0 else \
        "interval not wholly above 0: **not shown**" + (" (wholly below 0: the frequency rule is better)" if bd["high"] < 0 else "")
    md += ["", f"**EB − team freq: {sgn(bd, 100, 1, ' pp')}**; rule (lower bound > 0): {verdict}.", ""]
    return bd


STEP_GROUPS = {"first bans (steps 1-2)": [0, 1], "team picks (steps 3-4)": [2, 3], "second bans (steps 5-6)": [4, 5]}


def per_step_tables(view, md, fmt, title, steps_n=6, methods=METHODS + ["freqx", "shrunk"]):
    m = view.mask(fmt=fmt)
    if m.sum() == 0:
        return
    nm = int(m.sum())
    names = {0: ["ban 1", "ban 2", "ban 3", "ban 4", "ban 5", "ban 6"], 1: STEP_NAMES, 2: STEP_NAMES}[fmt]
    groups = {0: {"opening bans (steps 1-2)": [0, 1], "middle bans (steps 3-4)": [2, 3], "closing bans (steps 5-6)": [4, 5]},
              1: STEP_GROUPS, 2: {"bans (steps 1-2)": [0, 1], "team picks (steps 3-6)": [2, 3, 4, 5]}}[fmt]
    for key, what, f in (("hit1", "Top-1 hit rate (%)", pct), ("ll", "Log-loss (nats)", num), ("bs", "Brier (multiclass)", num)):
        md += [f"**{title}: {what}, next-step mode** ({nm} Matches)", "",
               "| Step | " + " | ".join(LABEL[x] for x in methods) + " |", "|---|" + "---|" * len(methods)]
        for g, st in list(groups.items()) + [(f"{n}", [i]) for i, n in enumerate(names)]:
            md.append(f"| {g} | " + " | ".join(f(row(view, x, key, st, m)) for x in methods) + " |")
        md.append("")
    md += [f"**{title}: difference to the team-frequency rule, next-step mode** (paired per step; negative log-loss/Brier = better)", "",
           "| Step group | Δ top-1 (pp), EB | Δ top-1 (pp), lab | Δ top-1 (pp), freq any format (expl.) | Δ log-loss, EB | Δ log-loss, lab | Δ log-loss, freq any format (expl.) | Δ log-loss, shrunk (expl.) |", "|---|---|---|---|---|---|---|---|"]
    for g, st in groups.items():
        cells = []
        for x, key, sc, d in (("eb", "hit1", 100, 1), ("lab", "hit1", 100, 1), ("freqx", "hit1", 100, 1), ("eb", "ll", 1, 3), ("lab", "ll", 1, 3), ("freqx", "ll", 1, 3), ("shrunk", "ll", 1, 3)):
            a, e = view.vals(x, key, st, m)
            b, _ = view.vals("freq", key, st, m)
            cells.append(sgn(ci(e, a - b), sc, d))
        md.append(f"| {g} | " + " | ".join(cells) + " |")
    md.append("")


def pre_veto_table(view, md, fmt, title):
    m = view.mask(fmt=fmt)
    if m.sum() == 0:
        return
    methods = ["uniform", "global", "freq", "freq_int", "eb", "lab"]
    names = ["ban 1", "ban 2", "pick 1", "pick 2", "ban 3", "ban 4", "decider"] if fmt else ["ban 1", "ban 2", "ban 3", "ban 4", "ban 5", "ban 6", "leftover (map played)"]
    for key, what, f in (("hit1", "Top-1 hit rate (%)", pct), ("ll", "Log-loss (nats)", num)):
        md += [f"**{title}: {what}, pre-veto mode** ({int(m.sum())} Matches; the earlier steps are not known)", "",
               "| Step | " + " | ".join(LABEL[x] for x in methods) + " |", "|---|" + "---|" * len(methods)]
        for i, n in enumerate(names):
            md.append(f"| {n} | " + " | ".join(f(row(view, x, key, [i], m, kind="pre")) for x in methods) + " |")
        md.append("")


def set_table(view, md):
    r = view.r["sets"]
    bo3 = view.mask(fmt=1)
    md += ["**Bo3 set of the three maps played and the pick order** (pre-veto: the acting teams are known, the earlier steps not; "
           "exact integration over the 5,040 ordered choices, torch on the GPU). Actual set = the two picked maps and the decider; "
           "'order' = (map 1, map 2, map 3). Ties count in expectation (uniform: 1/35 for the set, 1/210 for the order). "
           "'Expected overlap' = Σ over the three played maps of P(that map is played), maximum 3; "
           "'top set overlap' = maps shared by the most likely set and the actual one.", "",
           "| Forecast | Matches | Set: top-1 hit (%) | Set: top-3 hit (%) | Set log-loss | Expected overlap (of 3) | Top-set overlap (of 3) | Order: top-1 hit (%) | Order log-loss |",
           "|---|---|---|---|---|---|---|---|---|"]
    for name in ("uniform", "freq_int", "eb", "lab"):
        s = r[name]
        rows = s["rows"]
        keep = np.isin(rows, np.where(bo3)[0])
        e = view.D.ev[rows][keep]
        g = lambda k: s[k][keep]
        cells = [pct(ci(e, g("hit1_set"))), pct(ci(e, g("hit3_set"))), num(ci(e, -np.log(g("p_set")))),
                 num(ci(e, g("ov_exp")), 2), num(ci(e, g("ov_top")), 2), pct(ci(e, g("hit1_tri"))),
                 num(ci(e, -np.log(g("p_tri"))))]
        md.append(f"| {LABEL[name] if name != 'uniform' else 'uniform (every set equal)'} | {int(keep.sum())} | " + " | ".join(cells) + " |")
    ref = r["freq_int"]
    for name in ("eb", "lab"):
        s_ = r[name]
        rows = s_["rows"]
        keep = np.isin(rows, np.where(bo3)[0])
        e = view.D.ev[rows][keep]
        d = lambda k, f=lambda x: x: f(s_[k][keep]) - f(ref[k][keep])
        cells = [sgn(ci(e, d("hit1_set")), 100, 1), sgn(ci(e, d("hit3_set")), 100, 1), sgn(ci(e, d("p_set", lambda x: -np.log(x)))),
                 sgn(ci(e, d("ov_exp")), 1, 3), sgn(ci(e, d("ov_top")), 1, 3), sgn(ci(e, d("hit1_tri")), 100, 1),
                 sgn(ci(e, d("p_tri", lambda x: -np.log(x))))]
        md.append(f"| **{LABEL[name]} − team freq, integrated** (pp for hit rates) | {int(keep.sum())} | " + " | ".join(cells) + " |")
    md.append("")


def reliability(view, md, fmt=None, steps=(0, 1, 2, 3, 4), methods=("uniform", "freq", "eb"), kind="next", title=""):
    """Candidate-level reliability: every (step, candidate map) with its probability and 0/1."""
    m = view.mask(fmt=fmt)
    ev = np.repeat(view.D.ev[m][:, None], len(steps), 1)
    rows = {}
    for x in methods:
        dist = view.r[f"dist_{kind}_{x}"][m][:, list(steps)].astype(float)  # (m, s, V)
        cand = (view.r["cand"] if kind == "next" else np.broadcast_to(view.r["mcand"][:, None, :], view.r["cand"].shape[:1] + (7, V)))
        cand = cand[m][:, list(steps)]
        if x == "lab" and kind == "next":
            cand = view.r["labc"][m][:, list(steps)]
        act = view.D.A[m][:, list(steps)]
        hit = (np.arange(V)[None, None, :] == act[..., None]).astype(float)
        p, y = dist[cand], hit[cand]
        e = np.broadcast_to(ev[..., None], cand.shape)[cand]
        rows[x] = (p, y, e)
    bins = np.array([0, .05, .1, .2, .3, .4, .5, .6, .7, .8, .9, 1.0001])
    md += [f"**Reliability{title}**: probability given to each candidate map vs how often it was the choice "
           f"({int(m.sum())} Matches, steps {', '.join(str(s + 1) for s in steps)}; each cell: mean forecast → observed rate (n candidates)).", "",
           "| Forecast bin | " + " | ".join(LABEL[x] for x in methods) + " |", "|---|" + "---|" * len(methods)]
    for lo, hi in zip(bins[:-1], bins[1:]):
        cells = []
        for x in methods:
            p, y, _ = rows[x]
            sel = (p >= lo) & (p < hi)
            cells.append(f"{p[sel].mean():.3f} → {y[sel].mean():.3f} ({int(sel.sum())})" if sel.sum() else "–")
        md.append(f"| {lo:.2f}-{min(hi, 1):.2f} | " + " | ".join(cells) + " |")
    ece = []
    for x in methods:
        p, y, e = rows[x]
        idx = np.digitize(p, bins) - 1
        tot = sum(abs(p[idx == b].mean() - y[idx == b].mean()) * (idx == b).sum() for b in range(len(bins) - 1) if (idx == b).any()) / len(p)
        ece.append(f"{tot:.4f}")
    md += ["| expected calibration error | " + " | ".join(ece) + " |", ""]


def confidence_table(view, md):
    m = view.mask(fmt=1)
    md += ["**Top-1 hit rate by the confidence of the forecast** (Bo3, next-step mode; confidence = probability of the top choice; "
           "steps 1-2 and 5-6 are bans, 3-4 team picks)", "",
           "| Confidence of the top choice | Bans: n | Bans: EB hit (%) | Bans: freq hit (%) | Picks: n | Picks: EB hit (%) | Picks: freq hit (%) |", "|---|---|---|---|---|---|---|"]
    bins = [(0, .3), (.3, .5), (.5, .7), (.7, .9), (.9, 1.01)]
    out = {}
    for grp, st in (("bans", [0, 1, 4, 5]), ("picks", [2, 3])):
        for x in ("eb", "freq"):
            dist = view.r[f"dist_next_{x}"][m][:, st].astype(float)
            conf = dist.max(-1).ravel()
            out[(grp, x)] = (conf, view.r[f"next_{x}"]["hit1"][m][:, st].ravel())
    for lo, hi in bins:
        cells = []
        for grp in ("bans", "picks"):
            ce, he = out[(grp, "eb")]
            cf, hf = out[(grp, "freq")]
            se, sf = (ce >= lo) & (ce < hi), (cf >= lo) & (cf < hi)
            cells += [f"{int(se.sum())} / {int(sf.sum())}", f"{he[se].mean() * 100:.1f}" if se.sum() else "–", f"{hf[sf].mean() * 100:.1f}" if sf.sum() else "–"]
        md.append(f"| {lo:.1f}-{min(hi, 1):.1f} | " + " | ".join(cells) + " |")
    md += ["", "(n is shown as EB / freq in the first column of each group)", ""]


def true_changes(D):
    """Pool changes read from the scored Matches' own vetoes: the Events (by start) whose most common
    seven-map pool differs from the previous Event's. Returns [(event_id, first cutoff, new, old)]."""
    from collections import Counter
    by = {}
    for i, rec in enumerate(D.R):
        if rec["scored"]:
            by.setdefault(rec["event_id"], []).append((rec["cutoff"], frozenset(rec["maps"])))
    evs = sorted(by, key=lambda e: min(c for c, _ in by[e]))
    out, prev = [], None
    for e in evs:
        pool = Counter(p for _, p in by[e]).most_common(1)[0][0]
        if prev is not None and pool != prev:
            out.append((e, min(c for c, _ in by[e]), pool, prev))
        prev = pool
    return out


def post_change_flag(D, days=28):
    DAY = 86_400_000_000
    flag = np.zeros(len(D.R), bool)
    for _, t, _, _ in true_changes(D):
        flag |= (D.cut >= t) & (D.cut < t + days * DAY)
    return flag


def breakdowns(view, md):
    D = view.D
    nprior = view.r["nfmt"]
    groups = [("Bo3 team picks (steps 3-4)", 1, [2, 3]), ("Bo3 bans (steps 1, 2, 5, 6)", 1, [0, 1, 4, 5]), ("Bo1 bans (steps 1-6)", 0, [0, 1, 2, 3, 4, 5])]
    dims = []
    for title, fmt, st in groups:
        m = view.mask(fmt=fmt)
        if not m.sum():
            continue
        np_step = view.side_info(st, m, nprior)
        tier = np.repeat(D.tier[m][:, None], len(st), 1).ravel()
        change = np.repeat(post_change_flag(D)[m][:, None], len(st), 1).ravel()
        pool_ok = np.repeat(view.r["pool_ok"][m][:, None], len(st), 1).ravel()
        levels = [("tier S", tier == "s"), ("tier A", tier == "a"),
                  ("acting team: < 5 earlier vetoes of this format", np_step < 5), ("5-14", (np_step >= 5) & (np_step < 15)),
                  ("15-39", (np_step >= 15) & (np_step < 40)), ("≥ 40", np_step >= 40),
                  ("within 28 days after a true pool change", change), ("other times", ~change),
                  ("detected pool = actual seven maps", pool_ok), ("detected pool ≠ actual", ~pool_ok)]
        md += [f"**{title}** by subset (next-step mode; Matches {int(m.sum())})", "",
               "| Subset | Steps | uniform hit (%) | team freq hit (%) | EB hit (%) | EB − freq, hit (pp) | EB − freq, log-loss |", "|---|---|---|---|---|---|---|"]
        ev = np.repeat(D.ev[m][:, None], len(st), 1).ravel()
        vals = {x: {k: view.vals(x, k, st, m)[0] for k in ("hit1", "ll")} for x in ("uniform", "freq", "eb")}
        for name, sel in levels:
            if sel.sum() < 20:
                md.append(f"| {name} | {int(sel.sum())} | – | – | – | – | – |")
                continue
            e = ev[sel]
            c = lambda x, k: vals[x][k][sel]
            md.append(f"| {name} | {int(sel.sum())} | {pct(ci(e, c('uniform', 'hit1')))} | {pct(ci(e, c('freq', 'hit1')))} | "
                      f"{pct(ci(e, c('eb', 'hit1')))} | {sgn(ci(e, c('eb', 'hit1') - c('freq', 'hit1')), 100, 1)} | "
                      f"{sgn(ci(e, c('eb', 'll') - c('freq', 'll')), 1, 3)} |")
        md.append("")


def oracle_table(vd, vo, md):
    md += ["**Sensitivity: the detected pool vs the actual seven maps** (Bo3, scored; top-1 hit (%) and log-loss, next-step mode). "
           "With the oracle pool the candidates are the seven maps of the Match's own veto (not a real forecast: it reads the Match's pool).", "",
           "| Method | Pool | First bans: hit | Picks: hit | Second bans: hit | First bans: log-loss | Picks: log-loss | Second bans: log-loss |", "|---|---|---|---|---|---|---|---|"]
    for x in ("uniform", "freq", "eb", "lab"):
        for nm, v in (("detected", vd), ("actual", vo)):
            m = v.mask(fmt=1)
            cells = [pct(row(v, x, "hit1", st, m)) for st in ([0, 1], [2, 3], [4, 5])]
            cells += [num(row(v, x, "ll", st, m), 3) for st in ([0, 1], [2, 3], [4, 5])]
            md.append(f"| {LABEL[x]} | {nm} | " + " | ".join(cells) + " |")
    md.append("")
    a, e = vo.vals("eb", "hit1", [2, 3], vo.mask(fmt=1))
    b, _ = vo.vals("freq", "hit1", [2, 3], vo.mask(fmt=1))
    md += [f"With the oracle pool, the primary difference (EB − freq, Bo3 picks, top-1) is {sgn(ci(e, a - b), 100, 1, ' pp')} (exploratory).", ""]


def extra_table(view, md):
    m_all = view.mask(fmt=1, scored=False)
    D = view.D
    md += ["**Extended archive, not scored (exploratory)**: the same Bo3 comparison on Matches outside the scored set (tier B/C, "
           "older S/A, qualifiers; all before the cutoff of 2026-11-25, none in the Holdout). Their vetoes also feed the histories of the scored set.", "",
           "| Block | Matches | uniform picks hit (%) | team freq picks hit (%) | EB picks hit (%) | EB − freq, picks (pp) | team freq first-ban hit (%) | EB first-ban hit (%) |", "|---|---|---|---|---|---|---|---|"]
    for blk in ("scope", "sa_old", "b", "c", "qualifier"):
        m = m_all & (D.block == blk) & ~D.scored
        if m.sum() < 30:
            continue
        a, e = view.vals("eb", "hit1", [2, 3], m)
        b, _ = view.vals("freq", "hit1", [2, 3], m)
        md.append(f"| {blk} | {int(m.sum())} | {pct(row(view, 'uniform', 'hit1', [2, 3], m))} | {pct(ci(e, b))} | {pct(ci(e, a))} | "
                  f"{sgn(ci(e, a - b), 100, 1)} | {pct(row(view, 'freq', 'hit1', [0, 1], m))} | {pct(row(view, 'eb', 'hit1', [0, 1], m))} |")
    md.append("")


def maps_table(view, md):
    D = view.D
    m = view.mask(fmt=1)
    A = D.A[m]
    pool = np.zeros((m.sum(), V), bool)
    for k in range(7):
        pool[np.arange(len(A)), A[:, k]] = True
    md += [f"**What Bo3 vetoes look like** ({int(m.sum())} scored Bo3 Matches): for each map, how often it was the choice at each step when it was in the Match's pool (%).", "",
           "| Map | In the pool (Matches) | Banned at steps 1-2 | Picked (steps 3-4) | Banned at steps 5-6 | Decider |", "|---|---|---|---|---|---|"]
    for j, name in enumerate(MAPS):
        n = int(pool[:, j].sum())
        if n == 0:
            continue
        cells = [f"{100 * ((A[:, st] == j).any(1)).sum() / n:.1f}" for st in ([0, 1], [2, 3], [4, 5], [6])]
        md.append(f"| {name} | {n} | " + " | ".join(cells) + " |")
    md.append("")


def pool_table(view, md, D):
    import pandas as pd
    r = view.r
    m = view.mask(fmt=None)
    DAY = 86_400_000_000
    flag = post_change_flag(D)
    md += ["**Active-pool detector** (`ActivePool`: the seven maps most frequent in the 7-step vetoes of the last 14 days, refreshed at most every 7 days)", "",
           f"The detected pool equals the seven maps of the Match's own veto in {r['pool_ok'][m].mean() * 100:.1f}% of the {int(m.sum())} scored Matches "
           f"({int((m & ~r['pool_ok']).sum())} do not match); within 28 days after a true pool change (below) {r['pool_ok'][m & flag].mean() * 100:.1f}% of "
           f"{int((m & flag).sum())}, otherwise {r['pool_ok'][m & ~flag].mean() * 100:.1f}%. "
           f"The lab's pool (the seven maps of the latest earlier veto of that format) equals it in {r['lab_pool_ok'][m].mean() * 100:.1f}%.", "",
           "True pool changes, read from the scored Matches' own vetoes (the Event whose most common seven-map pool differs from the previous Event's), "
           "against the detector:", "",
           "| First Match of the new pool | Event | Map in | Map out | Detector saw it on | Lag (days) | Scored Matches played with a wrong detected pool | Matches since the change until the detector caught up |",
           "|---|---|---|---|---|---|---|---|"]
    det = [(t, a, d) for t, a, d in r["changes"]]
    for e, t, new_pool, old_pool in true_changes(D):
        ins = sorted(MAPS[i] for i in new_pool - old_pool)
        outs = sorted(MAPS[i] for i in old_pool - new_pool)
        # the first detector change at or after the true change that adds a new map
        seen = next((dt for dt, a_, d_ in det if dt >= t - 60 * DAY and set(a_) & set(ins)), None)
        end = max(seen, t) if seen is not None else t + 60 * DAY
        wrong = m & ~r["pool_ok"] & (D.cut >= t) & (D.cut < end)
        lag = "–" if seen is None else f"{(seen - t) / DAY:.1f}"
        ts = lambda x: pd.Timestamp(x, unit="us", tz="UTC").strftime("%Y-%m-%d")
        md.append(f"| {ts(t)} | {e} | {', '.join(ins)} | {', '.join(outs)} | {'not seen' if seen is None else ts(seen)} | {lag} | {int(wrong.sum())} | "
                  f"{int((m & (D.cut >= t) & (D.cut < end)).sum())} |")
    md += ["", "Detector changes (all Events of the archive, not only the scored ones), with the share of 7-step vetoes containing the map in the 28 days before and after:", "",
           "| Detected on | Map in | Map out | Share with the map in, before → after | Share with the map out, before → after |", "|---|---|---|---|---|"]
    t_ = np.array([x["cutoff"] for x in D.R])
    names = [[MAPS[i] for i in x["maps"]] for x in D.R]
    for t, a, d in r["changes"]:
        cells = []
        for mp in (a[0], d[0]):
            bef = [mp in n for n, tt in zip(names, t_) if t - 28 * DAY <= tt < t]
            aft = [mp in n for n, tt in zip(names, t_) if t <= tt < t + 28 * DAY]
            cells.append(f"{np.mean(bef) * 100:.0f}% → {np.mean(aft) * 100:.0f}%")
        md.append(f"| {pd.Timestamp(t, unit='us', tz='UTC').strftime('%Y-%m-%d')} | {', '.join(a)} | {', '.join(d)} | {cells[0]} | {cells[1]} |")
    md.append("")


def config_table(view, md):
    r = view.r
    m = view.mask(fmt=1)
    for name, grid, fmt in (("freq", FREQ_GRID, lambda g: f"N={g[0]}, α={g[1]}"), ("global", GLOB_GRID, lambda g: f"last {g}"), ("shrunk", SHR_GRID, lambda g: f"N={g[0]}, κ={g[1]}")):
        cnt = np.bincount(r[f"cfg_{name}"][m], minlength=len(grid))
        md.append(f"Walk-forward choice for `{name}` on the scored Bo3 Matches: " + "; ".join(f"{fmt(g)}: {c}" for g, c in zip(grid, cnt) if c) + ".")
    md.append("")


def main():
    D = Data()
    vd, vo = View(D, "detected"), View(D, "oracle")
    md = [MARK, "", "# Tables (generated by `uv run python -m kbml.veto_report`)", "",
          f"Scored vetoes: {int((vd.mask(fmt=None)).sum())} of the 2,087 anchor Matches (Bo1 {int(vd.mask(fmt=0).sum())}, Bo3 {int(vd.mask(fmt=1).sum())}, "
          f"Bo5 {int(vd.mask(fmt=2).sum())}); the rest have no linked HLTV page or no full veto. Intervals: 90%, Event bootstrap, 2000 resamples, seed 33. "
          "Pool = detected pool unless stated.", ""]
    primary(vd, md)
    md += ["## Step by step (exploratory)", ""]
    per_step_tables(vd, md, 1, "Bo3")
    per_step_tables(vd, md, 0, "Bo1")
    per_step_tables(vd, md, 2, "Bo5", methods=["uniform", "global", "freq", "eb", "lab"])
    config_table(vd, md)
    md += ["## Before the veto: what the simulators are used for (exploratory)", ""]
    pre_veto_table(vd, md, 1, "Bo3")
    pre_veto_table(vd, md, 0, "Bo1")
    set_table(vd, md)
    md += ["## Calibration (exploratory)", ""]
    reliability(vd, md, fmt=1, title=" (Bo3, next-step mode)")
    reliability(vd, md, fmt=0, steps=(0, 1, 2, 3, 4), title=" (Bo1, next-step mode)")
    reliability(vd, md, fmt=1, steps=(0, 1, 2, 3, 4, 5, 6), methods=("uniform", "eb", "lab"), kind="pre", title=" (Bo3, pre-veto mode)")
    confidence_table(vd, md)
    md += ["## Breakdowns (exploratory)", ""]
    maps_table(vd, md)
    breakdowns(vd, md)
    md += ["## Pool and sensitivity (exploratory)", ""]
    pool_table(vd, md, D)
    oracle_table(vd, vo, md)
    extra_table(vd, md)
    text = DOC.read_text(encoding="utf-8")
    head = text.split(MARK)[0].rstrip() + "\n\n"
    DOC.write_text(head + "\n".join(md) + "\n", encoding="utf-8")
    print(f"wrote {DOC}")


if __name__ == "__main__":
    main()
