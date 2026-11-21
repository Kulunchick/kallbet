import { useTranslation } from "react-i18next";

import { Logo } from "../../components/Logo";

const STEPS = ["league", "predict", "reveal", "leaderboard"] as const;

export function LandingPage() {
  const { t } = useTranslation("landing");
  return (
    <div className="flex flex-col gap-16 py-14 sm:py-20">
      <section className="flex flex-col items-center gap-7 text-center">
        <Logo className="h-16 w-auto sm:h-20" />
        <h1 className="max-w-3xl text-3xl font-extrabold leading-tight sm:text-5xl">
          {t("hero.titleLead")}{" "}
          <span className="block text-gold-gradient text-glow-gold">{t("hero.titleAccent")}</span>
        </h1>
        <p className="max-w-xl text-base text-mute sm:text-lg">{t("hero.subtitle")}</p>
        <span className="inline-flex items-center gap-2 rounded-full border border-line bg-panel px-4 py-2 text-sm text-mute">
          <span className="size-2 rounded-full bg-emerald-gradient glow-emerald" aria-hidden />
          {t("hero.badge")}
        </span>
      </section>

      <section aria-labelledby="how" className="flex flex-col gap-6">
        <h2 id="how" className="text-center text-xl font-bold text-gold-gradient sm:text-2xl">
          {t("how.title")}
        </h2>
        <ol className="grid gap-4 sm:grid-cols-2">
          {STEPS.map((step, index) => (
            <li key={step} className="card flex gap-4 p-5 transition-shadow hover:glow-emerald">
              <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-emerald-gradient font-display text-sm font-extrabold text-[#04130d]">
                {index + 1}
              </span>
              <div className="flex flex-col gap-1.5">
                <h3 className="text-sm font-bold">{t(`how.${step}.title`)}</h3>
                <p className="text-sm leading-relaxed text-mute">{t(`how.${step}.text`)}</p>
              </div>
            </li>
          ))}
        </ol>
      </section>
    </div>
  );
}
