import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { StripModel } from "./layout";
import { LockHelpSheet } from "./LockHelpSheet";
import { formatSpan } from "./time";

type Props = {
  model: StripModel;
  /** The page's clock tick; the countdown is plain text that follows it. */
  now: number;
  onPickNext: () => void;
};

/**
 * What is left to pick, when the next Lock is, and the way to the next Match.
 * Emerald while something is open to pick, calm and neutral otherwise.
 */
export function NextUpStrip({ model, now, onPickNext }: Props) {
  const { t } = useTranslation("picks");
  const { t: tMatches } = useTranslation("matches");
  const [helpOpen, setHelpOpen] = useState(false);
  if (model.kind === "hidden") return null;

  const urgent = model.kind === "unpicked";
  const nextLock = model.kind === "nothing" ? null : model.nextLock;
  const title = model.kind === "unpicked"
    ? t("strip.unpicked", { count: model.count })
    : model.kind === "allPicked"
      ? t("strip.allPicked")
      : t("strip.nothing");

  return (
    <section
      aria-label={title}
      className={`flex flex-col gap-3 rounded-2xl border p-4 md:flex-row md:items-center md:justify-between md:gap-4 md:px-5 ${
        urgent ? "border-emerald-a bg-emerald-tint" : "border-line bg-panel"
      }`}
    >
      <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 md:flex-col md:items-start md:justify-start">
        <p className="text-lg font-bold leading-snug text-fg">{title}</p>
        {nextLock !== null && (
          <p className={`inline-flex items-center gap-1.5 text-sm font-semibold ${urgent ? "text-emerald-b" : "text-mute"}`}>
            <ClockIcon />
            {t("strip.nextLock", { time: formatSpan(tMatches, nextLock - now) })}
          </p>
        )}
      </div>

      <div className="flex flex-col gap-1 md:flex-row-reverse md:items-center md:gap-3">
        {urgent && (
          <button type="button" className="btn-primary w-full md:w-auto" onClick={onPickNext}>
            {t("strip.pickNext")}
          </button>
        )}
        <button
          type="button"
          className="tap rounded-lg px-3 text-sm text-mute focus-visible:text-fg [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
          onClick={() => setHelpOpen(true)}
        >
          {t("strip.howLockWorks")}
        </button>
      </div>

      <LockHelpSheet open={helpOpen} onClose={() => setHelpOpen(false)} />
    </section>
  );
}

function ClockIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="size-4 shrink-0"
    >
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}
