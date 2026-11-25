import { useTranslation } from "react-i18next";

import type { MatchCard, RevealedPrediction } from "./api";
import {
  externalLink, factorLabelKey, rationalePicks, summaryFor, winPercent, type Rationale,
} from "./rationale";

/** The Rationales of a card's revealed picks, each a collapsed disclosure. Nothing before Reveal. */
export function RationaleList({ card }: { card: MatchCard }) {
  const picks = rationalePicks(card.predictions.revealed);
  if (picks.length === 0) return null;
  const teamName = (id: number) => [card.team_a, card.team_b].find((team) => team?.id === id)?.name ?? "";
  return (
    <div className="flex flex-col gap-1.5">
      {picks.map((pick) => (
        <RationalePanel key={pick.user_id} pick={pick} rationale={pick.rationale} team={teamName(pick.team_id)} />
      ))}
    </div>
  );
}

function RationalePanel({
  pick, rationale, team,
}: {
  pick: RevealedPrediction;
  rationale: Rationale;
  team: string;
}) {
  const { t, i18n } = useTranslation("matches");
  const percent = winPercent(rationale.p_win);
  const summary = summaryFor(rationale.summary, i18n.resolvedLanguage ?? "en");
  return (
    <details className="group rounded-lg border border-line bg-raised/40 text-xs">
      <summary className="flex cursor-pointer list-none flex-wrap items-center gap-x-2 gap-y-1 px-2.5 py-1.5 [&::-webkit-details-marker]:hidden">
        <span aria-hidden="true" className="text-mute transition-transform group-open:rotate-90">▸</span>
        <span className="font-semibold text-fg">
          {t(pick.is_bot ? "rationale.toggleBot" : "rationale.toggle", { nickname: pick.nickname })}
        </span>
        {percent !== null && (
          <span className="font-bold tabular-nums text-emerald-b">{t("rationale.winChance", { team, percent })}</span>
        )}
        {rationale.override_market && (
          <span
            title={t("rationale.againstMarketHint")}
            className="rounded-full border border-gold-a/60 px-1.5 py-px text-[0.625rem] font-bold uppercase leading-none text-gold-a"
          >
            {t("rationale.againstMarket")}
          </span>
        )}
        {pick.preliminary && <span className="text-mute">({t("rationale.preliminary")})</span>}
      </summary>
      <div className="flex flex-col gap-2 px-2.5 pb-2.5 pl-6">
        {rationale.override_market && <p className="sr-only">{t("rationale.againstMarketHint")}</p>}
        {summary ? (
          <p lang={summary.language} className="leading-relaxed text-fg">{summary.text}</p>
        ) : (
          <p className="text-mute">{t("rationale.noSummary")}</p>
        )}
        {summary?.fallback && (
          <p className="text-mute">
            {t("rationale.fallback", { language: t(`rationale.language.${summary.language}`) })}
          </p>
        )}
        {rationale.preliminary && <p className="text-mute">{t("rationale.draft")}</p>}
        {rationale.factors.length > 0 && (
          <div className="flex flex-wrap items-center gap-1">
            <span className="text-mute">{t("rationale.factors")}:</span>
            <ul className="flex flex-wrap gap-1">
              {rationale.factors.map((factor, index) => (
                <FactorPill key={index} kind={factor.kind} url={factor.evidence_url} />
              ))}
            </ul>
          </div>
        )}
      </div>
    </details>
  );
}

function FactorPill({ kind, url }: { kind: string; url: string }) {
  const { t } = useTranslation("matches");
  const label = t(factorLabelKey(kind));
  const href = externalLink(url);
  const pill = "inline-flex items-center gap-1 rounded-full border border-line bg-panel px-2 py-0.5";
  return (
    <li>
      {href ? (
        <a
          href={href}
          target="_blank"
          rel="noopener noreferrer"
          title={href}
          className={`${pill} text-fg transition hover:border-emerald-a hover:text-emerald-b`}
        >
          {label}
          <span aria-hidden="true">↗</span>
          <span className="sr-only">({t("rationale.newTab")})</span>
        </a>
      ) : (
        <span className={`${pill} text-mute`}>{label}</span>
      )}
    </li>
  );
}
