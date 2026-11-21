import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { checkLocales } from "./check-locales.mjs";

/** Writes `{ lang: { namespace: content } }` to a temp locales dir. */
function locales(tree) {
  const dir = mkdtempSync(join(tmpdir(), "kallbet-locales-"));
  for (const [lang, namespaces] of Object.entries(tree)) {
    mkdirSync(join(dir, lang));
    for (const [ns, content] of Object.entries(namespaces)) {
      writeFileSync(join(dir, lang, `${ns}.json`), JSON.stringify(content));
    }
  }
  return dir;
}

test("passes when en, ru and uk have identical key sets", () => {
  const dir = locales({
    en: { common: { title: "Hi", nav: { home: "Home" } } },
    ru: { common: { title: "Привет", nav: { home: "Главная" } } },
    uk: { common: { title: "Привіт", nav: { home: "Головна" } } },
  });
  assert.deepEqual(checkLocales(dir), []);
});

test("fails when a locale misses a key", () => {
  const dir = locales({
    en: { common: { title: "Hi", nav: { home: "Home" } } },
    ru: { common: { title: "Привет", nav: { home: "Главная" } } },
    uk: { common: { title: "Привіт", nav: {} } },
  });
  assert.deepEqual(checkLocales(dir), ["uk/common.json: missing key nav.home"]);
});

test("fails when a locale has a key the others lack", () => {
  const dir = locales({
    en: { common: { title: "Hi" } },
    ru: { common: { title: "Привет", extra: "Лишнее" } },
    uk: { common: { title: "Привіт" } },
  });
  assert.deepEqual(checkLocales(dir), [
    "en/common.json: missing key extra",
    "uk/common.json: missing key extra",
  ]);
});

test("fails when a locale misses a whole namespace file", () => {
  const dir = locales({
    en: { common: { title: "Hi" }, leagues: { create: "Create" } },
    ru: { common: { title: "Привет" }, leagues: { create: "Создать" } },
    uk: { common: { title: "Привіт" } },
  });
  assert.deepEqual(checkLocales(dir), ["uk/leagues.json: missing file"]);
});

test("fails when one of the three languages is absent", () => {
  const dir = locales({
    en: { common: { title: "Hi" } },
    ru: { common: { title: "Привет" } },
  });
  assert.deepEqual(checkLocales(dir), ["uk: missing locale directory"]);
});

test("plural forms of one key count as one key", () => {
  const dir = locales({
    en: { common: { points_one: "{{count}} Point", points_other: "{{count}} Points" } },
    ru: {
      common: {
        points_one: "{{count}} очко",
        points_few: "{{count}} очка",
        points_many: "{{count}} очков",
        points_other: "{{count}} очка",
      },
    },
    uk: {
      common: {
        points_one: "{{count}} очко",
        points_few: "{{count}} очки",
        points_many: "{{count}} очок",
        points_other: "{{count}} очка",
      },
    },
  });
  assert.deepEqual(checkLocales(dir), []);
});

test("fails on an empty translation", () => {
  const dir = locales({
    en: { common: { title: "Hi" } },
    ru: { common: { title: "" } },
    uk: { common: { title: "Привіт" } },
  });
  assert.deepEqual(checkLocales(dir), ["ru/common.json: empty value for title"]);
});
