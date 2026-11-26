import gzip
import json
import tempfile
import unittest
from pathlib import Path

import collect_research as research


class ArchiveTest(unittest.TestCase):
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
