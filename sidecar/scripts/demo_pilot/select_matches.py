#!/usr/bin/env python3
"""Pick the pilot Matches: recent tier S/A, non-qualifier, finished, Bo3, with a demo link on the saved HLTV page.
Newest first, at most 2 Matches per event so the sample is not one tournament.
Writes <out>/selection.tsv (match_id, demo_id, started_at, event, teams, format, panda_id) and a ids file for the collector.
"""
import argparse, os
import pandas as pd


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--recovery", default="D:/Projects/KallBet/target/research-archive-ext/derived/recovery")
    ap.add_argument("--out", default="D:/Projects/KallBet/target/research-archive-ext/derived/demo-pilot")
    ap.add_argument("--n", type=int, default=20)
    a = ap.parse_args()
    pm = pd.read_parquet(os.path.join(a.recovery, "panda_matches.parquet"))
    meta = pd.read_parquet(os.path.join(a.recovery, "pages_meta.parquet"))
    x = pm[pm.tier.isin(["s", "a"]) & pm.base_hltv.notna()].copy()
    x["event"] = (x.league + " " + x.serie).str.lower()
    x = x[~x.event.str.contains("rmr|qualifier")]
    x = x.merge(meta[["match_id", "demo_link", "format", "event_name", "has_stats", "start_unix_ms"]],
                left_on="base_hltv", right_on="match_id")
    x = x[x.demo_link.notna() & x.has_stats & (x.format == "bo3")].sort_values("start", ascending=False)
    x["demo_id"] = x.demo_link.str.extract(r"/download/demo/(\d+)")[0].astype(int)
    x["rank_in_event"] = x.groupby("event_name").cumcount()
    pick = x[x.rank_in_event < 2].sort_values("start", ascending=False).head(a.n)  # recent, at most 2 per event
    os.makedirs(a.out, exist_ok=True)
    pick["started_at"] = pd.to_datetime(pick.start, unit="s", utc=True).dt.strftime("%Y-%m-%dT%H:%M:%SZ")
    pick["teams"] = pick.a_name + " vs " + pick.b_name
    cols = ["match_id", "demo_id", "started_at", "event_name", "teams", "format", "panda_id", "tier"]
    pick[cols].to_csv(os.path.join(a.out, "selection.tsv"), sep="\t", index=False)
    pick[["demo_id"]].to_csv(os.path.join(a.out, "demo_ids.tsv"), sep="\t", index=False, header=False)
    print(pick[cols].to_string())


if __name__ == "__main__":
    main()
