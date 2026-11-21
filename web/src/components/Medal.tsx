import { useTranslation } from "react-i18next";

const MEDALS: Record<number, string> = { 1: "🥇", 2: "🥈", 3: "🥉" };

/**
 * The medal of a place on an archived League's final Leaderboard: places 1, 2
 * and 3 only. Shared places share a medal (1, 1, 3 gives two golds and a
 * bronze) because the server already numbers them that way.
 */
export function Medal({ place }: { place: number }) {
  const { t } = useTranslation("common");
  const medal = MEDALS[place];
  if (!medal) return null;
  return (
    <span role="img" aria-label={t(`medal.${place}`)} title={t(`medal.${place}`)}>
      {medal}
    </span>
  );
}
