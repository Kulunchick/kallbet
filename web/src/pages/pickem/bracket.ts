// Pure bracket logic for the Playoffs Pick'em (no React, no imports, so it
// can be tested under plain node). Slots follow the API: `rounds_before_final`
// is 0 for the final; the winners of slots 2n and 2n+1 of round r+1 meet in
// slot n of round r. A "first slot" (`leaf`) takes its teams from the data;
// every other slot takes its two participants from the picked winners of the
// slots feeding it.

export type Slot = {
  rounds_before_final: number;
  slot: number;
  team_a: number | null;
  team_b: number | null;
  leaf: boolean;
  winner: number | null;
};

/** Picked winners by slot key (see `slotKey`). */
export type Winners = Record<string, number>;

export const slotKey = (round: number, slot: number) => `${round}:${slot}`;

/** The slot's two participants as far as they are known. */
export function participants(slot: Slot, winners: Winners): [number | null, number | null] {
  if (slot.leaf) return [slot.team_a, slot.team_b];
  const { rounds_before_final: r, slot: s } = slot;
  return [winners[slotKey(r + 1, 2 * s)] ?? null, winners[slotKey(r + 1, 2 * s + 1)] ?? null];
}

const firstRoundFirst = (slots: Slot[]) =>
  [...slots].sort((a, b) => b.rounds_before_final - a.rounds_before_final || a.slot - b.slot);

/** Drops every pick whose team is no longer a participant of its slot. */
export function normalize(slots: Slot[], winners: Winners): Winners {
  const next: Winners = {};
  for (const slot of firstRoundFirst(slots)) {
    const key = slotKey(slot.rounds_before_final, slot.slot);
    const picked = winners[key];
    if (picked !== undefined && participants(slot, next).includes(picked)) next[key] = picked;
  }
  return next;
}

/**
 * Picks `team` as the winner of a slot. The pick propagates into the next
 * round (it becomes a participant there); and when it replaces an earlier
 * pick, every later pick that depended on the old winner is cleared.
 */
export function setWinner(slots: Slot[], winners: Winners, round: number, slot: number, team: number): Winners {
  return normalize(slots, { ...winners, [slotKey(round, slot)]: team });
}

/** True when every slot has a pick. */
export function isComplete(slots: Slot[], winners: Winners): boolean {
  return slots.length > 0 && slots.every((s) => winners[slotKey(s.rounds_before_final, s.slot)] !== undefined);
}

export type BracketPick = { rounds_before_final: number; slot: number; team_id: number };

export const toPicks = (slots: Slot[], winners: Winners): BracketPick[] =>
  firstRoundFirst(slots).flatMap((s) => {
    const team_id = winners[slotKey(s.rounds_before_final, s.slot)];
    return team_id === undefined ? [] : [{ rounds_before_final: s.rounds_before_final, slot: s.slot, team_id }];
  });

export const fromPicks = (picks: BracketPick[]): Winners =>
  Object.fromEntries(picks.map((p) => [slotKey(p.rounds_before_final, p.slot), p.team_id]));

// -- Numbering, progress and layout (the Playoffs board) ---------------------

/** The distinct rounds, first round first (the final, `0`, is last). */
export const roundsOf = (slots: Slot[]): number[] =>
  [...new Set(slots.map((s) => s.rounds_before_final))].sort((a, b) => b - a);

/** The round's slots in bracket order. */
export const slotsOfRound = (slots: Slot[], round: number): Slot[] =>
  slots.filter((s) => s.rounds_before_final === round).sort((a, b) => a.slot - b.slot);

/** Match numbers by slot key: first round first, top to bottom, from 1. */
export function matchNumbers(slots: Slot[]): Record<string, number> {
  const numbers: Record<string, number> = {};
  firstRoundFirst(slots).forEach((s, i) => {
    numbers[slotKey(s.rounds_before_final, s.slot)] = i + 1;
  });
  return numbers;
}

/**
 * The number of the Match whose winner fills `side` (0 = top, 1 = bottom) of a
 * slot that is not a first slot, or null when the data has no such Match.
 */
export function feederNumber(slots: Slot[], slot: Slot, side: 0 | 1): number | null {
  if (slot.leaf) return null;
  const key = slotKey(slot.rounds_before_final + 1, 2 * slot.slot + side);
  return matchNumbers(slots)[key] ?? null;
}

/** Picks placed, counting only slots that exist. */
export const placedPicks = (slots: Slot[], winners: Winners): number =>
  slots.filter((s) => winners[slotKey(s.rounds_before_final, s.slot)] !== undefined).length;

/** Picks placed in one round and the round's size. */
export function roundProgress(slots: Slot[], winners: Winners, round: number): { done: number; total: number } {
  const inRound = slotsOfRound(slots, round);
  return {
    done: inRound.filter((s) => winners[slotKey(round, s.slot)] !== undefined).length,
    total: inRound.length,
  };
}

/** The first round that still lacks a pick; the first round when all are picked. */
export function firstIncompleteRound(slots: Slot[], winners: Winners): number {
  const rounds = roundsOf(slots);
  return rounds.find((r) => {
    const { done, total } = roundProgress(slots, winners, r);
    return done < total;
  }) ?? rounds[0] ?? 0;
}

/**
 * Where a slot sits in the desktop grid (rows counted from 1, below the header
 * row): a first-round slot takes one row, each later round spans twice as many
 * rows as the round before, so a card is centered between the two cards that
 * feed it.
 */
export function gridPlacement(round: number, slot: number, firstRound: number): { row: number; span: number } {
  const span = 2 ** Math.max(0, firstRound - round);
  return { row: slot * span + 1, span };
}

/** A settled slot's verdict on my pick: null while unsettled or without a pick. */
export function verdict(slotWinner: number | null, picked: number | undefined): "correct" | "wrong" | null {
  if (slotWinner === null || picked === undefined) return null;
  return slotWinner === picked ? "correct" : "wrong";
}
