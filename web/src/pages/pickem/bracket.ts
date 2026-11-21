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
