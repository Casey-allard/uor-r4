#!/usr/bin/env python3
"""Tests for scripts/harness/mistakes.py and the lesson registry.

    python3 -m unittest discover -s tests/harness -t . -v

Reads the real docs/harness/mistakes/ registry (so a broken lesson file fails the
suite) and a temporary copy for the write paths. No network, no `gh`.
"""

from __future__ import annotations

import argparse
import io
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import mistakes as M  # noqa: E402
from core import HarnessPaths  # noqa: E402
from registries import Registry, load_mistakes, match_mistakes  # noqa: E402

LESSON = """---
id: {id}
title: {title}
date: 2026-10-12
severity: warn
class: process
trigger: [alpha phrase, beta phrase]
paths: [docs/labs/**, scripts/harness/*.py]
milestones: [m1]
mechanism:
guard: none: test fixture
evidence: [docs/labs/README.md]
status: active
---

## What happened

A thing happened once.

## Symptom

You see it again as a symptom.

## Lesson

The transferable rule.

## How to apply

- Do the thing.
"""


class RegistryFixtures(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = HarnessPaths.discover(repo=REPO)
        cls.reg = Registry.load(cls.paths)

    def rules(self, rule_id: str) -> list:
        rule = next((r for r in M.register_rules() if r.id == rule_id), None)
        self.assertIsNotNone(rule, f"{rule_id} is not registered")
        return rule.check(self.paths, self.reg)


class TestRealRegistry(RegistryFixtures):
    def test_the_registry_is_not_empty(self) -> None:
        lessons = M.load_lessons(self.paths)
        self.assertGreaterEqual(len(lessons), 8, "the lesson registry should hold the mined lessons")
        # The cap exists so the registry stays CURATED rather than becoming a log, and it was 20
        # while the registry held only lessons. It now also holds `class: assumption` findings -
        # negative results kept so they are not re-derived - and those accumulate with the research
        # rather than with mistakes. Raised by exactly the two findings added, and stated here so the
        # next person to hit it knows it is a budget to re-examine rather than a number to nudge.
        self.assertLessEqual(
            len(lessons), 24, "the registry is a curated set, not a log: displace one to add one"
        )
        # The property the cap is proxying for: every entry names evidence, so nothing is in here on
        # assertion alone. A count cap without this would be a number that says nothing.
        for lesson in lessons:
            with self.subTest(entry=lesson.id):
                self.assertTrue(lesson.fm.get("evidence"), f"{lesson.id} names no evidence")
        ids = [l.id for l in lessons]
        self.assertEqual(len(ids), len(set(ids)), "duplicate lesson ids")
        for lesson in lessons:
            self.assertEqual(lesson.id, lesson.path.stem)
            self.assertIn(lesson.severity, M.SEVERITIES)
            self.assertIn(lesson.class_, M.CLASSES)

    def test_every_lesson_parses_with_no_structural_error(self) -> None:
        broken = []
        for lesson in M.load_lessons(self.paths):
            if lesson.errors:
                broken.append(f"{lesson.path.name}: {lesson.errors}")
        self.assertEqual(broken, [], "a lesson file that does not parse is a lesson nobody reads")

    def test_every_lesson_has_the_four_sections(self) -> None:
        for lesson in M.load_lessons(self.paths):
            self.assertNotIn("body is missing", "\n".join(lesson.warnings), f"{lesson.id}: {lesson.warnings}")
            for section in M.SECTIONS:
                self.assertIn(f"## {section}", lesson.body, lesson.id)

    def test_registries_mistake_view_reads_lesson_and_symptom(self) -> None:
        """registries.Mistake.lesson/.symptom must be non-empty on every lesson."""
        loaded = load_mistakes(self.paths)
        self.assertEqual(len(loaded), len(M.load_lessons(self.paths)))
        for mistake in loaded:
            self.assertTrue(mistake.lesson, f"{mistake.id} has no Lesson section text")
            self.assertTrue(mistake.symptom, f"{mistake.id} has no Symptom section text")
            self.assertEqual(mistake.severity, mistake.severity.lower())

    def test_every_evidence_path_exists(self) -> None:
        """The task's hard requirement: a lesson cites records that really exist."""
        missing: list[str] = []
        for lesson in M.load_lessons(self.paths):
            for value in lesson.evidence:
                if M.resolve_path(self.paths, value) is None:
                    missing.append(f"{lesson.id}: {value}")
        self.assertEqual(missing, [])

    def test_registry_valid_rule_passes(self) -> None:
        problems = [r for r in self.rules("mistakes.registry_valid") if not r.ok]
        self.assertEqual([p.message for p in problems], [])

    def test_evidence_exists_rule_passes(self) -> None:
        problems = [r for r in self.rules("mistakes.evidence_exists") if not r.ok]
        self.assertEqual([p.message for p in problems], [])

    def test_guard_covered_rule_passes(self) -> None:
        """Every block lesson names a guard that exists, or explains why none can."""
        problems = [r for r in self.rules("mistakes.guard_covered") if not r.ok]
        self.assertEqual([p.message for p in problems], [])

    def test_every_block_lesson_names_a_real_guard_or_a_reason(self) -> None:
        known = M._known_rule_ids()
        checked = 0
        for lesson in M.load_lessons(self.paths):
            if lesson.severity != "block":
                continue
            checked += 1
            if lesson.guard.startswith("none"):
                self.assertGreater(len(lesson.guard.split(":", 1)[1].strip()), 4, lesson.guard)
            else:
                self.assertIn(lesson.guard, known, f"{lesson.id} cites an unknown guard")
        self.assertGreater(checked, 0, "no block lessons: the registry has lost its teeth")

    def test_stale_check_rule_passes(self) -> None:
        problems = [r for r in self.rules("mistakes.stale_check") if not r.ok]
        self.assertEqual([p.message for p in problems], [])

    def test_a_lesson_bites_on_the_file_it_names(self) -> None:
        """`mistakes find -f scripts/harness/prlint.py` must return something relevant."""
        matches = M.render_matches(self.reg, files=["scripts/harness/prlint.py"])
        ids = {m["id"] for m in matches}
        self.assertIn("preregistration-missing-or-after-the-fact", ids)
        self.assertIn("merged-prs-without-a-headline-move", ids)
        for m in matches:
            self.assertTrue(m["why"])

    def test_trigger_recalls_by_query(self) -> None:
        matches = M.render_matches(self.reg, query="host fallback step time")
        ids = {m["id"] for m in matches}
        self.assertIn("host-fallback-is-a-defect", ids)
        self.assertTrue(M.format_matches(matches).strip())

    def test_milestone_recalls_by_milestone(self) -> None:
        matches = M.render_matches(self.reg, milestone="m4")
        self.assertTrue(matches, "no lesson is attached to m4")

    def test_render_index_lists_every_lesson(self) -> None:
        text = M.render_index(self.reg.mistakes)
        for mistake in self.reg.mistakes:
            self.assertIn(mistake.id, text)
        self.assertIn("block", text)

    def test_no_match_is_an_answer(self) -> None:
        self.assertEqual(M.render_matches(self.reg, files=["nothing/here.txt"]), [])
        self.assertIn("no lesson matches", M.format_matches([]))


class TestParser(unittest.TestCase):
    def _lesson(self, text: str, name: str = "fixture.md") -> M.Lesson:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / name
            path.write_text(text, encoding="utf-8")
            return M.read_lesson(path)

    def test_a_real_lesson_parses(self) -> None:
        raw = (REPO / "docs" / "harness" / "mistakes" / "host-fallback-is-a-defect.md").read_text(encoding="utf-8")
        lesson = self._lesson(raw, "host-fallback-is-a-defect.md")
        self.assertEqual(lesson.errors, [])
        self.assertEqual(lesson.severity, "block")
        self.assertIn("host fallback", lesson.trigger)
        self.assertTrue(lesson.evidence)

    def test_inline_and_block_lists_parse(self) -> None:
        inline = LESSON.format(id="fixture", title="Inline lists")
        lesson = self._lesson(inline)
        self.assertEqual(lesson.errors, [])
        self.assertEqual(lesson.trigger, ["alpha phrase", "beta phrase"])
        self.assertEqual(lesson.paths, ["docs/labs/**", "scripts/harness/*.py"])

        block = inline.replace(
            "trigger: [alpha phrase, beta phrase]",
            "trigger:\n  - alpha phrase\n  - beta phrase",
        ).replace("paths: [docs/labs/**, scripts/harness/*.py]", "paths:\n  - docs/labs/**\n  - scripts/harness/*.py")
        lesson = self._lesson(block)
        self.assertEqual(lesson.errors, [])
        self.assertEqual(lesson.trigger, ["alpha phrase", "beta phrase"])
        self.assertEqual(len(lesson.paths), 2)

    def test_missing_front_matter_is_an_error(self) -> None:
        lesson = self._lesson("## Lesson\n\nno front matter here\n")
        self.assertTrue(any("front matter" in e for e in lesson.errors))

    def test_id_must_equal_the_file_stem(self) -> None:
        lesson = self._lesson(LESSON.format(id="other-id", title="x"), name="fixture.md")
        self.assertTrue(any("file stem" in e for e in lesson.errors))

    def test_bad_enum_values_are_errors(self) -> None:
        bad = LESSON.format(id="fixture", title="x").replace("severity: warn", "severity: fatal")
        self.assertTrue(any("severity" in e for e in self._lesson(bad).errors))
        bad = LESSON.format(id="fixture", title="x").replace("class: process", "class: vibes")
        self.assertTrue(any("class" in e for e in self._lesson(bad).errors))
        bad = LESSON.format(id="fixture", title="x").replace("date: 2026-10-12", "date: yesterday")
        self.assertTrue(any("date" in e for e in self._lesson(bad).errors))

    def test_guard_must_be_dotted_or_none(self) -> None:
        bad = LESSON.format(id="fixture", title="x").replace("guard: none: test fixture", "guard: somewhere")
        self.assertTrue(any("guard" in e for e in self._lesson(bad).errors))

    def test_a_lesson_needs_two_triggers(self) -> None:
        bad = LESSON.format(id="fixture", title="x").replace("trigger: [alpha phrase, beta phrase]", "trigger: [only-one]")
        self.assertTrue(any("trigger" in e for e in self._lesson(bad).errors))

    def test_missing_sections_warn_but_do_not_error(self) -> None:
        thin = LESSON.format(id="fixture", title="x").split("## What happened")[0] + "## What happened\n\nonly this.\n"
        lesson = self._lesson(thin)
        self.assertEqual(lesson.errors, [])
        self.assertTrue(any("Symptom" in w for w in lesson.warnings))
        self.assertTrue(any("Lesson" in w for w in lesson.warnings))


class TestCapture(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name)
        (self.repo / "docs" / "labs").mkdir(parents=True)
        (self.repo / "docs" / "harness" / "mistakes").mkdir(parents=True)
        shutil.copy(REPO / "docs" / "labs" / "README.md", self.repo / "docs" / "labs" / "README.md")
        self.paths = HarnessPaths.discover(repo=self.repo)
        real = HarnessPaths.discover(repo=REPO)
        self.reg = Registry(
            paths=self.paths,
            skills=[],
            roles=[],
            mistakes=[],
            milestones=Registry.load(real).milestones,
            panels=[],
            decisions=[],
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def args(self, **kw) -> argparse.Namespace:
        base = dict(
            action="capture",
            id="captured-lesson",
            title="A captured lesson",
            severity="warn",
            class_="process",
            trigger=["alpha phrase", "beta phrase"],
            paths=["docs/labs/**"],
            milestones=["m1"],
            mechanism=None,
            guard=None,
            evidence=["docs/labs/README.md"],
            symptom="It happened again.",
            lesson="Do not do it again.",
            force=False,
            json=False,
        )
        base.update(kw)
        return argparse.Namespace(**base)

    def test_capture_writes_a_valid_lesson(self) -> None:
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = M.capture(self.args(), self.paths, self.reg)
        self.assertEqual(code, 0, buf.getvalue())
        target = self.paths.mistakes_dir / "captured-lesson.md"
        self.assertTrue(target.exists())
        lesson = M.read_lesson(target)
        self.assertEqual(lesson.errors, [])
        self.assertEqual(lesson.id, "captured-lesson")
        self.assertTrue(lesson.trigger)
        self.assertTrue(lesson.evidence)
        self.assertTrue(lesson.guard.startswith("none"))
        self.assertIn("Do not do it again.", lesson.body)
        self.assertIn("## How to apply", lesson.body)

    def test_capture_refuses_to_overwrite_without_force(self) -> None:
        with redirect_stdout(io.StringIO()):
            self.assertEqual(M.capture(self.args(), self.paths, self.reg), 0)
        err = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(err):
            code = M.capture(self.args(title="Different title"), self.paths, self.reg)
        self.assertEqual(code, 1)
        self.assertIn("--force", err.getvalue())
        self.assertIn("A captured lesson", (self.paths.mistakes_dir / "captured-lesson.md").read_text())
        # --force rewrites it
        with redirect_stdout(io.StringIO()):
            code = M.capture(self.args(title="Forced title", force=True), self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("Forced title", (self.paths.mistakes_dir / "captured-lesson.md").read_text())

    def test_capture_refuses_a_bad_id(self) -> None:
        err = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(err):
            code = M.capture(self.args(id="Not An Id"), self.paths, self.reg)
        self.assertEqual(code, 2)
        self.assertIn("kebab-case", err.getvalue())

    def test_capture_refuses_an_unknown_evidence_path(self) -> None:
        err = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(err):
            code = M.capture(self.args(evidence=["docs/labs/does-not-exist-2099/README.md"]), self.paths, self.reg)
        self.assertEqual(code, 1, "a lesson whose evidence does not exist must not be written")
        self.assertIn("evidence", err.getvalue())
        self.assertFalse((self.paths.mistakes_dir / "captured-lesson.md").exists())

    def test_capture_output_is_registry_readable(self) -> None:
        text = M.build_lesson_text(
            lesson_id="round-trip",
            title="Round trip",
            evidence=["docs/labs/README.md"],
            paths=["docs/labs/**"],
            trigger=["one phrase", "two phrase"],
        )
        (self.paths.mistakes_dir / "round-trip.md").write_text(text, encoding="utf-8")
        lesson = M.read_lesson(self.paths.mistakes_dir / "round-trip.md")
        self.assertEqual(lesson.errors, [])
        loaded = load_mistakes(self.paths)
        self.assertEqual([m.id for m in loaded], ["round-trip"])
        self.assertTrue(loaded[0].lesson)
        self.assertTrue(loaded[0].symptom)

    def test_placeholders_still_pass_registry_valid(self) -> None:
        """A capture with nothing but flags must not write an invalid file."""
        text = M.build_lesson_text(lesson_id="bare", title="Bare capture")
        (self.paths.mistakes_dir / "bare.md").write_text(text, encoding="utf-8")
        self.assertEqual(M.read_lesson(self.paths.mistakes_dir / "bare.md").errors, [])
        self.assertEqual([r.message for r in M._check_registry_valid(self.paths, self.reg) if not r.ok], [])

    def test_written_lesson_matches_the_registry_rules(self) -> None:
        with redirect_stdout(io.StringIO()):
            M.capture(self.args(), self.paths, self.reg)
        for rule in M.register_rules():
            if rule.id == "mistakes.stale_check":
                continue  # the temp repo has no full tree for the glob check
            failing = [r.message for r in rule.check(self.paths, self.reg) if not r.ok]
            self.assertEqual(failing, [], f"{rule.id}: {failing}")


class TestModuleApi(RegistryFixtures):
    def test_contract_api(self) -> None:
        rules = M.register_rules()
        self.assertEqual({r.id for r in rules}, set(M.LOGICAL_RULES))
        for rule in rules:
            self.assertIn(rule.severity, ("error", "warn", "info"))
            self.assertTrue(callable(rule.check))

    def test_main_list_find_show_capture(self) -> None:
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = M.main(argparse.Namespace(action="list", json=False), self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("lessons in docs/harness/mistakes/", buf.getvalue())

        buf = io.StringIO()
        with redirect_stdout(buf):
            code = M.main(argparse.Namespace(action="find", files=["scripts/harness/prlint.py"], query="", milestone=None, json=False), self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("preregistration-missing-or-after-the-fact", buf.getvalue())

        buf = io.StringIO()
        with redirect_stdout(buf):
            code = M.main(argparse.Namespace(action="show", id="host-fallback-is-a-defect"), self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("## How to apply", buf.getvalue())

        err = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(err):
            code = M.main(argparse.Namespace(action="show", id="nope"), self.paths, self.reg)
        self.assertEqual(code, 2)

    def test_json_output_is_serialisable(self) -> None:
        import json

        for action in ("list", "find"):
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = M.main(argparse.Namespace(action=action, json=True, files=[], query="", milestone=None), self.paths, self.reg)
            self.assertEqual(code, 0)
            json.loads(buf.getvalue())


class TestAssumptionClass(unittest.TestCase):
    """`assumption` is a valid class: a belief that turned out false, not a mistake that was made.

    These entries are kept for the opposite reason to a lesson - so the next lab does not re-derive
    them. Without a class of their own they would either be filed as mistakes (wrong, and it would
    mislabel correct research) or not filed at all, which is how a negative result gets redone.
    """

    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = HarnessPaths.discover(repo=REPO)

    def test_assumption_is_a_valid_class(self) -> None:
        self.assertIn("assumption", M.CLASSES)

    def test_the_assumption_entries_validate(self) -> None:
        found = [l for l in M.load_lessons(self.paths) if l.class_ == "assumption"]
        self.assertTrue(found, "the mesh and portability findings are recorded as assumptions")
        for lesson in found:
            with self.subTest(entry=lesson.id):
                self.assertEqual(lesson.errors, [], f"{lesson.id} is invalid")
                self.assertNotIn("body is missing", "\n".join(lesson.warnings), f"{lesson.id}")

    def test_a_negative_finding_carries_no_guard(self) -> None:
        """A negative finding has nothing to intercept. Inventing a rule for one would be a rule that
        checks nothing, so `guard: none: <reason>` is correct and must stay accepted."""
        for lesson in M.load_lessons(self.paths):
            if lesson.class_ != "assumption":
                continue
            with self.subTest(entry=lesson.id):
                guard = str(lesson.fm.get("guard", ""))
                self.assertTrue(
                    guard.startswith("none:"),
                    f"{lesson.id} guard {guard!r}: a negative finding has nothing to intercept",
                )

    def test_the_mesh_finding_is_retrievable_by_its_own_question(self) -> None:
        """The property the entry exists for: a lab asking about the mesh finds the answer rather than
        running the search again."""
        reg = Registry.load(self.paths)
        hits = match_mistakes(reg.mistakes, query="connect to the mesh")
        self.assertTrue(
            any(m.id == "mesh-is-not-a-coordination-concept" for m, _ in hits),
            f"got {[m.id for m, _ in hits]}",
        )

    def test_the_mesh_finding_is_retrievable_by_path(self) -> None:
        reg = Registry.load(self.paths)
        hits = match_mistakes(reg.mistakes, files=["docs/harness/README.md"])
        self.assertTrue(any(m.id == "mesh-is-not-a-coordination-concept" for m, _ in hits))


if __name__ == "__main__":
    unittest.main()
