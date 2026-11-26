import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useSession } from "../account/session";
import { joinLeague, previewInvite } from "./api";
import { useLoad } from "../../api/useLoad";

/** Opened from an Invite Link: shows the League and joins on confirmation. */
export function JoinPage() {
  const { t } = useTranslation("leagues");
  const token = useParams().token ?? "";
  const navigate = useNavigate();
  const session = useSession();
  const { result } = useLoad(() => previewInvite(token), [token, session.user?.id]);
  const [error, setError] = useState<unknown>(null);

  const [joining, setJoining] = useState(false);

  async function join() {
    if (joining) return;
    setJoining(true);
    setError(null);
    try {
      const { id } = await joinLeague(token);
      navigate(`/leagues/${id}`);
    } catch (e) {
      setError(e);
    } finally {
      setJoining(false);
    }
  }

  return (
    <div className="mx-auto flex max-w-md flex-col gap-5 py-16">
      {!session.loading && !session.user && (
        <div className="card flex flex-col gap-4 p-6">
          <p>{t("join.signInNeeded")}</p>
          <Link to={`/login?next=${encodeURIComponent(`/join/${token}`)}`} className="btn-primary self-start">
            {t("join.signIn")}
          </Link>
        </div>
      )}
      {(session.loading || session.user) && result.state === "loading" && (
        <p className="text-mute">{t("loading")}</p>
      )}
      {session.user && result.state === "error" && (
        <div className="card flex flex-col gap-4 p-6">
          <p className="text-danger">{errorMessage(t, result.error)}</p>
          <Link to="/" className="btn-ghost self-start">
            {t("join.home")}
          </Link>
        </div>
      )}
      {session.user && result.state === "ready" && (
        <div className="card flex flex-col gap-4 p-6">
          <p className="text-sm text-mute">{t("join.invited")}</p>
          <h1 className="font-display text-2xl font-extrabold text-fg">{result.data.name}</h1>
          <p className="text-mute">
            {result.data.event.name} · {t("members.title", { count: result.data.member_count })}
          </p>
          {error !== null && <p className="text-danger">{errorMessage(t, error)}</p>}
          {result.data.already_member ? (
            <Link to={`/leagues/${result.data.id}`} className="btn-primary">
              {t("join.open")}
            </Link>
          ) : (
            <button type="button" className="btn-primary" disabled={joining} onClick={join}>
              {t(joining ? "joining" : "join.button")}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
