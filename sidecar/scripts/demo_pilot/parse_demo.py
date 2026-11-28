#!/usr/bin/env python3
"""Demo pilot: parse one CS2 .dem with demoparser2 into a per-round economy table. Run as:  python -I parse_demo.py DEM OUT.parquet [--match-id N --demo-id N]

One row per round: freeze-end equipment value of both teams, buy class, winner, pistol / second-round flags.
Nothing about the demo is kept except this table (the caller deletes the .dem).

Buy classes (average round-start equipment value per player; CS2 prices):
  pistol  round 1 and round 13 (MR12 halves)
  full    avg >= 4000   (rifle + armour + utility)
  eco     avg <  1500
  force   anything in between
"""
import argparse, os, sys
import pandas as pd
from demoparser2 import DemoParser

FULL, ECO = 4000, 1500


def buy_class(avg, pistol):
    if pistol:
        return "pistol"
    return "full" if avg >= FULL else "eco" if avg < ECO else "force"


def parse(path, match_id=None, demo_id=None):
    p = DemoParser(path)
    map_name = p.parse_header().get("map_name")
    fe = p.parse_event("round_freeze_end")
    re_ = p.parse_event("round_end")
    if fe is None or len(fe) == 0 or re_ is None or len(re_) == 0:
        return pd.DataFrame()
    # round_end rows: 'round' (1-based), 'winner' ('T'/'CT'), 'reason'; row 0 is the warm-up marker
    re_ = re_[(re_["round"] >= 1) & re_.winner.notna()].sort_values("tick").drop_duplicates("round", keep="last")
    ft = sorted(fe.tick.tolist())
    pairs = []
    for r in re_.itertuples():
        before = [t for t in ft if t < r.tick]
        if before:
            pairs.append((int(r.round), before[-1], int(r.tick), r.winner, r.reason))
    if not pairs:
        return pd.DataFrame()
    props = ["team_num", "team_clan_name", "current_equip_value", "total_rounds_played"]
    df = p.parse_ticks(props, ticks=sorted({x[1] for x in pairs}))
    rows = []
    for rnd, tf, te, winner, reason in sorted(pairs, key=lambda x: x[2]):
        g = df[(df.tick == tf) & df.team_num.isin([2, 3])]
        t, ct = g[g.team_num == 2], g[g.team_num == 3]
        if len(t) == 0 or len(ct) == 0:
            continue
        rows.append(dict(
            match_id=match_id, demo_id=demo_id, map_name=map_name, event_round=rnd, freeze_tick=tf, end_tick=te,
            reason=reason, winner_side=winner, t_clan=t.team_clan_name.iloc[0], ct_clan=ct.team_clan_name.iloc[0],
            t_equip=float(t.current_equip_value.mean()), ct_equip=float(ct.current_equip_value.mean()),
            t_n=len(t), ct_n=len(ct), rounds_played_before=int(g.total_rounds_played.max())))
    out = pd.DataFrame(rows)
    if out.empty:
        return out
    # A tech-pause restore replays a round: the same 'rounds played before' appears twice and only the last
    # attempt counts. The event's own 'round' counter does not skip the voided attempt, so it is not used.
    out["round_no"] = out.rounds_played_before + 1
    out = out.drop_duplicates("round_no", keep="last").sort_values("round_no").reset_index(drop=True)
    # Pistol rounds are found from the side swap (the T clan changes): the first round of the map and of each later half.
    swap = out.t_clan != out.t_clan.shift()
    out["half_start"] = swap | (out.round_no == out.round_no.min())
    out["half_no"] = out.half_start.cumsum()
    out["round_in_half"] = out.groupby("half_no").cumcount() + 1
    pistol = out.half_start & (out.half_no <= 2)
    out["is_pistol"] = pistol
    out["t_class"] = [buy_class(e, pi) for e, pi in zip(out.t_equip, pistol)]
    out["ct_class"] = [buy_class(e, pi) for e, pi in zip(out.ct_equip, pistol)]
    out["winner_clan"] = out.t_clan.where(out.winner_side == "T", out.ct_clan)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dem")
    ap.add_argument("out")
    ap.add_argument("--match-id", type=int)
    ap.add_argument("--demo-id", type=int)
    a = ap.parse_args()
    df = parse(a.dem, a.match_id, a.demo_id)
    df.to_parquet(a.out, index=False)
    print(os.path.basename(a.dem), "rounds", len(df), "map", df.map_name.iloc[0] if len(df) else None)


if __name__ == "__main__":
    main()
