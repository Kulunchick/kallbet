import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { useLoad, type Loaded } from "../api/useLoad";
import { getLeague, type LeagueSummary } from "../pages/leagues/api";
import { ChevronLeft } from "../pages/leagues/icons";
import { LeagueTabs, type LeagueTab } from "./LeagueTabs";

/** The namespace that holds each page's own title (the sr-only heading under the League name). */
const PAGE_NAMESPACE: Record<LeagueTab, "matches" | "pickem" | "leaderboard"> = {
  matches: "matches",
  pickem: "pickem",
  leaderboard: "leaderboard",
};

/**
 * The header of a League's sibling pages (Matches, Pick'em, Leaderboard): back
 * link to My Leagues, the League name, its Event and the tab row.
 *
 * A page that already loads the League passes its `useLoad` result as `loaded`,
 * so the League is requested once per visit; without it the header loads the
 * League itself.
 */
export function LeagueHeader({
  league,
  active,
  loaded,
}: {
  league: number;
  active: LeagueTab;
  loaded?: Loaded<LeagueSummary>;
}) {
  return loaded ? (
    <HeaderView league={league} active={active} loaded={loaded} />
  ) : (
    <SelfLoadingHeader league={league} active={active} />
  );
}

function SelfLoadingHeader({ league, active }: { league: number; active: LeagueTab }) {
  const { result } = useLoad(() => getLeague(league), [league]);
  return <HeaderView league={league} active={active} loaded={result} />;
}

function HeaderView({ league, active, loaded }: { league: number; active: LeagueTab; loaded: Loaded<LeagueSummary> }) {
  const { t } = useTranslation("ui");
  const { t: tPage } = useTranslation(PAGE_NAMESPACE[active]);
  const pageTitle = tPage("title");
  const known = loaded.state === "ready" ? loaded.data : null;

  return (
    <header className="flex flex-col gap-4 md:gap-6">
      <div className="flex flex-col">
        <Link
          to="/leagues"
          className="-ml-2 inline-flex min-h-11 items-center gap-1 self-start rounded-lg pl-1 pr-3 text-sm text-mute focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-emerald-b [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
        >
          <ChevronLeft className="size-4" />
          {t("leagueTabs.myLeagues")}
        </Link>
        {known ? (
          <>
            <h1 className="mt-1 truncate font-display text-[22px] font-bold leading-8 text-fg md:overflow-visible md:whitespace-normal md:break-words md:text-[28px] md:leading-9">
              {known.name}
            </h1>
            <p className="truncate text-base leading-6 text-mute md:overflow-visible md:whitespace-normal md:break-words">
              {known.event.name}
            </p>
            <h2 className="sr-only">{pageTitle}</h2>
          </>
        ) : loaded.state === "loading" ? (
          <>
            {/* Same height as the name and the Event, so nothing moves when they arrive. */}
            <div aria-hidden="true" className="mt-1 flex h-8 items-center md:h-9">
              <div className="h-5 w-48 max-w-full animate-pulse rounded bg-raised motion-reduce:animate-none" />
            </div>
            <div aria-hidden="true" className="flex h-6 items-center">
              <div className="h-3.5 w-32 max-w-full animate-pulse rounded bg-raised motion-reduce:animate-none" />
            </div>
            <h2 className="sr-only">{pageTitle}</h2>
          </>
        ) : (
          // The League did not load: the name is not needed to move around, and the page keeps a heading.
          <h1 className="sr-only">{pageTitle}</h1>
        )}
      </div>
      <LeagueTabs league={league} active={active} />
    </header>
  );
}
