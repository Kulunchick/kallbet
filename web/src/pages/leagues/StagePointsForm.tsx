import { useState } from "react";
import { useTranslation } from "react-i18next";

import { ConfirmDialog } from "../../components/ConfirmDialog";
import { saveStagePoints, type LeagueDetail } from "./api";

/**
 * Points per correct Match Prediction, per Stage (creator only). Saving
 * recalculates every completed Match, so it asks first. The dialog sits beside the
 * form, not inside it: it has a form of its own.
 */
export function StagePointsForm({ league, reload }: { league: LeagueDetail; reload: () => void }) {
  const { t } = useTranslation("leagues");
  const [changes, setChanges] = useState<Record<number, string>>({});
  const [confirming, setConfirming] = useState(false);
  const dirty = Object.keys(changes).length > 0;

  return (
    <>
      <form
        className="flex flex-col gap-3 px-4 pb-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (dirty) setConfirming(true);
        }}
      >
        <p className="text-sm leading-6 text-mute">{t("stagePoints.title")}</p>
        {league.stages.map((stage) => (
          <label key={stage.ordinal} className="flex items-center justify-between gap-3 text-sm">
            <span className="min-w-0">{stage.name}</span>
            <input
              type="number"
              inputMode="numeric"
              required
              min={0}
              max={1000}
              value={
                changes[stage.ordinal] ??
                league.stage_points.find((value) => value.stage === stage.ordinal)?.points ??
                1
              }
              onChange={(event) => setChanges({ ...changes, [stage.ordinal]: event.target.value })}
              className="min-h-12 w-24 shrink-0 rounded-xl border border-line bg-raised px-3 text-base text-fg"
            />
          </label>
        ))}
        <button className="btn-ghost self-start" disabled={!dirty}>
          {t("stagePoints.save")}
        </button>
      </form>
      <ConfirmDialog
        open={confirming}
        onClose={() => setConfirming(false)}
        title={t("stagePoints.confirmTitle")}
        body={t("stagePoints.recalculate")}
        confirmLabel={t("stagePoints.confirmAction")}
        tone="primary"
        onConfirm={async () => {
          await saveStagePoints(
            league.id,
            Object.entries(changes).map(([stage, points]) => ({ stage: Number(stage), points: Number(points) })),
          );
          setChanges({});
          reload();
        }}
      />
    </>
  );
}
