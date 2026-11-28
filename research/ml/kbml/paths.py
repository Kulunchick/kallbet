"""Where the read-only archive is and where derived caches go (never inside the repository)."""
import os
from pathlib import Path

ARCHIVE = Path(os.environ.get("KBML_ARCHIVE", "D:/Projects/KallBet/target/research-archive-t48-snapshot2"))
CACHE = Path(os.environ.get("KBML_CACHE", "D:/Projects/KallBet/target/ml-cache"))
#: Optional Match -> HLTV page links (a `links.tsv` with `panda_id`, `hltv_match_id`), replacing the
#: name-and-time linker (phase 5b: W5's recovered links). Use it with its own `KBML_CACHE`.
LINKS = os.environ.get("KBML_HLTV_LINKS") or None
REPO = Path(__file__).resolve().parents[3]
LAB_REPORT = REPO / "docs" / "lab" / "research" / "report.json"
