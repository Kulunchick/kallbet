import { useTranslation } from "react-i18next";
import { Link, Outlet } from "react-router";

import { HeaderUser } from "../components/HeaderUser";
import { LanguageSwitcher } from "../components/LanguageSwitcher";
import { Logo } from "../components/Logo";
import { ServerStatus } from "../components/ServerStatus";

/** The frame around every page: header, page, footer. */
export function Layout() {
  const { t } = useTranslation();
  return (
    <div className="flex min-h-dvh flex-col">
      <header className="sticky top-0 z-10 border-b border-line/60 bg-ink/80 backdrop-blur">
        <div className="mx-auto flex h-16 w-full max-w-5xl items-center justify-between gap-4 px-4">
          <Link to="/" aria-label={t("appName")}>
            <Logo className="h-8 w-auto" />
          </Link>
          <div className="flex items-center gap-3">
            <LanguageSwitcher />
            <HeaderUser />
          </div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-5xl flex-1 px-4">
        <Outlet />
      </main>

      <footer className="border-t border-line/60">
        <div className="mx-auto flex w-full max-w-5xl flex-wrap items-center justify-between gap-2 px-4 py-5 text-xs text-mute">
          <span>{t("footer.noMoney")}</span>
          <ServerStatus />
        </div>
      </footer>
    </div>
  );
}
