import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useLoad } from "../../api/useLoad";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { useSession } from "../account/session";
import { getLeague, kickMember, leaveLeague, setMemberRole, type LeagueDetail, type Member } from "./api";
import { AdminTools } from "./AdminTools";
import { ArchiveBox, ChevronLeft, ChevronRight } from "./icons";
import { InviteCard } from "./InviteCard";
import { MemberRow } from "./MemberRow";
import { LG, useMediaQuery } from "./useMediaQuery";

export function LeaguePage() {
  const { t } = useTranslation("leagues");
  const id = Number(useParams().id);
  const { result, reload } = useLoad(() => getLeague(id), [id]);
  const pending = result.state === "ready" && result.data.matches_pending;

  useEffect(() => {
    if (!pending) return;
    const timer = window.setInterval(reload, 10_000);
    return () => window.clearInterval(timer);
  }, [pending, reload]);

  if (result.state === "loading") return <LeagueSkeleton />;
  if (result.state === "error") {
    return (
      <div className="flex flex-col gap-4 py-8">
        <BackLink />
        <p role="alert" className="card px-4 py-3 text-danger">
          {errorMessage(t, result.error)}
        </p>
        <div className="flex flex-col gap-3 sm:flex-row">
          <button type="button" className="btn-ghost" onClick={reload}>
            {t("retry")}
          </button>
          <Link to="/leagues" className="btn-ghost">
            {t("backToLeagues")}
          </Link>
        </div>
      </div>
    );
  }
  return <LeagueView league={result.data} reload={reload} />;
}

function LeagueView({ league, reload }: { league: LeagueDetail; reload: () => void }) {
  const { t } = useTranslation("leagues");
  const navigate = useNavigate();
  const me = useSession().user?.id;
  const twoColumns = useMediaQuery(LG);
  const [actionError, setActionError] = useState<unknown>(null);
  const [busy, setBusy] = useState<Set<number>>(() => new Set());
  // Kept after the dialog closes, so its text does not change during the exit transition.
  const [kicking, setKicking] = useState<Member | null>(null);
  const [kickOpen, setKickOpen] = useState(false);
  const [leaving, setLeaving] = useState(false);
  // A kicked Member's row is gone, and with it the "···" button that had focus: land on the heading instead.
  const membersHeading = useRef<HTMLHeadingElement>(null);
  const refocusAfterKick = useRef(false);
  useEffect(() => {
    // Wait for the dialog to close: closing it hands focus back to its (now stale) trigger.
    if (kickOpen || !refocusAfterKick.current) return;
    refocusAfterKick.current = false;
    membersHeading.current?.focus();
  }, [kickOpen]);

  const isAdmin = league.my_role === "admin";
  const archived = league.archived_at !== null;
  const token = league.invite_token;
  const alone = league.members.length === 1;
  // Only League Admins receive a token; an archived League has no Invite Link to share.
  const showInvite = isAdmin && !archived && token !== undefined;
  // The page has one emerald button: Share invite while the creator is alone, otherwise Matches.
  const matchesPrimary = !(showInvite && alone);

  async function toggleAdmin(member: Member) {
    if (busy.has(member.user_id)) return;
    setBusy((set) => new Set(set).add(member.user_id));
    setActionError(null);
    try {
      await setMemberRole(league.id, member.user_id, member.role === "admin" ? "member" : "admin");
      reload();
    } catch (e) {
      setActionError(e);
    } finally {
      setBusy((set) => {
        const next = new Set(set);
        next.delete(member.user_id);
        return next;
      });
    }
  }

  const header = (
    <header className="flex flex-col gap-1">
      <BackLink />
      <h1 className="mt-1 break-words font-display text-2xl font-bold leading-tight text-fg">{league.name}</h1>
      <p className="text-mute">{league.event.name}</p>
      {league.matches_pending && (
        <p role="status" className="text-sm text-mute">
          {t("matchesPending")}
        </p>
      )}
      {archived && (
        <p className="card mt-2 flex items-start gap-2 px-3 py-2 text-sm text-mute">
          <ArchiveBox className="mt-0.5 shrink-0" />
          <span>{t("archived")}</span>
        </p>
      )}
    </header>
  );

  const invite = showInvite ? (
    <InviteCard leagueId={league.id} token={token} alone={alone} onReissued={reload} />
  ) : null;

  const nav = (
    <div className="flex flex-col gap-3">
      <Link to={`/leagues/${league.id}/matches`} className={`${matchesPrimary ? "btn-primary" : "btn-ghost"} w-full`}>
        {t("openMatches")}
      </Link>
      <nav aria-label={league.name} className="card overflow-hidden">
        <NavRow to={`/leagues/${league.id}/pickem`} title={t("pickem:open")} hint={t("nav.pickemHint")} />
        <NavRow
          to={`/leagues/${league.id}/leaderboard`}
          title={t("leaderboard:open")}
          hint={t("nav.leaderboardHint")}
          divider
        />
      </nav>
    </div>
  );

  const members = (
    <section className="flex flex-col gap-3" aria-labelledby="members-heading">
      <h2 id="members-heading" ref={membersHeading} tabIndex={-1} className="text-base font-semibold text-fg">
        {t("members.title", { count: league.members.length })}
      </h2>
      <ul className="flex flex-col gap-2">
        {league.members.map((m) => {
          // The creator can be neither kicked nor demoted, and nobody removes their own admin role here.
          const manageable = isAdmin && m.user_id !== league.creator_id && m.user_id !== me;
          return (
            <MemberRow
              key={m.user_id}
              member={m}
              isMe={m.user_id === me}
              manageable={manageable}
              kickable={manageable && !archived}
              busy={busy.has(m.user_id)}
              onToggleAdmin={toggleAdmin}
              onKick={(m) => {
                setKicking(m);
                setKickOpen(true);
              }}
            />
          );
        })}
      </ul>
      {actionError !== null && (
        <p role="alert" className="text-sm text-danger">
          {errorMessage(t, actionError)}
        </p>
      )}
    </section>
  );

  const tools = isAdmin ? <AdminTools league={league} reload={reload} /> : null;

  const leave = !league.is_creator ? (
    <button type="button" className="btn-ghost min-h-12 w-full text-mute" onClick={() => setLeaving(true)}>
      {t("leave.confirm")}
    </button>
  ) : null;

  let body: ReactNode;
  // Two columns only when there is something for the side column: a Member has no invite or tools.
  if (twoColumns && (invite || tools)) {
    body = (
      <div className="grid grid-cols-[minmax(0,1fr)_360px] items-start gap-8">
        <div className="flex flex-col gap-8">
          {nav}
          {members}
          {leave}
        </div>
        <aside className="flex flex-col gap-4">
          {invite}
          {tools}
        </aside>
      </div>
    );
  } else {
    body = (
      <div className="flex flex-col gap-6">
        {invite}
        {nav}
        {members}
        {tools}
        {leave}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 py-8">
      {header}
      {body}

      <ConfirmDialog
        open={kickOpen}
        onClose={() => setKickOpen(false)}
        title={t("manage.kickTitle", { name: kicking?.nickname ?? "" })}
        body={t("manage.kickBody", { name: kicking?.nickname ?? "" })}
        confirmLabel={t("manage.kickAction", { name: kicking?.nickname ?? "" })}
        tone="danger"
        onConfirm={async () => {
          if (!kicking) return;
          await kickMember(league.id, kicking.user_id);
          refocusAfterKick.current = true;
          reload();
        }}
      />
      <ConfirmDialog
        open={leaving}
        onClose={() => setLeaving(false)}
        title={t("leave.title")}
        body={t("leave.body")}
        confirmLabel={t("leave.confirm")}
        tone="danger"
        onConfirm={async () => {
          await leaveLeague(league.id);
          navigate("/leagues");
        }}
      />
    </div>
  );
}

function BackLink() {
  const { t } = useTranslation("leagues");
  return (
    <Link
      to="/leagues"
      className="-ml-1 inline-flex min-h-11 items-center gap-1 self-start rounded-lg pr-2 text-mute [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
    >
      <ChevronLeft />
      {t("back")}
    </Link>
  );
}

function NavRow({ to, title, hint, divider }: { to: string; title: string; hint: string; divider?: boolean }) {
  return (
    <Link
      to={to}
      className={`flex min-h-[52px] items-center justify-between gap-3 px-4 py-2 transition-colors duration-[140ms] ease-out [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised ${
        divider ? "border-t border-line" : ""
      }`}
    >
      <span className="flex min-w-0 flex-col">
        <span className="font-medium text-fg">{title}</span>
        <span className="text-sm leading-5 text-mute">{hint}</span>
      </span>
      <ChevronRight className="shrink-0 text-mute" />
    </Link>
  );
}

/** Card-shaped placeholders in the final layout, so the page does not jump when the League arrives. */
function LeagueSkeleton() {
  const { t } = useTranslation("leagues");
  const block = "animate-pulse rounded-2xl bg-raised motion-reduce:animate-none";
  return (
    <div className="flex flex-col gap-6 py-8" aria-busy="true">
      <p role="status" className="sr-only">
        {t("loading")}
      </p>
      <div aria-hidden className="flex flex-col gap-2">
        <div className="h-11" />
        <div className={`${block} h-8 w-2/3`} />
        <div className={`${block} h-6 w-1/2`} />
      </div>
      <div aria-hidden className="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_360px] lg:gap-8">
        <div className="flex flex-col gap-6">
          <div className={`${block} h-12`} />
          <div className={`${block} h-[105px]`} />
          <div className="flex flex-col gap-2">
            {[0, 1, 2].map((i) => (
              <div key={i} className={`${block} h-14`} />
            ))}
          </div>
        </div>
        <div className={`${block} hidden h-[52px] lg:block`} />
      </div>
    </div>
  );
}
