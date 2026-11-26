import {
  useEffect,
  useId,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import { useTranslation } from "react-i18next";

import { dampUp, releaseVelocity, shouldDismiss, type DragSample } from "./sheetMath";

/** Slightly longer than the slowest exit transition, so the dialog leaves the DOM after it has finished. */
const UNMOUNT_AFTER_MS = 320;
const SPRING_BACK = "transform 350ms cubic-bezier(.32,.72,0,1)";
const MD_QUERY = "(min-width: 48rem)";

type SheetProps = {
  /** Whether the sheet is shown. The caller owns this state. */
  open: boolean;
  /** Asked for on Escape, a scrim click, a drag past the threshold or the close button. The caller sets `open` to false. */
  onClose: () => void;
  /** The heading. Also the dialog's accessible name. */
  title: string;
  /** The scrolling content. Put one `data-autofocus` attribute on an element to give it the initial focus. */
  children: ReactNode;
  /** Pinned below the content (buttons). Supply the visible Cancel/close path here. */
  footer?: ReactNode;
  /** `false` ignores every close gesture (use while a request is pending). Default `true`. */
  dismissible?: boolean;
  /** Id of an element inside the sheet that describes it (`aria-describedby`). */
  describedBy?: string;
};

/**
 * A modal sheet on the native `<dialog>`: focus trap, Escape, inert background
 * and focus return come from `showModal()`. Below `md` it is a bottom sheet
 * with a grabber that can be dragged down to dismiss; from `md` up a centered
 * dialog. Locks page scroll while open. Content is mounted only while the sheet
 * is open (and during its exit), so local state inside `children` resets on close.
 */
export function Sheet({ open, onClose, title, children, footer, dismissible = true, describedBy }: SheetProps) {
  const { t } = useTranslation("ui");
  const titleId = useId();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const openRef = useRef(open);
  const pressedOn = useRef<EventTarget | null>(null);
  const drag = useRef<{ id: number; startY: number; base: number; samples: DragSample[] } | null>(null);
  const springTimer = useRef<number | undefined>(undefined);
  // The dialog stays in the DOM through its exit transition.
  const [present, setPresent] = useState(open);
  if (open && !present) setPresent(true);

  useEffect(() => {
    openRef.current = open;
  }, [open]);

  useEffect(() => {
    if (!open) return;
    lockScroll();
    return unlockScroll;
  }, [open]);

  useEffect(() => {
    if (open) return;
    const timer = window.setTimeout(() => setPresent(false), UNMOUNT_AFTER_MS);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
      dialog.querySelector<HTMLElement>("[data-autofocus]")?.focus();
    } else if (!open && dialog.open) {
      // Drop the drag's inline styles in the same step as closing, so the exit
      // transition starts from where the finger left the sheet.
      window.clearTimeout(springTimer.current);
      dialog.style.transition = "";
      dialog.style.transform = "";
      dialog.close();
    }
  }, [open, present]);

  if (!present) return null;

  const requestClose = () => {
    if (dismissible) onClose();
  };

  const springBack = (dialog: HTMLDialogElement) => {
    window.clearTimeout(springTimer.current);
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    dialog.style.transition = reduced ? "" : SPRING_BACK;
    dialog.style.transform = "";
    springTimer.current = window.setTimeout(() => {
      dialog.style.transition = "";
    }, 380);
  };

  const dragStart = (e: ReactPointerEvent<HTMLDivElement>) => {
    const dialog = dialogRef.current;
    if (!dialog || drag.current || !dismissible) return; // a second pointer is ignored
    if (window.matchMedia(MD_QUERY).matches) return; // the centered dialog does not drag
    if (e.pointerType === "mouse" && e.button !== 0) return;
    window.clearTimeout(springTimer.current);
    // Catching the sheet mid spring-back: carry on from where it visually is.
    const base = new DOMMatrix(getComputedStyle(dialog).transform).m42;
    e.currentTarget.setPointerCapture(e.pointerId);
    dialog.style.transition = "none";
    drag.current = { id: e.pointerId, startY: e.clientY, base, samples: [{ y: e.clientY, t: e.timeStamp }] };
  };

  const dragMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    const dialog = dialogRef.current;
    const d = drag.current;
    if (!dialog || !d || e.pointerId !== d.id) return;
    const raw = d.base + (e.clientY - d.startY);
    // Down: 1:1 with the finger, written straight onto the element. Up: a rubber band.
    dialog.style.transform = `translateY(${raw >= 0 ? raw : -dampUp(-raw)}px)`;
    d.samples.push({ y: e.clientY, t: e.timeStamp });
    if (d.samples.length > 12) d.samples.shift();
  };

  const dragEnd = (e: ReactPointerEvent<HTMLDivElement>) => {
    const dialog = dialogRef.current;
    const d = drag.current;
    if (!dialog || !d || e.pointerId !== d.id) return;
    drag.current = null;
    const cancelled = e.type === "pointercancel";
    const distance = d.base + (e.clientY - d.startY);
    const velocity = cancelled ? 0 : releaseVelocity(d.samples, e.timeStamp);
    if (!cancelled && shouldDismiss({ distance, height: dialog.offsetHeight, velocity })) {
      onClose();
      // If the caller kept the sheet open, do not leave it stranded off screen.
      window.setTimeout(() => {
        if (openRef.current && dialogRef.current) springBack(dialogRef.current);
      }, 50);
    } else {
      springBack(dialog);
    }
  };

  return (
    <>
      <SheetStyles />
      <dialog
        ref={dialogRef}
        aria-labelledby={titleId}
        aria-describedby={describedBy}
        className="kb-sheet fixed inset-x-0 bottom-0 top-auto m-0 max-h-[85dvh] w-full max-w-none flex-col overflow-hidden rounded-t-3xl border border-b-0 border-line bg-panel p-0 pb-[calc(1rem+env(safe-area-inset-bottom))] text-fg open:flex md:inset-0 md:m-auto md:h-fit md:w-[calc(100%-2rem)] md:max-w-md md:rounded-[20px] md:border-b md:pb-5"
        onCancel={(e) => {
          e.preventDefault(); // the caller decides, through `open`
          requestClose();
        }}
        onClose={(e) => {
          // Closed behind our back (a second Escape the browser did not offer us).
          if (!openRef.current) return;
          if (dismissible) onClose();
          else e.currentTarget.showModal();
        }}
        onPointerDown={(e) => {
          pressedOn.current = e.target;
        }}
        onClick={(e) => {
          // A click on the scrim lands on the dialog itself. Ignore a press that
          // started inside (a text selection dragged out onto the scrim).
          if (e.target === e.currentTarget && pressedOn.current === e.currentTarget) requestClose();
        }}
      >
        <div
          className="shrink-0 touch-none select-none px-5 md:pt-5"
          onPointerDown={dragStart}
          onPointerMove={dragMove}
          onPointerUp={dragEnd}
          onPointerCancel={dragEnd}
        >
          <div className="flex h-6 items-center justify-center md:hidden">
            <span className="h-[5px] w-9 rounded-full bg-mute/40">
              <span className="sr-only">{t("dragHandle")}</span>
            </span>
          </div>
          <h2 id={titleId} className="pb-3 text-lg font-semibold leading-snug text-fg">
            {title}
          </h2>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-5 pb-1">{children}</div>
        {footer ? <div className="shrink-0 px-5 pt-3">{footer}</div> : null}
        {dismissible ? (
          <button type="button" className="sr-only" onClick={onClose}>
            {t("close")}
          </button>
        ) : null}
      </dialog>
    </>
  );
}

// -- Motion ------------------------------------------------------------------
// Transitions with @starting-style, not keyframes: opening and closing can be
// interrupted at any moment and then reverse from where they are. The rule for
// a state carries the transition used to *reach* it, so enter and exit differ.
// React 19 hoists this <style> into <head> once (href + precedence).

const STYLES = `
.kb-sheet {
  transform: translateY(100%);
  transition: transform 250ms ease-out, display 250ms allow-discrete, overlay 250ms allow-discrete;
}
.kb-sheet[open] {
  transform: translateY(0);
  transition: transform 350ms cubic-bezier(.32,.72,0,1), display 350ms allow-discrete, overlay 350ms allow-discrete;
}
@starting-style {
  .kb-sheet[open] { transform: translateY(100%); }
}
.kb-sheet::backdrop {
  background: rgb(0 0 0 / .6);
  opacity: 0;
  transition: opacity 200ms ease-out, display 200ms allow-discrete, overlay 200ms allow-discrete;
}
.kb-sheet[open]::backdrop { opacity: 1; }
@starting-style {
  .kb-sheet[open]::backdrop { opacity: 0; }
}
@media (min-width: 48rem) {
  .kb-sheet {
    transform: scale(.95);
    transform-origin: center;
    opacity: 0;
    transition: transform 150ms ease-out, opacity 150ms ease-out, display 150ms allow-discrete, overlay 150ms allow-discrete;
  }
  .kb-sheet[open] {
    transform: none;
    opacity: 1;
    transition: transform 200ms cubic-bezier(.23,1,.32,1), opacity 200ms cubic-bezier(.23,1,.32,1), display 200ms allow-discrete, overlay 200ms allow-discrete;
  }
  @starting-style {
    .kb-sheet[open] { transform: scale(.95); opacity: 0; }
  }
}
@media (prefers-reduced-motion: reduce) {
  .kb-sheet, .kb-sheet[open] {
    transform: none;
    transition: opacity 200ms linear, display 200ms allow-discrete, overlay 200ms allow-discrete;
  }
  .kb-sheet { opacity: 0; }
  .kb-sheet[open] { opacity: 1; }
  @starting-style {
    .kb-sheet[open] { transform: none; opacity: 0; }
  }
}
`;

function SheetStyles() {
  return (
    <style href="kb-sheet" precedence="default">
      {STYLES}
    </style>
  );
}

// -- Page scroll lock ----------------------------------------------------------
// Counted, so a Sheet opened over another one (or a ConfirmDialog opened from a
// Sheet) keeps the page locked until the last one closes.

let scrollLocks = 0;
let restoreScroll: (() => void) | null = null;

function lockScroll() {
  if (scrollLocks++ > 0) return;
  const { body, documentElement } = document;
  const before = { overflow: body.style.overflow, paddingRight: body.style.paddingRight };
  const scrollbar = window.innerWidth - documentElement.clientWidth;
  body.style.overflow = "hidden";
  // Hiding the scrollbar would widen the page and shift it sideways.
  if (scrollbar > 0) body.style.paddingRight = `calc(${getComputedStyle(body).paddingRight} + ${scrollbar}px)`;
  restoreScroll = () => {
    body.style.overflow = before.overflow;
    body.style.paddingRight = before.paddingRight;
  };
}

function unlockScroll() {
  if (--scrollLocks > 0) return;
  restoreScroll?.();
  restoreScroll = null;
}
