// Tests of the Reveal moment's detection and stagger (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import {
  createRevealTracker, nextRevealState, revealDelayMs, revealKey,
} from "../src/pages/matches/reveal.ts";

/** An in-memory stand-in for sessionStorage. */
function memoryStorage() {
  const data = new Map();
  return {
    getItem: (key) => (data.has(key) ? data.get(key) : null),
    setItem: (key, value) => data.set(key, String(value)),
  };
}

/** A storage that throws on every call, like a blocked one. */
const brokenStorage = {
  getItem() { throw new Error("blocked"); },
  setItem() { throw new Error("blocked"); },
};

/**
 * One mounted card, the way MatchCard drives the tracker: start from
 * `initial`, compute the next state on each render, commit it afterwards.
 */
function mount(tracker, key) {
  let prev = tracker.initial(key);
  return {
    render(revealed) {
      const state = nextRevealState(prev, { revealed, played: tracker.played(key) });
      tracker.commit(key, state);
      prev = state;
      return state;
    },
  };
}

test("the next state: hidden, then revealed once, then it stays", () => {
  assert.equal(nextRevealState(undefined, { revealed: false, played: false }), "hidden");
  assert.equal(nextRevealState("hidden", { revealed: false, played: false }), "hidden");
  assert.equal(nextRevealState("hidden", { revealed: true, played: false }), "just-revealed");
  assert.equal(nextRevealState("just-revealed", { revealed: true, played: true }), "just-revealed",
    "a re-render of the same card keeps playing what it started");
  assert.equal(nextRevealState("hidden", { revealed: true, played: true }), "settled",
    "a Reveal that already played elsewhere does not play again");
});

test("a card already revealed on its first render never animates", () => {
  const tracker = createRevealTracker(memoryStorage());
  const card = mount(tracker, revealKey("7", 1));
  assert.equal(card.render(true), "settled");
  assert.equal(card.render(true), "settled");
  assert.equal(nextRevealState(undefined, { revealed: true, played: false }), "settled");
});

test("a card seen hidden and then revealed animates exactly once", () => {
  const tracker = createRevealTracker(memoryStorage());
  const card = mount(tracker, revealKey("7", 2));
  assert.equal(card.render(false), "hidden");
  assert.equal(card.render(false), "hidden", "polls while still open");
  assert.equal(card.render(true), "just-revealed", "the open to revealed transition");
  assert.equal(card.render(true), "just-revealed", "later re-renders keep the cue and classes");
});

test("a remount after the Reveal does not replay it", () => {
  const tracker = createRevealTracker(memoryStorage());
  const key = revealKey("7", 3);
  const first = mount(tracker, key);
  first.render(false);
  assert.equal(first.render(true), "just-revealed");

  const again = mount(tracker, key);
  assert.equal(again.render(true), "settled");
});

test("a reload does not replay a Reveal that played (sessionStorage)", () => {
  const storage = memoryStorage();
  const key = revealKey("7", 4);
  const before = mount(createRevealTracker(storage), key);
  before.render(false);
  assert.equal(before.render(true), "just-revealed");

  // After a reload: memory is empty, sessionStorage remembers the Reveal.
  const reloaded = createRevealTracker(storage);
  assert.equal(reloaded.played(key), true);
  assert.equal(mount(reloaded, key).render(true), "settled", "first render revealed");
  const stale = mount(reloaded, key);
  stale.render(false);
  assert.equal(stale.render(true), "settled", "even if stale data showed it hidden first");
});

test("without sessionStorage it still works from memory", () => {
  for (const storage of [null, brokenStorage]) {
    const tracker = createRevealTracker(storage);
    const key = revealKey("7", 5);
    const card = mount(tracker, key);
    assert.equal(card.render(false), "hidden");
    assert.equal(card.render(true), "just-revealed");
    assert.equal(tracker.played(key), true);
    assert.equal(mount(tracker, key).render(true), "settled", "no replay on remount");
  }
});

test("corrupt stored data is ignored", () => {
  const storage = memoryStorage();
  storage.setItem("kallbet.revealsPlayed", "{not json");
  const tracker = createRevealTracker(storage);
  const card = mount(tracker, revealKey("7", 6));
  card.render(false);
  assert.equal(card.render(true), "just-revealed");
});

test("Reveals are tracked per League and Match", () => {
  const tracker = createRevealTracker(memoryStorage());
  const a = mount(tracker, revealKey("1", 9));
  a.render(false);
  a.render(true);
  const b = mount(tracker, revealKey("2", 9));
  b.render(false);
  assert.equal(b.render(true), "just-revealed", "the same Match in another League");
  assert.equal(revealKey(undefined, 9), "-:9");
});

test("the stagger is non-uniform and capped at four steps", () => {
  assert.deepEqual([0, 1, 2, 3, 4, 9].map(revealDelayMs), [40, 90, 120, 160, 160, 160]);
  assert.equal(revealDelayMs(-1), 40);
});
