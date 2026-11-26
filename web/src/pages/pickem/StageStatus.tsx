import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useLoad } from "../../api/useLoad";
import { getPickem, type PickemView } from "./api";
import { Lock } from "./icons";

export function useStagePickem(league: number, stage: number, onReveal: () => void) {
  const load = useLoad(() => getPickem(league, stage), [league, stage]);
  const { result, reload } = load;
  const lock = result.state === "ready" ? result.data.lock : null;
  const locked = lock?.locked;
  const deadline = lock?.locks_at;

  useEffect(() => {
    if (locked) {
      onReveal();
      return;
    }
    const ms = deadline ? Date.parse(deadline) - Date.now() : Infinity;
    const delay = ms > 0 ? Math.min(ms + 250, 15_000) : 15_000;
    const timer = window.setTimeout(reload, delay);
    return () => window.clearTimeout(timer);
  }, [deadline, locked, onReveal, reload, result]);

  return load;
}

export function StageStatus({ view }: { view: PickemView }) {
  const { t, i18n } = useTranslation("pickem");
  const { lock } = view;
  const time = lock.locks_at
    ? new Date(lock.locks_at).toLocaleString(i18n.resolvedLanguage)
    : "";
  return (
    <div className="card flex items-start gap-3 p-4">
      <Lock className="mt-0.5 size-5 shrink-0 text-mute" />
      <div className="flex min-w-0 flex-col gap-0.5">
        <p className="text-sm text-fg">
          {lock.locked ? t("revealed")
            : lock.edit_window
              ? t(lock.can_edit ? "finalEditUntil" : "revealAt", { time })
              : time ? t("locksAt", { time }) : t("lockUnknown")}
        </p>
        <p className="text-[13px] text-mute">{t("submittedCount", { count: view.submitted, eligible: view.eligible })}</p>
      </div>
    </div>
  );
}
