import { api } from "../../api/client";
import type { EventInfo } from "../leagues/api";

export type DataSource = "hltv" | "pandascore" | "liquipedia";
export type MatchStatus = "upcoming" | "live" | "finished" | "cancelled";
export type StageKind = "swiss" | "playoffs" | "groups";

export type TeamRef = { id: number; name: string };

export type RevealedPrediction = {
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  team_id: number;
};

/** "N of M predicted"; `revealed` stays null until Reveal (`PredictionsView`). */
export type Predictions = {
  count: number;
  eligible: number;
  revealed_at: string | null;
  revealed: RevealedPrediction[] | null;
};

/** One Match as the signed-in Member sees it (`MatchCard` in src/api/matches.rs). */
export type MatchCard = {
  id: number;
  team_a: TeamRef | null;
  team_b: TeamRef | null;
  format: "bo1" | "bo3" | "bo5";
  /** Slot within its Playoffs round (`null` outside the Playoffs). */
  bracket_slot: number | null;
  scheduled_start: string | null;
  actual_start: string | null;
  status: MatchStatus;
  winner: number | null;
  score: { team_a: number; team_b: number } | null;
  lock: { locks_at: string | null; locked_at: string | null; locked: boolean };
  my_prediction: number | null;
  predictions: Predictions;
  my_result: "pending" | "correct" | "wrong" | "missing" | "void";
  my_points: number;
  /** `null` for a Match a League Admin added by hand. */
  source: DataSource | null;
  updated_at: string;
  /** League Overrides in force on this Match (#13). */
  overridden: ("scheduled_start" | "result" | "teams")[];
  added_by_hand: boolean;
};

export type MatchGroup =
  | { kind: "record"; wins: number; losses: number; matches: MatchCard[] }
  | { kind: "playoff_round"; rounds_before_final: number; matches: MatchCard[] }
  | { kind: "unplaced"; matches: MatchCard[] };

export type StageView = {
  ordinal: number;
  kind: StageKind;
  name: string;
  groups: MatchGroup[];
};

export type MatchesView = { event: EventInfo; stages: StageView[] };

export type Prediction = { match_id: number; team_id: number; updated_at: string };

export const getMatches = (league: number) => api<MatchesView>(`/leagues/${league}/matches`);
export const predict = (league: number, match: number, team_id: number) =>
  api<Prediction>(`/leagues/${league}/matches/${match}/prediction`, {
    method: "PUT",
    body: { team_id },
  });
