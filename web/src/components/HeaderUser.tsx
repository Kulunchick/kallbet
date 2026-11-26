import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Link, useLocation, useMatch, useNavigate } from "react-router";

import { signOut, startSession, useSession } from "../pages/account/session";
import { Avatar } from "./Avatar";
import { LanguageSegmentedControl, LanguageSwitcher, MenuChevron } from "./LanguageSwitcher";
import { Popover, popoverItemClass } from "./Popover";

/** The header's User corner: language + sign-in while signed out, or the account menu popover. */
export function HeaderUser() {
  const { t } = useTranslation(["account", "ui"]);
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const invite = useMatch("/join/:token");
  const { loading, user } = useSession();

  useEffect(startSession, []);

  if (loading) return null;
  if (!user) {
    const login = invite ? `/login?next=${encodeURIComponent(pathname)}` : "/login";
    return (
      <>
        <LanguageSwitcher />
        <Link
          to={login}
          className="btn-ghost !min-h-11 whitespace-nowrap !px-3 !py-2 text-sm sm:!px-4"
        >
          {t("signIn")}
        </Link>
      </>
    );
  }
  return (
    <Popover
      align="end"
      label={t("accountMenu", { ns: "ui" })}
      triggerClassName="inline-flex min-h-11 min-w-11 items-center justify-center gap-2 rounded-xl border border-line bg-panel pl-1.5 pr-2 text-sm font-semibold text-fg transition-[background-color,scale] duration-[140ms] ease-out active:scale-[.97] motion-reduce:transition-[background-color] motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised aria-expanded:bg-raised sm:pr-2.5"
      trigger={({ open }) => (
        <>
          <Avatar url={user.avatar_url} nickname={user.nickname} className="size-8" />
          <span className="hidden max-w-36 truncate sm:inline">{user.nickname}</span>
          <MenuChevron open={open} />
        </>
      )}
    >
      <div className="w-60 max-w-full">
        <div className="border-b border-line px-3 py-2.5">
          <p className="truncate text-sm font-semibold text-fg">{user.nickname}</p>
          <p className="mt-0.5 text-xs text-mute">{t("signedInWithTelegram", { ns: "ui" })}</p>
        </div>
        <div className="py-1">
          <Link to="/leagues" role="menuitem" className={popoverItemClass}>
            {t("myLeagues")}
          </Link>
          <Link to="/profile" role="menuitem" className={popoverItemClass}>
            {t("profile.title")}
          </Link>
        </div>
        <div className="border-t border-line px-1.5 py-2">
          <p className="px-1.5 pb-1.5 text-xs font-medium text-mute">{t("language", { ns: "ui" })}</p>
          <LanguageSegmentedControl />
        </div>
        <div className="border-t border-line pt-1">
          <button
            type="button"
            role="menuitem"
            className={popoverItemClass}
            onClick={() => void signOut().then(() => navigate("/"))}
          >
            {t("signOut")}
          </button>
        </div>
      </div>
    </Popover>
  );
}
