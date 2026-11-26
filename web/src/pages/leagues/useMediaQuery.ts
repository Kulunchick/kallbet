import { useSyncExternalStore } from "react";

/** Whether a CSS media query matches right now, and keeps matching as the window changes. */
export function useMediaQuery(query: string): boolean {
  return useSyncExternalStore(
    (notify) => {
      const list = window.matchMedia(query);
      list.addEventListener("change", notify);
      return () => list.removeEventListener("change", notify);
    },
    () => window.matchMedia(query).matches,
    () => false,
  );
}

/** Tailwind's `md` and `lg` breakpoints, to switch a Sheet for a Popover or one column for two. */
export const MD = "(min-width: 48rem)";
export const LG = "(min-width: 64rem)";
