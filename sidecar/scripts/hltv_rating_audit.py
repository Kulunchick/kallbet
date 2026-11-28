#!/usr/bin/env python3
"""Rating 3.0 audit from the saved HLTV pages (no network). Input: output of hltv_stats_extract.py.

Writes under --dir (the same directory):
  column_inventory.csv            header signature x match-era: pages, tables, first/last match, label
  column_coverage_by_quarter.csv  share of real values per column, per side, per match quarter
  formula_stability.csv           OLS of Rating 3.0 on its components per half-year (one formula => flat coefficients)
  hltv_player_match_stats.parquet the safe per player x Match x scope x side table (see audit text)
"""
import argparse, glob, json, os
import numpy as np
import pandas as pd

ARCHIVE_DEFAULT = "D:/Projects/KallBet/target/research-archive-ext"


def num(s):
    return pd.to_numeric(s, errors="coerce")


def pct(s):
    return num(s.str.replace("%", "", regex=False).str.replace("+", "", regex=False))


def load(d, archive):
    st = pd.read_parquet(os.path.join(d, "player_stats_all_columns.parquet"))
    hd = pd.read_parquet(os.path.join(d, "page_headers.parquet"))
    pg = pd.read_parquet(os.path.join(d, "pages.parquet"))
    pg["started_at"] = pd.to_datetime(pg.started_unix_ms, unit="ms", utc=True)
    rounds = {}
    for f in glob.glob(os.path.join(archive, "hltv", "matches", "hltv-match-*.json")):
        j = json.load(open(f, encoding="utf8"))
        tot = 0
        for m in j["maps"]:
            if m.get("team_a_score") is not None and m.get("team_b_score") is not None and m.get("played"):
                r = m["team_a_score"] + m["team_b_score"]
                tot += r
                if m.get("mapstats_id"):
                    rounds[(j["match_id"], str(m["mapstats_id"]))] = r
        rounds[(j["match_id"], "all")] = tot
    return st, hd, pg, rounds


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", required=True)
    ap.add_argument("--archive", default=ARCHIVE_DEFAULT)
    a = ap.parse_args()
    st, hd, pg, rounds = load(a.dir, a.archive)
    st = st.merge(pg[["match_id", "started_at", "fetched_at"]], on="match_id")
    st["q"] = st.started_at.dt.tz_localize(None).dt.to_period("Q").astype(str)

    # 1. header signatures (team name removed)
    hd["sig"] = hd.header_sig.str.replace(r"^players=[^|]*", "players=*", regex=True)
    hd = hd.merge(pg[["match_id", "started_at"]], on="match_id")
    inv = hd.groupby(["sig", "rating_label"]).agg(
        tables=("match_id", "size"), pages=("match_id", "nunique"),
        first_match=("started_at", "min"), last_match=("started_at", "max")).reset_index()
    inv.to_csv(os.path.join(a.dir, "column_inventory.csv"), index=False)

    # 2. real-value coverage per column (sentinels: null-null, 0.0, 0.0%, empty)
    st["has_ekd"] = ~st["kd:eco"].isin(["null-null", "0-0"]) & st["kd:eco"].notna()
    st["has_eadr"] = num(st["adr:eco"]).fillna(0) > 0
    st["has_ekast"] = pct(st["kast:eco"].fillna("")).fillna(0) > 0
    st["sw"] = pct(st["roundSwing"].fillna(""))
    st["rt"] = num(st.rating)
    cov = st.groupby(["side", "q", "rating_label"]).agg(
        rows=("rt", "size"), rating_mean=("rt", "mean"), eK_eD=("has_ekd", "mean"), eADR=("has_eadr", "mean"),
        eKAST=("has_ekast", "mean"), swing=("sw", lambda s: s.notna().mean()),
        swing_mean=("sw", "mean"), swing_sd=("sw", "std")).round(4).reset_index()
    cov.to_csv(os.path.join(a.dir, "column_coverage_by_quarter.csv"), index=False)

    # 3. formula stability: Rating 3.0 ~ KPR + DPR + KAST + ADR + swing, per-map overall rows, per half-year
    x = st[(st.side == "all") & (st.scope != "all") & (st.rating_label == "3.0")].copy()
    x["rounds"] = [rounds.get((m, s)) for m, s in zip(x.match_id, x.scope)]
    x = x.dropna(subset=["rounds"])
    k = x["kd:trad"].str.split("-", expand=True).astype(float)
    x["kpr"], x["dpr"] = k[0] / x.rounds, k[1] / x.rounds
    x["kast"] = pct(x["kast:trad"]) / 100
    x["adr"] = num(x["adr:trad"]) / 100
    x["swf"] = x.sw / 100
    x["half"] = x.started_at.dt.year.astype(str) + "H" + ((x.started_at.dt.month > 6) + 1).astype(str)
    out = []
    for h, g in x.groupby("half"):
        X = np.column_stack([np.ones(len(g)), g.kpr, g.dpr, g.kast, g.adr, g.swf])
        y = g.rt.values
        b = np.linalg.lstsq(X, y, rcond=None)[0]
        res = y - X @ b
        out.append(dict(half=h, n=len(g), intercept=b[0], kpr=b[1], dpr=b[2], kast=b[3], adr=b[4], swing=b[5],
                        r2=1 - (res ** 2).sum() / ((y - y.mean()) ** 2).sum(), rmse=np.sqrt((res ** 2).mean())))
    pd.DataFrame(out).round(4).to_csv(os.path.join(a.dir, "formula_stability.csv"), index=False)

    # 4. the safe table
    s = st.copy()
    kd = s["kd:trad"].str.split("-", expand=True).astype(float)
    ekd = s["kd:eco"].where(s.has_ekd).str.split("-", expand=True)
    is3 = s.rating_label == "3.0"
    t = pd.DataFrame({
        "match_id": s.match_id, "started_at": s.started_at.dt.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fetched_at": s.fetched_at, "scope": s.scope, "map_name": s.map_name, "side": s.side,
        "team_id": s.team_id, "player_id": s.player_id, "nickname": s.nickname,
        "rating_version": s.rating_label,
        "map_rounds": [rounds.get((m, c)) for m, c in zip(s.match_id, s.scope)],
        "kills": kd[0].astype("Int64"), "deaths": kd[1].astype("Int64"),
        "adr": num(s["adr:trad"]), "kast_pct": pct(s["kast:trad"]),
        # formula-dependent, Rating 3.0 era only (matches from 2023-10-16); null for Rating 2.0 pages
        "rating3": s.rt.where(is3), "swing_pct": s.sw.where(is3),
        "ekills": num(ekd[0]).where(is3 & s.has_ekd).astype("Int64"), "edeaths": num(ekd[1]).where(is3 & s.has_ekd).astype("Int64"),
        "eadr": num(s["adr:eco"]).where(is3 & s.has_eadr), "ekast_pct": pct(s["kast:eco"].fillna("")).where(is3 & s.has_ekast),
    })
    t.to_parquet(os.path.join(a.dir, "hltv_player_match_stats.parquet"), index=False)
    print("safe table rows", len(t), "matches", t.match_id.nunique())


if __name__ == "__main__":
    main()
