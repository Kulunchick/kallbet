// Tests of the Playoffs Pick'em's pure bracket logic (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import { fromPicks, isComplete, normalize, participants, setWinner, slotKey, toPicks } from "../src/pages/pickem/bracket.ts";

const slot = (r, s, teams = [null, null], leaf = r === 2) => ({
  rounds_before_final: r,
  slot: s,
  team_a: teams[0],
  team_b: teams[1],
  leaf,
  winner: null,
});

// Quarterfinals 1-2, 3-4, 5-6, 7-8, then two semifinals and the final.
const slots = [
  slot(2, 0, [1, 2]),
  slot(2, 1, [3, 4]),
  slot(2, 2, [5, 6]),
  slot(2, 3, [7, 8]),
  slot(1, 0),
  slot(1, 1),
  slot(0, 0),
];

const pick = (winners, r, s, team) => setWinner(slots, winners, r, s, team);

test("a picked winner becomes a participant of the next round's slot", () => {
  let w = pick({}, 2, 0, 1);
  w = pick(w, 2, 1, 4);
  assert.deepEqual(participants(slots[4], w), [1, 4]);
  assert.deepEqual(participants(slots[5], w), [null, null]);
});

test("a winner can only be picked from a slot's participants", () => {
  const w = pick({}, 2, 0, 1);
  // Nobody is known in the semifinal's other half, and team 9 is nobody's.
  assert.deepEqual(pick(w, 1, 0, 9), w);
  assert.deepEqual(pick(w, 1, 0, 1), { ...w, [slotKey(1, 0)]: 1 });
});

test("changing an early pick clears the picks that depended on it", () => {
  let w = {};
  for (const [r, s, t] of [[2, 0, 1], [2, 1, 3], [2, 2, 5], [2, 3, 7], [1, 0, 1], [1, 1, 7], [0, 0, 1]]) w = pick(w, r, s, t);
  assert.ok(isComplete(slots, w));

  // Team 1 no longer wins the first quarterfinal: its semifinal and final picks go.
  const changed = pick(w, 2, 0, 2);
  assert.equal(changed[slotKey(2, 0)], 2);
  assert.equal(changed[slotKey(1, 0)], undefined);
  assert.equal(changed[slotKey(0, 0)], undefined);
  // The other half of the bracket is untouched.
  assert.equal(changed[slotKey(1, 1)], 7);
  assert.equal(changed[slotKey(2, 3)], 7);
  assert.ok(!isComplete(slots, changed));
});

test("re-picking the same winner keeps the later picks", () => {
  let w = {};
  for (const [r, s, t] of [[2, 0, 1], [2, 1, 3], [1, 0, 1]]) w = pick(w, r, s, t);
  assert.deepEqual(pick(w, 2, 0, 1), w);
});

test("a team keeps winning later rounds only while it keeps winning", () => {
  let w = {};
  for (const [r, s, t] of [[2, 0, 1], [2, 1, 3], [2, 2, 5], [2, 3, 7], [1, 0, 1], [1, 1, 5], [0, 0, 5]]) w = pick(w, r, s, t);
  // Switching the semifinal winner to team 7... is not possible (5 is the other side's pick); switch the quarterfinal.
  const changed = pick(w, 2, 2, 6);
  assert.equal(changed[slotKey(1, 1)], undefined);
  assert.equal(changed[slotKey(0, 0)], undefined);
});

test("picks convert to the request body and back", () => {
  let w = {};
  for (const [r, s, t] of [[2, 0, 1], [2, 1, 3], [1, 0, 3]]) w = pick(w, r, s, t);
  const picks = toPicks(slots, w);
  assert.deepEqual(picks, [
    { rounds_before_final: 2, slot: 0, team_id: 1 },
    { rounds_before_final: 2, slot: 1, team_id: 3 },
    { rounds_before_final: 1, slot: 0, team_id: 3 },
  ]);
  assert.deepEqual(fromPicks(picks), w);
  assert.deepEqual(normalize(slots, fromPicks(picks)), w);
});

test("an empty bracket is never complete", () => {
  assert.equal(isComplete([], {}), false);
});
