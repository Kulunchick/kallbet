import gzip
import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.parse import parse_qs, urlparse

import collect_research as research


class ArchiveTest(unittest.TestCase):
    def test_rate_limit_names_source_and_continues_other_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = ["collector", "--archive", tmp, "--panda-cap", "0", "--until", "2024-08-01"]
            with (patch.object(research.sys, "argv", args),
                  patch.object(research, "collect_panda", side_effect=HTTPError("url", 429, "", {}, None)) as panda,
                  patch.object(research, "collect_valve", return_value=None) as valve,
                  patch.object(research, "collect_rosters", return_value=[]) as wiki,
                  redirect_stdout(StringIO()), redirect_stderr(StringIO()) as errors):
                self.assertEqual(research.main(), 2)
                self.assertIn("panda: HTTP 429", errors.getvalue())
                panda.assert_called_once()
                valve.assert_called_once()
                wiki.assert_called_once()

    # Finding 19 (review of 6eea194): the archive command also collects the
    # market history when given a plan, and reports its requests per source;
    # without one it says that no market history was collected.
    def test_the_archive_command_collects_the_market_history_from_its_plan(self):
        with tempfile.TemporaryDirectory() as tmp:
            plan = Path(tmp) / "plan.json"
            plan.write_text(json.dumps({"kalshi": [{"min_close": 1, "max_close": 2}]}))
            base = ["collector", "--archive", tmp, "--panda-cap", "0", "--until", "2024-08-01"]
            quiet = dict(return_value=None)
            for extra, market_called in ((["--market-plan", str(plan)], True), ([], False)):
                with (patch.object(research.sys, "argv", base + extra),
                      patch.object(research, "collect_panda", **quiet),
                      patch.object(research, "collect_valve", **quiet),
                      patch.object(research, "collect_rosters", return_value=[]),
                      patch.object(research, "collect_market",
                                   return_value=("kalshi requests: 3 this run", False)) as market,
                      redirect_stdout(StringIO()) as out, redirect_stderr(StringIO())):
                    self.assertEqual(research.main(), 0)
                self.assertEqual(market.called, market_called)
                if market_called:
                    self.assertEqual(market.call_args.args[1], plan)
                    self.assertIn("kalshi requests: 3 this run", out.getvalue())
                else:
                    self.assertIn("market: no --market-plan", out.getvalue())

    def test_postmatch_stage_collection_uses_one_resumable_liquipedia_request(self):
        body = (Path(__file__).resolve().parents[1] / "tests/fixtures/research"
                / "liquipedia-cologne-playoffs-postmatch.json").read_bytes()
        title = "Intel Extreme Masters/2026/Cologne/Playoffs"
        requests = []

        def fetch(source, request):
            self.assertEqual(source, "liquipedia")
            params = parse_qs(urlparse(request).query)
            self.assertEqual(params["action"], ["query"])
            self.assertEqual(params["prop"], ["revisions"])
            self.assertEqual(params["titles"], [title])
            requests.append(request)
            return body

        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            rows = [{"tournament_id": 20710, "tournament": {"end_at": "2026-06-21T18:06:13Z"}}]
            (path / "panda-20710-1.json.gz").write_bytes(gzip.compress(json.dumps(rows).encode()))
            a = research.Archive(path, {"liquipedia": 1}, fetch)
            for _ in range(2):
                self.assertEqual(research.collect_maps(a, research.date(2026, 6, 22), {20710: title}), [])
            self.assertEqual(len(requests), 1)
            self.assertEqual(gzip.decompress((path / "maps-20710.json.gz").read_bytes()), body)
            self.assertIn("maps-20710.json.gz", (path / "sources.tsv").read_text())
            a.db.close()

    def test_resume_spend_and_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            request = "https://example.org/page"
            def fetch(source, url):
                self.assertEqual(url, request)
                return b'{"page":1}'
            a = research.Archive(path, {"panda": 1}, fetch)
            self.assertEqual(a.read("panda", request, "catalog/page.json"), {"page": 1})
            a.db.close()
            b = research.Archive(path, {"panda": 1}, lambda *_: self.fail("duplicate request"))
            self.assertEqual(b.read("panda", request, "catalog/page.json"), {"page": 1})
            with self.assertRaises(research.Budget):
                b.read("panda", request + "2", "catalog/page2.json")
            self.assertIn(request, (path / "sources.tsv").read_text())
            self.assertEqual(gzip.decompress((path / "catalog/page.json.gz").read_bytes()), b'{"page":1}')
            b.db.close()

    def test_failed_request_charged_and_echoed_secret_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            a = research.Archive(Path(tmp), {"panda": 1}, lambda *_: b'"sample-token"', ["sample-token"])
            with self.assertRaisesRegex(ValueError, "credential"):
                a.read("panda", "https://example.org", "test.json")
            self.assertFalse((Path(tmp) / "test.json.gz").exists())
            with self.assertRaises(research.Budget):
                a.read("panda", "https://example.org", "test.json")
            a.db.close()

    def test_three_sources_emit_replay_inputs_without_online_access(self):
        until = research.FIRST
        fake_match = {"serie_id": 1, "league": {"name": "Test"},
                      "serie": {"full_name": "2024", "begin_at": "2024-08-01T12:00:00Z",
                                "end_at": "2024-08-01T18:00:00Z"}, "original_scheduled_at": "2024-08-01T12:00:00Z"}
        urls = []
        def fetch(source, url):
            urls.append(url)
            if "/tournaments/past?" in url:
                result = [{"tier": "s", "id": 7}, {"tier": "b", "id": 8}]
            elif "/tournaments/7/matches" in url:
                result = [fake_match]
            elif "/commits?" in url:
                result = [{"sha": "abc"}]
            elif "/trees/" in url:
                result = {"tree": [{"path": "live/2024/standings_global_2024_08_01.md", "url": "blob"}]}
            elif url == "blob":
                return b"Standings"
            elif "rvstart=2024-07-31T23%3A59%3A59Z" in url:
                result = {"query": {"pages": [{"title": "Test/2024", "revisions": []}]}}
            else:
                self.fail(url)
            return json.dumps(result).encode()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            a = research.Archive(path, research.DEFAULT_CAPS, fetch)
            for _ in range(2):
                research.collect_panda(a, until)
                research.collect_valve(a, until)
                research.collect_rosters(a, until, {"1": "Test/2024"})
            self.assertEqual(len(urls), 6)
            self.assertEqual(a.spent("panda"), 2)
            self.assertEqual(len(list(path.glob("roster-*.json.gz"))), 1)
            a.db.close()


class ExtensionTest(unittest.TestCase):
    def test_extension_window_and_tiers_use_their_own_catalog_names(self):
        urls = []

        def fetch(source, request):
            urls.append(request)
            params = parse_qs(urlparse(request).query)
            if "/tournaments/past?" in request:
                self.assertEqual(params["filter[tier]"], ["b,c"])
                self.assertTrue(params["range[end_at]"][0].startswith("2024-08-01T00:00:00Z,2024-08-31"))
                return json.dumps([{"tier": "b", "id": 5}, {"tier": "s", "id": 6}]).encode()
            self.assertIn("/tournaments/5/matches", request)
            return b"[]"

        with tempfile.TemporaryDirectory() as tmp:
            a = research.Archive(Path(tmp), research.DEFAULT_CAPS, fetch)
            research.collect_panda(a, research.date(2024, 8, 31), tiers=("b", "c"))
            self.assertEqual(len(urls), 2)
            self.assertTrue((Path(tmp) / "catalog/tournaments-b,c-2024-08-01-2024-08-31-1.json.gz").exists())
            a.db.close()

    def test_catalog_only_skips_each_events_matches(self):
        urls = []

        def fetch(source, request):
            urls.append(request)
            return json.dumps([{"tier": "b", "id": 5}]).encode()

        with tempfile.TemporaryDirectory() as tmp:
            a = research.Archive(Path(tmp), research.DEFAULT_CAPS, fetch)
            research.collect_panda(a, research.date(2024, 8, 31), tiers=("b",), catalog_only=True)
            self.assertEqual(len(urls), 1)
            a.db.close()

    def test_earlier_window_starts_at_the_requested_day(self):
        months = list(research.month_ranges(research.date(2023, 11, 5), research.date(2023, 9, 27)))
        self.assertEqual(months, [(research.date(2023, 9, 27), research.date(2023, 9, 30)),
                                  (research.date(2023, 10, 1), research.date(2023, 10, 31)),
                                  (research.date(2023, 11, 1), research.date(2023, 11, 5))])
        self.assertEqual(next(iter(research.month_ranges(research.date(2024, 8, 5)))),
                         (research.FIRST, research.date(2024, 8, 5)))

    def test_main_accepts_from_and_tiers_and_restricts_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = ["collector", "--archive", tmp, "--panda-cap", "0", "--from", "2023-09-27",
                    "--until", "2024-07-31", "--tiers", "s,a", "--sources", "panda"]
            with (patch.object(research.sys, "argv", args),
                  patch.object(research, "collect_panda", return_value=None) as panda,
                  patch.object(research, "collect_valve", return_value=None) as valve,
                  patch.object(research, "collect_rosters", return_value=[]) as wiki,
                  redirect_stdout(StringIO()), redirect_stderr(StringIO())):
                self.assertEqual(research.main(), 0)
            panda.assert_called_once()
            self.assertEqual(panda.call_args.args[1], research.date(2024, 7, 31))
            self.assertEqual(panda.call_args.kwargs, {"first": research.date(2023, 9, 27), "tiers": ("s", "a"), "catalog_only": False})
            valve.assert_not_called()
            wiki.assert_not_called()
