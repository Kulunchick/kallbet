import { useTranslation } from "react-i18next";

const DEMO_TEAM_A = "Astralis";
const DEMO_TEAM_B = "Falcons";
const HIDDEN_PICKS = [1, 2, 3, 4, 5] as const;
const REVEALED_TEAM_A = ["A", "K"] as const;
const REVEALED_TEAM_B = ["M", "D", "S"] as const;

function ClockIcon() {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="size-3.5 shrink-0"
      aria-hidden="true"
    >
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4.75V8l2.25 1.25" />
    </svg>
  );
}

function LockIcon() {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="size-3.5 shrink-0"
      aria-hidden="true"
    >
      <rect x="3.25" y="7.25" width="9.5" height="6.5" rx="1.5" />
      <path d="M5.5 7.25V5.25a2.5 2.5 0 0 1 5 0v2" />
    </svg>
  );
}

function ChevronDownIcon() {
  return (
    <svg
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="size-5 text-mute"
      aria-hidden="true"
    >
      <path d="M5 7.5 10 12.5 15 7.5" />
    </svg>
  );
}

/** Static illustration of how Predictions stay hidden until Lock and then Reveal together. */
export function RevealDemo() {
  const { t } = useTranslation("landing");

  return (
    <section
      aria-labelledby="reveal-demo-title"
      aria-describedby="reveal-demo-desc"
      className="reveal-demo flex min-w-0 flex-col gap-3"
    >
      <h2 id="reveal-demo-title" className="text-base font-bold text-fg sm:text-lg">
        {t("demo.title")}
      </h2>
      <p id="reveal-demo-desc" className="sr-only">
        {t("demo.describe")}
      </p>

      <div className="flex flex-col gap-2">
        <article
          data-state="before"
          aria-labelledby="reveal-demo-before-label"
          className="reveal-demo-card card flex min-w-0 flex-col gap-3.5 p-4"
        >
          <header className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 text-xs">
            <span
              id="reveal-demo-before-label"
              className="font-semibold uppercase tracking-wide text-mute"
            >
              {t("demo.before.label")}
            </span>
            <span className="inline-flex items-center gap-1.5 font-medium text-mute">
              <ClockIcon />
              <span>{t("demo.before.status")}</span>
            </span>
          </header>

          <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
            <span className="truncate text-sm font-bold text-fg sm:text-base">{DEMO_TEAM_A}</span>
            <span className="text-xs font-semibold uppercase text-mute">{t("demo.vs")}</span>
            <span className="truncate text-right text-sm font-bold text-fg sm:text-base">
              {DEMO_TEAM_B}
            </span>
          </div>

          <div className="reveal-demo-picks flex items-center justify-center gap-1.5" aria-hidden="true">
            {HIDDEN_PICKS.map((slot) => (
              <span
                key={slot}
                className="reveal-demo-slot flex size-7 items-center justify-center rounded-full border border-dashed border-line bg-raised text-xs font-semibold text-mute"
              >
                ?
              </span>
            ))}
          </div>

          <footer className="border-t border-line pt-2.5 text-xs text-mute">
            {t("demo.before.footer")}
          </footer>
        </article>

        <div className="reveal-demo-chevron flex justify-center py-0.5" aria-hidden="true">
          <ChevronDownIcon />
        </div>

        <article
          data-state="locked"
          aria-labelledby="reveal-demo-locked-label"
          className="reveal-demo-card card flex min-w-0 flex-col gap-3.5 p-4"
        >
          <header className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 text-xs">
            <span
              id="reveal-demo-locked-label"
              className="font-semibold uppercase tracking-wide text-mute"
            >
              {t("demo.locked.label")}
            </span>
            <span className="inline-flex items-center gap-1.5 rounded-full border border-line bg-raised px-2.5 py-0.5 font-semibold text-fg">
              <LockIcon />
              <span>{t("demo.locked.status")}</span>
            </span>
          </header>

          <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-start gap-2">
            <div className="flex min-w-0 flex-col items-start gap-2">
              <span className="max-w-full truncate text-sm font-bold text-fg sm:text-base">
                {DEMO_TEAM_A}
              </span>
              <div className="reveal-demo-picks flex flex-wrap gap-1.5" aria-hidden="true">
                {REVEALED_TEAM_A.map((initial) => (
                  <span
                    key={initial}
                    className="reveal-demo-avatar flex size-7 items-center justify-center rounded-full border border-line bg-raised font-display text-xs font-bold text-fg"
                  >
                    {initial}
                  </span>
                ))}
              </div>
              <span className="sr-only">
                {t("demo.locked.teamPicks", {
                  team: DEMO_TEAM_A,
                  count: REVEALED_TEAM_A.length,
                })}
              </span>
            </div>

            <span className="pt-0.5 text-xs font-semibold uppercase text-mute">
              {t("demo.vs")}
            </span>

            <div className="flex min-w-0 flex-col items-end gap-2 text-right">
              <span className="max-w-full truncate text-sm font-bold text-fg sm:text-base">
                {DEMO_TEAM_B}
              </span>
              <div className="reveal-demo-picks flex flex-wrap justify-end gap-1.5" aria-hidden="true">
                {REVEALED_TEAM_B.map((initial) => (
                  <span
                    key={initial}
                    className="reveal-demo-avatar flex size-7 items-center justify-center rounded-full border border-line bg-raised font-display text-xs font-bold text-fg"
                  >
                    {initial}
                  </span>
                ))}
              </div>
              <span className="sr-only">
                {t("demo.locked.teamPicks", {
                  team: DEMO_TEAM_B,
                  count: REVEALED_TEAM_B.length,
                })}
              </span>
            </div>
          </div>

          <footer className="border-t border-line pt-2.5 text-xs text-fg">
            {t("demo.locked.footer")}
          </footer>
        </article>
      </div>
    </section>
  );
}
