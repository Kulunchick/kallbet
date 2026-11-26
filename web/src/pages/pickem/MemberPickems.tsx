import { useId, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Avatar } from "../../components/Avatar";
import { TeamLogo } from "../../components/TeamLogo";
import type { MemberPickem } from "./api";
import { ChevronDown, ChevronRight } from "./icons";
import { Nobody } from "./StageStates";
import { CHOICES } from "./swiss";

/**
 * "Everyone's Pick'ems" after Lock: one collapsed 56 px card per Member (avatar,
 * nickname, Points, chevron). The viewer's own card, or else the first, starts
 * expanded. A Member without a Pick'em is not in the list (the API omits them).
 */
export function MemberPickems({
  members,
  me,
  children,
}: {
  members: MemberPickem[] | null;
  /** The signed-in User's id, when known. */
  me: number | undefined;
  /** The expanded body of one Member's card. */
  children: (member: MemberPickem) => ReactNode;
}) {
  const { t } = useTranslation("pickem");
  const [open, setOpen] = useState<Set<number>>(() => {
    const own = members?.find((m) => m.user_id === me) ?? members?.[0];
    return new Set(own ? [own.user_id] : []);
  });
  if (!members || members.length === 0) return <Nobody />;

  const toggle = (id: number) =>
    setOpen((current) => {
      const next = new Set(current);
      if (!next.delete(id)) next.add(id);
      return next;
    });

  return (
    <section aria-labelledby="everyone" className="flex flex-col gap-3">
      <h2 id="everyone" className="font-display text-lg text-fg">
        {t("everyone")}
      </h2>
      <ul className="flex flex-col gap-3">
        {members.map((member) => (
          <MemberCard key={member.user_id} member={member} expanded={open.has(member.user_id)} onToggle={() => toggle(member.user_id)}>
            {children(member)}
          </MemberCard>
        ))}
      </ul>
    </section>
  );
}

function MemberCard({
  member,
  expanded,
  onToggle,
  children,
}: {
  member: MemberPickem;
  expanded: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  const { t } = useTranslation("pickem");
  const bodyId = useId();
  return (
    <li className="card flex flex-col">
      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={bodyId}
        onClick={onToggle}
        className="flex min-h-14 w-full items-center gap-3 rounded-2xl px-4 py-2 text-left"
      >
        <Avatar url={member.avatar_url} nickname={member.nickname} className="size-8" />
        <span className="min-w-0 flex-1 truncate font-semibold text-fg">{member.nickname}</span>
        <span className="shrink-0 font-display text-sm text-gold-a">{t("points", { count: member.points })}</span>
        {expanded ? <ChevronDown className="size-5 shrink-0 text-mute" /> : <ChevronRight className="size-5 shrink-0 text-mute" />}
      </button>
      <div id={bodyId} hidden={!expanded} className="px-4 pb-4">
        {expanded && children}
      </div>
    </li>
  );
}

/**
 * A Member's Swiss Pick'em as chips (logo 20 + name, 36 px high). The chips are
 * neutral: the data the SPA gets does not say which Picks are settled or correct.
 */
export function SwissChips({
  picks,
  nameOf,
}: {
  picks: { three_nil: number[]; advance: number[]; zero_three: number[] };
  nameOf: (id: number) => string;
}) {
  const { t } = useTranslation("pickem");
  return (
    <div className="flex flex-col gap-3">
      {CHOICES.map((choice) => (
        <div key={choice} className="flex flex-col gap-2">
          <h3 className="font-sans text-[13px] font-normal text-mute">{t(`kind.${choice}`)}</h3>
          <ul className="flex flex-wrap gap-2">
            {picks[choice].map((team) => (
              <li
                key={team}
                className="inline-flex h-9 max-w-full items-center gap-2 rounded-full border border-line bg-raised px-3 text-sm text-fg"
              >
                <TeamLogo id={team} className="size-5" />
                <span className="truncate">{nameOf(team)}</span>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}
