"""Closure tests for the harness itself.

These run the harness the way a user does - through the CLI, as a subprocess - so they
catch integration breakage that unit tests importing modules in-process cannot see:
a rule that crashes the engine, a command that exits nonzero on success, a module that
only works because of import order.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CLI = REPO / "scripts" / "harness" / "cli.py"
TIMEOUT = 120


def harness(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(CLI), *args],
        cwd=str(REPO),
        capture_output=True,
        text=True,
        timeout=TIMEOUT,
        check=False,
    )


class TestCliEntryPoints(unittest.TestCase):
    """Every read-only command must run and must not fail with an internal error."""

    def test_help_exits_zero_and_lists_commands(self) -> None:
        got = harness("--help")
        self.assertEqual(got.returncode, 0, got.stderr)
        for cmd in ("status", "brief", "skill", "team", "mistakes", "prep", "panel", "lint", "check"):
            self.assertIn(cmd, got.stdout)

    def test_skill_list(self) -> None:
        got = harness("skill", "list")
        self.assertEqual(got.returncode, 0, got.stderr)
        self.assertIn("skills in docs/harness/skills/", got.stdout)

    def test_team_roster(self) -> None:
        got = harness("team", "roster")
        self.assertEqual(got.returncode, 0, got.stderr)
        self.assertIn("dispatchable roles", got.stdout)

    def test_brief_runs_and_cites_sources(self) -> None:
        got = harness("brief", "--milestone", "m1")
        self.assertEqual(got.returncode, 0, got.stderr)
        self.assertIn("docs/harness/milestones.json", got.stdout)
        self.assertIn("Sources, in load order", got.stdout)

    def test_status_json_is_a_single_object(self) -> None:
        import json

        got = harness("status", "--json")
        self.assertEqual(got.returncode, 0, got.stderr)
        data = json.loads(got.stdout)
        self.assertIn("milestones", data)

    def test_check_json_reports_a_summary(self) -> None:
        import json

        got = harness("check", "--json")
        # nonzero is correct when rules fail; the JSON must still parse
        data = json.loads(got.stdout)
        self.assertIn("summary", data)
        self.assertIn("checked", data["summary"])
        self.assertGreater(data["summary"]["checked"], 0)

    def test_check_never_crashes_the_engine(self) -> None:
        got = harness("check")
        self.assertIn(got.returncode, (0, 1), f"engine crashed:\n{got.stdout}\n{got.stderr}")
        self.assertNotIn("Traceback", got.stdout)
        self.assertNotIn("Traceback", got.stderr)

    def test_json_is_accepted_before_and_after_the_subcommand(self) -> None:
        """Regression: a subparser --json default silently clobbered the root one."""
        import json

        for args in (
            ("status", "--json"),
            ("--json", "status"),
            ("git", "--json", "status"),
            ("budget", "--json", "show"),
        ):
            got = harness(*args)
            self.assertEqual(got.returncode, 0, f"{args}: {got.stderr}")
            try:
                json.loads(got.stdout)
            except json.JSONDecodeError as exc:
                self.fail(f"{args} did not print JSON: {exc}\n{got.stdout[:200]}")

    def test_check_only_accepts_both_flag_forms(self) -> None:
        """Regression: nargs='*' made `--only a --only b` drop `a` silently."""
        repeated = harness("check", "--only", "prep", "--only", "budget")
        single = harness("check", "--only", "prep", "budget")
        self.assertEqual(repeated.returncode, single.returncode)
        self.assertEqual(repeated.stdout, single.stdout)

    def test_unknown_milestone_is_a_usage_error_not_a_traceback(self) -> None:
        got = harness("brief", "--milestone", "nope")
        self.assertNotEqual(got.returncode, 0)
        self.assertNotIn("Traceback", got.stderr)

    def test_unknown_skill_is_reported(self) -> None:
        got = harness("skill", "show", "not-a-skill")
        self.assertNotEqual(got.returncode, 0)
        self.assertNotIn("Traceback", got.stderr)


class TestFullSuite(unittest.TestCase):
    """Run every other test module in this package, in-process.

    This must NOT shell out to `unittest discover`, because discovery would import this
    module too and recurse until the recursion limit. Enumerate the sibling modules
    explicitly and skip this one.
    """

    def test_every_sibling_module_passes(self) -> None:
        import importlib
        import unittest as ut

        here = Path(__file__).resolve().parent
        modules = sorted(
            p.stem for p in here.glob("test_*.py") if p.stem not in ("test_entrypoints", "__init__")
        )
        self.assertTrue(modules, "no sibling test modules found")
        loader = ut.TestLoader()
        suite = ut.TestSuite()
        for name in modules:
            mod = importlib.import_module(f"tests.harness.{name}")
            suite.addTests(loader.loadTestsFromModule(mod))
        result = ut.TextTestRunner(verbosity=1).run(suite)
        self.assertTrue(result.wasSuccessful(), f"{len(result.failures)} failures, {len(result.errors)} errors")


class TestNoSecretsLeak(unittest.TestCase):
    def test_cli_output_has_no_token_shapes(self) -> None:
        import re

        pattern = re.compile(r"gh[pousr]_[A-Za-z0-9]{8,}|sk-[A-Za-z0-9_\-]{8,}|rpa_[A-Za-z0-9]{8,}")
        for args in (("status", "--json"), ("team", "roster"), ("skill", "list"), ("budget", "show")):
            got = harness(*args)
            self.assertIsNone(pattern.search(got.stdout), f"{args} printed a token-shaped string")
            self.assertIsNone(pattern.search(got.stderr), f"{args} printed a token-shaped string")


if __name__ == "__main__":
    unittest.main()
