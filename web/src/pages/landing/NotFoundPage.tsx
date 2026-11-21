import { useTranslation } from "react-i18next";
import { Link } from "react-router";

export function NotFoundPage() {
  const { t } = useTranslation("landing");
  return (
    <section className="flex flex-col items-center gap-6 py-28 text-center">
      <p className="font-display text-7xl font-extrabold text-gold-gradient text-glow-gold">404</p>
      <h1 className="text-xl font-semibold">{t("notFound.title")}</h1>
      <Link to="/" className="btn-ghost">
        {t("notFound.home")}
      </Link>
    </section>
  );
}
