import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";

import { errorMessage } from "../../api/client";
import { createLeague, listEvents, resolveEvent, type EventInfo } from "./api";
import { useLoad } from "./useLoad";

export function NewLeaguePage() {
  const { t, i18n } = useTranslation("leagues");
  const navigate = useNavigate();
  const events = useLoad(listEvents, []);
  const [picked, setPicked] = useState<EventInfo | null>(null);
  const [link, setLink] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  const date = (iso: string | null) =>
    iso ? new Date(iso).toLocaleDateString(i18n.language, { day: "numeric", month: "short" }) : "";

  async function useLink() {
    setError(null);
    setBusy(true);
    try {
      setPicked(await resolveEvent(link));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (!picked) return;
    setError(null);
    setBusy(true);
    try {
      const league = await createLeague(picked.id, name);
      navigate(`/leagues/${league.id}`);
    } catch (err) {
      setError(err);
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="flex max-w-2xl flex-col gap-8 py-10">
      <h1 className="text-2xl font-extrabold text-gold-gradient">{t("new.title")}</h1>

      <section className="flex flex-col gap-3">
        <h2 className="font-bold">{t("new.pickEvent")}</h2>
        {events.result.state === "loading" && <p className="text-mute">{t("loading")}</p>}
        {events.result.state === "error" && (
          <p className="text-danger">{errorMessage(t, events.result.error)}</p>
        )}
        {events.result.state === "ready" && events.result.data.length === 0 && (
          <p className="text-mute">{t("new.noEvents")}</p>
        )}
        {events.result.state === "ready" && (
          <ul className="flex flex-col gap-2">
            {events.result.data.map((event) => (
              <li key={event.id}>
                <button
                  type="button"
                  onClick={() => setPicked(event)}
                  aria-pressed={picked?.id === event.id}
                  className={`card flex w-full items-center justify-between gap-3 px-4 py-3 text-left hover:bg-raised ${
                    picked?.id === event.id ? "glow-emerald border-emerald-a" : ""
                  }`}
                >
                  <span className="font-semibold">{event.name}</span>
                  <span className="text-sm text-mute">
                    {date(event.starts_at)}
                    {event.ends_at ? ` – ${date(event.ends_at)}` : ""}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="flex flex-col gap-3">
        <label htmlFor="hltv-link" className="font-bold">
          {t("new.orLink")}
        </label>
        <div className="flex gap-2">
          <input
            id="hltv-link"
            value={link}
            onChange={(e) => setLink(e.target.value)}
            placeholder="https://www.hltv.org/events/…"
            className="min-w-0 flex-1 rounded-xl border border-line bg-raised px-4 py-2.5"
          />
          <button
            type="button"
            className="btn-ghost"
            disabled={busy || !link.trim()}
            onClick={useLink}
          >
            {t("new.useLink")}
          </button>
        </div>
      </section>

      {picked && (
        <section className="flex flex-col gap-3">
          <p className="text-sm text-mute">
            {t("new.selected")} <span className="font-semibold text-fg">{picked.name}</span>
          </p>
          <label htmlFor="league-name" className="font-bold">
            {t("new.name")}
          </label>
          <input
            id="league-name"
            value={name}
            maxLength={60}
            onChange={(e) => setName(e.target.value)}
            className="rounded-xl border border-line bg-raised px-4 py-2.5"
          />
        </section>
      )}

      {error !== null && <p className="text-danger">{errorMessage(t, error)}</p>}

      <div>
        <button type="submit" className="btn-primary" disabled={busy || !picked || !name.trim()}>
          {t("new.submit")}
        </button>
      </div>
    </form>
  );
}
