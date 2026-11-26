// The Reveal moment: when a card should play its one-time Reveal animation.
// Pure logic plus a small tracker (no React, no imports), so it can be tested
// under plain node: `web/scripts/reveal.test.mjs`.
//
// A card animates only when this page session saw its Predictions hidden and
// then saw them revealed (after a poll, a Prediction or the Lock timer). A card
// that is already revealed the first time it renders never animates, and a
// Reveal that has played is remembered (in memory and, when available, in
// sessionStorage) so a remount or a reload does not replay it.

/**
 * - `hidden`: Predictions are still hidden.
 * - `just-revealed`: seen hidden, now revealed: play the Reveal (once).
 * - `settled`: revealed and nothing to play.
 */
export type RevealState = "hidden" | "just-revealed" | "settled";

/**
 * The next state of one card. `prev` is what this mounted card last
 * committed (or, on mount, `RevealTracker.initial`); `played` says whether
 * this Reveal already played somewhere (another mount, before a reload).
 */
export function nextRevealState(
  prev: RevealState | undefined,
  now: { revealed: boolean; played: boolean },
): RevealState {
  if (!now.revealed) return "hidden";
  if (prev === "hidden") return now.played ? "settled" : "just-revealed";
  if (prev === "just-revealed") return "just-revealed";
  return "settled";
}

/** Transition delays of the Reveal stagger, by position: non-uniform on purpose. */
export const REVEAL_DELAYS_MS = [40, 90, 120, 160] as const;

/** The stagger delay of the pick at `index`; picks past the fourth share the last step. */
export function revealDelayMs(index: number): number {
  const step = Math.min(Math.max(0, Math.floor(index)), REVEAL_DELAYS_MS.length - 1);
  return REVEAL_DELAYS_MS[step] ?? 160;
}

/** The key of one Match in one League: `league:match`. */
export function revealKey(league: string | number | undefined, match: number): string {
  return `${league ?? "-"}:${match}`;
}

/** How long the "Predictions revealed" cue stays before it fades. */
export const REVEAL_CUE_MS = 6000;
/** How long the cue takes to fade out. */
export const REVEAL_CUE_FADE_MS = 200;

/** The part of `Storage` the tracker uses. */
export type RevealStorage = Pick<Storage, "getItem" | "setItem">;

const STORAGE_KEY = "kallbet.revealsPlayed";
const STORAGE_LIMIT = 500;

export type RevealTracker = {
  /** The state a card starts from when it mounts. */
  initial(key: string): RevealState | undefined;
  /** Whether this Reveal already played (here or before a reload). */
  played(key: string): boolean;
  /** Record what a mounted card rendered. */
  commit(key: string, state: RevealState): void;
};

/**
 * A tracker of what this page session has seen. `storage` may be null or
 * throw on every call (private mode, blocked site data): the tracker then
 * works from memory alone.
 */
export function createRevealTracker(storage: RevealStorage | null): RevealTracker {
  const seen = new Map<string, "hidden" | "revealed">();
  let playedKeys: string[] | null = null;

  function load(): string[] {
    if (playedKeys) return playedKeys;
    playedKeys = [];
    try {
      const raw = storage?.getItem(STORAGE_KEY);
      const parsed: unknown = raw ? JSON.parse(raw) : [];
      if (Array.isArray(parsed)) playedKeys = parsed.filter((key) => typeof key === "string");
    } catch {
      // No storage: memory only.
    }
    return playedKeys;
  }

  function markPlayed(key: string) {
    const keys = load();
    if (keys.includes(key)) return;
    keys.push(key);
    if (keys.length > STORAGE_LIMIT) keys.splice(0, keys.length - STORAGE_LIMIT);
    try {
      storage?.setItem(STORAGE_KEY, JSON.stringify(keys));
    } catch {
      // No storage: memory only.
    }
  }

  return {
    initial(key) {
      // A Reveal another mount already played starts settled.
      return seen.get(key) === "hidden" ? "hidden" : undefined;
    },
    played(key) {
      return load().includes(key);
    },
    commit(key, state) {
      seen.set(key, state === "hidden" ? "hidden" : "revealed");
      if (state === "just-revealed") markPlayed(key);
    },
  };
}

function sessionStore(): RevealStorage | null {
  try {
    return typeof sessionStorage === "undefined" ? null : sessionStorage;
  } catch {
    return null;
  }
}

/** The page session's tracker. */
export const revealTracker: RevealTracker = createRevealTracker(sessionStore());
