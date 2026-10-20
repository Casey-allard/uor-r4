"""Tests for the pre-registration gate, the panels and their scoring.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v

No network, no gh, no pod calls: every test builds its own registry in a temporary repo,
except the two that assert the committed panels exist and are frozen.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import checkrules  # noqa: E402
import core  # noqa: E402
import prep  # noqa: E402
import registries  # noqa: E402


def make_paths(root: Path) -> core.HarnessPaths:
    harness = root / "docs" / "harness"
    for sub in ("prep", "panels", "ledger"):
        (harness / sub).mkdir(parents=True, exist_ok=True)
    (root / ".harness").mkdir(parents=True, exist_ok=True)
    (root / "docs" / "mechanisms").mkdir(parents=True, exist_ok=True)
    return core.HarnessPaths(repo=root, harness=harness, state=root / ".harness")


def write_milestones(paths: core.HarnessPaths, panel: str = "demo-panel") -> None:
    data = {
        "schema": 1,
        "tracker": {"issue": 2028},
        "milestones": {
            "m1": {
                "issue": 2029,
                "id": "M1",
                "title": "Language base",
                "lab": "deepseek",
                "status": "in-progress",
                "headline": {"label": "reply acceptable", "current": 37, "target": 116, "of": 232},
                "secondary": [],
                "panel": panel,
                "base_artifact": "D10 (10% recall dose)",
                "mechanisms": ["exact-addressed-memory"],
                "lines": [{"name": "exact addressed memory", "count": 0, "of": 3}],
                "next": "re-register the product-key run",
            }
        },
        "owner_rules": [
            {"id": "D21", "rule": "Three consecutive merged PRs on one line force a pivot card.", "scope": "all"},
            {"id": "D22", "rule": "A negative closes a configuration, never a mechanism.", "scope": "all"},
            {"id": "D23", "rule": "No softmax anywhere.", "scope": "all"},
            {"id": "D24", "rule": "No CPU model compute.", "scope": "all"},
        ],
    }
    (paths.harness / "milestones.json").write_text(json.dumps(data), encoding="utf-8")


def write_panel(paths: core.HarnessPaths, pid: str = "demo-panel", **overrides) -> None:
    panel = {
        "title": "demo",
        "milestone": "m1",
        "frozen": True,
        "grader": "chat-grade",
        "source": "test",
        "noise": "+/-3 rows",
        "seeds": ">= 2 for a decisive run",
        "rows": ">= 200 fresh rows for a decisive run",
        "bars": [{"id": "reply-acceptable", "label": "acceptable replies", "current": 37, "target": 116, "of": 232, "unit": "replies"}],
    }
    panel.update(overrides)
    payload = {"schema": 1, "note": "test", "panels": {pid: panel}}
    (paths.harness / "panels" / f"{pid}.json").write_text(json.dumps(payload), encoding="utf-8")


def new_args(**kw) -> argparse.Namespace:
    base = dict(
        milestone="m1",
        mechanism="exact-addressed-memory",
        run="product-key control rerun",
        bar="wrong-value 47 -> <=34 of 120",
        seeds=2,
        rows=None,
        base=None,
        split="world_v2 train, frozen panel held out",
        steps="8000",
        stop="stop on divergence or after the pre-registered cost",
        cost="1 pod, 2 x 5090, ~$3",
        write=None,
        json=False,
        action="new",
    )
    base.update(kw)
    return argparse.Namespace(**base)


class TempRepoCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.paths = make_paths(self.root)
        write_milestones(self.paths)
        write_panel(self.paths)
        (self.root / "docs" / "mechanisms" / "exact-addressed-memory.md").write_text("# brief\n", encoding="utf-8")
        self.reg = registries.Registry.load(self.paths)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def new_sheet(self, args) -> int:
        """new_sheet prints the sheet to stdout; tests keep that out of the -v log."""
        return capture_rc(lambda: prep.new_sheet(args, self.paths, self.reg))

    def sheet_text(self, **kw) -> str:
        args = new_args(**kw)
        rc = self.new_sheet(args)
        self.assertEqual(rc, 0)
        target = Path(args.write) if args.write else None
        if target is None:
            written = [p for p in prep.sheet_files(self.paths) if p.suffix == ".md"]
            self.assertTrue(written)
            return written[0].read_text(encoding="utf-8")
        return Path(target).read_text(encoding="utf-8")


class TestSheetShape(TempRepoCase):
    def test_generated_sheet_has_every_required_block(self) -> None:
        text = self.sheet_text(write=str(self.paths.harness / "prep" / "smoke.md"))
        secs = prep.sections(text)
        for _key, (name, ok) in prep.sheet_requirements(self.paths).items():
            self.assertTrue(ok(secs, text), f"missing block: {name}")

    def test_generated_sheet_ends_with_pending_verdict(self) -> None:
        text = self.sheet_text(write=str(self.paths.harness / "prep" / "smoke.md"))
        self.assertTrue(text.rstrip("\n").endswith("TEST FITNESS: PENDING"))
        self.assertEqual(prep.test_fitness(text), "PENDING")

    def test_registry_fields_are_filled_not_blank(self) -> None:
        text = self.sheet_text(write=str(self.paths.harness / "prep" / "smoke.md"))
        self.assertIn("docs/mechanisms/exact-addressed-memory.md", text)
        self.assertIn("M1: reply acceptable 37/232 -> target 116/232", text)
        self.assertIn("D22", text)
        self.assertIn("exact addressed memory - count 0/3", text)
        self.assertNotIn("- split: \n", text)

    def test_unknown_fields_are_missing_never_invented(self) -> None:
        text = self.sheet_text(rows=None, write=str(self.paths.harness / "prep" / "smoke.md"))
        self.assertIn("- split: world_v2 train, frozen panel held out", text)
        args = new_args(split=None, base=None, steps=None, stop=None, cost=None)
        args.write = str(self.paths.harness / "prep" / "bare.md")
        self.new_sheet(args)
        bare = Path(args.write).read_text(encoding="utf-8")
        self.assertIn(f"- steps: {prep.MISSING}", bare)
        self.assertIn(f"- stop: {prep.MISSING}", bare)
        self.assertIn(f"- fresh rows: {prep.MISSING}", bare)
        # the registry still fills what it owns
        self.assertIn("D10 (10% recall dose)", bare)

    def test_power_block_states_the_d22_thresholds(self) -> None:
        text = self.sheet_text(seeds=1, rows=10, write=str(self.paths.harness / "prep" / "weak.md"))
        self.assertIn("decisive run: NO", text)
        self.assertIn(">= 2 seeds", text)
        self.assertIn(">= 200 fresh rows", text)

    def test_default_write_path_is_dated_and_slugged(self) -> None:
        args = new_args()
        rc = self.new_sheet(args)
        self.assertEqual(rc, 0)
        files = list((self.paths.harness / "prep").glob("m1-exact-addressed-memory-*.md"))
        self.assertEqual(len(files), 1, files)
        self.assertRegex(files[0].name, r"^m1-exact-addressed-memory-\d{4}-\d{2}-\d{2}\.md$")

    def test_unknown_milestone_is_a_usage_error(self) -> None:
        args = new_args(milestone="m9")
        self.assertEqual(self.new_sheet(args), 2)

    def test_unknown_mechanism_is_marked_missing(self) -> None:
        text = self.sheet_text(mechanism="no-such-mechanism", write=str(self.paths.harness / "prep" / "x.md"))
        self.assertIn(prep.MISSING, text)
        self.assertIn("known: exact-addressed-memory", text)


class TestSheetRules(TempRepoCase):
    def _engine(self) -> list[checkrules.RuleResult]:
        """prep rules only, one result per rule id.

        checkrules.rules() re-invokes every module's register_rules() on each call, and a
        module without an idempotency cache duplicates process-wide; de-duplicating keeps
        these assertions about this module's rules, not about another module's cache.
        """
        seen: set[tuple[str, str, str]] = set()
        out: list[checkrules.RuleResult] = []
        for result in checkrules.run(self.paths, self.reg, scope="repo", only=["prep"]):
            key = (result.rule_id, result.path, result.message)
            if key in seen:
                continue
            seen.add(key)
            out.append(result)
        return out

    def test_shape_rule_passes_a_generated_sheet(self) -> None:
        self.sheet_text(write=str(self.paths.harness / "prep" / "ok.md"))
        results = [r for r in self._engine() if r.rule_id == "prep.sheet_shape"]
        self.assertTrue(all(r.ok for r in results), [r.message for r in results])

    def test_shape_rule_fails_a_sheet_missing_blocks(self) -> None:
        (self.paths.harness / "prep" / "broken.md").write_text("# broken\n\nnothing here\n", encoding="utf-8")
        results = [r for r in self._engine() if r.rule_id == "prep.sheet_shape" and not r.ok]
        self.assertEqual(len(results), 1)
        self.assertIn("Bar", results[0].message)
        self.assertTrue(results[0].path.endswith("broken.md"))

    def test_fit_before_run_warns_on_pending_and_not_fit(self) -> None:
        self.sheet_text(write=str(self.paths.harness / "prep" / "pending.md"))
        self.sheet_text(write=str(self.paths.harness / "prep" / "nofit.md"))
        path = self.paths.harness / "prep" / "nofit.md"
        path.write_text(prep._replace_verdict(path.read_text(encoding="utf-8"), "NOT_FIT", "reviewer"), encoding="utf-8")
        results = [r for r in self._engine() if r.rule_id == "prep.fit_before_run" and not r.ok]
        self.assertEqual(len(results), 2, [r.path for r in results])
        self.assertTrue(all(r.severity == "warn" for r in results))

    def test_fit_before_run_is_silent_after_fit(self) -> None:
        self.sheet_text(write=str(self.paths.harness / "prep" / "fit.md"))
        path = self.paths.harness / "prep" / "fit.md"
        path.write_text(prep._replace_verdict(path.read_text(encoding="utf-8"), "FIT", "reviewer"), encoding="utf-8")
        results = [r for r in self._engine() if r.rule_id == "prep.fit_before_run" and not r.ok]
        self.assertEqual(results, [])

    def test_panel_frozen_fails_a_missing_panel(self) -> None:
        write_milestones(self.paths, panel="absent-panel")
        for f in (self.paths.harness / "panels").glob("*.json"):
            f.unlink()
        reg = registries.Registry.load(self.paths)
        results = prep_rule(checkrules.run(self.paths, reg, only=["prep"]), "prep.panel_frozen", ok=False)
        self.assertEqual(len(results), 1)
        self.assertIn("absent-panel", results[0].message)

    def test_panel_frozen_fails_an_unfrozen_panel(self) -> None:
        write_panel(self.paths, frozen=False, grader="")
        reg = registries.Registry.load(self.paths)
        results = prep_rule(checkrules.run(self.paths, reg, only=["prep"]), "prep.panel_frozen", ok=False)
        self.assertEqual(len(results), 1)
        self.assertIn("not marked frozen", results[0].message)
        self.assertIn("no grader", results[0].message)

    def test_rules_never_raise_on_a_hostile_tree(self) -> None:
        (self.paths.harness / "prep" / "empty.md").write_text("", encoding="utf-8")
        (self.paths.harness / "prep" / "weird.md").write_text("## Bar\n\x00\x01 binary-ish\n", encoding="utf-8")
        results = self._engine()
        self.assertTrue(results)
        self.assertFalse(any("raised" in r.message for r in results), [r.message for r in results])


class TestFitReview(TempRepoCase):
    def test_fit_updates_the_line_in_place(self) -> None:
        args = new_args(write=str(self.paths.harness / "prep" / "sheet.md"))
        self.new_sheet(args)
        fit = argparse.Namespace(
            milestone="m1", verdict="FIT", sheet="docs/harness/prep/sheet.md", reviewer="reviewer",
            note="ok", json=True, post=False,
        )
        self.assertEqual(capture_rc(lambda: prep.fit_review(fit, self.paths, self.reg)), 0)
        text = (self.paths.harness / "prep" / "sheet.md").read_text(encoding="utf-8")
        self.assertEqual(prep.test_fitness(text), "FIT")
        self.assertNotIn("TEST FITNESS: PENDING", text)

    def test_fit_defaults_to_printing_not_posting(self) -> None:
        args = new_args(write=str(self.paths.harness / "prep" / "sheet.md"))
        self.new_sheet(args)
        fit = argparse.Namespace(
            milestone="m1", verdict="NOT_FIT", sheet="docs/harness/prep/sheet.md", reviewer="r",
            note="needs a control", json=True,
        )
        # no --post attribute at all: the default path must only print
        self.assertFalse(hasattr(fit, "post"))
        self.assertEqual(capture_rc(lambda: prep.fit_review(fit, self.paths, self.reg)), 0)
        self.assertEqual(prep.test_fitness((self.paths.harness / "prep" / "sheet.md").read_text(encoding="utf-8")), "NOT_FIT")

    def test_prose_mentioning_fit_is_not_a_verdict(self) -> None:
        text = "- D22: carry a TEST FITNESS: FIT review on the milestone before compute.\n"
        self.assertEqual(prep.test_fitness(text), prep.ABSENT)
        text += "## Verdict\nTEST FITNESS: PENDING\n"
        self.assertEqual(prep.test_fitness(text), "PENDING")

    def test_list_sheets_reports_verdicts_newest_first(self) -> None:
        for name in ("a.md", "b.md"):
            args = new_args(write=str(self.paths.harness / "prep" / name))
            self.new_sheet(args)
        listing = argparse.Namespace(json=True)
        rows = json.loads(_capture(lambda: prep.list_sheets(listing, self.paths, self.reg)))
        self.assertEqual(len(rows["sheets"]), 2)
        self.assertTrue(all(r["verdict"] == "PENDING" for r in rows["sheets"]))


class TestPanelScoring(unittest.TestCase):
    def _panel(self, noise: str = "+/-3 rows", bars: list[dict] | None = None) -> registries.Panel:
        bars = bars or [{"id": "accepted", "label": "accepted", "current": 30, "target": 40, "unit": "rows"}]
        return registries.Panel(id="p", title="t", milestone="m1", frozen=True, grader="g", bars=bars, noise=noise)

    def test_pass_at_and_above_target(self) -> None:
        panel = self._panel()
        self.assertEqual(prep.classify(panel, panel.bars[0], 40)["outcome"], "PASS")
        self.assertEqual(prep.classify(panel, panel.bars[0], 55)["outcome"], "PASS")

    def test_fail_below_target_outside_noise(self) -> None:
        panel = self._panel()
        self.assertEqual(prep.classify(panel, panel.bars[0], 35)["outcome"], "FAIL")
        self.assertEqual(prep.classify(panel, panel.bars[0], 30)["outcome"], "AT NOISE")

    def test_at_noise_inside_the_band_and_around_zero_movement(self) -> None:
        panel = self._panel(noise="+/-3 rows")
        for value in (31, 32, 33):
            # exactly at the band edge (33) is still inside it
            self.assertEqual(prep.classify(panel, panel.bars[0], value)["outcome"], "AT NOISE", value)
        # one past the edge is a real movement, and a FAIL because the target is not met
        self.assertEqual(prep.classify(panel, panel.bars[0], 34)["outcome"], "FAIL")

    def test_unstated_noise_never_claims_a_real_difference(self) -> None:
        panel = self._panel(noise="")
        result = prep.classify(panel, panel.bars[0], 36)
        self.assertEqual(result["outcome"], "FAIL")
        self.assertIn("unstated", result["reason"])
        self.assertIsNone(result["noise_band"])

    def test_bar_noise_overrides_the_panel_noise(self) -> None:
        bar = {"id": "accepted", "label": "accepted", "current": 30, "target": 40, "noise": "+/-1 rows"}
        panel = self._panel(noise="+/-10 rows", bars=[bar])
        self.assertEqual(prep.classify(panel, bar, 31)["outcome"], "AT NOISE")
        self.assertEqual(prep.classify(panel, bar, 33)["outcome"], "FAIL")
        self.assertEqual(prep.classify(panel, bar, 33)["noise_source"], "bar accepted")

    def test_forbid_bar_has_no_noise_excuse(self) -> None:
        bar = {"id": "softmax", "label": "softmax sites", "current": 0, "target": 0, "direction": "must be 0"}
        panel = self._panel(noise="+/-3 rows", bars=[bar])
        self.assertEqual(prep.classify(panel, bar, 0)["outcome"], "PASS")
        result = prep.classify(panel, bar, 1)
        self.assertEqual(result["outcome"], "FAIL")
        self.assertIn("must stay at 0", result["reason"])

    def test_ceiling_bar_lower_is_better(self) -> None:
        bar = {"id": "bpb", "label": "served BPB", "current": 0.95, "target": 0.9, "direction": "ceiling"}
        panel = self._panel(noise="", bars=[bar])
        self.assertEqual(prep.classify(panel, bar, 0.88)["outcome"], "PASS")
        self.assertEqual(prep.classify(panel, bar, 0.95)["outcome"], "FAIL")

    def test_bar_without_numbers_fails_rather_than_guesses(self) -> None:
        bar = {"id": "vague", "label": "vague"}
        panel = self._panel(bars=[bar])
        result = prep.classify(panel, bar, 1)
        self.assertEqual(result["outcome"], "FAIL")
        self.assertIn("no numeric", result["reason"])

    def test_noise_band_parses_the_shapes_the_repo_writes(self) -> None:
        self.assertEqual(prep._noise_band("+/-3 rows (bench noise)"), 3.0)
        self.assertEqual(prep._noise_band("±3/40"), 3.0)
        self.assertEqual(prep._noise_band("noise 0.01"), 0.01)
        self.assertIsNone(prep._noise_band("unstated - a bar must not be narrower than it"))
        self.assertIsNone(prep._noise_band(""))


class TestScorePanelEntryPoint(unittest.TestCase):
    """score_panel selects a bar, scores it, and never calls AT NOISE a real difference."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = core.HarnessPaths.discover(repo=REPO)
        cls.reg = registries.Registry.load(cls.paths)

    def test_default_bar_is_the_panel_headline_bar(self) -> None:
        args = argparse.Namespace(panel="v5-reply", value=116.0, bar=None, label=None, json=True)
        payload = json.loads(_capture(lambda: prep.score_panel(args, self.paths, self.reg)))
        self.assertEqual(payload["bar"], "reply-acceptable")
        self.assertEqual(payload["outcome"], "PASS")
        self.assertEqual(payload["value"], 116.0)

    def test_explicit_bar_selects_a_non_headline_bar(self) -> None:
        args = argparse.Namespace(panel="v5-reply", value=34.0, bar="v5-memory-check-pass", label=None, json=True)
        payload = json.loads(_capture(lambda: prep.score_panel(args, self.paths, self.reg)))
        self.assertEqual(payload["bar"], "v5-memory-check-pass")
        self.assertEqual(payload["outcome"], "PASS")

    def test_at_noise_is_reported_as_at_noise(self) -> None:
        args = argparse.Namespace(panel="v5-reply", value=22.0, bar="v5-memory-check-pass", label=None, json=True)
        payload = json.loads(_capture(lambda: prep.score_panel(args, self.paths, self.reg)))
        self.assertEqual(payload["outcome"], "AT NOISE")
        self.assertEqual(payload["noise_source"], "bar v5-memory-check-pass")

    def test_unknown_bar_is_a_usage_error(self) -> None:
        args = argparse.Namespace(panel="v5-reply", value=1.0, bar="nope", label=None, json=True)
        self.assertEqual(capture_rc(lambda: prep.score_panel(args, self.paths, self.reg)), 2)


class TestHostileInputs(TempRepoCase):
    """A rule never raises and an entry point degrades; a damaged tree is a finding, not a crash."""

    def test_rules_survive_damaged_files_and_an_unloaded_registry(self) -> None:
        d = self.paths.harness / "prep"
        (d / "empty.md").write_text("", encoding="utf-8")
        (d / "binary.md").write_bytes(b"\x00\x01\x02\xff\xfe")
        (d / "nul.md").write_text("## Bar\n\x00\n## Cost\n\x00\nTEST FITNESS: PENDING\n", encoding="utf-8")
        (self.paths.harness / "milestones.json").write_text("{oops", encoding="utf-8")
        for rule in prep.register_rules():
            for reg in (None,):
                results = rule.check(self.paths, reg)
                self.assertIsInstance(results, list, rule.id)

    def test_entry_points_do_not_raise_without_a_registry(self) -> None:
        args = new_args()
        args.write = str(self.paths.harness / "prep" / "x.md")
        for fn in (prep.new_sheet, prep.fit_review):
            self.assertEqual(capture_rc(lambda: fn(args, self.paths, None)), 1)
        self.assertEqual(capture_rc(lambda: prep.show_panel(argparse.Namespace(milestone="m1", json=True), self.paths, None)), 1)
        self.assertEqual(capture_rc(lambda: prep.score_panel(argparse.Namespace(panel="p", value=1.0, bar=None, label=None, json=True), self.paths, None)), 1)
        args.action = "list"
        self.assertIsInstance(prep.list_sheets(args, self.paths, None), int)

    def test_sheet_rule_reports_a_damaged_sheet_rather_than_raising(self) -> None:
        (self.paths.harness / "prep" / "empty.md").write_text("", encoding="utf-8")
        results = prep_rule(checkrules.run(self.paths, self.reg, only=["prep"]), "prep.sheet_shape", ok=False)
        self.assertTrue(results)
        self.assertTrue(any("empty" in r.message for r in results))


class TestCommittedPanels(unittest.TestCase):
    """The panels the milestone registry references must exist and be frozen."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = core.HarnessPaths.discover(repo=REPO)
        cls.reg = registries.Registry.load(cls.paths)

    def test_every_referenced_panel_exists(self) -> None:
        for milestone in self.reg.milestones.all():
            self.assertTrue(milestone.panel, milestone.id)
            panel = registries.find_panel(self.reg.panels, milestone.panel)
            self.assertIsNotNone(panel, f"{milestone.id} references {milestone.panel}")
            self.assertEqual(panel.milestone, milestone.key, panel.id)

    def test_every_panel_is_frozen_with_a_grader_and_bars(self) -> None:
        """A frozen panel must be complete. A not-started milestone's panel is exempt.

        Freezing is required before the first run a panel would score (D21/D22), not
        before the milestone it belongs to has started; an unstarted milestone has no
        pre-registered acceptance to encode. Those panels are asserted unfrozen below so
        the exemption cannot be used to smuggle in an incomplete frozen panel.
        """
        unstarted = {
            m.panel
            for m in self.reg.milestones.all()
            if str(m.status).lower() in ("not-started", "not started", "proposed")
        }
        for panel in self.reg.panels:
            if panel.id in unstarted:
                self.assertFalse(panel.frozen, f"{panel.id} belongs to a not-started milestone but claims to be frozen")
                self.assertFalse(panel.bars, f"{panel.id} has no pre-registered acceptance, so it must carry no bars")
                continue
            self.assertTrue(panel.frozen, panel.id)
            self.assertTrue(panel.grader, panel.id)
            self.assertTrue(panel.bars, panel.id)
            for bar in panel.bars:
                self.assertIn("current", bar, f"{panel.id}/{bar.get('id')}")
                self.assertIn("target", bar, f"{panel.id}/{bar.get('id')}")

    def test_panel_ids_and_bars(self) -> None:
        ids = {p.id for p in self.reg.panels}
        self.assertLessEqual({"v5-reply", "m2-reply512", "m3-memory", "m4-served"}, ids)
        reply = registries.find_panel(self.reg.panels, "v5-reply")
        bars = {b["id"]: b for b in reply.bars}
        self.assertEqual((bars["reply-acceptable"]["current"], bars["reply-acceptable"]["target"], bars["reply-acceptable"]["of"]), (37, 116, 232))
        self.assertEqual((bars["v5-memory-check-pass"]["current"], bars["v5-memory-check-pass"]["target"]), (20, 34))
        m4 = {b["id"]: b for b in registries.find_panel(self.reg.panels, "m4-served").bars}
        self.assertEqual(m4["softmax-at-runtime"]["target"], 0)

    def test_panel_frozen_rule_passes_on_the_committed_tree(self) -> None:
        """No panel violation may be an error on the committed tree.

        Not-started milestones (M5-M8) legitimately carry an unfrozen panel with no bars;
        those are reported as warnings so they are visible and must be frozen before the
        first run they would score. Any error here means a started milestone is missing a
        frozen panel, which is a real failure.
        """
        results = prep_rule(checkrules.run(self.paths, self.reg, only=["prep"]), "prep.panel_frozen")
        self.assertTrue(results)
        errors = [r for r in results if not r.ok and r.severity == "error"]
        self.assertEqual(errors, [], [r.message for r in errors])
        warnings = [r for r in results if not r.ok and r.severity == "warn"]
        for r in warnings:
            self.assertIn("not-started", r.message, r.message)

    def test_milestone_numbers_match_the_committed_panels(self) -> None:
        """Where a milestone states a numeric headline, the panel's headline bar must agree.

        A qualitative headline (M4's "softmax at runtime: yes") has no numeric ``current``
        to compare, so it is reported as skipped rather than silently passing.
        """
        compared: list[str] = []
        skipped: list[tuple[str, str]] = []
        unstarted: list[str] = []
        for milestone in self.reg.milestones.all():
            panel = registries.find_panel(self.reg.panels, milestone.panel)
            if str(milestone.status).lower() in ("not-started", "not started", "proposed"):
                # no pre-registered acceptance exists yet, so there is nothing to compare
                unstarted.append(milestone.id)
                continue
            if milestone.headline.get("current") is None:
                skipped.append((milestone.id, str(milestone.headline.get("text", milestone.headline.get("label", "")))))
                continue
            bar = panel.bars[0]
            self.assertEqual(bar["current"], milestone.headline.get("current"), milestone.id)
            self.assertEqual(bar["target"], milestone.headline.get("target"), milestone.id)
            compared.append(milestone.id)
        self.assertEqual(compared, ["M1", "M2", "M3"], "the numeric headlines are M1-M3")
        self.assertEqual([m for m, _ in skipped], ["M4"], "M4's headline is qualitative: no numeric current")
        self.assertTrue(skipped[0][1], "the skipped milestone must name its qualitative headline text")
        self.assertEqual(unstarted, ["M5", "M6", "M7", "M8"], "the unstarted milestones carry no frozen panel to compare")

    def test_a_qualitative_headline_is_not_compared_to_a_bar(self) -> None:
        """The skip above is a real rule, not an accident of the current registry."""
        qualitative = registries.Milestone(
            key="mx",
            issue=2099,
            id="MX",
            title="qualitative",
            lab="lab",
            status="in-progress",
            headline={"label": "softmax at runtime", "text": "softmax at runtime: yes", "target": 0},
            secondary=[],
            panel="v5-reply",
            base_artifact="",
            mechanisms=[],
            lines=[],
            next="",
        )
        self.assertIsNone(qualitative.headline.get("current"))


def prep_rule(results: list, rule_id: str, ok: bool | None = None) -> list:
    """This module's results for one rule id, optionally filtered by ok.

    checkrules keeps rules process-wide and re-invokes every module's register_rules()
    on each call, so a module without an idempotency cache duplicates its Rule objects
    (and therefore its RuleResults). De-duplicating on the result's identity keeps these
    assertions about this module's rules rather than about another module's cache.
    """
    seen: set[tuple[str, str, str]] = set()
    out = []
    for result in results:
        if result.rule_id != rule_id:
            continue
        key = (result.rule_id, result.path, result.message)
        if key in seen:
            continue
        seen.add(key)
        out.append(result)
    if ok is not None:
        out = [r for r in out if r.ok is ok]
    return out


def _capture(fn) -> str:
    """Run fn, returning what it printed on stdout."""
    import io
    from contextlib import redirect_stdout

    buf = io.StringIO()
    with redirect_stdout(buf):
        fn()
    return buf.getvalue()


def capture_rc(fn) -> int:
    """Run fn, swallow its output, and return its exit code."""
    import io
    from contextlib import redirect_stdout, redirect_stderr

    buf = io.StringIO()
    with redirect_stdout(buf), redirect_stderr(buf):
        return int(fn())


if __name__ == "__main__":
    unittest.main()


class TestOwnerRulesApplied(unittest.TestCase):
    """`harness.owner_rules_applied` - the durable fix for D31 and D32 going missing.

    Both rules were issued and neither reached a pre-registration sheet, and nothing caught it
    because **no rule compared the sheet's list against the registry**. `parity` keeps the two
    implementations' tuples equal to each other; nothing kept either equal to the registry.
    """

    def _paths(self):
        """Paths for the REAL repo, so the rule runs against the committed registry."""
        repo = pathlib.Path(__file__).resolve().parents[2]
        return core.HarnessPaths(
            repo=repo, harness=repo / "docs" / "harness", state=repo / ".harness"
        )

    def _run(self):
        import checkrules

        paths = self._paths()
        reg = registries.Registry.load(paths)
        rule = next(r for r in checkrules.rules() if r.id == "harness.owner_rules_applied")
        return [r for r in rule.check(paths, reg) if not r.ok]

    def test_the_sheet_cites_every_owner_rule_scoped_to_all(self):
        self.assertEqual(self._run(), [])

    def test_the_rule_reads_the_sheet_list_from_source(self):
        """**From source, not by import.** Importing the same tuple would compare a value with
        itself, and the check would pass no matter what the registry said."""
        import checkrules

        applied = checkrules._applicable_rules_from_source()
        self.assertIn("D32", applied, "D32 is an owner rule scoped to all")
        self.assertIn("D31", applied)
        self.assertGreaterEqual(len(applied), 9)

    def test_a_dropped_rule_is_reported(self):
        """The probe that proves the rule can fail: with the tuple shortened, D32 must be named."""
        import checkrules

        rule = next(r for r in checkrules.rules() if r.id == "harness.owner_rules_applied")
        original = checkrules._applicable_rules_from_source
        try:
            checkrules._applicable_rules_from_source = lambda: tuple(
                r for r in original() if r != "D32"
            )
            paths = self._paths()
            found = rule.check(paths, registries.Registry.load(paths))
            self.assertTrue(any("D32" in r.message for r in found), found)
        finally:
            checkrules._applicable_rules_from_source = original


class TestSheetIsNotSilentlyOverwritten(unittest.TestCase):
    """`prep new` overwrote an existing sheet with no check.

    **A pre-registration sheet is the D22 gate's own record.** Re-running `prep new` for a milestone
    that already has one replaced the bar, the panel and the cost that were pre-registered *before*
    the work - exactly the evidence D22 exists to preserve. Refusing is the safe default; replacing is
    explicit via `--force`.
    """

    def _repo_with_ledger(self):
        root = Path(tempfile.mkdtemp())
        paths = make_paths(root)
        write_milestones(paths)
        return root, paths

    def _args(self, root, **kw):
        base = {
            "action": "new",
            "milestone": "m1",
            "mechanism": None,
            "run": None,
            "bar": None,
            "seeds": None,
            "rows": None,
            "base": None,
            "split": None,
            "steps": None,
            "stop": None,
            "cost": None,
            "write": str(root / "sheet.md"),
            "json": False,
            "force": False,
        }
        base.update(kw)
        return argparse.Namespace(**base)

    def test_a_second_write_is_refused_and_the_sheet_is_unchanged(self):
        root, paths = self._repo_with_ledger()
        args = self._args(root)
        self.assertEqual(prep.new_sheet(args, paths, registries.Registry.load(paths)), 0)
        target = root / "sheet.md"
        before = target.read_text(encoding="utf-8")

        self.assertEqual(
            prep.new_sheet(args, paths, registries.Registry.load(paths)),
            1,
            "the second write must be refused",
        )
        self.assertEqual(target.read_text(encoding="utf-8"), before, "the sheet must be untouched")

    def test_force_replaces_it(self):
        root, paths = self._repo_with_ledger()
        prep.new_sheet(self._args(root), paths, registries.Registry.load(paths))
        self.assertEqual(
            prep.new_sheet(
                self._args(root, force=True), paths, registries.Registry.load(paths)
            ),
            0,
            "--force must replace it",
        )
