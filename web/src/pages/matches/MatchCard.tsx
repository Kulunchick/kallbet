import type { ReactNode } from "react";
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
  /** Also show where the data comes from, how fresh it is and League Overrides. */
  showMeta?: boolean;
};

export function MatchCard({ card, now, saving, onPredict, showMeta = false }: Props) {
  const { t, i18n } = useTranslation("matches");
  const language = i18n.resolvedLanguage ?? "en";
  const locksAt = card.lock.locks_at ? Date.parse(card.lock.locks_at) : null;
  const locked = card.lock.locked || (locksAt !== null && locksAt <= now);
  const state = cardState(card);
  const settled = state === "finished" || state === "void";
  const showChoices = state === "upcoming" && card.lock.can_predict && !locked
    && card.team_a !== null && card.team_b !== null;
  const { count, eligible, revealed } = card.predictions;
  const myTeam = [card.team_a, card.team_b].find((team) => team?.id === card.my_prediction);
  const unpicked = showChoices && card.my_prediction === null;
  const showResult = !showChoices && (myTeam !== undefined || card.my_result !== "pending");

  function side(team: TeamRef | null, align: "start" | "end") {
    return (
      <TeamSide
        team={team}
        align={align}
        winner={team !== null && card.winner === team.id}
        loser={team !== null && card.winner !== null && card.winner !== team.id}
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
      <header className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
        <LockLine card={card} locked={locked} locksAt={locksAt} now={now} />
        <span className="ml-auto text-[13px] text-mute">
          <span className="uppercase">{t(`format.${card.format}`)}</span>
          {" · "}
          {card.scheduled_start ? formatStart(card.scheduled_start, language) : t("startUnknown")}
        </span>
      </header>

      <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-start gap-2">
        {side(card.team_a, "start")}
        {card.score ? (
          <span className="self-start whitespace-nowrap pt-0.5 text-center text-2xl font-extrabold leading-none tabular-nums sm:text-3xl">
            {card.score.team_a}<span className="mx-0.5 font-medium text-mute">:</span>{card.score.team_b}
          </span>
        ) : (
          <span className="self-center text-xs font-bold uppercase text-mute">{t("vs")}</span>
        )}
        {side(card.team_b, "end")}
      </div>

      <RationaleList card={card} />

      {hasPreliminaryPick(revealed) && (
        <p className="text-xs text-mute">
          <span aria-hidden="true">*{" "}</span>
          {t("bot.preliminaryHint")}
        </p>
      )}

      {(revealed === null || showResult || showMeta) && (
        <footer className="flex flex-wrap items-baseline gap-x-3 gap-y-1 text-[13px]">
          {revealed === null && (
            <span className="text-mute">
              {t("predictedCount", { n: count, m: eligible })}
              {unpicked && <>{" · "}{t("unpickedHint")}</>}
            </span>
          )}
          {showResult && (
            <PredictionResult result={card.my_result} team={myTeam ?? undefined} points={card.my_points} />
          )}
          {showMeta && <MetaLine card={card} settled={settled} now={now} language={language} />}
        </footer>
      )}
    </article>
  );
}

/** Source, freshness and League Overrides: small print, only when asked for. */
function MetaLine({
  card, settled, now, language,
}: {
  card: Card;
  settled: boolean;
  now: number;
  language: string;
}) {
  const { t } = useTranslation("matches");
  const source = card.source === null ? t("addedByHand") : t(`source.${card.source}`);
  const overridden = card.overridden.length > 0
    ? t("overridden", {
      fields: card.overridden.map((field) => t(`overriddenField.${field}`)).join(", "),
    })
    : null;
  return (
    <span className="ml-auto flex flex-wrap items-baseline justify-end gap-x-2 text-right text-xs text-mute">
      <span>
        {settled || card.source === null
          ? source
          : t("freshness", { ago: formatAgo(card.updated_at, now, language), source })}
      </span>
      {overridden && (
        <span className="inline-flex items-center gap-1" title={overridden}>
          <PencilIcon />
          {overridden}
        </span>
      )}
    </span>
  );
}

function PredictionResult({ result, team, points }: { result: Card["my_result"]; team?: TeamRef; points: number }) {
  const { t } = useTranslation("matches");
  const correct = result === "correct";
  const wrong = result === "wrong";
  if (correct || wrong) {
    return (
      <span className={`font-semibold ${correct ? "text-emerald-b" : "text-danger"}`}>
        <span aria-hidden="true">{correct ? "✓" : "✕"}{" "}{t(`resultShort.${result}`)}</span>
        <span className="sr-only">{t(`result.${result}`)}.</span>
        {team && (
          <>
            <span aria-hidden="true">{" · "}</span>
            <span className="sr-only">{" "}</span>
            {t("youPicked", { team: team.name })}
          </>
        )}
        {correct && points > 0 && (
          <>
            <span aria-hidden="true">{" · "}</span>
            <span className="font-bold text-gold-a">{t("pointsEarned", { count: points })}</span>
          </>
        )}
      </span>
    );
  }
  return (
    <span className="text-mute">
      {team ? t("youPicked", { team: team.name }) : result !== "pending" && t(`result.${result}`)}
    </span>
  );
}

function TeamSide({
  team, align, winner, loser, mine, choice, saving, picks, onPick,
}: {
  team: TeamRef | null;
  align: "start" | "end";
  winner: boolean;
  loser: boolean;
  mine: boolean;
  choice: boolean;
  saving: boolean;
  picks: RevealedPrediction[] | null;
  onPick: (team: TeamRef) => void;
}) {
  const { t } = useTranslation("matches");
  const end = align === "end";
  const logo = team && <TeamLogo id={team.id} className="size-5 shrink-0" />;
  const name = <span className="min-w-0 break-words" title={team?.name}>{team?.name ?? t("tbd")}</span>;
  if (choice && team) {
    return (
      <button
        type="button"
        aria-pressed={mine}
        aria-label={`${t("pick")}: ${team.name}`}
        disabled={saving}
        onClick={() => onPick(team)}
        className={`flex min-h-12 min-w-0 items-center justify-center gap-2 rounded-lg px-2 py-2 text-base font-bold transition-transform duration-[140ms] ease-[cubic-bezier(.23,1,.32,1)] active:scale-[.97] motion-reduce:transition-none motion-reduce:active:scale-100 disabled:cursor-wait ${mine
          ? "border-2 border-emerald-a bg-emerald-a/10 text-emerald-b"
          : "border border-line bg-raised [@media(hover:hover)_and_(pointer:fine)]:hover:border-emerald-a"}`}
      >
        {logo}
        {name}
        {mine && <CheckIcon className="size-4 shrink-0" />}
      </button>
    );
  }

  const teamPicks = picks?.filter((pick) => team && pick.team_id === team.id) ?? [];
  const tone = winner ? "font-bold text-fg" : loser ? "font-semibold text-mute" : team ? "font-semibold text-fg" : "text-mute";
  return (
    <div className={`flex min-w-0 flex-col gap-1.5 ${end ? "items-end text-right" : "items-start"}`}>
      <div className={`flex min-w-0 max-w-full items-center gap-1.5 text-base ${end ? "flex-row-reverse" : ""} ${tone}`}>
        {logo}
        {name}
      </div>
      {winner && (
        <span className="inline-flex items-center gap-1 rounded-full bg-emerald-a/10 px-2 py-0.5 text-xs font-semibold text-emerald-b">
          <CheckIcon className="size-3 shrink-0" />
          {t("winner")}
        </span>
      )}
      {picks !== null && team && (
        <ul
          aria-label={t("pickedBy", { team: team.name })}
          className={`flex min-h-6 flex-wrap gap-1 ${end ? "justify-end" : ""}`}
        >
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
      className={`flex items-center gap-1 rounded-full border border-line bg-raised pr-1.5 ${preliminary ? "border-dashed" : ""}`}
    >
      <Avatar url={pick.avatar_url} nickname={pick.nickname} className="size-6" />
      <span aria-hidden="true" className="text-[0.625rem] font-bold uppercase leading-none text-mute">
        {t("bot.badge")}{preliminary && "*"}
      </span>
      <span className="sr-only">{label}</span>
    </li>
  );
}

/** The Lock status, the most important fact on the card. */
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
  let content: ReactNode;
  let tone = "font-semibold";
  if (state === "live") {
    tone = "font-bold uppercase text-danger";
    content = <><span aria-hidden="true" className="size-2 shrink-0 rounded-full bg-danger" />{t("status.live")}</>;
  } else if (state === "void") {
    tone = "font-semibold text-mute";
    content = (
      <span title={t(card.status === "cancelled" ? "void.cancelled" : "void.noWinner")}>{t("void.badge")}</span>
    );
  } else if (state === "finished") {
    tone = "font-semibold text-mute";
    content = t("status.finished");
  } else if (locked) {
    tone = "font-semibold text-fg";
    content = <><LockIcon />{t("locked")}</>;
  } else if (locksAt === null) {
    tone = "font-semibold text-mute";
    content = t("lockUnknown");
  } else {
    tone = "font-semibold text-emerald-b";
    const key = card.lock.edit_window && card.lock.can_predict ? "editWindow" : "locksIn";
    content = <><ClockIcon />{t(key, { time: formatSpan(t, locksAt - now) })}</>;
  }
  return <span className={`inline-flex min-w-0 items-center gap-1.5 text-base ${tone}`}>{content}</span>;
}

const iconProps = {
  "aria-hidden": true,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round",
  strokeLinejoin: "round",
} as const;

function ClockIcon() {
  return (
    <svg {...iconProps} className="size-4 shrink-0">
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}

function LockIcon() {
  return (
    <svg {...iconProps} className="size-4 shrink-0">
      <rect x="5" y="11" width="14" height="10" rx="2" />
      <path d="M8 11V7a4 4 0 0 1 8 0v4" />
    </svg>
  );
}

function CheckIcon({ className }: { className: string }) {
  return (
    <svg {...iconProps} strokeWidth={2.5} className={className}>
      <path d="M5 12.5l4.5 4.5L19 7.5" />
    </svg>
  );
}

function PencilIcon() {
  return (
    <svg {...iconProps} className="size-3 shrink-0">
      <path d="M4 20h4L19 9l-4-4L4 16v4z" />
    </svg>
  );
}
