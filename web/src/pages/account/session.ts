// The signed-in User, shared by the header, the login page and the profile.
// A tiny external store, so no provider has to be added to the app shell.

import { useSyncExternalStore } from "react";

import { ApiError } from "../../api/client";
import i18n, { LANGUAGES, type Language } from "../../i18n";
import { getMe, signOutRequest, updateMe, type User } from "./api";

type Session = { loading: boolean; user: User | null };

let session: Session = { loading: true, user: null };
const listeners = new Set<() => void>();

function publish(next: Session) {
  session = next;
  listeners.forEach((listener) => listener());
}

export function useSession(): Session {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => session,
  );
}

function isLanguage(code: string | undefined): code is Language {
  return LANGUAGES.some((language) => language === code);
}

/** The signed-in User's saved language drives the interface. */
function applyLanguage(user: User) {
  if (user.language !== i18n.resolvedLanguage) void i18n.changeLanguage(user.language);
}

/** Records who is signed in (after login or a profile edit). */
export function setUser(user: User) {
  publish({ loading: false, user });
  applyLanguage(user);
}

let started = false;

/** Finds out once whether the visitor has a session, and starts remembering
 * language changes made while signed in. Idempotent. */
export function startSession() {
  if (started) return;
  started = true;

  getMe()
    .then(setUser)
    .catch((error: unknown) => {
      // 401 just means signed out; anything else leaves them signed out too.
      if (!(error instanceof ApiError)) console.warn(error);
      publish({ loading: false, user: null });
    });

  // The language switcher changes the interface; keep the saved language in step.
  i18n.on("languageChanged", () => {
    const { user } = session;
    const language = i18n.resolvedLanguage;
    if (user && isLanguage(language) && language !== user.language) {
      void updateMe({ language })
        .then(setUserKeepingLanguage)
        .catch(() => {});
    }
  });
}

/** Like `setUser` but does not push the saved language back onto the UI, so
 * quickly switching languages twice cannot flip the interface back. */
function setUserKeepingLanguage(user: User) {
  publish({ loading: false, user });
}

export async function signOut() {
  await signOutRequest();
  publish({ loading: false, user: null });
}
