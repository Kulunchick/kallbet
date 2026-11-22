import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "../../api/client";
import { Avatar } from "../../components/Avatar";
import { TeamLogo } from "../../components/TeamLogo";
import { useLoad } from "../../api/useLoad";
import { getPickem, savePickem, type PickemView } from "./api";
import { fromPicks, isComplete, participants, setWinner, slotKey, toPicks, type Slot, type Winners } from "./bracket";

/** A Playoffs Stage's Pick'em: the bracket form until Lock, everyone's brackets after it. */
export function PlayoffsStage({
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
  const [draft, setDraft] = useState<Winners>({});
  const [error, setError] = useState<unknown>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (result.state !== "ready") return;
    const mine = result.data.my_pickem;
    setDraft(mine?.shape === "playoffs" ? fromPicks(mine.picks) : {});
  }, [result]);

  if (result.state === "loading") return <p className="text-mute">{t("loading")}</p>;
  if (result.state === "error") return <p className="text-danger">{errorMessage(t, result.error)}</p>;
  const view = result.data;
  const slots = view.bracket?.slots ?? [];
  const nameOf = (id: number | null) =>
    id === null ? t("tbd") : (view.teams.find((x) => x.id === id)?.name ?? `#${id}`);

  async function save() {
    setError(null);
    try {
      await savePickem(league, stage, { picks: toPicks(slots, draft) });
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
          ? t("lockedBracket")
          : view.lock.locks_at
            ? t("locksAt", { time: new Date(view.lock.locks_at).toLocaleString() })
            : t("lockUnknown")}
      </p>

      {!view.lock.locked && !view.bracket?.ready && <p className="card p-5 text-mute">{t("bracketNotReady")}</p>}

      {!view.lock.locked && view.bracket?.ready && (
        <div className="card flex flex-col gap-4 p-5">
          <p className="text-sm text-mute">{t("bracketHelp")}</p>
          <Bracket
            slots={slots}
            winners={draft}
            nameOf={nameOf}
            onPick={(r, s, team) => {
              setSaved(false);
              setDraft((d) => setWinner(slots, d, r, s, team));
            }}
          />
          {error != null && <p className="text-danger">{errorMessage(t, error)}</p>}
          {saved && <p className="text-emerald-b">{t("saved")}</p>}
          <button
            type="button"
            className="btn-primary self-start"
            disabled={!isComplete(slots, draft)}
            onClick={save}
          >
            {t("save")}
          </button>
        </div>
      )}

      {view.lock.locked && <LockedView view={view} nameOf={nameOf} />}
    </div>
  );
}

/** Heading of a round: its distance from the final decides the name. */
function RoundName({ round }: { round: number }) {
  const { t } = useTranslation("pickem");
  if (round === 0) return <>{t("round.final")}</>;
  if (round === 1) return <>{t("round.semifinal")}</>;
  if (round === 2) return <>{t("round.quarterfinal")}</>;
  return <>{t("round.roundOf", { teams: 2 ** (round + 1) })}</>;
}

function Bracket({
  slots,
  winners,
  nameOf,
  onPick,
  readOnly = false,
}: {
  slots: Slot[];
  winners: Winners;
  nameOf: (id: number | null) => string;
  onPick?: (round: number, slot: number, team: number) => void;
  readOnly?: boolean;
}) {
  const rounds = [...new Set(slots.map((s) => s.rounds_before_final))].sort((a, b) => b - a);
  return (
    <div className="flex gap-6 overflow-x-auto pb-2">
      {rounds.map((round) => (
        <div key={round} className="flex min-w-44 flex-col justify-around gap-3">
          <h3 className="text-center text-xs font-bold uppercase tracking-wide text-gold-a">
            <RoundName round={round} />
          </h3>
          {slots
            .filter((s) => s.rounds_before_final === round)
            .sort((a, b) => a.slot - b.slot)
            .map((slot) => {
              const picked = winners[slotKey(round, slot.slot)];
              return (
                <div key={slot.slot} className="flex flex-col gap-1 rounded-xl border border-line bg-raised p-1.5">
                  {participants(slot, winners).map((team, i) => {
                    const mine = team !== null && picked === team;
                    const verdict = slot.winner !== null && mine ? (slot.winner === team ? "✓" : "✗") : null;
                    return (
                      <button
                        key={i}
                        type="button"
                        disabled={readOnly || team === null}
                        aria-pressed={mine}
                        onClick={() => team !== null && onPick?.(round, slot.slot, team)}
                        className={`flex items-center justify-between gap-2 rounded-lg px-2 py-1.5 text-left text-sm ${
                          mine ? "bg-emerald-gradient font-semibold text-ink glow-emerald" : "text-fg"
                        } ${team === null ? "text-mute" : ""}`}
                      >
                        <span className="flex min-w-0 items-center gap-2">
                          {team !== null && <TeamLogo id={team} className="size-5" />}
                          <span className="truncate">{nameOf(team)}</span>
                        </span>
                        {verdict && <span>{verdict}</span>}
                      </button>
                    );
                  })}
                </div>
              );
            })}
        </div>
      ))}
    </div>
  );
}

function LockedView({ view, nameOf }: { view: PickemView; nameOf: (id: number | null) => string }) {
  const { t } = useTranslation("pickem");
  if (!view.pickems || view.pickems.length === 0) return <p className="text-mute">{t("nobody")}</p>;
  const slots = view.bracket?.slots ?? [];
  return (
    <ul className="flex flex-col gap-3">
      {view.pickems.map((p) => (
        <li key={p.user_id} className="card flex flex-col gap-3 p-4">
          <div className="flex items-center gap-3">
            <Avatar url={p.avatar_url} nickname={p.nickname} className="size-8" />
            <span className="font-semibold">{p.nickname}</span>
            <span className="ml-auto font-display text-gold-a">{t("points", { count: p.points })}</span>
          </div>
          {p.shape === "playoffs" && <Bracket slots={slots} winners={fromPicks(p.picks)} nameOf={nameOf} readOnly />}
        </li>
      ))}
    </ul>
  );
}
