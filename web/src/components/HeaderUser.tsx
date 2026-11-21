import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";

import { signOut, startSession, useSession } from "../pages/account/session";
import { Avatar } from "./Avatar";

/** The header's User corner: the sign-in button, or the avatar and Nickname with sign-out. */
export function HeaderUser() {
  const { t } = useTranslation("account");
  const navigate = useNavigate();
  const { loading, user } = useSession();

  useEffect(startSession, []);

  if (loading) return null;
  if (!user) {
    return (
      <Link to="/login" className="btn-primary !px-4 !py-2 text-sm">
        {t("signIn")}
      </Link>
    );
  }
  return (
    <div className="flex items-center gap-2">
      <Link
        to="/profile"
        className="flex items-center gap-2 rounded-xl py-1 pl-1 pr-2 transition-colors hover:bg-raised"
      >
        <Avatar url={user.avatar_url} nickname={user.nickname} />
        <span className="max-w-32 truncate text-sm font-semibold">{user.nickname}</span>
      </Link>
      <button
        type="button"
        className="rounded-xl px-2 py-2 text-xs text-mute transition-colors hover:text-fg"
        onClick={() => void signOut().then(() => navigate("/"))}
      >
        {t("signOut")}
      </button>
    </div>
  );
}
