"""Tests for the brief assembler and its rule checks.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import briefing  # noqa: E402
import core  # noqa: E402
import registries  # noqa: E402
import util  # noqa: E402


class TestMarkdownSection(unittest.TestCase):
    def test_stops_at_next_heading(self) -> None:
        text = "## A\n\nbody a\n\n## B\n\nbody b\n"
        self.assertEqual(util.first_markdown_section(text, "A"), "body a")

    def test_a_quoted_heading_is_not_a_boundary(self) -> None:
        """session-goal.md quotes '## Owner direction' inside the contract text."""
        text = "## A\n\nbody one\n\n> ## Owner direction\n> quoted\n\nbody two\n\n## B\n\nx\n"
        got = util.first_markdown_section(text, "A")
        self.assertIn("body two", got)
        self.assertIn("> quoted", got)

    def test_indented_nested_bullets_do_not_end_the_section(self) -> None:
        text = "## A\n\nclause\n    - nested item\n\n## B\n\nx\n"
        self.assertIn("nested item", util.first_markdown_section(text, "A"))

    def test_fenced_code_keeps_headings(self) -> None:
        text = "## A\n\n```text\n## not a heading\n```\n\ntail\n\n## B\n"
        got = util.first_markdown_section(text, "A")
        self.assertIn("not a heading", got)
        self.assertIn("tail", got)


class TestBriefAssembly(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = core.HarnessPaths.discover(repo=REPO)
        cls.reg = registries.Registry.load(cls.paths)

    def test_pack_names_its_sources(self) -> None:
        pack = briefing.assemble(self.paths, self.reg, milestone="m1")
        self.assertIn("M1", pack.title)
        paths = [s.path for s in pack.sources]
        self.assertTrue(any("milestones.json" in p for p in paths))
        self.assertTrue(any("DECISIONS.md" in p for p in paths))
        self.assertGreater(pack.tokens, 500)

    def test_headline_and_rules_are_in_the_pack(self) -> None:
        text = briefing.assemble(self.paths, self.reg, milestone="m1").render()
        self.assertIn("37/232", text)
        self.assertIn("target 116", text)
        for rule in ("D21", "D22", "D23", "D24"):
            self.assertIn(rule, text, f"{rule} missing from the brief")

    def test_all_ten_contract_clauses_are_present(self) -> None:
        text = briefing.assemble(self.paths, self.reg, milestone="m1").render()
        for clause in ("1. **Your number.**", "10. **Delivery rules stay as they are**"):
            self.assertIn(clause, text)
        self.assertNotIn("…", text.split("## Decision")[0].split("## The deepseek")[0].rstrip()[-40:], "contract truncated")

    def test_mechanism_briefs_come_from_the_milestone(self) -> None:
        text = briefing.assemble(self.paths, self.reg, milestone="m1").render()
        for mech in self.reg.milestones.get("m1").mechanisms:
            self.assertIn(mech, text)

    def test_the_lab_prompt_matches_the_milestone_lab(self) -> None:
        text = briefing.assemble(self.paths, self.reg, milestone="m1").render()
        self.assertIn("deepseek lab", text.lower())
        self.assertNotIn("## The codex lab prompt", text)
        # the FINISH LINE clause is the part a session needs verbatim
        self.assertIn("FINISH LINE", text)

    def test_no_milestone_lists_every_milestone(self) -> None:
        pack = briefing.assemble(self.paths, self.reg)
        text = pack.render()
        for m in self.reg.milestones.all():
            self.assertIn(m.id, text)

    def test_overflow_is_reported_not_hidden(self) -> None:
        pack = briefing.assemble(self.paths, self.reg, milestone="m1", budget_tokens=10)
        self.assertTrue(pack.dropped)
        self.assertTrue(any("budget" in path for path, _ in pack.dropped))

    def test_missing_mechanism_is_reported_as_dropped(self) -> None:
        reg = registries.Registry.load(self.paths)
        reg.milestones.by_key["m1"].mechanisms = ["definitely-not-a-mechanism"]
        pack = briefing.assemble(self.paths, reg, milestone="m1")
        self.assertTrue(any("definitely-not-a-mechanism" in path for path, _ in pack.dropped))

    def test_files_route_skills_and_lessons(self) -> None:
        pack = briefing.assemble(self.paths, self.reg, milestone="m1", files=["scripts/harness/briefing.py"])
        self.assertIn("scripts/harness/briefing.py", pack.render())

    def test_render_lists_what_was_not_loaded(self) -> None:
        pack = briefing.assemble(self.paths, self.reg, milestone="m1", budget_tokens=10)
        self.assertIn("Not loaded, and why", pack.render())


if __name__ == "__main__":
    unittest.main()
