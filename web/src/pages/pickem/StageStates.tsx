import { useTranslation } from "react-i18next";

import { errorMessage } from "../../api/client";
import { Lock } from "./icons";

/** A static, card-shaped placeholder while a Stage loads. */
export function StageSkeleton() {
  const { t } = useTranslation("pickem");
  return (
    <div aria-busy="true" className="flex flex-col gap-4">
      <span className="sr-only">{t("loading")}</span>
      <div aria-hidden className="h-[72px] rounded-2xl border border-line bg-panel" />
      <div aria-hidden className="h-72 rounded-2xl border border-line bg-panel" />
      <div aria-hidden className="h-14 rounded-2xl border border-line bg-panel" />
    </div>
  );
}

/** A Stage that did not load: why, and the one primary action, Retry. */
export function LoadError({ error, onRetry }: { error: unknown; onRetry: () => void }) {
  const { t } = useTranslation("pickem");
  return (
    <div role="alert" className="flex flex-col gap-3 rounded-2xl border border-danger/60 p-4">
      <h3 className="font-display text-base text-fg">{t("loadErrorTitle")}</h3>
      <p className="text-sm text-mute">{errorMessage(t, error)}</p>
      <button type="button" className="btn-primary w-full" onClick={onRetry}>
        {t("retry")}
      </button>
    </div>
  );
}

/** The bracket cannot be filled in yet (first-round teams unknown). */
export function BracketNotReady() {
  const { t } = useTranslation("pickem");
  return (
    <p className="card flex items-start gap-3 p-4 text-sm text-mute">
      <Lock className="mt-0.5 size-5 shrink-0" />
      <span>{t("bracketNotReady")}</span>
    </p>
  );
}

/** A Stage that Locked with no Pick'em from anybody. */
export function Nobody() {
  const { t } = useTranslation("pickem");
  return <p className="card p-4 text-fg">{t("nobody")}</p>;
}
