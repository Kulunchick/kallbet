import { useTranslation } from "react-i18next";

import { Avatar } from "../../components/Avatar";
import type { MatchCard, RevealedPrediction, TeamRef } from "./api";

/**
 * Before Reveal: "N of M predicted". After Reveal: each Member's avatar on
 * the side of the team they picked (ADR-0003).
 */
export function PredictionsLine({ card }: { card: MatchCard }) {
  const { t } = useTranslation("matches");
  const { count, eligible, revealed } = card.predictions;
  const counter = (
    <span className="text-xs text-mute">{t("predictedCount", { n: count, m: eligible })}</span>
  );

  if (revealed === null) return <div className="text-center">{counter}</div>;

  return (
    <div className="flex flex-col gap-2 border-t border-line pt-3">
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-semibold uppercase tracking-wide text-gold-a">
          {t("revealed")}
        </span>
        {counter}
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Side team={card.team_a} picks={revealed} align="start" />
        <Side team={card.team_b} picks={revealed} align="end" />
      </div>
    </div>
  );
}

function Side({
  team,
  picks,
  align,
}: {
  team: TeamRef | null;
  picks: RevealedPrediction[];
  align: "start" | "end";
}) {
  const { t } = useTranslation("matches");
  const here = team === null ? [] : picks.filter((p) => p.team_id === team.id);
  return (
    <div className={`flex flex-col gap-1 ${align === "end" ? "items-end" : "items-start"}`}>
      <span className="max-w-full truncate text-xs text-mute">
        {team?.name ?? t("tbd")} · {here.length}
      </span>
      {here.length === 0 ? (
        <span className="text-xs text-mute">{t("nobodyPicked")}</span>
      ) : (
        <ul className={`flex flex-wrap gap-1 ${align === "end" ? "justify-end" : ""}`}>
          {here.map((p) => (
            <li key={p.user_id} title={p.nickname}>
              <Avatar url={p.avatar_url} nickname={p.nickname} className="size-7" />
              <span className="sr-only">{p.nickname}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
