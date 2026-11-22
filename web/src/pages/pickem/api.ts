import { api } from "../../api/client";
import type { BracketPick, Slot } from "./bracket";

export type TeamRef = { id: number; name: string };

/** A Swiss Pick'em: team ids by kind (`shape: "swiss"`). */
export type SwissPickem = {
  shape: "swiss";
  three_nil: number[];
  zero_three: number[];
  advance: number[];
  updated_at: string;
};

/** A Playoffs Pick'em: the picked winner of every bracket slot. */
export type PlayoffsPickem = {
  shape: "playoffs";
  picks: BracketPick[];
  updated_at: string;
};

export type Pickem = SwissPickem | PlayoffsPickem;

export type MemberPickem = Pickem & {
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  points: number;
};

export type PickemView = {
  stage: { ordinal: number; kind: "swiss" | "playoffs" | "groups"; name: string };
  teams: TeamRef[];
  lock: { locks_at: string | null; locked_at: string | null; locked: boolean };
  my_pickem: Pickem | null;
  /** Everyone's Pick'ems: null until the Stage's Lock. */
  pickems: MemberPickem[] | null;
  /** Playoffs Stages only. `ready` is false while first-round teams are TBD. */
  bracket?: { ready: boolean; slots: Slot[] };
};

export type LeaderboardEntry = {
  place: number;
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  points: number;
};

export type SwissBody = Pick<SwissPickem, "three_nil" | "zero_three" | "advance">;
export type BracketBody = { picks: BracketPick[] };

export const getPickem = (league: number, stage: number) =>
  api<PickemView>(`/leagues/${league}/stages/${stage}/pickem`);
export const savePickem = (league: number, stage: number, body: SwissBody | BracketBody) =>
  api<Pickem>(`/leagues/${league}/stages/${stage}/pickem`, { method: "PUT", success: "pickemSaved", body });
export const getLeaderboard = (league: number) =>
  api<{ entries: LeaderboardEntry[] }>(`/leagues/${league}/pickem-leaderboard`);
