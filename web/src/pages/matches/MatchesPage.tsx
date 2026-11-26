import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useLoad } from "../../api/useLoad";
import { getLeague } from "../leagues/api";
import { MatchCard } from "./MatchCard";
import { getMatches, predict, type MatchCard as Card, type MatchesView, type TeamRef } from "./api";
import { currentStage, nextUnpicked, orderMatchGroups, stripModel, undoTarget } from "./layout";
import { LoadFailed, MatchesSkeleton, NoMatches, RefreshFailed } from "./MatchesStates";
import { NextUpStrip } from "./NextUpStrip";
import { PickToast, type PickToastData } from "./PickToast";
import { StageSection } from "./StageSection";
import { useNow } from "./time";

/** How often the page refetches, so new Swiss rounds and Locks show up. */
const REFRESH_MS = 60_000;

export function MatchesPage() {
  const { t } = useTranslation("matches");
  const { t: tPicks } = useTranslation("picks");
  const league = Number(useParams().id);
  const { result, reload } = useLoad(() => getMatches(league), [league]);
  // One extra request: it says whether this is an admin (the source line on each
  // card) and whether the League is archived (no picking, so no strip).
  const { result: leagueResult } = useLoad(() => getLeague(league), [league]);
  const now = useNow();
  const [view, setView] = useState<MatchesView | null>(null);
  const [saving, setSaving] = useState<Set<number>>(() => new Set());
  const [error, setError] = useState<{ match: number; error: unknown } | null>(null);
  const [toast, setToast] = useState<PickToastData | null>(null);
  const dismissToast = useCallback(() => setToast(null), []);

  useEffect(() => {
    if (result.state === "ready") setView(result.data);
  }, [result]);

  useEffect(() => {
    const id = window.setInterval(reload, REFRESH_MS);
    return () => window.clearInterval(id);
  }, [reload]);

  useEffect(() => {
    const deadlines = view?.stages.flatMap((stage) =>
      stage.groups.flatMap((group) => group.matches.flatMap((match) =>
        match.lock.edit_window && match.lock.locks_at ? [Date.parse(match.lock.locks_at)] : [],
      )),
    );
    if (!deadlines?.length) return;
    const next = Math.min(...deadlines);
    const timer = window.setTimeout(reload, Math.max(0, next - Date.now()) + 250);
    return () => window.clearTimeout(timer);
  }, [view, reload]);

  const cards = view?.stages.flatMap((stage) => stage.groups.flatMap((group) => group.matches)) ?? [];

  // Undo runs from a toast made in an earlier render: it must see the cards of now.
  const latest = useRef({ cards, saving });
  useEffect(() => {
    latest.current = { cards, saving };
  });

  async function onPredict(match: number, team: TeamRef, undo = false) {
    const card = latest.current.cards.find((c) => c.id === match);
    if (latest.current.saving.has(match) || card?.my_prediction === team.id) return;
    const previous = card ? undoTarget(card, team.id) : null;
    latest.current.saving = new Set(latest.current.saving).add(match);
    setSaving((pending) => new Set(pending).add(match));
    setError(null);
    try {
      await predict(league, match, team.id, true);
      setView((v) => v && withPrediction(v, match, team.id));
      setToast({
        id: Date.now(),
        text: tPicks(undo ? "toast.restored" : "toast.saved", { team: team.name }),
        // Only a changed pick can be undone: there is no way to take a pick back.
        onUndo: previous && !undo ? () => void onPredict(match, previous, true) : null,
      });
      // The count changed, and this Prediction may have opened the final edit window.
      reload();
    } catch (e) {
      setError({ match, error: e });
      setToast(null);
      reload();
    } finally {
      setSaving((pending) => {
        const next = new Set(pending);
        next.delete(match);
        return next;
      });
    }
  }

  function pickNext() {
    const target = nextUnpicked(cards, now);
    const element = target && document.getElementById(`match-${target.id}`);
    if (!element) return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    element.scrollIntoView({ behavior: reduced ? "auto" : "smooth", block: "center" });
    // The scroll is already under way: do not let focusing restart it.
    (element.querySelector<HTMLElement>("button[aria-pressed]") ?? element).focus({ preventScroll: true });
  }

  const header = (
    <header className="flex flex-col">
      <Link to={`/leagues/${league}`} className="-ml-2 inline-flex min-h-11 items-center gap-1 self-start rounded-lg px-2 text-sm text-mute hover:text-fg">
        <span aria-hidden="true">←</span>
        {t("backToLeague")}
      </Link>
      <h1 className="font-display text-2xl font-extrabold text-fg md:text-3xl">{t("title")}</h1>
      {view && <p className="mt-1 text-mute">{view.event.name}</p>}
    </header>
  );
  const frame = "flex flex-col gap-6 py-4 md:py-8";

  if (view === null) {
    return (
      <div className={frame}>
        {header}
        {result.state === "error" ? <LoadFailed league={league} onRetry={reload} /> : <MatchesSkeleton />}
      </div>
    );
  }

  const isAdmin = leagueResult.state === "ready" && leagueResult.data.my_role === "admin";
  const canPick = leagueResult.state === "ready" && leagueResult.data.archived_at === null;
  const strip = canPick ? stripModel(cards, now) : null;

  const renderCard = (card: Card) => (
    <div key={card.id} id={`match-${card.id}`} tabIndex={-1} className="flex scroll-mt-24 flex-col gap-1 outline-none">
      <MatchCard
        card={card}
        now={now}
        saving={saving.has(card.id)}
        onPredict={(team) => void onPredict(card.id, team)}
        showMeta={isAdmin}
      />
      {error?.match === card.id && (
        <p role="alert" className="px-1 text-sm text-danger">{errorMessage(t, error.error)}</p>
      )}
    </div>
  );

  const stages = orderMatchGroups(view.stages.map((stage) => ({
    ...stage,
    groups: orderMatchGroups(stage.groups),
    matches: stage.groups.flatMap((group) => group.matches),
  })));
  const current = currentStage(stages);

  return (
    <div className={frame}>
      {header}

      {result.state === "error" && <RefreshFailed onRetry={reload} />}

      {strip && <NextUpStrip model={strip} now={now} onPickNext={pickNext} />}

      {view.stages.length === 0 && <NoMatches onCheck={reload} />}

      {stages.map((stage) => (
        <StageSection key={stage.ordinal} stage={stage} current={current} now={now} renderCard={renderCard} />
      ))}

      <PickToast toast={toast} onDismiss={dismissToast} />
    </div>
  );
}

function withPrediction(view: MatchesView, match: number, team: number): MatchesView {
  return {
    ...view,
    stages: view.stages.map((stage) => ({
      ...stage,
      groups: stage.groups.map((group) => ({
        ...group,
        matches: group.matches.map((m) => (m.id === match ? { ...m, my_prediction: team } : m)),
      })),
    })),
  };
}
