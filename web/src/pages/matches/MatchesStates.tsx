import { useTranslation } from "react-i18next";
import { Link } from "react-router";

/** The Matches could not be loaded and nothing is shown yet. Picks already made are safe. */
export function LoadFailed({ league, onRetry }: { league: number; onRetry: () => void }) {
  const { t } = useTranslation("picks");
  return (
    <section className="card flex flex-col gap-4 border-danger/60 p-4 md:p-5" aria-labelledby="matches-load-failed">
      <div className="flex flex-col gap-1">
        <h2 id="matches-load-failed" className="text-lg font-bold text-fg">{t("states.loadFailed.title")}</h2>
        <p className="text-base text-mute">{t("states.loadFailed.body")}</p>
      </div>
      <div className="flex flex-col gap-2 md:flex-row">
        <button type="button" className="btn-primary md:min-w-40" onClick={onRetry}>
          {t("states.loadFailed.retry")}
        </button>
        <Link to={`/leagues/${league}`} className="btn-ghost md:min-w-40">
          {t("states.loadFailed.back")}
        </Link>
      </div>
    </section>
  );
}

/** A refresh failed while Matches are on screen: keep them, offer a retry, stay quiet. */
export function RefreshFailed({ onRetry }: { onRetry: () => void }) {
  const { t } = useTranslation("picks");
  return (
    <div
      role="status"
      className="flex min-h-11 items-center justify-between gap-3 rounded-xl border border-line bg-panel pl-4 pr-1 text-sm text-mute"
    >
      <span>{t("states.refreshFailed.text")}</span>
      <button type="button" className="tap rounded-lg px-3 font-semibold text-fg" onClick={onRetry}>
        {t("states.refreshFailed.retry")}
      </button>
    </div>
  );
}

/** The Event has no schedule yet. */
export function NoMatches({ onCheck }: { onCheck: () => void }) {
  const { t } = useTranslation("picks");
  return (
    <section className="card flex flex-col gap-4 p-4 md:p-5" aria-labelledby="matches-empty">
      <div className="flex flex-col gap-1">
        <h2 id="matches-empty" className="text-lg font-bold text-fg">{t("states.empty.title")}</h2>
        <p className="text-base text-mute">{t("states.empty.body")}</p>
      </div>
      <button type="button" className="btn-ghost md:self-start md:min-w-40" onClick={onCheck}>
        {t("states.empty.checkAgain")}
      </button>
    </section>
  );
}

/** Card-shaped placeholders while the first load runs. Static: no shimmer. */
export function MatchesSkeleton() {
  const { t } = useTranslation("matches");
  return (
    <div className="flex flex-col gap-6" aria-busy="true">
      <span role="status" className="sr-only">{t("loading")}</span>
      <div aria-hidden="true" className="h-6 w-40 rounded-md bg-raised" />
      <div aria-hidden="true" className="grid gap-3 md:grid-cols-2">
        {[0, 1, 2, 3].map((n) => (
          <div key={n} className="card flex h-[150px] flex-col justify-between p-4">
            <div className="flex justify-between">
              <div className="h-4 w-28 rounded bg-raised" />
              <div className="h-4 w-20 rounded bg-raised" />
            </div>
            <div className="flex gap-3">
              <div className="h-12 flex-1 rounded-lg bg-raised" />
              <div className="h-12 flex-1 rounded-lg bg-raised" />
            </div>
            <div className="h-3 w-32 rounded bg-raised" />
          </div>
        ))}
      </div>
    </div>
  );
}
