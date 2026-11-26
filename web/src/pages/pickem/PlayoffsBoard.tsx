import { useId, useState, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";

import { TeamLogo } from "../../components/TeamLogo";
import {
  feederNumber, firstIncompleteRound, gridPlacement, matchNumbers, participants, roundProgress, roundsOf,
  slotKey, slotsOfRound, verdict, type Slot, type Winners,
} from "./bracket";
import { Check, Cross } from "./icons";
import { SegmentedTabs } from "./SegmentedTabs";
import { MD, useMedia } from "./useMedia";

type NameOf = (id: number | null) => string;

/** The name of a round: its distance from the final decides. */
function useRoundName() {
  const { t } = useTranslation("pickem");
  return (round: number) =>
    round === 0 ? t("round.final")
    : round === 1 ? t("round.semifinal")
    : round === 2 ? t("round.quarterfinal")
    : t("round.roundOf", { teams: 2 ** (round + 1) });
}

/** What an empty slot says: the Match that will fill it, or "To be decided". */
function usePlaceholder(slots: Slot[]) {
  const { t } = useTranslation("pickem");
  return (slot: Slot, side: 0 | 1) => {
    const number = feederNumber(slots, slot, side);
    return number === null ? t("tbd") : t("winnerOf", { number });
  };
}

type RowProps = {
  team: number | null;
  placeholder: string;
  name: string;
  picked: boolean;
  slotWinner: number | null;
  /** The pick can be changed (a button). */
  interactive: boolean;
  disabled: boolean;
  compact: boolean;
  onClick: () => void;
};

function TeamRow({ team, placeholder, name, picked, slotWinner, interactive, disabled, compact, onClick }: RowProps) {
  const { t } = useTranslation("pickem");
  const height = compact ? "min-h-9" : "min-h-[52px]";

  if (team === null) {
    return (
      <div
        className={`${height} flex w-full items-center rounded-xl border-2 border-dashed border-line px-3 text-sm text-mute`}
      >
        {placeholder}
      </div>
    );
  }

  const result = picked ? verdict(slotWinner, team) : null;
  const tone = picked
    ? result === "wrong"
      ? "border-danger bg-raised text-danger"
      : "border-emerald-a bg-emerald-tint text-emerald-b"
    : "border-transparent bg-raised text-fg";
  const content = (
    <>
      <TeamLogo id={team} className={compact ? "size-5" : "size-7"} />
      <span className={`min-w-0 flex-1 truncate ${slotWinner === team ? "font-bold" : "font-semibold"}`}>{name}</span>
      {slotWinner === team && <span className="sr-only">{t("winnerMark")}</span>}
      {picked && (result === "wrong" ? <Cross className="size-5 shrink-0" /> : <Check className="size-5 shrink-0" />)}
      {picked && (
        <span className="sr-only">{result === "correct" ? t("correct") : result === "wrong" ? t("wrong") : t("pickedMark")}</span>
      )}
    </>
  );
  const classes = `${height} flex w-full min-w-0 items-center gap-3 rounded-xl border-2 px-3 text-left text-[15px] ${tone}`;

  return interactive ? (
    <button type="button" disabled={disabled} aria-pressed={picked} onClick={onClick} className={classes}>
      {content}
    </button>
  ) : (
    <div className={classes}>{content}</div>
  );
}

function MatchCard({
  slot,
  slots,
  winners,
  nameOf,
  onPick,
  disabled,
  compact = false,
  style,
}: {
  slot: Slot;
  slots: Slot[];
  winners: Winners;
  nameOf: NameOf;
  onPick?: (round: number, slot: number, team: number) => void;
  disabled?: boolean;
  compact?: boolean;
  style?: CSSProperties;
}) {
  const { t } = useTranslation("pickem");
  const placeholder = usePlaceholder(slots);
  const number = matchNumbers(slots)[slotKey(slot.rounds_before_final, slot.slot)];
  const picked = winners[slotKey(slot.rounds_before_final, slot.slot)];
  const sides = participants(slot, winners);
  return (
    <li
      style={style}
      className={`flex flex-col rounded-2xl border border-line bg-panel ${compact ? "gap-1.5 p-2" : "gap-2 p-3"}`}
    >
      <span className="text-[13px] text-mute">
        {t("matchLabel", { number })}
      </span>
      {sides.map((team, side) => (
        <TeamRow
          key={side}
          team={team}
          placeholder={placeholder(slot, side as 0 | 1)}
          name={nameOf(team)}
          picked={team !== null && picked === team}
          slotWinner={slot.winner}
          interactive={onPick !== undefined && team !== null}
          disabled={disabled ?? false}
          compact={compact}
          onClick={() => team !== null && onPick?.(slot.rounds_before_final, slot.slot, team)}
        />
      ))}
    </li>
  );
}

/**
 * The Playoffs bracket. Phones: a round tab control and one round at a time.
 * From `md`: every round as a column, later rounds centered between the Matches
 * that feed them. Without `onPick` it is read-only.
 */
export function PlayoffsBoard({
  slots,
  winners,
  nameOf,
  onPick,
  disabled = false,
}: {
  slots: Slot[];
  winners: Winners;
  nameOf: NameOf;
  onPick?: (round: number, slot: number, team: number) => void;
  disabled?: boolean;
}) {
  const desktop = useMedia(MD);
  return desktop ? (
    <BracketColumns slots={slots} winners={winners} nameOf={nameOf} onPick={onPick} disabled={disabled} />
  ) : (
    <RoundsOnePerTab slots={slots} winners={winners} nameOf={nameOf} onPick={onPick} disabled={disabled} />
  );
}

type BoardProps = {
  slots: Slot[];
  winners: Winners;
  nameOf: NameOf;
  onPick?: (round: number, slot: number, team: number) => void;
  disabled: boolean;
};

function RoundsOnePerTab({ slots, winners, nameOf, onPick, disabled }: BoardProps) {
  const { t } = useTranslation("pickem");
  const roundName = useRoundName();
  const rounds = roundsOf(slots);
  // The first round that still lacks a pick when the board opens; the player's choice after that.
  const [chosen, setChosen] = useState<number | null>(null);
  const [initial] = useState(() => firstIncompleteRound(slots, winners));
  const round = chosen !== null && rounds.includes(chosen) ? chosen : initial;
  const panel = useId();
  const prefix = useId();
  return (
    <div className="flex flex-col gap-3">
      <SegmentedTabs
        items={rounds.map((r) => {
          const { done, total } = roundProgress(slots, winners, r);
          return { id: r, label: roundName(r), hint: `${done}/${total}` };
        })}
        value={round}
        onChange={setChosen}
        label={t("rounds")}
        idPrefix={prefix}
        panelId={panel}
        height="min-h-[52px]"
      />
      <div role="tabpanel" id={panel} aria-labelledby={`${prefix}-tab-${round}`}>
        <ol className="flex flex-col gap-3">
          {slotsOfRound(slots, round).map((slot) => (
            <MatchCard key={slot.slot} slot={slot} slots={slots} winners={winners} nameOf={nameOf} onPick={onPick} disabled={disabled} />
          ))}
        </ol>
      </div>
    </div>
  );
}

function BracketColumns({ slots, winners, nameOf, onPick, disabled }: BoardProps) {
  const roundName = useRoundName();
  const rounds = roundsOf(slots);
  const first = rounds[0] ?? 0;
  const rows = slotsOfRound(slots, first).length;
  return (
    <div
      className="grid gap-x-5 gap-y-3"
      style={{
        gridTemplateColumns: `repeat(${rounds.length}, minmax(0, 1fr))`,
        gridTemplateRows: `auto repeat(${Math.max(rows, 1)}, auto)`,
      }}
    >
      {rounds.map((round, column) => {
        const { done, total } = roundProgress(slots, winners, round);
        return (
          <div key={`h-${round}`} style={{ gridColumn: column + 1, gridRow: 1 }} className="flex items-baseline justify-between gap-2">
            <h3 className="font-display text-base text-fg">{roundName(round)}</h3>
            <span className="text-sm text-mute">
              {done}/{total}
            </span>
          </div>
        );
      })}
      {rounds.flatMap((round, column) =>
        slotsOfRound(slots, round).map((slot) => {
          const { row, span } = gridPlacement(round, slot.slot, first);
          return (
            <ol key={`${round}:${slot.slot}`} className="contents">
              <MatchCard
                slot={slot}
                slots={slots}
                winners={winners}
                nameOf={nameOf}
                onPick={onPick}
                disabled={disabled}
                style={{ gridColumn: column + 1, gridRow: `${row + 1} / span ${span}`, alignSelf: "center" }}
              />
            </ol>
          );
        }),
      )}
    </div>
  );
}

/** A Member's bracket in the locked view: every round, each Match as a compact card. */
export function CompactBracket({ slots, winners, nameOf }: { slots: Slot[]; winners: Winners; nameOf: NameOf }) {
  const roundName = useRoundName();
  const rounds = roundsOf(slots);
  return (
    <div
      className="grid gap-4 md:grid-cols-[repeat(var(--rounds),minmax(0,1fr))]"
      style={{ "--rounds": rounds.length } as CSSProperties}
    >
      {rounds.map((round) => (
        <section key={round} aria-label={roundName(round)} className="flex flex-col gap-2">
          <h3 className="font-sans text-[13px] font-normal text-mute">{roundName(round)}</h3>
          <ol className="flex flex-col gap-2">
            {slotsOfRound(slots, round).map((slot) => (
              <MatchCard key={slot.slot} slot={slot} slots={slots} winners={winners} nameOf={nameOf} compact />
            ))}
          </ol>
        </section>
      ))}
    </div>
  );
}
