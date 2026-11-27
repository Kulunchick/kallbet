import gzip
import json
import tempfile
import unittest
from pathlib import Path

import collect_liquipedia_maps as maps


class TitleResolutionTest(unittest.TestCase):
    def test_pick_title_requires_one_exact_word_match(self):
        hits = [{"title": "BLAST/Premier/2024/Fall/Groups"}, {"title": "BLAST/Premier/2024/Fall"}]
        self.assertEqual(maps.pick_title("BLAST Premier Fall Groups 2024", hits), "BLAST/Premier/2024/Fall/Groups")
        self.assertIsNone(maps.pick_title("BLAST Premier Fall Finals 2024", hits))
        twin = hits[:1] * 2
        self.assertIsNone(maps.pick_title("BLAST Premier Fall Groups 2024", twin))

    def test_resolve_titles_searches_once_and_caches(self):
        calls = []

        class Limiter:
            calls = 0
            def fetch(self, params):
                calls.append(params)
                body = json.dumps({"query": {"search": [{"title": "CCT/2024/Europe/Series 5"}]}}).encode()
                return body, "https://liquipedia.net/counterstrike/api.php?x"

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            series = {"1": "CCT Europe Series 5 2024", "2": "Known League 2024"}
            for _ in range(2):
                titles = maps.resolve_titles(series, {"2": "Known/League/2024"}, out, out / "s.tsv", Limiter(), 10)
                self.assertEqual(titles, {"1": "CCT/2024/Europe/Series 5"})
            self.assertEqual(len(calls), 1)
            self.assertEqual(calls[0]["srsearch"], "CCT Europe Series 5 2024")
            self.assertIn("search-1.json.gz", (out / "s.tsv").read_text())


if __name__ == "__main__":
    unittest.main()
