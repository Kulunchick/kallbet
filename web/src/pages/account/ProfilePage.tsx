import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Navigate } from "react-router";

import { errorMessage } from "../../api/client";
import { Avatar } from "../../components/Avatar";
import { LANGUAGES } from "../../i18n";
import { updateMe, type User } from "./api";
import { setUser, useSession } from "./session";

const NICKNAME_MAX = 20;
const NOTIFICATION_TYPES = ["reminder", "reveal", "digest"] as const;

export function ProfilePage() {
  const { loading, user } = useSession();
  if (loading) return null;
  if (!user) return <Navigate to="/login" replace />;
  return <Profile user={user} />;
}

function Profile({ user }: { user: User }) {
  const { t } = useTranslation("account");
  const [nickname, setNickname] = useState(user.nickname);
  const [error, setError] = useState<unknown>(null);
  const [saved, setSaved] = useState(false);

  const changed = nickname.trim() !== user.nickname;

  const [pending, setPending] = useState(false);

  async function save(changes: Parameters<typeof updateMe>[0], onSaved = setUser) {
    if (pending) return;
    setError(null);
    setPending(true);
    try {
      onSaved(await updateMe(changes));
    } catch (error) {
      setError(error);
    } finally {
      setPending(false);
    }
  }

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (!changed) return;
    setSaved(false);
    void save({ nickname }, (updated) => {
      setUser(updated);
      setNickname(updated.nickname);
      setSaved(true);
    });
  };

  return (
    <div className="mx-auto flex max-w-md flex-col gap-6 py-12">
      <div className="flex items-center gap-4">
        <Avatar url={user.avatar_url} nickname={user.nickname} className="size-16" />
        <h1 className="truncate text-2xl font-extrabold text-gold-gradient">{user.nickname}</h1>
      </div>

      <form onSubmit={onSubmit} className="card flex flex-col gap-3 p-5">
        <label htmlFor="nickname" className="text-sm font-semibold">
          {t("profile.nickname")}
        </label>
        <input
          id="nickname"
          value={nickname}
          maxLength={NICKNAME_MAX}
          disabled={pending}
          onChange={(event) => {
            setNickname(event.target.value);
            setSaved(false);
          }}
          className="rounded-xl border border-line bg-ink px-3 py-2.5 text-sm outline-none focus:border-emerald-a"
        />
        <p className="text-xs text-mute">{t("profile.nicknameHint")}</p>
        {error !== null && (
          <p role="alert" className="text-sm text-danger">
            {errorMessage(t, error)}
          </p>
        )}
        {saved && <p className="text-sm text-emerald-b">{t("profile.saved")}</p>}
        <button type="submit" className="btn-primary self-start" disabled={!changed || pending}>
          {t("profile.save")}
        </button>
      </form>

      <div className="card flex flex-col gap-3 p-5">
        <p className="text-sm font-semibold">{t("profile.language")}</p>
        <div className="flex gap-2">
          {LANGUAGES.map((language) => (
            <button
              key={language}
              type="button"
              lang={language}
              aria-pressed={language === user.language}
              disabled={pending}
              onClick={() => void save({ language })}
              className={language === user.language ? "btn-primary !py-2" : "btn-ghost !py-2"}
            >
              {t(`common:language.${language}`)}
            </button>
          ))}
        </div>
      </div>

      <div className="card flex flex-col gap-3 p-5">
        <p className="text-sm font-semibold">{t("profile.notifications")}</p>
        {NOTIFICATION_TYPES.map((type) => (
          <label key={type} className="flex items-start gap-3 text-sm">
            <input
              type="checkbox"
              checked={user.notifications[type]}
              disabled={pending}
              onChange={(event) =>
                void save({ notifications: { [type]: event.target.checked } })
              }
              className="mt-0.5 size-4 accent-emerald-500"
            />
            <span>{t(`profile.notify.${type}`)}</span>
          </label>
        ))}
        <p className="text-xs text-mute">{t("profile.notificationsHint")}</p>
      </div>
    </div>
  );
}
