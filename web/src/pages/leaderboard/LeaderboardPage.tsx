import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useSession } from "../account/session";
import { Medal } from "../../components/Medal";
import { getLeague } from "../leagues/api";
import { Avatar } from "../leagues/Avatar";
import { useLoad } from "../leagues/useLoad";
import { getLeaderboard } from "./api";

/** How often the page refetches, so a result shows up without a reload. */
const REFRESH_MS = 60_000;

export function LeaderboardPage() {
  const { t } = useTranslation("leaderboard");
  const league = Number(useParams().id);
  const { result, reload } = useLoad(() => getLeaderboard(league), [league]);
  const archived = useLoad(() => getLeague(league), [league]).result;
  const isArchived = archived.state === "ready" && archived.data.archived_at !== null;
  const me = useSession().user?.id;

  useEffect(() => {
    const id = window.setInterval(reload, REFRESH_MS);
    return () => window.clearInterval(id);
  }, [reload]);

  const back = (
    <Link to={`/leagues/${league}`} className="text-sm text-mute hover:text-fg">
      ← {t("backToLeague")}
    </Link>
  );

  if (result.state === "loading") return <p className="py-10 text-mute">{t("loading")}</p>;
  if (result.state === "error") {
    return (
      <div className="flex flex-col gap-4 py-10">
        {back}
        <p className="text-danger">{errorMessage(t, result.error)}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 py-10">
      <header className="flex flex-col gap-1">
        {back}
        <h1 className="text-2xl font-extrabold text-gold-gradient">{t("title")}</h1>
      </header>

      <div className="card overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-line text-left text-xs uppercase tracking-wide text-mute">
              <th scope="col" className="px-3 py-3 text-center">
                {t("place")}
              </th>
              <th scope="col" className="px-3 py-3">
                {t("member")}
              </th>
              <th scope="col" className="px-3 py-3 text-right">
                {t("points")}
              </th>
              <th scope="col" className="px-3 py-3 text-right">
                {t("playoffPoints")}
              </th>
              <th scope="col" className="px-3 py-3 text-right">
                {t("percent")}
              </th>
            </tr>
          </thead>
          <tbody>
            {result.data.rows.map((row) => {
              const leader = row.place === 1;
              const mine = row.user_id === me;
              return (
                <tr
                  key={row.user_id}
                  aria-current={mine ? "true" : undefined}
                  className={`border-b border-line last:border-b-0 ${mine ? "bg-raised" : ""}`}
                >
                  <td
                    className={`px-3 py-3 text-center font-display font-bold ${leader ? "text-gold-a text-glow-gold" : "text-mute"}`}
                  >
                    {isArchived && <Medal place={row.place} />} {row.place}
                  </td>
                  <td className="px-3 py-3">
                    <span className="flex items-center gap-3">
                      <Avatar nickname={row.nickname} url={row.avatar_url} className="size-8" />
                      <span
                        className={`min-w-0 truncate font-semibold ${leader ? "text-gold-a" : ""} ${mine ? "text-emerald-b" : ""}`}
                      >
                        {row.nickname}
                      </span>
                      {mine && (
                        <span className="text-xs font-semibold uppercase tracking-wide text-emerald-b">
                          {t("you")}
                        </span>
                      )}
                    </span>
                  </td>
                  <td
                    className={`px-3 py-3 text-right font-display font-bold ${leader ? "text-gold-a" : ""}`}
                  >
                    {row.points}
                  </td>
                  <td className="px-3 py-3 text-right text-mute">{row.playoff_points}</td>
                  <td className="px-3 py-3 text-right text-mute">
                    {row.percent_correct === null ? "—" : `${row.percent_correct}%`}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className="text-xs text-mute">{t("hint")}</p>
    </div>
  );
}
