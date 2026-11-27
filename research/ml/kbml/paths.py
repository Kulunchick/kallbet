"""Where the read-only archive is and where derived caches go (never inside the repository)."""
import os
from pathlib import Path

ARCHIVE = Path(os.environ.get("KBML_ARCHIVE", "D:/Projects/KallBet/target/research-archive-t48-snapshot2"))
CACHE = Path(os.environ.get("KBML_CACHE", "D:/Projects/KallBet/target/ml-cache"))
REPO = Path(__file__).resolve().parents[3]
LAB_REPORT = REPO / "docs" / "lab" / "research" / "report.json"
