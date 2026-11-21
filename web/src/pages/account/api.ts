import { api } from "../../api/client";
import type { Language } from "../../i18n";

/** The signed-in User, as `GET /api/me` returns it. */
export type User = {
  id: number;
  nickname: string;
  avatar_url: string | null;
  language: Language;
};

/** What the Telegram Login Widget hands the page (plus our `hash` check). */
export type TelegramAuth = Record<string, string | number>;

export type SignInOptions = {
  bot_username: string | null;
  dev_login: boolean;
};

export const getMe = () => api<User>("/me");

export const getSignInOptions = () => api<SignInOptions>("/auth/config");

export const signInWithTelegram = (auth: TelegramAuth, language: string) =>
  api<User>("/auth/telegram", { method: "POST", body: { auth, language } });

export const devSignIn = (nickname: string) =>
  api<User>("/dev/login", { method: "POST", body: { nickname } });

export const signOutRequest = () => api<void>("/auth/logout", { method: "POST" });

export const updateMe = (changes: { nickname?: string; language?: Language }) =>
  api<User>("/me", { method: "PATCH", body: changes });
