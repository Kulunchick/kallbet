import {
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
  type ReactNode,
} from "react";

/** How long the panel stays mounted after closing, to play its 120 ms exit. */
const EXIT_MS = 140;
const ITEMS = '[role^="menuitem"]:not(:disabled):not([aria-disabled="true"])';

/**
 * Class for a menu item inside a `Popover`: at least 44 px tall, quiet hover
 * (only on a real mouse), press feedback. Use it on a `<button>` or a router
 * `<Link>` together with `role="menuitem"`:
 * `<button role="menuitem" className={popoverItemClass} onClick={...}>`.
 */
export const popoverItemClass =
  "flex min-h-11 w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm text-fg transition-[background-color,scale] duration-[140ms] ease-out focus-visible:bg-line focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-emerald-b active:scale-[.97] motion-reduce:transition-[background-color] motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:bg-line";

type PopoverProps = {
  /** The button's content: an element, or a function of the open state. */
  trigger: ReactNode | ((state: { open: boolean }) => ReactNode);
  /** The menu items: `role="menuitem"` buttons or links (see `popoverItemClass`). Activating one closes the menu. */
  children: ReactNode;
  /** Which edge of the trigger the panel lines up with. Default `"end"`. */
  align?: "end" | "start";
  /** Accessible name of the trigger button and of the menu. */
  label: string;
  /** Extra classes for the trigger button (it has none of its own). */
  triggerClassName?: string;
};

/**
 * A menu button with a dropdown panel (the account menu). Arrow Up/Down, Home
 * and End move between items, Escape closes and returns focus to the button,
 * an outside click or Tab closes. The panel opens under the trigger and flips
 * above when it would run off the viewport. Opening or closing from the
 * keyboard is instant; with a pointer the panel scales in from the trigger's
 * corner (opacity only under reduced motion).
 */
export function Popover({ trigger, children, align = "end", label, triggerClassName }: PopoverProps) {
  const id = useId();
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [above, setAbove] = useState(false);
  // Set by the last interaction: keyboard-driven changes skip the animation.
  const [instant, setInstant] = useState(false);
  const focusOnOpen = useRef<"first" | "last">("first");

  const show = (fromKeyboard: boolean, focus: "first" | "last" = "first") => {
    focusOnOpen.current = focus;
    setInstant(fromKeyboard);
    setAbove(false);
    setMounted(true);
    setOpen(true);
  };

  const hide = useCallback((fromKeyboard: boolean, restoreFocus: boolean) => {
    setInstant(fromKeyboard);
    setOpen(false);
    if (fromKeyboard) setMounted(false);
    if (restoreFocus) button.current?.focus();
  }, []);

  // Leave the DOM after the exit transition.
  useEffect(() => {
    if (open || !mounted) return;
    const timer = window.setTimeout(() => setMounted(false), EXIT_MS);
    return () => window.clearTimeout(timer);
  }, [open, mounted]);

  // Flip above the trigger when the panel would run past the bottom of the viewport.
  useLayoutEffect(() => {
    if (!open || !panel.current || !button.current) return;
    const box = panel.current.getBoundingClientRect();
    const trig = button.current.getBoundingClientRect();
    if (box.bottom > window.innerHeight - 8 && trig.top - box.height - 8 > 0) setAbove(true);
  }, [open]);

  // Move focus into the menu once it is there.
  useLayoutEffect(() => {
    if (!open) return;
    const items = panel.current?.querySelectorAll<HTMLElement>(ITEMS);
    if (!items?.length) return;
    items[focusOnOpen.current === "last" ? items.length - 1 : 0].focus({ preventScroll: true });
  }, [open]);

  // A press anywhere outside closes it.
  useEffect(() => {
    if (!open) return;
    const onPointerDown = (e: PointerEvent) => {
      if (!root.current?.contains(e.target as Node)) hide(false, false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [open, hide]);

  const onTriggerClick = (e: ReactMouseEvent) => {
    // detail is 0 for a click made with Enter or Space.
    if (open) hide(e.detail === 0, false);
    else show(e.detail === 0);
  };

  const onTriggerKeyDown = (e: ReactKeyboardEvent) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    if (!open) show(true, e.key === "ArrowUp" ? "last" : "first");
  };

  const onPanelKeyDown = (e: ReactKeyboardEvent) => {
    const items = Array.from(panel.current?.querySelectorAll<HTMLElement>(ITEMS) ?? []);
    const at = items.indexOf(document.activeElement as HTMLElement);
    const move = (to: number) => {
      e.preventDefault();
      items[(to + items.length) % items.length]?.focus();
    };
    switch (e.key) {
      case "ArrowDown":
        return move(at + 1);
      case "ArrowUp":
        return move(at < 0 ? -1 : at - 1);
      case "Home":
        return move(0);
      case "End":
        return move(-1);
      case " ":
        if (at >= 0 && items[at].tagName !== "BUTTON") {
          e.preventDefault();
          if (!e.repeat) items[at].click();
        }
        return;
      case "Tab":
        // Leave from the button, so Tab carries on to its neighbour.
        button.current?.focus();
        return hide(true, false);
    }
  };

  const onRootKeyDown = (e: ReactKeyboardEvent) => {
    if (e.key === "Escape" && open) {
      e.preventDefault();
      e.stopPropagation(); // do not also close a Sheet this menu sits in
      hide(true, true);
    }
  };

  const triggerContent = typeof trigger === "function" ? trigger({ open }) : trigger;

  return (
    <div ref={root} className="relative inline-block" onKeyDown={onRootKeyDown}>
      <button
        ref={button}
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={mounted ? id : undefined}
        aria-label={label}
        className={triggerClassName}
        onClick={onTriggerClick}
        onKeyDown={onTriggerKeyDown}
      >
        {triggerContent}
      </button>
      {mounted ? (
        <>
          <PopoverStyles />
          <div
            ref={panel}
            id={id}
            role="menu"
            aria-label={label}
            data-state={open ? "open" : "closed"}
            data-instant={instant || undefined}
            className={`kb-popover absolute z-50 min-w-48 max-w-[calc(100vw-1rem)] rounded-[14px] border border-line bg-raised p-1.5 shadow-[0_12px_32px_-8px_rgb(0_0_0/.55)] ${
              align === "end" ? "right-0" : "left-0"
            } ${above ? "bottom-full mb-2" : "top-full mt-2"} ${
              align === "end" ? (above ? "origin-bottom-right" : "origin-top-right") : above ? "origin-bottom-left" : "origin-top-left"
            }`}
            onKeyDown={onPanelKeyDown}
            onClick={(e) => {
              // Any activated item closes the menu and hands focus back.
              if ((e.target as HTMLElement).closest('[role^="menuitem"]')) hide(e.detail === 0, true);
            }}
          >
            {children}
          </div>
        </>
      ) : null}
    </div>
  );
}

// Entering uses @starting-style (the panel is new in the DOM); leaving flips
// data-state. 95% and fully transparent, never 0: nothing grows out of nothing.
const STYLES = `
.kb-popover {
  opacity: 1;
  transform: none;
  transition: transform 150ms cubic-bezier(.23,1,.32,1), opacity 150ms cubic-bezier(.23,1,.32,1);
}
.kb-popover[data-state="closed"] {
  opacity: 0;
  transform: scale(.95);
  transition-duration: 120ms;
  transition-timing-function: ease-out;
  pointer-events: none;
}
@starting-style {
  .kb-popover { opacity: 0; transform: scale(.95); }
}
.kb-popover[data-instant] { transition: none; }
@media (prefers-reduced-motion: reduce) {
  .kb-popover, .kb-popover[data-state="closed"] { transform: none; transition-property: opacity; }
  @starting-style {
    .kb-popover { transform: none; }
  }
}
`;

function PopoverStyles() {
  return (
    <style href="kb-popover" precedence="default">
      {STYLES}
    </style>
  );
}
