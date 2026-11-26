import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Avatar } from "../../components/Avatar";
import { Popover, popoverItemClass } from "../../components/Popover";
import { Sheet } from "../../components/Sheet";
import { Dots } from "./icons";
import type { Member } from "./api";
import { MD, useMediaQuery } from "./useMediaQuery";

export type MemberRowProps = {
  member: Member;
  /** Is this row the signed-in User? */
  isMe: boolean;
  /** The row's actions (League Admin on someone who is not the creator). */
  manageable: boolean;
  /** Kicking is offered only to someone else, and never in an archived League. */
  kickable: boolean;
  busy: boolean;
  onToggleAdmin: (member: Member) => void;
  onKick: (member: Member) => void;
};

const chip =
  "shrink-0 rounded-full border border-line px-2.5 py-0.5 text-xs font-semibold text-mute";
const menuButton =
  "tap shrink-0 rounded-full text-mute transition-transform duration-[140ms] ease-out focus-visible:text-fg active:scale-[.97] disabled:opacity-50 motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg";
const sheetItem =
  "flex min-h-12 w-full items-center rounded-xl px-3 text-left text-base transition-transform duration-[140ms] ease-out active:scale-[.97] motion-reduce:active:scale-100 [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised";

/** One Member: avatar, nickname, role chip, and (for a League Admin) a "···" menu of actions. */
export function MemberRow({ member, isMe, manageable, kickable, busy, onToggleAdmin, onKick }: MemberRowProps) {
  const { t } = useTranslation("leagues");
  const isAdmin = member.role === "admin";
  const chipText = isAdmin ? t(isMe ? "role.adminYou" : "role.adminShort") : isMe ? t("role.you") : null;

  return (
    <li className="card flex min-h-14 items-center gap-3 py-1.5 pl-4 pr-2">
      <Avatar nickname={member.nickname} url={member.avatar_url} />
      <span className="min-w-0 flex-1 truncate font-semibold">{member.nickname}</span>
      {chipText && <span className={chip}>{chipText}</span>}
      {manageable ? (
        <MemberMenu
          member={member}
          kickable={kickable}
          busy={busy}
          onToggleAdmin={onToggleAdmin}
          onKick={onKick}
        />
      ) : null}
    </li>
  );
}

function MemberMenu({
  member,
  kickable,
  busy,
  onToggleAdmin,
  onKick,
}: Pick<MemberRowProps, "member" | "kickable" | "busy" | "onToggleAdmin" | "onKick">) {
  const { t } = useTranslation("leagues");
  const { t: ui } = useTranslation("ui");
  const desktop = useMediaQuery(MD);
  const [open, setOpen] = useState(false);
  const label = t("manage.menuLabel", { name: member.nickname });
  const toggleLabel = member.role === "admin" ? t("manage.removeAdmin") : t("manage.makeAdmin");

  if (desktop) {
    return (
      <Popover label={label} triggerClassName={menuButton} trigger={<Dots />}>
        <button
          type="button"
          role="menuitem"
          className={popoverItemClass}
          disabled={busy}
          onClick={() => onToggleAdmin(member)}
        >
          {toggleLabel}
        </button>
        {kickable && (
          <button
            type="button"
            role="menuitem"
            className={`${popoverItemClass} text-danger!`}
            disabled={busy}
            onClick={() => onKick(member)}
          >
            {t("manage.kick")}
          </button>
        )}
      </Popover>
    );
  }

  // Below `md` a Popover would be cramped against the screen edge: use a bottom sheet.
  const choose = (action: (member: Member) => void) => {
    setOpen(false);
    action(member);
  };
  return (
    <>
      <button
        type="button"
        aria-label={label}
        aria-haspopup="dialog"
        className={menuButton}
        disabled={busy}
        onClick={() => setOpen(true)}
      >
        <Dots />
      </button>
      <Sheet
        open={open}
        onClose={() => setOpen(false)}
        title={member.nickname}
        footer={
          <button type="button" className="btn-ghost w-full" onClick={() => setOpen(false)}>
            {ui("cancel")}
          </button>
        }
      >
        <div className="flex flex-col gap-1 pb-2">
          <button type="button" className={sheetItem} data-autofocus="" onClick={() => choose(onToggleAdmin)}>
            {toggleLabel}
          </button>
          {kickable && (
            <button type="button" className={`${sheetItem} text-danger`} onClick={() => choose(onKick)}>
              {t("manage.kick")}
            </button>
          )}
        </div>
      </Sheet>
    </>
  );
}
