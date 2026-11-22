import { useState } from "react";

/** Team ids whose logo failed to load this session (a 404 for teams without one), so it is not asked again. */
const missing = new Set<number>();

/**
 * A team's logo, served by KallBet (`/api/teams/{id}/logo`, never the Data
 * Source's URL). Teams without one get nothing, so the name stands alone.
 */
export function TeamLogo({ id, className = "size-6" }: { id: number; className?: string }) {
  const [failed, setFailed] = useState(missing.has(id));
  if (failed) return null;
  return (
    <img
      src={`/api/teams/${id}/logo`}
      alt=""
      aria-hidden
      loading="lazy"
      onError={() => {
        missing.add(id);
        setFailed(true);
      }}
      className={`${className} shrink-0 object-contain`}
    />
  );
}
