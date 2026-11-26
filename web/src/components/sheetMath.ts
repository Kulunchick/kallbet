// Pure helpers for the Sheet's drag-to-dismiss gesture. No DOM, so `npm test`
// can cover the numbers (scripts/sheet-math.test.mjs).

/** A drag that ends past this fraction of the sheet's height dismisses it. */
export const DISMISS_DISTANCE_RATIO = 1 / 3;

/** A downward flick faster than this (pixels per millisecond) dismisses it. */
export const DISMISS_VELOCITY = 0.11;

/** The furthest, in pixels, the sheet can be pulled up past its resting place. */
export const RUBBER_BAND_LIMIT = 48;

/** Only the pointer moves of this recent window count towards the release velocity. */
export const VELOCITY_WINDOW_MS = 100;

/** One pointer position seen during a drag: `y` in px, `t` in ms. */
export type DragSample = { y: number; t: number };

/**
 * Whether releasing the sheet dismisses it. `distance` is how far it was pulled
 * down in px (negative: up), `height` its height in px, `velocity` the downward
 * speed at release in px/ms (negative: moving up). Dismiss when pulled further
 * than a third of the height or flicked faster than `DISMISS_VELOCITY`; both
 * bounds are exclusive, so exactly on a bound springs back.
 */
export function shouldDismiss(drag: { distance: number; height: number; velocity: number }): boolean {
  const { distance, height, velocity } = drag;
  return (height > 0 && distance > height * DISMISS_DISTANCE_RATIO) || velocity > DISMISS_VELOCITY;
}

/**
 * Rubber band for pulling the sheet up: the on-screen travel (px, never above
 * `RUBBER_BAND_LIMIT`) for a finger that moved `distance` px above the start.
 * Starts at about half the finger's pace and flattens out. Zero or negative
 * input gives 0.
 */
export function dampUp(distance: number): number {
  if (!(distance > 0)) return 0;
  return RUBBER_BAND_LIMIT * (1 - 1 / ((distance * 0.55) / RUBBER_BAND_LIMIT + 1));
}

/**
 * The downward velocity (px/ms) at release, from the pointer samples of the
 * last `VELOCITY_WINDOW_MS` before `now`. A finger that paused before lifting
 * (fewer than two recent samples) has no velocity.
 */
export function releaseVelocity(samples: readonly DragSample[], now: number): number {
  const recent = samples.filter((s) => now - s.t <= VELOCITY_WINDOW_MS);
  if (recent.length < 2) return 0;
  const first = recent[0];
  const last = recent[recent.length - 1];
  const elapsed = last.t - first.t;
  return elapsed > 0 ? (last.y - first.y) / elapsed : 0;
}
