import { useId, useState } from "react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "../../api/client";
import type { Loaded } from "../../api/useLoad";
import { Avatar } from "../../components/Avatar";
import { Medal } from "../../components/Medal";
import type { LeaderboardEntry } from "./api";
import { ChevronDown, ChevronRight } from "./icons";

export type BoardState = { result: Loaded<{ entries: LeaderboardEntry[] }>; archived: boolean };

/**
 * The Pick'em Leaderboard. While the shown Stage is open to edit it is a
 * collapsed 56 px disclosure row; once the Stage is locked it is an open card.
 */
export function LeaderboardPanel({ board, open }: { board: BoardState; open: boolean }) {
  return open ? <LeaderboardCard board={board} /> : <LeaderboardDisclosure board={board} />;
}

function LeaderboardCard({ board }: { board: BoardState }) {
  const { t } = useTranslation("pickem");
  return (
    <section aria-labelledby="pickem-board" className="card flex flex-col">
      <div className="flex items-center justify-between gap-3 px-4 py-4">
        <h2 id="pickem-board" className="font-display text-base text-fg">
          {t("leaderboard")}
        </h2>
        <span className="text-[13px] text-mute">{t("pointsHeading")}</span>
      </div>
      <LeaderboardRows board={board} />
    </section>
  );
}

function LeaderboardDisclosure({ board }: { board: BoardState }) {
  const { t } = useTranslation("pickem");
  const [expanded, setExpanded] = useState(false);
  const bodyId = useId();
  const count = board.result.state === "ready" ? board.result.data.entries.length : null;
  return (
    <section aria-labelledby={`${bodyId}-title`} className="card flex flex-col">
      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={bodyId}
        onClick={() => setExpanded((value) => !value)}
        className="flex min-h-14 w-full items-center gap-3 rounded-2xl px-4 py-2 text-left"
      >
        <span className="flex min-w-0 flex-1 flex-col">
          <span id={`${bodyId}-title`} className="font-display text-[15px] leading-tight text-fg">
            {t("leaderboard")}
          </span>
          {count !== null && <span className="text-[13px] text-mute">{t("leaderboardHint", { count })}</span>}
        </span>
        {expanded ? <ChevronDown className="size-5 shrink-0 text-mute" /> : <ChevronRight className="size-5 shrink-0 text-mute" />}
      </button>
      <div id={bodyId} hidden={!expanded} className="border-t border-line">
        {expanded && (
          <>
            <div className="flex justify-end px-4 pt-3 text-[13px] text-mute">{t("pointsHeading")}</div>
            <LeaderboardRows board={board} />
          </>
        )}
      </div>
    </section>
  );
}

function LeaderboardRows({ board }: { board: BoardState }) {
  const { t } = useTranslation("pickem");
  const { result, archived } = board;
  if (result.state === "loading") return <div aria-hidden className="mx-4 mb-4 h-14 rounded-xl bg-raised" />;
  if (result.state === "error") {
    return (
      <p role="alert" className="px-4 pb-4 text-sm text-danger">
        {errorMessage(t, result.error)}
      </p>
    );
  }
  return (
    <ol>
      {result.data.entries.map((e) => (
        <li key={e.user_id} className="flex h-14 items-center gap-3 border-t border-line px-4">
          <span className="w-6 text-center text-sm text-mute">{e.place}</span>
          {archived && <Medal place={e.place} />}
          <Avatar url={e.avatar_url} nickname={e.nickname} className="size-8" />
          <span className="min-w-0 flex-1 truncate font-semibold text-fg">{e.nickname}</span>
          <span className="font-display text-gold-a">{e.points}</span>
        </li>
      ))}
    </ol>
  );
}
