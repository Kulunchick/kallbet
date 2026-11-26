// Pure logic of the Swiss Pick'em board (no React, no imports, so it can be
// tested under plain node). A draft maps a team id to the group it is placed in;
// a team is placed at most once.

export type Choice = "three_nil" | "advance" | "zero_three";

/** The groups in display order. */
export const CHOICES: Choice[] = ["three_nil", "advance", "zero_three"];
/** How many teams a group takes. */
export const LIMITS: Record<Choice, number> = { three_nil: 2, advance: 6, zero_three: 2 };
/** Picks needed for a complete Swiss Pick'em. */
export const TOTAL = 10;

export type Draft = Record<number, Choice>;

export const countIn = (draft: Draft, choice: Choice): number =>
  Object.values(draft).filter((c) => c === choice).length;

export const isFull = (draft: Draft, choice: Choice): boolean => countIn(draft, choice) >= LIMITS[choice];

export const placedCount = (draft: Draft): number => Object.keys(draft).length;

/** How many more teams must be placed before the Pick'em can be saved. */
export const picksLeft = (draft: Draft): number => Math.max(0, TOTAL - placedCount(draft));

/** True when every group holds exactly its limit (so 2 + 6 + 2 teams). */
export const isSwissComplete = (draft: Draft): boolean => CHOICES.every((c) => countIn(draft, c) === LIMITS[c]);

export type Availability = { choice: Choice; count: number; limit: number; full: boolean };

/** What each group offers right now: its fill and whether it is full. */
export function availability(draft: Draft): Availability[] {
  return CHOICES.map((choice) => ({
    choice,
    count: countIn(draft, choice),
    limit: LIMITS[choice],
    full: isFull(draft, choice),
  }));
}

/**
 * Places a team. Refused (the same draft comes back) when the group is full or
 * the team is already in it; a team in another group moves.
 */
export function place(draft: Draft, team: number, choice: Choice): Draft {
  if (draft[team] === choice || isFull(draft, choice)) return draft;
  return { ...draft, [team]: choice };
}

/** Takes a team out of its group. */
export function unplace(draft: Draft, team: number): Draft {
  if (draft[team] === undefined) return draft;
  const next = { ...draft };
  delete next[team];
  return next;
}

/** The teams not placed yet, in their given order. */
export function unplaced<T extends { id: number }>(teams: T[], draft: Draft): T[] {
  return teams.filter((team) => draft[team.id] === undefined);
}

/** The team ids in a group, in the teams' order. */
export function inGroup<T extends { id: number }>(teams: T[], draft: Draft, choice: Choice): T[] {
  return teams.filter((team) => draft[team.id] === choice);
}

/**
 * Where focus goes after `placed` leaves the pool: the tile that took its place
 * (the next one), else the one before it, else nothing (the pool is empty).
 */
export function nextPoolFocus(pool: number[], placed: number): number | null {
  const index = pool.indexOf(placed);
  if (index === -1) return pool[0] ?? null;
  return pool[index + 1] ?? pool[index - 1] ?? null;
}

export type SwissLists = { three_nil: number[]; advance: number[]; zero_three: number[] };

/** The draft that holds a saved Pick'em. */
export function draftFrom(saved: SwissLists | null | undefined): Draft {
  const draft: Draft = {};
  if (saved) for (const choice of CHOICES) for (const team of saved[choice]) draft[team] = choice;
  return draft;
}

/** The request body: team ids by group, in the teams' order. */
export function toBody<T extends { id: number }>(teams: T[], draft: Draft): SwissLists {
  const ids = (choice: Choice) => inGroup(teams, draft, choice).map((team) => team.id);
  return { three_nil: ids("three_nil"), advance: ids("advance"), zero_three: ids("zero_three") };
}
