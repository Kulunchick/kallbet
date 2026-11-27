"""The Python anchor reproduces the Rust report on the research snapshot (skipped without it)."""
import pytest

from kbml import parity, paths


@pytest.mark.skipif(not paths.ARCHIVE.is_dir() or not paths.LAB_REPORT.is_file(), reason="research archive not present")
def test_anchor_parity_with_the_rust_report():
    assert parity.check(log=lambda *a: None)
