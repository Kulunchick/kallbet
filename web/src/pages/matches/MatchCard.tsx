import { useTranslation } from "react-i18next";

import { TeamLogo } from "../../components/TeamLogo";
import { PredictionsLine } from "./PredictionsLine";
import type { MatchCard as Card, TeamRef } from "./api";
import { cardState } from "./layout";
import { formatAgo, formatSpan, formatStart } from "./time";

type Props = {
  card: Card;
  now: number;
  /** Set while this Match's Prediction is being saved. */
  saving: boolean;
  onPredict: (team: TeamRef) => void;
};

/**
 * One Match: teams (tap one to predict), format, Scheduled Start in local
 * time, Lock countdown, "N of M predicted" or the revealed avatars, and data
 * freshness. #7 adds the ✓/✗ next to the score.
 */
export function MatchCard({ card, now, saving, onPredict }: Props) {
  const { t, i18n } = useTranslation("matches");
  const language = i18n.resolvedLanguage ?? "en";
  const locksAt = card.lock.locks_at ? new Date(card.lock.locks_at).getTime() : null;
  const locked = card.lock.locked || (locksAt !== null && locksAt <= now);
  const canPredict = card.lock.can_predict && !locked && card.team_a !== null && card.team_b !== null && !saving;
  const isVoid = cardState(card) === "void";

  return (
    <article
      className={`card flex flex-col gap-3 p-4 ${card.my_prediction !== null && !locked ? "glow-emerald" : ""} ${isVoid ? "opacity-75" : ""}`}
    >
      <header className="flex flex-wrap items-center justify-between gap-2 text-xs text-mute">
        <span className="flex items-center gap-2">
          <span className="font-semibold uppercase tracking-wide">{t(`format.${card.format}`)}</span>
          {isVoid && (
            <span className="rounded-full border border-line bg-raised px-2 py-0.5 font-bold uppercase tracking-wide text-fg">
              {t("void.badge")}
            </span>
          )}
        </span>
        <span>
          {card.scheduled_start ? formatStart(card.scheduled_start, language) : t("startUnknown")}
        </span>
      </header>

      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2">
        <TeamButton
          team={card.team_a}
          mine={card.my_prediction !== null && card.my_prediction === card.team_a?.id}
          winner={card.winner !== null && card.winner === card.team_a?.id}
          disabled={!canPredict}
          onPick={onPredict}
          label={t("pick")}
        />
        <span className="flex flex-col items-center text-center text-sm font-bold text-mute">
          {card.score ? `${card.score.team_a} : ${card.score.team_b}` : t("vs")}
          <ResultMark result={card.my_result} />
        </span>
        <TeamButton
          team={card.team_b}
          mine={card.my_prediction !== null && card.my_prediction === card.team_b?.id}
          winner={card.winner !== null && card.winner === card.team_b?.id}
          disabled={!canPredict}
          onPick={onPredict}
          label={t("pick")}
        />
      </div>

      <PredictionsLine card={card} />

      <footer className="flex flex-wrap items-center justify-between gap-2 text-xs">
        <LockLine card={card} locked={locked} locksAt={locksAt} now={now} />
        <span className="text-mute">
          {card.source === null
            ? t("addedByHand")
            : t("freshness", {
                ago: formatAgo(card.updated_at, now, language),
                source: t(`source.${card.source}`),
              })}
          {card.overridden.length > 0 && (
            <span className="ml-2 font-semibold text-gold-a">
              {t("overridden", {
                fields: card.overridden.map((f) => t(`overriddenField.${f}`)).join(", "),
              })}
            </span>
          )}
        </span>
      </footer>
    </article>
  );
}

/** #7: ✓/✗ for my Prediction on a finished Match. */
function ResultMark({ result }: { result: Card["my_result"] }) {
  const { t } = useTranslation("matches");
  if (result === "pending") return null;
  const look = {
    correct: { mark: "✓", color: "text-emerald-b" },
    wrong: { mark: "✗", color: "text-danger" },
    missing: { mark: "–", color: "text-mute" },
    void: { mark: "∅", color: "text-mute" },
  }[result];
  return (
    <span
      role="img"
      aria-label={t(`result.${result}`)}
      title={t(`result.${result}`)}
      className={`text-lg leading-none ${look.color}`}
    >
      {look.mark}
    </span>
  );
}

function TeamButton({
  team,
  mine,
  winner,
  disabled,
  onPick,
  label,
}: {
  team: TeamRef | null;
  mine: boolean;
  winner: boolean;
  disabled: boolean;
  onPick: (team: TeamRef) => void;
  label: string;
}) {
  const { t } = useTranslation("matches");
  if (team === null) {
    return (
      <div className="rounded-xl border border-dashed border-line px-3 py-3 text-center text-mute">
        {t("tbd")}
      </div>
    );
  }
  const look = mine
    ? "bg-emerald-gradient text-[#04130d] glow-emerald border-transparent"
    : "border-line bg-raised hover:border-emerald-a";
  return (
    <button
      type="button"
      aria-pressed={mine}
      aria-label={`${label}: ${team.name}`}
      disabled={disabled}
      onClick={() => onPick(team)}
      className={`flex min-w-0 items-center justify-center gap-2 rounded-xl border px-3 py-3 font-bold transition disabled:cursor-default disabled:hover:border-line ${look}`}
    >
      <TeamLogo id={team.id} />
      <span className={`truncate ${winner ? "font-bold text-gold-a" : ""}`}>
        {team.name}
      </span>
    </button>
  );
}

function LockLine({
  card,
  locked,
  locksAt,
  now,
}: {
  card: Card;
  locked: boolean;
  locksAt: number | null;
  now: number;
}) {
  const { t } = useTranslation("matches");
  const state = cardState(card);
  if (state === "live") return <span className="font-semibold text-danger">{t("status.live")}</span>;
  if (state === "void") {
    return (
      <span className="font-semibold text-fg">
        {t(card.status === "cancelled" ? "void.cancelled" : "void.noWinner")}
      </span>
    );
  }
  if (state === "finished") return <span className="text-mute">{t("status.finished")}</span>;
  if (locked) return <span className="font-semibold text-gold-a">{t("locked")}</span>;
  if (locksAt === null) return <span className="text-mute">{t("lockUnknown")}</span>;
  return (
    <span className="font-semibold text-emerald-b">
      {t(card.lock.edit_window && card.lock.can_predict ? "editWindow" : "locksIn", { time: formatSpan(t, locksAt - now) })}
    </span>
  );
}
