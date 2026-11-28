"""The one real KallBet League (production backup, read only): member behaviour, a fitted opponent
model, and a point-in-time replay of Singapore 2026 Stage 1 with the DP and plain bots. See
CATCHUP.md, "Real League".

PRIVACY: the backup holds personal data. Nothing here prints or returns nicknames, Telegram ids or
avatars; members are "A", "B", ... by ascending user id, bots "bot 1", ...; only aggregates leave
this module. Never commit the database or per-user rows. Stage 2 (Holdout) is never read beyond
counting its predictions.
"""
from __future__ import annotations

import sqlite3
from pathlib import Path

import numpy as np
import pandas as pd

from . import archive, evaluate, features, hltv_data, paths
from . import timeutil as T

DB = Path("D:/Projects/KallBet/target/kallbet-prod/kallbet-20261127T161629Z.db")
CACHE = paths.CACHE / "league-real-stage1.parquet"
STAGE2_FROM = T.parse("2026-11-28T04:00:00Z")


def connect(db: Path = DB):
    return sqlite3.connect(f"file:{db.as_posix()}?mode=ro", uri=True)


def stage1_probs(log=print) -> pd.DataFrame:
    """Per Stage 1 Match (PandaScore id): cutoff, known finish, team names, anchor, market and
    lgbm-ext P(team A wins), winner. lgbm-ext: one fit per Event as in the walk-forward (training
    rows known strictly before the Stage opened, hyperparameters by inner walk-forward on Events
    completed by then); features of each Match as of its own cutoff (earlier Stage 1 results
    count, through the timeline)."""
    if CACHE.exists():
        return pd.read_parquet(CACHE)
    from .forecaster import EXT, TRAIN_BLOCKS
    from .pickem_data import FIXTURES, rank_history
    from .pickem_forecast import STAGE1, history_row, read
    from .anchor import Markets, Standings
    from .run import with_hltv
    from .table import load_table
    history, maps, _ = load_table(EXT, log=lambda *a: None)
    history, hltv = with_hltv(history, EXT)
    ranks = rank_history(history, hltv)
    fixture = archive.load(FIXTURES, with_hltv=False)
    standings, markets = Standings(fixture.standings), Markets(fixture)
    rows = read("panda-22378-matches.json.gz")
    stage = archive.stage_from_pandascore(STAGE1, rows)
    lan = {"online": False, "offline": True}.get(rows[0]["tournament"]["type"])
    s1 = pd.DataFrame([history_row(m, stage, standings, markets, ranks, lan) for m in stage.matches])
    s1["block"] = "scope"
    s1 = hltv_data.weekly_columns(s1, {}, hltv_data.weekly_rankings(EXT))
    frame = pd.concat([history, s1], ignore_index=True)
    for c in ("round", "maps_a", "maps_b", "vrs_points_a", "vrs_points_b", "hltv_match_id", "event_ended",
              "known_finish", "market_observed", "vrs_observed", "rw_rank_a", "rw_rank_b", "rw_points_a",
              "rw_points_b", "rw_size", "rw_known"):
        if c in frame:
            frame[c] = frame[c].astype("Int64")
    X, kinds = features.compute(frame, maps, features.resolve("p3_all"), hltv)
    cut = stage.cutoff
    h = history
    finish = h["known_finish"].astype("float").to_numpy()
    tr = (~h["is_qualifier"] & h["block"].isin(TRAIN_BLOCKS)).to_numpy() & (finish < cut)
    events = evaluate.event_infos(h)
    completed = [e for e in events.values() if e.ended is not None and e.ended < cut]
    cols = [c for g in features.SETS["p3_all"] for c in features.REGISTRY[g].columns]
    anti = np.array([kinds[c] == "anti" for c in cols])
    xt = X.loc[h["match_id"], cols].to_numpy(float)
    y = h["team_a_won"].to_numpy(float)
    cls = evaluate.MODELS["lgbm"]
    params = evaluate.select(cls, completed, h["event_id"].to_numpy(), tr, xt, y, anti, 150, 33, None,
                             h["in_scope"].to_numpy())
    fit = cls(anti, 33).fit(xt[tr], y[tr], params)
    s1["lgbm_p"] = fit.predict(X.loc[s1["match_id"], cols].to_numpy(float))
    log(f"lgbm-ext for Stage 1: {int(tr.sum())} training Matches, params {params}")
    out = s1[["match_id", "cutoff", "known_finish", "team_a", "team_b", "team_a_won", "anchor_p", "market_p",
              "lgbm_p", "format"]].copy()
    out.to_parquet(CACHE)
    return out


def load(db: Path = DB, log=print):
    """(games, picks, members): Stage 1 games with probabilities, oriented to PandaScore team A;
    picks: one row per (anonymised member, Match): picked_a, updated_at, minutes before the cutoff;
    members: anonymised id → is_bot, joined_at."""
    probs = stage1_probs(log)
    c = connect(db)
    ext = dict(c.execute("select external_id, canonical_id from external_ids "
                         "where source='pandascore' and kind='match'").fetchall())
    teams = dict(c.execute("select id, name from teams").fetchall())
    dbm = pd.read_sql("select id, stage_ordinal, team_a, team_b, winner, scheduled_start, actual_start, status "
                      "from matches", c).set_index("id")
    users = pd.read_sql("select u.id, u.is_bot, m.joined_at, m.left_at from users u "
                        "join memberships m on m.user_id = u.id", c).sort_values("id")
    labels, nb, nh = {}, 0, 0
    for r in users.itertuples():
        if r.is_bot:
            nb += 1
            labels[r.id] = f"bot {nb}"
        else:
            labels[r.id] = chr(ord("A") + nh)
            nh += 1
    members = pd.DataFrame({"member": [labels[u] for u in users.id], "is_bot": users.is_bot.astype(bool).to_numpy(),
                            "joined_at": [T.parse(j) for j in users.joined_at]})
    pr = pd.read_sql("select match_id, user_id, team_id, updated_at from predictions", c)
    c.close()
    from .teams import team_key
    games = []
    for r in probs.itertuples():
        mid = ext.get(str(r.match_id))
        if mid is None or mid not in dbm.index:
            continue
        m = dbm.loc[mid]
        if m.stage_ordinal != 1:
            continue
        names = {teams[m.team_a]: m.team_a, teams[m.team_b]: m.team_b}
        by_key = {team_key(n): t for n, t in names.items()}
        a = by_key.get(team_key(r.team_a))
        b = by_key.get(team_key(r.team_b))
        assert a is not None and b is not None and a != b, f"teams of match {mid} not mapped"
        assert (m.winner == a) == bool(r.team_a_won), f"winner of match {mid} disagrees"
        games.append({"db_id": int(mid), "team_a_db": int(a), "cutoff": int(r.cutoff),
                      "known_finish": None if pd.isna(r.known_finish) else int(r.known_finish),
                      "team_a_won": bool(r.team_a_won), "anchor_p": r.anchor_p,
                      "market_p": None if pd.isna(r.market_p) else float(r.market_p), "lgbm_p": float(r.lgbm_p),
                      "format": r.format})
    games = pd.DataFrame(games).sort_values(["cutoff", "db_id"]).reset_index(drop=True)
    gi = games.set_index("db_id")
    pr = pr[pr.match_id.isin(gi.index)].copy()
    pr["member"] = pr.user_id.map(labels)
    pr["picked_a"] = pr.team_id.to_numpy() == gi.loc[pr.match_id, "team_a_db"].to_numpy()
    upd = np.array([T.parse(u) for u in pr.updated_at], dtype=np.int64)
    pr["minutes_before_cutoff"] = (gi.loc[pr.match_id, "cutoff"].to_numpy() - upd) / 60e6
    pr = pr[["match_id", "member", "picked_a", "minutes_before_cutoff"]].rename(columns={"match_id": "db_id"})
    return games, pr.reset_index(drop=True), members


def stage2_count(db: Path = DB) -> int:
    c = connect(db)
    n = c.execute("select count(*) from predictions p join matches m on m.id = p.match_id "
                  "where m.stage_ordinal <> 1").fetchone()[0]
    c.close()
    return int(n)


# ---------------------------------------------------------------- behaviour

def standings_before(games: pd.DataFrame, picks: pd.DataFrame, cutoff: int) -> dict:
    """Published Points per member strictly before `cutoff` (results with known finish < cutoff)."""
    done = games[(games.known_finish.notna()) & (games.known_finish < cutoff)]
    pts = {m: 0 for m in picks.member.unique()}
    sub = picks[picks.db_id.isin(done.db_id)].merge(done[["db_id", "team_a_won"]], on="db_id")
    for m, ok in zip(sub.member, sub.picked_a == sub.team_a_won):
        pts[m] += int(ok)
    return pts


def annotate(games, picks, members):
    """Per pick: which side the anchor, market and lgbm-ext favour, whether the member is human,
    and whether the pick matches the published leader's pick (humans only, leader = most published
    Points before the cutoff among humans; ties: every tied leader counts)."""
    g = games.set_index("db_id")
    df = picks.merge(members[["member", "is_bot"]], on="member")
    pa = g.loc[df.db_id]
    df["anchor_fav_a"] = (pa.anchor_p >= 0.5).to_numpy()
    df["p_anchor_fav"] = np.maximum(pa.anchor_p, 1 - pa.anchor_p).to_numpy()
    df["market_fav_a"] = np.where(pa.market_p.isna(), np.nan, (pa.market_p >= 0.5).astype(float))
    df["lgbm_fav_a"] = (pa.lgbm_p >= 0.5).to_numpy()
    df["p_lgbm_fav"] = np.maximum(pa.lgbm_p, 1 - pa.lgbm_p).to_numpy()
    df["won_a"] = pa.team_a_won.to_numpy()
    df["picks_anchor_fav"] = df.picked_a == df.anchor_fav_a
    df["picks_lgbm_fav"] = df.picked_a == df.lgbm_fav_a
    df["picks_market_fav"] = np.where(df.market_fav_a.isna(), np.nan, (df.picked_a == (df.market_fav_a == 1)).astype(float))
    df["correct"] = df.picked_a == df.won_a
    return df


def boot_matches(df, stat, resamples=2000, seed=33):
    """90% interval of `stat(df)` resampling Matches (clusters of picks), stats.SeededRng draws."""
    from .stats import draws
    ids = np.array(sorted(df.db_id.unique()))
    d = draws(len(ids), resamples, seed)
    groups = {i: df[df.db_id == i] for i in ids}
    vals = np.sort([stat(pd.concat([groups[ids[k]] for k in row], ignore_index=True)) for row in d])
    return float(stat(df)), float(np.quantile(vals, 0.05)), float(np.quantile(vals, 0.95))
