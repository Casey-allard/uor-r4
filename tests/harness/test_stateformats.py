"""Tests for the frozen state-format contract and its rule.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "scripts" / "harness"))


class TestCannotRunIsNotAPass(unittest.TestCase):
    """A check that cannot run must not report as a pass.

    `harness.state_formats` returned `info` with `ok=True` when the Rust binary was not built, so a
    machine without it got a **green `check`** on the one rule whose whole job is to notice that
    persisted state drifted from its frozen contract. The Rust port cannot enter that state (the rule
    IS the binary), so this fixes the Python half.
    """

    def test_a_missing_binary_reports_skipped_rather_than_ok(self):
        import stateformats

        repo = pathlib.Path(tempfile.mkdtemp())
        status, detail = stateformats.check_contract(repo)
        self.assertEqual(status, "skipped", detail)
        self.assertIn("did not run", detail)

    def test_the_rule_reports_a_warning_not_a_green_line(self):
        """**Warn rather than error** - a lab that has not built the binary is not violating
        anything, and a permanently-red check gets ignored. What changes is that it is VISIBLE."""
        import stateformats

        repo = pathlib.Path(tempfile.mkdtemp())
        original = stateformats.check_contract
        try:
            stateformats.check_contract = lambda _repo, timeout=120: ("skipped", "not built here")
            rule = next(
                r for r in stateformats.register_rules() if r.id == "harness.state_formats"
            )
            from core import HarnessPaths

            paths = HarnessPaths(repo=repo, harness=repo / "docs/harness", state=repo / ".harness")
            results = rule.check(paths, None)
            self.assertEqual(len(results), 1)
            self.assertFalse(results[0].ok, "a check that did not run is not ok")
            self.assertEqual(results[0].severity, "warn")
            self.assertIn("NOT checked", results[0].message)
        finally:
            stateformats.check_contract = original
