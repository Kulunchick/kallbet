// Tests of the Sheet's drag-to-dismiss maths (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import {
  DISMISS_VELOCITY, RUBBER_BAND_LIMIT, dampUp, releaseVelocity, shouldDismiss,
} from "../src/components/sheetMath.ts";

const H = 600; // a third of it is 200

test("a slow drag dismisses only past a third of the height", () => {
  assert.equal(shouldDismiss({ distance: 0, height: H, velocity: 0 }), false);
  assert.equal(shouldDismiss({ distance: 199, height: H, velocity: 0 }), false);
  assert.equal(shouldDismiss({ distance: 201, height: H, velocity: 0 }), true);
  assert.equal(shouldDismiss({ distance: H, height: H, velocity: 0 }), true);
});

test("exactly a third of the height springs back", () => {
  assert.equal(shouldDismiss({ distance: H / 3, height: H, velocity: 0 }), false);
});

test("a fast downward flick dismisses even a short drag", () => {
  assert.equal(shouldDismiss({ distance: 10, height: H, velocity: 0.2 }), true);
  assert.equal(shouldDismiss({ distance: 0, height: H, velocity: DISMISS_VELOCITY + 0.001 }), true);
});

test("exactly the velocity threshold springs back", () => {
  assert.equal(shouldDismiss({ distance: 10, height: H, velocity: DISMISS_VELOCITY }), false);
});

test("a drag up, or a flick up, never dismisses", () => {
  assert.equal(shouldDismiss({ distance: -80, height: H, velocity: 0 }), false);
  assert.equal(shouldDismiss({ distance: -80, height: H, velocity: -0.5 }), false);
  assert.equal(shouldDismiss({ distance: 10, height: H, velocity: -0.5 }), false);
});

test("a sheet with no height is not dismissed by distance", () => {
  assert.equal(shouldDismiss({ distance: 5, height: 0, velocity: 0 }), false);
});

test("dampUp gives no travel for no pull, or a pull the wrong way", () => {
  assert.equal(dampUp(0), 0);
  assert.equal(dampUp(-20), 0);
  assert.equal(dampUp(Number.NaN), 0);
});

test("dampUp grows with the pull but always lags the finger and stays under the limit", () => {
  let previous = 0;
  for (const d of [1, 5, 20, 60, 200, 1000, 100000]) {
    const out = dampUp(d);
    assert.ok(out > previous, `${d} should travel further than before`);
    assert.ok(out < d, `${d} should lag the finger`);
    assert.ok(out < RUBBER_BAND_LIMIT, `${d} should stay under the limit`);
    previous = out;
  }
});

test("dampUp starts at about half the finger's pace", () => {
  assert.ok(Math.abs(dampUp(1) - 0.54) < 0.02);
});

test("releaseVelocity is px/ms over the last 100 ms of moves", () => {
  const samples = [{ y: 0, t: 0 }, { y: 10, t: 950 }, { y: 30, t: 1000 }, { y: 50, t: 1050 }];
  // Only the last three are within 100 ms of t = 1050: 40 px in 100 ms.
  assert.equal(releaseVelocity(samples, 1050), 0.4);
});

test("releaseVelocity is negative for an upward move", () => {
  assert.ok(releaseVelocity([{ y: 100, t: 0 }, { y: 60, t: 40 }], 40) < 0);
});

test("a finger that paused before lifting has no velocity", () => {
  const samples = [{ y: 0, t: 0 }, { y: 80, t: 50 }];
  assert.equal(releaseVelocity(samples, 400), 0);
  assert.equal(releaseVelocity([], 0), 0);
  assert.equal(releaseVelocity([{ y: 5, t: 10 }], 10), 0);
});

test("samples at the same instant give no velocity instead of dividing by zero", () => {
  assert.equal(releaseVelocity([{ y: 0, t: 5 }, { y: 50, t: 5 }], 5), 0);
});
