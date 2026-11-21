import i18n from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import { initReactI18next } from "react-i18next";

export const LANGUAGES = ["en", "ru", "uk"] as const;
export type Language = (typeof LANGUAGES)[number];

/** Where the chosen language is remembered between visits. */
export const LANGUAGE_STORAGE_KEY = "kallbet.language";

// Locales live in src/locales/<lang>/<namespace>.json, one namespace per
// feature area. Every file is picked up here automatically; `npm run
// check:locales` verifies the three languages have identical keys.
const files = import.meta.glob<Record<string, unknown>>("../locales/*/*.json", {
  eager: true,
  import: "default",
});

const resources: Record<string, Record<string, Record<string, unknown>>> = {};
for (const [path, content] of Object.entries(files)) {
  const [, lang, namespace] = path.match(/locales\/([^/]+)\/([^/]+)\.json$/)!;
  (resources[lang] ??= {})[namespace] = content;
}

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    supportedLngs: LANGUAGES,
    nonExplicitSupportedLngs: true,
    fallbackLng: "en",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: LANGUAGE_STORAGE_KEY,
      caches: ["localStorage"],
    },
  });

function syncDocumentLanguage(language: string) {
  document.documentElement.lang = language;
}
syncDocumentLanguage(i18n.resolvedLanguage ?? "en");
i18n.on("languageChanged", () => syncDocumentLanguage(i18n.resolvedLanguage ?? "en"));

export default i18n;
