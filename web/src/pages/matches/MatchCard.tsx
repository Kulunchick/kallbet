import { useTranslation } from "react-i18next";

import { Avatar } from "../../components/Avatar";
import { TeamLogo } from "../../components/TeamLogo";
import type { MatchCard as Card, RevealedPrediction, TeamRef } from "./api";
import { cardState, hasPreliminaryPick, pickMark } from "./layout";
import { RationaleList } from "./RationalePanel";
import { formatAgo, formatSpan, formatStart } from "./time";

type Props = {
  card: Card;
  now: number;
  saving: boolean;
  onPredict: (team: TeamRef) => void;
};

export function MatchCard({ card, now, saving, onPredict }: Props) {
  const { t, i18n } = useTranslation("matches");
  const language = i18n.resolvedLanguage ?? "en";
  const locksAt = card.lock.locks_at ? Date.parse(card.lock.locks_at) : null;
  const locked = card.lock.locked || (locksAt !== null && locksAt <= now);
  const state = cardState(card);
  const settled = state === "finished" || state === "void";
  const showChoices = state === "upcoming" && card.lock.can_predict && !locked
    && card.team_a !== null && card.team_b !== null;
  const { count, eligible, revealed } = card.predictions;
  const source = card.source === null ? t("addedByHand") : t(`source.${card.source}`);
  const myTeam = [card.team_a, card.team_b].find((team) => team?.id === card.my_prediction);

  function side(team: TeamRef | null, align: "start" | "end") {
    return (
      <TeamSide
        team={team}
        align={align}
        winner={team !== null && card.winner === team.id}
        mine={team !== null && card.my_prediction === team.id}
        choice={showChoices}
        saving={saving}
        picks={revealed}
        onPick={onPredict}
      />
    );
  }

  return (
    <article className="card flex min-w-0 flex-col gap-3 px-3 py-3.5 sm:px-4" aria-busy={saving}>
      <header className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 text-xs text-mute">
        <span>
          <span className="font-semibold uppercase">{t(`format.${card.format}`)}</span>
          {" · "}
          {card.scheduled_start ? formatStart(card.scheduled_start, language) : t("startUnknown")}
        </span>
        <LockLine card={card} locked={locked} locksAt={locksAt} now={now} />
      </header>

      <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2 sm:gap-3">
        {side(card.team_a, "start")}
        {card.score ? (
          <span className="whitespace-nowrap text-2xl font-extrabold leading-none tabular-nums sm:text-3xl">
            {card.score.team_a}<span className="mx-0.5 font-medium text-mute">:</span>{card.score.team_b}
          </span>
        ) : (
          <span className="text-xs font-bold uppercase text-mute">{t("vs")}</span>
        )}
        {side(card.team_b, "end")}
      </div>

      <RationaleList card={card} />

      <footer className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 border-t border-line pt-2.5 text-xs">
        <div className="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
          {revealed === null && (
            <span className="text-mute">{t("predictedCount", { n: count, m: eligible })}</span>
          )}
          {!showChoices && (myTeam || card.my_result !== "pending") && (
            <PredictionResult result={card.my_result} team={myTeam ?? undefined} />
          )}
          {hasPreliminaryPick(revealed) && (
            <span className="text-mute">{t("bot.preliminaryHint")}</span>
          )}
        </div>
        <span className="ml-auto text-right text-mute">
          {settled || card.source === null
            ? source
            : t("freshness", { ago: formatAgo(card.updated_at, now, language), source })}
          {card.overridden.length > 0 && (
            <span className="ml-2 font-semibold text-gold-a">
              {t("overridden", {
                fields: card.overridden.map((field) => t(`overriddenField.${field}`)).join(", "),
              })}
            </span>
          )}
        </span>
      </footer>
    </article>
  );
}

function PredictionResult({ result, team }: { result: Card["my_result"]; team?: TeamRef }) {
  const { t } = useTranslation("matches");
  const correct = result === "correct";
  const wrong = result === "wrong";
  return (
    <span className={correct ? "font-semibold text-emerald-b" : wrong ? "font-semibold text-danger" : "text-mute"}>
      {(correct || wrong) && (
        <>
          <span aria-hidden="true">{correct ? "✓" : "✕"}{" "}</span>
          <span className="sr-only">{t(`result.${result}`)}.{" "}</span>
        </>
      )}
      {team ? t("myPrediction", { team: team.name }) : result !== "pending" && t(`result.${result}`)}
    </span>
  );
}

function TeamSide({
  team, align, winner, mine, choice, saving, picks, onPick,
}: {
  team: TeamRef | null;
  align: "start" | "end";
  winner: boolean;
  mine: boolean;
  choice: boolean;
  saving: boolean;
  picks: RevealedPrediction[] | null;
  onPick: (team: TeamRef) => void;
}) {
  const { t } = useTranslation("matches");
  const end = align === "end";
  const content = (
    <>
      {team && <TeamLogo id={team.id} className="size-5 shrink-0" />}
      <span className="min-w-0 break-words" title={team?.name}>{team?.name ?? t("tbd")}</span>
    </>
  );
  if (choice && team) {
    return (
      <button
        type="button"
        aria-pressed={mine}
        aria-label={`${t("pick")}: ${team.name}`}
        disabled={saving}
        onClick={() => onPick(team)}
        className={`flex min-w-0 items-center justify-center gap-1.5 rounded-lg border px-2 py-2 text-sm font-bold transition disabled:cursor-wait ${mine
          ? "border-emerald-a bg-emerald-a/10 text-emerald-b"
          : "border-line bg-raised hover:border-emerald-a"}`}
      >
        {content}
      </button>
    );
  }

  const teamPicks = picks?.filter((pick) => team && pick.team_id === team.id) ?? [];
  return (
    <div className={`flex min-w-0 flex-col gap-1.5 ${end ? "items-end text-right" : "items-start"}`}>
      <div className={`flex min-w-0 max-w-full items-center gap-1.5 text-sm font-bold sm:text-base ${end ? "flex-row-reverse" : ""} ${winner ? "text-gold-a" : team ? "text-fg" : "text-mute"}`}>
        {content}
      </div>
      {picks !== null && (
        <ul className={`flex min-h-6 flex-wrap gap-1 ${end ? "justify-end" : ""}`}>
          {teamPicks.map((pick) => <RevealedPick key={pick.user_id} pick={pick} />)}
        </ul>
      )}
    </div>
  );
}

/** One revealed Prediction: the Member's avatar; a Bot User's also carries a bot badge. */
function RevealedPick({ pick }: { pick: RevealedPrediction }) {
  const { t } = useTranslation("matches");
  const mark = pickMark(pick);
  if (mark === "human") {
    return (
      <li title={pick.nickname}>
        <Avatar url={pick.avatar_url} nickname={pick.nickname} className="size-6" />
        <span className="sr-only">{pick.nickname}</span>
      </li>
    );
  }
  const preliminary = mark === "bot_preliminary";
  const label = t(preliminary ? "bot.preliminaryPrediction" : "bot.pick", { nickname: pick.nickname });
  return (
    <li
      title={label}
      className={`flex items-center gap-1 rounded-full border bg-raised pr-1.5 ${preliminary ? "border-dashed border-gold-a/60" : "border-line"}`}
    >
      <Avatar url={pick.avatar_url} nickname={pick.nickname} className="size-6" />
      <span aria-hidden="true" className="text-[0.625rem] font-bold uppercase leading-none text-gold-a">
        {t("bot.badge")}{preliminary && "*"}
      </span>
      <span className="sr-only">{label}</span>
    </li>
  );
}

function LockLine({
  card, locked, locksAt, now,
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
      <span className="font-semibold" title={t(card.status === "cancelled" ? "void.cancelled" : "void.noWinner")}>
        {t("void.badge")}
      </span>
    );
  }
  if (state === "finished") return <span>{t("status.finished")}</span>;
  if (locked) return <span className="font-semibold text-gold-a">{t("locked")}</span>;
  if (locksAt === null) return <span>{t("lockUnknown")}</span>;
  return (
    <span className="font-semibold text-emerald-b">
      {t(card.lock.edit_window && card.lock.can_predict ? "editWindow" : "locksIn", { time: formatSpan(t, locksAt - now) })}
    </span>
  );
}
