import { useTranslation } from "react-i18next";
import { Link } from "react-router";

export type LeagueTab = "matches" | "pickem" | "leaderboard";

const TABS: { id: LeagueTab; path: string }[] = [
  { id: "matches", path: "matches" },
  { id: "pickem", path: "pickem" },
  { id: "leaderboard", path: "leaderboard" },
];

/**
 * The tab row of a League's sibling pages: Matches, Pick'em, Leaderboard. Plain
 * links (each tab is a route of its own), `aria-current="page"` on the open one.
 * On a phone the tabs share the width; from `md` they hug their label.
 */
export function LeagueTabs({ league, active }: { league: number; active: LeagueTab }) {
  const { t } = useTranslation("ui");
  return (
    <nav aria-label={t("leagueTabs.label")} className="flex border-b border-line">
      {TABS.map((tab) => {
        const selected = tab.id === active;
        return (
          <Link
            key={tab.id}
            to={`/leagues/${league}/${tab.path}`}
            aria-current={selected ? "page" : undefined}
            className={`-mb-px flex min-h-12 min-w-0 flex-1 items-center justify-center rounded-t-lg border-b-[3px] px-2 text-center text-sm transition-colors duration-[140ms] ease-out focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-emerald-b motion-reduce:transition-none md:flex-none md:px-6 md:text-base ${
              selected
                ? "border-emerald-a font-semibold text-fg"
                : "border-transparent font-medium text-mute [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
            }`}
          >
            <span className="min-w-0 truncate">{t(`leagueTabs.${tab.id}`)}</span>
          </Link>
        );
      })}
    </nav>
  );
}
