// Pure helpers of the Pick'em page's Stage switcher (no React, no imports).

/** The ordinal a `#stage-N` hash names, when that Stage is playable. */
export function stageFromHash(hash: string, ordinals: number[]): number | null {
  const match = /^#stage-(\d+)$/.exec(hash);
  if (!match) return null;
  const ordinal = Number(match[1]);
  return ordinals.includes(ordinal) ? ordinal : null;
}

/** The Stage to show first: the first one open to edit, else the latest. */
export function defaultStage(stages: { ordinal: number; canEdit: boolean }[]): number | null {
  if (stages.length === 0) return null;
  const ordered = [...stages].sort((a, b) => a.ordinal - b.ordinal);
  return (ordered.find((s) => s.canEdit) ?? ordered[ordered.length - 1]).ordinal;
}
