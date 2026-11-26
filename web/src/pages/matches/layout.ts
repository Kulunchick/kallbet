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

// -- Picking: what is still open, what comes next -------------------------------

type Team = { id: number; name: string };

type PickableCard = {
  status: string;
  winner: number | null;
  team_a: Team | null;
  team_b: Team | null;
  lock: { locks_at: string | null; locked: boolean; can_predict: boolean };
  my_prediction: number | null;
};

/** The same test the card uses to show its two team buttons, plus the clock. */
export function isOpenToPick(card: PickableCard, now: number): boolean {
  if (cardState(card) !== "upcoming" || !card.lock.can_predict || card.lock.locked) return false;
  if (card.team_a === null || card.team_b === null) return false;
  return card.lock.locks_at === null || Date.parse(card.lock.locks_at) > now;
}

export function unpickedOpen<C extends PickableCard>(cards: C[], now: number): C[] {
  return cards.filter((card) => card.my_prediction === null && isOpenToPick(card, now));
}

/** The earliest Lock still ahead among these cards (ms), or `null`. */
export function nextLockAt(cards: PickableCard[], now: number): number | null {
  let next: number | null = null;
  for (const card of cards) {
    if (card.lock.locks_at === null) continue;
    const at = Date.parse(card.lock.locks_at);
    if (at > now && (next === null || at < next)) next = at;
  }
  return next;
}

/** The unpicked open Match that Locks first (one without a Lock time goes last). */
export function nextUnpicked<C extends PickableCard>(cards: C[], now: number): C | null {
  let best: C | null = null;
  let bestAt = Infinity;
  for (const card of unpickedOpen(cards, now)) {
    const at = card.lock.locks_at === null ? Infinity : Date.parse(card.lock.locks_at);
    if (best === null || at < bestAt) {
      best = card;
      bestAt = at;
    }
  }
  return best;
}

/** What the next-up strip says. `hidden`: nothing is going on, so say nothing. */
export type StripModel =
  | { kind: "unpicked"; count: number; nextLock: number | null }
  | { kind: "allPicked"; nextLock: number | null }
  | { kind: "nothing" }
  | { kind: "hidden" };

export function stripModel(cards: PickableCard[], now: number): StripModel {
  const unpicked = unpickedOpen(cards, now);
  if (unpicked.length > 0) {
    return { kind: "unpicked", count: unpicked.length, nextLock: nextLockAt(unpicked, now) };
  }
  const open = cards.filter((card) => isOpenToPick(card, now));
  if (open.length > 0) return { kind: "allPicked", nextLock: nextLockAt(open, now) };
  const pending = cards.some((card) => {
    const state = cardState(card);
    return state === "upcoming" || state === "live";
  });
  return pending ? { kind: "nothing" } : { kind: "hidden" };
}

/** The team to put back when a pick is undone: the previous one, if there was one. */
export function undoTarget(card: PickableCard, newTeam: number): Team | null {
  if (card.my_prediction === null || card.my_prediction === newTeam) return null;
  return [card.team_a, card.team_b].find((team) => team?.id === card.my_prediction) ?? null;
}

// -- Progressive disclosure -----------------------------------------------------

type Section<C> = { matches: C[] };

/** Live Matches, and upcoming ones whose two teams are known: the Stage is under way. */
export function isActive(card: PickableCard): boolean {
  const state = cardState(card);
  return state === "live" || (state === "upcoming" && card.team_a !== null && card.team_b !== null);
}

/**
 * The Stage shown expanded: the first with an active Match; else the latest one
 * that has results; else the first. Later Stages with nothing active are
 * collapsed (`collapsedStage`).
 */
export function currentStage<C extends PickableCard>(
  stages: { ordinal: number; groups: Section<C>[] }[],
): number | null {
  if (stages.length === 0) return null;
  const cardsOf = (stage: { groups: Section<C>[] }) => stage.groups.flatMap((group) => group.matches);
  const ordinals = stages.map((stage) => stage.ordinal);
  const active = stages.filter((stage) => cardsOf(stage).some(isActive)).map((stage) => stage.ordinal);
  if (active.length > 0) return Math.min(...active);
  const begun = stages
    .filter((stage) => cardsOf(stage).some((card) => cardState(card) !== "upcoming"))
    .map((stage) => stage.ordinal);
  return begun.length > 0 ? Math.max(...begun) : Math.min(...ordinals);
}

export function collapsedStage<C extends PickableCard>(
  stage: { ordinal: number; groups: Section<C>[] },
  current: number | null,
): boolean {
  if (current === null || stage.ordinal <= current) return false;
  return !stage.groups.some((group) => group.matches.some(isActive));
}

/** Splits a Stage's groups into what is still going on and what is over (finished or Void). */
export function splitFinished<C extends PickableCard, G extends Section<C>>(
  groups: G[],
): { open: G[]; finished: G[]; finishedCount: number } {
  const over = (card: C) => {
    const state = cardState(card);
    return state === "finished" || state === "void";
  };
  const open: G[] = [];
  const finished: G[] = [];
  let finishedCount = 0;
  for (const group of groups) {
    const done = group.matches.filter(over);
    const rest = group.matches.filter((card) => !over(card));
    finishedCount += done.length;
    if (rest.length > 0) open.push({ ...group, matches: rest });
    if (done.length > 0) finished.push({ ...group, matches: done });
  }
  return { open, finished, finishedCount };
}

/** The earliest Scheduled Start among these cards (ISO as given), or `null`. */
export function earliestStart(cards: { scheduled_start: string | null }[]): string | null {
  let best: string | null = null;
  let bestAt = Infinity;
  for (const card of cards) {
    if (card.scheduled_start === null) continue;
    const at = Date.parse(card.scheduled_start);
    if (at < bestAt) {
      best = card.scheduled_start;
      bestAt = at;
    }
  }
  return best;
}

// -- Masonry --------------------------------------------------------------------

/** Space between cards, in px (Tailwind `gap-3`). */
export const MASONRY_GAP = 12;
/** The height of one implicit grid row, in px (`grid-auto-rows`). */
export const MASONRY_ROW = 4;

/**
 * How many `MASONRY_ROW` rows a card of `height` px (measured, never estimated)
 * spans in the masonry grid, the gap below it included. The grid has no row gap
 * of its own, so a card's area is always at least its height plus `gap`: cards
 * never overlap, and the space under each is `gap` to `gap + row - 1` px.
 */
export function masonrySpan(height: number, gap = MASONRY_GAP, row = MASONRY_ROW): number {
  const h = Number.isFinite(height) ? Math.max(0, height) : 0;
  return Math.max(1, Math.ceil((h + Math.max(0, gap)) / row));
}
