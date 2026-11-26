"""Offline tests of collect_market.py: fake fetchers, no network, no credential.

  python -I sidecar/scripts/test_collect_market.py
"""
import gzip, hashlib, importlib.util, json, pathlib, tempfile, unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("collect_market", HERE / "collect_market.py")
cm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cm)
FIXTURES = HERE.parent / "tests/fixtures/backtest"


def real(name):
    return gzip.decompress((FIXTURES / name).read_bytes())


class FakeOdds:
    """Stands for OddsPapi: answers by path, records every call, never sees a key."""

    def __init__(self, history=None, fixtures=None):
        self.calls, self.history, self.fixtures = [], history or {}, fixtures or {}

    def __call__(self, path, params):
        self.calls.append((path, dict(params)))
        url = "https://api.oddspapi.io/v4/" + path + "?" + cm.urllib.parse.urlencode(params)
        if path == "historical-odds":
            r = self.history.get(params["fixtureId"], "HTTP 404")
        else:
            r = self.fixtures.get((params["tournamentId"], params["from"], params["to"]), b"[]")
        if isinstance(r, str):
            raise RuntimeError(r)
        return r, url

    def count(self, path):
        return sum(1 for c in self.calls if c[0] == path)


class Remote:
    def __init__(self, answers):
        self.urls, self.answers = [], answers

    def __call__(self, url):
        self.urls.append(url)
        for key, body in self.answers.items():
            if key in url:
                return body
        raise RuntimeError("remote curl failed (22)")


def make(tmp, odds=None, remote=None, **caps):
    return cm.Collector(pathlib.Path(tmp), oddspapi=odds, remote=remote,
                        now=lambda: cm.datetime(2026, 11, 26, 12, 0, 0, tzinfo=cm.timezone.utc),
                        sleep=lambda s: None, secrets=["SECRET-KEY"], **caps)


def put_fixtures(tmp, name, fixtures):
    (pathlib.Path(tmp) / (name + ".json.gz")).write_bytes(gzip.compress(json.dumps(fixtures).encode()))


def fx(i, start="2026-03-01T10:00:00.000Z", status=2):
    return {"fixtureId": i, "statusId": status, "startTime": start}


class History(unittest.TestCase):
    def test_saves_trimmed_pinnacle_history_and_lists_it(self):
        with tempfile.TemporaryDirectory() as t:
            put_fixtures(t, "oddspapi-fixtures-x", [fx("id1")])
            body = real("id1704610171653816.json.gz")
            c = make(t, FakeOdds(history={"id1": body}))
            c.oddspapi_history()
            saved = json.loads(gzip.decompress((pathlib.Path(t) / "id1.json.gz").read_bytes()))
            self.assertEqual(list(saved["bookmakers"]["pinnacle+30"]["markets"]), ["171"])
            rows = [l.split("\t") for l in (pathlib.Path(t) / "sources-market.tsv").read_text().splitlines()]
            (file, request, at, sha), = [r for r in rows if r[0] == "id1.json.gz"]
            self.assertEqual(request, "https://api.oddspapi.io/v4/historical-odds?fixtureId=id1&bookmakers=pinnacle%2B30"
                             " (trimmed to market 171)")
            self.assertEqual(at, "2026-11-26T12:00:00Z")
            raw = gzip.decompress((pathlib.Path(t) / "id1.json.gz").read_bytes())
            self.assertEqual(sha, hashlib.sha256(raw).hexdigest())
            self.assertNotIn("apiKey", request)

    def test_second_run_makes_no_request_and_a_404_is_remembered(self):
        with tempfile.TemporaryDirectory() as t:
            put_fixtures(t, "oddspapi-fixtures-x", [fx("id1"), fx("id2")])
            odds = FakeOdds(history={"id1": real("id1704610171653816.json.gz")})
            make(t, odds).oddspapi_history()
            self.assertEqual(odds.count("historical-odds"), 2)
            again = FakeOdds()
            make(t, again).oddspapi_history()
            self.assertEqual(again.calls, [])
            self.assertTrue((pathlib.Path(t) / "catalog/market-missing.tsv").read_text().startswith("id2\t"))

    def test_only_2026_finished_fixtures_and_429_stops_the_run(self):
        with tempfile.TemporaryDirectory() as t:
            put_fixtures(t, "oddspapi-fixtures-x", [
                fx("old", "2025-12-31T23:00:00.000Z"), fx("live", status=1), fx("a"), fx("b")])
            odds = FakeOdds(history={"a": "HTTP 429"})
            make(t, odds).oddspapi_history()
            self.assertEqual([c[1]["fixtureId"] for c in odds.calls], ["a"])

    def test_history_is_paced_at_one_call_per_5_seconds(self):
        with tempfile.TemporaryDirectory() as t:
            put_fixtures(t, "oddspapi-fixtures-x", [fx("a")])
            slept = []
            c = make(t, FakeOdds())
            c.sleep = slept.append
            c.oddspapi_history()
            self.assertEqual(slept, [5.2])


class Fixtures(unittest.TestCase):
    PLAN = [{"tournament": 46101, "name": "cologne-2026", "from": "2026-06-02", "to": "2026-06-22"}]

    def test_saves_the_window_and_counts_one_billable_request(self):
        with tempfile.TemporaryDirectory() as t:
            body = json.dumps([fx("a")]).encode()
            odds = FakeOdds(fixtures={(46101, "2026-06-02", "2026-06-22"): body})
            c = make(t, odds)
            c.oddspapi_fixtures(self.PLAN)
            self.assertTrue((pathlib.Path(t) / "oddspapi-fixtures-cologne-2026-2026-06-02-2026-06-22.json.gz").exists())
            self.assertEqual(c.spent["oddspapi_billable"], 1)
            odds2 = FakeOdds()
            make(t, odds2).oddspapi_fixtures(self.PLAN)
            self.assertEqual(odds2.calls, [])

    def test_full_page_continues_from_its_last_day_and_keeps_only_complete_days(self):
        with tempfile.TemporaryDirectory() as t:
            days = ["2026-06-02", "2026-06-03", "2026-06-04", "2026-06-05"]
            full = json.dumps([fx(str(i), days[i // 25] + "T10:00:00.000Z") for i in range(100)]).encode()
            tail = json.dumps([fx("h", "2026-06-05T12:00:00.000Z"), fx("i", "2026-06-09T12:00:00.000Z")]).encode()
            odds = FakeOdds(fixtures={(46101, "2026-06-02", "2026-06-22"): full,
                                      (46101, "2026-06-05", "2026-06-22"): tail})
            make(t, odds).oddspapi_fixtures(self.PLAN)
            self.assertEqual(odds.count("fixtures"), 2)
            root = pathlib.Path(t)
            first = json.loads(gzip.decompress((root / "oddspapi-fixtures-cologne-2026-2026-06-02-2026-06-04.json.gz").read_bytes()))
            self.assertEqual(len(first), 75)  # the 06-05 fixtures come again with the next page
            self.assertTrue((root / "oddspapi-fixtures-cologne-2026-2026-06-05-2026-06-22.json.gz").exists())
            again = FakeOdds()
            make(t, again).oddspapi_fixtures(self.PLAN)
            self.assertEqual(again.calls, [])

    def test_a_429_stops_the_fixtures_run_and_requests_are_paced(self):
        with tempfile.TemporaryDirectory() as t:
            plan = [{"tournament": i, "name": f"t{i}", "from": "2026-06-02", "to": "2026-06-03"} for i in range(4)]
            slept = []
            odds = FakeOdds(fixtures={(1, "2026-06-02", "2026-06-03"): "HTTP 429"})
            c = make(t, odds)
            c.sleep = slept.append
            c.oddspapi_fixtures(plan)
            self.assertEqual([x[1]["tournamentId"] for x in odds.calls], [0, 1])
            self.assertEqual(slept, [cm.FIXTURES_PACE] * 2)
            self.assertIn("429", " ".join(c.notes))

    def test_a_404_is_remembered_as_an_empty_window(self):
        with tempfile.TemporaryDirectory() as t:
            odds = FakeOdds(fixtures={(46101, "2026-06-02", "2026-06-22"): "HTTP 404"})
            make(t, odds).oddspapi_fixtures(self.PLAN)
            again = FakeOdds()
            make(t, again).oddspapi_fixtures(self.PLAN)
            self.assertEqual(again.calls, [])
            self.assertTrue((pathlib.Path(t) / "catalog/oddspapi-fixtures-cologne-2026-2026-06-02-2026-06-22.json.gz").exists())

    def test_empty_list_goes_to_catalog(self):
        with tempfile.TemporaryDirectory() as t:
            make(t, FakeOdds()).oddspapi_fixtures(self.PLAN)
            self.assertTrue((pathlib.Path(t) / "catalog/oddspapi-fixtures-cologne-2026-2026-06-02-2026-06-22.json.gz").exists())

    def test_caps_stop_before_the_request_and_the_ledger_persists(self):
        with tempfile.TemporaryDirectory() as t:
            plan = [{"tournament": i, "name": f"t{i}", "from": "2026-06-02", "to": "2026-06-03"} for i in range(5)]
            odds = FakeOdds()
            c = make(t, odds, oddspapi_run_cap=2)
            c.oddspapi_fixtures(plan)
            self.assertEqual(odds.count("fixtures"), 2)
            self.assertIn("run cap", " ".join(c.notes))
            odds2 = FakeOdds()
            c2 = make(t, odds2, oddspapi_month_cap=3)
            c2.oddspapi_fixtures(plan)
            self.assertEqual(odds2.count("fixtures"), 1)  # 2 already spent this month
            self.assertIn("monthly cap", " ".join(c2.notes))

    def test_report_names_requests_and_budgets(self):
        with tempfile.TemporaryDirectory() as t:
            c = make(t, FakeOdds(), oddspapi_run_cap=7)
            c.oddspapi_fixtures(self.PLAN)
            self.assertIn("oddspapi billable: 1 this run, 1 this month (2026-11) of 100 monthly cap, run cap 7", c.report())


POLY_EVENTS = [{"slug": "cs2-a-b-2026-03-01", "markets": [
    {"sportsMarketType": "moneyline", "gameStartTime": "2026-03-01 10:00:00+00", "clobTokenIds": "[\"tok1\",\"tok2\"]"}]},
    {"slug": "prop", "markets": [{"sportsMarketType": "totals"}]}]


class Polymarket(unittest.TestCase):
    PLAN = [{"tag": "cs2"}]

    def test_collects_event_page_and_history_then_resumes_without_requests(self):
        with tempfile.TemporaryDirectory() as t:
            remote = Remote({"gamma-api.polymarket.com/events?tag_slug=cs2&closed=true&limit=100&offset=0":
                             json.dumps(POLY_EVENTS).encode(),
                             "offset=100": b"[]",
                             "clob.polymarket.com/prices-history?market=tok1&startTs=1772186400&endTs=1772366400&fidelity=5":
                             b'{"history":[{"t":1,"p":0.5}]}'})
            c = make(t, remote=remote)
            c.polymarket(self.PLAN)
            root = pathlib.Path(t)
            self.assertTrue((root / "polymarket-history-cs2-a-b-2026-03-01.json.gz").exists())
            self.assertEqual(len(remote.urls), 3)
            self.assertEqual(c.spent["remote"], 3)
            remote2 = Remote({})
            make(t, remote=remote2).polymarket(self.PLAN)
            self.assertEqual(remote2.urls, [])

    def test_refuses_a_non_market_host(self):
        with self.assertRaises(RuntimeError):
            cm.check_remote_url("https://example.com/x")
        cm.check_remote_url("https://clob.polymarket.com/prices-history?market=1")


class Kalshi(unittest.TestCase):
    def test_collects_markets_pages_then_candles_per_ticker(self):
        with tempfile.TemporaryDirectory() as t:
            ticker = "KXCS2GAME-26MAR011000ABCDEF-ABC"
            page1 = json.dumps({"markets": [{"ticker": ticker, "rules_primary": "x BLAST y"}], "cursor": "c2"}).encode()
            page2 = json.dumps({"markets": [], "cursor": ""}).encode()
            remote = Remote({"cursor=c2": page2, "min_close_ts=100": page1,
                             f"markets/{ticker}/candlesticks": b'{"candlesticks":[]}'})
            plan = [{"min_close": 100, "max_close": 200}]
            make(t, remote=remote).kalshi(plan)
            root = pathlib.Path(t)
            self.assertTrue((root / f"kalshi-candles-{ticker}.json.gz").exists())
            self.assertEqual(len(remote.urls), 3)
            r2 = Remote({})
            make(t, remote=r2).kalshi(plan)
            self.assertEqual(r2.urls, [])

    def test_event_text_filters_markets(self):
        with tempfile.TemporaryDirectory() as t:
            page = json.dumps({"markets": [{"ticker": "KXCS2GAME-26MAR011000ABCDEF-ABC", "rules_primary": "other"}]}).encode()
            remote = Remote({"min_close_ts=100": page})
            make(t, remote=remote).kalshi([{"min_close": 100, "max_close": 200, "event": "BLAST"}])
            self.assertEqual(len(remote.urls), 1)


class Manifest(unittest.TestCase):
    def test_rows_append_and_compact_keeps_the_last_row_per_file(self):
        with tempfile.TemporaryDirectory() as t:
            c = make(t)
            c.save("a.json", b"1", "r1")
            c.save("a.json", b"2", "r2")
            c.save("b.json", b"3", "r3")
            path = pathlib.Path(t) / "sources-market.tsv"
            self.assertEqual(len(path.read_text().splitlines()), 3)
            c.compact_manifest()
            rows = [l.split("	") for l in path.read_text().splitlines()]
            self.assertEqual([(r[0], r[1]) for r in rows], [("a.json.gz", "r2"), ("b.json.gz", "r3")])


class Plan(unittest.TestCase):
    def test_a_tournament_list_file_joins_the_plan(self):
        with tempfile.TemporaryDirectory() as t:
            plan = pathlib.Path(t) / "plan.json"
            plan.write_text(json.dumps({"polymarket": [{"tag": "cs2"}]}))
            lst = pathlib.Path(t) / "tournaments.json"
            lst.write_text(json.dumps([{"id": 7, "name": "blast-2026", "from": "2026-02-01", "to": "2026-02-10"}]))
            got = cm.load_plan(plan, lst)
            self.assertEqual(got["oddspapi"], [{"tournament": 7, "name": "blast-2026", "from": "2026-02-01", "to": "2026-02-10"}])
            self.assertEqual(got["polymarket"], [{"tag": "cs2"}])
            self.assertEqual(cm.load_plan(None, lst)["kalshi"] if "kalshi" in cm.load_plan(None, lst) else [], [])


class Audit(unittest.TestCase):
    def test_finds_a_credential_in_a_file_or_manifest(self):
        with tempfile.TemporaryDirectory() as t:
            c = make(t)
            self.assertEqual(c.audit(), [])
            (pathlib.Path(t) / "bad.json.gz").write_bytes(gzip.compress(b'{"k":"SECRET-KEY"}'))
            (pathlib.Path(t) / "sources-market.tsv").write_text("f\thttps://x?apiKey=1\t2026\tab\n")
            self.assertEqual(sorted(c.audit()), ["bad.json.gz", "sources-market.tsv"])


if __name__ == "__main__":
    unittest.main()
