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
  api<EventInfo>("/events/resolve", { method: "POST", body: { link } });
export const createLeague = (event_id: number, name: string) =>
  api<LeagueDetail>("/leagues", { method: "POST", body: { event_id, name } });
export const myLeagues = () => api<LeagueSummary[]>("/leagues");
export const getLeague = (id: number) => api<LeagueDetail>(`/leagues/${id}`);
export const reissueInvite = (id: number) =>
  api<{ invite_token: string }>(`/leagues/${id}/invite/reissue`, { method: "POST" });
export const leaveLeague = (id: number) => api<void>(`/leagues/${id}/leave`, { method: "POST" });
export const previewInvite = (token: string) =>
  api<InvitePreview>(`/invites/${encodeURIComponent(token)}`);
export const joinLeague = (token: string) =>
  api<{ id: number }>(`/invites/${encodeURIComponent(token)}/join`, { method: "POST" });

/** The Invite Link a League Admin shares. */
export const inviteUrl = (token: string) => `${window.location.origin}/join/${token}`;
