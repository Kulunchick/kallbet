import { useSyncExternalStore } from "react";

/** From `md` up: groups side by side, bracket columns, centered dialogs. */
export const MD = "(min-width: 48rem)";
/** A mouse or trackpad: where drag and drop is offered. */
export const FINE_POINTER = "(hover: hover) and (pointer: fine)";

/** Whether a media query matches, kept up to date. */
export function useMedia(query: string): boolean {
  return useSyncExternalStore(
    (listener) => {
      const list = window.matchMedia(query);
      list.addEventListener("change", listener);
      return () => list.removeEventListener("change", listener);
    },
    () => window.matchMedia(query).matches,
    () => false,
  );
}
