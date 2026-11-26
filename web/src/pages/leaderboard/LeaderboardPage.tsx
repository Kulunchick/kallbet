import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useSession } from "../account/session";
import { LeagueHeader } from "../../components/LeagueHeader";
import { Medal } from "../../components/Medal";
import { getLeague } from "../leagues/api";
import { Avatar } from "../../components/Avatar";
import { useLoad } from "../../api/useLoad";
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

  const header = <LeagueHeader league={league} active="leaderboard" loaded={archived} />;

  if (result.state === "loading") {
    return (
      <div className="flex flex-col gap-6 py-4 md:py-8">
        {header}
        <p className="text-mute">{t("loading")}</p>
      </div>
    );
  }
  if (result.state === "error") {
    return (
      <div className="flex flex-col gap-6 py-4 md:py-8">
        {header}
        <p className="text-danger">{errorMessage(t, result.error)}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 py-4 md:py-8">
      {header}

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
                    className={`px-3 py-3 text-center font-display font-bold ${leader ? "text-gold-a" : "text-mute"}`}
                  >
                    {isArchived && <Medal place={row.place} />} {row.place}
                  </td>
                  <td className="px-3 py-3">
                    <span className="flex items-center gap-3">
                      <Avatar nickname={row.nickname} url={row.avatar_url} className="size-8" />
                      <span
                        className={`min-w-0 truncate font-semibold ${mine ? "text-emerald-b" : "text-fg"}`}
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
                  <td className="px-3 py-3 text-right font-display font-bold text-gold-a">
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
