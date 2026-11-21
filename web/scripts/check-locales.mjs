// Checks that the en, ru and uk locales have identical key sets.
//
//   npm run check:locales        (exits 1 and lists the problems if they differ)
//
// Locales live in src/locales/<lang>/<namespace>.json. Every namespace file
// must exist for every language and contain the same (nested) keys. Plural
// forms (`key_one`, `key_few`, `key_many`, `key_other`, ...) count as the one
// key `key`, because languages need different sets of them.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const LANGUAGES = ["en", "ru", "uk"];

const PLURAL_SUFFIX = /_(zero|one|two|few|many|other)$/;

/** Flattens nested JSON to `a.b.c` keys → values. */
function flatten(value, prefix, out) {
  for (const [key, child] of Object.entries(value)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (child !== null && typeof child === "object" && !Array.isArray(child)) {
      flatten(child, path, out);
    } else {
      out.set(path, child);
    }
  }
  return out;
}

/** Returns a sorted list of problems; empty means the locales agree. */
export function checkLocales(localesDir) {
  const problems = [];
  const present = LANGUAGES.filter((lang) => {
    const ok = existsSync(join(localesDir, lang));
    if (!ok) problems.push(`${lang}: missing locale directory`);
    return ok;
  });

  /** namespace → lang → Set of keys (plural forms merged) */
  const keysByNamespace = new Map();
  for (const lang of present) {
    for (const file of readdirSync(join(localesDir, lang))) {
      if (!file.endsWith(".json")) continue;
      const namespace = file.slice(0, -".json".length);
      let entries;
      try {
        entries = flatten(JSON.parse(readFileSync(join(localesDir, lang, file), "utf8")), "", new Map());
      } catch (error) {
        problems.push(`${lang}/${file}: invalid JSON (${error.message})`);
        continue;
      }
      const keys = new Set();
      for (const [key, value] of entries) {
        if (typeof value !== "string" || value.trim() === "") {
          problems.push(`${lang}/${file}: empty value for ${key}`);
        }
        keys.add(key.replace(PLURAL_SUFFIX, ""));
      }
      if (!keysByNamespace.has(namespace)) keysByNamespace.set(namespace, new Map());
      keysByNamespace.get(namespace).set(lang, keys);
    }
  }

  for (const [namespace, byLang] of keysByNamespace) {
    const allKeys = new Set([...byLang.values()].flatMap((keys) => [...keys]));
    for (const lang of present) {
      const keys = byLang.get(lang);
      if (!keys) {
        problems.push(`${lang}/${namespace}.json: missing file`);
        continue;
      }
      for (const key of allKeys) {
        if (!keys.has(key)) problems.push(`${lang}/${namespace}.json: missing key ${key}`);
      }
    }
  }

  return problems.sort();
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const localesDir = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "locales");
  const problems = checkLocales(localesDir);
  if (problems.length > 0) {
    console.error(`Locale check failed (${problems.length}):`);
    for (const problem of problems) console.error(`  ${problem}`);
    process.exit(1);
  }
  console.log(`Locale check passed: ${LANGUAGES.join(", ")} have identical key sets.`);
}
