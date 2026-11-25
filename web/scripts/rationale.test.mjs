// Tests of the Rationale panel's pure presentation logic (run by `npm test`).
import assert from "node:assert/strict";
import test from "node:test";

import {
  externalLink, factorLabelKey, rationalePicks, summaryFor, winPercent,
} from "../src/pages/matches/rationale.ts";

const all = { en: "In English.", ru: "По-русски.", uk: "Українською." };

test("the summary is shown in the active interface language", () => {
  assert.deepEqual(summaryFor(all, "ru"), { text: "По-русски.", language: "ru", fallback: false });
  assert.deepEqual(summaryFor(all, "uk"), { text: "Українською.", language: "uk", fallback: false });
  assert.deepEqual(summaryFor(all, "en"), { text: "In English.", language: "en", fallback: false });
});

test("a locale tag counts as its language", () => {
  assert.equal(summaryFor(all, "uk-UA").language, "uk");
  assert.equal(summaryFor(all, "RU").language, "ru");
});

test("a missing language falls back to English, then Russian, then Ukrainian", () => {
  assert.deepEqual(summaryFor({ en: "E", ru: "R" }, "uk"), { text: "E", language: "en", fallback: true });
  assert.deepEqual(summaryFor({ ru: "R", uk: "U" }, "en"), { text: "R", language: "ru", fallback: true });
  assert.deepEqual(summaryFor({ uk: "U" }, "en"), { text: "U", language: "uk", fallback: true });
  assert.deepEqual(summaryFor({ uk: "U" }, "de"), { text: "U", language: "uk", fallback: true });
});

test("blank summaries count as missing, and no summary at all is null", () => {
  assert.equal(summaryFor({ ru: "  ", en: "E" }, "ru").language, "en");
  assert.equal(summaryFor({}, "en"), null);
  assert.equal(summaryFor(undefined, "en"), null);
});

test("the win probability reads as a whole percentage", () => {
  assert.equal(winPercent(0.634), 63);
  assert.equal(winPercent(0.635), 64);
  assert.equal(winPercent(1), 100);
  assert.equal(winPercent(0), 0);
  assert.equal(winPercent(1.2), null, "out of range");
  assert.equal(winPercent(Number.NaN), null);
});

test("only absolute http(s) links are rendered as links", () => {
  assert.equal(externalLink("https://www.hltv.org/news/1"), "https://www.hltv.org/news/1");
  assert.equal(externalLink("http://liquipedia.net/x"), "http://liquipedia.net/x");
  assert.equal(externalLink("javascript:alert(1)"), null);
  assert.equal(externalLink("data:text/html,hi"), null);
  assert.equal(externalLink("//evil.example"), null);
  assert.equal(externalLink("/relative"), null);
  assert.equal(externalLink("not a url"), null);
});

test("a known factor has its own label, an unknown one a generic label", () => {
  assert.equal(factorLabelKey("stand_in"), "rationale.factor.stand_in");
  assert.equal(factorLabelKey("roster_change_30d"), "rationale.factor.roster_change_30d");
  assert.equal(factorLabelKey("technical_or_dq"), "rationale.factor.technical_or_dq");
  assert.equal(factorLabelKey("vibes"), "rationale.factor.other");
});

test("the panel lists revealed picks that carry a Rationale, Bot Users first", () => {
  const r = { p_win: 0.6, override_market: false, preliminary: false, summary: { en: "x" }, factors: [] };
  const picks = [
    { nickname: "alice", is_bot: false, rationale: r },
    { nickname: "bob", is_bot: false, rationale: null },
    { nickname: "Oracle", is_bot: true, rationale: r },
    { nickname: "Old", is_bot: true }, // a server from before Rationales
  ];
  assert.deepEqual(rationalePicks(picks).map((p) => p.nickname), ["Oracle", "alice"]);
  assert.deepEqual(rationalePicks(null), [], "nothing revealed yet");
});
