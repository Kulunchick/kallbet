import type { TFunction } from "i18next";
import { type CSSProperties, type ReactNode, useId, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { MatchCard as Card, MatchGroup, StageView } from "./api";
import {
  bracketColumns, collapsedStage, compareMatchOrder, earliestStart, MASONRY_ROW, masonrySpan, splitFinished,
} from "./layout";

type RenderCard = (card: Card) => ReactNode;

/**
 * One Stage. A later Stage that has not started is a single collapsed row; the
 * results of a Stage fold into a "Finished" row (the Playoffs bracket keeps all
 * its Matches, so its slots stay where they are). Expansion is local state:
 * the row stays put and the content opens below it.
 */
export function StageSection({
  stage, current, now, renderCard,
}: {
  stage: StageView;
  current: number | null;
  now: number;
  renderCard: RenderCard;
}) {
  const { t, i18n } = useTranslation("picks");
  const { t: tMatches } = useTranslation("matches");
  const [opened, setOpened] = useState(false);
  const bodyId = useId();
  const cards = stage.groups.flatMap((group) => group.matches);
  const collapsed = collapsedStage(stage, current) && !opened;

  if (collapsedStage(stage, current)) {
    const start = earliestStart(cards);
    const parts = [
      cards.length > 0 ? t("disclosure.matchCount", { count: cards.length }) : t("disclosure.noMatchesYet"),
      start !== null ? t("disclosure.startsOn", { day: formatDay(start, now, i18n.resolvedLanguage ?? "en") }) : null,
    ].filter((part) => part !== null);
    return (
      <section className="flex flex-col gap-3">
        <h2>
          <DisclosureRow
            expanded={!collapsed}
            controls={bodyId}
            onToggle={() => setOpened(!opened)}
            title={stage.name}
            summary={parts.join(" · ")}
            heading
          />
        </h2>
        {!collapsed && <div id={bodyId}><StageBody t={tMatches} stage={stage} renderCard={renderCard} /></div>}
      </section>
    );
  }

  return (
    <section className="flex flex-col gap-4">
      <h2 className="font-display text-lg font-extrabold text-fg">{stage.name}</h2>
      <StageBody t={tMatches} stage={stage} renderCard={renderCard} />
    </section>
  );
}

function StageBody({ t, stage, renderCard }: { t: TFunction; stage: StageView; renderCard: RenderCard }) {
  if (stage.kind === "playoffs") return <PlayoffsBracket t={t} stage={stage} renderCard={renderCard} />;
  return <GroupedMatches t={t} groups={stage.groups} renderCard={renderCard} />;
}

/** Open Matches by group, then the finished ones behind a "Finished" row. */
function GroupedMatches({ t, groups, renderCard }: { t: TFunction; groups: MatchGroup[]; renderCard: RenderCard }) {
  const { t: tPicks } = useTranslation("picks");
  const [expanded, setExpanded] = useState(false);
  const bodyId = useId();
  const { open, finished, finishedCount } = splitFinished(groups);
  return (
    <div className="flex flex-col gap-4">
      {open.map((group) => <GroupBlock key={groupKey(group)} t={t} group={group} renderCard={renderCard} />)}
      {finishedCount > 0 && (
        <div className="flex flex-col gap-4">
          <DisclosureRow
            expanded={expanded}
            controls={bodyId}
            onToggle={() => setExpanded(!expanded)}
            title={tPicks("disclosure.finished")}
            summary={tPicks("disclosure.matchCount", { count: finishedCount })}
          />
          {expanded && (
            <div id={bodyId} className="flex flex-col gap-4">
              {finished.map((group) => (
                <GroupBlock key={groupKey(group)} t={t} group={group} renderCard={renderCard} />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function GroupBlock({ t, group, renderCard }: { t: TFunction; group: MatchGroup; renderCard: RenderCard }) {
  return (
    <div className="flex flex-col gap-3">
      <h3 className="text-sm font-semibold uppercase tracking-wide text-mute">{groupTitle(t, group)}</h3>
      <MatchGrid cards={group.matches} renderCard={renderCard} />
    </div>
  );
}

/**
 * The cards of a section, in time order. Below `md` one column. From `md` a
 * packed two-column masonry: a CSS grid of 4 px rows with no row gap, where each
 * card spans the rows of its measured height plus the gap (`masonrySpan`).
 * Auto-placement puts each card in the first free spot after the previous one,
 * so the DOM order, the Tab order and what a screen reader reads all stay
 * chronological while the columns pack. The trailing gap under the last cards
 * is taken back by the negative bottom margin.
 */
function MatchGrid({ cards, renderCard }: { cards: Card[]; renderCard: RenderCard }) {
  return (
    <div
      className="grid grid-cols-1 items-start gap-3 md:-mb-3 md:auto-rows-[var(--masonry-row)] md:grid-cols-2 md:gap-y-0"
      style={{ "--masonry-row": `${MASONRY_ROW}px` } as CSSProperties}
    >
      {cards.map((card) => <MasonryItem key={card.id}>{renderCard(card)}</MasonryItem>)}
    </div>
  );
}

/**
 * One grid item that spans the rows its content needs. It is measured before
 * the first paint and again whenever its size changes (width, fonts, images, a
 * reasoning panel opened, the Reveal): the span is written straight to the
 * element in the same frame, so a card never paints over the one below it.
 */
function MasonryItem({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const apply = (height: number) => element.style.setProperty("--masonry-span", String(masonrySpan(height)));
    apply(element.getBoundingClientRect().height);
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const box = entry.borderBoxSize?.[0];
        apply(box ? box.blockSize : entry.target.getBoundingClientRect().height);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return (
    <div ref={ref} className="min-w-0 md:[grid-row-end:span_var(--masonry-span,1)]">
      {children}
    </div>
  );
}

/** A 56 px row that opens its content below itself. */
function DisclosureRow({
  expanded, controls, onToggle, title, summary, heading = false,
}: {
  expanded: boolean;
  controls: string;
  onToggle: () => void;
  title: string;
  summary: string;
  heading?: boolean;
}) {
  return (
    <button
      type="button"
      aria-expanded={expanded}
      aria-controls={expanded ? controls : undefined}
      onClick={onToggle}
      className="card flex min-h-14 w-full items-center justify-between gap-3 px-4 py-2 text-left [@media(hover:hover)_and_(pointer:fine)]:hover:bg-raised"
    >
      <span className="flex min-w-0 flex-col">
        <span className={heading ? "font-display text-base font-extrabold text-fg" : "text-base font-bold text-fg"}>
          {title}
        </span>
        <span className="text-[13px] text-mute">{summary}</span>
      </span>
      <svg
        aria-hidden="true"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={`size-5 shrink-0 text-mute ${expanded ? "rotate-90" : ""}`}
      >
        <path d="M9 6l6 6-6 6" />
      </svg>
    </button>
  );
}

/**
 * The Playoffs as a bracket (story 29): one column per round, first round
 * first, each slot level with the two it is fed by. Wide brackets scroll
 * sideways inside their box; the page itself never does.
 */
function PlayoffsBracket({ t, stage, renderCard }: { t: TFunction; stage: StageView; renderCard: RenderCard }) {
  const rounds = stage.groups.flatMap((g) => (g.kind === "playoff_round" ? [g] : []));
  const { columns, extra } = bracketColumns(rounds);
  const others = [...extra, ...stage.groups.flatMap((g) => (g.kind === "playoff_round" ? [] : g.matches))];
  return (
    <div className="flex flex-col gap-4">
      {columns.length > 0 && (
        <div className="overflow-x-auto pb-2">
          <div className="flex min-w-max gap-4">
            {columns.map((column) => (
              <div key={column.round} className="flex w-72 flex-col gap-3 sm:w-80">
                <h3 className="text-center text-sm font-semibold uppercase tracking-wide text-mute">
                  {groupTitle(t, { kind: "playoff_round", rounds_before_final: column.round, matches: [] })}
                </h3>
                <div className="flex flex-1 flex-col justify-around gap-3">
                  {column.cells.map((card, slot) =>
                    card ? (
                      renderCard(card)
                    ) : (
                      <div
                        key={`empty-${slot}`}
                        className="rounded-2xl border border-dashed border-line p-4 text-center text-sm text-mute"
                      >
                        {t("bracket.emptySlot")}
                      </div>
                    ),
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
      {others.length > 0 && (
        <div className="flex flex-col gap-3">
          <h3 className="text-sm font-semibold uppercase tracking-wide text-mute">{t("bracket.other")}</h3>
          <MatchGrid cards={others.sort(compareMatchOrder)} renderCard={renderCard} />
        </div>
      )}
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

/** "Mon" within the coming week, else "Mon, 12 Jan". */
function formatDay(iso: string, now: number, language: string): string {
  const days = (Date.parse(iso) - now) / 86_400_000;
  return new Intl.DateTimeFormat(
    language,
    days < 6 ? { weekday: "short" } : { weekday: "short", day: "numeric", month: "short" },
  ).format(new Date(iso));
}
