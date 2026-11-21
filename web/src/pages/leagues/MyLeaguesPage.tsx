import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { errorMessage } from "../../api/client";
import { myLeagues } from "./api";
import { useLoad } from "./useLoad";

export function MyLeaguesPage() {
  const { t } = useTranslation("leagues");
  const { result } = useLoad(myLeagues, []);

  return (
    <div className="flex flex-col gap-6 py-10">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-extrabold text-gold-gradient">{t("mine.title")}</h1>
        <Link to="/leagues/new" className="btn-primary">
          {t("mine.create")}
        </Link>
      </div>

      {result.state === "loading" && <p className="text-mute">{t("loading")}</p>}
      {result.state === "error" && <p className="text-danger">{errorMessage(t, result.error)}</p>}
      {result.state === "ready" && result.data.length === 0 && (
        <p className="card p-6 text-mute">{t("mine.empty")}</p>
      )}
      {result.state === "ready" && result.data.length > 0 && (
        <ul className="grid gap-4 sm:grid-cols-2">
          {result.data.map((league) => (
            <li key={league.id}>
              <Link
                to={`/leagues/${league.id}`}
                className="card flex h-full flex-col gap-2 p-5 transition-shadow hover:glow-emerald"
              >
                <span className="text-lg font-bold">{league.name}</span>
                <span className="text-sm text-mute">{league.event.name}</span>
                {league.my_place !== null && (
                  <span className="text-sm font-semibold text-gold-a">
                    {t("mine.place", { place: league.my_place })} ·{" "}
                    {t("mine.points", { count: league.my_points })}
                  </span>
                )}
                {league.my_role === "admin" && (
                  <span className="text-xs font-semibold uppercase tracking-wide text-gold-a">
                    {t("role.admin")}
                  </span>
                )}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
