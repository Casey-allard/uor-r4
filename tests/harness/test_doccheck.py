"""Tests for doccheck: the commands the docs tell labs to run must exist.

Every case here is a documentation error that actually shipped, or a shape the docs genuinely use.
The point of the file is the first group: without them, a checker that passes everything would look
identical to a checker that works.
"""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve()
HARNESS_DIR = HERE.parents[2] / "scripts" / "harness"
if str(HARNESS_DIR) not in sys.path:
    sys.path.insert(0, str(HARNESS_DIR))

#: doccheck loads the CLI's parser itself, so import it as a module rather than importing cli.
_spec = importlib.util.spec_from_file_location("_doccheck_under_test", HARNESS_DIR / "doccheck.py")
assert _spec and _spec.loader
doccheck = importlib.util.module_from_spec(_spec)
sys.modules["_doccheck_under_test"] = doccheck
_spec.loader.exec_module(doccheck)

REPO = HERE.parents[2]

#: (line of documentation, should the command be accepted)
REAL_ERRORS = [
    # 1. a flag that never existed. Caught by hand while writing a record.
    ("python3 scripts/harness/cli.py heartbeat -m m1 --dry-run", False),
    # 2. a top-level command that is really a subcommand. Also caught by hand.
    ("python3 scripts/harness/cli.py install-hooks", False),
    # 3. a subcommand that does not exist. Caught by hand, in the onboarding prompt.
    ("python3 scripts/harness/cli.py team list", False),
]

VALID_SHAPES = [
    ("python3 scripts/harness/cli.py heartbeat -m m1 --print", True),
    ("python3 scripts/harness/cli.py git install-hooks", True),
    ("python3 scripts/harness/cli.py team roster", True),
    ("python3 scripts/harness/cli.py git clean-merged --apply", True),
    ("python3 scripts/harness/cli.py check --repo .", True),
    ("python3 scripts/harness/cli.py --repo <repo> multilab presence", True),
    ('python3 scripts/harness/cli.py --repo "$(git rev-parse --show-toplevel)" check', True),
    ("python3 scripts/harness/cli.py prep new -m m1", True),
    ("python3 scripts/harness/cli.py mistakes find -f a.rs b.rs", True),
    ("python3 scripts/harness/cli.py skill use uor-research-architect --outcome started", True),
    ("python3 scripts/harness/cli.py multilab presence", True),
    ("python3 scripts/harness/cli.py nonexistent-command", False),
    ("python3 scripts/harness/cli.py git nosuchsub", False),
]


class TestDocumentedCommands(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.parser = doccheck._build_parser()

    def _accepts(self, line: str) -> bool:
        rows = doccheck.commands_in_text(line, self.parser)
        return bool(rows) and all(r["ok"] for r in rows)

    def test_it_catches_every_documentation_error_that_shipped(self) -> None:
        """The reason this module exists. A checker that passes everything is indistinguishable
        from one that works, so these cases are the load-bearing ones."""
        for line, should_pass in REAL_ERRORS:
            with self.subTest(line=line):
                self.assertFalse(
                    self._accepts(line),
                    f"doccheck accepted {line!r}, which is a command that does not exist",
                )

    def test_it_accepts_every_shape_the_docs_actually_use(self) -> None:
        """False positives are how a checker gets ignored, so the valid shapes are tested too."""
        for line, should_pass in VALID_SHAPES:
            with self.subTest(line=line):
                self.assertEqual(self._accepts(line), should_pass, line)

    def test_a_top_level_command_that_is_really_a_subcommand_is_caught(self) -> None:
        """This exact error slipped past the first version, because `git` was in the skip list and
        the walk stopped before reaching `install-hooks`. It is the error the check exists for."""
        self.assertFalse(self._accepts("cli.py install-hooks"))
        self.assertTrue(self._accepts("cli.py git install-hooks"))

    def test_a_shell_substitution_is_a_value_not_a_command(self) -> None:
        """`$(git rev-parse --show-toplevel)` contains spaces; splitting it naively invents command
        words and the whole line is misparsed."""
        self.assertTrue(self._accepts('cli.py --repo "$(git rev-parse --show-toplevel)" check'))

    def test_global_flags_are_accepted_before_and_after_the_subcommand(self) -> None:
        """argparse only takes `--repo` first, but `cli.py check --repo .` is what the docs write and
        it works. The check tests that the flag EXISTS, not where it was placed."""
        self.assertTrue(self._accepts("cli.py check --repo ."))
        self.assertTrue(self._accepts("cli.py --repo . check"))

    def test_a_bare_flag_with_no_command_is_ignored(self) -> None:
        """`cli.py --lab kimi` names no subcommand, so there is nothing to verify."""
        self.assertEqual(doccheck.commands_in_text("export UOR_LAB=kimi  # or: cli.py --lab kimi", self.parser), [])


class TestTheRealDocs(unittest.TestCase):
    """The docs in this repository. This is what `check` runs."""

    def test_every_documented_command_exists(self) -> None:
        rows = doccheck.documented_commands(REPO)
        self.assertGreater(len(rows), 20, f"only found {len(rows)} commands; the scan may be broken")
        bad = [r for r in rows if not r["ok"]]
        self.assertEqual(
            bad,
            [],
            "documented commands that do not exist:\n"
            + "\n".join(f"  {r['doc']}:{r['line']} {r['command']} ({r['problem']})" for r in bad),
        )

    def test_the_scan_covers_the_files_that_matter(self) -> None:
        """AGENTS.md is the one file every lab reads; a wrong command there costs the most."""
        rows = doccheck.documented_commands(REPO)
        docs = {r["doc"] for r in rows}
        self.assertIn("AGENTS.md", docs)
        self.assertIn("docs/harness/ONBOARD-PROMPT.md", docs)

    def test_it_returns_a_nonzero_exit_when_a_command_is_missing(self) -> None:
        self.assertEqual(doccheck.check(json_out=True, repo=REPO), 0)


if __name__ == "__main__":
    unittest.main()
