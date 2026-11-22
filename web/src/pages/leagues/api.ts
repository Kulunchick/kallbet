import { api } from "../../api/client";

export type EventInfo = {
  id: number;
  name: string;
  starts_at: string | null;
  ends_at: string | null;
};
export type Role = "member" | "admin";
export type LeagueSummary = {
  id: number;
  name: string;
  event: EventInfo;
  my_role: Role;
  archived_at: string | null;
};
/** An entry of "My Leagues": my place (shared places share the number) and my Match Points. */
export type LeagueListItem = LeagueSummary & { my_place: number | null; my_points: number };
export type Member = {
  user_id: number;
  nickname: string;
  avatar_url: string | null;
  role: Role;
  joined_at: string;
};
export type LeagueDetail = LeagueSummary & {
  creator_id: number;
  is_creator: boolean;
  members: Member[];
  stages: { ordinal: number; name: string }[];
  matches_pending: boolean;
  stage_points: StagePoints[];
  /** Only League Admins receive it. */
  invite_token?: string;
};
export type InvitePreview = {
  id: number;
  name: string;
  event: EventInfo;
  member_count: number;
  already_member: boolean;
};

export const listEvents = () => api<EventInfo[]>("/events");
export const resolveEvent = (link: string) =>
  api<EventInfo>("/events/resolve", { method: "POST", success: "eventSelected", body: { link } });
export const createLeague = (event_id: number, name: string) =>
  api<LeagueDetail>("/leagues", { method: "POST", success: "leagueCreated", body: { event_id, name } });
export const myLeagues = () => api<LeagueListItem[]>("/leagues");
export const getLeague = (id: number) => api<LeagueDetail>(`/leagues/${id}`);
export type StagePoints = { stage: number; points: number };

export const saveStagePoints = (id: number, settings: StagePoints[]) =>
  api<void>(`/leagues/${id}/stage-points`, { method: "PUT", body: settings });
export const reissueInvite = (id: number) =>
  api<{ invite_token: string }>(`/leagues/${id}/invite/reissue`, { method: "POST" });
export const leaveLeague = (id: number) => api<void>(`/leagues/${id}/leave`, { method: "POST" });
export const kickMember = (league: number, user: number) =>
  api<void>(`/leagues/${league}/members/${user}`, { method: "DELETE" });
export const setMemberRole = (league: number, user: number, role: Role) =>
  api<void>(`/leagues/${league}/members/${user}/role`, { method: "PUT", body: { role } });
export const deleteLeague = (league: number, confirm_name: string) =>
  api<void>(`/leagues/${league}/delete`, { method: "POST", body: { confirm_name } });
export const previewInvite = (token: string) =>
  api<InvitePreview>(`/invites/${encodeURIComponent(token)}`);
export const joinLeague = (token: string) =>
  api<{ id: number }>(`/invites/${encodeURIComponent(token)}/join`, { method: "POST", success: "leagueJoined" });

/** The Invite Link a League Admin shares. */
export const inviteUrl = (token: string) => `${window.location.origin}/join/${token}`;
