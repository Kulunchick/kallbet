// Tests of the Pick'em boards' pure logic: Swiss placing and progress, bracket
// numbering, round progress and grid placement, the Stage switcher (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import {
  feederNumber, firstIncompleteRound, gridPlacement, matchNumbers, placedPicks, roundProgress, roundsOf,
  setWinner, slotsOfRound, verdict,
} from "../src/pages/pickem/bracket.ts";
import {
  availability, countIn, draftFrom, inGroup, isFull, isSwissComplete, nextPoolFocus, picksLeft, place,
  placedCount, toBody, unplace, unplaced,
} from "../src/pages/pickem/swiss.ts";
import { defaultStage, stageFromHash } from "../src/pages/pickem/stages.ts";

const teams = Array.from({ length: 16 }, (_, i) => ({ id: i + 1, name: `T${i + 1}` }));

// -- Swiss ---------------------------------------------------------------------

test("placing fills a group and refuses a full one", () => {
  let d = place({}, 1, "three_nil");
  d = place(d, 2, "three_nil");
  assert.equal(isFull(d, "three_nil"), true);
  assert.deepEqual(place(d, 3, "three_nil"), d, "a full group takes nobody");
  assert.equal(place(d, 1, "three_nil"), d, "a team already in the group changes nothing");
});

test("a placed team moves to another group and a team is placed once", () => {
  const d = place(place({}, 1, "three_nil"), 1, "advance");
  assert.deepEqual(d, { 1: "advance" });
  assert.equal(placedCount(d), 1);
});

test("removing frees the slot and the team returns to the pool", () => {
  const d = place(place({}, 1, "zero_three"), 2, "zero_three");
  const after = unplace(d, 1);
  assert.equal(isFull(after, "zero_three"), false);
  assert.deepEqual(unplaced(teams.slice(0, 3), after).map((x) => x.id), [1, 3]);
  assert.equal(unplace(after, 99), after, "removing a team that is not placed changes nothing");
});

test("availability, counts and progress", () => {
  let d = {};
  for (const [team, group] of [[1, "three_nil"], [2, "three_nil"], [3, "advance"], [4, "zero_three"]]) d = place(d, team, group);
  assert.deepEqual(availability(d).map((a) => [a.choice, a.count, a.limit, a.full]), [
    ["three_nil", 2, 2, true],
    ["advance", 1, 6, false],
    ["zero_three", 1, 2, false],
  ]);
  assert.equal(countIn(d, "advance"), 1);
  assert.equal(placedCount(d), 4);
  assert.equal(picksLeft(d), 6);
  assert.equal(isSwissComplete(d), false);
});

test("a Swiss Pick'em is complete at 2 + 6 + 2 and builds the request body in team order", () => {
  let d = {};
  teams.slice(0, 2).forEach((x) => (d = place(d, x.id, "three_nil")));
  teams.slice(2, 8).forEach((x) => (d = place(d, x.id, "advance")));
  teams.slice(8, 10).forEach((x) => (d = place(d, x.id, "zero_three")));
  assert.equal(isSwissComplete(d), true);
  assert.equal(picksLeft(d), 0);
  assert.deepEqual(toBody(teams, d), {
    three_nil: [1, 2],
    advance: [3, 4, 5, 6, 7, 8],
    zero_three: [9, 10],
  });
  assert.deepEqual(inGroup(teams, d, "zero_three").map((x) => x.id), [9, 10]);
});

test("a saved Pick'em becomes a draft and back", () => {
  const saved = { three_nil: [3, 1], advance: [2], zero_three: [] };
  const d = draftFrom(saved);
  assert.deepEqual(d, { 3: "three_nil", 1: "three_nil", 2: "advance" });
  assert.deepEqual(draftFrom(null), {});
  assert.deepEqual(toBody(teams.slice(0, 3), d), { three_nil: [1, 3], advance: [2], zero_three: [] });
});

test("focus goes to the tile that takes the placed one's place, else the one before, else nowhere", () => {
  assert.equal(nextPoolFocus([1, 2, 3], 2), 3);
  assert.equal(nextPoolFocus([1, 2, 3], 3), 2);
  assert.equal(nextPoolFocus([1], 1), null);
  assert.equal(nextPoolFocus([], 1), null);
});

// -- Bracket ---------------------------------------------------------------------

const slot = (r, s, teamsOf = [null, null], leaf = r === 2, winner = null) => ({
  rounds_before_final: r, slot: s, team_a: teamsOf[0], team_b: teamsOf[1], leaf, winner,
});
const slots = [
  slot(2, 0, [1, 2]), slot(2, 1, [3, 4]), slot(2, 2, [5, 6]), slot(2, 3, [7, 8]),
  slot(1, 0), slot(1, 1), slot(0, 0),
];
const pick = (w, r, s, team) => setWinner(slots, w, r, s, team);

test("Matches are numbered first round first, top to bottom", () => {
  assert.deepEqual(matchNumbers(slots), {
    "2:0": 1, "2:1": 2, "2:2": 3, "2:3": 4, "1:0": 5, "1:1": 6, "0:0": 7,
  });
  assert.deepEqual(roundsOf(slots), [2, 1, 0]);
  assert.deepEqual(slotsOfRound(slots, 1).map((s) => s.slot), [0, 1]);
});

test("a later slot names the Matches that feed it", () => {
  assert.equal(feederNumber(slots, slots[4], 0), 1);
  assert.equal(feederNumber(slots, slots[4], 1), 2);
  assert.equal(feederNumber(slots, slots[5], 1), 4);
  assert.equal(feederNumber(slots, slots[6], 0), 5);
  assert.equal(feederNumber(slots, slots[6], 1), 6);
  assert.equal(feederNumber(slots, slots[0], 0), null, "a first slot has no feeder");
  const orphan = [slot(1, 0), slot(0, 0)];
  assert.equal(feederNumber(orphan, orphan[0], 0), null, "no feeding Match in the data");
});

test("round progress counts picks per round and across the bracket", () => {
  let w = pick({}, 2, 0, 1);
  w = pick(w, 2, 1, 4);
  w = pick(w, 2, 2, 5);
  assert.deepEqual(roundProgress(slots, w, 2), { done: 3, total: 4 });
  assert.deepEqual(roundProgress(slots, w, 1), { done: 0, total: 2 });
  assert.equal(placedPicks(slots, w), 3);
  assert.equal(firstIncompleteRound(slots, w), 2);
  w = pick(w, 2, 3, 8);
  assert.equal(firstIncompleteRound(slots, w), 1);
  w = pick(pick(w, 1, 0, 1), 1, 1, 5);
  w = pick(w, 0, 0, 1);
  assert.equal(placedPicks(slots, w), 7);
  assert.equal(firstIncompleteRound(slots, w), 2, "all picked: back to the first round");
});

test("clearing an earlier pick lowers the later rounds' progress", () => {
  let w = pick(pick({}, 2, 0, 1), 2, 1, 3);
  w = pick(w, 1, 0, 1);
  assert.equal(placedPicks(slots, w), 3);
  w = pick(w, 2, 0, 2);
  assert.equal(placedPicks(slots, w), 2, "the semifinal pick on team 1 is cleared");
});

test("grid placement centers a later card between the cards that feed it", () => {
  assert.deepEqual(gridPlacement(2, 0, 2), { row: 1, span: 1 });
  assert.deepEqual(gridPlacement(2, 3, 2), { row: 4, span: 1 });
  assert.deepEqual(gridPlacement(1, 0, 2), { row: 1, span: 2 });
  assert.deepEqual(gridPlacement(1, 1, 2), { row: 3, span: 2 });
  assert.deepEqual(gridPlacement(0, 0, 2), { row: 1, span: 4 });
  // A bracket that starts at the semifinals.
  assert.deepEqual(gridPlacement(1, 1, 1), { row: 2, span: 1 });
  assert.deepEqual(gridPlacement(0, 0, 1), { row: 1, span: 2 });
});

test("a verdict needs both a settled slot and my pick", () => {
  assert.equal(verdict(null, 3), null);
  assert.equal(verdict(3, undefined), null);
  assert.equal(verdict(3, 3), "correct");
  assert.equal(verdict(3, 4), "wrong");
});

// -- Stage switcher -----------------------------------------------------------------

test("a #stage-N hash names a playable Stage only", () => {
  assert.equal(stageFromHash("#stage-2", [1, 2]), 2);
  assert.equal(stageFromHash("#stage-3", [1, 2]), null);
  assert.equal(stageFromHash("#other", [1, 2]), null);
  assert.equal(stageFromHash("", [1, 2]), null);
});

test("the first Stage shown is the first open to edit, else the latest", () => {
  assert.equal(defaultStage([{ ordinal: 1, canEdit: false }, { ordinal: 2, canEdit: true }, { ordinal: 3, canEdit: true }]), 2);
  assert.equal(defaultStage([{ ordinal: 2, canEdit: false }, { ordinal: 1, canEdit: false }]), 2);
  assert.equal(defaultStage([]), null);
});
