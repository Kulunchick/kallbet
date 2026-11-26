import { useTranslation } from "react-i18next";

import { Sheet } from "../../components/Sheet";

/** The three rules of a Lock, in a bottom sheet (a centered dialog from `md`). */
export function LockHelpSheet({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useTranslation("picks");
  return (
    <Sheet
      open={open}
      onClose={onClose}
      title={t("lockHelp.title")}
      footer={
        <button type="button" data-autofocus className="btn-primary w-full" onClick={onClose}>
          {t("lockHelp.gotIt")}
        </button>
      }
    >
      <ul className="flex flex-col gap-3 pb-2 text-base text-fg">
        {(["line1", "line2", "line3"] as const).map((line) => (
          <li key={line} className="flex gap-3">
            <span aria-hidden="true" className="mt-2 size-1.5 shrink-0 rounded-full bg-mute" />
            <span>{t(`lockHelp.${line}`)}</span>
          </li>
        ))}
      </ul>
    </Sheet>
  );
}
