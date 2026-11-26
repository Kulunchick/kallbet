import { useTranslation } from "react-i18next";

import type { MatchCard, RevealedPrediction } from "./api";
import {
  externalLink, factorLabelKey, initialOf, rationalePicks, summaryFor, winSplit, type Rationale,
} from "./rationale";

const CHIP = "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-semibold text-mute";

/** The Rationales of a card's revealed picks, each a collapsed disclosure. Nothing before Reveal. */
export function RationaleList({ card }: { card: MatchCard }) {
  const picks = rationalePicks(card.predictions.revealed);
  if (picks.length === 0) return null;
  const teamName = (id: number) => [card.team_a, card.team_b].find((team) => team?.id === id)?.name ?? "";
  const opponentName = (id: number) =>
    [card.team_a, card.team_b].find((team) => team != null && team.id !== id)?.name ?? "";
  return (
    <div className="flex w-full flex-col gap-2">
      {picks.map((pick) => (
        <RationalePanel
          key={pick.user_id}
          pick={pick}
          rationale={pick.rationale}
          team={teamName(pick.team_id)}
          opponent={opponentName(pick.team_id)}
        />
      ))}
    </div>
  );
}

function RationalePanel({
  pick, rationale, team, opponent,
}: {
  pick: RevealedPrediction;
  rationale: Rationale;
  team: string;
  opponent: string;
}) {
  const { t, i18n } = useTranslation(["rationale", "matches"]);
  const split = winSplit(rationale.p_win);
  const summary = summaryFor(rationale.summary, i18n.resolvedLanguage ?? "en");
  const preliminaryNote = t("matches:rationale.preliminary");
  return (
    <details className="group w-full rounded-lg border border-line bg-panel">
      <summary className="flex min-h-[52px] cursor-pointer list-none items-center gap-3 rounded-lg p-3 [&::-webkit-details-marker]:hidden">
        <span
          aria-hidden="true"
          className={`flex size-7 shrink-0 items-center justify-center rounded-full border border-line bg-panel text-[0.6875rem] font-bold text-fg ${
            pick.preliminary ? "border-dashed" : ""
          }`}
        >
          {pick.is_bot ? t("rationale:aiMark") : initialOf(pick.nickname)}
        </span>
        <span className="flex min-w-0 flex-1 flex-col gap-0.5">
          <span className="break-words text-sm font-semibold text-fg">
            {t(pick.is_bot ? "matches:rationale.toggleBot" : "matches:rationale.toggle", { nickname: pick.nickname })}
          </span>
          {split && (
            <span className="text-[0.8125rem] text-mute">
              {t("rationale:toWin", { team })}{" "}
              <span className="font-bold tabular-nums text-fg">{split.pick}%</span>
            </span>
          )}
          {(rationale.override_market || pick.preliminary) && (
            <span className="mt-1 flex flex-wrap gap-1.5">
              {rationale.override_market && (
                <span title={t("matches:rationale.againstMarketHint")} className={`${CHIP} border-line`}>
                  <SwapIcon />
                  {t("matches:rationale.againstMarket")}
                </span>
              )}
              {pick.preliminary && (
                <span className={`${CHIP} border-dashed border-line`}>
                  {t("rationale:preliminaryChip", { note: preliminaryNote })}
                </span>
              )}
            </span>
          )}
        </span>
        <ChevronIcon />
      </summary>
      <div className="flex flex-col gap-3.5 border-t border-line p-3">
        {rationale.override_market && <p className="sr-only">{t("matches:rationale.againstMarketHint")}</p>}
        {split && <WinBar team={team} opponent={opponent} split={split} />}
        <div className="flex flex-col gap-2">
          {summary && (
            <p className="flex flex-wrap items-center gap-2 text-[0.8125rem] text-mute">
              <span className="rounded-full border border-line px-2 py-0.5 text-xs font-semibold">
                {summary.language.toUpperCase()}
              </span>
              {summary.fallback && (
                <span>
                  {t("matches:rationale.fallback", {
                    language: t(`matches:rationale.language.${summary.language}`),
                  })}
                </span>
              )}
            </p>
          )}
          {summary ? (
            <p lang={summary.language} className="text-[0.9375rem] leading-[22px] text-fg">{summary.text}</p>
          ) : (
            <p className="text-[0.9375rem] leading-[22px] text-mute">{t("matches:rationale.noSummary")}</p>
          )}
        </div>
        {rationale.preliminary && (
          <p className="flex items-start gap-2 rounded-lg border border-dashed border-line p-2.5 text-[0.8125rem] leading-snug text-mute">
            <ClockIcon />
            <span>{t("matches:rationale.draft")}</span>
          </p>
        )}
        {rationale.factors.length > 0 && (
          <div className="flex flex-col gap-2">
            <span className="text-[0.8125rem] text-mute">{t("matches:rationale.factors")}</span>
            <ul className="flex flex-wrap gap-2">
              {rationale.factors.map((factor, index) => (
                <FactorChip key={index} kind={factor.kind} url={factor.evidence_url} />
              ))}
            </ul>
          </div>
        )}
      </div>
    </details>
  );
}

/** Two segments of one track: the picked team's share in `fg`, the rest in `line`. Never colour alone: labels carry the numbers. */
function WinBar({
  team, opponent, split,
}: {
  team: string;
  opponent: string;
  split: { pick: number; opponent: number };
}) {
  const { t } = useTranslation(["rationale", "matches"]);
  const label = opponent
    ? t("rationale:barLabel", { team, percent: split.pick, opponent, opponentPercent: split.opponent })
    : t("matches:rationale.winChance", { team, percent: split.pick });
  return (
    <div className="flex flex-col gap-1.5">
      <div role="img" aria-label={label} className="flex h-2 gap-0.5">
        {split.pick > 0 && <span className="rounded-full bg-fg" style={{ width: `${split.pick}%` }} />}
        {split.opponent > 0 && <span className="rounded-full bg-line" style={{ width: `${split.opponent}%` }} />}
      </div>
      <div aria-hidden="true" className="flex justify-between gap-3 text-[0.8125rem]">
        <span className="min-w-0 break-words text-fg">
          {team} <span className="font-bold tabular-nums">{split.pick}%</span>
        </span>
        {opponent && (
          <span className="min-w-0 break-words text-right text-mute">
            {opponent} <span className="tabular-nums">{split.opponent}%</span>
          </span>
        )}
      </div>
    </div>
  );
}

function FactorChip({ kind, url }: { kind: string; url: string }) {
  const { t } = useTranslation("matches");
  const label = t(factorLabelKey(kind));
  const href = externalLink(url);
  const chip = "inline-flex min-h-10 items-center gap-1.5 rounded-full border border-line bg-panel px-3.5 text-[0.8125rem] font-medium";
  return (
    <li>
      {href ? (
        <a
          href={href}
          target="_blank"
          rel="noopener noreferrer"
          title={href}
          className={`${chip} text-fg active:scale-[.97] transition-transform duration-[140ms] ease-out motion-reduce:transition-none [@media(hover:hover)_and_(pointer:fine)]:hover:border-mute`}
        >
          {label}
          <ExternalIcon />
          <span className="sr-only">({t("rationale.newTab")})</span>
        </a>
      ) : (
        <span className={`${chip} text-mute`}>{label}</span>
      )}
    </li>
  );
}

const ICON = { width: 14, height: 14, viewBox: "0 0 16 16", fill: "none", stroke: "currentColor", strokeWidth: 1.5, strokeLinecap: "round", strokeLinejoin: "round", "aria-hidden": true, className: "shrink-0" } as const;

function ChevronIcon() {
  return (
    <svg {...ICON} width={16} height={16} className="shrink-0 text-mute transition-transform duration-200 ease-out group-open:rotate-90 motion-reduce:transition-none">
      <path d="M6 3.5 10.5 8 6 12.5" />
    </svg>
  );
}

function SwapIcon() {
  return (
    <svg {...ICON} width={12} height={12}>
      <path d="M2.5 5.5h10M10 3l2.5 2.5L10 8M13.5 10.5h-10M6 8l-2.5 2.5L6 13" />
    </svg>
  );
}

function ClockIcon() {
  return (
    <svg {...ICON} width={16} height={16} className="mt-px shrink-0">
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4.5V8l2.2 1.4" />
    </svg>
  );
}

function ExternalIcon() {
  return (
    <svg {...ICON} width={13} height={13}>
      <path d="M9 2.5h4.5V7M13.5 2.5 7.5 8.5M12 9.5v3a1 1 0 0 1-1 1H3.5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h3" />
    </svg>
  );
}
