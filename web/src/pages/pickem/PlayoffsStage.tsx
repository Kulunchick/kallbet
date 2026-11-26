import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { useSession } from "../account/session";
import { savePickem, type PickemView } from "./api";
import { fromPicks, isComplete, placedPicks, setWinner, toPicks, type Slot, type Winners } from "./bracket";
import { LeaderboardPanel, type BoardState } from "./LeaderboardPanel";
import { MemberPickems } from "./MemberPickems";
import { CompactBracket, PlayoffsBoard } from "./PlayoffsBoard";
import { SaveBar } from "./SaveBar";
import { BracketNotReady, LoadError, StageSkeleton } from "./StageStates";
import { StageStatus, useStagePickem } from "./StageStatus";
import { MD, useMedia } from "./useMedia";

/** A Playoffs Stage's Pick'em: the bracket until Lock, everyone's brackets after it. */
export function PlayoffsStage({
  league,
  stage,
  board,
  onSaved,
}: {
  league: number;
  stage: number;
  board: BoardState;
  onSaved: () => void;
}) {
  const { t } = useTranslation("pickem");
  const { result, reload } = useStagePickem(league, stage, onSaved);
  const me = useSession().user?.id;
  // A failed refresh keeps the last good view, so a draft is never thrown away.
  const lastGood = useRef<PickemView | null>(null);
  if (result.state === "ready") lastGood.current = result.data;
  const view = result.state === "ready" ? result.data : lastGood.current;

  if (!view) {
    return result.state === "error" ? <LoadError error={result.error} onRetry={reload} /> : <StageSkeleton />;
  }
  const slots = view.bracket?.slots ?? [];
  const nameOf = (id: number | null) =>
    id === null ? t("tbd") : (view.teams.find((x) => x.id === id)?.name ?? `#${id}`);
  const leaderboard = <LeaderboardPanel board={board} open={view.lock.locked} />;

  let body: ReactNode;
  if (view.lock.locked) {
    body = (
      <>
        {leaderboard}
        <MemberPickems members={view.pickems} me={me}>
          {(member) =>
            member.shape === "playoffs" ? (
              <CompactBracket slots={slots} winners={fromPicks(member.picks)} nameOf={nameOf} />
            ) : null
          }
        </MemberPickems>
      </>
    );
  } else if (view.lock.can_edit && view.bracket?.ready) {
    body = (
      <PlayoffsEditor view={view} slots={slots} nameOf={nameOf} league={league} stage={stage} reload={reload} onSaved={onSaved} leaderboard={leaderboard} />
    );
  } else {
    body = (
      <>
        {view.my_pickem?.shape === "playoffs" && (
          <section aria-labelledby="waiting" className="flex flex-col gap-3">
            <h2 id="waiting" className="font-display text-lg text-fg">
              {t("waitingToReveal")}
            </h2>
            <PlayoffsBoard slots={slots} winners={fromPicks(view.my_pickem.picks)} nameOf={nameOf} />
          </section>
        )}
        {!view.bracket?.ready && <BracketNotReady />}
        {leaderboard}
      </>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {result.state === "error" && <LoadError error={result.error} onRetry={reload} />}
      <StageStatus view={view} />
      {body}
    </div>
  );
}

function PlayoffsEditor({
  view,
  slots,
  nameOf,
  league,
  stage,
  reload,
  onSaved,
  leaderboard,
}: {
  view: PickemView;
  slots: Slot[];
  nameOf: (id: number | null) => string;
  league: number;
  stage: number;
  reload: () => void;
  onSaved: () => void;
  leaderboard: ReactNode;
}) {
  const { t } = useTranslation("pickem");
  const desktop = useMedia(MD);
  const mine = view.my_pickem?.shape === "playoffs" ? view.my_pickem : null;
  const [draft, setDraft] = useState<Winners>(() => (mine ? fromPicks(mine.picks) : {}));
  const [error, setError] = useState<unknown>(null);
  const [saving, setSaving] = useState(false);
  const savedAt = view.my_pickem?.updated_at;

  // A saved Pick'em that arrives (after Save, or from another tab) replaces the draft.
  const first = useRef(true);
  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    setDraft(mine ? fromPicks(mine.picks) : {});
  }, [league, stage, savedAt]); // eslint-disable-line react-hooks/exhaustive-deps

  const complete = isComplete(slots, draft);

  async function save() {
    if (saving || !complete) return;
    setSaving(true);
    setError(null);
    try {
      await savePickem(league, stage, { picks: toPicks(slots, draft) });
      reload();
      onSaved();
    } catch (e) {
      setError(e);
    } finally {
      setSaving(false);
    }
  }

  const bar = (
    <SaveBar
      placed={placedPicks(slots, draft)}
      total={slots.length}
      complete={complete}
      saving={saving}
      hasSaved={view.my_pickem !== null}
      moreKey="pickMore"
      error={error}
      inline={desktop}
      onSave={save}
      onShowEveryone={() => {
        setError(null);
        reload();
      }}
    />
  );

  return (
    <div className="flex flex-col gap-4" aria-busy={saving}>
      <div className="flex flex-col gap-4 md:card md:p-6">
        <p className="text-sm text-mute">{t("bracketHelp")}</p>
        <PlayoffsBoard
          slots={slots}
          winners={draft}
          nameOf={nameOf}
          disabled={saving}
          onPick={(round, slot, team) => setDraft((d) => setWinner(slots, d, round, slot, team))}
        />
        {desktop && bar}
      </div>
      {leaderboard}
      {!desktop && bar}
    </div>
  );
}
