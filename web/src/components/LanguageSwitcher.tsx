import { useTranslation } from "react-i18next";

import { LANGUAGES } from "../i18n";

/** Switches the interface language; the choice is remembered (localStorage). */
export function LanguageSwitcher() {
  const { t, i18n } = useTranslation();
  const current = i18n.resolvedLanguage;
  return (
    <div
      role="group"
      aria-label={t("language.label")}
      className="flex items-center rounded-xl border border-line bg-panel p-0.5 text-xs font-semibold"
    >
      {LANGUAGES.map((language) => {
        const active = language === current;
        return (
          <button
            key={language}
            type="button"
            lang={language}
            title={t(`language.${language}`)}
            aria-pressed={active}
            onClick={() => void i18n.changeLanguage(language)}
            className={
              active
                ? "rounded-[0.6rem] bg-emerald-gradient px-2.5 py-1.5 uppercase text-[#04130d]"
                : "rounded-[0.6rem] px-2.5 py-1.5 uppercase text-mute transition-colors hover:text-fg"
            }
          >
            {language}
          </button>
        );
      })}
    </div>
  );
}
