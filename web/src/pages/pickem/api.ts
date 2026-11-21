import { api } from "../../api/client";

export type TeamRef = { id: number; name: string };

/** A Swiss Pick'em: team ids by kind (`shape: "swiss"`; the Playoffs shape is added by #15). */
export type SwissPickem = {
  shape: "swiss";
  three_nil: number[];
  zero_three: number[];
  advance: number[];
  updated_at: string;
};

export type MemberPickem = SwissPickem & {
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  points: number;
};

export type PickemView = {
  stage: { ordinal: number; kind: "swiss" | "playoffs" | "groups"; name: string };
  teams: TeamRef[];
  lock: { locks_at: string | null; locked_at: string | null; locked: boolean };
  my_pickem: SwissPickem | null;
  /** Everyone's Pick'ems: null until the Stage's Lock. */
  pickems: MemberPickem[] | null;
};

export type LeaderboardEntry = {
  place: number;
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  points: number;
};

export type SwissBody = Pick<SwissPickem, "three_nil" | "zero_three" | "advance">;

export const getPickem = (league: number, stage: number) =>
  api<PickemView>(`/leagues/${league}/stages/${stage}/pickem`);
export const savePickem = (league: number, stage: number, body: SwissBody) =>
  api<SwissPickem>(`/leagues/${league}/stages/${stage}/pickem`, { method: "PUT", body });
export const getLeaderboard = (league: number) =>
  api<{ entries: LeaderboardEntry[] }>(`/leagues/${league}/pickem-leaderboard`);
