#!/usr/bin/env python3
"""Aggregate the per-round pilot tables into per map x team rates (run with the venv python, e.g. python -I aggregate_pilot.py DIR).

Reads DIR/rounds-*.parquet, writes DIR/demo_rounds.parquet (all rounds), DIR/demo_team_map.parquet (one row per map x team)
and DIR/demo_team_summary.parquet (per match x team, sums over its maps).

Per team and map:
  rounds, round_wins
  pistol_played / pistol_won              first round of each half (2 per map)
  r2_after_pistol_win_opp / _won          second round of the half when the team won the pistol  ("conversion")
  r2_after_pistol_loss_opp / _won         second round of the half when the team lost the pistol (anti-eco / force break)
  full_v_full_rounds / _wins              both teams on a full buy (>= 4000 avg equipment), not a pistol round
  eco_rounds / eco_wins                   team < 1500 avg while the opponent is on a full buy
  force_rounds / force_wins               team 1500..4000 avg while the opponent is on a full buy
"""
import glob, os, sys
import pandas as pd


def long_view(r):
    t = r.assign(team=r.t_clan, opp=r.ct_clan, side="T", own_class=r.t_class, opp_class=r.ct_class, won=r.winner_side == "T")
    c = r.assign(team=r.ct_clan, opp=r.t_clan, side="CT", own_class=r.ct_class, opp_class=r.t_class, won=r.winner_side == "CT")
    cols = ["match_id", "demo_id", "map_name", "round_no", "half_no", "round_in_half", "is_pistol", "team", "opp", "side",
            "own_class", "opp_class", "won"]
    return pd.concat([t[cols], c[cols]], ignore_index=True)


def per_map(lv):
    out = []
    for (mid, did, mp, team), g in lv.groupby(["match_id", "demo_id", "map_name", "team"]):
        g = g.sort_values("round_no")
        pist = g[g.is_pistol]
        row = dict(match_id=mid, demo_id=did, map_name=mp, team=team, opp=g.opp.iloc[0], rounds=len(g), round_wins=int(g.won.sum()),
                   pistol_played=len(pist), pistol_won=int(pist.won.sum()))
        won_half = {h: bool(x.won.iloc[0]) for h, x in pist.groupby("half_no")}
        second = g[(g.round_in_half == 2) & g.half_no.isin(won_half)]
        conv = second[second.half_no.map(won_half)]
        bonus = second[~second.half_no.map(won_half).astype(bool)]
        row.update(r2_after_pistol_win_opp=len(conv), r2_after_pistol_win_won=int(conv.won.sum()),
                   r2_after_pistol_loss_opp=len(bonus), r2_after_pistol_loss_won=int(bonus.won.sum()))
        np_ = g[~g.is_pistol]
        ff = np_[(np_.own_class == "full") & (np_.opp_class == "full")]
        eco = np_[(np_.own_class == "eco") & (np_.opp_class == "full")]
        force = np_[(np_.own_class == "force") & (np_.opp_class == "full")]
        row.update(full_v_full_rounds=len(ff), full_v_full_wins=int(ff.won.sum()), eco_rounds=len(eco), eco_wins=int(eco.won.sum()),
                   force_rounds=len(force), force_wins=int(force.won.sum()))
        out.append(row)
    return pd.DataFrame(out)


def main():
    d = sys.argv[1]
    parts = [pd.read_parquet(f) for f in sorted(glob.glob(os.path.join(d, "rounds-*.parquet")))]
    parts = [p for p in parts if len(p)]
    rounds = pd.concat(parts, ignore_index=True)
    rounds.to_parquet(os.path.join(d, "demo_rounds.parquet"), index=False)
    tm = per_map(long_view(rounds))
    tm.to_parquet(os.path.join(d, "demo_team_map.parquet"), index=False)
    num = [c for c in tm.columns if c not in ("match_id", "demo_id", "map_name", "team", "opp")]
    ts = tm.groupby(["match_id", "demo_id", "team"], as_index=False)[num].sum()
    ts.to_parquet(os.path.join(d, "demo_team_summary.parquet"), index=False)
    tot = tm[num].sum()
    rate = lambda w, n: round(float(w) / float(n), 3) if n else None  # noqa: E731
    print(f"demos {tm.demo_id.nunique()} maps {tm.groupby(['match_id','map_name']).ngroups} rounds {len(rounds)}")
    print("pistol win rate (sanity, ~0.5):", rate(tot.pistol_won, tot.pistol_played))
    print("round 2 after pistol WIN:", rate(tot.r2_after_pistol_win_won, tot.r2_after_pistol_win_opp), int(tot.r2_after_pistol_win_opp))
    print("round 2 after pistol LOSS:", rate(tot.r2_after_pistol_loss_won, tot.r2_after_pistol_loss_opp), int(tot.r2_after_pistol_loss_opp))
    print("eco vs full:", rate(tot.eco_wins, tot.eco_rounds), int(tot.eco_rounds), " force vs full:", rate(tot.force_wins, tot.force_rounds), int(tot.force_rounds))
    print("full vs full rounds:", int(tot.full_v_full_rounds))


if __name__ == "__main__":
    main()
