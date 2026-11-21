/** A Member's Telegram avatar, or their initial when there is none. */
export function Avatar({
  nickname,
  url,
  className = "size-9",
}: {
  nickname: string;
  url: string | null;
  className?: string;
}) {
  if (url) {
    return <img src={url} alt="" className={`${className} shrink-0 rounded-full object-cover`} />;
  }
  return (
    <span
      aria-hidden
      className={`${className} flex shrink-0 items-center justify-center rounded-full bg-raised font-display text-xs font-bold text-gold-a`}
    >
      {nickname.slice(0, 1).toUpperCase()}
    </span>
  );
}
