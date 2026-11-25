"""Expand the committed source captures into an offline replay directory."""

import argparse
import gzip
import json
from pathlib import Path
import re
import shutil

FIXTURES = Path(__file__).resolve().parents[1] / "tests/fixtures"
ARCHIVE = FIXTURES / "backtest"
ALIASES = {
    "nip": "ninjasinpyjamas",
    "tl": "liquid",
    "fq": "flyquest",
    "vit": "vitality",
    "imp": "imperial",
    "gl": "gamerlegion",
    "lvg": "lynnvision",
    "mongolz": "themongolz",
    "col": "complexity",
    "c9": "cloud9",
    "navi": "natusvincere",
    "vp": "virtuspro",
    "pvision": "parivision",
    "dendelecs": "dendele",
    "chinggis": "chinggiswarriors",
    "ef": "eternalfire",
    "100t": "100thieves",
    "fnc": "fnatic",
    "gaimin": "gaimingladiators",
    "gosh": "bcgame",
    "thunderdownunder": "thunderdownunder",
}


def key(name):
    words = re.findall(r"[a-z0-9]+", name.lower())
    word = "".join(w for w in words if w not in {"team", "esports", "gaming", "clan"})
    return ALIASES.get(word, word)


def stage_input(revision, stage_name, teams):
    page = revision["slots"]["main"]["content"]
    ids = {key(n): t["id"] for t in teams for n in (t["name"], t["acronym"]) if n}
    playoffs = stage_name == "Playoffs"
    # Leaf slots only. Later Swiss pairings and bracket advancements never enter the input.
    pair_pattern = r"\|opponent1=\{\{TeamOpponent\|([^}|]+)\}\}\s*\|opponent2=\{\{TeamOpponent\|([^}|]+)"
    if playoffs:
        pairs = re.findall(r"\|R1M[1-4]=\{\{Match\s*" + pair_pattern, page, re.I)
    else:
        opening = re.split(r"====\s*Round 1\s*====", page, flags=re.I)[-1]
        opening = re.split(r"====\s*Round 2\s*====", opening, flags=re.I)[0]
        pairs = re.findall(pair_pattern, opening, re.I)
    n = 4 if playoffs else 8
    if len(pairs) < n:
        return None
    try:
        pairs = [(ids[key(a)], ids[key(b)]) for a, b in pairs[:n]]
    except KeyError as e:
        print("unmapped Liquipedia team", e)
        return None
    participants = [x for pair in pairs for x in pair]
    if len(set(participants)) != 2 * n:
        return None
    payload = {
        "teams": [{"id": i} for i in participants],
        "stage": {"name": stage_name, "kind": "playoffs" if playoffs else "swiss"},
    }
    if playoffs:
        slots = [
            {
                "rounds_before_final": 2,
                "slot": i,
                "leaf": True,
                "team_a": a,
                "team_b": b,
            }
            for i, (a, b) in enumerate(pairs)
        ]
        for r in (1, 0):
            slots.extend(
                {"rounds_before_final": r, "slot": i, "leaf": False}
                for i in range(2**r)
            )
        if "abbr/bo5" in page.lower():
            slots[-1]["format"] = "bo5"
        payload["bracket"] = {"ready": True, "slots": slots}
    else:
        seeds = re.findall(r"\|team\d+=([^|\n]+)\|temp_tie\d+=(\d+)", page)
        if not seeds:
            seeds = re.findall(r"\{\{TeamOpponent\|([^|}]+)\|tiebreaker=(\d+)", page)
        if len(seeds) == 16 and all(key(t) in ids for t, _ in seeds):
            payload["seeds"] = [
                ids[key(t)] for t, _ in sorted(seeds, key=lambda a: -int(a[1]))
            ]
        payload["all_bo3"] = bool(
            re.search(r"all (?:matches|series) are.*?(?:Bo3|best.of.three)", page, re.I)
        )
        payload["groups"] = [
            {
                "wins": 0,
                "losses": 0,
                "matches": [
                    {"team_a": {"id": a}, "team_b": {"id": b}} for a, b in pairs
                ],
            }
        ]
    return {"observed": revision["timestamp"], "payload": payload}


def prepare(out):
    out.mkdir(parents=True, exist_ok=True)
    if not (out / "panel.json").exists():
        shutil.copyfile(ARCHIVE / "panel.json", out / "panel.json")
    for source in ARCHIVE.glob("*.gz"):
        (out / source.stem).write_bytes(gzip.decompress(source.read_bytes()))
    for source in [
        *FIXTURES.glob("vrs/*.md"),
        *FIXTURES.glob("odds/oddspapi-*settled.json"),
        *FIXTURES.glob("odds/*historical*.json"),
        *ARCHIVE.glob("standings_global*.md"),
    ]:
        if source.name != "README.md":
            shutil.copyfile(source, out / source.name)
    # Resolve each source's team aliases independently of all other snapshots.
    for file in out.glob("standings_global*"):
        text = file.read_text(encoding="utf8")
        text = re.sub(
            r"^(\|\s*\d+\s*\|\s*\d+\s*\|)([^|]+)\|",
            lambda m: m[1] + key(m[2]) + "|",
            text,
            flags=re.M,
        )
        file.write_text(text, encoding="utf8")
    stages = []
    for file in sorted(out.glob("panda-*.json")):
        rows = json.loads(file.read_bytes())
        first = rows[0]
        teams = {
            o["opponent"]["id"]: o["opponent"] for m in rows for o in m["opponents"]
        }

        def team(t):
            return {"id": t["id"], "name": t["name"], "rating_name": key(t["name"])}

        completed = []
        for m in rows:
            if (
                m["status"] != "finished"
                or m["forfeit"]
                or not m["winner_id"]
                or len(m["opponents"]) != 2
            ):
                continue
            a, b = [team(o["opponent"]) for o in m["opponents"]]
            if (
                not m["scheduled_at"]
                or not m["begin_at"]
                or m["number_of_games"] not in (1, 3, 5)
            ):
                continue
            completed.append(
                {
                    "game": {
                        "id": m["id"],
                        "event_id": m["serie_id"],
                        "format": "bo" + str(m["number_of_games"]),
                        "team_a": a,
                        "team_b": b,
                        "scheduled_start": m["scheduled_at"],
                    },
                    "actual": m["begin_at"],
                    "winner": m["winner_id"],
                }
            )
        stage = first["tournament"]["name"]
        id = first["tournament_id"]
        lp = out / ("shanghai-playoff.json" if id == 15479 else f"lp-{id}.json")
        view = None
        if lp.exists():
            revision = json.loads(lp.read_bytes())["query"]["pages"][0]["revisions"][0]
            view = stage_input(revision, stage, teams.values())
        stages.append(
            {
                "name": first["serie"]["full_name"] + " / " + stage,
                "id": id,
                "teams": [team(t) for t in teams.values()],
                "cutoff": min(
                    m["begin_at"]
                    for m in rows
                    if m["status"] == "finished" and m["begin_at"]
                ),
                "complete": all(m["status"] in ("finished", "canceled") for m in rows),
                "matches": completed,
                "view": view,
            }
        )
    (out / "stages.json").write_text(json.dumps(stages), encoding="utf8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    prepare(parser.parse_args().directory)
