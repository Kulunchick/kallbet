import { api } from "../../api/client";
import type { MatchStatus, StageKind, TeamRef } from "../matches/api";

export type Field = "scheduled_start" | "result" | "teams" | "hidden" | "added";

/** A Match as the history names it (`MatchRef` in src/api/overrides.rs). */
export type MatchRef = {
  id: number;
  stage: number;
  team_a: TeamRef | null;
  team_b: TeamRef | null;
  scheduled_start: string | null;
  status: MatchStatus;
};

/** One entry of a League's Override history (`EntryView`). */
export type Entry = {
  id: number;
  match_id: number;
  match: MatchRef | null;
  field: Field;
  cleared: boolean;
  scheduled_start: string | null;
  winner: TeamRef | null;
  void: boolean;
  team_a: TeamRef | null;
  team_b: TeamRef | null;
  author: { user_id: number; nickname: string } | null;
  created_at: string;
};

export type OverridesView = {
  /** Newest first. */
  entries: Entry[];
  teams: TeamRef[];
  stages: { ordinal: number; kind: StageKind; name: string }[];
  hidden: MatchRef[];
};

export type Warnings = {
  conflicts: {
    match_id: number;
    match: MatchRef | null;
    kept_source: string;
    kept_winner: TeamRef | null;
    other_source: string;
    other_winner: TeamRef | null;
    detected_at: string;
  }[];
  unmatched: {
    source: string;
    kind: string;
    external_id: string;
    description: string;
    seen_at: string;
  }[];
};

export type OverrideBody =
  | { field: "scheduled_start"; scheduled_start: string }
  | { field: "result"; winner: number }
  | { field: "result"; void: true }
  | { field: "teams"; team_a: number; team_b: number }
  | { field: "hidden" };

export type NewMatch = {
  stage: number;
  team_a: number;
  team_b: number;
  scheduled_start: string;
  format: "bo1" | "bo3" | "bo5";
};

export const getOverrides = (league: number) =>
  api<OverridesView>(`/leagues/${league}/overrides`);
export const getWarnings = (league: number) => api<Warnings>(`/leagues/${league}/data-warnings`);
export const setOverride = (league: number, match: number, body: OverrideBody) =>
  api<Entry>(`/leagues/${league}/matches/${match}/overrides`, { method: "POST", body });
export const removeOverride = (league: number, match: number, field: Field) =>
  api<Entry>(`/leagues/${league}/matches/${match}/overrides/${field}`, { method: "DELETE" });
export const addMatch = (league: number, body: NewMatch) =>
  api<{ id: number }>(`/leagues/${league}/matches`, { method: "POST", body });

/** The entries whose Override is still in force (the latest of its Match and field). */
export function inForce(entries: Entry[]): Entry[] {
  const seen = new Set<string>();
  const result: Entry[] = [];
  for (const e of entries) {
    const key = `${e.match_id}:${e.field}`;
    if (seen.has(key)) continue;
    seen.add(key);
    if (!e.cleared && e.field !== "added") result.push(e);
  }
  return result;
}
