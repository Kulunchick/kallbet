// Pure presentation logic of a Prediction's Rationale (no React, no imports,
// so it can be tested under plain node: `web/scripts/rationale.test.mjs`).

/** The interface languages a summary may be written in. */
export type SummaryLanguage = "en" | "ru" | "uk";

/** The closed vocabulary of adjustment factors (`FactorKind` in src/domain/rationale.rs). */
export const FACTOR_KINDS = [
  "stand_in",
  "roster_change_30d",
  "igl_or_coach_change",
  "visa_or_travel",
  "illness_or_emergency",
  "technical_or_dq",
] as const;

export type FactorKind = (typeof FACTOR_KINDS)[number];

/** A Prediction's Rationale as the API sends it (`Rationale` in src/domain/rationale.rs). */
export type Rationale = {
  /** Estimated win probability of the picked team, 0..1. */
  p_win: number;
  /** The pick goes against the market favourite. */
  override_market: boolean;
  /** The author's own draft flag: an early analysis. */
  preliminary: boolean;
  summary: Partial<Record<SummaryLanguage, string>>;
  factors: { kind: string; evidence_url: string }[];
};

/** Fallback order after the viewer's own language. */
const FALLBACK: SummaryLanguage[] = ["en", "ru", "uk"];

export type ShownSummary = { text: string; language: SummaryLanguage; fallback: boolean };

/**
 * The summary to show a viewer whose interface is in `language` (a code or
 * a locale tag): theirs if written, else the first of English, Russian,
 * Ukrainian. `null` when there is none.
 */
export function summaryFor(
  summary: Partial<Record<string, string>> | undefined,
  language: string,
): ShownSummary | null {
  if (!summary) return null;
  const wanted = language.split(/[-_]/)[0].toLowerCase();
  const order = [wanted, ...FALLBACK.filter((code) => code !== wanted)];
  for (const code of order) {
    const text = summary[code];
    if ((FALLBACK as string[]).includes(code) && typeof text === "string" && text.trim() !== "") {
      return { text, language: code as SummaryLanguage, fallback: code !== wanted };
    }
  }
  return null;
}

/** `p_win` as a whole percentage; `null` if it is not a probability. */
export function winPercent(p: number): number | null {
  if (!Number.isFinite(p) || p < 0 || p > 1) return null;
  return Math.round(p * 100);
}

/**
 * The win-chance bar's two shares as whole percentages that always add up to
 * 100: the picked team's (`winPercent`) and the opponent's remainder. `null`
 * if `p_win` is not a probability.
 */
export function winSplit(p: number): { pick: number; opponent: number } | null {
  const pick = winPercent(p);
  return pick === null ? null : { pick, opponent: 100 - pick };
}

/** First letter of a nickname, upper-cased (whole code point, so emoji and surrogate pairs stay intact). */
export function initialOf(nickname: string): string {
  return Array.from(nickname.trim())[0]?.toUpperCase() ?? "";
}

/** The URL if it is an absolute http(s) link, so it is safe as an `href`; else `null`. */
export function externalLink(url: string): string | null {
  if (!/^https?:\/\/[^/]/i.test(url)) return null;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:" ? url : null;
  } catch {
    return null;
  }
}

/** The `matches` namespace key of a factor's label. */
export function factorLabelKey(kind: string): string {
  return (FACTOR_KINDS as readonly string[]).includes(kind)
    ? `rationale.factor.${kind}`
    : "rationale.factor.other";
}

type PickWithRationale = { is_bot?: boolean; rationale?: Rationale | null };

/** The revealed picks that carry a Rationale, Bot Users' first, otherwise in order. */
export function rationalePicks<P extends PickWithRationale>(
  revealed: P[] | null,
): (P & { rationale: Rationale })[] {
  if (revealed === null) return [];
  const withRationale = revealed.filter(
    (pick): pick is P & { rationale: Rationale } => pick.rationale != null,
  );
  return [
    ...withRationale.filter((pick) => pick.is_bot),
    ...withRationale.filter((pick) => !pick.is_bot),
  ];
}
