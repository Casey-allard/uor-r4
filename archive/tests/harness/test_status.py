"""Tests for scripts/harness/status.py and githelp.py.

No network and no `gh`: build_status runs against a temporary repo + registry with
the module's gh helper monkeypatched. Everything here is standard library.

    python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import pathlib
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve()
HARNESS_DIR = HERE.parents[2] / "scripts" / "harness"
if str(HARNESS_DIR) not in sys.path:
    sys.path.insert(0, str(HARNESS_DIR))

import githelp  # noqa: E402
import status  # noqa: E402
from core import HarnessPaths, now_iso  # noqa: E402
from registries import Registry  # noqa: E402

MILESTONES = {
    "schema": 1,
    "note": "test fixture",
    "tracker": {"issue": 2028, "title": "tracker"},
    "milestones": {
        "m1": {
            "issue": 2029,
            "id": "M1",
            "title": "language base",
            "lab": "deepseek",
            "status": "in-progress",
            "headline": {"label": "reply acceptable", "current": 37, "target": 116, "of": 232},
            "secondary": [{"label": "v5 memory check_pass", "current": 20, "target": 34, "of": 40}],
            "panel": "v5-reply",
            "base_artifact": "D10",
            "mechanisms": ["exact-addressed-memory"],
            "lines": [{"name": "exact addressed memory", "count": 0, "of": 3}],
            "next": "re-register the cycle-8 run",
        },
        "m2": {
            "issue": 2030,
            "id": "M2",
            "title": "grounded reply",
            "lab": "codex",
            "status": "in-progress",
            "headline": {"label": "complete correct replies (dev)", "current": 437, "target": 256, "of": 512},
            "secondary": [],
            "panel": "m2-reply512",
            "base_artifact": "saved437",
            "mechanisms": ["protected-legal-construction"],
            "lines": [{"name": "protected/discrete constructor", "count": 5, "of": 3}],
            "next": "resolve the CUDA producer gates",
        },
        "m3": {
            "issue": 2031,
            "id": "M3",
            "title": "durable memory",
            "lab": "codex",
            "status": "in-progress",
            "headline": {"label": "typed outcomes qualified", "current": 0, "target": 1, "of": 1, "vacuous": True},
            "secondary": [],
            "panel": "m3-memory",
            "base_artifact": "stack_grounded_session",
            "mechanisms": ["exact-addressed-memory"],
            "lines": [{"name": "learned grounding and abstention", "count": 0, "of": 3}],
            "next": "qualify grounding and abstention",
        },
        "m4": {
            "issue": 2032,
            "id": "M4",
            "title": "one served model",
            "lab": "claude",
            "status": "in-progress",
            "headline": {
                "label": "softmax at runtime",
                "text": "softmax at runtime: yes",
                "target": 0,
                "target_display": "no (D23 prohibits softmax project-wide)",
                "of": 1,
                "unit": "prohibited site",
            },
            "secondary": [{"label": "served BPB at 512 windows", "current": 0.886838, "target": 0.9, "of": None}],
            "panel": "m4-served",
            "base_artifact": "19.9M chat stack",
            "mechanisms": ["starling-read", "flock-rank-reads"],
            "lines": [{"name": "starling read", "count": 1, "of": 3}],
            "next": "ten-seed starling confirmation",
        },
    },
    "owner_rules": [
        {"id": "D21", "rule": "three consecutive merged PRs on one line force a pivot card", "scope": "all"},
        {"id": "D24", "rule": "no CPU model compute", "scope": "all"},
    ],
}


def iso(hours_ago: float = 0.0) -> str:
    return (datetime.now(timezone.utc) - timedelta(hours=hours_ago)).strftime("%Y-%m-%dT%H:%M:%SZ")


def issue(number: int, hours_ago: float = 1.0, comments: list[dict] | None = None) -> dict:
    return {
        "number": number,
        "title": f"milestone {number}",
        "state": "OPEN",
        "updatedAt": iso(hours_ago),
        "url": f"https://github.com/UOR-Foundation/uor-r4/issues/{number}",
        "comments": comments or [],
    }


class FakeGh:
    """A stand-in for status.gh_json: records calls, returns canned answers."""

    def __init__(
        self,
        issues: dict[int, dict] | None = None,
        open_prs: list[dict] | None = None,
        merged_prs: list[dict] | None = None,
        branch_prs: dict[str, list[dict]] | None = None,
        fail: str = "",
    ) -> None:
        self.issues = issues if issues is not None else {2029: issue(2029, 1.0), 2030: issue(2030, 30.0), 2031: issue(2031, 2.0), 2032: issue(2032, 3.0)}
        self.open_prs = open_prs if open_prs is not None else []
        self.merged_prs = merged_prs if merged_prs is not None else []
        self.branch_prs = branch_prs if branch_prs is not None else {}
        self.fail = fail
        self.calls: list[list[str]] = []

    def __call__(self, subargs, repo=None, timeout=20):
        self.calls.append(list(subargs))
        if self.fail:
            return False, None, f"`gh {' '.join(subargs)}` exited 1: {self.fail}"
        if subargs[:2] == ["pr", "list"]:
            if "--head" in subargs:
                head = subargs[subargs.index("--head") + 1]
                return True, list(self.branch_prs.get(head, [])), ""
            if "open" in subargs:
                return True, list(self.open_prs), ""
            return True, list(self.merged_prs), ""
        if subargs[:2] == ["issue", "view"]:
            number = int(subargs[2])
            item = dict(self.issues.get(number, issue(number)))
            if any("comments" in field for field in subargs):
                item.setdefault("comments", [])
            return True, item, ""
        return False, None, f"unexpected gh call: {subargs}"


class StatusTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-status-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        (self.repo / "docs" / "harness" / "ledger").mkdir(parents=True, exist_ok=True)
        (self.repo / "docs" / "harness" / "milestones.json").write_text(json.dumps(MILESTONES), encoding="utf-8")
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")
        self.reg = Registry.load(self.paths)

    def write_receipts(self, rows: list[dict]) -> None:
        from core import month

        path = self.paths.ledger_dir / f"{month()}.jsonl"
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def receipts(self) -> list[dict]:
        """Three receipts with distinct, increasing local timestamps (newest last)."""
        base = datetime.now(timezone.utc).astimezone() - timedelta(seconds=5)

        def ts(offset: int) -> str:
            return (base + timedelta(seconds=offset)).isoformat(timespec="seconds")

        return [
            {"ts": ts(0), "kind": "skill", "skill": "harness-status-check", "milestone": "m1"},
            {"ts": ts(1), "kind": "team", "role": "history-curator", "milestone": "m2", "count": 1},
            {"ts": ts(2), "kind": "heartbeat", "milestone": "m1", "line": "exact addressed memory", "count": "0/3"},
        ]


class BuildStatusTest(StatusTestBase):
    def test_documented_keys_and_json_serialisable_when_gh_is_unavailable(self) -> None:
        fake = FakeGh(fail="not authenticated")
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg, refresh=True)
        for key in ("schema", "generated_at", "repo", "milestones", "rules", "receipts", "prs", "delivery", "stale"):
            self.assertIn(key, data)
        self.assertFalse(data["prs"]["available"])
        self.assertIn("not authenticated", data["prs"]["reason"])
        json.dumps(data)  # must be JSON-serialisable
        rendered = status.render_status(data)
        self.assertIn("UNAVAILABLE", rendered)
        self.assertIn("1. MILESTONES", rendered)
        self.assertIn("2. RECEIPTS / VISIBILITY", rendered)
        self.assertIn("3. DELIVERY", rendered)
        self.assertIn("4. GITHUB", rendered)

    def test_build_status_never_raises_when_gh_helper_raises(self) -> None:
        with mock.patch.object(status, "gh_json", side_effect=RuntimeError("boom")):
            data = status.build_status(self.paths, self.reg, refresh=True)
        self.assertFalse(data["prs"]["available"])
        self.assertIn("boom", data["prs"]["reason"])
        self.assertFalse(data["delivery"]["branch_pr_check"]["available"])
        json.dumps(data)

    def test_milestone_fields_come_from_the_registry(self) -> None:
        fake = FakeGh(fail="gh: command not found")
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg)
        rows = {m["id"]: m for m in data["milestones"]}
        self.assertEqual(rows["M1"]["headline"], "M1: reply acceptable 37/232 -> target 116/232")
        self.assertEqual(rows["M1"]["target"], 116)
        self.assertEqual(rows["M1"]["issue"], 2029)
        self.assertEqual(rows["M1"]["lab"], "deepseek")
        self.assertEqual(rows["M1"]["line_counts"], "exact addressed memory 0/3")
        self.assertEqual(rows["M1"]["panel"], "v5-reply")
        self.assertEqual(rows["M1"]["mechanisms"][0]["id"], "exact-addressed-memory")
        self.assertFalse(rows["M1"]["mechanisms"][0]["exists"])  # no docs/mechanisms/ in the fixture
        self.assertTrue(rows["M1"]["next"])
        self.assertEqual([r["id"] for r in data["rules"]], ["D21", "D24"])

    def test_qualitative_headline_does_not_render_as_none(self) -> None:
        """M4's live headline has `text`/`target_display` and no numeric current."""
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            data = status.build_status(self.paths, self.reg)
        rows = {m["id"]: m for m in data["milestones"]}
        self.assertEqual(rows["M4"]["headline"], "M4: softmax at runtime: yes -> target no (D23 prohibits softmax project-wide)")
        self.assertNotIn("None", rows["M4"]["headline"])
        self.assertNotIn("None", status.render_status(data))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            status.heartbeat(argparse.Namespace(milestone="m4", line=None, count=None, next_step=None, json=False, post=False, print_card=False), self.paths, self.reg)
        self.assertNotIn("None", buffer.getvalue())
        self.assertIn("softmax at runtime: yes", buffer.getvalue())

    def test_receipts_counts_and_used_ids(self) -> None:
        self.write_receipts(self.receipts())
        fake = FakeGh(fail="offline")
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg)
        receipts = data["receipts"]
        self.assertTrue(receipts["available"])
        self.assertEqual(receipts["today"]["total"], 3)
        self.assertEqual(receipts["month"]["total"], 3)
        for kind in ("skill", "team", "heartbeat", "context", "prep", "pr"):
            self.assertIn(kind, receipts["month"]["by_kind"])
        self.assertEqual(receipts["month"]["by_kind"]["skill"], 1)
        self.assertEqual(receipts["used_ids"]["skill"], {"harness-status-check": 1})
        self.assertEqual(receipts["used_ids"]["team"], {"history-curator": 1})
        self.assertEqual(len(receipts["last5"]), 3)
        self.assertEqual(receipts["last5"][0]["kind"], "heartbeat")

    def test_ledger_fallback_when_receipts_module_is_absent(self) -> None:
        self.write_receipts(self.receipts())
        with mock.patch.dict(sys.modules, {"receipts": None}):
            rows, note = status._receipt_rows(self.paths, datetime.now(timezone.utc))
        self.assertEqual(len(rows), 3)
        self.assertEqual(note, "")

    def test_stale_section_flags_old_issue_and_missing_receipts(self) -> None:
        self.write_receipts([{"ts": now_iso(), "kind": "heartbeat", "milestone": "m1"}])
        fake = FakeGh()
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg)
        stale = data["stale"]
        self.assertTrue(stale["issues"]["available"])
        issues = {i["milestone"]: i for i in stale["issues"]["items"]}
        self.assertTrue(issues["M2"]["stale"])  # updated 30h ago
        self.assertFalse(issues["M1"]["stale"])
        milestones = {i["milestone"]: i for i in stale["milestones"]["items"]}
        self.assertFalse(milestones["M1"]["stale"])
        self.assertTrue(milestones["M2"]["stale"])
        self.assertIn("M2", stale["stale_milestones"])

    def test_stale_issues_are_unavailable_without_gh_but_receipts_still_work(self) -> None:
        self.write_receipts([{"ts": now_iso(), "kind": "heartbeat", "milestone": "m3"}])
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            data = status.build_status(self.paths, self.reg)
        self.assertFalse(data["stale"]["issues"]["available"])
        self.assertTrue(data["stale"]["milestones"]["available"])
        milestones = {i["milestone"]: i for i in data["stale"]["milestones"]["items"]}
        self.assertFalse(milestones["M3"]["stale"])
        self.assertTrue(milestones["M2"]["stale"])


class CacheTest(StatusTestBase):
    def setUp(self) -> None:
        super().setUp()
        self.merged = [
            {"number": 2257, "title": "the operator phase (References #2029)", "headRefName": "kimi/operator-phase-2026-10-12", "headRefOid": "b2d83de2", "url": "u", "mergedAt": iso(1.0)},
            {"number": 2254, "title": "README credit", "headRefName": "claude/ack", "headRefOid": "19f969c2", "url": "u", "mergedAt": iso(10.0)},
            {"number": 2100, "title": "old (References #2032)", "headRefName": "claude/old", "headRefOid": "aaaa", "url": "u", "mergedAt": iso(72.0)},
        ]

    def test_missing_cache_calls_gh_and_returns_live_data(self) -> None:
        fake = FakeGh(
            open_prs=[{"number": 2260, "title": "open (References #2030)", "headRefName": "codex/x", "url": "u", "createdAt": iso(2.0)}],
            merged_prs=self.merged,
        )
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg)
        self.assertFalse(data["prs"]["from_cache"])
        self.assertTrue(data["prs"]["available"])
        self.assertEqual(data["prs"]["open"]["count"], 1)
        self.assertEqual(data["prs"]["merged_48h"]["count"], 2)  # the 72h-old one is excluded
        self.assertTrue(fake.calls)
        self.assertTrue(status.cache_path(self.paths).exists())

    def test_fresh_cache_is_used_and_stale_cache_is_refetched(self) -> None:
        fake = FakeGh(merged_prs=self.merged[:1])
        with mock.patch.object(status, "gh_json", fake):
            first = status.build_status(self.paths, self.reg)
        self.assertFalse(first["prs"]["from_cache"])
        calls_after_first = len(fake.calls)

        fake.merged_prs = self.merged
        with mock.patch.object(status, "gh_json", fake):
            second = status.build_status(self.paths, self.reg)
        self.assertTrue(second["prs"]["from_cache"])
        self.assertEqual(len(fake.calls), calls_after_first)  # nothing was re-asked
        self.assertEqual(second["prs"]["merged_48h"]["count"], 1)

        # Age the cache past the 15-minute TTL: the next build must call gh again.
        cache_file = status.cache_path(self.paths)
        cache = json.loads(cache_file.read_text(encoding="utf-8"))
        cache["prs"]["fetched_at"] = iso(0.5)  # 30 minutes ago
        cache["issues"] = {}
        cache["branch_prs"] = {}
        cache_file.write_text(json.dumps(cache), encoding="utf-8")
        with mock.patch.object(status, "gh_json", fake):
            third = status.build_status(self.paths, self.reg)
        self.assertFalse(third["prs"]["from_cache"])
        self.assertGreater(len(fake.calls), calls_after_first)
        self.assertEqual(third["prs"]["merged_48h"]["count"], 2)

    def test_refresh_bypasses_a_fresh_cache(self) -> None:
        fake = FakeGh(merged_prs=self.merged[:1])
        with mock.patch.object(status, "gh_json", fake):
            status.build_status(self.paths, self.reg)
        calls = len(fake.calls)
        fake.merged_prs = self.merged
        with mock.patch.object(status, "gh_json", fake):
            data = status.build_status(self.paths, self.reg, refresh=True)
        self.assertGreater(len(fake.calls), calls)
        self.assertFalse(data["prs"]["from_cache"])
        self.assertEqual(data["prs"]["merged_48h"]["count"], 2)


class DegradationTest(StatusTestBase):
    def test_disk_failure_is_unavailable_not_zero(self) -> None:
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            data = status.build_status(self.paths, self.reg)
        with mock.patch.object(status.shutil, "disk_usage", side_effect=OSError("disk exploded")):
            with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
                data = status.build_status(self.paths, self.reg, refresh=True)
        disk = data["delivery"]["disk"]
        self.assertFalse(disk["available"])
        self.assertIn("disk exploded", disk["reason"])
        self.assertNotIn("free_gib", disk)
        rendered = status.render_status(data)
        self.assertIn("disk: UNAVAILABLE: shutil.disk_usage", rendered)
        self.assertNotIn("0 GiB", rendered)

    def test_worktree_failure_is_unavailable_not_zero(self) -> None:
        def fake_git(subargs, paths, timeout=20):
            if subargs[:2] == ["worktree", "list"]:
                return False, "", "`git worktree list --porcelain` exited 128: not a git repository"
            if subargs[:1] == ["rev-parse"]:
                return True, "deadbeef\n", ""
            if subargs[:1] == ["log"]:
                return True, "deadbeef subject\n", ""
            if subargs[:1] == ["for-each-ref"]:
                return True, "harness/v1\tdeadbeef\n", ""
            if subargs[:1] == ["status"]:
                return True, "", ""
            if subargs[:1] == ["merge-base"]:
                return True, "", ""
            if subargs[:1] == ["fetch"]:
                return True, "", ""
            if subargs[:1] == ["remote"]:
                return True, "git@github.com:UOR-Foundation/uor-r4.git\n", ""
            return True, "", ""

        with mock.patch.object(status, "_git", fake_git):
            with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
                data = status.build_status(self.paths, self.reg, refresh=True)
        worktrees = data["delivery"]["worktrees"]
        self.assertFalse(worktrees["available"])
        self.assertIn("not a git repository", worktrees["reason"])
        self.assertEqual(worktrees["count"], 0)
        self.assertEqual(worktrees["cleanup_candidates"], [])
        rendered = status.render_status(data)
        self.assertIn("worktrees: UNAVAILABLE:", rendered)
        self.assertNotIn("0 total", rendered)

    def test_delivery_failure_is_unavailable(self) -> None:
        def dead_git(subargs, paths, timeout=20):
            return False, "", f"`git {' '.join(subargs)}` could not run: executable not found (git)"

        with mock.patch.object(status, "_git", dead_git):
            with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
                data = status.build_status(self.paths, self.reg, refresh=True)
        self.assertFalse(data["delivery"]["available"])
        rendered = status.render_status(data)
        self.assertIn("UNAVAILABLE", rendered)

    def test_render_status_survives_partial_data(self) -> None:
        rendered = status.render_status({})
        self.assertIn("UNAVAILABLE", rendered)
        rendered = status.render_status({"milestones": [{"id": "M1"}], "receipts": {}, "prs": {}, "delivery": {}, "stale": {}})
        self.assertIn("1. MILESTONES", rendered)
        self.assertIsInstance(status.render_status("not a dict"), str)


class HeartbeatTest(StatusTestBase):
    def args(self, **overrides):
        base = {"milestone": "m1", "line": None, "count": None, "next_step": None, "json": False, "post": False, "print_card": False}
        base.update(overrides)
        return argparse.Namespace(**base)

    def run_card(self, **overrides) -> tuple[int, str]:
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = status.heartbeat(self.args(**overrides), self.paths, self.reg)
        return code, buffer.getvalue()

    def test_card_has_the_contract_shape(self) -> None:
        code, out = self.run_card()
        self.assertEqual(code, 0)
        self.assertIn("· count 0/3", out)
        self.assertIn("line: exact addressed memory", out)
        self.assertIn("M1: reply acceptable 37/232 -> target 116/232", out)
        self.assertIn("next: re-register the cycle-8 run", out)
        self.assertIn("moves the headline", out)

    def test_overrides_change_line_count_and_next(self) -> None:
        code, out = self.run_card(line="protected/discrete constructor", count="6/3", next_step="resolve the gates")
        self.assertEqual(code, 0)
        self.assertIn("line: protected/discrete constructor", out)
        self.assertIn("count 6/3", out)
        self.assertIn("next: resolve the gates", out)

    def test_default_line_is_the_one_closest_to_its_pivot(self) -> None:
        code, out = self.run_card(milestone="m4")
        self.assertEqual(code, 0)
        self.assertIn("line: starling read", out)
        self.assertIn("count 1/3", out)

    def test_json_card_is_a_single_object(self) -> None:
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = status.heartbeat(self.args(json=True), self.paths, self.reg)
        self.assertEqual(code, 0)
        payload = json.loads(buffer.getvalue())
        self.assertEqual(payload["milestone"], "m1")
        self.assertIn("· count 0/3", payload["card"])
        self.assertFalse(payload["posted"])

    def test_unknown_milestone_is_a_usage_error(self) -> None:
        buffer = io.StringIO()
        with contextlib.redirect_stderr(buffer):
            code = status.heartbeat(self.args(milestone="nope"), self.paths, self.reg)
        self.assertEqual(code, 2)
        self.assertIn("unknown milestone", buffer.getvalue())

    def test_heartbeat_survives_a_missing_receipts_module(self) -> None:
        with mock.patch.dict(sys.modules, {"receipts": None}):
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = status.heartbeat(self.args(), self.paths, self.reg)
        self.assertEqual(code, 0)
        self.assertIn("· count 0/3", buffer.getvalue())
        ledger = list(self.paths.ledger_dir.glob("*.jsonl"))
        self.assertFalse([p for p in ledger if p.read_text(encoding="utf-8").strip()])

    def test_heartbeat_records_a_receipt_without_posting(self) -> None:
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = status.heartbeat(self.args(), self.paths, self.reg)
        self.assertEqual(code, 0)
        files = [p for p in self.paths.ledger_dir.glob("*.jsonl") if p.read_text(encoding="utf-8").strip()]
        if not files:  # the receipts module is optional
            self.skipTest("receipts module not importable in this checkout")
        rows = [json.loads(l) for l in files[0].read_text(encoding="utf-8").splitlines() if l.strip()]
        self.assertTrue(any(r.get("kind") == "heartbeat" and r.get("milestone") == "m1" for r in rows))

    def test_heartbeat_never_posts_unless_asked(self) -> None:
        with mock.patch.object(status, "_post_card") as post:
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = status.heartbeat(self.args(), self.paths, self.reg)
        self.assertEqual(code, 0)
        post.assert_not_called()

    def test_heartbeat_post_is_opt_in_and_reports_failure(self) -> None:
        with mock.patch.object(status, "_post_card", return_value=(True, "", "https://github.com/x/y/issues/2029#issuecomment-1")) as post:
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = status.heartbeat(self.args(post=True), self.paths, self.reg)
        self.assertEqual(code, 0)
        post.assert_called_once()
        self.assertIn("· count 0/3", buffer.getvalue())

        with mock.patch.object(status, "_post_card", return_value=(False, "`gh issue comment 2029 --body-file -` exited 1: offline", "")):
            buffer = io.StringIO()
            errors = io.StringIO()
            with contextlib.redirect_stdout(buffer), contextlib.redirect_stderr(errors):
                code = status.heartbeat(self.args(post=True, json=True), self.paths, self.reg)
        self.assertEqual(code, 1)
        payload = json.loads(buffer.getvalue())
        self.assertFalse(payload["posted"])
        self.assertIn("gh issue comment 2029 --body-file -", payload["post_command"])


class RuleTest(StatusTestBase):
    def test_register_rules_ids_and_severities(self) -> None:
        rules = status.register_rules()
        self.assertEqual(
            [r.id for r in rules],
            [
                "status.heartbeat_receipts",
                "status.registry_vs_github",
                "status.no_orphan_branches",
                "status.protected_branches_present",
            ],
        )
        # A missing protected branch (main, gh-pages, gh-pages-preview) is an error; the rest warn.
        self.assertEqual(
            [r.severity for r in rules], ["warn", "warn", "warn", "error"]
        )
        self.assertTrue(all(callable(r.check) for r in rules))

    def test_every_check_never_raises_when_everything_is_broken(self) -> None:
        broken_paths = HarnessPaths.discover(repo=self.repo / "does-not-exist", state=self.repo / ".harness-broken")
        with mock.patch.object(status, "gh_json", side_effect=RuntimeError("boom")):
            with mock.patch.object(status, "_git", side_effect=RuntimeError("git boom")):
                for rule in status.register_rules():
                    results = rule.check(broken_paths, self.reg)
                    self.assertIsInstance(results, list)
                    for r in results:
                        self.assertIsInstance(r, status.RuleResult)

    def test_heartbeat_rule_flags_milestones_without_a_receipt(self) -> None:
        rule = [r for r in status.register_rules() if r.id == "status.heartbeat_receipts"][0]
        results = rule.check(self.paths, self.reg)
        self.assertEqual({r.meta.get("milestone") for r in results}, {"m1", "m2", "m3", "m4"})
        self.assertTrue(all("cli.py heartbeat" in r.fix for r in results))

        self.write_receipts([{"ts": now_iso(), "kind": "heartbeat", "milestone": "m1"}])
        results = rule.check(self.paths, self.reg)
        self.assertNotIn("m1", {r.meta.get("milestone") for r in results})

    def test_registry_vs_github_warns_and_does_not_pass_without_gh(self) -> None:
        """**A check that cannot run is not a check that passed.**

        This asserted `ok=True` and `severity="info"` - so a lab without GitHub access got a green
        `check` on the rule whose entire job is to notice that `milestones.json` and the milestone
        issue have drifted apart. **The test named the defect and pinned it as intended behaviour.**
        `warn`/`ok=False` now, because a permanently-red check gets ignored and a silent green one
        is worse.
        """
        rule = [r for r in status.register_rules() if r.id == "status.registry_vs_github"][0]
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            results = rule.check(self.paths, self.reg)
        self.assertEqual(len(results), 1)
        self.assertFalse(results[0].ok, "a check that could not run is not ok")
        self.assertEqual(results[0].severity, "warn")
        self.assertIn("NOT cross-checked", results[0].message)

    def test_registry_vs_github_compares_the_newest_card(self) -> None:
        rule = [r for r in status.register_rules() if r.id == "status.registry_vs_github"][0]
        matching = {2029: issue(2029, 1.0, [{"createdAt": iso(1.0), "body": "**STATUS CARD** headline: reply 37/232 (target 116) · count 0/3"}])}
        with mock.patch.object(status, "gh_json", FakeGh(issues=matching)):
            results = rule.check(self.paths, self.reg)
        self.assertFalse([r for r in results if not r.ok])  # M1 agrees; the others have no card

        disagreeing = {2029: issue(2029, 1.0, [{"createdAt": iso(1.0), "body": "**STATUS CARD** M1: 12/232 -> target 116/232"}])}
        status.cache_path(self.paths).unlink(missing_ok=True)  # a fresh cache would hide the new card
        with mock.patch.object(status, "gh_json", FakeGh(issues=disagreeing)):
            results = rule.check(self.paths, self.reg)
        warns = [r for r in results if not r.ok]
        self.assertEqual(len(warns), 1)
        self.assertIn("12/232", warns[0].message)
        self.assertIn("37/232", warns[0].message)

    def test_registry_vs_github_quotes_a_card_with_no_number(self) -> None:
        rule = [r for r in status.register_rules() if r.id == "status.registry_vs_github"][0]
        issues = {2029: issue(2029, 1.0, [{"createdAt": iso(1.0), "body": "**BLOCKED CARD** headline: durable qualification UNQUALIFIED (target: acceptance)"}] )}
        with mock.patch.object(status, "gh_json", FakeGh(issues=issues)):
            results = rule.check(self.paths, self.reg)
        info = [r for r in results if r.ok and r.meta.get("milestone") == "m1"]
        self.assertEqual(len(info), 1)
        self.assertIn("UNQUALIFIED", info[0].message)

    def test_registry_vs_github_warns_when_a_card_answers_yes_no(self) -> None:
        rule = [r for r in status.register_rules() if r.id == "status.registry_vs_github"][0]
        issues = {2029: issue(2029, 1.0, [{"createdAt": iso(1.0), "body": "**BLOCKED CARD** headline: reply acceptable yes, panel unchanged"}] )}
        with mock.patch.object(status, "gh_json", FakeGh(issues=issues)):
            results = rule.check(self.paths, self.reg)
        warns = [r for r in results if not r.ok]
        self.assertEqual(len(warns), 1)
        self.assertIn("'yes'", warns[0].message)
        self.assertIn("37/232", warns[0].message)

    def test_no_orphan_branches_degrades_without_git_or_gh(self) -> None:
        rule = [r for r in status.register_rules() if r.id == "status.no_orphan_branches"][0]
        with mock.patch.object(status, "_git", side_effect=RuntimeError("git boom")):
            with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
                results = rule.check(self.paths, self.reg)
        self.assertTrue(all(r.ok for r in results))
        self.assertTrue(all(r.severity == "info" for r in results))


class DeliveryWorktreeTest(StatusTestBase):
    """Real git in a temp repo: a fresh worktree must not look like completed work."""

    def setUp(self) -> None:
        super().setUp()
        run = subprocess.run(["git", "init", "-q", "-b", "main", str(self.repo)], capture_output=True, text=True, check=False)
        if run.returncode != 0:
            self.skipTest(f"git init unavailable: {run.stderr.strip()}")
        (self.repo / "file.txt").write_text("x\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.repo), "add", "file.txt"], capture_output=True, text=True, check=False)
        subprocess.run(
            ["git", "-C", str(self.repo), "-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"],
            capture_output=True,
            text=True,
            check=False,
        )
        head = subprocess.run(["git", "-C", str(self.repo), "rev-parse", "HEAD"], capture_output=True, text=True, check=False).stdout.strip()
        subprocess.run(["git", "-C", str(self.repo), "update-ref", "refs/remotes/origin/main", head], capture_output=True, text=True, check=False)
        self.worktree = self.repo / "wt-feature"
        subprocess.run(
            ["git", "-C", str(self.repo), "worktree", "add", "-q", "-b", "feat", str(self.worktree)],
            capture_output=True,
            text=True,
            check=False,
        )

    def build(self) -> dict:
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            return status.build_status(self.paths, self.reg, refresh=True)

    def worktree_item(self, data: dict) -> dict:
        items = {Path(i["path"]).name: i for i in data["delivery"]["worktrees"]["items"]}
        self.assertIn("wt-feature", items)
        return items["wt-feature"]

    def test_fresh_clean_worktree_at_origin_main_is_not_completed_work(self) -> None:
        item = self.worktree_item(self.build())
        self.assertTrue(item["clean"])
        self.assertTrue(item["contained_in_origin_main"])
        self.assertFalse(item["merged_pr_head"])
        self.assertFalse(item["cleanup_candidate"])
        self.assertFalse(item["stale"])
        rule = [r for r in status.register_rules() if r.id == "status.no_orphan_branches"][0]
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            results = rule.check(self.paths, self.reg)
        self.assertFalse([r for r in results if not r.ok and r.meta.get("kind") == "worktree"])

    def test_idle_clean_worktree_is_reported_as_completed_work(self) -> None:
        old = (datetime.now(timezone.utc) - timedelta(hours=30)).timestamp()
        for root, dirs, files in os.walk(self.worktree):
            for name in files:
                os.utime(os.path.join(root, name), (old, old))
        item = self.worktree_item(self.build())
        self.assertTrue(item["idle"])
        self.assertTrue(item["cleanup_candidate"])
        self.assertEqual(item["reapable_reason"], "contained-in-origin-main")
        rule = [r for r in status.register_rules() if r.id == "status.no_orphan_branches"][0]
        with mock.patch.object(status, "gh_json", FakeGh(fail="offline")):
            results = rule.check(self.paths, self.reg)
        worktree_warns = [r for r in results if not r.ok and r.meta.get("kind") == "worktree"]
        self.assertEqual(len(worktree_warns), 1)
        self.assertIn("24h", worktree_warns[0].message)
        self.assertIn("git worktree remove", worktree_warns[0].fix)

    def test_a_branch_with_a_merged_pr_is_an_orphan(self) -> None:
        head = subprocess.run(["git", "-C", str(self.repo), "rev-parse", "feat"], capture_output=True, text=True, check=False).stdout.strip()
        branch_prs = {"feat": [{"number": 42, "title": "feat (References #2029)", "state": "MERGED", "headRefOid": head, "mergedAt": iso(5.0), "url": "u"}]}
        with mock.patch.object(status, "gh_json", FakeGh(branch_prs=branch_prs)):
            data = status.build_status(self.paths, self.reg, refresh=True)
        orphans = {o["branch"]: o for o in data["delivery"]["branches_with_merged_pr"]}
        self.assertIn("feat", orphans)
        self.assertTrue(orphans["feat"]["head_matches"])
        self.assertNotIn("main", orphans)  # the trunk is never an orphan


class GitHelpHookTest(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-githelp-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        run = subprocess.run(["git", "init", "-q", "-b", "main", str(self.repo)], capture_output=True, text=True, check=False)
        if run.returncode != 0:
            self.skipTest(f"git init unavailable: {run.stderr.strip()}")
        (self.repo / "file.txt").write_text("x\n", encoding="utf-8")
        for cmd in (["git", "-C", str(self.repo), "add", "file.txt"], ["git", "-C", str(self.repo), "-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"]):
            subprocess.run(cmd, capture_output=True, text=True, check=False)
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")

    def hook_path(self) -> Path:
        return self.repo / ".git" / "hooks" / "pre-commit"

    def test_install_is_idempotent_and_never_appends(self) -> None:
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(githelp.install_hooks(argparse.Namespace(), self.paths), 0)
            first = self.hook_path().read_text(encoding="utf-8")
            self.assertEqual(githelp.install_hooks(argparse.Namespace(), self.paths), 0)
            second = self.hook_path().read_text(encoding="utf-8")
        self.assertEqual(first, second)
        self.assertEqual(second.count(githelp.HOOK_START), 1)
        self.assertIn("check --staged", second)
        self.assertTrue(os.access(self.hook_path(), os.X_OK))
        self.assertTrue(second.startswith("#!/bin/sh"))

    def test_installed_hook_allows_a_commit_without_python3_or_the_harness(self) -> None:
        with contextlib.redirect_stdout(io.StringIO()):
            githelp.install_hooks(argparse.Namespace(), self.paths)
        # No scripts/harness/cli.py in this repo: the hook must exit 0 and stay quiet.
        run = subprocess.run(["/bin/sh", str(self.hook_path())], cwd=str(self.repo), capture_output=True, text=True, check=False)
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual(run.stdout.strip(), "")

    def test_a_foreign_hook_is_never_overwritten(self) -> None:
        self.hook_path().parent.mkdir(parents=True, exist_ok=True)
        foreign = "#!/bin/sh\necho my own hook\n"
        self.hook_path().write_text(foreign, encoding="utf-8")
        errors = io.StringIO()
        with contextlib.redirect_stderr(errors):
            code = githelp.install_hooks(argparse.Namespace(), self.paths)
        self.assertEqual(code, 2)
        self.assertIn("CONFLICT", errors.getvalue())
        self.assertIn("pre-commit", errors.getvalue())
        self.assertEqual(self.hook_path().read_text(encoding="utf-8"), foreign)

    def test_a_symlinked_hook_is_never_overwritten(self) -> None:
        target = self.repo / "elsewhere.sh"
        target.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        self.hook_path().parent.mkdir(parents=True, exist_ok=True)
        self.hook_path().symlink_to(target)
        errors = io.StringIO()
        with contextlib.redirect_stderr(errors):
            code = githelp.install_hooks(argparse.Namespace(), self.paths)
        self.assertEqual(code, 2)
        self.assertIn("symlink", errors.getvalue())
        self.assertTrue(self.hook_path().is_symlink())

    # --- the safety property: warn/info pass, error blocks, staged content is untouched

    def _stub_cli(self, payload: dict) -> None:
        cli = self.repo / "scripts" / "harness" / "cli.py"
        cli.parent.mkdir(parents=True, exist_ok=True)
        cli.write_text(f"import json\nprint(json.dumps({payload!r}))\n", encoding="utf-8")

    def _install_and_run_hook(self) -> subprocess.CompletedProcess[str]:
        with contextlib.redirect_stdout(io.StringIO()):
            githelp.install_hooks(argparse.Namespace(), self.paths)
        return subprocess.run(["/bin/sh", str(self.hook_path())], cwd=str(self.repo), capture_output=True, text=True, check=False)

    def test_hook_passes_warnings_and_info(self) -> None:
        self._stub_cli(
            {
                "results": [
                    {"rule_id": "x.warn", "severity": "warn", "ok": False, "message": "careful", "fix": "do it"},
                    {"rule_id": "x.info", "severity": "info", "ok": False, "message": "fyi", "fix": ""},
                ]
            }
        )
        run = self._install_and_run_hook()
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertIn("harness warn: x.warn: careful", run.stdout)
        self.assertNotIn("ERROR", run.stdout)

    def test_hook_blocks_errors_only(self) -> None:
        self._stub_cli({"results": [{"rule_id": "x.err", "severity": "error", "ok": False, "message": "broken", "fix": "fix it"}]})
        run = self._install_and_run_hook()
        self.assertEqual(run.returncode, 1)
        self.assertIn("harness ERROR: x.err: broken", run.stdout)
        self.assertIn("fix: fix it", run.stdout)
        self.assertIn("commit blocked", run.stdout)

    def test_hook_allows_the_commit_when_the_harness_output_is_unreadable(self) -> None:
        cli = self.repo / "scripts" / "harness" / "cli.py"
        cli.parent.mkdir(parents=True, exist_ok=True)
        cli.write_text("raise SystemExit(3)\n", encoding="utf-8")
        run = self._install_and_run_hook()
        self.assertEqual(run.returncode, 0, run.stderr)

    def test_hook_does_not_modify_staged_content(self) -> None:
        self._stub_cli({"results": [{"rule_id": "x.warn", "severity": "warn", "ok": False, "message": "careful", "fix": ""}]})
        (self.repo / "file.txt").write_text("staged change\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.repo), "add", "file.txt"], capture_output=True, text=True, check=False)
        before = subprocess.run(["git", "-C", str(self.repo), "diff", "--cached"], capture_output=True, text=True, check=False).stdout
        run = self._install_and_run_hook()
        after = subprocess.run(["git", "-C", str(self.repo), "diff", "--cached"], capture_output=True, text=True, check=False).stdout
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual(before, after)
        self.assertNotIn("staged change", run.stdout)


class GitHelpStatusTest(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-gitstatus-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        run = subprocess.run(["git", "init", "-q", "-b", "main", str(self.repo)], capture_output=True, text=True, check=False)
        if run.returncode != 0:
            self.skipTest(f"git init unavailable: {run.stderr.strip()}")
        (self.repo / "file.txt").write_text("x\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.repo), "add", "file.txt"], capture_output=True, text=True, check=False)
        subprocess.run(["git", "-C", str(self.repo), "-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"], capture_output=True, text=True, check=False)
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")

    def test_status_reports_state_and_a_next_command_without_gh(self) -> None:
        with mock.patch.object(githelp, "_gh", lambda subargs, paths, timeout=20: (False, None, "`gh pr view` exited 1: no pull requests found")):
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = githelp.status(argparse.Namespace(json=False), self.paths)
        out = buffer.getvalue()
        self.assertEqual(code, 0)
        self.assertIn("branch main", out)
        self.assertIn("no pull requests found", out)
        self.assertIn("git push -u origin main", out)
        self.assertIn("UNAVAILABLE", out)  # no origin/main in a local-only repo

    def test_status_json_path(self) -> None:
        with mock.patch.object(githelp, "_gh", lambda subargs, paths, timeout=20: (False, None, "offline")):
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = githelp.status(argparse.Namespace(json=True), self.paths)
        self.assertEqual(code, 0)
        payload = json.loads(buffer.getvalue())
        self.assertEqual(payload["branch"], "main")
        self.assertFalse(payload["pr"]["available"])
        self.assertEqual(payload["state"], "no-pr")
        self.assertIn("git push", payload["next_command"])


class TestRemoteBranchesWithoutPr(unittest.TestCase):
    """The half of the branch list the merged-PR check cannot see: work that was pushed and never
    opened as a PR. Such a branch holds unmerged work, so the answer is archive-or-PR, never delete.
    """

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-branch-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        (self.repo / "docs" / "harness" / "ledger").mkdir(parents=True, exist_ok=True)
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")

    def _rows(self) -> list:
        return githelp.remote_branches_without_pr(self.paths)

    def test_it_never_returns_a_fully_merged_branch(self) -> None:
        """A branch with nothing ahead of main is finished on its own merits; the merged-PR path
        handles it, and reporting it here would double-count."""
        for r in self._rows():
            self.assertGreater(r["ahead"], 0, f"{r['branch']} is not ahead of main")

    def test_the_published_pages_branch_is_excluded(self) -> None:
        """gh-pages is published by GitHub, not delivery transport. Flagging it is noise that
        trains people to ignore the rule."""
        self.assertNotIn("gh-pages", [r["branch"] for r in self._rows()])

    def test_it_is_advisory_and_never_raises(self) -> None:
        """A advisory rule must not fail a check when git or gh is unavailable."""
        try:
            self._rows()
        except Exception as exc:  # noqa: BLE001
            self.fail(f"the branch scan raised {type(exc).__name__}: {exc}")


class TestGuardedBranchDelete(unittest.TestCase):
    """Deleting a branch must be PROVEN, because an unmerged remote branch is usually the only copy.

    This exists because of a real mistake: `gh pr merge --auto` was followed by a poll and then an
    UNCONDITIONAL remote delete. The PR auto-closed as DIRTY mid-poll, the loop exited without
    noticing, and the delete ran anyway - removing the only remote copy of unmerged work. It was
    recovered from a local commit, but nothing in the sequence prevented the loss.
    """

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-delete-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        (self.repo / "docs" / "harness").mkdir(parents=True, exist_ok=True)
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")

    def test_a_branch_with_no_merged_pr_is_refused(self) -> None:
        """The default answer is NO, whatever the reason for not being able to prove a merge.

        Asserted on `safe_to_delete` - the flag the caller acts on - rather than on the wording of
        the reason. The first version of this test checked for a word in the message, which is
        testing prose; the property that matters is that nothing is deleted without proof.
        """
        r = githelp.delete_merged_branch(self.paths, "someone/unmerged-work")
        self.assertFalse(r["safe_to_delete"], r)
        self.assertTrue(r["reason"], "a refusal must say why")

    def test_a_claimed_branch_is_refused_even_when_merged(self) -> None:
        """A merged PR is NOT proof a branch is finished.

        A lab can merge and keep going, or hold the branch while a follow-up lands. The first
        version recommended deleting `kimi/plan-refresh-2026-10-13` - merged, but under Kimi's live
        claim - which would have destroyed a peer's working branch.
        """
        (self.repo / "docs" / "harness" / "claims.json").write_text(
            json.dumps({"someone/claimed": {"branch": "someone/claimed", "lab": "kimi", "claimed_at": "2026-10-13T00:00:00-04:00"}}),
            encoding="utf-8",
        )
        r = githelp.delete_merged_branch(self.paths, "someone/claimed")
        self.assertFalse(r["safe_to_delete"], r)
        self.assertIn("live claim", r["reason"])
        self.assertIn("team release", r["reason"], "the refusal must say how to proceed")

    def test_the_content_proof_is_git_cherry_not_ancestry_or_a_tree_diff(self) -> None:
        """The guard got this wrong THREE times, in three different directions. The test pins the
        question it means to ask: does this branch introduce anything not already upstream?

          * ancestry (`rev-list --count main..branch`): a SQUASH merge leaves no ancestor, so it
            reported 2 commits for a branch byte-identical to main and refused every squash-merged
            branch — most of them here.
          * a whole-tree diff (`git diff --stat main..branch`): a branch merely BEHIND main shows as
            removing main's newer files, so it refused a branch whose PR had just merged, flagging
            five files belonging to other labs that this branch never touched.
          * `git cherry`: '-' means the commit's changes are already upstream. That is the question.
        """
        import re

        source = (HARNESS_DIR / "githelp.py").read_text(encoding="utf-8")
        body = source[source.index("def delete_merged_branch"):]
        body = body[: body.index("\ndef ", 10)]
        calls = re.findall(r'git\(\s*"([a-z-]+)"', body)
        self.assertIn("cherry", calls, f"git cherry is the content proof: {calls}")
        self.assertNotIn("rev-list", calls, f"ancestry cannot see a squash merge: {calls}")

    def test_reporting_changes_nothing(self) -> None:
        """Without --apply it must be read-only: no deletion, whatever the verdict."""
        r = githelp.delete_merged_branch(self.paths, "someone/unmerged-work", apply=False)
        self.assertFalse(r.get("deleted"), r)

    def test_the_flag_and_the_reason_never_disagree(self) -> None:
        """The bug that took two attempts: a guard set the reason but left the flag, so a REFUSED
        branch still printed 'would delete'. The flag is what acts; the reason is what a human
        reads, and they must agree."""
        (self.repo / "docs" / "harness" / "claims.json").write_text(
            json.dumps({"x/y": {"branch": "x/y", "lab": "codex", "claimed_at": "2026-10-13T00:00:00-04:00"}}),
            encoding="utf-8",
        )
        for branch in ("x/y", "no/merged-pr"):
            with self.subTest(branch=branch):
                r = githelp.delete_merged_branch(self.paths, branch)
                self.assertFalse(r["safe_to_delete"], f"{branch}: {r}")
                self.assertNotIn("merged as", r["reason"], f"{branch}: reason claims a merge")


class TestClaimsPreservingCheckout(unittest.TestCase):
    """`git switch` must not discard an uncommitted claim.

    `docs/harness/claims.json` is tracked (so a claim reaches the other labs) and appended to by
    every claim/release, so the ordinary "claim, then check something out" sequence fails with
    "local changes would be overwritten". The easy workaround - `git checkout -f` - discards the
    claim, and an uncommitted claim is still a lab saying "I am working here".
    """

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="harness-switch-test-")
        self.addCleanup(self._tmp.cleanup)
        self.repo = Path(self._tmp.name)
        (self.repo / "docs" / "harness").mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "init", "-q"], cwd=self.repo, check=False)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=self.repo, check=False)
        subprocess.run(["git", "config", "user.name", "t"], cwd=self.repo, check=False)
        (self.repo / "README").write_text("x", encoding="utf-8")
        self.paths = HarnessPaths.discover(repo=self.repo, state=self.repo / ".harness")
        subprocess.run(["git", "add", "-A"], cwd=self.repo, check=False)
        subprocess.run(["git", "commit", "-qm", "init"], cwd=self.repo, check=False)
        self.claims = self.repo / "docs" / "harness" / "claims.json"
        self.claims.write_text('{"a/b": {"branch": "a/b", "lab": "kimi"}}', encoding="utf-8")
        subprocess.run(["git", "add", "-A"], cwd=self.repo, check=False)
        subprocess.run(["git", "commit", "-qm", "claims"], cwd=self.repo, check=False)
        subprocess.run(["git", "branch", "other"], cwd=self.repo, check=False)

    def test_an_uncommitted_claim_survives_a_checkout(self) -> None:
        """The property the command exists for."""
        self.claims.write_text('{"a/b": {"branch": "a/b", "lab": "kimi"}, "mine/x": {"branch": "mine/x", "lab": "deepseek"}}', encoding="utf-8")
        r = githelp.checkout_preserving_claims(self.paths, "other")
        self.assertTrue(r["switched"], r)
        self.assertTrue(r["claims_preserved"], r)
        self.assertIn("mine/x", self.claims.read_text(encoding="utf-8"), "the uncommitted claim must survive")

    def test_a_clean_tree_is_not_disturbed(self) -> None:
        r = githelp.checkout_preserving_claims(self.paths, "other")
        self.assertTrue(r["switched"], r)
        self.assertFalse(r["claims_preserved"], "nothing was uncommitted, so nothing needed carrying")
        self.assertIn("a/b", self.claims.read_text(encoding="utf-8"))

    def test_a_failed_checkout_reports_it(self) -> None:
        """A switch that did not happen must say so rather than reporting success."""
        r = githelp.checkout_preserving_claims(self.paths, "no-such-branch")
        self.assertFalse(r["switched"], r)
        self.assertIn("error", r)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()


class TestRegistryVsGithubCannotRun(unittest.TestCase):
    """A check that cannot run must not report as a pass.

    **Both unavailability paths returned `info` with `ok=True`**, so a lab without GitHub access got
    a green `check` on the rule whose entire job is to notice that `milestones.json` and the
    milestone issue have drifted apart. Same defect as `harness.state_formats`, same fix.
    """

    def _paths_and_reg(self):
        import core
        import registries

        repo = pathlib.Path(__file__).resolve().parents[2]
        paths = core.HarnessPaths(
            repo=repo, harness=repo / "docs" / "harness", state=repo / ".harness"
        )
        return paths, registries.Registry.load(paths)

    def test_a_raising_lookup_reports_a_warning_not_a_pass(self):
        import status

        paths, reg = self._paths_and_reg()
        original = status.github_issues
        try:
            status.github_issues = lambda *a, **k: (_ for _ in ()).throw(
                RuntimeError("gh exploded")
            )
            results = status._rule_registry_vs_github(paths, reg)
        finally:
            status.github_issues = original
        self.assertEqual(len(results), 1)
        self.assertFalse(results[0].ok, "a lookup that raised is not a pass")
        self.assertEqual(results[0].severity, "warn")
        self.assertIn("NOT cross-checked", results[0].message)

    def test_github_reported_unavailable_reports_a_warning_not_a_pass(self):
        import status

        paths, reg = self._paths_and_reg()
        original = status.github_issues
        try:
            status.github_issues = lambda *a, **k: {"available": False, "reason": "rate limited"}
            results = status._rule_registry_vs_github(paths, reg)
        finally:
            status.github_issues = original
        self.assertEqual(len(results), 1)
        self.assertFalse(results[0].ok, "GitHub being unavailable is not a pass")
        self.assertEqual(results[0].severity, "warn")
        self.assertIn("NOT cross-checked", results[0].message)
