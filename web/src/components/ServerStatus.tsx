import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { api } from "../api/client";

type Status = "checking" | "online" | "offline";

/** A small "is the API reachable" light in the footer. */
export function ServerStatus() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<Status>("checking");

  useEffect(() => {
    const controller = new AbortController();
    api<{ status: string }>("/health", { signal: controller.signal })
      .then(() => setStatus("online"))
      .catch(() => {
        if (!controller.signal.aborted) setStatus("offline");
      });
    return () => controller.abort();
  }, []);

  const dot =
    status === "online" ? "bg-emerald-b" : status === "offline" ? "bg-danger" : "bg-mute";
  return (
    <span className="inline-flex items-center gap-2" data-status={status}>
      <span className={`size-2 rounded-full ${dot}`} aria-hidden />
      {t(`server.${status}`)}
    </span>
  );
}
