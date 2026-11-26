import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { useSession } from "../account/session";
import { RevealDemo } from "./RevealDemo";

const STEPS = ["league", "predict", "reveal", "leaderboard"] as const;

export function LandingPage() {
  const { t } = useTranslation("landing");
  const { user } = useSession();

  return (
    <div className="flex flex-col gap-16 py-10 sm:py-14 lg:gap-20 lg:py-16">
      <div className="grid grid-cols-1 items-center gap-10 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)] lg:gap-12">
        <section className="flex min-w-0 flex-col items-start gap-6 text-left">
          <h1 className="text-3xl font-extrabold leading-[1.15] text-fg sm:text-4xl lg:text-[2.625rem]">
            {t("hero.titleLead")}{" "}
            <span className="block">{t("hero.titleAccent")}</span>
          </h1>
          <p className="max-w-xl text-base leading-relaxed text-mute sm:text-lg">
            {t("hero.subtitle")}
          </p>
          <div className="flex w-full flex-col items-start gap-3 sm:w-auto sm:flex-row sm:flex-wrap sm:items-center sm:gap-4">
            <Link
              to={user ? "/leagues" : "/login"}
              className="btn-primary min-h-12 w-full px-6 text-base sm:w-auto"
            >
              {t(user ? "cta.leagues" : "cta.signIn")}
            </Link>
            <p className="text-sm text-mute">{t("cta.note")}</p>
          </div>
        </section>

        <RevealDemo />
      </div>

      <section aria-labelledby="how" className="flex flex-col gap-6">
        <h2 id="how" className="text-xl font-bold text-fg sm:text-2xl">
          {t("how.title")}
        </h2>
        <ol className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {STEPS.map((step, index) => (
            <li key={step} className="card flex min-w-0 flex-col gap-3 p-5">
              <span
                aria-hidden="true"
                className="flex size-8 shrink-0 items-center justify-center rounded-full border border-line bg-raised font-display text-xs font-bold text-fg tabular-nums"
              >
                {index + 1}
              </span>
              <div className="flex min-w-0 flex-col gap-1.5">
                <h3 className="text-sm font-bold leading-snug text-fg">
                  {t(`how.${step}.title`)}
                </h3>
                <p className="text-sm leading-relaxed text-mute">{t(`how.${step}.text`)}</p>
              </div>
            </li>
          ))}
        </ol>
      </section>
    </div>
  );
}
