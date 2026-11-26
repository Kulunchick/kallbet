import { useTranslation } from "react-i18next";

import { ApiError, errorMessage } from "../../api/client";
import { Cross } from "./icons";

/**
 * The save area of a board. Phones: sticky at the bottom above the safe area,
 * progress and hint on one line, the button full width. Desktop (`inline`): the
 * same content as a row at the bottom of the board card. The one primary
 * button is enabled only when the Pick'em is complete.
 */
export function SaveBar({
  placed,
  total,
  complete,
  saving,
  hasSaved,
  moreKey,
  error,
  inline,
  onSave,
  onShowEveryone,
}: {
  placed: number;
  total: number;
  complete: boolean;
  saving: boolean;
  /** A Pick'em is already saved: the button says "Save changes". */
  hasSaved: boolean;
  /** Which "N more to save" wording: placing teams (Swiss) or picking winners (Playoffs). */
  moreKey: "placeMore" | "pickMore";
  error: unknown;
  inline: boolean;
  onSave: () => void;
  onShowEveryone: () => void;
}) {
  const { t } = useTranslation("pickem");
  const hint = complete ? t("readyToSave") : t(moreKey, { n: total - placed });
  const button = complete
    ? "btn-primary"
    : "inline-flex min-h-12 items-center justify-center rounded-xl bg-raised px-5 py-3 font-semibold text-mute cursor-not-allowed";

  return (
    <div
      className={
        inline
          ? "flex flex-col gap-3 border-t border-line pt-4"
          : "sticky bottom-0 z-10 -mx-4 flex flex-col gap-3 border-t border-line bg-ink px-4 pb-[calc(0.75rem+env(safe-area-inset-bottom))] pt-3"
      }
    >
      <div className={inline ? "flex items-center justify-between gap-4" : "flex flex-col gap-3"}>
        <div className={inline ? "flex flex-col" : "flex items-baseline justify-between gap-3"}>
          <p aria-live="polite" className="text-sm font-semibold text-fg">
            {t("progress", { placed, total })}
          </p>
          <p className="text-[13px] text-mute">{hint}</p>
        </div>
        <button
          type="button"
          className={`${button} ${inline ? "" : "w-full"}`}
          disabled={!complete || saving}
          onClick={onSave}
        >
          {saving ? t("common:saving") : t(hasSaved ? "saveChanges" : "save")}
        </button>
      </div>
      <SaveError error={error} onShowEveryone={onShowEveryone} />
    </div>
  );
}

/** What a failed save says, under the bar. A refusal because the Stage Locked offers to show everyone's Pick'ems. */
export function SaveError({ error, onShowEveryone }: { error: unknown; onShowEveryone: () => void }) {
  const { t } = useTranslation("pickem");
  if (error == null) return null;
  const locked = error instanceof ApiError && error.code === "pickem_locked";
  return (
    <div
      role="alert"
      className={`flex flex-col gap-3 rounded-xl border p-3 text-sm ${locked ? "border-danger/60" : "border-line"}`}
    >
      <p className="flex items-start gap-2 text-danger">
        <Cross className="mt-0.5 size-4 shrink-0" />
        <span>{errorMessage(t, error)}</span>
      </p>
      {locked && (
        <button type="button" className="btn-ghost w-full" onClick={onShowEveryone}>
          {t("showEveryone")}
        </button>
      )}
    </div>
  );
}
