import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { useLoad, type Loaded } from "../../api/useLoad";
import { getLeague } from "../leagues/api";
import { getMatches, type StageView } from "../matches/api";
import { getLeaderboard, getPickem } from "./api";
import { ChevronLeft } from "./icons";
import type { BoardState } from "./LeaderboardPanel";
import { PickemStage } from "./PickemStage";
import { PlayoffsStage } from "./PlayoffsStage";
import { SegmentedTabs, tabId } from "./SegmentedTabs";
import { LoadError, StageSkeleton } from "./StageStates";
import { defaultStage, stageFromHash } from "./stages";

/** The League's Pick'em: the Stage switcher and one Stage at a time. */
export function PickemPage() {
  const { t } = useTranslation("pickem");
  const league = Number(useParams().id);
  const stages = useLoad(() => getMatches(league), [league]);
  const board = useLoad(() => getLeaderboard(league), [league]);
  const detail = useLoad(() => getLeague(league), [league]).result;
  const archived = detail.state === "ready" && detail.data.archived_at !== null;
  const eventName = detail.state === "ready" ? detail.data.event.name : stages.result.state === "ready" ? stages.result.data.event.name : null;

  const playable =
    stages.result.state === "ready"
      ? stages.result.data.stages.filter((stage) => stage.kind === "swiss" || stage.kind === "playoffs")
      : [];

  return (
    <div className="flex flex-col gap-6 pb-10 pt-4">
      <header className="flex flex-col">
        <Link to={`/leagues/${league}`} className="inline-flex min-h-11 items-center gap-1 self-start text-sm text-mute [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg">
          <ChevronLeft className="size-4" />
          {t("backToLeague")}
        </Link>
        <h1 className="font-display text-2xl font-extrabold text-fg">{t("title")}</h1>
        {eventName && <p className="mt-1 text-mute">{eventName}</p>}
      </header>

      {stages.result.state === "loading" && <StageSkeleton />}
      {stages.result.state === "error" && <LoadError error={stages.result.error} onRetry={stages.reload} />}
      {stages.result.state === "ready" && playable.length > 0 && (
        <StageArea league={league} stages={playable} board={{ result: board.result, archived }} onSaved={board.reload} />
      )}
    </div>
  );
}

function StageArea({
  league,
  stages,
  board,
  onSaved,
}: {
  league: number;
  stages: StageView[];
  board: BoardState;
  onSaved: () => void;
}) {
  const { t } = useTranslation("pickem");
  const ordinals = stages.map((s) => s.ordinal);
  const key = ordinals.join(",");
  const fromHash = () => stageFromHash(window.location.hash, ordinals);
  const [chosen, setChosen] = useState<number | null>(fromHash);
  const panel = useId();
  const prefix = useId();

  // Which Stage to open on when the link does not say: the first one open to edit.
  const needsDefault = chosen === null && stages.length > 1;
  const peek: { result: Loaded<number | null> } = useLoad(async () => {
    if (!needsDefault) return null;
    const views = await Promise.all(
      ordinals.map((ordinal) =>
        getPickem(league, ordinal).then(
          (view) => ({ ordinal, canEdit: view.lock.can_edit }),
          () => ({ ordinal, canEdit: false }),
        ),
      ),
    );
    return defaultStage(views);
  }, [league, key, needsDefault]);

  useEffect(() => {
    const onHash = () => {
      const next = stageFromHash(window.location.hash, ordinals);
      if (next !== null) setChosen(next);
    };
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, [key]); // eslint-disable-line react-hooks/exhaustive-deps

  const shown =
    chosen !== null && ordinals.includes(chosen)
      ? chosen
      : stages.length === 1
        ? ordinals[0]
        : peek.result.state === "ready"
          ? (peek.result.data ?? ordinals[ordinals.length - 1])
          : peek.result.state === "error"
            ? ordinals[ordinals.length - 1]
            : null;

  const select = (ordinal: number) => {
    setChosen(ordinal);
    window.history.replaceState(null, "", `#stage-${ordinal}`);
  };

  const stage = stages.find((s) => s.ordinal === shown);

  return (
    <>
      {stages.length > 1 && (
        <SegmentedTabs
          items={stages.map((s) => ({ id: s.ordinal, label: s.name }))}
          value={shown ?? ordinals[0]}
          onChange={select}
          label={t("stageSwitcher")}
          idPrefix={prefix}
          panelId={panel}
          height="min-h-11"
        />
      )}
      <div
        id={panel}
        role={stages.length > 1 ? "tabpanel" : undefined}
        aria-labelledby={stages.length > 1 && stage ? tabId(prefix, stage.ordinal) : undefined}
      >
        <section
          id={stage ? `stage-${stage.ordinal}` : undefined}
          aria-label={stage?.name}
          className="flex scroll-mt-6 flex-col gap-4"
        >
          {stage === undefined ? (
            <StageSkeleton />
          ) : stage.kind === "playoffs" ? (
            <PlayoffsStage key={stage.ordinal} league={league} stage={stage.ordinal} board={board} onSaved={onSaved} />
          ) : (
            <PickemStage key={stage.ordinal} league={league} stage={stage.ordinal} board={board} onSaved={onSaved} />
          )}
        </section>
      </div>
    </>
  );
}
