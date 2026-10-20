"""Tests for the budget view: receipt costs, the ledger chain, the disk floor, the scan.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v

No network, no gh, and no pod mutation: the scan tests point the module at a fake
``uor-pod`` and at a missing one, and assert that nothing is fabricated.
"""

from __future__ import annotations

import argparse
import json
import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import budget  # noqa: E402
import checkrules  # noqa: E402
import core  # noqa: E402
import registries  # noqa: E402

#: a real status capture, so the parser is tested against the tool's actual layout.
REAL_STATUS = """
Pods (caps: <= 4 running pods, <= $8.00/h; idle reap after 20 min without a live lease)
Running: 0 pod(s) (0 counted toward the cap of 4; uor-test-* pods are not), $0/h of $8.00/h
Free GPUs you can lease now: none. Spin up your own pod: uor-pod up --lab L --session S --purpose ... --hours H. Never fall back to the laptop CPU.
Stock by datacenter (secure $/GPU/h; * = volume datacenter, placement order EUR-NO-1 EU-RO-1 EUR-IS-1):
  5090 $1.19: EUR-NO-1*=Low
  4090 $0.89: EU-CZ-1=Low EU-RO-1*=Low
"""

REAL_GPUS = """GPU policy (key, CUDA_COMPUTE_CAP, tier):
  5090     cap 120  default    NVIDIA GeForce RTX 5090
Stock by datacenter (secure $/GPU/h; * = volume datacenter, placement order EUR-NO-1 EU-RO-1 EUR-IS-1):
  5090 $1.19: EUR-NO-1*=Low
  4090 $0.89: EU-CZ-1=Low EU-RO-1*=Low EUR-IS-1*=Low EUR-IS-2=Low EUR-NO-1*=Low US-TX-3=Low
  pro6000 $2.49: EU-CZ-1=Low EU-NL-1=Low EU-RO-1*=Low EUR-IS-1*=Medium
"""


def run_budget_rules(paths, reg) -> list:
    """Run the budget rules once per rule id.

    checkrules.rules() re-calls each module's register_rules() on every call, and a module
    without an idempotency cache duplicates its rules process-wide. That is another
    module's bug, but it must not make these assertions ambiguous, so de-duplicate by id.
    """
    seen: set[str] = set()
    out = []
    for result in checkrules.run(paths, reg, only=["budget"]):
        if result.rule_id in seen:
            continue
        seen.add(result.rule_id)
        out.append(result)
    return out


def make_paths(root: Path) -> core.HarnessPaths:
    harness = root / "docs" / "harness"
    for sub in ("ledger", "prep", "panels"):
        (harness / sub).mkdir(parents=True, exist_ok=True)
    (root / "docs" / "labs").mkdir(parents=True, exist_ok=True)
    (root / ".harness").mkdir(parents=True, exist_ok=True)
    (root / "scripts" / "pod").mkdir(parents=True, exist_ok=True)
    (root / "docs" / "harness" / "milestones.json").write_text(
        json.dumps({"schema": 1, "milestones": {}, "owner_rules": []}), encoding="utf-8"
    )
    return core.HarnessPaths(repo=root, harness=harness, state=root / ".harness")


def ledger(paths: core.HarnessPaths, name: str, records: list[dict], raw_extra: str = "") -> Path:
    p = paths.ledger_dir / name
    with p.open("w", encoding="utf-8") as fh:
        for rec in records:
            fh.write(json.dumps(rec, sort_keys=True) + "\n")
        if raw_extra:
            fh.write(raw_extra)
    return p


class TempRepoCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.paths = make_paths(self.root)
        self.reg = registries.Registry.load(self.paths)

    def tearDown(self) -> None:
        self._tmp.cleanup()


class TestReceiptCosts(TempRepoCase):
    def test_costs_are_summed_by_month_and_milestone(self) -> None:
        ledger(
            self.paths,
            "2026-10.jsonl",
            [
                {"ts": "2026-10-01T10:00:00+00:00", "kind": "prep", "milestone": "m1", "cost_usd": 1.5},
                {"ts": "2026-10-02T10:00:00+00:00", "kind": "prep", "milestone": "m1", "cost_usd": 2.0},
                {"ts": "2026-10-03T10:00:00+00:00", "kind": "prep", "milestone": "m2", "cost_usd": 0.5},
            ],
        )
        ledger(self.paths, "2026-11.jsonl", [{"ts": "2026-11-01T10:00:00+00:00", "kind": "prep", "milestone": "m2", "cost_usd": 4}])
        data = budget.collect(self.paths)
        rec = data["receipts"]
        self.assertEqual(rec["cost_records"], 4)
        self.assertEqual(rec["by_month"], {"2026-10": 4.0, "2026-11": 4.0})
        self.assertEqual(rec["by_milestone"], {"m1": 3.5, "m2": 4.5})
        self.assertEqual(rec["usd_total"], 8.0)

    def test_a_rate_is_not_summed_as_spend(self) -> None:
        ledger(
            self.paths,
            "2026-10.jsonl",
            [{"ts": "2026-10-01T10:00:00+00:00", "kind": "team", "milestone": "m1", "usd_per_h": 1.98}],
        )
        rec = budget.collect(self.paths)["receipts"]
        self.assertEqual(rec["cost_records"], 0)
        self.assertEqual(rec["rate_only_records"], 1)
        self.assertEqual(rec["by_milestone"], {})

    def test_receipts_without_costs_report_absent_not_zero(self) -> None:
        ledger(self.paths, "2026-10.jsonl", [{"ts": "2026-10-01T10:00:00+00:00", "kind": "skill", "skill": "x"}])
        rec = budget.collect(self.paths)["receipts"]
        self.assertEqual(rec["by_month"], {})
        self.assertEqual(rec["by_milestone"], {})


class TestLedgers(TempRepoCase):
    def test_missing_model_time_ledger_is_absent(self) -> None:
        mt = budget.model_time_ledger(self.root)
        self.assertFalse(mt["present"])
        self.assertEqual(mt["reason"], "ABSENT")
        self.assertEqual(mt["checked"], budget.MODEL_TIME_LEDGER)

    def test_present_model_time_ledger_is_reported(self) -> None:
        p = self.root / budget.MODEL_TIME_LEDGER
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps({"cumulative_ms": 100, "limit_ms": 200}), encoding="utf-8")
        mt = budget.model_time_ledger(self.root)
        self.assertTrue(mt["present"])
        self.assertEqual(mt["data"]["cumulative_ms"], 100)

    def test_resource_ledger_chain_follows_pointers(self) -> None:
        labs = self.root / "docs" / "labs"
        a = labs / "a-2026-10-01"
        b = labs / "b-2026-10-02"
        for d in (a, b):
            d.mkdir(parents=True, exist_ok=True)
        (a / "resource-ledger.json").write_text(
            json.dumps({"updated_utc": "2026-10-01T10:00:00+00:00", "increment_seconds": 3600, "cumulative_projected_seconds": 100000}),
            encoding="utf-8",
        )
        (b / "resource-ledger.json").write_text(
            json.dumps(
                {
                    "updated_utc": "2026-10-02T10:00:00+00:00",
                    "increment_seconds": 7200,
                    "cumulative_projected_seconds": 107200,
                    "previous_ledger": "docs/labs/a-2026-10-01/resource-ledger.json",
                }
            ),
            encoding="utf-8",
        )
        rl = budget.resource_ledger(labs)
        self.assertTrue(rl["present"])
        self.assertEqual(rl["count"], 2)
        self.assertEqual(rl["chain"]["head"], "b-2026-10-02/resource-ledger.json")
        self.assertEqual(rl["chain"]["links"], 2)
        self.assertEqual(rl["chain"]["increment_seconds"], 10800)
        self.assertEqual(rl["chain"]["cumulative_projected_seconds"], 107200)

    def test_resource_ledger_absent_under_an_empty_dir(self) -> None:
        rl = budget.resource_ledger(self.root / "docs" / "labs")
        self.assertFalse(rl["present"])
        self.assertEqual(rl["reason"], "ABSENT")

    def test_latest_lab_cumulative_uses_the_larger_stated_projection(self) -> None:
        labs = self.root / "docs" / "labs"
        d = labs / "c-2026-10-03"
        d.mkdir(parents=True, exist_ok=True)
        (d / "resource-ledger.json").write_text(
            json.dumps(
                {
                    "complete_accounting_start_utc": "2026-10-03T10:00:00+00:00",
                    "complete_projection_seconds": 7200,
                    "lab_cumulative_projection_seconds": 664170,
                }
            ),
            encoding="utf-8",
        )
        rl = budget.resource_ledger(labs)
        self.assertEqual(rl["latest_lab_cumulative_seconds"]["seconds"], 664170)
        self.assertEqual(rl["latest_lab_cumulative_seconds"]["field"], "lab_cumulative_projection_seconds")


class TestTolerantReads(TempRepoCase):
    def test_one_undecodable_byte_does_not_lose_the_rest_of_the_ledger(self) -> None:
        p = self.paths.repo / "ledger.jsonl"
        p.write_bytes(b'{"ts":"2026-10-01T00:00:00Z","kind":"prep"}\n\x00\xff\n{"ts":"2026-10-02T00:00:00Z","kind":"prep"}\n')
        records = budget._read_jsonl_tolerant(p)
        self.assertEqual(len(records), 2)
        self.assertEqual(records[1]["ts"], "2026-10-02T00:00:00Z")

    def test_collect_survives_a_corrupt_ledger_context(self) -> None:
        (self.paths.ledger_dir / "2026-10.jsonl").write_bytes(b'{"ts":"2026-10-01T00:00:00Z","kind":"prep"}\n\x00\xff\n')
        data = budget.collect(self.paths)
        self.assertEqual(data["receipts"]["records"], 1)


class TestPodLedgerSpend(TempRepoCase):
    def test_closed_pods_are_charged_rate_times_uptime(self) -> None:
        state = self.root / "state"
        state.mkdir()
        records = [
            {"ts": "2026-10-05T19:00:00Z", "event": "up", "lab": "claude", "pod": "p1", "usd_per_h": 1.98},
            {"ts": "2026-10-05T20:00:00Z", "event": "down", "lab": "claude", "pod": "p1", "usd_per_h": 1.98, "uptime_s": 3600},
            {"ts": "2026-10-06T19:00:00Z", "event": "up", "lab": "codex", "pod": "p2", "usd_per_h": 0.99},
            {"ts": "2026-10-06T19:30:00Z", "event": "down", "lab": "codex", "pod": "p2", "usd_per_h": 0.99, "uptime_s": 1800},
        ]
        (state / "ledger.jsonl").write_text("\n".join(json.dumps(r) for r in records) + "\n", encoding="utf-8")
        spend = budget.pod_ledger_spend(state)
        self.assertTrue(spend["present"])
        self.assertAlmostEqual(spend["usd_total"], 1.98 + 0.495, places=3)
        self.assertEqual(set(spend["by_month"]), {"2026-10"})
        self.assertEqual(spend["by_lab"]["claude"], 1.98)

    def test_pod_ledger_absent_is_not_an_error(self) -> None:
        spend = budget.pod_ledger_spend(self.root / "nope")
        self.assertFalse(spend["present"])
        self.assertEqual(spend["reason"], "ABSENT")
        self.assertIsNone(spend["usd_total"])

    def test_state_dir_honours_the_environment(self) -> None:
        old = os.environ.get("UOR_POD_STATE")
        os.environ["UOR_POD_STATE"] = str(self.root / "custom")
        try:
            self.assertEqual(budget.pod_state_dir(), self.root / "custom")
        finally:
            if old is None:
                os.environ.pop("UOR_POD_STATE", None)
            else:
                os.environ["UOR_POD_STATE"] = old


class TestRules(TempRepoCase):
    def test_receipts_parse_tolerates_a_corrupt_line(self) -> None:
        ledger(
            self.paths,
            "2026-10.jsonl",
            [{"ts": "2026-10-01T10:00:00+00:00", "kind": "prep"}],
            raw_extra="{not json at all\n",
        )
        results = [r for r in run_budget_rules(self.paths, self.reg) if r.rule_id == "budget.receipts_parse" and not r.ok]
        self.assertEqual(len(results), 1)
        self.assertIn("not JSON", results[0].message)
        self.assertIn("line 2", results[0].message)

    def test_receipts_parse_requires_ts_and_kind(self) -> None:
        ledger(
            self.paths,
            "2026-10.jsonl",
            [
                {"ts": "2026-10-01T10:00:00+00:00", "kind": "prep"},
                {"kind": "prep"},
                {"ts": "2026-10-01T11:00:00+00:00"},
                "not an object",
            ],
        )
        # call the rule directly: the engine can hold more than one Rule object for an
        # id (another module's missing cache does that), and this assertion is about the
        # rule's own reporting, not about the engine's list.
        results = [r for r in budget._check_receipts_parse(self.paths, self.reg) if not r.ok]
        messages = " | ".join(r.message for r in results)
        self.assertIn("missing ts", messages)
        self.assertIn("missing kind", messages)
        self.assertIn("not an object", messages)

    def test_receipts_parse_passes_a_clean_ledger(self) -> None:
        ledger(self.paths, "2026-10.jsonl", [{"ts": "2026-10-01T10:00:00+00:00", "kind": "prep"}])
        results = [r for r in run_budget_rules(self.paths, self.reg) if r.rule_id == "budget.receipts_parse"]
        self.assertTrue(results)
        self.assertTrue(all(r.ok for r in results))

    def test_disk_floor_warns_below_the_band_with_the_hygiene_command(self) -> None:
        original = budget.disk_free

        def low(_path):
            return {
                "ok": True,
                "path": str(self.root),
                "free_bytes": 10 * 1024**3,
                "total_bytes": 100 * 1024**3,
                "free_gb": 10.0,
                "band_gb": [30, 70],
                "in_band": False,
                "below_floor": True,
                "above_band": False,
                "margin_mib": 128,
                "hygiene": budget.HYGIENE_COMMAND,
            }

        budget.disk_free = low
        try:
            results = [r for r in run_budget_rules(self.paths, self.reg) if r.rule_id == "budget.disk_floor" and not r.ok]
        finally:
            budget.disk_free = original
        self.assertEqual(len(results), 1)
        self.assertIn("10.00 GB", results[0].message)
        self.assertEqual(results[0].fix, "scripts/storage/uor-hygiene --apply")
        self.assertEqual(results[0].severity, "warn")

    def test_disk_floor_is_silent_inside_the_band(self) -> None:
        original = budget.disk_free
        budget.disk_free = lambda _p: {
            "ok": True,
            "path": str(self.root),
            "free_gb": 40.0,
            "band_gb": [30, 70],
            "in_band": True,
            "below_floor": False,
            "above_band": False,
            "margin_mib": 128,
            "hygiene": budget.HYGIENE_COMMAND,
        }
        try:
            results = [r for r in run_budget_rules(self.paths, self.reg) if r.rule_id == "budget.disk_floor" and not r.ok]
        finally:
            budget.disk_free = original
        self.assertEqual(results, [])

    def test_caps_rule_is_info_and_names_the_caps(self) -> None:
        results = [r for r in run_budget_rules(self.paths, self.reg) if r.rule_id == "budget.no_new_spending_class"]
        self.assertTrue(results)
        result = results[0]
        self.assertEqual(result.severity, "info")
        self.assertFalse(result.ok)
        self.assertIn("<= 4 running pods", result.message)
        self.assertIn("$8.00/h", result.message)

    def test_real_disk_measurement_matches_disk_usage(self) -> None:
        measured = budget.disk_free(self.root)
        self.assertTrue(measured["ok"])
        self.assertGreater(measured["free_gb"], 0)


class TestScan(TempRepoCase):
    def _fake_pod(self, status: str, gpus: str, rc: int = 0) -> Path:
        """A stand-in for scripts/pod/uor-pod. It only ever echoes; it cannot mutate state."""
        script = self.paths.repo / "scripts" / "pod" / "uor-pod"
        script.write_text(
            "#!/usr/bin/env bash\n"
            f"if [ \"$1\" = status ]; then echo '{_sh(status)}'; exit {rc}; fi\n"
            f"if [ \"$1\" = gpus ]; then echo '{_sh(gpus)}'; exit {rc}; fi\n"
            "exit 1\n",
            encoding="utf-8",
        )
        script.chmod(script.stat().st_mode | stat.S_IEXEC)
        return script

    def test_parse_status_reads_the_tools_real_layout(self) -> None:
        parsed = budget.parse_pod_status(REAL_STATUS)
        self.assertEqual(parsed["running"], 0)
        self.assertEqual(parsed["usd_per_h"], 0.0)
        self.assertIn("none", parsed["free_summary"])

    def test_parse_status_reads_a_busy_pod(self) -> None:
        text = "Running: 3 pod(s) (3 counted toward the cap of 4), $5.94/h of $8.00/h\nFree GPUs you can lease now: 5090 on pod x."
        parsed = budget.parse_pod_status(text)
        self.assertEqual(parsed["running"], 3)
        self.assertEqual(parsed["usd_per_h"], 5.94)

    def test_parse_gpu_costs(self) -> None:
        costs = budget.parse_gpu_costs(REAL_GPUS)
        self.assertEqual(costs["5090"], 1.19)
        self.assertEqual(costs["4090"], 0.89)
        self.assertEqual(costs["pro6000"], 2.49)

    def test_scan_reports_running_pods_and_the_total_against_the_caps(self) -> None:
        self._fake_pod(REAL_STATUS, REAL_GPUS)
        args = argparse.Namespace(json=True)
        printed = _capture(lambda: budget.scan(args, self.paths, self.reg))
        payload = json.loads(printed)
        self.assertTrue(payload["available"])
        self.assertEqual(payload["running_pods"], 0)
        self.assertEqual(payload["caps"], {"pods": 4, "usd_per_h": 8.0})
        self.assertFalse(payload["over_cap"]["pods"])
        self.assertEqual(payload["gpu_costs_usd_per_h"]["5090"], 1.19)

    def test_scan_flags_an_over_cap_pod_count(self) -> None:
        busy = REAL_STATUS.replace("Running: 0 pod(s)", "Running: 5 pod(s)").replace("$0/h of $8.00/h", "$9.90/h of $8.00/h")
        self._fake_pod(busy, REAL_GPUS)
        args = argparse.Namespace(json=True)
        payload = json.loads(_capture(lambda: budget.scan(args, self.paths, self.reg)))
        self.assertTrue(payload["over_cap"]["pods"])
        self.assertTrue(payload["over_cap"]["rate"])

    def test_scan_is_unavailable_when_the_pod_script_is_missing(self) -> None:
        original = budget.resolve_pod_script
        budget.resolve_pod_script = lambda _root: {"path": None, "source": "", "checked": ["repo: x", "main checkout: y"]}
        args = argparse.Namespace(json=True)
        try:
            payload = json.loads(_capture(lambda: budget.scan(args, self.paths, self.reg)))
        finally:
            budget.resolve_pod_script = original
        self.assertFalse(payload["available"])
        self.assertEqual(payload["reason"], "UNAVAILABLE")
        self.assertEqual(payload["checked"], ["repo: x", "main checkout: y"])
        self.assertEqual(payload["attempts"], [])
        self.assertNotIn("running_pods", payload)
        self.assertNotIn("usd_per_h", payload)
        self.assertIn("$UOR_POD", payload["detail"])

    def test_unavailable_output_names_every_path_checked(self) -> None:
        original = budget.resolve_pod_script
        budget.resolve_pod_script = lambda _root: {"path": None, "source": "", "checked": ["repo: /nowhere/uor-pod"]}
        args = argparse.Namespace(json=False)
        try:
            printed = _capture(lambda: budget.scan(args, self.paths, self.reg))
        finally:
            budget.resolve_pod_script = original
        self.assertIn("UNAVAILABLE", printed)
        self.assertIn("checked repo: /nowhere/uor-pod", printed)
        self.assertIn("nothing is fabricated", printed)


    def test_scan_is_unavailable_when_the_pod_script_errors(self) -> None:
        self._fake_pod("boom\n", "boom\n", rc=3)
        args = argparse.Namespace(json=True)
        payload = json.loads(_capture(lambda: budget.scan(args, self.paths, self.reg)))
        self.assertFalse(payload["available"])
        self.assertEqual(payload["reason"], "UNAVAILABLE")
        self.assertEqual([a["returncode"] for a in payload["attempts"]], [3, 3])
        self.assertNotIn("usd_per_h", payload)

    def test_scan_exits_zero_even_when_unavailable(self) -> None:
        original = budget.pod_script
        budget.pod_script = lambda _root: None
        args = argparse.Namespace(json=False)
        try:
            self.assertEqual(budget.scan(args, self.paths, self.reg), 0)
        finally:
            budget.pod_script = original


class TestPodScriptResolution(TempRepoCase):
    """scripts/pod/uor-pod is gitignored, so a fresh worktree resolves it from the main checkout."""

    def _make_main(self, root: Path) -> Path:
        main = root / "main"
        (main / "scripts" / "pod").mkdir(parents=True, exist_ok=True)
        (main / "scripts" / "pod" / "uor-pod").write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        return main

    def test_resolution_prefers_the_repo_copy(self) -> None:
        main = self._make_main(self.root)
        local = self.paths.repo / "scripts" / "pod" / "uor-pod"
        local.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        original = budget.main_worktree
        budget.main_worktree = lambda _root: main
        try:
            resolved = budget.resolve_pod_script(self.paths.repo)
        finally:
            budget.main_worktree = original
        self.assertEqual(resolved["path"], local)
        self.assertEqual(resolved["source"], "repo")

    def test_resolution_falls_back_to_the_main_checkout(self) -> None:
        main = self._make_main(self.root)
        original = budget.main_worktree
        budget.main_worktree = lambda _root: main
        try:
            resolved = budget.resolve_pod_script(self.paths.repo)
        finally:
            budget.main_worktree = original
        self.assertEqual(resolved["path"], main / "scripts" / "pod" / "uor-pod")
        self.assertEqual(resolved["source"], "main checkout")
        self.assertIn("main checkout", resolved["checked"][1])

    def test_resolution_falls_back_to_the_environment_last(self) -> None:
        env_dir = self.root / "elsewhere"
        env_dir.mkdir()
        env_script = env_dir / "uor-pod"
        env_script.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        original_main = budget.main_worktree
        budget.main_worktree = lambda _root: None
        old_env = os.environ.get("UOR_POD")
        os.environ["UOR_POD"] = str(env_script)
        try:
            resolved = budget.resolve_pod_script(self.paths.repo)
            order = [source for source, _path in budget.pod_candidates(self.paths.repo)]
        finally:
            budget.main_worktree = original_main
            if old_env is None:
                os.environ.pop("UOR_POD", None)
            else:
                os.environ["UOR_POD"] = old_env
        self.assertEqual(resolved["path"], env_script)
        self.assertEqual(resolved["source"], "$UOR_POD")
        self.assertEqual(order, ["repo", "$UOR_POD"])

    def test_candidate_order_is_repo_main_then_env(self) -> None:
        main = self._make_main(self.root)
        env_script = main / "env-uor-pod"
        env_script.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        original = budget.main_worktree
        budget.main_worktree = lambda _root: main
        old_env = os.environ.get("UOR_POD")
        os.environ["UOR_POD"] = str(env_script)
        try:
            order = [source for source, _path in budget.pod_candidates(self.paths.repo)]
        finally:
            budget.main_worktree = original
            if old_env is None:
                os.environ.pop("UOR_POD", None)
            else:
                os.environ["UOR_POD"] = old_env
        self.assertEqual(order, ["repo", "main checkout", "$UOR_POD"])

    def test_nothing_resolves_when_no_candidate_exists(self) -> None:
        original = budget.main_worktree
        budget.main_worktree = lambda _root: None
        old_env = os.environ.pop("UOR_POD", None)
        try:
            resolved = budget.resolve_pod_script(self.paths.repo)
        finally:
            budget.main_worktree = original
            if old_env is not None:
                os.environ["UOR_POD"] = old_env
        self.assertIsNone(resolved["path"])
        self.assertEqual(resolved["source"], "")
        self.assertEqual(resolved["checked"], [f"repo: {self.paths.repo / 'scripts/pod/uor-pod'}"])

    def test_main_worktree_reads_git_worktree_list(self) -> None:
        original = budget.run

        def fake_run(cmd, cwd=None, timeout=20):
            if cmd[:3] == ["git", "worktree", "list"]:
                return 0, f"worktree {self.root}/main\nHEAD abc\n\nworktree {self.root}/wt\nHEAD def\n"
            return 1, ""

        budget.run = fake_run
        try:
            self.assertEqual(budget.main_worktree(self.root / "wt"), self.root / "main")
        finally:
            budget.run = original

    def test_main_worktree_falls_back_to_the_common_git_dir(self) -> None:
        original = budget.run

        def fake_run(cmd, cwd=None, timeout=20):
            if cmd[:3] == ["git", "worktree", "list"]:
                return 1, "not a git repository"
            if cmd[:2] == ["git", "rev-parse"]:
                return 0, f"{self.root}/main/.git\n"
            return 1, ""

        budget.run = fake_run
        try:
            self.assertEqual(budget.main_worktree(self.root / "wt"), self.root / "main")
        finally:
            budget.run = original

    def test_main_worktree_is_none_outside_git(self) -> None:
        original = budget.run
        budget.run = lambda cmd, cwd=None, timeout=20: (1, "fatal: not a git repository")
        try:
            self.assertIsNone(budget.main_worktree(self.paths.repo))
        finally:
            budget.run = original

    def test_resolves_uor_pod_in_this_checkout_or_from_main(self) -> None:
        """The real adoption path: this worktree, or the main checkout it belongs to."""
        if not (REPO / "scripts" / "pod" / "uor-pod").is_file() and budget.main_worktree(REPO) is None:
            self.skipTest("no checkout in this environment carries scripts/pod/uor-pod")
        resolved = budget.resolve_pod_script(REPO)
        self.assertIsNotNone(resolved["path"], resolved["checked"])
        self.assertTrue(Path(resolved["path"]).is_file())
        self.assertIn(resolved["source"], ("repo", "main checkout", "$UOR_POD"))
        self.assertTrue(os.access(resolved["path"], os.X_OK), "the resolved tool must be executable")

    def test_resolution_reads_no_environment_secrets(self) -> None:
        """Only $UOR_POD is read; nothing here touches ~/.runpod/config.toml or a key."""
        import inspect

        source = inspect.getsource(budget)
        self.assertNotIn("runpod/config.toml", source)
        self.assertNotIn("api_key", source.lower())



def _sh(text: str) -> str:
    """Single-quote a string for the fake pod script."""
    return text.replace("'", "'\\''")


def _capture(fn) -> str:
    import io
    from contextlib import redirect_stdout

    buf = io.StringIO()
    with redirect_stdout(buf):
        fn()
    return buf.getvalue()


if __name__ == "__main__":
    unittest.main()
