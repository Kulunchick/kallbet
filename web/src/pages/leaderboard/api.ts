import { api } from "../../api/client";

/** One row of the Leaderboard (`Row` in src/api/leaderboard.rs). */
export type LeaderboardRow = {
  /** Members with equal Points share a place (1, 1, 3). */
  place: number;
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  points: number;
  playoff_points: number;
  /** `null` until a decided, non-Void Match has been predicted. */
  percent_correct: number | null;
};

export type Leaderboard = { rows: LeaderboardRow[] };

export const getLeaderboard = (league: number) =>
  api<Leaderboard>(`/leagues/${league}/leaderboard`);
