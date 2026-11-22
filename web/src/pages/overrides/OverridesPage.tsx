import { useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";

import { errorMessage } from "../../api/client";
import { getLeague } from "../leagues/api";
import { useLoad } from "../../api/useLoad";
import { getMatches } from "../matches/api";
import { formatStart } from "../matches/time";
import {
  addMatch,
  getOverrides,
  getWarnings,
  inForce,
  removeOverride,
  setOverride,
} from "./api";
import type { Entry, MatchRef, OverrideBody, OverridesView } from "./api";

/**
 * League-scoped Overrides (#13). Every Member sees the history (who changed
 * what, when); League Admins of a League that is not archived also get the
 * forms, the Overrides in force with a Remove button, and the Data Source
 * warnings of the Event.
 */
export function OverridesPage() {
  const { t } = useTranslation("overrides");
  const league = Number(useParams().id);
  const detail = useLoad(() => getLeague(league), [league]).result;
  const { result, reload } = useLoad(() => getOverrides(league), [league]);

  const back = (
    <Link to={`/leagues/${league}`} className="text-sm text-mute hover:text-fg">
      ← {t("backToLeague")}
    </Link>
  );
  if (result.state === "loading" || detail.state === "loading") {
    return <p className="py-10 text-mute">{t("loading")}</p>;
  }
  if (result.state === "error" || detail.state === "error") {
    const error = result.state === "error" ? result.error : (detail as { error: unknown }).error;
    return (
      <div className="flex flex-col gap-4 py-10">
        {back}
        <p className="text-danger">{errorMessage(t, error)}</p>
      </div>
    );
  }
  const admin = detail.data.my_role === "admin" && detail.data.archived_at === null;

  return (
    <div className="flex flex-col gap-6 py-10">
      <header className="flex flex-col gap-1">
        {back}
        <h1 className="text-2xl font-extrabold text-gold-gradient">{t("title")}</h1>
        <p className="text-sm text-mute">{t("intro")}</p>
      </header>
      {admin && <AdminTools league={league} data={result.data} onChange={reload} />}
      <History entries={result.data.entries} />
    </div>
  );
}

function matchLabel(t: (key: string) => string, m: MatchRef | null, id: number): string {
  if (!m) return `#${id}`;
  return `${m.team_a?.name ?? t("tbd")} – ${m.team_b?.name ?? t("tbd")}`;
}

function describe(t: ReturnType<typeof useTranslation>["t"], e: Entry, language: string): string {
  if (e.cleared) return t("describe.cleared", { field: t(`field.${e.field}`) });
  switch (e.field) {
    case "scheduled_start":
      return t("describe.scheduled_start", {
        time: e.scheduled_start ? formatStart(e.scheduled_start, language) : "?",
      });
    case "result":
      return e.void ? t("describe.void") : t("describe.winner", { team: e.winner?.name ?? "?" });
    case "teams":
      return t("describe.teams", { a: e.team_a?.name ?? "?", b: e.team_b?.name ?? "?" });
    case "hidden":
      return t("describe.hidden");
    case "added":
      return t("describe.added");
  }
}

function History({ entries }: { entries: Entry[] }) {
  const { t, i18n } = useTranslation("overrides");
  const language = i18n.resolvedLanguage ?? "en";
  return (
    <section className="card flex flex-col gap-3 p-4">
      <h2 className="font-display text-lg font-bold text-gold-gradient">{t("history")}</h2>
      {entries.length === 0 ? (
        <p className="text-sm text-mute">{t("noHistory")}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-line">
          {entries.map((e) => (
            <li key={e.id} className="flex flex-col gap-1 py-2 text-sm">
              <span className="font-semibold">
                {matchLabel(t, e.match, e.match_id)}: {describe(t, e, language)}
              </span>
              <span className="text-xs text-mute">
                {t("by", {
                  author: e.author?.nickname ?? t("unknownAuthor"),
                  time: formatStart(e.created_at, language),
                })}
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

type Action = "scheduled_start" | "winner" | "void" | "teams" | "hidden";

function AdminTools({
  league,
  data,
  onChange,
}: {
  league: number;
  data: OverridesView;
  onChange: () => void;
}) {
  const { t, i18n } = useTranslation("overrides");
  const language = i18n.resolvedLanguage ?? "en";
  const matches = useLoad(() => getMatches(league), [league, data]).result;
  const warnings = useLoad(() => getWarnings(league), [league]).result;
  const [error, setError] = useState<unknown>(null);

  const visible: MatchRef[] =
    matches.state === "ready"
      ? matches.data.stages.flatMap((s) =>
          s.groups.flatMap((g) =>
            g.matches.map((m) => ({
              id: m.id,
              stage: s.ordinal,
              team_a: m.team_a,
              team_b: m.team_b,
              scheduled_start: m.scheduled_start,
              status: m.status,
            })),
          ),
        )
      : [];
  const hiddenIds = new Set(data.hidden.map((m) => m.id));
  const all = [...visible.filter((m) => !hiddenIds.has(m.id)), ...data.hidden];

  const [pending, setPending] = useState<string | null>(null);

  const run = async (key: string, action: () => Promise<unknown>) => {
    if (pending !== null) return;
    setPending(key);
    setError(null);
    try {
      await action();
      onChange();
    } catch (e) {
      setError(e);
    } finally {
      setPending(null);
    }
  };

  return (
    <>
      {error !== null && (
        <p role="alert" className="text-danger">
          {errorMessage(t, error)}
        </p>
      )}
      <fieldset disabled={pending !== null} aria-busy={pending === "override"}>
        <OverrideForm
          matches={all}
          teams={data.teams}
          onSubmit={(match, body) => run("override", () => setOverride(league, match, body))}
        />
      </fieldset>
      <section className="card flex flex-col gap-3 p-4">
        <h2 className="font-display text-lg font-bold text-gold-gradient">{t("inForce")}</h2>
        {inForce(data.entries).length === 0 ? (
          <p className="text-sm text-mute">{t("noneInForce")}</p>
        ) : (
          <ul className="flex flex-col divide-y divide-line">
            {inForce(data.entries).map((e) => (
              <li key={e.id} className="flex items-center justify-between gap-3 py-2 text-sm">
                <span>
                  {matchLabel(t, e.match, e.match_id)}: {describe(t, e, language)}
                </span>
                <button
                  type="button"
                  className="btn-ghost"
                  disabled={pending !== null}
                  onClick={() => run(String(e.id), () => removeOverride(league, e.match_id, e.field))}
                >
                  {e.field === "hidden" ? t("unhide") : t("remove")}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      <fieldset disabled={pending !== null} aria-busy={pending === "add"}>
        <AddMatchForm data={data} onSubmit={(body) => run("add", () => addMatch(league, body))} />
      </fieldset>
      <section className="card flex flex-col gap-3 p-4">
        <h2 className="font-display text-lg font-bold text-gold-gradient">{t("warnings.title")}</h2>
        <p className="text-xs text-mute">{t("warnings.hint")}</p>
        {warnings.state === "ready" &&
          warnings.data.conflicts.length === 0 &&
          warnings.data.unmatched.length === 0 && (
            <p className="text-sm text-mute">{t("warnings.none")}</p>
          )}
        {warnings.state === "ready" && (
          <ul className="flex flex-col gap-2 text-sm">
            {warnings.data.conflicts.map((c) => (
              <li key={`c${c.match_id}${c.other_source}`} className="text-gold-a">
                {t("warnings.conflict", {
                  match: matchLabel(t, c.match, c.match_id),
                  kept: c.kept_source,
                  keptWinner: c.kept_winner?.name ?? t("void"),
                  other: c.other_source,
                  otherWinner: c.other_winner?.name ?? t("void"),
                })}
              </li>
            ))}
            {warnings.data.unmatched.map((u) => (
              <li key={`u${u.source}${u.kind}${u.external_id}`} className="text-mute">
                {t("warnings.unmatched", {
                  source: u.source,
                  kind: t(`warnings.kind.${u.kind}`),
                  description: u.description,
                })}
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}

function OverrideForm({
  matches,
  teams,
  onSubmit,
}: {
  matches: MatchRef[];
  teams: { id: number; name: string }[];
  onSubmit: (match: number, body: OverrideBody) => void;
}) {
  const { t } = useTranslation("overrides");
  const [matchId, setMatchId] = useState<number | null>(null);
  const [action, setAction] = useState<Action>("scheduled_start");
  const [start, setStart] = useState("");
  const [winner, setWinner] = useState<number | null>(null);
  const [teamA, setTeamA] = useState<number | null>(null);
  const [teamB, setTeamB] = useState<number | null>(null);
  const selected = matches.find((m) => m.id === matchId) ?? null;
  const players = [selected?.team_a, selected?.team_b].filter((x) => x != null);

  const body = (): OverrideBody | null => {
    switch (action) {
      case "scheduled_start":
        return start ? { field: "scheduled_start", scheduled_start: new Date(start).toISOString() } : null;
      case "winner":
        return winner !== null ? { field: "result", winner } : null;
      case "void":
        return { field: "result", void: true };
      case "teams":
        return teamA !== null && teamB !== null ? { field: "teams", team_a: teamA, team_b: teamB } : null;
      case "hidden":
        return { field: "hidden" };
    }
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const b = body();
    if (matchId !== null && b) onSubmit(matchId, b);
  };

  return (
    <form onSubmit={submit} className="card flex flex-col gap-3 p-4">
      <h2 className="font-display text-lg font-bold text-gold-gradient">{t("form.title")}</h2>
      <label className="flex flex-col gap-1 text-sm">
        {t("form.match")}
        <select
          className="rounded-lg border border-line bg-raised px-3 py-2"
          value={matchId ?? ""}
          onChange={(e) => setMatchId(e.target.value ? Number(e.target.value) : null)}
        >
          <option value="">{t("form.choose")}</option>
          {matches.map((m) => (
            <option key={m.id} value={m.id}>
              {matchLabel(t, m, m.id)}
            </option>
          ))}
        </select>
      </label>
      <label className="flex flex-col gap-1 text-sm">
        {t("form.action")}
        <select
          className="rounded-lg border border-line bg-raised px-3 py-2"
          value={action}
          onChange={(e) => setAction(e.target.value as Action)}
        >
          {(["scheduled_start", "winner", "void", "teams", "hidden"] as const).map((a) => (
            <option key={a} value={a}>
              {t(`action.${a}`)}
            </option>
          ))}
        </select>
      </label>
      {action === "scheduled_start" && (
        <label className="flex flex-col gap-1 text-sm">
          {t("form.start")}
          <input
            type="datetime-local"
            className="rounded-lg border border-line bg-raised px-3 py-2"
            value={start}
            onChange={(e) => setStart(e.target.value)}
          />
        </label>
      )}
      {action === "winner" && (
        <TeamSelect label={t("form.winner")} teams={players} value={winner} onChange={setWinner} />
      )}
      {action === "teams" && (
        <>
          <p className="text-xs text-mute">{t("form.teamsHint")}</p>
          <TeamSelect label={t("form.teamA")} teams={teams} value={teamA} onChange={setTeamA} />
          <TeamSelect label={t("form.teamB")} teams={teams} value={teamB} onChange={setTeamB} />
        </>
      )}
      <button type="submit" className="btn-primary self-start" disabled={matchId === null || body() === null}>
        {t("form.save")}
      </button>
    </form>
  );
}

function TeamSelect({
  label,
  teams,
  value,
  onChange,
}: {
  label: string;
  teams: { id: number; name: string }[];
  value: number | null;
  onChange: (id: number | null) => void;
}) {
  const { t } = useTranslation("overrides");
  return (
    <label className="flex flex-col gap-1 text-sm">
      {label}
      <select
        className="rounded-lg border border-line bg-raised px-3 py-2"
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value ? Number(e.target.value) : null)}
      >
        <option value="">{t("form.choose")}</option>
        {teams.map((team) => (
          <option key={team.id} value={team.id}>
            {team.name}
          </option>
        ))}
      </select>
    </label>
  );
}

function AddMatchForm({
  data,
  onSubmit,
}: {
  data: OverridesView;
  onSubmit: (body: {
    stage: number;
    team_a: number;
    team_b: number;
    scheduled_start: string;
    format: "bo1" | "bo3" | "bo5";
  }) => void;
}) {
  const { t } = useTranslation("overrides");
  const [stage, setStage] = useState<number | null>(data.stages[0]?.ordinal ?? null);
  const [teamA, setTeamA] = useState<number | null>(null);
  const [teamB, setTeamB] = useState<number | null>(null);
  const [start, setStart] = useState("");
  const [format, setFormat] = useState<"bo1" | "bo3" | "bo5">("bo3");
  const ready = stage !== null && teamA !== null && teamB !== null && start !== "";

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!ready) return;
    onSubmit({
      stage: stage!,
      team_a: teamA!,
      team_b: teamB!,
      scheduled_start: new Date(start).toISOString(),
      format,
    });
  };

  return (
    <form onSubmit={submit} className="card flex flex-col gap-3 p-4">
      <h2 className="font-display text-lg font-bold text-gold-gradient">{t("add.title")}</h2>
      <label className="flex flex-col gap-1 text-sm">
        {t("add.stage")}
        <select
          className="rounded-lg border border-line bg-raised px-3 py-2"
          value={stage ?? ""}
          onChange={(e) => setStage(e.target.value ? Number(e.target.value) : null)}
        >
          {data.stages.map((s) => (
            <option key={s.ordinal} value={s.ordinal}>
              {s.name}
            </option>
          ))}
        </select>
      </label>
      <TeamSelect label={t("form.teamA")} teams={data.teams} value={teamA} onChange={setTeamA} />
      <TeamSelect label={t("form.teamB")} teams={data.teams} value={teamB} onChange={setTeamB} />
      <label className="flex flex-col gap-1 text-sm">
        {t("form.start")}
        <input
          type="datetime-local"
          className="rounded-lg border border-line bg-raised px-3 py-2"
          value={start}
          onChange={(e) => setStart(e.target.value)}
        />
      </label>
      <label className="flex flex-col gap-1 text-sm">
        {t("add.format")}
        <select
          className="rounded-lg border border-line bg-raised px-3 py-2"
          value={format}
          onChange={(e) => setFormat(e.target.value as "bo1" | "bo3" | "bo5")}
        >
          {(["bo1", "bo3", "bo5"] as const).map((f) => (
            <option key={f} value={f}>
              {t(`matches:format.${f}`)}
            </option>
          ))}
        </select>
      </label>
      <button type="submit" className="btn-primary self-start" disabled={!ready}>
        {t("add.save")}
      </button>
    </form>
  );
}
