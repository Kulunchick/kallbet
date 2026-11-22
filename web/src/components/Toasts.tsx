import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

type Notice = { success: string } | { error: string };
const eventName = "kallbet:notice";

export function notify(notice: Notice) {
  window.dispatchEvent(new CustomEvent(eventName, { detail: notice }));
}

export function Toasts() {
  const { t } = useTranslation("common");
  const [notice, setNotice] = useState<Notice | null>(null);

  useEffect(() => {
    let timeout: number | undefined;
    const receive = (event: Event) => {
      setNotice((event as CustomEvent<Notice>).detail);
      window.clearTimeout(timeout);
      timeout = window.setTimeout(() => setNotice(null), 5000);
    };
    window.addEventListener(eventName, receive);
    return () => {
      window.clearTimeout(timeout);
      window.removeEventListener(eventName, receive);
    };
  }, []);

  if (!notice) return null;
  const message = "error" in notice
    ? t([
        `common:errors.${notice.error}`, `leagues:errors.${notice.error}`,
        `matches:errors.${notice.error}`, `pickem:errors.${notice.error}`,
        `overrides:errors.${notice.error}`, `account:errors.${notice.error}`,
        "common:errors.unknown",
      ])
    : t(`notices.${notice.success}`);
  return (
    <div className="card fixed right-4 top-4 z-50 flex max-w-[calc(100vw-2rem)] items-center gap-4 bg-raised px-4 py-3 shadow-lg">
      <span role={"error" in notice ? "alert" : "status"}>{message}</span>
      <button type="button" className="text-mute" aria-label={t("notices.dismiss")} onClick={() => setNotice(null)}>
        &times;
      </button>
    </div>
  );
}
