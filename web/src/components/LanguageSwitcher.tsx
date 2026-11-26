import type { KeyboardEvent as ReactKeyboardEvent } from "react";
import { useTranslation } from "react-i18next";

import { LANGUAGES } from "../i18n";
import { Popover } from "./Popover";

/** Small downward chevron that turns up while its menu is open. */
export function MenuChevron({ open }: { open: boolean }) {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 16 16"
      width={14}
      height={14}
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={`shrink-0 text-mute transition-transform duration-150 ease-out motion-reduce:transition-none ${
        open ? "rotate-180" : ""
      }`}
    >
      <path d="M4 6l4 4 4-4" />
    </svg>
  );
}

/** 3-option EN / RU / UK segmented control used inside header popovers. */
export function LanguageSegmentedControl() {
  const { t, i18n } = useTranslation(["ui", "common"]);
  const current = i18n.resolvedLanguage;

  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    const buttons = Array.from(e.currentTarget.querySelectorAll<HTMLButtonElement>("button"));
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (at < 0) return;
    e.preventDefault();
    e.stopPropagation();
    const delta = e.key === "ArrowRight" ? 1 : -1;
    buttons[(at + delta + buttons.length) % buttons.length]?.focus();
  };

  return (
    <div
      role="group"
      aria-label={t("language")}
      onKeyDown={onKeyDown}
      className="grid grid-cols-3 gap-1 rounded-xl border border-line bg-panel p-1"
    >
      {LANGUAGES.map((language) => {
        const active = language === current;
        return (
          <button
            key={language}
            type="button"
            role="menuitem"
            lang={language}
            title={t(`language.${language}`, { ns: "common" })}
            aria-pressed={active}
            onClick={() => void i18n.changeLanguage(language)}
            className={`inline-flex min-h-10 min-w-11 items-center justify-center rounded-lg px-2 text-xs font-semibold uppercase outline-none transition-[background-color,color,transform] duration-[140ms] ease-out focus-visible:ring-2 focus-visible:ring-emerald-b active:scale-[.97] motion-reduce:transition-[background-color,color] motion-reduce:active:scale-100 ${
              active
                ? "bg-emerald-tint text-emerald-b"
                : "text-mute [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
            }`}
          >
            {language}
          </button>
        );
      })}
    </div>
  );
}

/** Signed-out language button opening a popover with the 3-option segmented control. */
export function LanguageSwitcher() {
  const { t, i18n } = useTranslation("ui");
  const current = (i18n.resolvedLanguage ?? "en").toUpperCase();

  return (
    <Popover
      align="end"
      label={t("language")}
      triggerClassName="inline-flex min-h-11 min-w-11 items-center justify-center gap-1 rounded-xl border border-transparent px-2.5 text-xs font-semibold uppercase text-fg transition-[background-color,border-color,transform] duration-[140ms] ease-out active:scale-[.97] motion-reduce:transition-[background-color,border-color] motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:border-line [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised aria-expanded:border-line aria-expanded:bg-raised"
      trigger={({ open }) => (
        <>
          <span>{current}</span>
          <MenuChevron open={open} />
        </>
      )}
    >
      <div className="px-1 py-1">
        <p className="px-2 pb-1.5 text-xs font-medium text-mute">{t("language")}</p>
        <LanguageSegmentedControl />
      </div>
    </Popover>
  );
}
