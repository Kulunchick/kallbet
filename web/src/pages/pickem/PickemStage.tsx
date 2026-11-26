import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { useSession } from "../account/session";
import { savePickem, type PickemView } from "./api";
import { LeaderboardPanel, type BoardState } from "./LeaderboardPanel";
import { MemberPickems, SwissChips } from "./MemberPickems";
import { SaveBar } from "./SaveBar";
import { LoadError, StageSkeleton } from "./StageStates";
import { StageStatus, useStagePickem } from "./StageStatus";
import { SwissBoard, SwissGroupsReadOnly } from "./SwissBoard";
import { MD, useMedia } from "./useMedia";
import {
  TOTAL, draftFrom, isSwissComplete, place, placedCount, toBody, unplace, type Choice, type Draft,
} from "./swiss";

/** A Swiss Stage's Pick'em: the board until Lock, everyone's Pick'ems after it. */
export function PickemStage({
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
  const nameOf = (id: number) => view.teams.find((x) => x.id === id)?.name ?? `#${id}`;
  const leaderboard = <LeaderboardPanel board={board} open={view.lock.locked} />;

  let body: ReactNode;
  if (view.lock.locked) {
    body = (
      <>
        {leaderboard}
        <MemberPickems members={view.pickems} me={me}>
          {(member) => (member.shape === "swiss" ? <SwissChips picks={member} nameOf={nameOf} /> : null)}
        </MemberPickems>
      </>
    );
  } else if (view.lock.can_edit) {
    body = <SwissEditor view={view} league={league} stage={stage} reload={reload} onSaved={onSaved} leaderboard={leaderboard} />;
  } else {
    body = (
      <>
        {view.my_pickem?.shape === "swiss" && (
          <section aria-labelledby="waiting" className="flex flex-col gap-3">
            <h2 id="waiting" className="font-display text-lg text-fg">
              {t("waitingToReveal")}
            </h2>
            <SwissGroupsReadOnly picks={view.my_pickem} nameOf={nameOf} />
          </section>
        )}
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

function SwissEditor({
  view,
  league,
  stage,
  reload,
  onSaved,
  leaderboard,
}: {
  view: PickemView;
  league: number;
  stage: number;
  reload: () => void;
  onSaved: () => void;
  leaderboard: ReactNode;
}) {
  const { t } = useTranslation("pickem");
  const desktop = useMedia(MD);
  const mine = view.my_pickem?.shape === "swiss" ? view.my_pickem : null;
  const [draft, setDraft] = useState<Draft>(() => draftFrom(mine));
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
    setDraft(draftFrom(mine));
  }, [league, stage, savedAt]); // eslint-disable-line react-hooks/exhaustive-deps

  const complete = isSwissComplete(draft);

  async function save() {
    if (!complete || saving) return;
    setSaving(true);
    setError(null);
    try {
      await savePickem(league, stage, toBody(view.teams, draft));
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
      placed={placedCount(draft)}
      total={TOTAL}
      complete={complete}
      saving={saving}
      hasSaved={view.my_pickem !== null}
      moreKey="placeMore"
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
        <p className="text-sm text-mute">{t("boardHelp")}</p>
        <SwissBoard
          teams={view.teams}
          draft={draft}
          disabled={saving}
          onPlace={(team: number, choice: Choice) => setDraft((d) => place(d, team, choice))}
          onRemove={(team: number) => setDraft((d) => unplace(d, team))}
        />
        {desktop && bar}
      </div>
      {leaderboard}
      {!desktop && bar}
    </div>
  );
}
