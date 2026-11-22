// Tests of the Matches page's pure layout logic (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import { bracketColumns, cardState } from "../src/pages/matches/layout.ts";

const card = (id, bracket_slot) => ({ id, bracket_slot });
const ids = (columns) => columns.map((c) => [c.round, c.cells.map((x) => (x ? x.id : null))]);

test("a Void Match reads as void, never as finished", () => {
  assert.equal(cardState({ status: "finished", winner: null }), "void");
  assert.equal(cardState({ status: "cancelled", winner: null }), "void");
  assert.equal(cardState({ status: "finished", winner: 7 }), "finished");
  assert.equal(cardState({ status: "live", winner: null }), "live");
  assert.equal(cardState({ status: "upcoming", winner: null }), "upcoming");
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
