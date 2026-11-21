import { useEffect, useRef, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Navigate, useNavigate, useSearchParams } from "react-router";

import { errorMessage } from "../../api/client";
import {
  devSignIn,
  getSignInOptions,
  signInWithTelegram,
  type SignInOptions,
  type TelegramAuth,
  type User,
} from "./api";
import { setUser, useSession } from "./session";

declare global {
  interface Window {
    onTelegramAuth?: (auth: TelegramAuth) => void;
  }
}

/** Renders Telegram's own sign-in button for the bot; it calls `onAuth` with the signed payload. */
function TelegramButton({ bot, onAuth }: { bot: string; onAuth: (auth: TelegramAuth) => void }) {
  const holder = useRef<HTMLDivElement>(null);

  useEffect(() => {
    window.onTelegramAuth = onAuth;
    const script = document.createElement("script");
    script.src = "https://telegram.org/js/telegram-widget.js?22";
    script.async = true;
    script.dataset.telegramLogin = bot;
    script.dataset.size = "large";
    script.dataset.radius = "12";
    script.dataset.requestAccess = "write";
    script.dataset.onauth = "onTelegramAuth(user)";
    holder.current?.replaceChildren(script);
    return () => {
      holder.current?.replaceChildren();
      delete window.onTelegramAuth;
    };
  }, [bot, onAuth]);

  return <div ref={holder} className="flex min-h-12 justify-center" />;
}

/** Where to go after signing in: `?next=` when it is a path on this site, else the start page. */
function safeNext(raw: string | null): string {
  return raw && /^\/(?![/\\])/.test(raw) ? raw : "/";
}

export function LoginPage() {
  const { t, i18n } = useTranslation("account");
  const navigate = useNavigate();
  const { user } = useSession();
  const next = safeNext(useSearchParams()[0].get("next"));
  const [options, setOptions] = useState<SignInOptions | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [nickname, setNickname] = useState("");

  useEffect(() => {
    getSignInOptions().then(setOptions).catch(setError);
  }, []);

  if (user) return <Navigate to={next} replace />;

  const done = (signedIn: User) => {
    setUser(signedIn);
    navigate(next, { replace: true });
  };
  const onAuth = (auth: TelegramAuth) => {
    setError(null);
    signInWithTelegram(auth, i18n.resolvedLanguage ?? "en")
      .then(done)
      .catch(setError);
  };
  const onDevSubmit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    devSignIn(nickname).then(done).catch(setError);
  };

  return (
    <div className="mx-auto flex max-w-md flex-col gap-6 py-16 text-center">
      <h1 className="text-2xl font-extrabold text-gold-gradient">{t("login.title")}</h1>
      <p className="text-mute">{t("login.subtitle")}</p>

      <div className="card flex flex-col gap-5 p-6">
        {options?.bot_username ? (
          <TelegramButton bot={options.bot_username} onAuth={onAuth} />
        ) : (
          options && <p className="text-sm text-mute">{t("login.notConfigured")}</p>
        )}

        {options?.dev_login && (
          <form onSubmit={onDevSubmit} className="flex flex-col gap-3 border-t border-line pt-5">
            <p className="text-xs uppercase tracking-wide text-mute">{t("login.devLabel")}</p>
            <input
              value={nickname}
              onChange={(event) => setNickname(event.target.value)}
              placeholder={t("profile.nickname")}
              className="rounded-xl border border-line bg-ink px-3 py-2.5 text-sm outline-none focus:border-emerald-a"
            />
            <button type="submit" className="btn-ghost" disabled={!nickname.trim()}>
              {t("login.devSubmit")}
            </button>
          </form>
        )}

        {error !== null && (
          <p role="alert" className="text-sm text-danger">
            {errorMessage(t, error)}
          </p>
        )}
      </div>
    </div>
  );
}
