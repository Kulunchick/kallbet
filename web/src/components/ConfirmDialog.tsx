import { useId, useState, type FormEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "../api/client";
import { Sheet } from "./Sheet";

type ConfirmDialogProps = {
  /** Whether the dialog is shown. The caller owns this state. */
  open: boolean;
  /** Cancel, Escape, scrim click or drag. Not called while `onConfirm` is pending. */
  onClose: () => void;
  /** The heading. */
  title: string;
  /** What is about to happen, in plain words. */
  body: ReactNode;
  /** The confirm button's text, a verb ("Delete League"). Default: the `ui:confirm` string. */
  confirmLabel?: string;
  /** `"danger"` for a destructive action, `"primary"` for the screen's one primary action. */
  tone: "danger" | "primary";
  /** Typed confirmation: the confirm button stays disabled until the trimmed input equals this (the League's name). */
  requireText?: string;
  /**
   * The action. The button shows a pending state while it runs; the dialog
   * closes (`onClose`) when it resolves and shows the error inline when it
   * rejects (an `ApiError` gets its `errors.<code>` text), staying open to retry.
   */
  onConfirm: () => Promise<void>;
  /** Default: the `ui:cancel` string. */
  cancelLabel?: string;
};

const button =
  "inline-flex min-h-12 w-full items-center justify-center gap-2 rounded-xl px-5 py-3 font-semibold transition-transform duration-[140ms] ease-out active:scale-[.97] md:flex-1";
// Local until the shared `btn-danger` lands: white on #b91c1c, never the primary emerald.
const dangerButton = `${button} bg-[#b91c1c] text-white [@media(hover:hover)_and_(pointer:fine)]:hover:bg-[#c42424]`;

/**
 * A confirmation on a `Sheet`: a bottom sheet on phones (confirm above Cancel),
 * a centered dialog on desktop (confirm first). Focus starts on Cancel, or on
 * the input when `requireText` is set, so a danger action is never one Enter
 * away. The typed text and the error reset each time it opens.
 */
export function ConfirmDialog({
  open,
  onClose,
  title,
  body,
  confirmLabel,
  tone,
  requireText,
  onConfirm,
  cancelLabel,
}: ConfirmDialogProps) {
  const { t } = useTranslation("ui");
  const formId = useId();
  const bodyId = useId();
  const [typed, setTyped] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    if (open) {
      setTyped("");
      setError(null);
      setPending(false);
    }
  }

  const matches = requireText === undefined || typed.trim() === requireText;

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (pending || !matches) return;
    setError(null);
    setPending(true);
    try {
      await onConfirm();
      onClose();
    } catch (cause) {
      setError(errorMessage(t, cause));
    } finally {
      setPending(false);
    }
  };

  const confirmClass = tone === "danger" ? dangerButton : `btn-primary ${button}`;
  const cancel = (
    <button
      type="button"
      className={`btn-ghost ${button}`}
      aria-disabled={pending || undefined}
      data-autofocus={requireText === undefined ? "" : undefined}
      onClick={() => {
        if (!pending) onClose();
      }}
    >
      {cancelLabel ?? t("cancel")}
    </button>
  );

  return (
    <Sheet
      open={open}
      onClose={onClose}
      title={title}
      dismissible={!pending}
      describedBy={bodyId}
      footer={
        // Confirm first in the DOM: on top in the bottom sheet, left on desktop.
        <div className="flex flex-col gap-3 md:flex-row">
          <button
            type="submit"
            form={formId}
            className={`${confirmClass} ${pending ? "cursor-progress opacity-70" : ""}`}
            disabled={!matches}
            aria-busy={pending}
            aria-disabled={pending || undefined}
          >
            {pending ? (
              <span
                aria-hidden
                className="size-4 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-pulse"
              />
            ) : null}
            {confirmLabel ?? t("confirm")}
          </button>
          {cancel}
        </div>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-4">
        <div id={bodyId} className="text-sm leading-6 text-mute">
          {body}
        </div>
        {requireText !== undefined ? (
          <label className="flex flex-col gap-2 text-sm text-fg">
            {t("typeToConfirm", { text: requireText })}
            <input
              type="text"
              value={typed}
              onChange={(e) => setTyped(e.target.value)}
              readOnly={pending}
              autoComplete="off"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              data-autofocus=""
              className="min-h-12 w-full rounded-xl border border-line bg-raised px-4 text-base text-fg"
            />
          </label>
        ) : null}
        {error ? (
          <p role="alert" className="text-sm font-medium text-danger">
            {error}
          </p>
        ) : null}
      </form>
    </Sheet>
  );
}
