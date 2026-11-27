"""Parity check: the Python anchor on the research snapshot against the Rust report.

`uv run python -m kbml.parity` compares, on the scored Matches (non-qualifier, training side):
the number of scored Matches, of Matches with the anchor and with a market line; the anchor's
mean log-loss, Brier and accuracy (report.json: `learned_weights.baseline`, which is paired on
every anchor Match, and `selection.anchor_log_loss`); and every Event's id, start and Match count.
"""
from __future__ import annotations

import json
import sys

import numpy as np

from . import paths, stats
from . import timeutil as T
from .table import load_table

TOL = 1e-9


def check(archive=paths.ARCHIVE, report=paths.LAB_REPORT, log=print) -> bool:
    r = json.loads(report.read_text(encoding="utf-8"))
    m, _, _ = load_table(archive, log=lambda *a: None)
    scored = m[~m["is_qualifier"]]
    a = scored[scored["anchor_p"].notna()]
    p, y = a["anchor_p"].to_numpy(float), a["team_a_won"].to_numpy(bool)
    mine = {"log_loss": float(stats.log_loss(p, y).mean()), "brier": float(stats.brier(p, y).mean()),
            "accuracy": float(stats.correct(p, y).mean())}
    lw = next(c for c in r["candidates"] if c["name"] == "learned_weights")
    ok = True

    def cmp(label, got, want, tol=0.0):
        nonlocal ok
        good = abs(got - want) <= tol
        ok &= good
        log(f"{'ok ' if good else 'BAD'} {label:<34} python {got!r:<22} rust {want!r}")

    cmp("scored Matches", len(scored), sum(e["matches"] for e in r["events"]))
    cmp("anchor Matches", len(a), lw["paired"])
    cmp("anchor Matches (selection)", len(a), r["selection"]["matches"])
    cmp("Matches with a market line", int(a["market_p"].notna().sum()), lw["market"]["matches"])
    cmp("anchor log-loss", mine["log_loss"], lw["baseline"]["log_loss"], TOL)
    cmp("anchor log-loss (selection)", mine["log_loss"], r["selection"]["anchor_log_loss"], TOL)
    cmp("anchor Brier", mine["brier"], lw["baseline"]["brier"], TOL)
    cmp("anchor accuracy", mine["accuracy"], lw["baseline"]["accuracy"], TOL)
    tiers = scored["tier"].value_counts().to_dict()
    for t, n in r["tier_view"]["matches_by_tier"].items():
        cmp(f"tier {t} Matches", tiers.get(t, 0), n)
    q = m[m["is_qualifier"]].groupby("event_id").size().to_dict()
    for e in r["qualifiers"]["excluded"]:
        cmp(f"qualifier {e['event_id']} Matches", q.get(e["event_id"], 0), e["matches"])
    ev = scored.groupby("event_id").agg(begins=("event_begins", "min"), n=("match_id", "size"))
    bad = []
    for e in r["events"]:
        got = ev.loc[e["id"]] if e["id"] in ev.index else None
        if got is None or got["n"] != e["matches"] or T.iso(int(got["begins"])) != e["begins"]:
            bad.append(e["id"])
    ok &= not bad and len(ev) == len(r["events"])
    log(f"{'ok ' if not bad else 'BAD'} Events (id, start, Matches)       {len(ev)} python, {len(r['events'])} rust"
        + (f"; differing {bad}" if bad else ""))
    log("PARITY " + ("OK" if ok else "FAILED"))
    return ok


if __name__ == "__main__":
    sys.exit(0 if check() else 1)
