import type { TFunction } from "i18next";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { useLoad } from "../leagues/useLoad";
import { MatchCard } from "./MatchCard";
import { getMatches, predict, type MatchGroup, type MatchesView, type TeamRef } from "./api";
import { useNow } from "./time";

/** How often the page refetches, so new Swiss rounds and Locks show up. */
const REFRESH_MS = 60_000;

export function MatchesPage() {
  const { t } = useTranslation("matches");
  const league = Number(useParams().id);
  const { result, reload } = useLoad(() => getMatches(league), [league]);
  const now = useNow();
  const [view, setView] = useState<MatchesView | null>(null);
  const [saving, setSaving] = useState<number | null>(null);
  const [error, setError] = useState<{ match: number; error: unknown } | null>(null);

  useEffect(() => {
    if (result.state === "ready") setView(result.data);
  }, [result]);

  useEffect(() => {
    const id = window.setInterval(reload, REFRESH_MS);
    return () => window.clearInterval(id);
  }, [reload]);

  async function onPredict(match: number, team: TeamRef) {
    setSaving(match);
    setError(null);
    try {
      await predict(league, match, team.id);
      setView((v) => v && withPrediction(v, match, team.id));
    } catch (e) {
      setError({ match, error: e });
      reload();
    } finally {
      setSaving(null);
    }
  }

  const back = (
    <Link to={`/leagues/${league}`} className="text-sm text-mute hover:text-fg">
      ← {t("backToLeague")}
    </Link>
  );

  if (view === null) {
    if (result.state === "error") {
      return (
        <div className="flex flex-col gap-4 py-10">
          {back}
          <p className="text-danger">{errorMessage(t, result.error)}</p>
        </div>
      );
    }
    return <p className="py-10 text-mute">{t("loading")}</p>;
  }

  return (
    <div className="flex flex-col gap-8 py-10">
      <header className="flex flex-col gap-1">
        {back}
        <h1 className="text-2xl font-extrabold text-gold-gradient">{t("title")}</h1>
        <p className="text-mute">{view.event.name}</p>
      </header>

      {view.stages.length === 0 && <p className="card p-6 text-mute">{t("empty")}</p>}

      {view.stages.map((stage) => (
        <section key={stage.ordinal} className="flex flex-col gap-4">
          <h2 className="text-lg font-bold text-gold-gradient">{stage.name}</h2>
          {stage.groups.map((group) => (
            <div key={groupKey(group)} className="flex flex-col gap-3">
              <h3 className="text-sm font-semibold uppercase tracking-wide text-mute">
                {groupTitle(t, group)}
              </h3>
              <div className="grid gap-3 md:grid-cols-2">
                {group.matches.map((card) => (
                  <div key={card.id} className="flex flex-col gap-1">
                    <MatchCard
                      card={card}
                      now={now}
                      saving={saving === card.id}
                      onPredict={(team) => void onPredict(card.id, team)}
                    />
                    {error?.match === card.id && (
                      <p className="px-1 text-sm text-danger">{errorMessage(t, error.error)}</p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </section>
      ))}
    </div>
  );
}

function groupKey(group: MatchGroup): string {
  switch (group.kind) {
    case "record":
      return `r${group.wins}-${group.losses}`;
    case "playoff_round":
      return `p${group.rounds_before_final}`;
    case "unplaced":
      return "u";
  }
}

function groupTitle(t: TFunction, group: MatchGroup): string {
  switch (group.kind) {
    case "record":
      return t("group.record", { wins: group.wins, losses: group.losses });
    case "playoff_round":
      if (group.rounds_before_final === 0) return t("group.final");
      if (group.rounds_before_final === 1) return t("group.semifinals");
      if (group.rounds_before_final === 2) return t("group.quarterfinals");
      return t("group.roundOf", {
        teams: 2 ** (group.rounds_before_final + 1),
        fraction: 2 ** group.rounds_before_final,
      });
    case "unplaced":
      return t("group.matches");
  }
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
