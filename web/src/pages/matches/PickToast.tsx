import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

export type PickToastData = {
  /** A new id replaces the toast on screen and restarts its timer. */
  id: number;
  text: string;
  /** Present only when there is a previous pick to put back. */
  onUndo: (() => void) | null;
};

const SHOWN_MS = 5_000;
const SHOWN_WITH_UNDO_MS = 8_000;

/**
 * The confirmation after a pick: "Pick saved · team", with a 44 px Undo when
 * the Member had picked the other team before. The live region is always in the
 * page, so a screen reader announces each new message. The timer stops while
 * the pointer or focus is on the toast.
 */
export function PickToast({ toast, onDismiss }: { toast: PickToastData | null; onDismiss: () => void }) {
  const { t } = useTranslation("picks");
  const [held, setHeld] = useState(false);
  const id = toast?.id;
  const undoable = toast?.onUndo != null;

  useEffect(() => {
    setHeld(false);
  }, [id]);

  useEffect(() => {
    if (id === undefined || held) return;
    const timer = window.setTimeout(onDismiss, undoable ? SHOWN_WITH_UNDO_MS : SHOWN_MS);
    return () => window.clearTimeout(timer);
  }, [id, held, undoable, onDismiss]);

  return (
    <div
      role="status"
      aria-live="polite"
      className="pointer-events-none fixed inset-x-4 bottom-[calc(1rem+env(safe-area-inset-bottom))] z-40 flex justify-center"
    >
      {toast && (
        <div
          key={toast.id}
          className="pointer-events-auto card flex min-h-12 w-full max-w-md items-center gap-3 bg-raised py-1 pl-4 pr-1.5 shadow-lg"
          onPointerEnter={() => setHeld(true)}
          onPointerLeave={() => setHeld(false)}
          onFocus={() => setHeld(true)}
          onBlur={() => setHeld(false)}
        >
          <svg
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={2.5}
            strokeLinecap="round"
            strokeLinejoin="round"
            className="size-4 shrink-0 text-emerald-b"
          >
            <path d="M5 12.5l4.5 4.5L19 7.5" />
          </svg>
          <span className="min-w-0 flex-1 break-words text-base text-fg">{toast.text}</span>
          {toast.onUndo && (
            <button
              type="button"
              className="tap rounded-lg px-3 font-semibold text-emerald-b"
              onClick={() => {
                const undo = toast.onUndo;
                onDismiss();
                undo?.();
              }}
            >
              {t("toast.undo")}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
