"""The real-League report leaks no personal data (runs only where the production backup exists)."""
from pathlib import Path

import pytest

from kbml import league_real as L

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.skipif(not L.DB.exists(), reason="production backup not present")
def test_report_and_repository_hold_no_member_identity():
    c = L.connect()
    rows = c.execute("select nickname, telegram_id, coalesce(avatar_url, '') from users").fetchall()
    c.close()
    texts = [p.read_text(encoding="utf-8", errors="ignore").lower() for p in
             [ROOT / "CATCHUP.md", *sorted((ROOT / "kbml").glob("*.py")), *sorted((ROOT / "tests").glob("*.py"))]]
    for nick, tg, avatar in rows:
        for t in texts:
            assert len(nick) < 3 or f" {nick.lower()} " not in t and f"| {nick.lower()} |" not in t
            assert len(str(abs(tg))) < 6 or str(tg) not in t
            assert not avatar or avatar.lower() not in t


def test_database_is_never_inside_the_repository():
    assert not any(ROOT.rglob("*.db"))
