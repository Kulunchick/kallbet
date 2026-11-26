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
