"""Instants as integer microseconds since the Unix epoch (UTC): exact, fast to compare."""
import re
from datetime import datetime, timezone

US = 1_000_000
MINUTE = 60 * US
HOUR = 60 * MINUTE
DAY = 24 * HOUR

_FRAC = re.compile(r"\.(\d+)")


def parse(value) -> int | None:
    """An RFC 3339 string (as chrono parses it) to microseconds; None if absent or unreadable."""
    if not isinstance(value, str) or not value:
        return None
    s = value.strip()
    if s.endswith("Z") or s.endswith("z"):
        s = s[:-1] + "+00:00"
    m = _FRAC.search(s)
    if m:
        s = s[: m.start()] + "." + (m.group(1) + "000000")[:6] + s[m.end():]
    try:
        dt = datetime.fromisoformat(s)
    except ValueError:
        return None
    if dt.tzinfo is None:
        return None
    return to_us(dt)


def to_us(dt: datetime) -> int:
    delta = dt - datetime(1970, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * US + delta.microseconds


def from_seconds(s: int) -> int:
    return int(s) * US


def iso(us: int | None) -> str | None:
    if us is None:
        return None
    sec, micro = divmod(us, US)
    dt = datetime.fromtimestamp(sec, tz=timezone.utc).replace(microsecond=micro)
    return dt.strftime("%Y-%m-%dT%H:%M:%S") + (f".{micro:06d}" if micro else "") + "Z"


SPEC_PUBLISHED = parse("2026-11-25T23:46:46Z")
HARD_LOCK_LEAD = 15 * MINUTE
