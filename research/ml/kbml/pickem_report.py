"""Render stage-level Swiss replay results and fixed-seed stage bootstrap intervals."""

from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor
from functools import cache

import numpy as np

from . import timeutil as T
from .pickem_check import evaluate_strategy, scores

BASE_ROLES = np.repeat(np.arange(4), [2, 2, 6, 6])


def random_reference():
    @cache
    def count(i, remaining, correct):
        if i == 16:
            return int(correct >= 5), 1
        passes = total = 0
        for role, n in enumerate(remaining):
            if n:
                rest = list(remaining)
                rest[role] -= 1
                p, t = count(i + 1, tuple(rest), correct + int(role != 3 and role == BASE_ROLES[i]))
                passes += p
                total += t
        return passes, total

    return count(0, (2, 2, 6, 6), 0)


def baseline_rows(stage, ranks):
    # Keep #1–6 in ordinary advance as requested. Complete the four other slots by rank.
    order = sorted(range(16), key=lambda i: (ranks[i] or 999, i))
    hltv = np.full(16, 3)
    hltv[order[:6]], hltv[order[6:8]], hltv[order[-2:]] = 2, 0, 1
    random = np.random.default_rng([33, stage.id]).permutation(BASE_ROLES)
    return [
        (name, "", roles, int(scores(stage.truth, roles)), None, None, None)
        for name, roles in (("HLTV top six", hltv), ("Random slip", random))
    ]


def render(rows, results, runs):
    groups = defaultdict(list)
    details = []
    for stage, _, (ranks, lines), training, _ in rows:
        entries = results[stage.id] + baseline_rows(stage, ranks)
        coverage = sum(r is not None for r in ranks)
        seeds = "published initial seeds" if stage.published else "anchor-derived initial seeds"
        details += [
            f"### {stage.title}",
            "",
            f"Stage {stage.id} opened {T.iso(stage.cutoff)}. The fit had {training:,} earlier matches, "
            f"{lines}/8 opening lines and {coverage}/16 archived HLTV ranks. The simulation used {seeds}. "
            + (
                "Every match was BO3."
                if set(stage.formats.values()) == {3}
                else "Qualification and elimination matches were BO3. Earlier matches were BO1."
            ),
            "",
            "| Model | Strategy | 3–0 | 0–3 | Other advance | Correct | Expected | Simulated ≥5 | Changed slots |",
            "|---|---|---|---|---|---:|---:|---:|---:|",
        ]
        for model, strategy, roles, score, changed, expected, chance in entries:
            groups[model, strategy].append((score, changed))
            cells = [", ".join(t for i, t in enumerate(stage.teams) if roles[i] == k) for k in range(3)]
            sims = "— | — | —" if expected is None else f"{expected:.2f} | {chance:.1%} | {changed}"
            details.append(f"| {model} | {strategy or '—'} | {' | '.join(cells)} | {score} | {sims} |")
        actual = [", ".join(t for i, t in enumerate(stage.teams) if stage.truth[i] == k) for k in range(3)]
        details += [f"| **Actual** | | {' | '.join(actual)} | | | | |", ""]
    n = len(rows)
    ids = np.random.default_rng(33).integers(n, size=(20000, n))
    summary = [
        "| Model | Strategy | Mean correct (95% CI) | Stages ≥5 | Share (95% CI) | Same slip at half count | Mean changed slots |",
        "|---|---|---:|---:|---:|---:|---:|",
    ]
    for (model, strategy), records in groups.items():
        observed = np.array([r[0] for r in records])
        passes = observed >= 5
        lo, hi = np.quantile(observed[ids].mean(axis=1), [0.025, 0.975])
        plo, phi = np.quantile(passes[ids].mean(axis=1), [0.025, 0.975])
        change = [r[1] for r in records if r[1] is not None]
        stability = f"{change.count(0)}/{n} | {np.mean(change):.2f}" if change else "— | —"
        summary.append(
            f"| {model} | {strategy or '—'} | {observed.mean():.2f} ({lo:.2f}–{hi:.2f}) | "
            f"{passes.sum()}/{n} | {passes.mean():.1%} ({plo:.1%}–{phi:.1%}) | {stability} |"
        )
    count, total = random_reference()
    qualified = [s for s, *_ in rows if s.published]
    rank_coverage = [sum(r is not None for r in meta[0]) for _, _, meta, _, _ in rows]
    rank_scores = np.array([score for score, _ in groups["HLTV top six", ""]])
    complete_ranks = rank_scores[np.array(rank_coverage) == 16]
    coverage_note = (
        f"With complete rank coverage alone, the HLTV rule averaged {complete_ranks.mean():.2f} "
        f"and passed on {sum(complete_ranks >= 5)}/{len(complete_ranks)} stages."
        if len(complete_ranks)
        else "No stage had complete rank coverage."
    )
    return "\n".join(
        [
            "# Swiss Pick’em replay",
            "",
            (
                f"This replay scores ten picks on **{n} completed stages**. Each correct pick earns one point. "
                "An ordinary advance pick earns a point only for a 3–1 or 3–2 finish."
            ),
            "",
            *summary,
            "",
            *(
                [
                    (
                        "On Singapore Stage 1, anchor A reached five. The earlier 20,000-run replay scored four. "
                        "The larger simulation swapped Heroic into 3–0 and PVISION into ordinary advance. "
                        "On the same 200,000 outcomes the slips were almost tied at 4.5453 and 4.5445 expected points."
                    ),
                    "",
                ]
                if runs == 200000 and any(stage.id == 22378 for stage, *_ in rows)
                else []
            ),
            (
                f"A uniform random slip has an exact mean of **2.75** and a **{count / total:.2%}** chance of at least five "
                f"({count:,} of {total:,} legal slips for any fixed stage result). The random row shows one reproducible "
                "uniform draw per stage and includes the same stage-bootstrap intervals as the other rows."
            ),
            "",
            (
                f"The intervals resample {n} whole stages with replacement 20,000 times, seed 33, taking the 2.5th and "
                "97.5th percentiles. These scores describe the archived contests. Stages from the same Major share teams "
                "and history, which can make stage-bootstrap intervals too narrow. This sample does not establish which "
                "model or strategy will perform best in future contests."
            ),
            "",
            "## Forecast and selection rules",
            "",
            (
                "- The research snapshot supplies the completed Swiss rounds. The Singapore fixture supplies its first 24 "
                "results and opening odds. The final nine results come from the PandaScore tournament 22378 matches endpoint. "
                "Canceled listings without winners are ignored. Officially awarded matches count for standings, including "
                "ESL Pro League S22 Stage 2 and PGL Bucharest 2026. They are simulated as matches before the stage."
            ),
            (
                "- The opening time freezes ratings, lines, rank badges and match history for all 120 possible opponents in both "
                "formats. Each learned model fits non-qualifier results with a known finish strictly before that time, including "
                "earlier stages of the same event. Hyperparameters minimize inner walk-forward loss on earlier completed events. "
                "Baseline and all correspond to the prototype’s baseline and full sets. Future match pages, lineups and "
                "match-specific form panels are unavailable at stage start and use the feature groups’ missing-data behavior. "
                "Venue comes from opening-match venue metadata when consistent, otherwise from the tournament listing."
            ),
            (
                "- The anchor blends de-vigged market and VRS logits 80/20 when both exist. The historical opening line applies "
                "only to that pairing and format. A missing source leaves the available source at full weight. Both missing "
                "gives 50%. The veto variant replaces BO1 with the prototype map_pool probability when both teams have earlier "
                "vetoes. It weights maps by ban/pick tendencies and historical map win shares. BO3 retains the anchor. "
                "This uses the Python map-pool approximation to the Rust six-ban simulation."
            ),
            (
                f"- Each model simulates {runs:,} joint outcomes per stage with seed 33, using actual round-one pairings and "
                f"observed formats by win/loss bucket. The sidecar Buchholz ranking and rematch priorities govern later rounds. "
                f"Pre-stage Liquipedia seed tables cover {len(qualified)} stages. The remaining {n - len(qualified)} use the "
                "sidecar fallback of sorting teams by summed BO1 win probability, breaking ties by team ID. "
                "The anchor supplies these seeds for every model so the pairing rule stays fixed. For non-Major "
                "stages this common pairing procedure may differ from the organizer’s. Replaying recorded winners "
                "reproduced every pairing and final record on all 12 Major stages with published seeds."
            ),
            (
                "- A solves the 2/2/6 disjoint assignment exactly for simulated expected points. B searches for the highest "
                "simulated chance of five points using best-improving swaps of two teams’ roles, including unpicked. It starts "
                "from A and two seeded random slips and also tries the half-count optimum on the full run. Search stops at "
                "a local optimum, so a better unsearched slip can exist. Simulated probabilities are in-sample estimates. "
                "Changed slots counts team/category selections replaced when increasing the simulation count from "
                f"{runs // 2:,} to {runs:,} on the same random stream. Reordering picks within a category counts as zero."
            ),
            (
                f"- The HLTV baseline puts ranks 1–6 among entrants into ordinary advance, 7–8 into 3–0 and the lowest two into "
                f"0–3. The archive supplies usable dated rank badges for all 16 teams in {rank_coverage.count(16)}/{n} stages "
                f"(range {min(rank_coverage)}–16). Missing ranks sort last, with published seed order or team name breaking "
                "ties. When coverage is partial, this measures a ranking rule with missing data. The Singapore baseline "
                f"needed a fallback even for its sixth advance pick. {coverage_note}"
            ),
            "",
            "### Excluded RMR and qualifier Swiss stages",
            "",
            *[f"- {name}" for name in sorted(set(rows[0][-1]))],
            "",
            (
                "The replay also excludes the BLAST Premier Bounty Closed Qualifier 2025 Playoffs and the Austin "
                "2025 European Regional Qualifier Play-Ins. Both used other formats."
            ),
            "",
            "### Reproduce",
            "",
            "From `research/ml/` using the snapshot and the prototype’s HLTV page cache or source files.",
            "",
            "```powershell",
            "uv sync",
            "uv run python -m kbml.pickem_check --runs 200000 --jobs 3",
            "```",
            "",
            "## Stage results",
            "",
            *details,
        ]
    )


def run_report(rows, runs, jobs, path):
    if runs < 2 or jobs < 1:
        raise ValueError("Need at least two simulations and one process")
    tasks = [
        (stage, model, values, runs) for stage, forecasts, *_ in rows for model, values in forecasts.items()
    ]
    results = defaultdict(list)
    with ProcessPoolExecutor(jobs) as pool:
        for stage_id, entries in pool.map(evaluate_strategy, tasks):
            results[stage_id].extend(entries)
    path.write_text(render(rows, results, runs), encoding="utf-8")
    print(f"Wrote {len(rows)} stages to {path}")
