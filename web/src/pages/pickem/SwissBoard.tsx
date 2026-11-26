import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { TeamLogo } from "../../components/TeamLogo";
import type { TeamRef } from "./api";
import { Check } from "./icons";
import { TeamChooser } from "./TeamChooser";
import { FINE_POINTER, useMedia } from "./useMedia";
import { CHOICES, LIMITS, inGroup, isFull, nextPoolFocus, unplaced, type Choice, type Draft } from "./swiss";

/** The grid of a group's slots: Advance has three columns on phones. */
const SLOT_COLUMNS: Record<Choice, string> = {
  three_nil: "grid-cols-2",
  advance: "grid-cols-3",
  zero_three: "grid-cols-2",
};

/** The three groups side by side from `md`, in the proportions 1 : 2 : 1. */
const GROUPS_LAYOUT = "grid gap-3 md:grid-cols-[1fr_2fr_1fr] md:items-start md:gap-4";

/**
 * The editable Swiss board: three group cards, the pool of unplaced teams and
 * the chooser that places one. Tapping a pool tile opens the chooser; a placed
 * team is a button that removes it; on a mouse a tile can also be dragged into
 * a group.
 */
export function SwissBoard({
  teams,
  draft,
  disabled,
  onPlace,
  onRemove,
}: {
  teams: TeamRef[];
  draft: Draft;
  disabled: boolean;
  onPlace: (team: number, choice: Choice) => void;
  onRemove: (team: number) => void;
}) {
  const { t } = useTranslation("pickem");
  const canDrag = useMedia(FINE_POINTER);
  const pool = unplaced(teams, draft);
  const [chooser, setChooser] = useState<{ team: TeamRef; open: boolean } | null>(null);
  const [dragging, setDragging] = useState<number | null>(null);
  const [target, setTarget] = useState<Choice | null>(null);
  const tiles = useRef(new Map<number, HTMLButtonElement>());
  const counters = useRef(new Map<Choice, HTMLElement>());
  const pendingFocus = useRef<{ tile: number } | { counter: Choice } | null>(null);

  // After a team is placed or removed the button that had focus is gone: put it
  // somewhere useful (runs after the chooser's own effects, so after it closed).
  useEffect(() => {
    const wanted = pendingFocus.current;
    if (!wanted) return;
    pendingFocus.current = null;
    if ("tile" in wanted) tiles.current.get(wanted.tile)?.focus();
    else counters.current.get(wanted.counter)?.focus();
  });

  function choose(team: number, choice: Choice) {
    if (disabled || !teams.some((entry) => entry.id === team) || draft[team] === choice || isFull(draft, choice)) return;
    const next = nextPoolFocus(pool.map((entry) => entry.id), team);
    pendingFocus.current = next === null ? { counter: choice } : { tile: next };
    onPlace(team, choice);
    setChooser((current) => (current ? { ...current, open: false } : current));
  }

  function remove(team: number) {
    pendingFocus.current = { tile: team };
    onRemove(team);
  }

  const endDrag = () => {
    setDragging(null);
    setTarget(null);
  };

  return (
    <div className="flex flex-col gap-4">
      <div className={GROUPS_LAYOUT}>
        {CHOICES.map((choice) => {
          const placed = inGroup(teams, draft, choice);
          const full = placed.length >= LIMITS[choice];
          const active = target === choice;
          const accepts = !full && !disabled;
          return (
            <section
              key={choice}
              aria-labelledby={`group-${choice}`}
              onDragOver={(event) => {
                if (dragging === null || !accepts) return;
                event.preventDefault();
                setTarget(choice);
              }}
              onDragLeave={(event) => {
                if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
                  setTarget((current) => (current === choice ? null : current));
                }
              }}
              onDrop={(event) => {
                event.preventDefault();
                const id = Number(event.dataTransfer.getData("text/plain"));
                endDrag();
                choose(id, choice);
              }}
              className={`flex flex-col gap-3 rounded-2xl border bg-panel p-4 md:bg-ink ${
                active
                  ? "border-emerald-a ring-1 ring-emerald-a"
                  : dragging !== null && accepts
                    ? "border-emerald-a/40"
                    : "border-line"
              }`}
            >
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <h3 id={`group-${choice}`} className="font-display text-base text-fg">
                    {t(`kind.${choice}`)}
                  </h3>
                  <p className="text-[13px] text-mute">{t(`explain.${choice}`)}</p>
                </div>
                <span
                  ref={(el) => {
                    if (el) counters.current.set(choice, el);
                    else counters.current.delete(choice);
                  }}
                  tabIndex={-1}
                  className={`inline-flex shrink-0 items-center gap-1 text-sm font-semibold ${full ? "text-emerald-b" : "text-mute"}`}
                >
                  {full && <Check className="size-4" />}
                  <span className="sr-only">{t("groupCount", { n: placed.length, limit: LIMITS[choice] })}</span>
                  <span aria-hidden>
                    {placed.length}/{LIMITS[choice]}
                  </span>
                </span>
              </div>
              <ul className={`grid gap-2 ${SLOT_COLUMNS[choice]}`}>
                {Array.from({ length: LIMITS[choice] }, (_, index) => {
                  const team = placed[index];
                  return (
                    <li key={team?.id ?? `empty-${index}`} className="min-w-0">
                      {team ? (
                        <button
                          type="button"
                          disabled={disabled}
                          aria-label={t("removeTeam", { team: team.name })}
                          onClick={() => remove(team.id)}
                          className="flex h-16 w-full min-w-0 flex-col items-center justify-center gap-1 rounded-xl border border-line bg-raised px-1 md:h-[68px] [@media(hover:hover)_and_(pointer:fine)]:hover:border-danger"
                        >
                          <TeamLogo id={team.id} className="size-6" />
                          <span className="max-w-full truncate text-[13px] text-fg">{team.name}</span>
                        </button>
                      ) : (
                        <div
                          aria-hidden
                          className="flex h-16 items-center justify-center rounded-xl border border-dashed border-line px-1 text-center text-[13px] leading-tight text-mute md:h-[68px]"
                        >
                          {active && index === placed.length ? (
                            <span className="font-semibold text-emerald-b">{t("dropHere")}</span>
                          ) : (
                            "+"
                          )}
                        </div>
                      )}
                    </li>
                  );
                })}
              </ul>
            </section>
          );
        })}
      </div>

      <section
        aria-labelledby="teams-pool"
        className="flex flex-col gap-3 rounded-2xl border border-line bg-panel p-4 md:border-0 md:bg-transparent md:p-0"
      >
        <div className="flex items-baseline justify-between gap-2">
          <h3 id="teams-pool" className="font-display text-base text-fg">
            {t("pool.title")}
          </h3>
          <span className="text-sm text-mute">{t("pool.left", { n: pool.length })}</span>
        </div>
        {pool.length === 0 ? (
          <p className="text-sm text-mute">{t("pool.empty")}</p>
        ) : (
          <ul className="grid grid-cols-2 gap-2 md:grid-cols-5">
            {pool.map((team) => (
              <li key={team.id} className="min-w-0">
                <button
                  type="button"
                  ref={(el) => {
                    if (el) tiles.current.set(team.id, el);
                    else tiles.current.delete(team.id);
                  }}
                  disabled={disabled}
                  draggable={canDrag && !disabled}
                  aria-haspopup="dialog"
                  onClick={() => setChooser({ team, open: true })}
                  onDragStart={(event) => {
                    event.dataTransfer.setData("text/plain", String(team.id));
                    event.dataTransfer.effectAllowed = "move";
                    setDragging(team.id);
                  }}
                  onDragEnd={endDrag}
                  className={`flex h-12 w-full min-w-0 items-center gap-2 rounded-xl border border-line bg-raised px-3 text-left text-sm font-semibold text-fg [@media(hover:hover)_and_(pointer:fine)]:hover:border-emerald-a ${
                    dragging === team.id ? "opacity-50" : ""
                  }`}
                >
                  <TeamLogo id={team.id} className="size-6" />
                  <span className="truncate">{team.name}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <TeamChooser
        open={chooser?.open ?? false}
        team={chooser?.team ?? null}
        draft={draft}
        onChoose={choose}
        onClose={() => setChooser((current) => (current ? { ...current, open: false } : current))}
      />
    </div>
  );
}

/** A saved Swiss Pick'em, read-only: the same group cards with chips instead of buttons. */
export function SwissGroupsReadOnly({
  picks,
  nameOf,
}: {
  picks: { three_nil: number[]; advance: number[]; zero_three: number[] };
  nameOf: (id: number) => string;
}) {
  const { t } = useTranslation("pickem");
  return (
    <div className={GROUPS_LAYOUT}>
      {CHOICES.map((choice) => (
        <section
          key={choice}
          aria-labelledby={`saved-${choice}`}
          className="flex flex-col gap-3 rounded-2xl border border-line bg-panel p-4 md:bg-ink"
        >
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <h3 id={`saved-${choice}`} className="font-display text-base text-fg">
                {t(`kind.${choice}`)}
              </h3>
              <p className="text-[13px] text-mute">{t(`explain.${choice}`)}</p>
            </div>
            <span className="shrink-0 text-sm font-semibold text-mute">
              {picks[choice].length}/{LIMITS[choice]}
            </span>
          </div>
          <ul className="flex flex-wrap gap-2">
            {picks[choice].map((team) => (
              <li
                key={team}
                className="inline-flex h-9 max-w-full items-center gap-2 rounded-full border border-line bg-raised px-3 text-sm text-fg"
              >
                <TeamLogo id={team} className="size-5" />
                <span className="truncate">{nameOf(team)}</span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
