import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { Avatar } from "./Avatar";
import { getLeague, inviteUrl, leaveLeague, reissueInvite } from "./api";
import { useLoad } from "./useLoad";

export function LeaguePage() {
  const { t } = useTranslation("leagues");
  const id = Number(useParams().id);
  const navigate = useNavigate();
  const { result, reload } = useLoad(() => getLeague(id), [id]);
  const [error, setError] = useState<unknown>(null);
  const [copied, setCopied] = useState(false);

  async function run(action: () => Promise<unknown>, then: () => void) {
    setError(null);
    try {
      await action();
      then();
    } catch (e) {
      setError(e);
    }
  }

  if (result.state === "loading") return <p className="py-10 text-mute">{t("loading")}</p>;
  if (result.state === "error") {
    return (
      <div className="flex flex-col gap-4 py-10">
        <p className="text-danger">{errorMessage(t, result.error)}</p>
        <Link to="/leagues" className="btn-ghost self-start">
          {t("backToLeagues")}
        </Link>
      </div>
    );
  }

  const league = result.data;
  const token = league.invite_token;

  async function copy(value: string) {
    try {
      await navigator.clipboard.writeText(inviteUrl(value));
      setCopied(true);
    } catch {
      setCopied(false);
    }
  }

  return (
    <div className="flex flex-col gap-8 py-10">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-extrabold text-gold-gradient">{league.name}</h1>
        <p className="text-mute">{league.event.name}</p>
      </header>

      {token && (
        <section className="card flex flex-col gap-3 p-5">
          <h2 className="font-bold">{t("invite.title")}</h2>
          <input
            readOnly
            aria-label={t("invite.title")}
            value={inviteUrl(token)}
            onFocus={(e) => e.currentTarget.select()}
            className="rounded-xl border border-line bg-raised px-4 py-2.5 text-sm"
          />
          <div className="flex flex-wrap gap-2">
            <button type="button" className="btn-primary" onClick={() => copy(token)}>
              {copied ? t("invite.copied") : t("invite.copy")}
            </button>
            <button
              type="button"
              className="btn-ghost"
              onClick={() =>
                run(
                  () => reissueInvite(league.id),
                  () => {
                    setCopied(false);
                    reload();
                  },
                )
              }
            >
              {t("invite.reissue")}
            </button>
          </div>
          <p className="text-xs text-mute">{t("invite.reissueHint")}</p>
        </section>
      )}

      <section className="flex flex-col gap-3">
        <h2 className="font-bold">{t("members.title", { count: league.members.length })}</h2>
        <ul className="grid gap-2 sm:grid-cols-2">
          {league.members.map((m) => (
            <li key={m.user_id} className="card flex items-center gap-3 px-4 py-3">
              <Avatar nickname={m.nickname} url={m.avatar_url} />
              <span className="min-w-0 flex-1 truncate font-semibold">{m.nickname}</span>
              {m.role === "admin" && (
                <span className="text-xs font-semibold uppercase tracking-wide text-gold-a">
                  {t("role.admin")}
                </span>
              )}
            </li>
          ))}
        </ul>
      </section>

      {error !== null && <p className="text-danger">{errorMessage(t, error)}</p>}

      {!league.is_creator && (
        <div>
          <button
            type="button"
            className="btn-ghost"
            onClick={() => {
              if (window.confirm(t("leave.confirm"))) {
                void run(
                  () => leaveLeague(league.id),
                  () => navigate("/leagues"),
                );
              }
            }}
          >
            {t("leave.button")}
          </button>
        </div>
      )}
    </div>
  );
}
