import { useTranslation } from "react-i18next";
import { Link, Outlet } from "react-router";

import { HeaderUser } from "../components/HeaderUser";
import { Logo } from "../components/Logo";
import { ServerStatus } from "../components/ServerStatus";

/** The frame around every page: skip link, sticky header, page, footer. */
export function Layout() {
  const { t } = useTranslation(["common", "ui"]);
  return (
    <div className="flex min-h-dvh flex-col">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:left-[max(1rem,env(safe-area-inset-left))] focus:top-[calc(0.5rem+env(safe-area-inset-top))] focus:z-50 focus:inline-flex focus:min-h-11 focus:items-center focus:rounded-xl focus:border focus:border-emerald-b focus:bg-raised focus:px-4 focus:text-sm focus:font-semibold focus:text-fg focus:shadow-lg"
      >
        {t("skipToContent", { ns: "ui" })}
      </a>

      <header className="sticky top-0 z-20 border-b border-line/60 bg-ink/80 pt-[env(safe-area-inset-top)] backdrop-blur [@media(prefers-reduced-transparency:reduce)]:bg-ink [@media(prefers-reduced-transparency:reduce)]:backdrop-blur-none">
        <div className="mx-auto flex h-16 w-full max-w-5xl items-center justify-between gap-2 pl-[max(1rem,env(safe-area-inset-left))] pr-[max(1rem,env(safe-area-inset-right))] sm:gap-4">
          <Link
            to="/"
            aria-label={t("appName")}
            className="inline-flex min-h-11 shrink-0 items-center rounded-lg"
          >
            <Logo className="h-8 w-auto" />
          </Link>
          <nav aria-label={t("primaryNav", { ns: "ui" })} className="flex items-center gap-2">
            <HeaderUser />
          </nav>
        </div>
      </header>

      <main
        id="main"
        tabIndex={-1}
        className="mx-auto w-full max-w-5xl flex-1 pl-[max(1rem,env(safe-area-inset-left))] pr-[max(1rem,env(safe-area-inset-right))] focus:outline-none"
      >
        <Outlet />
      </main>

      <footer className="border-t border-line/60 pb-[env(safe-area-inset-bottom)]">
        <div className="mx-auto flex w-full max-w-5xl flex-wrap items-center justify-between gap-2 py-5 pl-[max(1rem,env(safe-area-inset-left))] pr-[max(1rem,env(safe-area-inset-right))] text-xs text-mute">
          <span>{t("footer.noMoney")}</span>
          <ServerStatus />
        </div>
      </footer>
    </div>
  );
}
