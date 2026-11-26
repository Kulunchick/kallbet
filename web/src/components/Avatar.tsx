import { useState } from "react";

/** A User's Telegram avatar, or their initial when there is none (or it fails to load). */
export function Avatar({
  url,
  nickname,
  className = "size-9",
}: {
  url: string | null;
  nickname: string;
  className?: string;
}) {
  const [failed, setFailed] = useState(false);
  if (url && !failed) {
    return (
      <img
        src={url}
        alt=""
        referrerPolicy="no-referrer"
        onError={() => setFailed(true)}
        className={`${className} shrink-0 rounded-full border border-line object-cover`}
      />
    );
  }
  return (
    <span
      aria-hidden
      className={`${className} flex shrink-0 items-center justify-center rounded-full border border-line bg-raised font-display text-xs font-bold text-fg`}
    >
      {Array.from(nickname)[0]?.toUpperCase()}
    </span>
  );
}
