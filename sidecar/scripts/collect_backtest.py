"""Collect the replay archive of `kallbet-ai backtest` (ticket #31).

Every request this script makes is recorded in `sources.tsv` next to the saved
file (without credentials). Responses are saved gzip-compressed, unchanged,
except `trim-odds`, which keeps only the series-winner market of a saved
OddsPapi history (see README.md in the archive).

Credentials are read into memory from an env file (default `.env` at the
repository root): `PANDASCORE_TOKEN`, `ODDSPAPI_API_KEY`. They are never
printed, logged or written; the OddsPapi key travels in the `apiKey` query
parameter, so no full OddsPapi request URL is ever printed or saved.

  python -I sidecar/scripts/collect_backtest.py stage-revisions        # Liquipedia, free
  python -I sidecar/scripts/collect_backtest.py event-revision TITLE ISO_TIME NAME
  python -I sidecar/scripts/collect_backtest.py pandascore TOURNAMENT...
  python -I sidecar/scripts/collect_backtest.py oddspapi-settled TOURNAMENT NAME FROM TO   # 1 billable request
  python -I sidecar/scripts/collect_backtest.py oddspapi-history [--missing]              # free, 1 per 5 s
  python -I sidecar/scripts/collect_backtest.py trim-odds
  python -I sidecar/scripts/collect_backtest.py account                 # prints the request counter only
  python -I sidecar/scripts/collect_backtest.py --ssh-host USER@HOST --ssh-key PATH --known-hosts PATH       polymarket pgl-major-singapore-2026                                # through a remote curl
  python -I sidecar/scripts/collect_backtest.py --ssh-host ... kalshi MIN_CLOSE MAX_CLOSE "PGL Major Singapore"
"""

import argparse
import gzip
import json
import pathlib
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timedelta, timezone

ROOT = pathlib.Path(__file__).resolve().parents[2]
ARCHIVE = ROOT / "sidecar/tests/fixtures/backtest"
LIQUIPEDIA = "https://liquipedia.net/counterstrike/api.php"
USER_AGENT = "KallBet/0.1 (https://kallbet.kulunchick.tech)"
SERIES_WINNER = "171"
SECRETS = {}


def load_secrets(env_file):
    for line in pathlib.Path(env_file).read_text(encoding="utf-8-sig").splitlines():
        key, _, value = line.partition("=")
        if key.strip() in ("ODDSPAPI_API_KEY", "PANDASCORE_TOKEN"):
            SECRETS[key.strip()] = value.strip().strip('"').strip("'")


def get(url, params, headers=None, echoes_secret=False):
    request = urllib.request.Request(
        url + "?" + urllib.parse.urlencode(params), headers=headers or {}
    )
    try:
        body = urllib.request.urlopen(request, timeout=40).read()
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"HTTP {e.code}") from None
    except Exception:
        raise RuntimeError("transport failure") from None
    if body[:2] == b"\x1f\x8b":
        body = gzip.decompress(body)
    if not echoes_secret and any(s and s.encode() in body for s in SECRETS.values()):
        raise RuntimeError("a credential appears in the response; not saved")
    return body


def save(name, body, request):
    path = ARCHIVE / (name + ".gz")
    path.write_bytes(gzip.compress(body, mtime=0))
    lines = [
        l
        for l in (ARCHIVE / "sources.tsv").read_text(encoding="utf8").splitlines()
        if not l.startswith(path.name + "\t")
    ]
    lines.append(path.name + "\t" + request)
    (ARCHIVE / "sources.tsv").write_text("\n".join(sorted(lines)) + "\n", encoding="utf8")
    print("saved", path.name, flush=True)


def load(path):
    data = path.read_bytes()
    return json.loads(gzip.decompress(data) if path.suffix == ".gz" else data)


def iso(t):
    return t.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_time(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def stage_cutoff(rows):
    """As `archive.rs`: earliest Scheduled Start or actual start of a Match."""
    times = [
        parse_time(t)
        for m in rows
        if m["status"] != "canceled"
        for t in (m["scheduled_at"], m["begin_at"])
        if t
    ]
    return min(times)


def revision(title, before):
    params = {
        "action": "query",
        "prop": "revisions",
        "rvprop": "ids|timestamp|content",
        "rvslots": "main",
        "rvlimit": 1,
        "rvstart": iso(before),
        "rvdir": "older",
        "formatversion": 2,
        "format": "json",
        "titles": title,
    }
    body = get(LIQUIPEDIA, params, {"User-Agent": USER_AGENT, "Accept-Encoding": "gzip"})
    time.sleep(2.1)  # Liquipedia API terms: at most 1 request per 2 s.
    return body, LIQUIPEDIA + "?" + urllib.parse.urlencode(params)


def stage_revisions(_):
    """Refetch each Stage revision that is not strictly before its Stage cutoff."""
    for panda in sorted(ARCHIVE.glob("panda-*.json.gz")):
        tournament = panda.name[len("panda-") : -len(".json.gz")]
        page_file = ARCHIVE / f"lp-{tournament}.json.gz"
        if not page_file.exists():
            continue
        cutoff = stage_cutoff(load(panda))
        page = load(page_file)["query"]["pages"][0]
        saved = parse_time(page["revisions"][0]["timestamp"])
        if saved < cutoff:
            continue
        print(f"lp-{tournament}: saved revision {iso(saved)} >= cutoff {iso(cutoff)}", flush=True)
        body, request = revision(page["title"], cutoff - timedelta(seconds=1))
        if not json.loads(body)["query"]["pages"][0].get("revisions"):
            print("  no earlier revision", flush=True)
            continue
        save(f"lp-{tournament}.json", body, request)


def event_revision(args):
    before = parse_time(args.time) - timedelta(seconds=1)
    body, request = revision(args.title, before)
    if not json.loads(body)["query"]["pages"][0].get("revisions"):
        sys.exit("no revision before that time")
    save(args.name + ".json", body, request)


def pandascore(args):
    for tournament in args.tournaments:
        url = f"https://api.pandascore.co/tournaments/{tournament}/matches"
        params = {"per_page": 100}
        body = get(url, params, {"Authorization": "Bearer " + SECRETS["PANDASCORE_TOKEN"]})
        save(f"panda-{tournament}.json", body, url + "?" + urllib.parse.urlencode(params))


def oddspapi(path, params, echoes_secret=False):
    url = "https://api.oddspapi.io/v4/" + path
    body = get(url, params | {"apiKey": SECRETS["ODDSPAPI_API_KEY"]}, echoes_secret=echoes_secret)
    # The recorded request names the parameters but never the key.
    return body, url + "?" + urllib.parse.urlencode(params)


def oddspapi_settled(args):
    params = {"tournamentId": args.tournament, "statusId": 2, "from": args.start, "to": args.end}
    body, request = oddspapi("fixtures", params)
    save(args.name + ".json", body, request)


def oddspapi_history(args):
    lists = [p for p in ARCHIVE.glob("*.json.gz") if isinstance(load(p), list) and load(p) and "fixtureId" in load(p)[0]]
    for fixtures in lists:
        for f in load(fixtures):
            if f.get("statusId") != 2:
                continue
            name = f["fixtureId"] + ".json"
            if args.missing and (ARCHIVE / (name + ".gz")).exists():
                continue
            params = {"fixtureId": f["fixtureId"], "bookmakers": "pinnacle+30"}
            try:
                body, request = oddspapi("historical-odds", params)
            except RuntimeError as e:
                print(f["fixtureId"], e, flush=True)
                if "429" in str(e):
                    return
                continue
            finally:
                time.sleep(5.2)  # historical-odds: one call per 5 s.
            save(name, trim(body), request + " (trimmed, see README)")


def account(_):
    """Print only the request counters of `/v4/account` (the body is not saved)."""
    body, _ = oddspapi("account", {}, echoes_secret=True)

    def walk(value, path=""):
        if isinstance(value, dict):
            for k, v in value.items():
                walk(v, path + "." + k)
        elif isinstance(value, list):
            for v in value:
                walk(v, path)
        elif any(w in path.lower() for w in ("request", "limit", "usage")):
            if isinstance(value, (int, float)) and str(value) not in SECRETS.values():
                print(path.lstrip("."), value, flush=True)

    walk(json.loads(body))


def trim(body):
    """Keep only the series-winner market (171) of every bookmaker."""
    history = json.loads(body)
    for book in history.get("bookmakers", {}).values():
        markets = book.get("markets", {})
        book["markets"] = {k: v for k, v in markets.items() if k == SERIES_WINNER}
    return json.dumps(history, separators=(",", ":")).encode()


def trim_odds(_):
    for path in sorted(ARCHIVE.glob("*.json.gz")):
        value = load(path)
        if isinstance(value, dict) and "bookmakers" in value and "fixtureId" in value:
            path.write_bytes(gzip.compress(trim(json.dumps(value).encode()), mtime=0))


REMOTE = {}
REMOTE_HOSTS = ("gamma-api.polymarket.com", "clob.polymarket.com", "api.elections.kalshi.com")


def remote_get(url):
    """GET through `curl` on a remote host (Polymarket and Kalshi are blocked from
    the workstation). Only public Polymarket and Kalshi URLs are allowed."""
    import subprocess

    host = urllib.parse.urlparse(url).hostname
    if host not in REMOTE_HOSTS or "'" in url:
        raise RuntimeError(f"not a public market URL: {host}")
    command = [
        "ssh", "-i", REMOTE["key"], "-o", "BatchMode=yes", "-o", "ConnectTimeout=12",
        "-o", "UserKnownHostsFile=" + REMOTE["known_hosts"], REMOTE["host"],
        f"curl -sS -m 40 --compressed -A '{USER_AGENT}' '{url}'",
    ]
    result = subprocess.run(command, capture_output=True, timeout=90)
    time.sleep(1.1)  # at most about one request per second
    if result.returncode != 0:
        raise RuntimeError(f"remote curl failed ({result.returncode})")
    json.loads(result.stdout)  # an error page is not saved
    return result.stdout


REMOTE_NOTE = " (fetched with curl on a remote host: the API is blocked from the workstation)"


def polymarket(args):
    events = []
    for offset in range(0, 500, 100):
        url = (
            "https://gamma-api.polymarket.com/events?tag_slug="
            f"{args.tag}&closed=true&limit=100&offset={offset}"
        )
        body = remote_get(url)
        page = json.loads(body)
        if not page:
            break
        save(f"polymarket-events-{args.tag}-closed-{offset}.json", body, url + REMOTE_NOTE)
        events.extend(page)
    for event in events:
        lines = [m for m in event.get("markets", []) if m.get("sportsMarketType") == "moneyline"]
        if not lines or not lines[0].get("gameStartTime"):
            continue
        start = parse_time(lines[0]["gameStartTime"].replace(" ", "T").replace("+00", "+00:00"))
        token = json.loads(lines[0]["clobTokenIds"])[0]
        start_ts = int((start - timedelta(hours=48)).timestamp())
        end_ts = int((start + timedelta(hours=2)).timestamp())
        url = (
            f"https://clob.polymarket.com/prices-history?market={token}"
            f"&startTs={start_ts}&endTs={end_ts}&fidelity=5"
        )
        save(f"polymarket-history-{event['slug']}.json", remote_get(url), url + REMOTE_NOTE)


def us_eastern_offset(local):
    """-4 h during US daylight saving time (second Sunday of March 02:00 to
    first Sunday of November 02:00), else -5 h."""

    def sunday(month, nth):
        first = datetime(local.year, month, 1)
        return first + timedelta(days=(6 - first.weekday()) % 7 + 7 * (nth - 1), hours=2)

    return timedelta(hours=-4 if sunday(3, 2) <= local < sunday(11, 1) else -5)


def kalshi_start(ticker):
    """`KXCS2GAME-26NOV250530CWGOSH-GOSH` -> 2026-11-25 05:30 US Eastern, in UTC."""
    local = datetime.strptime(ticker.split("-")[1][:11], "%y%b%d%H%M")
    return (local - us_eastern_offset(local)).replace(tzinfo=timezone.utc)


def kalshi(args):
    url = (
        "https://api.elections.kalshi.com/trade-api/v2/markets?series_ticker=KXCS2GAME"
        f"&status=settled&limit=200&min_close_ts={args.min_close}&max_close_ts={args.max_close}"
    )
    body = remote_get(url)
    save(f"kalshi-markets-KXCS2GAME-settled-{args.min_close}.json", body, url + REMOTE_NOTE)
    for market in json.loads(body)["markets"]:
        if args.event not in market.get("rules_primary", ""):
            continue
        start = kalshi_start(market["ticker"])
        start_ts = int((start - timedelta(hours=6)).timestamp())
        end_ts = int((start + timedelta(hours=2)).timestamp())
        url = (
            "https://api.elections.kalshi.com/trade-api/v2/series/KXCS2GAME/markets/"
            f"{market['ticker']}/candlesticks?start_ts={start_ts}&end_ts={end_ts}&period_interval=1"
        )
        save(f"kalshi-candles-{market['ticker']}.json", remote_get(url), url + REMOTE_NOTE)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--env", default=str(ROOT / ".env"))
    parser.add_argument("--ssh-host", help="user@host that runs curl for Polymarket and Kalshi")
    parser.add_argument("--ssh-key", help="path of the key for --ssh-host (never read here)")
    parser.add_argument("--known-hosts", help="UserKnownHostsFile for --ssh-host")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("stage-revisions").set_defaults(run=stage_revisions)
    p = sub.add_parser("event-revision")
    p.add_argument("title")
    p.add_argument("time", help="ISO instant; the revision is the last one before it")
    p.add_argument("name")
    p.set_defaults(run=event_revision)
    p = sub.add_parser("pandascore")
    p.add_argument("tournaments", nargs="+")
    p.set_defaults(run=pandascore)
    p = sub.add_parser("oddspapi-settled")
    for a in ("tournament", "name", "start", "end"):
        p.add_argument(a)
    p.set_defaults(run=oddspapi_settled)
    p = sub.add_parser("oddspapi-history")
    p.add_argument("--missing", action="store_true")
    p.set_defaults(run=oddspapi_history)
    sub.add_parser("trim-odds").set_defaults(run=trim_odds)
    sub.add_parser("account").set_defaults(run=account)
    p = sub.add_parser("polymarket")
    p.add_argument("tag", help="e.g. pgl-major-singapore-2026")
    p.set_defaults(run=polymarket)
    p = sub.add_parser("kalshi")
    p.add_argument("min_close", help="unix seconds")
    p.add_argument("max_close", help="unix seconds")
    p.add_argument("event", help="text of rules_primary that names the Event")
    p.set_defaults(run=kalshi)
    args = parser.parse_args()
    if args.command in ("polymarket", "kalshi"):
        REMOTE.update(host=args.ssh_host, key=args.ssh_key, known_hosts=args.known_hosts)
    if args.command in ("pandascore", "oddspapi-settled", "oddspapi-history", "account"):
        load_secrets(args.env)
    args.run(args)


if __name__ == "__main__":
    main()
