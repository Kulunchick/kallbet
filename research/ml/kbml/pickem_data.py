"""Stage manifests, point-in-time inputs, and walk-forward fitting for the Pick'em replay."""

import gzip
import json
import re
from dataclasses import dataclass
from itertools import combinations

import numpy as np
import pandas as pd
from scipy.special import expit

from . import archive, evaluate, features, paths
from . import timeutil as T
from .anchor import Markets, Standings, anchor
from .hltv_data import RankHistory, end_of_day
from .run import with_hltv
from .table import is_qualifier, load_table
from .teams import team_key

FIXTURES = paths.REPO / "sidecar/tests/fixtures/backtest"
# PandaScore tournaments/22378/matches, final nine matches by ascending ID, winner first.
LAST_RESULTS = (
    ("Heroic", "FaZe"),
    ("Eternal Fire", "M80"),
    ("PVISION", "Lynn Vision"),
    ("100 Thieves", "9z"),
    ("Liquid", "Imperial"),
    ("Wildcard", "Chinggis Warriors"),
    ("Lynn Vision", "Heroic"),
    ("Imperial", "Wildcard"),
    ("100 Thieves", "M80"),
)


@dataclass
class Stage:
    id: int
    title: str
    cutoff: int
    teams: list
    opening: list
    formats: dict
    truth: np.ndarray
    published: bool
    lan: bool | None
    team_ids: list
    seed: np.ndarray | None = None


def wiki_order(stage, teams):
    path = FIXTURES / f"lp-{stage.id}.json.gz"
    if not path.exists():
        return None
    revision = json.loads(gzip.decompress(path.read_bytes()))["query"]["pages"][0]["revisions"][0]
    if T.parse(revision["timestamp"]) >= stage.cutoff:
        return None
    content = revision["slots"]["main"]["content"]
    entries = re.findall(r"TeamOpponent\|([^|{}]+)\|tiebreaker=(\d+)", content)
    if len(entries) != 16:
        fields = dict(re.findall(r"\|(team\d+|temp_tie\d+)=([^|}\n]+)", content))
        entries = [
            (fields.get(f"team{i}", ""), fields[f"temp_tie{i}"])
            for i in range(1, 17)
            if f"temp_tie{i}" in fields
        ]
    by_key = {team_key(t): t for t in teams}
    names = [by_key.get(team_key(name.strip())) for name, rank in sorted(entries, key=lambda x: -int(x[1]))]
    return names if len(names) == 16 and set(names) == set(teams) else None


def manifest(stage, directory):
    rows = []
    for path in directory.glob(f"panda-{stage.id}*.json.gz"):
        rows.extend(json.loads(gzip.decompress(path.read_bytes())))
    rounds = []
    for r in rows:
        m = re.match(r"Round ([1-5]):", r["name"])
        if m and r["winner_id"] and len(r["opponents"]) == 2:
            sides = [o["opponent"] for o in r["opponents"]]
            a, b = (s["name"] for s in sides)
            winner = next(s["name"] for s in sides if s["id"] == r["winner_id"])
            rounds.append((int(m[1]), a, b, winner, r["number_of_games"], r["id"]))
    if stage.id == 22378:
        rounds += [(4 if i < 6 else 5, a, b, a, 3, 1816921 + i) for i, (a, b) in enumerate(LAST_RESULTS)]
    if len(rounds) != 33 or sum(r[0] == 1 for r in rounds) != 8:
        return None
    opening = sorted([r for r in rounds if r[0] == 1], key=lambda r: r[-1])
    names = sorted({t for r in opening for t in r[1:3]})
    if len(names) != 16:
        return None
    published = wiki_order(stage, names)
    names = published or names
    w, l = np.zeros(16, int), np.zeros(16, int)
    formats = {}
    for rnd in range(1, 6):
        for _, a, b, winner, fmt, _ in [r for r in rounds if r[0] == rnd]:
            i, j = names.index(a), names.index(b)
            assert (w[i], l[i]) == (w[j], l[j]) and w[i] + l[i] == rnd - 1
            bucket = (w[i], l[i])
            assert fmt in (1, 3) and formats.setdefault(bucket, fmt) == fmt
            win, lose = (i, j) if a == winner else (j, i)
            w[win] += 1
            l[lose] += 1
    truth = np.select([(w == 3) & (l == 0), (l == 3) & (w == 0), (w == 3) & (l > 0)], [0, 1, 2], default=3)
    assert np.bincount(truth).tolist() == [2, 2, 6, 6]
    ids = {t.name: t.id for t in stage.teams.values()}
    return Stage(
        stage.id,
        f"{stage.event} / {stage.name}",
        stage.cutoff,
        names,
        [(names.index(r[1]), names.index(r[2])) for r in opening],
        formats,
        truth,
        bool(published),
        {"online": False, "offline": True}.get(rows[0]["tournament"]["type"]),
        [ids[t] for t in names],
    )


def rank_history(history, hltv):
    """Dated HLTV rank badges of every archived match page, by team key."""
    ranks = RankHistory()
    for row in history.itertuples():
        pre = hltv[0].get(row.match_id)
        if pre:
            for key, badge in zip((row.team_a_key, row.team_b_key), pre["ranks"]):
                if badge:
                    ranks.add(key, end_of_day(badge["date"]), badge["rank"])
    ranks.finish()
    return ranks


def forecast_all():
    history, maps, _ = load_table()
    history, hltv = with_hltv(history, paths.ARCHIVE)
    ranks = rank_history(history, hltv)
    stages, skipped, synthetic, metadata = {}, [], [], {}
    for directory in (paths.ARCHIVE, FIXTURES):
        data = archive.load(directory, with_hltv=False)
        standings, markets = Standings(data.standings), Markets(data)
        for replay in data.stages:
            if (directory == FIXTURES) != (replay.id == 22378):
                continue
            if not any(m.round == 5 for m in replay.matches) and replay.id != 22378:
                continue
            if is_qualifier(replay.event):
                skipped.append(f"{replay.event} / {replay.name}")
                continue
            if not replay.complete and replay.id != 22378:
                continue
            stage = manifest(replay, directory)
            if stage is None:
                raise ValueError(f"Unresolved Swiss candidate {replay.id}: {replay.event}")
            venue = (
                history.loc[(history.stage_id == stage.id) & (history["round"] == 1), "hltv_is_lan"]
                .dropna()
                .unique()
            )
            if len(venue) == 1:
                stage.lan = bool(venue[0])
            stages[stage.id] = stage
            lines = {}
            for m in replay.matches:
                if m.round != 1:
                    continue
                quote = markets.quote(m, stage.cutoff)
                if quote:
                    lines[m.team_a.name, m.team_b.name] = quote.p
                    lines[m.team_b.name, m.team_a.name] = 1 - quote.p
            team_ranks = [ranks.at(team_key(t), stage.cutoff) for t in stage.teams]
            metadata[stage.id] = (team_ranks, len(lines) // 2)
            for fmt in ("bo1", "bo3"):
                for a, b in combinations(stage.teams, 2):
                    rating = standings.estimate(a, b, fmt, stage.cutoff)
                    market = lines.get((a, b)) if fmt == f"bo{stage.formats[0, 0]}" else None
                    vr = rating[1] if rating else None
                    blended = anchor(market, vr)
                    row = {
                        "match_id": -len(synthetic) - 1,
                        "stage_id": stage.id,
                        "cutoff": stage.cutoff,
                        "known_finish": None,
                        "team_a_key": team_key(a),
                        "team_b_key": team_key(b),
                        "format": fmt,
                        "hltv_is_lan": stage.lan,
                        "market_p": market,
                        "vrs_series_p": vr,
                        "vrs_map_p": rating[0] if rating else None,
                        "anchor_p": 0.5 if blended is None else blended,
                        "vrs_points_a": rating[3] if rating else None,
                        "vrs_points_b": rating[4] if rating else None,
                    }
                    for suffix, t in (("a", a), ("b", b)):
                        for col, days in (("hltv_rank", 0), ("hltv_rank28", 28), ("hltv_rank84", 84)):
                            row[f"{col}_{suffix}"] = ranks.at(team_key(t), stage.cutoff - days * T.DAY)
                    synthetic.append(row)
    future = pd.DataFrame(synthetic)
    frame = pd.concat([history, future], ignore_index=True)
    groups = features.resolve("full")
    x, kinds = features.compute(frame, maps, groups, hltv)
    baseline = [c for g in features.resolve("baseline") for c in features.REGISTRY[g].columns]
    events = evaluate.event_infos(history)
    identifiers = history.event_id.to_numpy()
    targets = history.team_a_won.to_numpy(float)
    for stage in sorted(stages.values(), key=lambda s: s.cutoff):
        rows = future[future.stage_id == stage.id]
        forecasts = {"Anchor": rows.anchor_p.to_numpy(), "VRS": rows.vrs_series_p.fillna(0.5).to_numpy()}
        if not stage.published:
            a, b = np.triu_indices(16, 1)
            matrix = np.zeros((16, 16))
            matrix[a, b] = forecasts["Anchor"][:120]
            matrix[b, a] = 1 - matrix[a, b]
            stage.seed = np.argsort(np.lexsort((stage.team_ids, -matrix.sum(axis=1).round(12))))
        veto = x.loc[rows.match_id]
        forecasts["Anchor + veto"] = np.where(
            (rows.format == "bo1") & (veto.mp_vetoes_min.to_numpy() > 0),
            expit(veto.mp_series_logit.to_numpy()),
            forecasts["Anchor"],
        )
        tr = ((history.known_finish < stage.cutoff) & ~history.is_qualifier).fillna(False).to_numpy(bool)
        # Give inner validation only fully completed prior events; the final fit can also
        # use a completed match in the same event's earlier stage.
        completed = [e for e in events.values() if e.ended is not None and e.ended < stage.cutoff]
        for label, cols in (("baseline", baseline), ("all", list(kinds))):
            train = x.loc[history.match_id, cols].to_numpy(float)
            anti = np.array([kinds[c] == "anti" for c in cols])
            for name in ("logreg", "lgbm"):
                cls = evaluate.MODELS[name]
                pred = forecasts["Anchor"]
                if tr.sum() >= 150:
                    params = evaluate.select(cls, completed, identifiers, tr, train, targets, anti, 150, 33)
                    model = cls(anti, 33).fit(train[tr], targets[tr], params)
                    pred = model.predict(x.loc[rows.match_id, cols].to_numpy(float))
                forecasts[f"{name} {label}"] = pred
        yield stage, forecasts, metadata[stage.id], int(tr.sum()), skipped
