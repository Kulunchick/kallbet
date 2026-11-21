import { useEffect, useState } from "react";
import type { TFunction } from "i18next";

/** The current time, refreshed every `everyMs`. */
export function useNow(everyMs = 15_000): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), everyMs);
    return () => window.clearInterval(id);
  }, [everyMs]);
  return now;
}

/** A Scheduled Start in the viewer's timezone, e.g. "Sat, 10 Jan, 14:00". */
export function formatStart(iso: string, language: string): string {
  return new Intl.DateTimeFormat(language, {
    weekday: "short",
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(iso));
}

/** A positive span as "2 d 3 h", "1 h 23 min" or "4 min". */
export function formatSpan(t: TFunction, ms: number): string {
  const minutes = Math.max(0, Math.ceil(ms / 60_000));
  const d = Math.floor(minutes / 1440);
  const h = Math.floor((minutes % 1440) / 60);
  const m = minutes % 60;
  if (d > 0) return t("span.dh", { d, h });
  if (h > 0) return t("span.hm", { h, m });
  return t("span.m", { m });
}

/** "5 minutes ago", localised by the browser. */
export function formatAgo(iso: string, now: number, language: string): string {
  const minutes = Math.round((new Date(iso).getTime() - now) / 60_000);
  const rtf = new Intl.RelativeTimeFormat(language, { numeric: "auto" });
  if (Math.abs(minutes) < 60) return rtf.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 48) return rtf.format(hours, "hour");
  return rtf.format(Math.round(hours / 24), "day");
}
