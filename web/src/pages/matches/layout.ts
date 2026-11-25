// Pure layout logic of the Matches page (no React, no imports, so it can be
// tested under plain node: `web/scripts/matches-layout.test.mjs`).

/** What a card's status line says. A Void Match is cancelled, or finished without a winner. */
export type CardState = "upcoming" | "live" | "finished" | "void";

export function cardState(card: { status: string; winner: number | null }): CardState {
  if (card.status === "cancelled") return "void";
  if (card.status === "finished") return card.winner === null ? "void" : "finished";
  if (card.status === "live") return "live";
  return "upcoming";
}

/**
 * How a revealed Prediction is marked on a card. A Bot User's carries a bot
 * badge, and until Hard Lock it is preliminary: the bot may still change it.
 */
export type PickMark = "human" | "bot" | "bot_preliminary";

type MarkedPick = { is_bot?: boolean; preliminary?: boolean };

export function pickMark(pick: MarkedPick): PickMark {
  if (!pick.is_bot) return "human";
  return pick.preliminary ? "bot_preliminary" : "bot";
}

/** Whether a card must say that bot picks may still change. */
export function hasPreliminaryPick(revealed: MarkedPick[] | null): boolean {
  return revealed !== null && revealed.some((pick) => pickMark(pick) === "bot_preliminary");
}

type DatedMatch = {
  status: string;
  scheduled_start: string | null;
  actual_start: string | null;
};

export function compareMatchOrder(a: DatedMatch, b: DatedMatch): number {
  const priority = (match: DatedMatch) =>
    match.status === "live" ? 0 : match.status === "upcoming" ? 1 : 2;
  const order = priority(a) - priority(b);
  if (order !== 0) return order;

  const time = (match: DatedMatch) => {
    const start = match.actual_start ?? match.scheduled_start;
    return start === null ? null : Date.parse(start);
  };
  const aTime = time(a);
  const bTime = time(b);
  if (aTime === null) return bTime === null ? 0 : 1;
  if (bTime === null) return -1;
  return a.status === "upcoming" ? aTime - bTime : bTime - aTime;
}

/** Orders sections by their first match and orders the matches within them. */
export function orderMatchGroups<C extends DatedMatch, G extends { matches: C[] }>(groups: G[]): G[] {
  return groups.map((group) => ({ ...group, matches: [...group.matches].sort(compareMatchOrder) }))
    .sort((a, b) => {
      const first = a.matches[0];
      const second = b.matches[0];
      if (!first) return second ? 1 : 0;
      return second ? compareMatchOrder(first, second) : -1;
    });
}

/** A Playoffs round as the API groups it. */
export type RoundGroup<C> = { rounds_before_final: number; matches: C[] };

/** One column of the bracket drawing: a round, with one cell per bracket slot. */
export type BracketColumn<C> = { round: number; cells: (C | null)[] };

/** Deeper rounds than this are not drawn as a bracket (2^7 = 128 slots). */
const MAX_DEPTH = 7;

/**
 * Lays a Playoffs Stage's rounds out as a bracket, first round first. The
 * deepest round R that has a Match gives 2^R cells, each later round half as
 * many, down to the final's one; a cell is the Match in that slot, or `null`
 * while the data has no Match there yet. Winners of cells 2n and 2n+1 meet
 * in cell n of the next column. Cards without a slot, or outside the tree,
 * are returned in `extra` so nothing is hidden.
 */
export function bracketColumns<C extends { bracket_slot: number | null }>(
  rounds: RoundGroup<C>[],
): { columns: BracketColumn<C>[]; extra: C[] } {
  const extra: C[] = [];
  const placed = rounds.filter((r) => r.rounds_before_final <= MAX_DEPTH && r.matches.length > 0);
  for (const r of rounds) if (!placed.includes(r)) extra.push(...r.matches);
  if (placed.length === 0) return { columns: [], extra };

  const depth = Math.max(...placed.map((r) => r.rounds_before_final));
  const columns: BracketColumn<C>[] = [];
  for (let round = depth; round >= 0; round--) {
    const cells: (C | null)[] = Array.from({ length: 2 ** round }, () => null);
    for (const card of placed.find((r) => r.rounds_before_final === round)?.matches ?? []) {
      const slot = card.bracket_slot;
      if (slot === null || slot >= cells.length || cells[slot] !== null) extra.push(card);
      else cells[slot] = card;
    }
    columns.push({ round, cells });
  }
  return { columns, extra };
}
