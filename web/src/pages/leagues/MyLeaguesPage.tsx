import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { errorMessage } from "../../api/client";
import { myLeagues } from "./api";
import { useLoad } from "../../api/useLoad";

export function MyLeaguesPage() {
  const { t } = useTranslation("leagues");
  const { result } = useLoad(myLeagues, []);

  return (
    <div className="flex flex-col gap-6 py-10">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="font-display text-2xl font-extrabold text-fg">{t("mine.title")}</h1>
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
                className="card flex h-full flex-col gap-2 p-5 transition-colors duration-[140ms] ease-out [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised"
              >
                <span className="text-lg font-bold">{league.name}</span>
                <span className="text-sm text-mute">{league.event.name}</span>
                {league.my_place !== null && (
                  <span className="text-sm font-semibold text-mute">
                    <span className={league.my_place === 1 ? "text-gold-a" : "text-fg"}>
                      {t("mine.place", { place: league.my_place })}
                    </span>
                    {" · "}
                    <span className="text-gold-a">
                      {t("mine.points", { count: league.my_points })}
                    </span>
                  </span>
                )}
                {league.my_role === "admin" && (
                  <span className="text-xs font-semibold uppercase tracking-wide text-mute">
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
