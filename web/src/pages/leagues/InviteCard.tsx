import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { errorMessage } from "../../api/client";
import { inviteUrl, reissueInvite } from "./api";

type InviteCardProps = {
  leagueId: number;
  token: string;
  /** The creator is the only Member: the card is the page's lead, with the Share invite button as its primary action. */
  alone: boolean;
  /** Called after the old link was replaced; the page reloads to get the new token. */
  onReissued: () => void;
};

const copiedForMs = 3000;

/**
 * The Invite Link for League Admins. Alone in the League it is the first thing on
 * the page (one primary action: Share invite); with other Members it shrinks to a
 * quiet row so "Matches" is the primary action again. The caller renders it only
 * for an admin of a League that is not archived.
 */
export function InviteCard({ leagueId, token, alone, onReissued }: InviteCardProps) {
  const { t } = useTranslation("leagues");
  const url = inviteUrl(token);
  const [copied, setCopied] = useState(false);
  const [failed, setFailed] = useState<unknown>(null);
  const [reissuing, setReissuing] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  // A reissued link is a different link: forget that the old one was copied.
  useEffect(() => setCopied(false), [token]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setCopied(false), copiedForMs);
    } catch {
      setCopied(false);
    }
  }

  async function share() {
    if (typeof navigator.share === "function") {
      try {
        await navigator.share({ url });
        return;
      } catch (cause) {
        // Closing the share sheet is not a failure; anything else falls back to copying.
        if (cause instanceof DOMException && cause.name === "AbortError") return;
      }
    }
    await copy();
  }

  async function reissue() {
    if (reissuing) return;
    setReissuing(true);
    setFailed(null);
    try {
      await reissueInvite(leagueId);
      onReissued();
    } catch (cause) {
      setFailed(cause);
    } finally {
      setReissuing(false);
    }
  }

  const reissueButton = (
    <button
      type="button"
      className="min-h-11 self-start rounded-lg text-sm text-mute underline underline-offset-4 disabled:opacity-60 [@media(hover:hover)_and_(pointer:fine)]:hover:text-fg"
      disabled={reissuing}
      onClick={reissue}
    >
      {t(reissuing ? "invite.reissueWorking" : "invite.reissue")}
    </button>
  );
  // Announced to screen readers; sighted users see the Copy button's label change.
  const status = (
    <p role="status" className="sr-only">
      {copied ? t("invite.copied") : ""}
    </p>
  );
  const copyLabel = copied ? t("invite.copied") : t("invite.copy");
  const problem = failed !== null && (
    <p role="alert" className="text-sm text-danger">
      {errorMessage(t, failed)}
    </p>
  );

  if (alone) {
    return (
      <section
        aria-labelledby="invite-heading"
        className="flex flex-col gap-3 rounded-2xl border border-emerald-a/50 bg-emerald-tint p-4 sm:p-5"
      >
        <div className="flex flex-col gap-1">
          <h2 id="invite-heading" className="text-lg font-semibold leading-snug text-fg">
            {t("invite.onlyMember")}
          </h2>
          <p className="text-sm leading-6 text-mute">{t("invite.onlyMemberHint")}</p>
        </div>
        <input
          readOnly
          aria-label={t("invite.title")}
          value={url}
          onFocus={(e) => e.currentTarget.select()}
          className="min-h-12 w-full rounded-xl border border-line bg-ink px-4 text-base text-fg"
        />
        <div className="flex flex-col gap-2">
          <button type="button" className="btn-primary w-full" onClick={share}>
            {t("invite.share")}
          </button>
          <button type="button" className="btn-ghost w-full" onClick={copy}>
            {copyLabel}
          </button>
        </div>
        {status}
        {problem}
        <div className="flex flex-col">
          {reissueButton}
          <p className="text-xs leading-5 text-mute">{t("invite.reissueHint")}</p>
        </div>
      </section>
    );
  }

  return (
    <section aria-labelledby="invite-heading" className="card flex flex-col gap-3 p-4">
      <h2 id="invite-heading" className="text-base font-semibold text-fg">
        {t("invite.title")}
      </h2>
      <div className="flex flex-wrap gap-2">
        <button type="button" className="btn-ghost min-h-11 min-w-[9rem] flex-1 py-2" onClick={share}>
          {t("invite.share")}
        </button>
        <button type="button" className="btn-ghost min-h-11 min-w-[9rem] flex-1 py-2" onClick={copy}>
          {copyLabel}
        </button>
      </div>
      {status}
      {problem}
      <div className="flex flex-col">
        {reissueButton}
        <p className="text-xs leading-5 text-mute">{t("invite.reissueHint")}</p>
      </div>
    </section>
  );
}
