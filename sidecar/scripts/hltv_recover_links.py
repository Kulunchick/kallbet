#!/usr/bin/env python3
"""Liquipedia-independent PandaScore <-> HLTV Match linking (no network; reads saved files only).

Inputs : <archive>/panda-*.json.gz, <archive>/hltv/{liquipedia,matches}, derived/recovery/{pages_meta,seen_matches}.parquet
         (written by hltv_page_links.py)
Outputs under --out (default <archive>/derived/recovery):
  panda_matches.parquet      every finished PandaScore Match with its baseline HLTV link (team key pair, start +-6 h)
  links_baseline.parquet / links_composite.parquet   Match -> HLTV id, how it was linked, scores
  fetch_candidates.tsv       HLTV match ids named in saved widgets that match an unlinked Match (slug pair + date), not on disk
  recovery_summary.json      counts per block, tier, with/without Liquipedia title
"""
import argparse, glob, gzip, json, os, re
from bisect import bisect_left
from datetime import datetime, timezone

import numpy as np
import pandas as pd

NOISE = {"team", "esports", "esport", "gaming", "clan", "club", "gg"}
ALIASES = dict([("nip", "ninjasinpyjamas"), ("navi", "natusvincere"), ("tl", "liquid"), ("fq", "flyquest"),
                ("fly", "flyquest"), ("vit", "vitality"), ("imp", "imperial"), ("gl", "gamerlegion"),
                ("lvg", "lynnvision"), ("mongolz", "themongolz"), ("col", "complexity"), ("c9", "cloud9"),
                ("vp", "virtuspro"), ("pvision", "parivision"), ("pv", "parivision"), ("dendelecs", "dendele"),
                ("chinggis", "chinggiswarriors"), ("cw", "chinggiswarriors"), ("ef", "eternalfire"),
                ("100t", "100thieves"), ("fnc", "fnatic"), ("gaimin", "gaimingladiators"), ("gosh", "bcgame"),
                ("wc", "wildcard"), ("bb", "betboom"), ("bbteam", "betboom")])


def team_key(name):
    """Port of sidecar/src/team_names.rs::team_key."""
    words = [w for w in re.split(r"[^0-9a-zA-Z\u00C0-\uFFFF]+", (name or "").lower()) if w]
    kept = "".join(w for w in words if w not in NOISE)
    key = kept if kept else "".join(words)
    return ALIASES.get(key, key)


def slugify(name):
    return re.sub(r"[^a-z0-9]+", "-", (name or "").lower()).strip("-")


def ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp() if s else None


def load_panda(archive, orig):
    rows, seen = [], set()
    orig_names = {os.path.basename(f) for f in glob.glob(os.path.join(orig, "panda-*.json.gz"))}
    for f in sorted(glob.glob(os.path.join(archive, "panda-*.json.gz"))):
        name = os.path.basename(f)
        for m in json.loads(gzip.decompress(open(f, "rb").read())):
            if m["id"] in seen:
                continue
            seen.add(m["id"])
            opp = [o["opponent"] for o in m.get("opponents", []) if o.get("opponent")]
            if len(opp) != 2:
                continue
            res = {r["team_id"]: r["score"] for r in m.get("results", [])}
            t = ts(m.get("begin_at")) or ts(m.get("scheduled_at"))
            rows.append(dict(
                panda_id=m["id"], status=m["status"], start=t, scheduled=ts(m.get("scheduled_at")), begin=ts(m.get("begin_at")),
                n_games=m.get("number_of_games"), forfeit=m.get("forfeit"),
                a_id=opp[0]["id"], a_name=opp[0]["name"], b_id=opp[1]["id"], b_name=opp[1]["name"],
                a_score=res.get(opp[0]["id"]), b_score=res.get(opp[1]["id"]),
                serie_id=m["serie_id"], tier=(m["tournament"].get("tier") or ""), tournament_id=m["tournament_id"],
                league=m["league"]["name"], serie=m["serie"]["full_name"],
                is_new=name not in orig_names))
    return pd.DataFrame(rows)


def jacc(a, b):
    a, b = set(a), set(b)
    return len(a & b) / len(a | b) if a | b else 0.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--archive", default="D:/Projects/KallBet/target/research-archive-ext")
    ap.add_argument("--orig", default="D:/Projects/KallBet/target/research-archive")
    ap.add_argument("--out")
    a = ap.parse_args()
    out = a.out or os.path.join(a.archive, "derived", "recovery")
    pm = load_panda(a.archive, a.orig)
    pm = pm[pm.status == "finished"].copy()
    meta = pd.read_parquet(os.path.join(out, "pages_meta.parquet"))
    meta = meta.dropna(subset=["start_unix_ms", "team1_id", "team2_id"]).copy()
    meta["t"] = meta.start_unix_ms / 1000
    meta["k1"], meta["k2"] = meta.team1_name.map(team_key), meta.team2_name.map(team_key)
    meta["lin"] = meta.lineups.map(lambda s: {int(k): v for k, v in json.loads(s).items()} if isinstance(s, str) else {})
    pm["ka"], pm["kb"] = pm.a_name.map(team_key), pm.b_name.map(team_key)

    # series with a Liquipedia event page that held HLTV ids (approximation of "has a verified title")
    lp = os.path.join(a.archive, "hltv", "liquipedia")
    sids_event = {re.search(r"event-(\d+)", os.path.basename(f)).group(1) for f in glob.glob(os.path.join(lp, "event-*.json.gz"))}
    sids_roster = {os.path.basename(f).split("-")[1] for f in glob.glob(os.path.join(a.archive, "roster-*.json.gz"))}
    pm["has_title"] = pm.serie_id.astype(str).isin(sids_event | sids_roster)

    # 1. baseline link (what the Lab and the notes use): team-key pair (either order) and start within +-6 h
    by_pair = {}
    for r in meta.itertuples():
        by_pair.setdefault(frozenset((r.k1, r.k2)), []).append(r)
    base = {}
    for r in pm.itertuples():
        for h in by_pair.get(frozenset((r.ka, r.kb)), []):
            if abs(h.t - r.start) <= 6 * 3600:
                base.setdefault(r.panda_id, []).append(h.match_id)
    pm["base_hltv"] = pm.panda_id.map(lambda i: base.get(i, [None])[0] if len(base.get(i, [])) == 1 else None)
    pm["base_ambiguous"] = pm.panda_id.map(lambda i: len(base.get(i, [])) > 1)
    pm["base_via"] = np.where(pm.base_hltv.notna(), "unique", "")
    # same pair played twice within hours (ESL Challenger days): greedy one-to-one by series score, then |start delta|
    taken = {int(x) for x in pm.base_hltv.dropna()}
    pmi = pm.set_index("panda_id")
    meta_by_id = meta.set_index("match_id")

    def page_score(hid):
        jf = os.path.join(a.archive, "hltv", "matches", f"hltv-match-{int(hid)}.json")
        if not os.path.exists(jf):
            return None
        j = json.load(open(jf, encoding="utf8"))
        return (sum(1 for m in j["maps"] if str(m.get("winner")).lower() == "a"), sum(1 for m in j["maps"] if str(m.get("winner")).lower() == "b"),
                key_of(j["team_a"]["name"]))

    def key_of(n):
        return team_key(n)

    pairs = []
    for pid, hs in base.items():
        if len(hs) < 2:
            continue
        r = pmi.loc[pid]
        for hid in hs:
            ps = page_score(hid)
            ok = False
            if ps and r.a_score is not None:
                wa, wb, ka = ps
                exp = (r.a_score, r.b_score) if ka == r.ka else (r.b_score, r.a_score)
                ok = (wa, wb) == exp
            pairs.append((not ok, abs(meta_by_id.loc[hid].t - r.start), pid, hid))
    resolved = {}
    for bad, dt, pid, hid in sorted(pairs):
        if pid in resolved or hid in taken:
            continue
        resolved[pid] = hid
        taken.add(hid)
    for pid, hid in resolved.items():
        pm.loc[pm.panda_id == pid, ["base_hltv", "base_via"]] = [hid, "resolved"]

    # 2. composite: start +-24 h, Bo format, per-team (name key OR roster Jaccard >= 0.6)
    # panda team -> lineups seen on baseline-linked pages (the Jaccard roster of that team over time)
    meta_i = meta.set_index("match_id")
    roster = {}  # panda team id -> list of (t, frozenset players)
    for r in pm[pm.base_hltv.notna()].itertuples():
        h = meta_i.loc[int(r.base_hltv)]
        for pid, key, hid in ((r.a_id, r.ka, None), (r.b_id, r.kb, None)):
            side = 1 if h.k1 == key else 2 if h.k2 == key else None
            if side is None:
                continue
            tid = h.team1_id if side == 1 else h.team2_id
            pl = h.lin.get(int(tid))
            if pl:
                roster.setdefault(pid, []).append((h.t, frozenset(pl)))
    used = set(int(x) for x in pm.base_hltv.dropna())
    ts_sorted = meta.sort_values("t")
    tarr = ts_sorted.t.values
    cand_rows = list(ts_sorted.itertuples())

    def team_ok(pid, key, hkey, hteam_id, h):
        if key == hkey:
            return 1.0, "name"
        pl = h.lin.get(int(hteam_id))
        best = 0.0
        if pl:
            for t0, s in roster.get(pid, []):
                if abs(t0 - h.t) <= 120 * 86400:
                    best = max(best, jacc(pl, s))
        return best, "roster"

    fmt_n = {1: "bo1", 2: "bo2", 3: "bo3", 5: "bo5"}
    comp = []
    for r in pm[pm.base_hltv.isna()].itertuples():
        lo, hi = bisect_left(tarr, r.start - 24 * 3600), bisect_left(tarr, r.start + 24 * 3600 + 1)
        hits = []
        for h in cand_rows[lo:hi]:
            if h.match_id in used:
                continue
            if fmt_n.get(r.n_games) and h.format and fmt_n[r.n_games] != h.format:
                continue
            best = None
            for swap in (False, True):
                ha, hb = (h.team2_id, h.team1_id) if swap else (h.team1_id, h.team2_id)
                ka, kb = (h.k2, h.k1) if swap else (h.k1, h.k2)
                ja, ma = team_ok(r.a_id, r.ka, ka, ha, h)
                jb, mb = team_ok(r.b_id, r.kb, kb, hb, h)
                if ja >= 0.6 and jb >= 0.6 and (best is None or ja + jb > best[0]):
                    best = (ja + jb, swap, ma, mb, ja, jb)
            if best:
                hits.append((h, best))
        if len(hits) == 1:
            h, b = hits[0]
            comp.append(dict(panda_id=r.panda_id, hltv_id=int(h.match_id), swap=b[1], via=b[2] + "+" + b[3],
                             jaccard_a=b[4], jaccard_b=b[5], dt_h=(h.t - r.start) / 3600))
        elif len(hits) > 1:
            comp.append(dict(panda_id=r.panda_id, hltv_id=None, swap=None, via="ambiguous", jaccard_a=None, jaccard_b=None, dt_h=None))
    comp = pd.DataFrame(comp)
    comp_ok = comp[comp.hltv_id.notna()] if len(comp) else comp
    # a page may only serve one Match
    if len(comp_ok):
        comp_ok = comp_ok.drop_duplicates("hltv_id", keep=False)
    pm = pm.merge(comp_ok[["panda_id", "hltv_id", "via", "jaccard_a", "jaccard_b", "dt_h", "swap"]] if len(comp_ok) else
                  pd.DataFrame(columns=["panda_id", "hltv_id", "via", "jaccard_a", "jaccard_b", "dt_h", "swap"]),
                  on="panda_id", how="left")

    # score check of the composite links against the page's own map tally
    sc_ok = sc_n = 0
    if len(comp_ok):
        for r in pm[pm.hltv_id.notna()].itertuples():
            jf = os.path.join(a.archive, "hltv", "matches", f"hltv-match-{int(r.hltv_id)}.json")
            if not os.path.exists(jf):
                continue
            j = json.load(open(jf, encoding="utf8"))
            wa = sum(1 for m in j["maps"] if str(m.get("winner")).lower() == "a")
            wb = sum(1 for m in j["maps"] if str(m.get("winner")).lower() == "b")
            if r.a_score is None or (wa + wb) == 0:
                continue
            pa, pb = (r.b_score, r.a_score) if r.swap else (r.a_score, r.b_score)
            sc_n += 1
            sc_ok += (wa, wb) == (pa, pb)

    # 3. fetch candidates from the widgets of saved pages: slug pair + date window, id not on disk
    seen = pd.read_parquet(os.path.join(out, "seen_matches.parquet"))
    ondisk = set(meta.match_id) | {int(x) for x in meta_i.index}
    s = seen[~seen.seen_id.isin(ondisk)].drop_duplicates("seen_id").copy()
    ids_known = meta.sort_values("match_id")[["match_id", "t"]]
    ik, it = ids_known.match_id.values, ids_known.t.values
    s["est_t"] = np.interp(s.seen_id.values, ik, it)  # id -> start (monotone enough for +-days)
    s["est_t"] = np.where(s.unix_ms.notna(), s.unix_ms / 1000, s.est_t)
    s["exact"] = s.unix_ms.notna()
    s["slug_pair"] = s.slug.str.split("-vs-", n=1)
    un = pm[pm.base_hltv.isna() & pm.hltv_id.isna()]
    un_idx = {}
    for r in un.itertuples():
        un_idx.setdefault((slugify(r.a_name), slugify(r.b_name)), []).append(r)
        un_idx.setdefault((slugify(r.b_name), slugify(r.a_name)), []).append(r)
    cands = []
    for r in s.itertuples():
        if not isinstance(r.slug_pair, list) or len(r.slug_pair) != 2:
            continue
        left, rest = r.slug_pair
        for (sa, sb), lst in un_idx.items():
            if left == sa and (rest == sb or rest.startswith(sb + "-")):
                win = 12 * 3600 if r.exact else 5 * 86400
                for pmrow in lst:
                    if abs(r.est_t - pmrow.start) <= win:
                        cands.append(dict(seen_id=r.seen_id, slug=r.slug, exact=r.exact, panda_id=pmrow.panda_id,
                                          tier=pmrow.tier, serie_id=pmrow.serie_id, start=pmrow.start))
    cands = pd.DataFrame(cands)
    if len(cands):
        cands = cands.drop_duplicates(["seen_id", "panda_id"])
        nper = cands.groupby("seen_id").panda_id.nunique()
        cands["ambiguous_seen"] = cands.seen_id.map(nper) > 1

    os.makedirs(out, exist_ok=True)
    cands.to_parquet(os.path.join(out, "fetch_candidates.parquet"), index=False) if len(cands) else None
    if len(cands):
        ids = sorted(set(cands[~cands.ambiguous_seen].seen_id), reverse=True)
        with open(os.path.join(out, "fetch_candidates.tsv"), "w", newline="\n") as f:
            f.write("match_id\turl\n")
            for i in ids:
                f.write(f"{i}\thttps://www.hltv.org/matches/{i}/match\n")

    def block(x):
        return np.where(x.tier.isin(["s", "a"]), np.where(x.start < ts("2024-08-01T00:00:00Z"), "S/A 2023-09..2024-07", "S/A 2024-08+"),
                        np.where(x.tier == "b", "B", "C"))
    pm["block"] = block(pm)
    pm["linked"] = np.where(pm.base_hltv.notna(), "baseline", np.where(pm.hltv_id.notna(), "composite", "none"))
    pm["hltv_match_id"] = pm.base_hltv.fillna(pm.hltv_id)
    pm.to_parquet(os.path.join(out, "panda_matches.parquet"), index=False)
    lk = pm[pm.hltv_match_id.notna()][["panda_id", "hltv_match_id", "linked", "base_via", "via", "jaccard_a", "jaccard_b", "dt_h"]].copy()
    lk["hltv_match_id"] = lk.hltv_match_id.astype(int)
    lk.to_csv(os.path.join(out, "links.tsv"), sep="	", index=False)
    summ = {
        "finished_matches": int(len(pm)),
        "series_total": int(pm.serie_id.nunique()),
        "series_without_title": int(pm[~pm.has_title].serie_id.nunique()),
        "baseline_linked_unique": int((pm.base_via == "unique").sum()),
        "baseline_linked_resolved_same_pair_same_day": int((pm.base_via == "resolved").sum()),
        "baseline_linked": int(pm.base_hltv.notna().sum()),
        "baseline_ambiguous_before_resolution": int(pm.base_ambiguous.sum()),
        "composite_linked": int((pm.linked == "composite").sum()),
        "composite_ambiguous": int((comp.via == "ambiguous").sum()) if len(comp) else 0,
        "composite_score_check": [sc_ok, sc_n],
        "by_block": pm.groupby(["block", "linked"]).size().unstack(fill_value=0).to_dict("index"),
        "no_title_series_linked": pm[~pm.has_title].groupby("linked").size().to_dict(),
        "fetch_candidates": int(len(set(cands[~cands.ambiguous_seen].seen_id))) if len(cands) else 0,
        "fetch_candidates_exact_date": int(cands[~cands.ambiguous_seen & cands.exact].seen_id.nunique()) if len(cands) else 0,
        "candidates_by_tier": cands[~cands.ambiguous_seen].groupby("tier").seen_id.nunique().to_dict() if len(cands) else {},
    }
    json.dump(summ, open(os.path.join(out, "recovery_summary.json"), "w"), indent=1, default=int)
    print(json.dumps(summ, indent=1, default=int))


if __name__ == "__main__":
    main()
