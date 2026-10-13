#!/usr/bin/env python3
"""Tests for scripts/harness/prlint.py (the PR body contract).

    python3 -m unittest discover -s tests/harness -t . -v

No network, no `gh`, no model compute: every fixture is a body string and the
frozen registry under docs/harness/.
"""

from __future__ import annotations

import argparse
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import prlint  # noqa: E402  (needs the sys.path insert above)
from core import HarnessPaths  # noqa: E402
from registries import Registry  # noqa: E402

TEMPLATE = REPO / ".github" / "pull_request_template.md"

#: The exact shape of .github/pull_request_template.md: HTML comments, the empty
#: Line declaration, and a checklist of unticked boxes.
TEMPLATE_BODY = """<!-- Delivery cadence: CONTRIBUTING.md § Delivery cadence. Delete lines that do not apply, and say why. -->

## Line and headline (D21, docs/labs/session-goal.md)

<!-- Line: <name> · count n/3 · headline before → after (for example `M2 8/512 → 8/512`). At 3/3 the next action on this line is a pivot card. -->

Line:  · count /3 · headline  → 

## What and why

<!-- One paragraph: the change, the question it answers, the milestone (#2029–#2036). -->

## Result

<!-- Numbers with their artifact, data split and comparison. Negative results count. Decision: KEEP / REJECT / NOT YET PROMOTED. -->

## Checklist

- [ ] `python3 scripts/harness/cli.py check` passes, and `python3 scripts/harness/cli.py lint check --body <this body>` reports no unresolved finding
- [ ] Any model compute here is covered by a linked pre-registration carrying `TEST FITNESS: FIT` (D22)
- [ ] Checks run at this exact head (commands and outcomes posted on this PR)
- [ ] Research record: `docs/labs/<topic>-<YYYY-MM-DD>/README.md` (or `docs/evidence/…`)
- [ ] New entry at the top of `docs/integration/current-state.md`, with **Next:**
- [ ] Every document this change makes stale is updated (`git grep -n -i '<what changed>' -- '*.md'`)
- [ ] STATUS.md / ROADMAP.md / #2028 table updated, or state unchanged
- [ ] README updated in its existing sections, or nothing a reader relies on changed
- [ ] `python3 scripts/check_claim_wording.py` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch and worktree, `scripts/storage/uor-hygiene --apply`, then comment the result on the milestone issue

References #
"""


def fill(
    line: str = "exact addressed memory · count 1/3 · headline 37/232 → 37/232",
    result: str = (
        "v5 memory check_pass 20/40 against a 34/40 target, on the saved artifact, development split.\n"
        "Decision: KEEP\n\n"
        "Pre-registration: TEST FITNESS: FIT at #issuecomment-6124773996."
    ),
    record: str = "docs/labs/pointer-gate-finetune-2026-10-10/README.md",
    refs: str = "References #2029",
    tick: bool = True,
    checklist_extra: str = "",
) -> str:
    """A body that uses the template's own shape, filled in."""
    body = TEMPLATE_BODY
    body = body.replace("Line:  · count /3 · headline  → ", f"Line: {line}")
    body = body.replace(
        "<!-- Numbers with their artifact, data split and comparison. Negative results count. Decision: KEEP / REJECT / NOT YET PROMOTED. -->",
        result,
    )
    body = body.replace("`docs/labs/<topic>-<YYYY-MM-DD>/README.md`", f"`{record}`")
    body = body.replace("References #", refs)
    if checklist_extra:
        body = body.replace("\nReferences #", f"\n{checklist_extra}\nReferences #")
    if tick:
        body = body.replace("- [ ]", "- [x]")
    return body


class Fixtures(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = HarnessPaths.discover(repo=REPO)
        cls.reg = Registry.load(cls.paths)

    def rules(self, body: str) -> list[str]:
        result = prlint.check_pr(body, self.paths, self.reg)
        return [p["rule"] for p in result["problems"]]

    def problems(self, body: str) -> list[dict]:
        return prlint.check_pr(body, self.paths, self.reg)["problems"]


class TestTemplateShape(Fixtures):
    def test_repo_template_is_template_shaped(self) -> None:
        """The real template must parse, and fail because nothing is filled in."""
        if not TEMPLATE.exists():
            self.skipTest("no .github/pull_request_template.md")
        body = TEMPLATE.read_text(encoding="utf-8")
        fields = prlint.parse_pr_body(body)
        self.assertEqual(fields["line_name"], "")
        self.assertIsNone(fields["line_count"])
        self.assertEqual(fields["decision"], "")
        self.assertGreaterEqual(len(fields["checklist"]), 8)
        self.assertEqual(fields["checked"], 0)
        self.assertIn(prlint.MILESTONE_ISSUES.start, [int(i) for i in prlint._ISSUE_RE.findall(body)])

        result = prlint.check_pr(body, self.paths, self.reg)
        self.assertFalse(result["ok"], "an unfilled template must not pass")
        rules = {p["rule"] for p in result["problems"]}
        self.assertLessEqual({"prlint.line_declared", "prlint.headline_present", "prlint.decision_present"}, rules)

    def test_html_comments_are_not_parsed_as_fields(self) -> None:
        body = TEMPLATE_BODY
        fields = prlint.parse_pr_body(body)
        # "for example `M2 8/512 → 8/512`" lives inside a comment: it must not
        # become the declared headline.
        self.assertIsNone(fields["headline_after"]["value"])
        self.assertEqual(fields["line_name"], "")

    def test_both_separators_are_accepted(self) -> None:
        arrow = "with arrow, middle dot"
        fields = prlint.parse_pr_body(fill(line="exact addressed memory · count 1/3 · headline 37/232 → 37/232"))
        self.assertEqual(fields["line_name"], "exact addressed memory", arrow)
        self.assertEqual(fields["line_count"], 1)
        self.assertEqual(fields["headline_before"]["value"], 37)
        self.assertEqual(fields["headline_after"]["value"], 37)

        fields = prlint.parse_pr_body(fill(line="exact addressed memory - count 1/3 - headline 37/232 -> 37/232"))
        self.assertEqual(fields["line_name"], "exact addressed memory", "hyphen separator")
        self.assertEqual(fields["line_count"], 1)
        self.assertEqual(fields["headline_before"]["value"], 37)
        self.assertEqual(fields["headline_after"]["value"], 37)

    def test_template_shaped_filled_body_passes(self) -> None:
        result = prlint.check_pr(fill(), self.paths, self.reg)
        self.assertTrue(result["ok"], result["problems"])
        self.assertEqual(result["problems"], [])
        fields = result["fields"]
        self.assertEqual(fields["line_name"], "exact addressed memory")
        self.assertEqual(fields["count_text"], "1/3")
        self.assertEqual(fields["decision"], "KEEP")
        self.assertEqual(fields["milestone_issue"], 2029)
        self.assertEqual(fields["record_path"], "docs/labs/pointer-gate-finetune-2026-10-10/README.md")
        self.assertEqual(fields["references"], [2029])
        self.assertTrue(fields["claims_current_state"])
        self.assertTrue(all(c["checked"] for c in fields["checklist"]))


class TestPreregistration(Fixtures):
    def test_run_without_preregistration_fails(self) -> None:
        """D22: a run that produced model numbers needs a pre-registration."""
        body = fill(
            result=(
                "Three fits at 3 seeds on the saved artifact; v5 memory check_pass 20/40, reply 41/232.\n"
                "Decision: KEEP"
            )
        )
        result = prlint.check_pr(body, self.paths, self.reg)
        self.assertFalse(result["ok"])
        problems = [p for p in result["problems"] if p["rule"] == "prlint.prereg_before_compute"]
        self.assertEqual(len(problems), 1)
        self.assertEqual(problems[0]["severity"], "error")
        self.assertIn("pre-registration", problems[0]["message"])
        self.assertIn("TEST FITNESS", problems[0]["fix"])

    def test_preregistration_forms_are_accepted(self) -> None:
        for prereg in (
            "Pre-registration: TEST FITNESS: FIT at [6105293249](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105293249).",
            "Pre-registration: #issuecomment-6100142868.",
            "Pre-registration: prep sheet `m1-cycle8-v2`.",
            "This run was pre-registered with TEST FITNESS: NOT FIT before compute.",
        ):
            with self.subTest(prereg=prereg):
                body = fill(
                    result=(
                        "Three fits at 3 seeds; v5 memory check_pass 20/40, reply 41/232.\n"
                        "Decision: KEEP\n\n" + prereg
                    )
                )
                rules = self.rules(body)
                self.assertNotIn("prlint.prereg_before_compute", rules)

    def test_cpu_model_compute_is_flagged(self) -> None:
        body = fill(
            result=(
                "CPU training at 3 seeds: v5 memory check_pass 20/40, reply 41/232. Decision: KEEP\n\n"
                "Pre-registration: TEST FITNESS: FIT at #issuecomment-1."
            )
        )
        problems = [p for p in self.problems(body) if p["rule"] == "prlint.no_cpu_model_compute"]
        self.assertEqual(len(problems), 1)
        self.assertIn("D24", problems[0]["fix"])

    def test_softmax_rule_delegates_to_the_interlock(self) -> None:
        """prlint.softmax_ban must call scripts/check_no_softmax.py, not rescan."""
        import tempfile

        stub = (
            "ROOT = __import__('pathlib').Path(__file__).resolve().parent.parent\n"
            "CALLS = []\n"
            "def scan():\n"
            "    CALLS.append('scan')\n"
            "    return {'crates/x/src/read.rs': 1}\n"
            "def load_baseline():\n"
            "    return {'crates/x/src/read.rs': 1}\n"
        )
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "scripts").mkdir()
            (repo / "scripts" / "check_no_softmax.py").write_text(stub, encoding="utf-8")
            paths = HarnessPaths.discover(repo=repo)
            prlint._SOFTMAX_MODULES.clear()
            prlint._SOFTMAX_SCANS.clear()
            script = (repo / "scripts" / "check_no_softmax.py").resolve()
            module = prlint._load_softmax_module(script)
            self.assertIsNotNone(module, "the interlock must be importable by path")
            self.assertTrue(callable(getattr(module, "scan", None)))
            # the interlock's own scan() is what runs: record each call in the
            # same module instance that prlint caches (keyed by resolved path)
            calls: list[str] = []
            cached = prlint._SOFTMAX_MODULES[str(script)]
            original = cached.scan

            def counting_scan() -> dict[str, int]:
                calls.append("scan")
                return original()

            cached.scan = counting_scan
            # at baseline -> no error, and the delegated scan ran exactly once
            problems = prlint._check_softmax(paths, {"softmax_claims": ["softmax"]})
            self.assertEqual([p.rule for p in problems if p.severity == "error"], [])
            self.assertEqual(calls, ["scan"])
            # above baseline -> an error result that names the file and the fix
            module.scan = lambda: {"crates/x/src/read.rs": 2}
            prlint._SOFTMAX_SCANS.clear()
            problems = prlint._check_softmax(paths, {"softmax_claims": ["softmax"]})
            self.assertEqual(len(problems), 1)
            self.assertEqual(problems[0].severity, "error")
            self.assertIn("crates/x/src/read.rs", problems[0].message)
            self.assertIn("check_no_softmax.py", problems[0].fix)

    def test_real_interlock_is_reachable(self) -> None:
        script = REPO / "scripts" / "check_no_softmax.py"
        if not script.exists():
            self.skipTest("no D23 interlock in this checkout")
        prlint._SOFTMAX_MODULES.clear()
        prlint._SOFTMAX_SCANS.clear()
        module = prlint._load_softmax_module(script)
        self.assertIsNotNone(module)
        self.assertEqual(module.ROOT, REPO, "delegated scan must resolve the same repo root")
        body = fill(result=fill.__defaults__[1] + "\n\nThe stack read is softmax-free (D23, no normalized exponential).")
        self.assertNotIn("error", [p["severity"] for p in self.problems(body) if p["rule"] == "prlint.softmax_ban"])


class TestProgressControl(Fixtures):
    def test_three_of_three_requires_a_pivot(self) -> None:
        body = fill(line="exact addressed memory · count 3/3 · headline 37/232 → 37/232")
        problems = [p for p in self.problems(body) if p["rule"] == "prlint.pivot_at_three"]
        self.assertEqual(len(problems), 1)
        self.assertEqual(problems[0]["severity"], "error")
        self.assertIn("pivot", problems[0]["message"].lower())
        self.assertIn("D21", problems[0]["fix"])
        self.assertFalse(prlint.check_pr(body, self.paths, self.reg)["ok"])

    def test_three_of_three_with_a_pivot_card_passes(self) -> None:
        body = fill(
            line="exact addressed memory · count 3/3 · headline 37/232 → 37/232",
            result=(
                "Pivot card posted on #2029: stop and archive this line after three non-moving merges.\n"
                "Decision: REJECT\n\n"
                "Pre-registration: TEST FITNESS: FIT at #issuecomment-1."
            ),
        )
        self.assertNotIn("prlint.pivot_at_three", self.rules(body))
        self.assertTrue(prlint.check_pr(body, self.paths, self.reg)["ok"])

    def test_headline_must_be_numbers_not_prose(self) -> None:
        body = fill(line="exact addressed memory · count 1/3 · headline much better now")
        self.assertIn("prlint.headline_present", self.rules(body))

    def test_registry_count_above_the_declared_one_warns(self) -> None:
        """The registry says the starling read is already at 1/3 (M4 #2032)."""
        body = fill(
            line="starling read · count 0/3 · headline 0.886838 → 0.886838",
            result="BPB unchanged at 0.886838. Decision: NOT YET PROMOTED\n\nPre-registration: TEST FITNESS: FIT at #issuecomment-1.",
            refs="References #2032",
        )
        problems = [p for p in self.problems(body) if p["rule"] == "prlint.count_matches_registry"]
        self.assertEqual(len(problems), 1)
        self.assertIn("1/3", problems[0]["message"])
        self.assertIn("starling read", problems[0]["message"])

    def test_headline_move_is_stated(self) -> None:
        body = fill(line="exact addressed memory · count 0/3 · headline 37/232 → 45/232")
        self.assertNotIn("prlint.headline_vs_registry", self.rules(body))

    def test_registry_mismatch_without_a_move_warns(self) -> None:
        body = fill(line="exact addressed memory · count 0/3 · headline 45/232 → 45/232")
        self.assertIn("prlint.headline_vs_registry", self.rules(body))

    def test_missing_decision_fails(self) -> None:
        body = fill(result="v5 memory check_pass 20/40; nothing else.\n\nPre-registration: TEST FITNESS: FIT at #issuecomment-1.")
        self.assertIn("prlint.decision_present", self.rules(body))

    def test_partial_work_references_rather_than_closes(self) -> None:
        body = fill(refs="References #2029")
        fields = prlint.parse_pr_body(body)
        self.assertEqual(fields["references"], [2029])
        self.assertEqual(fields["closes"], [])
        self.assertNotIn("prlint.org_rules", self.rules(body))

        bare = fill(refs="")
        self.assertIn("prlint.org_rules", self.rules(bare))


class TestRecordsAndChecks(Fixtures):
    def test_missing_record_file_warns(self) -> None:
        body = fill(record="docs/labs/nope-2099-01-01/README.md")
        problems = [p for p in self.problems(body) if p["rule"] == "prlint.record_path"]
        self.assertEqual(len(problems), 1)
        self.assertIn("does not exist", problems[0]["message"])

    def test_no_dated_record_path_warns(self) -> None:
        body = fill().replace("`docs/labs/pointer-gate-finetune-2026-10-10/README.md`", "the research record")
        self.assertIn("prlint.record_path", self.rules(body))

    def test_current_state_claim_is_checked(self) -> None:
        body = fill()
        self.assertNotIn("prlint.current_state_entry", self.rules(body))
        unticked = fill(tick=False)
        self.assertFalse(prlint.parse_pr_body(unticked)["claims_current_state"])

    def test_checks_at_head_accepts_a_box_or_commands(self) -> None:
        unticked = fill(tick=False)
        self.assertIn("prlint.checks_at_head", self.rules(unticked))
        with_commands = unticked.replace(
            "## Result",
            "## Result\n\nChecks at this exact head: `python3 -m unittest discover -s tests/harness -t .` (12 passed).",
        )
        self.assertNotIn("prlint.checks_at_head", self.rules(with_commands))


class TestRendering(Fixtures):
    def test_format_renders_a_parsable_body(self) -> None:
        fields = prlint.parse_pr_body(fill())
        rendered = prlint.format_pr_body(fields)
        again = prlint.parse_pr_body(rendered)
        self.assertEqual(again["line_name"], fields["line_name"])
        self.assertEqual(again["line_count"], fields["line_count"])
        self.assertEqual(again["headline_after"]["value"], fields["headline_after"]["value"])
        self.assertEqual(again["decision"], fields["decision"])
        self.assertIn("References #2029", rendered)
        self.assertIn("Line: exact addressed memory · count 1/3 · headline 37/232 → 37/232", rendered)
        self.assertGreaterEqual(len(again["checklist"]), 8)

    def test_format_survives_an_empty_body(self) -> None:
        rendered = prlint.format_pr_body(prlint.parse_pr_body(""))
        self.assertIn("Line:", rendered)
        self.assertIn("Decision:", rendered)

    def test_json_problems_are_serialisable(self) -> None:
        import json

        result = prlint.check_pr(TEMPLATE_BODY, self.paths, self.reg)
        json.dumps(result)  # must not raise
        for problem in result["problems"]:
            self.assertEqual(set(problem), {"rule", "severity", "message", "fix"})


class TestModuleApi(Fixtures):
    def test_contract_api(self) -> None:
        rules = prlint.register_rules()
        ids = {r.id for r in rules}
        self.assertEqual(len(ids), len(rules), "duplicate rule ids")
        # every gate the module raises is declared, so a lesson may cite it as a guard
        self.assertIn("prlint.gates_declared", ids)
        self.assertIn("prlint.softmax_ban", ids)
        declared = {rid.split(".", 1)[1] for rid in prlint.LOGICAL_RULES}
        self.assertEqual(declared, set(prlint._GATE_IDS), "LOGICAL_RULES and check_pr() must agree")
        for rid in prlint.LOGICAL_RULES:
            self.assertTrue(rid.startswith("prlint."))
        # the D23 interlock rule has real repo work, not just an id
        interlock = next(r for r in rules if r.id == "prlint.softmax_ban")
        results = interlock.check(self.paths, self.reg)
        self.assertTrue(all(r.ok for r in results), [r.message for r in results])
        gates = next(r for r in rules if r.id == "prlint.gates_declared")
        results = gates.check(self.paths, self.reg)
        self.assertTrue(all(r.ok for r in results), [r.message for r in results])

    def test_main_check_and_format(self) -> None:
        body = fill()
        args = argparse.Namespace(action="check", body=body, body_file=None, json=False)
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = prlint.main(args, self.paths, self.reg)
        self.assertEqual(code, 0, buf.getvalue())
        self.assertTrue(buf.getvalue().strip())

        result = prlint.check_pr(TEMPLATE_BODY, self.paths, self.reg)
        self.assertFalse(result["ok"])
        args = argparse.Namespace(action="check", body=TEMPLATE_BODY, body_file=None, json=True)
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = prlint.main(args, self.paths, self.reg)
        self.assertEqual(code, 1)
        import json

        self.assertIn("problems", json.loads(buf.getvalue()))

        args = argparse.Namespace(action="format", body=body, body_file=None, json=False)
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = prlint.main(args, self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("Line:", buf.getvalue())

    def test_main_rejects_a_missing_body(self) -> None:
        args = argparse.Namespace(action="check", body=None, body_file=None, json=False)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(prlint.main(args, self.paths, self.reg), 2)

    def test_a_hostile_body_never_raises(self) -> None:
        for body in (
            "",
            "Line:",
            "Line:  · count /3 · headline  → ",
            "count 3/3",
            "Line: x · count 9/3 · headline NaN → NaN",
            "## Line\n\n" + "x" * 5000,
            "Line: a - count 1/3 - headline 1/2 → 3/4\nDecision: KEEP",
        ):
            with self.subTest(body=body[:40]):
                prlint.check_pr(body, self.paths, self.reg)


if __name__ == "__main__":
    unittest.main()
