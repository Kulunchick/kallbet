"""`<gpu python> -m kbml.league_report [--device cuda]` → CATCHUP.md, section "Real League".

1. Behaviour of the real League's members on Singapore 2026 Stage 1 (aggregates, anonymised).
2. The fitted opponent model (`kbml.opponent_model`) with Match-bootstrap intervals.
3. The 57 archived Events replayed against fitted opponents (10 and 12 members), plain, 40-60 and
   DP bots (exploratory; the primary was decided in the earlier section).
4. The real League replayed point-in-time with a plain, a 40-60 and a DP lgbm-ext bot added.

PRIVACY: see `kbml.league_real`. Only aggregates and anonymised member letters are written.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
import time
from pathlib import Path

import numpy as np
import pandas as pd
import torch

from . import catchup, catchup_dp as dp, catchup_gpu as cg, catchup_report as cr, league_real as L
from . import opponent_model as om, paths, stats

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "CATCHUP.md"
START, END = "<!-- real-league:start (kbml.league_report rewrites this block) -->", "<!-- real-league:end -->"
EVENT_MATCHES = 33 + 33 + 33 + 1 + 7  # Singapore 2026: three Swiss Stages, a decider, an 8-team bracket
EPS_BINS = 12


def pct(x):
    return f"{100 * x:.0f}%"


# ---------------------------------------------------------------- 1. behaviour

def behaviour(games, picks, members):
    df = L.annotate(games, picks, members)
    h, b = df[~df.is_bot], df[df.is_bot]
    out = {}
    g = games
    acc = {"anchor": float(((g.anchor_p >= 0.5) == g.team_a_won).mean()),
           "lgbm": float(((g.lgbm_p >= 0.5) == g.team_a_won).mean()),
           "market": float(((g.market_p >= 0.5) == g.team_a_won)[g.market_p.notna()].mean())}
    out["acc"] = acc
    out["n"] = {"games": len(g), "market": int(g.market_p.notna().sum()), "human_picks": len(h), "humans": h.member.nunique(),
                "bot_picks": len(b), "bots": b.member.nunique(),
                "anchor_lgbm_same": float(((g.anchor_p >= 0.5) == (g.lgbm_p >= 0.5)).mean()),
                "anchor_market_same": float(((g.anchor_p >= 0.5) == (g.market_p >= 0.5))[g.market_p.notna()].mean())}
    rows = []
    for name, d in (("humans", h), ("bot", b)):
        rows.append((name, len(d), d.picks_anchor_fav.mean(), d.picks_lgbm_fav.mean(),
                     d.picks_market_fav.mean() if d.picks_market_fav.notna().any() else float("nan"),
                     int(d.picks_market_fav.notna().sum()), d.correct.mean()))
    out["who"] = rows
    out["boot"] = {k: L.boot_matches(h, f) for k, f in
                   (("anchor", lambda d: d.picks_anchor_fav.mean()), ("lgbm", lambda d: d.picks_lgbm_fav.mean()),
                    ("market", lambda d: d.picks_market_fav.mean()), ("correct", lambda d: d.correct.mean()))}
    out["members"] = h.groupby("member").agg(n=("db_id", "size"), anchor=("picks_anchor_fav", "mean"),
                                             lgbm=("picks_lgbm_fav", "mean"), correct=("correct", "sum")).reset_index()
    edges = [0.5, 0.55, 0.6, 0.7, 0.8, 1.0]
    hb = h.assign(bin=pd.cut(h.p_anchor_fav, edges, include_lowest=True))
    out["bins"] = hb.groupby("bin", observed=True).agg(matches=("db_id", "nunique"), picks=("db_id", "size"),
                                                       follow=("picks_anchor_fav", "mean")).reset_index()
    hl = h.assign(bin=pd.cut(h.p_lgbm_fav, edges, include_lowest=True))
    out["bins_lgbm"] = hl.groupby("bin", observed=True).agg(matches=("db_id", "nunique"), picks=("db_id", "size"),
                                                            follow=("picks_lgbm_fav", "mean")).reset_index()
    # pairwise agreement and agreement with the published leader
    agree, tot, lead_agree, lead_tot, lead_fav = 0, 0, 0, 0, 0
    for k, gk in h.groupby("db_id"):
        y = gk.picked_a.to_numpy()
        n = len(y)
        same = sum(int(y[i] == y[j]) for i in range(n) for j in range(i + 1, n))
        agree += same
        tot += n * (n - 1) // 2
        cut = int(games.set_index("db_id").loc[k, "cutoff"])
        pts = L.standings_before(games, h, cut)
        if max(pts.values()) == 0:
            continue
        top = max(pts.values())
        leaders = {m for m, v in pts.items() if v == top}
        lp = gk[gk.member.isin(leaders)]
        others = gk[~gk.member.isin(leaders)]
        for lpick in lp.picked_a:
            lead_agree += int((others.picked_a == lpick).sum())
            lead_tot += len(others)
        lead_fav += int(lp.picks_anchor_fav.sum())
    out["pairwise"] = agree / tot
    out["with_leader"] = lead_agree / lead_tot if lead_tot else float("nan")
    t = h.minutes_before_cutoff
    out["timing"] = {"median_h": float(t.median() / 60), "q25_h": float(t.quantile(0.25) / 60),
                     "q75_h": float(t.quantile(0.75) / 60), "lt60": float((t < 60).mean()),
                     "lt15": float((t < 15).mean()), "gt24h": float((t > 1440).mean()),
                     "bot_median_min": float(b.minutes_before_cutoff.median()) if len(b) else float("nan"),
                     "bot_max_min": float(b.minutes_before_cutoff.max()) if len(b) else float("nan")}
    timing_by_p = h.assign(close=h.p_anchor_fav <= 0.6).groupby("close").minutes_before_cutoff.median() / 60
    out["timing_close"] = {bool(k): float(v) for k, v in timing_by_p.items()}
    out["stage2_predictions"] = L.stage2_count()
    return out, h


# ---------------------------------------------------------------- 2. opponent model

def fit_model(h, device):
    x = np.log(h.p_anchor_fav / (1 - h.p_anchor_fav)).to_numpy()
    y = h.picks_anchor_fav.to_numpy(float)
    mem = h.member.astype("category").cat.codes.to_numpy()
    mat = h.db_id.astype("category").cat.codes.to_numpy()
    t = time.time()
    res = om.estimate(x, y, mem, mat, device)
    if device != "cpu":
        torch.cuda.synchronize()
    res["seconds"] = time.time() - t
    return res


# ---------------------------------------------------------------- 3. archive replay

def eps_grid(theta):
    lo, hi = 1 - float(om.follow_prob(theta, 0.0)), 1 - float(om.follow_prob(theta, 4.0))
    return np.linspace(min(lo, hi), max(lo, hi), EPS_BINS)


def eps_bin_of(theta, grid, anchor_p):
    a = np.asarray(anchor_p, float)
    pf = np.maximum(a, 1 - a)
    x = np.log(pf / (1 - pf))
    e = 1 - om.follow_prob(theta, np.nan_to_num(x))
    return np.abs(e[:, None] - grid[None, :]).argmin(1)


def game_types(games, src, theta, grid):
    """Per game: (has consensus, q rounded to 1/40, noise bin, favourite's p), vectorised."""
    anc = np.array([np.nan if g[2] is None else g[2] for g in games], float)
    p = anc if src == "anchor" else np.array([np.nan if g[5].get(src) is None else g[5][src] for g in games], float)
    ok = ~np.isnan(p)
    has = ok & ~np.isnan(anc)
    q = np.where(anc >= 0.5, p, 1 - p)
    e = eps_bin_of(theta, grid, np.where(np.isnan(anc), 0.5, anc))
    return {"has": has[ok], "q": np.round(q[ok] * 40) / 40, "e": e[ok], "fav": np.maximum(p, 1 - p)[ok]}


def mixed_types(parts):
    """(types, w_none, f_none) for `build_mixed` from earlier games' `game_types`."""
    has = np.concatenate([t["has"] for t in parts])
    q = np.concatenate([t["q"] for t in parts])[has]
    e = np.concatenate([t["e"] for t in parts])[has]
    fav = np.concatenate([t["fav"] for t in parts])[~has]
    keys, w = np.unique(np.c_[q, e], axis=0, return_counts=True)
    types = [(float(k[0]), int(k[1]), float(c)) for k, c in zip(keys, w)]
    return types, float(len(fav)), float(fav.mean()) if len(fav) else 0.75


def event_table(events, ids, infos, e, per_event, n_opp, grid, rmax, device, fixed=False):
    prior = cr.earlier(ids, infos, e)
    if sum(len(events[j]) for j in prior) < 100:
        types = [(float(q), 0 if fixed else EPS_BINS // 2, 1.0) for q in np.linspace(0.1, 0.9, 17)]
        wn, fn = 0.0, 0.75
    else:
        types, wn, fn = mixed_types([per_event[j] for j in prior])
        if fixed:
            types = [(q, 0, w) for q, _, w in types]
    return cg.build_mixed(n_opp, grid, types, wn, fn, rmax, device)


def archive_replay(events, ids, infos, theta, boot, size, device, trials=2000, seed=33, log=print):
    """Both opponent variants; each Event's three DP tables are built when its trials run and freed
    after (a table set for every Event at once would not fit a shared GPU)."""
    n_opp = size - 1
    rmax = max(len(g) for g in events) + 2
    grid, grid10 = eps_grid(theta), np.array([0.10])
    per = {src: [game_types(g, src, theta, grid) for g in events] for src in ("anchor", "lgbm ext")}
    rng = stats.SeededRng(seed)
    drawn = np.array([int(catchup.next_f64(rng) * len(events)) for _ in range(trials)])
    labels = (("fitted, point estimate", False), ("fitted, parameter uncertainty", True))
    gens = {label: torch.Generator().manual_seed(seed) for label, _ in labels}
    bdraw = np.random.default_rng(seed).integers(0, len(boot), trials)
    names = [("anchor", "plain"), ("anchor", "static 40-60"), ("anchor", "DP (fitted opponents)"),
             ("lgbm ext", "plain"), ("lgbm ext", "static 40-60"), ("lgbm ext", "DP (fitted opponents)"),
             ("lgbm ext", "DP built for 10% noise")]
    places = {label: np.zeros((trials, len(names)), dtype=int) for label, _ in labels}
    t_tables, t_replay = 0.0, 0.0
    for e in range(len(events)):
        sel = np.flatnonzero(drawn == e)
        if not len(sel):
            continue
        t0 = time.time()
        ta = event_table(events, ids, infos, e, per["anchor"], n_opp, grid, rmax, device)
        tl = event_table(events, ids, infos, e, per["lgbm ext"], n_opp, grid, rmax, device)
        t10 = event_table(events, ids, infos, e, per["lgbm ext"], n_opp, grid10, rmax, device, fixed=True)
        t_tables += time.time() - t0
        t0 = time.time()
        ev = cg.event_arrays(events[e], ["anchor", "lgbm ext"], device)
        eb = torch.tensor(eps_bin_of(theta, grid, np.nan_to_num(ev["anchor"].cpu().numpy(), nan=0.5)), device=device)
        bots = [("anchor", "plain", None), ("anchor", "window", None), ("anchor", "dp", ta),
                ("lgbm ext", "plain", None), ("lgbm ext", "window", None), ("lgbm ext", "dp", tl)]
        for label, per_trial in labels:
            th_t = boot[bdraw[sel]] if per_trial else None
            pick = cg.fitted_opponents(ev, len(sel), n_opp, theta, gens[label], device, th_t)
            pl = cg.replay(ev, pick, bots, eps_bin=eb)
            pl10 = cg.replay(ev, pick, [("lgbm ext", "dp", t10)], eps_bin=torch.zeros_like(eb))
            places[label][sel] = torch.cat([pl, pl10], 1).cpu().numpy()
            del pick, pl, pl10
        del ta, tl, t10, ev, eb
        if device != "cpu":
            torch.cuda.synchronize()
            torch.cuda.empty_cache()
        t_replay += time.time() - t0
    res = {}
    for label, _ in labels:
        P = places[label]
        first = (P == 1).astype(float)
        rows = {}
        for i, (src, pol) in enumerate(names):
            base = names.index((src, "plain"))
            d = first[:, i] - first[:, base]
            rows[(src, pol)] = {"first": catchup.hoeffding(first[:, i], 0, 1),
                                "place": catchup.hoeffding(P[:, i], 1, size),
                                "d_first": catchup.hoeffding(d, -1, 1), "d_boot": stats.event_bootstrap(drawn, d),
                                "d_place": catchup.hoeffding(P[:, i] - P[:, base], -(size - 1), size - 1)}
        res[label] = rows
    log(f"archive replay {size} members: tables {t_tables:.1f}s, replays {t_replay:.1f}s ({device})")
    return res, t_tables, t_replay


# ---------------------------------------------------------------- 4. the real League

def real_replay(games, picks, members, events, theta, device):
    """The real League's 10 members (picks as made) and added bots, point-in-time, Stage 1 only."""
    mem = sorted(picks.member.unique())
    G = len(games)
    pick = np.full((1, len(mem), G), np.nan)
    pos = {k: i for i, k in enumerate(games.db_id)}
    for r in picks.itertuples():
        pick[0, mem.index(r.member), pos[r.db_id]] = float(r.picked_a)
    tup = [(int(r.cutoff), r.known_finish if r.known_finish is None or not pd.isna(r.known_finish) else None,
            float(r.anchor_p), None if r.market_p is None or pd.isna(r.market_p) else float(r.market_p),
            bool(r.team_a_won), {"lgbm ext": float(r.lgbm_p)}) for r in games.itertuples()]
    tup = [(c, None if k is None else int(k), a, m, w, ps) for c, k, a, m, w, ps in tup]
    ev = cg.event_arrays(tup, ["lgbm ext"], device)
    grid = eps_grid(theta)
    types, wn, fn = mixed_types([game_types(g, "lgbm ext", theta, grid) for g in events])
    n_opp = len(mem)
    eb = torch.tensor(eps_bin_of(theta, grid, games.anchor_p.to_numpy()), device=device)
    out = {}
    for horizon, extra in (("whole Event (107 Matches)", EVENT_MATCHES - G), ("Stage 1 only (33 Matches)", 0)):
        tab = cg.build_mixed(n_opp, grid, types, wn, fn, EVENT_MATCHES + 2, device)
        # the replay counts unpublished results of Stage 1; add the Event's later Matches
        tab.U = tab.U[extra:] if extra else tab.U
        bots = [("lgbm ext", "plain", None), ("lgbm ext", "window", None), ("lgbm ext", "dp", tab)]
        _, bp = cg.replay(ev, torch.tensor(pick, device=device), bots, eps_bin=eb, return_picks=True)
        picks_made = bp[0].cpu().numpy()
        pts_bot = [int(((picks_made[k] == ev["won"].cpu().numpy()).sum())) for k in range(len(bots))]
        pts_real = np.nansum(pick[0] == ev["won"].cpu().numpy()[None, :], axis=1).astype(int)
        diff = np.flatnonzero(picks_made[2] != picks_made[0])
        out[horizon] = {"bot_points": pts_bot, "real_points": sorted(pts_real.tolist(), reverse=True),
                        "places": [1 + int((pts_real > p).sum()) for p in pts_bot],
                        "differs": [(int(j), float(games.lgbm_p[j]), float(games.anchor_p[j]),
                                     bool(picks_made[2][j] == ev["won"].cpu().numpy()[j])) for j in diff]}
    return out


def write_block(text: str):
    cur = REPORT.read_text(encoding="utf-8")
    block = START + "\n\n" + text.strip() + "\n\n" + END
    if START in cur:
        head, rest = cur.split(START, 1)
        tail = rest.split(END, 1)[1]
        new = head + block + tail
    else:
        head, sep, tail = cur.partition(cr.MARK)
        new = head.rstrip() + "\n\n" + block + "\n\n" + sep + tail
    REPORT.write_text(new, encoding="utf-8")


# ---------------------------------------------------------------- text

def fmt(t, k=3):
    return f"{t[0]:.{k}f} [{t[1]:.{k}f}, {t[2]:.{k}f}]"


def fmtd(t, k=3):
    return f"{t[0]:+.{k}f} [{t[1]:+.{k}f}, {t[2]:+.{k}f}]"


def markdown(beh, model, arch, real, timings, n_events, trials):
    n = beh["n"]
    o = ["## Real League (exploratory, written by `kbml.league_report`)", "",
         "Operator-approved, read-only copy of the production backup (2026-11-27 16:16 UTC): one League of "
         f"{n['humans']} human members and {n['bots']} Bot User, on PGL Major Singapore 2026. Only Stage 1 is used: "
         f"its {n['games']} Matches had all finished by then. The {beh['stage2_predictions']} Predictions on Stage 2 "
         "(Holdout) are only counted. Members are A-I by ascending user id; no names, Telegram ids or rows "
         "leave the database. Anchor and market probabilities per Match are point-in-time (the Stage 1 fixture, "
         f"{n['market']} market lines). lgbm-ext is fitted once on results known before the Stage opened "
         "(11,046 Matches), with each Match's features as of its own cutoff. KallBet keeps only the last "
         "`updated_at` of a Prediction, so how often members *changed* picks cannot be measured, only when "
         "they last set them.", "",
         "### How members pick", "",
         "| Who | Picks | Anchor favourite | lgbm-ext favourite | Market favourite (Matches with a price) | Correct |",
         "|---|---:|---|---|---|---|"]
    bt = beh["boot"]
    for name, cnt, a, l, mk, nm, c in beh["who"]:
        if name == "humans":
            o.append(f"| humans | {cnt} | {pct(a)} [{pct(bt['anchor'][1])}, {pct(bt['anchor'][2])}] | "
                     f"{pct(l)} [{pct(bt['lgbm'][1])}, {pct(bt['lgbm'][2])}] | {pct(mk)} [{pct(bt['market'][1])}, "
                     f"{pct(bt['market'][2])}] ({nm} picks) | {pct(c)} [{pct(bt['correct'][1])}, {pct(bt['correct'][2])}] |")
        else:
            o.append(f"| Bot User | {cnt} | {pct(a)} | {pct(l)} | {'—' if nm == 0 else pct(mk)} | {pct(c)} |")
    acc = beh["acc"]
    o += ["", f"Brackets: 90% Match bootstrap (2000 resamples, seed 33). For reference, on these {n['games']} Matches "
          f"the anchor favourite won {pct(acc['anchor'])}, the lgbm-ext favourite {pct(acc['lgbm'])}, the market "
          f"favourite {pct(acc['market'])} (of {n['market']}). The anchor and lgbm-ext favour the same team in "
          f"{pct(n['anchor_lgbm_same'])} of Matches, the anchor and the market in {pct(n['anchor_market_same'])}.", "",
          "| Member | Picks | Anchor favourite | lgbm-ext favourite | Correct |", "|---|---:|---:|---:|---:|"]
    for r in beh["members"].itertuples():
        o.append(f"| {r.member} | {r.n} | {pct(r.anchor)} | {pct(r.lgbm)} | {r.correct} |")
    for key, label in (("bins", "anchor"), ("bins_lgbm", "lgbm-ext")):
        o += ["", f"| {label} favourite's probability | Matches | Human picks | Share on the {label} favourite |",
              "|---|---:|---:|---:|"]
        for r in beh[key].itertuples():
            o.append(f"| {r.bin.left:.2f}-{r.bin.right:.2f} | {r.matches} | {r.picks} | {pct(r.follow)} |")
    t = beh["timing"]
    o += ["", f"Agreement: two humans pick the same team on {pct(beh['pairwise'])} of shared Matches; a "
          f"non-leading human matches the published leader(s) on {pct(beh['with_leader'])}. Timing: the last change of "
          f"a human Prediction is a median {t['median_h']:.1f} h before Lock (quartiles {t['q25_h']:.1f}-{t['q75_h']:.1f} h); "
          f"{pct(t['lt60'])} within the last hour, {pct(t['lt15'])} within 15 minutes, {pct(t['gt24h'])} over a day ahead. "
          f"Median for Matches with a 50-60% anchor favourite: {beh['timing_close'].get(True, float('nan')):.1f} h, others "
          f"{beh['timing_close'].get(False, float('nan')):.1f} h. The Bot User predicted {n['bot_picks']} Stage 1 Matches, "
          f"all on the anchor favourite, a median {t['bot_median_min']:.0f} min before Lock (at most "
          f"{t['bot_max_min']:.0f}).", ""]
    p, lo, hi = model["point"], np.quantile(model["boot"], 0.05, axis=0), np.quantile(model["boot"], 0.95, axis=0)
    names = ["α (intercept)", "β (slope on logit of the anchor favourite's p)", "τ (member lean, sd)",
             "σ (shared per-Match swing, sd)"]
    o += ["### Fitted opponent model", "",
          "P(member j picks the anchor favourite of Match k) = σ(α + β·logit p_fav,k + u_j + v_k), u_j ~ N(0, τ²), "
          "v_k ~ N(0, σ²). α, β and τ come from maximum marginal likelihood (Gauss-Hermite over u). σ is set by matching the "
          f"observed pairwise agreement ({pct(model['observed_agreement'])}). Brackets: 90% Match bootstrap, 2000 "
          "resamples, every resample refitted.", "",
          "| Parameter | Estimate [90%] |", "|---|---|"]
    for k in range(4):
        o.append(f"| {names[k]} | {p[k]:.2f} [{lo[k]:.2f}, {hi[k]:.2f}] |")
    fp = lambda x: float(om.follow_prob(p, x))
    o += ["", f"Implied share on the anchor favourite: {pct(fp(math.log(0.55 / 0.45)))} at 55%, "
          f"{pct(fp(math.log(0.7 / 0.3)))} at 70%, {pct(fp(math.log(0.9 / 0.1)))} at 90%. A favourite follower with 10% "
          "noise would be at 90% everywhere.", ""]
    o += ["### The 57 archived Events against fitted opponents", "",
          f"{n_events} in-scope Events, {trials} paired trials (the Event draws of `kbml.catchup`, seed 33). The opponents "
          "follow the fitted model on each Match's anchor favourite, with their own lean u_j drawn per trial and one "
          "swing v_k per Match. \"Parameter uncertainty\" draws (α, β, τ, σ) per trial from the bootstrap. "
          "The DP (fitted opponents) uses the model's mean noise for each Match type, 1 − E[P(follow)]; the DP built "
          "for 10% noise is the earlier section's. Δ is against plain with the same probabilities. Brackets: 90% "
          "Hoeffding; Δ also Event-bootstrap.", ""]
    for (size, label), rows in arch.items():
        o += [f"#### {size} members, {label}", "",
              "| Probabilities | Policy | P(1st) | Mean place | Δ P(1st) (Hoeffding) | Δ P(1st) (Event bootstrap) | Δ mean place |",
              "|---|---|---|---|---|---|---|"]
        for (src, pol), v in rows.items():
            if pol == "plain":
                o.append(f"| {src} | plain | {fmt(v['first'])} | {fmt(v['place'], 2)} | — | — | — |")
            else:
                b = v["d_boot"]
                o.append(f"| {src} | {pol} | {fmt(v['first'])} | {fmt(v['place'], 2)} | {fmtd(v['d_first'])} | "
                         f"{b['mean']:+.3f} [{b['low']:+.3f}, {b['high']:+.3f}] | {fmtd(v['d_place'], 2)} |")
        o.append("")
    o += ["### Replaying the real League", "",
          "Stage 1 point-in-time. Each added bot is the League's 11th member and sees only the standings published "
          "before each Match's cutoff (the real members' Points from results known by then). The real members' picks "
          "are as made; a missing Prediction scores 0. The DP assumes fitted opponents, 10 of them, and takes its "
          "Match types from the 57 archived Events. \"Whole Event\" counts the Event's later 74 Matches as still to come, "
          "which is what a live bot would face. \"Stage 1 only\" pretends the League ended after Stage 1. Sanity check "
          "only: the opponent model was fitted on these same picks.", ""]
    real_pts = []
    for horizon, r in real.items():
        real_pts = r["real_points"]
        diffs = "; ".join(f"Match {j + 1} of 33 (lgbm-ext {lp:.2f}, anchor {ap:.2f}), the DP's pick "
                          f"{'won' if ok else 'lost'}" for j, lp, ap, ok in r["differs"])
        o += [f"**{horizon}.** Points after Stage 1: plain lgbm-ext {r['bot_points'][0]} (place {r['places'][0]}), "
              f"40-60 lgbm-ext {r['bot_points'][1]} (place {r['places'][1]}), DP lgbm-ext {r['bot_points'][2]} "
              f"(place {r['places'][2]}). The DP differs from plain on {len(r['differs'])} Matches"
              + (": " + diffs if diffs else "") + ".", ""]
    o += [f"The real members' Points after Stage 1, sorted (anonymous): {', '.join(map(str, real_pts))}. A place "
          "counts the real members with more Points (ties share).", "",
          "### Compute", "", "; ".join(timings) + ".", ""]
    return "\n".join(o)


def main(argv=None):
    ap = argparse.ArgumentParser(prog="kbml.league_report")
    ap.add_argument("--device", default="cuda" if torch.cuda.is_available() else "cpu")
    ap.add_argument("--trials", type=int, default=2000)
    ap.add_argument("--replay-device", default="cpu",
                    help="the replay loops over Matches with small tensors: faster on the CPU (measured)")
    args = ap.parse_args(argv)
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    dev = args.device
    games, picks, members = L.load(log=print)
    beh, h = behaviour(games, picks, members)
    timings = []
    model = fit_model(h, dev)
    timings.append(f"opponent-model bootstrap (2000 refits): {model['seconds']:.1f} s on {dev}")
    theta = tuple(float(v) for v in model["point"])
    boot = model["boot"]
    events, ids, infos = cr.load()
    arch = {}
    for size in (10, 12):
        t = time.time()
        res, t_tab, t_rep = archive_replay(events, ids, infos, theta, boot, size, args.replay_device, args.trials)
        timings.append(f"{size} members: DP tables {t_tab:.1f} s, both replays of {args.trials} trials "
                       f"{t_rep:.1f} s on {args.replay_device}")
        for label, rows in res.items():
            arch[(size, label)] = rows
    real = real_replay(games, picks, members, events, theta, args.replay_device)
    text = markdown(beh, model, arch, real, timings, len(events), args.trials)
    write_block(text)
    js = {"model_point": model["point"].tolist(), "model_q05": np.quantile(boot, 0.05, 0).tolist(),
          "model_q95": np.quantile(boot, 0.95, 0).tolist(),
          "archive": {f"{k[0]} | {k[1]}": {f"{s} | {p}": v for (s, p), v in r.items()} for k, r in arch.items()},
          "real": real, "timings": timings}
    (paths.CACHE / "league-real-results.json").write_text(json.dumps(js, indent=1, default=float))
    print(text)
    if dev != "cpu":
        torch.cuda.empty_cache()


if __name__ == "__main__":
    main()
