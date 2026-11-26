import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";

import { ConfirmDialog } from "../../components/ConfirmDialog";
import { deleteLeague, type LeagueDetail } from "./api";
import { ChevronDown, ChevronRight } from "./icons";
import { StagePointsForm } from "./StagePointsForm";

const summary =
  "flex w-full cursor-pointer list-none items-center justify-between gap-3 px-4 text-left [&::-webkit-details-marker]:hidden";
const rowText = "flex min-w-0 flex-col";
const hint = "text-sm leading-5 text-mute";
const press =
  "transition-colors duration-[140ms] ease-out [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised";

/**
 * League Admin tools in one collapsed disclosure, so the page keeps a single
 * primary action: Overrides, Points per Stage (creator only) and Delete League
 * (creator only). The delete dialog is outside the `<details>` so it opens even
 * if the disclosure is closed behind it.
 */
export function AdminTools({ league, reload }: { league: LeagueDetail; reload: () => void }) {
  const { t } = useTranslation("leagues");
  const navigate = useNavigate();
  const [deleting, setDeleting] = useState(false);
  const showStagePoints = league.is_creator && league.stages.length > 0;

  return (
    <>
      <details className="card group overflow-hidden">
        <summary className={`${summary} min-h-[52px] ${press}`}>
          <h2 className="text-base font-semibold text-fg">{t("adminTools.title")}</h2>
          <ChevronDown className="shrink-0 text-mute transition-transform duration-200 ease-out group-open:rotate-180 motion-reduce:transition-none" />
        </summary>

        <div className="flex flex-col border-t border-line">
          <Link to={`/leagues/${league.id}/overrides`} className={`${summary} min-h-14 py-2 ${press}`}>
            <span className={rowText}>
              <span className="font-medium text-fg">{t("overrides:open")}</span>
              <span className={hint}>{t("adminTools.overridesHint")}</span>
            </span>
            <ChevronRight className="shrink-0 text-mute" />
          </Link>

          {showStagePoints && (
            <details className="group/stages border-t border-line">
              <summary className={`${summary} min-h-14 py-2 ${press}`}>
                <span className={rowText}>
                  <span className="font-medium text-fg">{t("stagePoints.rowTitle")}</span>
                  <span className={hint}>{t("stagePoints.rowHint")}</span>
                </span>
                <ChevronDown className="shrink-0 text-mute transition-transform duration-200 ease-out group-open/stages:rotate-180 motion-reduce:transition-none" />
              </summary>
              <StagePointsForm key={league.id} league={league} reload={reload} />
            </details>
          )}

          {league.is_creator && (
            <div className="flex flex-col gap-3 border-t border-line p-4">
              <p className={hint}>{t("delete.zoneHint")}</p>
              <button
                type="button"
                className="inline-flex min-h-12 w-full items-center justify-center rounded-xl border border-danger/60 px-5 py-3 font-semibold text-danger transition-transform duration-[140ms] ease-out active:scale-[.97] motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:bg-danger/10"
                onClick={() => setDeleting(true)}
              >
                {t("delete.open")}
              </button>
            </div>
          )}
        </div>
      </details>

      {league.is_creator && (
        <ConfirmDialog
          open={deleting}
          onClose={() => setDeleting(false)}
          title={t("delete.dialogTitle", { name: league.name })}
          body={t("delete.dialogBody")}
          confirmLabel={t("delete.confirm")}
          tone="danger"
          requireText={league.name}
          onConfirm={async () => {
            await deleteLeague(league.id, league.name);
            navigate("/leagues");
          }}
        />
      )}
    </>
  );
}
