// Tests of the Matches page's pure layout logic (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import {
  bracketColumns, cardState, collapsedStage, currentStage, earliestStart,
  hasPreliminaryPick, isOpenToPick, MASONRY_GAP, MASONRY_ROW, masonrySpan, nextLockAt, nextUnpicked, orderMatchGroups,
  pickMark, splitFinished, stripModel, undoTarget, unpickedOpen,
} from "../src/pages/matches/layout.ts";

const card = (id, bracket_slot) => ({ id, bracket_slot });
const ids = (columns) => columns.map((c) => [c.round, c.cells.map((x) => (x ? x.id : null))]);

test("a Void Match reads as void, never as finished", () => {
  assert.equal(cardState({ status: "finished", winner: null }), "void");
  assert.equal(cardState({ status: "cancelled", winner: null }), "void");
  assert.equal(cardState({ status: "finished", winner: 7 }), "finished");
  assert.equal(cardState({ status: "live", winner: null }), "live");
  assert.equal(cardState({ status: "upcoming", winner: null }), "upcoming");
});

test("a Bot User's revealed pick carries a bot badge, a human's none", () => {
  assert.equal(pickMark({ is_bot: false, preliminary: false }), "human");
  assert.equal(pickMark({ is_bot: true, preliminary: false }), "bot");
  // A server from before Bot Users sends neither field.
  assert.equal(pickMark({}), "human");
});

test("a Bot User's pick is preliminary until Hard Lock, a human's never", () => {
  assert.equal(pickMark({ is_bot: true, preliminary: true }), "bot_preliminary");
  assert.equal(pickMark({ is_bot: false, preliminary: true }), "human");
});

test("a card warns about preliminary picks only while a revealed bot pick may change", () => {
  const human = { is_bot: false, preliminary: false };
  assert.equal(hasPreliminaryPick(null), false, "nothing revealed yet");
  assert.equal(hasPreliminaryPick([]), false);
  assert.equal(hasPreliminaryPick([human, { is_bot: true, preliminary: true }]), true);
  assert.equal(hasPreliminaryPick([human, { is_bot: true, preliminary: false }]), false, "after Hard Lock");
});

test("an 8-team bracket has 4, 2 and 1 cells, first round first", () => {
  const { columns, extra } = bracketColumns([
    { rounds_before_final: 2, matches: [card(1, 0), card(2, 1), card(3, 2), card(4, 3)] },
    { rounds_before_final: 1, matches: [card(5, 0), card(6, 1)] },
    { rounds_before_final: 0, matches: [card(7, 0)] },
  ]);
  assert.deepEqual(ids(columns), [
    [2, [1, 2, 3, 4]],
    [1, [5, 6]],
    [0, [7]],
  ]);
  assert.deepEqual(extra, []);
});

test("slots without a Match yet are empty cells, and rounds with none still appear", () => {
  const { columns } = bracketColumns([
    { rounds_before_final: 2, matches: [card(3, 2)] },
    { rounds_before_final: 0, matches: [card(7, 0)] },
  ]);
  assert.deepEqual(ids(columns), [
    [2, [null, null, 3, null]],
    [1, [null, null]],
    [0, [7]],
  ]);
});

test("a card without a usable slot is kept aside, not dropped", () => {
  const { columns, extra } = bracketColumns([
    { rounds_before_final: 1, matches: [card(1, 0), card(2, 5), card(3, null), card(4, 0)] },
  ]);
  assert.deepEqual(ids(columns), [
    [1, [1, null]],
    [0, [null]],
  ]);
  assert.deepEqual(extra.map((c) => c.id), [2, 3, 4]);
});

test("no Playoffs Matches means no bracket", () => {
  assert.deepEqual(bracketColumns([]), { columns: [], extra: [] });
});

test("match groups put live and near upcoming matches above recent results", () => {
  const match = (id, status, day) => ({
    id, status, actual_start: null,
    scheduled_start: day === null ? null : `2026-11-${day}T12:00:00Z`,
  });
  const archived = { matches: [match(1, "finished", 10), match(2, "finished", 12)] };
  const upcoming = { matches: [
    match(3, "upcoming", null), match(4, "upcoming", 23), match(5, "upcoming", 21),
  ] };
  const live = { matches: [match(6, "live", 20)] };
  assert.deepEqual(
    orderMatchGroups([archived, upcoming, live]).map((g) => g.matches.map((m) => m.id)),
    [[6], [5, 4, 3], [2, 1]],
  );
  assert.deepEqual(archived.matches.map((m) => m.id), [1, 2]);
});

// -- Picking ----------------------------------------------------------------------

const NOW = Date.parse("2026-11-20T12:00:00Z");
const minutes = (n) => new Date(NOW + n * 60_000).toISOString();
const A = { id: 1, name: "Astralis" };
const B = { id: 2, name: "Falcons" };

/** A card with sensible defaults: upcoming, open, nothing picked. */
const pick = (id, over = {}) => ({
  id,
  status: "upcoming",
  winner: null,
  team_a: A,
  team_b: B,
  scheduled_start: minutes(60),
  my_prediction: null,
  predictions: { revealed: null },
  ...over,
  lock: { locks_at: minutes(45), locked: false, can_predict: true, ...over.lock },
});

test("a Match is open to pick only while upcoming, not Locked, with both teams", () => {
  assert.equal(isOpenToPick(pick(1), NOW), true);
  assert.equal(isOpenToPick(pick(1, { lock: { locked: true } }), NOW), false);
  assert.equal(isOpenToPick(pick(1, { lock: { can_predict: false } }), NOW), false, "bot window or other owner");
  assert.equal(isOpenToPick(pick(1, { lock: { locks_at: minutes(-1) } }), NOW), false, "lock time passed, page not refreshed yet");
  assert.equal(isOpenToPick(pick(1, { lock: { locks_at: null } }), NOW), true, "no start time yet");
  assert.equal(isOpenToPick(pick(1, { team_b: null }), NOW), false);
  assert.equal(isOpenToPick(pick(1, { status: "live" }), NOW), false);
  assert.equal(isOpenToPick(pick(1, { status: "finished", winner: 1 }), NOW), false);
});

test("unpicked count ignores picked, Locked and finished Matches", () => {
  const cards = [
    pick(1), pick(2, { my_prediction: 1 }), pick(3, { lock: { locked: true } }),
    pick(4, { status: "finished", winner: 2 }), pick(5),
  ];
  assert.deepEqual(unpickedOpen(cards, NOW).map((c) => c.id), [1, 5]);
  assert.deepEqual(unpickedOpen([], NOW), []);
});

test("the next Lock is the earliest one still ahead", () => {
  const cards = [
    pick(1, { lock: { locks_at: minutes(90) } }),
    pick(2, { lock: { locks_at: minutes(12) } }),
    pick(3, { lock: { locks_at: minutes(-5) } }),
    pick(4, { lock: { locks_at: null } }),
  ];
  assert.equal(nextLockAt(cards, NOW), NOW + 12 * 60_000);
  assert.equal(nextLockAt([pick(4, { lock: { locks_at: null } })], NOW), null);
  assert.equal(nextLockAt([], NOW), null);
});

test("jump-to-next targets the unpicked Match that Locks first, a Match without a Lock time last", () => {
  const cards = [
    pick(1, { lock: { locks_at: null } }),
    pick(2, { lock: { locks_at: minutes(90) } }),
    pick(3, { lock: { locks_at: minutes(20) }, my_prediction: 1 }),
    pick(4, { lock: { locks_at: minutes(30) } }),
  ];
  assert.equal(nextUnpicked(cards, NOW).id, 4);
  assert.equal(nextUnpicked([cards[0]], NOW).id, 1);
  assert.equal(nextUnpicked([cards[2]], NOW), null);
});

test("the strip counts unpicked Matches and times the Lock of those, not of picked ones", () => {
  const cards = [
    pick(1, { lock: { locks_at: minutes(12) }, my_prediction: 1 }), // picked, sooner
    pick(2, { lock: { locks_at: minutes(70) } }),
    pick(3, { lock: { locks_at: minutes(130) } }),
  ];
  assert.deepEqual(stripModel(cards, NOW), { kind: "unpicked", count: 2, nextLock: NOW + 70 * 60_000 });
  assert.deepEqual(stripModel([cards[1]], NOW), { kind: "unpicked", count: 1, nextLock: NOW + 70 * 60_000 });
});

test("with everything picked the strip turns calm, and keeps the next Lock", () => {
  const cards = [pick(1, { my_prediction: 1, lock: { locks_at: minutes(25) } })];
  assert.deepEqual(stripModel(cards, NOW), { kind: "allPicked", nextLock: NOW + 25 * 60_000 });
});

test("nothing open but things pending says so; a finished Event says nothing", () => {
  assert.deepEqual(stripModel([pick(1, { lock: { locked: true } }), pick(2, { status: "live" })], NOW), { kind: "nothing" });
  assert.deepEqual(stripModel([pick(1, { team_a: null, team_b: null })], NOW), { kind: "nothing" });
  assert.deepEqual(stripModel([pick(1, { status: "finished", winner: 1 })], NOW), { kind: "hidden" });
  assert.deepEqual(stripModel([], NOW), { kind: "hidden" });
});

test("undo puts back the previous team, and only when there was one", () => {
  assert.equal(undoTarget(pick(1), 2), null, "a first pick has nothing to undo");
  assert.deepEqual(undoTarget(pick(1, { my_prediction: 1 }), 2), A);
  assert.equal(undoTarget(pick(1, { my_prediction: 2 }), 2), null, "same team again");
  assert.equal(undoTarget(pick(1, { my_prediction: 99 }), 2), null, "previous team no longer in the Match");
});

// -- Progressive disclosure -------------------------------------------------------

const done = (id) => pick(id, { status: "finished", winner: 1 });
const stage = (ordinal, ...groups) => ({ ordinal, groups: groups.map((matches) => ({ matches })) });

test("the current Stage is the first with a live or fixed upcoming Match", () => {
  const s1 = stage(1, [done(1), pick(2)]);
  const s2 = stage(2, [pick(3, { team_a: null, team_b: null })]);
  assert.equal(currentStage([s1, s2]), 1);
  assert.equal(collapsedStage(s1, 1), false);
  assert.equal(collapsedStage(s2, 1), true, "playoffs not started: collapsed");
});

test("a later Stage with a live Match is not collapsed", () => {
  const s1 = stage(1, [done(1)]);
  const s2 = stage(2, [pick(2, { status: "live" })]);
  assert.equal(currentStage([s1, s2]), 2);
  assert.equal(collapsedStage(s1, 2), false, "earlier Stages stay, their results fold");
  assert.equal(collapsedStage(s2, 2), false);
});

test("a Locked upcoming Match still keeps its Stage current", () => {
  const s1 = stage(1, [pick(1, { lock: { locked: true } })]);
  const s2 = stage(2, [pick(2, { team_a: null, team_b: null })]);
  assert.equal(currentStage([s1, s2]), 1);
});

test("with no active Match the latest Stage with results is current; with none at all the first", () => {
  const over = [stage(1, [done(1)]), stage(2, [done(2)]), stage(3, [pick(3, { team_a: null, team_b: null })])];
  assert.equal(currentStage(over), 2);
  assert.equal(collapsedStage(over[2], 2), true);
  const fresh = [stage(5, [pick(1, { team_a: null, team_b: null })]), stage(6, [])];
  assert.equal(currentStage(fresh), 5);
  assert.equal(collapsedStage(fresh[1], 5), true);
  assert.equal(currentStage([]), null);
  assert.equal(collapsedStage(stage(1), null), false);
});

test("finished and Void Matches fold away per Stage, group by group, nothing lost", () => {
  const voided = pick(4, { status: "cancelled" });
  const groups = [
    { kind: "record", matches: [done(1), pick(2), done(3)] },
    { kind: "record", matches: [voided] },
    { kind: "record", matches: [pick(5, { status: "live" })] },
  ];
  const { open, finished, finishedCount } = splitFinished(groups);
  assert.equal(finishedCount, 3);
  assert.deepEqual(open.map((g) => g.matches.map((m) => m.id)), [[2], [5]]);
  assert.deepEqual(finished.map((g) => g.matches.map((m) => m.id)), [[1, 3], [4]]);
  assert.equal(open[0].kind, "record");
  assert.deepEqual(groups[0].matches.map((m) => m.id), [1, 2, 3], "input untouched");
});

test("a Stage with nothing finished has no Finished row", () => {
  assert.equal(splitFinished([{ matches: [pick(1)] }]).finishedCount, 0);
  assert.deepEqual(splitFinished([]), { open: [], finished: [], finishedCount: 0 });
});

test("a Stage starts at its earliest Scheduled Start", () => {
  const cards = [{ scheduled_start: minutes(300) }, { scheduled_start: null }, { scheduled_start: minutes(100) }];
  assert.equal(earliestStart(cards), minutes(100));
  assert.equal(earliestStart([{ scheduled_start: null }]), null);
  assert.equal(earliestStart([]), null);
});

// -- Masonry ----------------------------------------------------------------------

test("a card's span covers its measured height plus the gap, so cards never overlap", () => {
  for (let tenths = 0; tenths <= 6000; tenths += 7) {
    const height = tenths / 10;
    const span = masonrySpan(height);
    assert.ok(span * MASONRY_ROW >= height + MASONRY_GAP, `${height}px fits in ${span} rows`);
    assert.ok((span - 1) * MASONRY_ROW < height + MASONRY_GAP, `${height}px: no spare row`);
  }
});

test("the span rounds up to whole rows", () => {
  assert.equal(masonrySpan(100), 28, "100 + 12 = 112 = 28 rows of 4");
  assert.equal(masonrySpan(100.5), 29);
  assert.equal(masonrySpan(101), 29);
  assert.equal(masonrySpan(150, 12, 4), 41);
  assert.equal(masonrySpan(20, 0, 10), 2, "custom gap and row");
});

test("an unmeasured or broken height still spans at least the gap", () => {
  assert.equal(masonrySpan(0), 3);
  assert.equal(masonrySpan(-5), 3);
  assert.equal(masonrySpan(Number.NaN), 3);
  assert.equal(masonrySpan(0, 0), 1, "never fewer than one row");
});
