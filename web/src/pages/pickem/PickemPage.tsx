import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { Avatar } from "../../components/Avatar";
import { Medal } from "../../components/Medal";
import { getLeague } from "../leagues/api";
import { useLoad } from "../leagues/useLoad";
import { getMatches } from "../matches/api";
import { getLeaderboard } from "./api";
import { PickemStage } from "./PickemStage";
import { PlayoffsStage } from "./PlayoffsStage";

/** The League's Pick'em: the Pick'em Leaderboard and one section per Swiss Stage. */
export function PickemPage() {
  const { t } = useTranslation("pickem");
  const league = Number(useParams().id);
  const stages = useLoad(() => getMatches(league), [league]);
  const board = useLoad(() => getLeaderboard(league), [league]);
  const archived = useLoad(() => getLeague(league), [league]).result;
  const isArchived = archived.state === "ready" && archived.data.archived_at !== null;

  return (
    <div className="flex flex-col gap-8 py-10">
      <header className="flex flex-col gap-1">
        <Link to={`/leagues/${league}`} className="text-sm text-mute hover:text-fg">
          ← {t("backToLeague")}
        </Link>
        <h1 className="text-2xl font-extrabold text-gold-gradient">{t("title")}</h1>
      </header>

      <section className="flex flex-col gap-3">
        <h2 className="text-lg font-bold text-gold-gradient">{t("leaderboard")}</h2>
        {board.result.state === "error" && (
          <p className="text-danger">{errorMessage(t, board.result.error)}</p>
        )}
        {board.result.state === "ready" && (
          <ol className="card divide-y divide-line">
            {board.result.data.entries.map((e) => (
              <li key={e.user_id} className="flex items-center gap-3 p-3">
                <span className="w-6 text-center font-display text-gold-a">{e.place}</span>
                {isArchived && <Medal place={e.place} />}
                <Avatar url={e.avatar_url} nickname={e.nickname} className="size-8" />
                <span className="font-semibold">{e.nickname}</span>
                <span className="ml-auto font-display text-gold-a">{e.points}</span>
              </li>
            ))}
          </ol>
        )}
      </section>

      {stages.result.state === "error" && (
        <p className="text-danger">{errorMessage(t, stages.result.error)}</p>
      )}
      {stages.result.state === "ready" &&
        stages.result.data.stages
          .filter((s) => s.kind === "swiss" || s.kind === "playoffs")
          .map((s) => (
            <section key={s.ordinal} className="flex flex-col gap-3">
              <h2 className="text-lg font-bold text-gold-gradient">{s.name}</h2>
              {s.kind === "playoffs" ? (
                <PlayoffsStage league={league} stage={s.ordinal} onSaved={board.reload} />
              ) : (
                <PickemStage league={league} stage={s.ordinal} onSaved={board.reload} />
              )}
            </section>
          ))}
    </div>
  );
}
