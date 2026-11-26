import { useRef, type KeyboardEvent, type ReactNode } from "react";

export type TabItem<Id extends string | number> = {
  id: Id;
  label: ReactNode;
  /** A second line under the label (a round's progress, "3/4"). */
  hint?: ReactNode;
};

/**
 * A segmented control that behaves as tabs: one selected segment
 * (`aria-selected`), arrow keys, Home and End move between them. The selected
 * segment is `bg-emerald-tint text-emerald-b`. The caller renders the panel and
 * gives it `role="tabpanel"` with `aria-labelledby={tabId(prefix, id)}`.
 */
export function SegmentedTabs<Id extends string | number>({
  items,
  value,
  onChange,
  label,
  idPrefix,
  height,
  panelId,
}: {
  items: TabItem<Id>[];
  value: Id;
  onChange: (id: Id) => void;
  label: string;
  idPrefix: string;
  height: "min-h-11" | "min-h-[52px]";
  panelId: string;
}) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);

  const onKeyDown = (event: KeyboardEvent, index: number) => {
    const last = items.length - 1;
    const target =
      event.key === "ArrowRight" ? (index === last ? 0 : index + 1)
      : event.key === "ArrowLeft" ? (index === 0 ? last : index - 1)
      : event.key === "Home" ? 0
      : event.key === "End" ? last
      : null;
    if (target === null) return;
    event.preventDefault();
    onChange(items[target].id);
    refs.current[target]?.focus();
  };

  return (
    <div
      role="tablist"
      aria-label={label}
      className="flex gap-1 rounded-2xl border border-line bg-panel p-1"
    >
      {items.map((item, index) => {
        const selected = item.id === value;
        return (
          <button
            key={item.id}
            ref={(el) => {
              refs.current[index] = el;
            }}
            type="button"
            role="tab"
            id={tabId(idPrefix, item.id)}
            aria-selected={selected}
            aria-controls={panelId}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(item.id)}
            onKeyDown={(event) => onKeyDown(event, index)}
            className={`${height} flex min-w-0 flex-1 flex-col items-center justify-center rounded-xl px-1 py-1 text-[13px] font-semibold leading-tight transition-colors duration-150 ${
              selected ? "bg-emerald-tint text-emerald-b" : "text-mute [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
            }`}
          >
            <span className="max-w-full text-center">{item.label}</span>
            {item.hint !== undefined && (
              <span className={`text-xs font-normal ${selected ? "text-emerald-b" : "text-mute"}`}>{item.hint}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export const tabId = (prefix: string, id: string | number) => `${prefix}-tab-${id}`;
