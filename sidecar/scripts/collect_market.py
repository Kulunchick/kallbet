"""Collect market price history for the research archive (ticket #35, market part).

Read-only collection of public historical prices for a research backtest.
Nothing here places or facilitates a wager.

What it saves, into the research archive (default `target/research-archive`,
replayed by `kallbet-ai backtest --archive`), with the same names and shapes as
`sidecar/tests/fixtures/backtest`:

  oddspapi-fixtures-<name>-<from>-<to>.json.gz   OddsPapi /v4/fixtures (1 BILLABLE request each)
  <fixtureId>.json.gz                            OddsPapi /v4/historical-odds, Pinnacle, market 171 only (free, 1 per 5 s)
  polymarket-events-<tag>-closed-<offset>.json.gz, polymarket-history-<slug>.json.gz
  kalshi-markets-KXCS2GAME-settled-<min>-<page>.json.gz, kalshi-candles-<ticker>.json.gz
  sources-market.tsv                             file, request, fetched_at, sha256 (of the uncompressed payload)
  catalog/                                       empty lists, 404 memory (market-missing.tsv), budget ledger

No credential is ever written: the OddsPapi key travels in the apiKey query
parameter and the recorded request omits it. `audit` scans the archive.

Budgets are settings (flags) with conservative defaults and every run ends with
a report of the requests spent per Data Source. Re-running skips what is saved;
an interrupted run costs only the missing requests.

  python -I sidecar/scripts/collect_market.py --plan plan.json [--tournaments list.json] collect [--only oddspapi-fixtures,oddspapi-history,polymarket,kalshi]
  python -I sidecar/scripts/collect_market.py tournaments        # 1 billable request: OddsPapi tournament list for the plan
  python -I sidecar/scripts/collect_market.py audit              # exit 1 if a credential is found
  python -I sidecar/scripts/collect_market.py report

Polymarket and Kalshi need `--ssh-host USER@HOST --ssh-key PATH --known-hosts PATH`
(plain `curl` reads of public endpoints on the remote host the operator allowed).

plan.json:
  {"oddspapi":   [{"tournament": 46101, "name": "cologne-2026", "from": "2026-06-02", "to": "2026-06-22"}],
   "polymarket": [{"tag": "cs2", "start_date_min": "2026-01-01", "start_date_max": "2026-12-31"}],
   "kalshi":     [{"min_close": 1767225600, "max_close": 1769904000, "event": "optional text of rules_primary"}]}
"""

import argparse
import gzip
import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import time
import urllib.parse
from datetime import date, datetime, timedelta, timezone

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DEFAULT_ARCHIVE = ROOT / "target/research-archive"
MANIFEST = "sources-market.tsv"
REMOTE_NOTE = " (fetched with curl on a remote host: the API is blocked from the workstation)"
REMOTE_HOSTS = ("gamma-api.polymarket.com", "clob.polymarket.com", "api.elections.kalshi.com")
FIXTURES_PACE = 12.0  # seconds between billable calls; a burst was answered with 429
FIXTURE_PAGE = 100  # a full page means the window may be truncated: split it
SAFE = re.compile(r"[^A-Za-z0-9._-]")


def load_backtest_collector():
    """The existing collector, loaded by path (`python -I` keeps this directory off sys.path)."""
    spec = importlib.util.spec_from_file_location("collect_backtest", HERE / "collect_backtest.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_remote_url(url):
    host = urllib.parse.urlparse(url).hostname
    if host not in REMOTE_HOSTS or "'" in url:
        raise RuntimeError(f"not a public market URL: {host}")


def iso(t):
    return t.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


class Collector:
    def __init__(self, archive, oddspapi=None, remote=None, now=None, sleep=time.sleep, secrets=(),
                 oddspapi_run_cap=40, oddspapi_month_cap=100, remote_run_cap=6000):
        self.archive = pathlib.Path(archive)
        self.catalog = self.archive / "catalog"
        self.oddspapi, self.remote = oddspapi, remote
        self.now = now or (lambda: datetime.now(timezone.utc))
        self.sleep, self.secrets = sleep, [s for s in secrets if s]
        self.run_cap, self.month_cap, self.remote_cap = oddspapi_run_cap, oddspapi_month_cap, remote_run_cap
        self.spent = {"oddspapi_billable": 0, "oddspapi_free": 0, "remote": 0}
        self.saved = 0
        self.notes = []
        self.archive.mkdir(parents=True, exist_ok=True)
        self.catalog.mkdir(exist_ok=True)

    # ---- files, manifest, ledger -------------------------------------------------

    def exists(self, name):
        return (self.archive / (name + ".gz")).exists() or (self.catalog / (name + ".gz")).exists()

    def read(self, name):
        for d in (self.archive, self.catalog):
            if (d / (name + ".gz")).exists():
                return gzip.decompress((d / (name + ".gz")).read_bytes())
        raise FileNotFoundError(name)

    def save(self, name, body, request, catalog=False):
        folder = self.catalog if catalog else self.archive
        (folder / (name + ".gz")).write_bytes(gzip.compress(body, mtime=0))
        rel = ("catalog/" if catalog else "") + name + ".gz"
        # Append-only, so two collectors running side by side (OddsPapi and remote) never lose a row;
        # `compact_manifest` (the report command) keeps the last row per file.
        row = "\t".join([rel, request, iso(self.now()), hashlib.sha256(body).hexdigest()])
        with open(self.archive / MANIFEST, "a", encoding="utf8", newline="\n") as out:
            out.write(row + "\n")
        self.saved += 1

    def compact_manifest(self):
        path = self.archive / MANIFEST
        if not path.exists():
            return
        rows = {}
        for line in path.read_text(encoding="utf8").splitlines():
            if line.strip():
                rows[line.split("\t")[0]] = line
        path.write_text("\n".join(sorted(rows.values())) + "\n", encoding="utf8", newline="\n")

    def month(self):
        return self.now().strftime("%Y-%m")

    def ledger(self):
        path = self.catalog / "market-ledger.json"
        return json.loads(path.read_text()) if path.exists() else {"oddspapi_billable": {}}

    def month_spent(self):
        return self.ledger()["oddspapi_billable"].get(self.month(), 0)

    def spend_billable(self):
        """Take one billable request from the budgets, or return False (and say why)."""
        if self.spent["oddspapi_billable"] >= self.run_cap:
            self.notes.append(f"OddsPapi run cap {self.run_cap} reached; stopped before the next billable request")
            return False
        if self.month_spent() >= self.month_cap:
            self.notes.append(f"OddsPapi monthly cap {self.month_cap} reached; stopped before the next billable request")
            return False
        ledger = self.ledger()
        ledger["oddspapi_billable"][self.month()] = self.month_spent() + 1
        (self.catalog / "market-ledger.json").write_text(json.dumps(ledger, indent=1))
        self.spent["oddspapi_billable"] += 1
        return True

    # ---- OddsPapi ---------------------------------------------------------------------

    def oddspapi_fixtures(self, plan):
        for item in plan:
            stop = self._fixture_window(item["tournament"], item["name"],
                                        date.fromisoformat(item["from"]), date.fromisoformat(item["to"]))
            if stop:
                return

    def _fixture_window(self, tournament, label, start, end):
        """Fetch [start, end] page by page. The API returns the earliest 100 fixtures of a window, so a
        full page is kept up to the day before its last day and the next request starts on that day.
        Returns True when a budget stops the run."""
        cur = start
        while cur <= end:
            base = f"oddspapi-fixtures-{label}-{cur}-{end}"
            if self.exists(base + ".json"):
                return False
            if self.exists(base + "-truncated.json"):
                body, request = self.read(base + "-truncated.json"), "(saved full page)"
            else:
                if not self.spend_billable():
                    return True
                params = {"tournamentId": tournament, "statusId": 2, "from": str(cur), "to": str(end)}
                try:
                    body, request = self.oddspapi("fixtures", params)
                except RuntimeError as e:
                    self.sleep(FIXTURES_PACE)
                    if "404" in str(e):  # OddsPapi has nothing for this window: remember it
                        self.save(base + ".json", b"[]", f"{params} answered HTTP 404", catalog=True)
                        return False
                    self.notes.append(f"fixtures {label} {cur}..{end}: {e}")
                    return "429" in str(e)  # rate limited: stop the run, resume later
                self.sleep(FIXTURES_PACE)
            fixtures = json.loads(body)
            if len(fixtures) < FIXTURE_PAGE or cur >= end:
                self.save(base + ".json", body, request, catalog=not fixtures)
                return False
            if not self.exists(base + "-truncated.json"):
                self.save(base + "-truncated.json", body, request, catalog=True)
            days = [f["startTime"][:10] for f in fixtures]
            last = date.fromisoformat(max(days))
            if last <= cur:  # the whole page is one day: keep it and move on
                self.save(f"oddspapi-fixtures-{label}-{cur}-{cur}.json", body, request)
                cur += timedelta(days=1)
                continue
            before = [f for f in fixtures if f["startTime"][:10] < str(last)]
            self.save(f"oddspapi-fixtures-{label}-{cur}-{last - timedelta(days=1)}.json",
                      json.dumps(before).encode(), request + " (days before the last day of the page)")
            cur = last
        return False

    def fixture_lists(self):
        for path in sorted(self.archive.glob("oddspapi-fixtures-*.json.gz")):
            data = json.loads(gzip.decompress(path.read_bytes()))
            if isinstance(data, list):
                yield from data

    def missing(self):
        path = self.catalog / "market-missing.tsv"
        return {l.split("\t")[0] for l in path.read_text().splitlines()} if path.exists() else set()

    def oddspapi_history(self):
        backtest = load_backtest_collector()
        missing = self.missing()
        for f in self.fixture_lists():
            fid = f["fixtureId"]
            if f.get("statusId") != 2 or not str(f.get("startTime", "")).startswith("2026"):
                continue
            if fid in missing or (self.archive / (fid + ".json.gz")).exists():
                continue
            params = {"fixtureId": fid, "bookmakers": "pinnacle+30"}
            try:
                body, request = self.oddspapi("historical-odds", params)
            except RuntimeError as e:
                self.spent["oddspapi_free"] += 1
                if "404" in str(e):
                    with open(self.catalog / "market-missing.tsv", "a", encoding="utf8") as out:
                        out.write(f"{fid}\tHTTP 404\t{iso(self.now())}\n")
                else:
                    self.notes.append(f"{fid}: {e}")
                    if "429" in str(e):
                        return
                self.sleep(5.2)
                continue
            self.spent["oddspapi_free"] += 1
            self.save(fid + ".json", backtest.trim(body), request + " (trimmed to market 171)")
            self.sleep(5.2)  # historical-odds: one call per 5 s.

    # ---- remote: Polymarket and Kalshi --------------------------------------------------

    def remote_get(self, url):
        if self.spent["remote"] >= self.remote_cap:
            self.notes.append(f"remote run cap {self.remote_cap} reached")
            return None
        check_remote_url(url)
        self.spent["remote"] += 1
        try:
            return self.remote(url)
        except RuntimeError as e:
            self.notes.append(f"{urllib.parse.urlparse(url).path}: {e}")
            return None

    def _cached_or_remote(self, name, url):
        """The saved payload if there is one, else a fetched (and saved) one."""
        if self.exists(name):
            return self.read(name)
        body = self.remote_get(url)
        if body is not None:
            self.save(name, body, url + REMOTE_NOTE)
        return body

    def polymarket(self, plan, max_pages=40):
        for item in plan:
            tag = item["tag"]
            for page in range(max_pages):
                offset = page * 100
                url = f"https://gamma-api.polymarket.com/events?tag_slug={tag}&closed=true&limit=100&offset={offset}"
                for key in ("start_date_min", "start_date_max"):
                    if item.get(key):
                        url += f"&{key}={item[key]}"
                name = f"polymarket-events-{tag}-{item['label']}-closed-{offset}.json" if item.get("label") else f"polymarket-events-{tag}-closed-{offset}.json"
                if self.exists(name):
                    body = self.read(name)
                else:
                    body = self.remote_get(url)
                    if body is None:
                        break
                    if not isinstance(json.loads(body), list):
                        self.notes.append(f"{name}: answer is not a list ({body[:80]!r}); window ends")
                        break
                    self.save(name, body, url + REMOTE_NOTE, catalog=not json.loads(body))
                events = json.loads(body)
                if not events:
                    break
                for event in events:
                    words = item.get("title_any")
                    if words and not any(w.lower() in event.get("title", "").lower() for w in words):
                        continue
                    self._polymarket_history(event)

    def _polymarket_history(self, event):
        lines = [m for m in event.get("markets", []) if m.get("sportsMarketType") == "moneyline"]
        if not lines or not lines[0].get("gameStartTime") or not lines[0].get("clobTokenIds"):
            return
        name = "polymarket-history-" + SAFE.sub("_", event["slug"])
        if self.exists(name + ".json"):
            return
        start = datetime.fromisoformat(lines[0]["gameStartTime"].replace(" ", "T").replace("+00", "+00:00"))
        token = json.loads(lines[0]["clobTokenIds"])[0]
        a, b = int((start - timedelta(hours=48)).timestamp()), int((start + timedelta(hours=2)).timestamp())
        url = f"https://clob.polymarket.com/prices-history?market={token}&startTs={a}&endTs={b}&fidelity=5"
        body = self.remote_get(url)
        if body is not None:
            self.save(name + ".json", body, url + REMOTE_NOTE)

    def kalshi(self, plan, max_pages=60):
        backtest = load_backtest_collector()
        for item in plan:
            cursor = ""
            for page in range(max_pages):
                url = ("https://api.elections.kalshi.com/trade-api/v2/markets?series_ticker=KXCS2GAME"
                       f"&status=settled&limit=200&min_close_ts={item['min_close']}&max_close_ts={item['max_close']}")
                if cursor:
                    url += "&cursor=" + urllib.parse.quote(cursor)
                name = f"kalshi-markets-KXCS2GAME-settled-{item['min_close']}-{page}.json"
                body = self._cached_or_remote(name, url)
                if body is None:
                    break
                data = json.loads(body)
                for market in data.get("markets", []):
                    if item.get("event") and item["event"] not in market.get("rules_primary", ""):
                        continue
                    ticker = market["ticker"]
                    cname = f"kalshi-candles-{SAFE.sub('_', ticker)}.json"
                    if self.exists(cname):
                        continue
                    start = backtest.kalshi_start(ticker)
                    a, b = int((start - timedelta(hours=6)).timestamp()), int((start + timedelta(hours=2)).timestamp())
                    curl = ("https://api.elections.kalshi.com/trade-api/v2/series/KXCS2GAME/markets/"
                            f"{ticker}/candlesticks?start_ts={a}&end_ts={b}&period_interval=1")
                    cbody = self.remote_get(curl)
                    if cbody is not None:
                        self.save(cname, cbody, curl + REMOTE_NOTE)
                cursor = data.get("cursor") or ""
                if not cursor or not data.get("markets"):
                    break

    # ---- audit and report ------------------------------------------------------------------

    def audit(self):
        """Relative names of files that hold a credential."""
        bad = []
        for path in sorted(p for p in self.archive.rglob("*") if p.is_file()):
            data = path.read_bytes()
            if path.suffix == ".gz":
                data = gzip.decompress(data)
            hit = any(s.encode() in data for s in self.secrets)
            if path.suffix != ".gz":
                hit = hit or b"apiKey=" in data
            if hit:
                bad.append(path.relative_to(self.archive).as_posix())
        return bad

    def report(self):
        lines = [
            f"oddspapi billable: {self.spent['oddspapi_billable']} this run, {self.month_spent()} this month "
            f"({self.month()}) of {self.month_cap} monthly cap, run cap {self.run_cap}",
            f"oddspapi history (free): {self.spent['oddspapi_free']} this run",
            f"remote (polymarket, kalshi): {self.spent['remote']} this run, run cap {self.remote_cap}",
            f"files saved this run: {self.saved}",
        ] + [f"note: {n}" for n in self.notes]
        return "\n".join(lines)


def load_plan(plan_path, tournaments_path=None):
    """The plan file plus an optional tournament list: [{"id" or "tournament", "name", "from", "to"}]."""
    plan = json.loads(pathlib.Path(plan_path).read_text(encoding="utf8")) if plan_path else {}
    plan.setdefault("oddspapi", [])
    if tournaments_path:
        for t in json.loads(pathlib.Path(tournaments_path).read_text(encoding="utf8")):
            plan["oddspapi"].append({"tournament": t.get("tournament", t.get("id")), "name": t["name"],
                                     "from": t["from"], "to": t["to"]})
    return plan


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--archive", default=str(DEFAULT_ARCHIVE))
    parser.add_argument("--env", default=str(ROOT / ".env"))
    parser.add_argument("--plan")
    parser.add_argument("--tournaments", help="JSON list of OddsPapi tournaments [{id, name, from, to}] added to the plan")
    parser.add_argument("--ssh-host")
    parser.add_argument("--ssh-key")
    parser.add_argument("--known-hosts")
    parser.add_argument("--oddspapi-run-cap", type=int, default=40, help="billable requests per run")
    parser.add_argument("--oddspapi-month-cap", type=int, default=100,
                        help="billable requests per calendar month for this archive (the free plan has 250 shared with the live sidecar)")
    parser.add_argument("--remote-run-cap", type=int, default=6000)
    parser.add_argument("--only", default="oddspapi-fixtures,oddspapi-history,polymarket,kalshi")
    parser.add_argument("command", choices=["collect", "tournaments", "audit", "report"])
    args = parser.parse_args()
    backtest = load_backtest_collector()
    needs_odds = args.command in ("tournaments", "audit") or (
        args.command == "collect" and {"oddspapi-fixtures", "oddspapi-history"} & set(args.only.split(",")))
    if needs_odds:
        backtest.load_secrets(args.env)
    backtest.REMOTE.update(host=args.ssh_host, key=args.ssh_key, known_hosts=args.known_hosts)
    collector = Collector(
        args.archive, oddspapi=lambda path, params: backtest.oddspapi(path, params),
        remote=backtest.remote_get, secrets=backtest.SECRETS.values(),
        oddspapi_run_cap=args.oddspapi_run_cap, oddspapi_month_cap=args.oddspapi_month_cap,
        remote_run_cap=args.remote_run_cap)
    if args.command == "audit":
        bad = collector.audit()
        print("\n".join(bad) if bad else "no credential found")
        sys.exit(1 if bad else 0)
    if args.command == "tournaments":
        if collector.spend_billable():
            body, request = collector.oddspapi("tournaments", {"sportId": 17})
            collector.save("oddspapi-tournaments-17.json", body, request, catalog=True)
    elif args.command == "report":
        collector.compact_manifest()
    elif args.command == "collect":
        plan = load_plan(args.plan, args.tournaments)
        steps = args.only.split(",")
        try:
            if "oddspapi-fixtures" in steps:
                collector.oddspapi_fixtures(plan.get("oddspapi", []))
            if "oddspapi-history" in steps:
                collector.oddspapi_history()
            if "polymarket" in steps:
                collector.polymarket(plan.get("polymarket", []))
            if "kalshi" in steps:
                collector.kalshi(plan.get("kalshi", []))
        finally:
            print(collector.report())
        return
    print(collector.report())


if __name__ == "__main__":
    main()
