import { useTranslation } from "react-i18next";

import { Sheet } from "../../components/Sheet";
import { ChevronRight, Lock } from "./icons";
import { availability, type Choice, type Draft } from "./swiss";

/**
 * "Place {team}": one step to put a team in a group. A bottom sheet on phones,
 * a centered dialog on desktop (both the `Sheet`). A full group is disabled and
 * says how to free a slot. Choosing an option places the team and closes.
 */
export function TeamChooser({
  open,
  team,
  draft,
  onChoose,
  onClose,
}: {
  open: boolean;
  /** Kept while the sheet leaves, so its text does not change mid-exit. */
  team: { id: number; name: string } | null;
  draft: Draft;
  onChoose: (team: number, choice: Choice) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation("pickem");
  const { t: tUi } = useTranslation("ui");
  const options = availability(draft);
  const firstFree = options.find((o) => !o.full)?.choice;

  return (
    <Sheet
      open={open}
      onClose={onClose}
      title={t("placeTitle", { team: team?.name ?? "" })}
      footer={
        <button type="button" className="btn-ghost w-full" onClick={onClose} data-autofocus={firstFree ? undefined : ""}>
          {tUi("cancel")}
        </button>
      }
    >
      <ul className="flex flex-col gap-2 pb-1">
        {options.map((o) => {
          const group = t(`kind.${o.choice}`);
          return (
            <li key={o.choice}>
              <button
                type="button"
                disabled={o.full || team === null}
                data-autofocus={o.choice === firstFree ? "" : undefined}
                onClick={() => team && onChoose(team.id, o.choice)}
                className={`flex min-h-16 w-full items-center gap-3 rounded-2xl border px-4 py-2 text-left ${
                  o.full
                    ? "cursor-not-allowed border-dashed border-line text-mute"
                    : "border-line bg-raised text-fg [@media(hover:hover)_and_(pointer:fine)]:hover:border-emerald-a"
                }`}
              >
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className={`font-display text-base ${o.full ? "text-mute" : "text-fg"}`}>{group}</span>
                  <span className="text-[13px] text-mute">
                    {o.full ? t("optionFull", { group }) : t(`explain.${o.choice}`)}
                  </span>
                </span>
                <span className="flex shrink-0 items-center gap-1.5 text-sm font-semibold">
                  {t("optionCount", { n: o.count, limit: o.limit })}
                  {o.full ? <Lock className="size-5" /> : <ChevronRight className="size-5 text-mute" />}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </Sheet>
  );
}
