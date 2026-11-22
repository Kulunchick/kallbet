import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { ApiError, errorMessage } from "../../api/client";
import { Avatar } from "../../components/Avatar";
import { TeamLogo } from "../../components/TeamLogo";
import { StageStatus, useStagePickem } from "./StageStatus";
import { savePickem, type PickemView, type SwissBody } from "./api";

type Choice = "three_nil" | "zero_three" | "advance";
const CHOICES: Choice[] = ["three_nil", "advance", "zero_three"];
const LIMITS: Record<Choice, number> = { three_nil: 2, zero_three: 2, advance: 6 };

/** A Swiss Stage's Pick'em: the form until Lock, everyone's Pick'ems after it. */
export function PickemStage({
  league,
  stage,
  onSaved,
}: {
  league: number;
  stage: number;
  onSaved: () => void;
}) {
  const { t } = useTranslation("pickem");
  const { result, reload } = useStagePickem(league, stage, onSaved);
  const [draft, setDraft] = useState<Record<number, Choice>>({});
  const [error, setError] = useState<unknown>(null);
  const [saving, setSaving] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);

  const savedAt = result.state === "ready" ? result.data.my_pickem?.updated_at : undefined;

  useEffect(() => {
    if (result.state !== "ready") return;
    const mine = result.data.my_pickem;
    const next: Record<number, Choice> = {};
    if (mine?.shape === "swiss") for (const c of CHOICES) for (const team of mine[c]) next[team] = c;
    setDraft(next);
  }, [league, stage, savedAt]);

  if (result.state === "loading") return <p className="text-mute">{t("loading")}</p>;
  if (result.state === "error") return <p className="text-danger">{errorMessage(t, result.error)}</p>;
  const view = result.data;
  const count = (c: Choice) => Object.values(draft).filter((v) => v === c).length;
  const complete = CHOICES.every((c) => count(c) === LIMITS[c]);
  const nameOf = (id: number) => view.teams.find((x) => x.id === id)?.name ?? `#${id}`;

  function choose(team: number, choice: Choice) {
    if (saving || !view.teams.some((entry) => entry.id === team)) return;
    if (draft[team] === choice || count(choice) >= LIMITS[choice]) return;
    setDraft({ ...draft, [team]: choice });
    setSelected(null);
  }

  function remove(team: number) {
    const next = { ...draft };
    delete next[team];
    setDraft(next);
  }

  async function save() {
    if (!complete || saving) return;
    setSaving(true);
    const body = Object.fromEntries(
      CHOICES.map((c) => [c, view.teams.filter((x) => draft[x.id] === c).map((x) => x.id)]),
    ) as SwissBody;
    setError(null);
    try {
      await savePickem(league, stage, body);
      reload();
      onSaved();
    } catch (e) {
      setError(e);
      if (e instanceof ApiError && e.code === "pickem_locked") reload();
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <StageStatus view={view} />
      {!view.lock.locked && !view.lock.can_edit && view.my_pickem?.shape === "swiss" && (
        <div className="card flex flex-col gap-2 p-4">
          <h3 className="font-semibold">{t("waitingToReveal")}</h3>
          <SwissPicks picks={view.my_pickem} nameOf={nameOf} />
        </div>
      )}

      {view.lock.can_edit && (
        <div className="card flex flex-col gap-5 p-4 sm:p-6" aria-busy={saving}>
          <p className="text-sm text-mute">{t("boardHelp")}</p>
          <ul className="grid grid-cols-2 gap-2 sm:grid-cols-4 lg:grid-cols-8">
            {view.teams.map((team) => (
              <li key={team.id}>
                <button
                  type="button"
                  draggable={!saving}
                  disabled={saving}
                  onDragStart={(event) => event.dataTransfer.setData("text/plain", String(team.id))}
                  aria-pressed={selected === team.id}
                  onClick={() => setSelected(selected === team.id ? null : team.id)}
                  className={`flex h-full w-full flex-col items-center gap-2 rounded-lg border p-2 text-xs font-semibold ${
                    selected === team.id ? "border-gold-a bg-raised text-gold-a" : "border-line bg-ink hover:border-gold-a"
                  } ${draft[team.id] ? "opacity-65" : ""}`}
                >
                  <TeamLogo id={team.id} className="size-8" />
                  <span>{team.name}</span>
                </button>
              </li>
            ))}
          </ul>
          <div className="grid gap-3 md:grid-cols-[1fr_2fr_1fr]">
            {CHOICES.map((choice) => {
              const teams = view.teams.filter((team) => draft[team.id] === choice);
              const available = teams.length < LIMITS[choice];
              return (
                <section key={choice} className="rounded-xl border border-line bg-ink p-3"
                  onDragOver={(event) => { if (available && !saving) event.preventDefault(); }}
                  onDrop={(event) => {
                    event.preventDefault();
                    choose(Number(event.dataTransfer.getData("text/plain")), choice);
                  }}
                >
                  <h3 className="mb-3 flex justify-between gap-2 text-sm font-bold text-gold-a">
                    {t(`kind.${choice}`)}
                    <span className="text-mute">{teams.length}/{LIMITS[choice]}</span>
                  </h3>
                  <div className={`grid gap-2 ${choice === "advance" ? "grid-cols-3" : "grid-cols-2"}`}>
                    {Array.from({ length: LIMITS[choice] }, (_, index) => {
                      const team = teams[index];
                      return team ? (
                        <button key={team.id} type="button" disabled={saving}
                          aria-label={t("removeTeam", { team: team.name })}
                          onClick={() => remove(team.id)}
                          className="flex min-h-16 flex-col items-center justify-center gap-1 rounded-lg border border-line bg-raised p-1 text-xs hover:border-danger">
                          <TeamLogo id={team.id} className="size-7" />
                          <span className="max-w-full truncate">{team.name}</span>
                        </button>
                      ) : (
                        <button key={`empty-${index}`} type="button"
                          aria-label={t("fillSlot", { group: t(`kind.${choice}`), number: index + 1 })}
                          disabled={selected === null || saving}
                          onClick={() => selected !== null && choose(selected, choice)}
                          className="min-h-16 rounded-lg border border-dashed border-line text-lg text-mute enabled:border-gold-a enabled:text-gold-a">+</button>
                      );
                    })}
                  </div>
                </section>
              );
            })}
          </div>
          {error != null && <p className="text-danger">{errorMessage(t, error)}</p>}
          <button type="button" className="btn-primary self-start" disabled={!complete || saving} onClick={save}>
            {saving ? t("common:saving") : t(view.my_pickem ? "saveChanges" : "save")}
          </button>
        </div>
      )}

      {view.lock.locked && <LockedView view={view} nameOf={nameOf} />}
    </div>
  );
}

function LockedView({ view, nameOf }: { view: PickemView; nameOf: (id: number) => string }) {
  const { t } = useTranslation("pickem");
  if (!view.pickems || view.pickems.length === 0) return <p className="text-mute">{t("nobody")}</p>;
  return (
    <ul className="flex flex-col gap-3">
      {view.pickems.map((p) => (
        <li key={p.user_id} className="card flex flex-col gap-2 p-4">
          <div className="flex items-center gap-3">
            <Avatar url={p.avatar_url} nickname={p.nickname} className="size-8" />
            <span className="font-semibold">{p.nickname}</span>
            <span className="ml-auto font-display text-gold-a">{t("points", { count: p.points })}</span>
          </div>
          {p.shape === "swiss" && (
            <SwissPicks picks={p} nameOf={nameOf} />
          )}
        </li>
      ))}
    </ul>
  );
}

function SwissPicks({ picks, nameOf }: { picks: SwissBody; nameOf: (id: number) => string }) {
  const { t } = useTranslation("pickem");
  return (
    <div className="grid gap-2 md:grid-cols-[1fr_2fr_1fr]">
      {CHOICES.map((choice) => (
        <section key={choice} className="rounded-lg border border-line bg-ink p-3">
          <h3 className="mb-2 text-sm font-bold text-gold-a">{t(`kind.${choice}`)}</h3>
          <ul className="flex flex-wrap gap-2">
            {picks[choice].map((team) => (
              <li key={team} className="flex items-center gap-1.5 rounded bg-raised px-2 py-1 text-xs">
                <TeamLogo id={team} className="size-5" />{nameOf(team)}
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
