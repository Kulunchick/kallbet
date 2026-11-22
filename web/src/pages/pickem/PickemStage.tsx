import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "../../api/client";
import { Avatar } from "../../components/Avatar";
import { TeamLogo } from "../../components/TeamLogo";
import { useLoad } from "../../api/useLoad";
import { getPickem, savePickem, type PickemView, type SwissBody } from "./api";

type Choice = "three_nil" | "zero_three" | "advance";
const CHOICES: Choice[] = ["three_nil", "zero_three", "advance"];
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
  const { result, reload } = useLoad(() => getPickem(league, stage), [league, stage]);
  const [draft, setDraft] = useState<Record<number, Choice>>({});
  const [error, setError] = useState<unknown>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (result.state !== "ready") return;
    const mine = result.data.my_pickem;
    const next: Record<number, Choice> = {};
    if (mine?.shape === "swiss") for (const c of CHOICES) for (const team of mine[c]) next[team] = c;
    setDraft(next);
  }, [result]);

  if (result.state === "loading") return <p className="text-mute">{t("loading")}</p>;
  if (result.state === "error") return <p className="text-danger">{errorMessage(t, result.error)}</p>;
  const view = result.data;
  const count = (c: Choice) => Object.values(draft).filter((v) => v === c).length;
  const complete = CHOICES.every((c) => count(c) === LIMITS[c]);
  const nameOf = (id: number) => view.teams.find((x) => x.id === id)?.name ?? `#${id}`;

  function choose(team: number, choice: Choice | "") {
    setSaved(false);
    setDraft((d) => {
      const next = { ...d };
      if (choice === "") delete next[team];
      else next[team] = choice;
      return next;
    });
  }

  async function save() {
    const body = Object.fromEntries(
      CHOICES.map((c) => [c, view.teams.filter((x) => draft[x.id] === c).map((x) => x.id)]),
    ) as SwissBody;
    setError(null);
    try {
      await savePickem(league, stage, body);
      setSaved(true);
      reload();
      onSaved();
    } catch (e) {
      setError(e);
      reload();
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-mute">
        {view.lock.locked
          ? t("locked")
          : view.lock.locks_at
            ? t("locksAt", { time: new Date(view.lock.locks_at).toLocaleString() })
            : t("lockUnknown")}
      </p>

      {!view.lock.locked && (
        <div className="card flex flex-col gap-4 p-5">
          <p className="text-sm text-mute">{t("help")}</p>
          <ul className="grid gap-2 sm:grid-cols-2">
            {view.teams.map((team) => (
              <li key={team.id} className="flex items-center justify-between gap-3">
                <span className="flex min-w-0 items-center gap-2">
                  <TeamLogo id={team.id} className="size-5" />
                  <span className="truncate">{team.name}</span>
                </span>
                <select
                  aria-label={team.name}
                  value={draft[team.id] ?? ""}
                  onChange={(e) => choose(team.id, e.target.value as Choice | "")}
                  className="rounded-lg border border-line bg-raised px-2 py-1 text-sm"
                >
                  <option value="">{t("none")}</option>
                  {CHOICES.map((c) => (
                    <option key={c} value={c}>
                      {t(`kind.${c}`)}
                    </option>
                  ))}
                </select>
              </li>
            ))}
          </ul>
          <p className="text-sm">
            {CHOICES.map((c) => `${t(`kind.${c}`)} ${count(c)}/${LIMITS[c]}`).join(" · ")}
          </p>
          {error != null && <p className="text-danger">{errorMessage(t, error)}</p>}
          {saved && <p className="text-emerald-b">{t("saved")}</p>}
          <button type="button" className="btn-primary self-start" disabled={!complete} onClick={save}>
            {t("save")}
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
          {p.shape === "swiss" &&
            CHOICES.map((c) => (
              <p key={c} className="text-sm">
                <span className="text-mute">{t(`kind.${c}`)}: </span>
                {p[c].map(nameOf).join(", ")}
              </p>
            ))}
        </li>
      ))}
    </ul>
  );
}
