import { useCallback, useEffect, useState } from "react";

export type Loaded<T> =
  | { state: "loading" }
  | { state: "error"; error: unknown }
  | { state: "ready"; data: T };

/** Runs `load` on mount and whenever `deps` change; `reload` runs it again. */
export function useLoad<T>(load: () => Promise<T>, deps: unknown[]) {
  const [result, setResult] = useState<Loaded<T>>({ state: "loading" });
  const [tick, setTick] = useState(0);
  const reload = useCallback(() => setTick((n) => n + 1), []);

  useEffect(() => {
    let current = true;
    load().then(
      (data) => current && setResult({ state: "ready", data }),
      (error) => current && setResult({ state: "error", error }),
    );
    return () => {
      current = false;
    };
  }, [...deps, tick]); // eslint-disable-line react-hooks/exhaustive-deps

  return { result, reload };
}
