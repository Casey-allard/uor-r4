"""Synthetic retained-campaign admission and accounting checks for #1094.

No real assembly, model, worker, runtime identity or corpus is opened here.
Runtime and subprocess boundaries use in-memory stubs; fixtures live only in
TemporaryDirectory. These checks are not an execution-release receipt.
"""

from __future__ import annotations

import io
import json
import os
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import call, patch

from r4_softmax_trainer.text_clause_adapter import campaign, contract


HISTORICAL_BYTES = 3_465_401


class SyntheticCase(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(self.stack.enter_context(tempfile.TemporaryDirectory())).resolve()
        self.output = self.root / "synthetic-output"
        self.output.mkdir()
        self.args = SimpleNamespace(
            phase="run-retained", repo=Path(campaign.__file__).resolve().parents[5],
            corpus=self.root / "unopened-synthetic-corpus", output=self.output,
            python=self.root / "synthetic-python",
            review=self.output / "release.json",
            assembly=self.output / "retained-preparation.json",
        )
        self.clock = self.stack.enter_context(patch.object(campaign.time, "monotonic", return_value=100.0))
        self.stack.enter_context(patch.object(
            campaign.resource, "getrusage", return_value=SimpleNamespace(ru_maxrss=1),
        ))

    def budget(self, phase="execution", carried=0, carried_forwards=0):
        value = campaign.Budget(self.args, phase, carried, carried_forwards)
        value.output_validated = True
        return value

    def write(self, name, value):
        path = self.output / name
        path.write_bytes(contract.canonical(value))
        return path


class RetainedBudgetTests(SyntheticCase):
    def test_historical_debit_and_bytes_survive_execution_to_replay(self):
        # The corpus does not exist: retained snapshots must use the audited
        # ledger rather than traversing or reading a newly selected corpus.
        self.assertFalse(self.args.corpus.exists())
        payload = b"one new retained receipt\n"
        (self.output / "synthetic-receipt.txt").write_bytes(payload)
        execution = self.budget()
        execution.row_forwards = 3200
        self.clock.return_value = 110.0
        first = execution.snapshot()
        self.assertEqual(first["historical_preparation_policy_debit_seconds"], 120)
        self.assertEqual(first["phase_elapsed_seconds"], 10)
        self.assertEqual(first["cumulative_elapsed_seconds"], 130)
        self.assertEqual(first["new_bytes"], HISTORICAL_BYTES + len(payload))

        replay = self.budget("replay", execution.carried + execution.elapsed,
                             execution.row_forwards)
        replay.row_forwards = 3200
        self.clock.return_value = 115.0
        second = replay.snapshot()
        self.assertEqual(second["phase_elapsed_seconds"], 5)
        self.assertEqual(second["cumulative_elapsed_seconds"], 135)
        self.assertEqual(second["cumulative_logical_row_forwards"], 6400)
        self.assertEqual(second["new_bytes"], first["new_bytes"])
        replay.check()

    def test_independent_caps_refuse_at_the_resource_boundary(self):
        for kind in ("phase", "cumulative", "bytes", "forwards", "rss"):
            with self.subTest(kind=kind):
                self.clock.return_value = 100.0
                budget = self.budget("replay", 350 if kind == "cumulative" else 120)
                limits = dict(contract.LIMITS)
                if kind == "phase":
                    self.clock.return_value = 221.0
                elif kind == "cumulative":
                    self.clock.return_value = 111.0
                elif kind == "bytes":
                    limits["new_bytes"] = HISTORICAL_BYTES - 1
                elif kind == "forwards":
                    budget.carried_forwards, budget.row_forwards = 6400, 1
                else:
                    budget.worker_peak = limits["peak_rss_bytes"]
                with patch.dict(contract.LIMITS, limits, clear=True):
                    with self.assertRaises(campaign.CampaignFailure) as failure:
                        budget.check()
                self.assertEqual(failure.exception.status, "INCOMPLETE_RESOURCE")

    def test_unadmitted_output_is_not_traversed_and_aliases_are_refused(self):
        missing = self.args.output = self.root / "not-admitted-and-absent"
        budget = campaign.Budget(self.args, "execution")
        self.assertFalse(budget.output_validated)
        self.assertEqual(budget.snapshot()["new_bytes"], HISTORICAL_BYTES)
        self.assertFalse(missing.exists())

        target = self.root / "synthetic-target"
        target.mkdir()
        (target / "synthetic-secret").write_bytes(b"never inventory through alias")
        link = self.output / "redirect"
        link.symlink_to(target, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            campaign.retained_output_bytes(self.output)
        link.unlink()
        original = self.output / "original"
        original.write_bytes(b"one allocation")
        os.link(original, self.output / "hard-link")
        with self.assertRaisesRegex(ValueError, "hard-linked"):
            campaign.retained_output_bytes(self.output)


class SyntheticProcess:
    """Popen substitute containing protocol text only; it starts no process."""

    def __init__(self, events):
        self.stdin = io.StringIO()
        self.stdout = io.StringIO("".join(json.dumps(event) + "\n" for event in events))
        self.stderr = io.StringIO()
        self.pid = -1

    def poll(self):
        return 0

    def wait(self, timeout=None):
        return 0


class RetainedAdmissionTests(SyntheticCase):
    def setUp(self):
        super().setUp()
        from r4_softmax_trainer.text_clause_adapter import retained
        self.retained = retained
        bindings_path = self.write("bindings.json", {"hardware": {"synthetic": True}})
        profile_path = self.output / "worker.sb"
        profile_path.write_bytes(b"synthetic profile text; never launched\n")
        self.preparation = {
            "schema": "uor-r4.text-clause-retained-preparation/1",
            "status": "PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE",
            "output_paths": {"corpus": str(self.args.corpus), "output": str(self.output)},
            "runtime_identity": {"interpreter": {"launcher": str(self.args.python)}},
            "bindings": contract.record(bindings_path),
            "sandbox": contract.record(profile_path),
            "coordinator_source": {"synthetic": "coordinator-only"},
            "worker_source": {"synthetic": "unchanged-worker-only"},
            "clean_environment": {"synthetic": "exact environment"},
            "profile_delta": {"synthetic": "exact profile delta"},
            "corpus_manifest": {"sha256": "synthetic-population"},
            "selection": {"sha256": "synthetic-selection"},
            "corpus_commitments": {"files": [
                {"path": "withheld/synthetic.jsonl", "sha256": "synthetic-unopened"},
            ]},
        }
        self.preparation["assembly_record"] = contract.record(
            self.write("retained-preparation.json", self.preparation))
        self.args.review.write_bytes(contract.canonical({"schema": "synthetic-review-only"}))
        self.load = self.stack.enter_context(patch.object(
            retained, "load_for_release", return_value=self.preparation,
        ))
        self.validate = self.stack.enter_context(patch.object(
            retained, "validate_assembly", return_value=self.preparation,
        ))
        self.runtime = self.stack.enter_context(patch.object(
            retained, "verify_runtime", return_value={"synthetic": True},
        ))
        self.reader = self.stack.enter_context(patch.object(
            campaign, "read_jsonl", side_effect=AssertionError("payload reader reached"),
        ))
        self.launcher = self.stack.enter_context(patch.object(
            campaign.subprocess, "Popen", side_effect=AssertionError("worker launcher reached"),
        ))
        self.verify_synthetic_record = contract.verify_record
        self.payload_calls = []

        def verify_metadata_or_tripwire(item):
            if Path(item["path"]).is_relative_to(self.args.corpus):
                self.payload_calls.append(item)
                raise AssertionError("payload hash reached")
            # Only records under this test's TemporaryDirectory are permitted.
            self.assertTrue(Path(item["path"]).is_relative_to(self.root))
            self.verify_synthetic_record(item)

        self.hash_payload = self.stack.enter_context(patch.object(
            contract, "verify_record", side_effect=verify_metadata_or_tripwire,
        ))

    def assert_unopened(self):
        self.reader.assert_not_called()
        self.launcher.assert_not_called()
        self.assertEqual(self.payload_calls, [])
        self.assertFalse((self.output / "execution-started.json").exists())

    def test_loaded_coordinator_cannot_bind_a_different_repository(self):
        self.args.repo = self.root / "different-synthetic-repository"
        with self.assertRaisesRegex(ValueError, "executing coordinator source is outside bound repo"):
            campaign.run_retained(self.args)
        self.load.assert_not_called()
        self.validate.assert_not_called()
        self.runtime.assert_not_called()
        self.assert_unopened()
        self.assertFalse((self.output / "admission-started.json").exists())

    def test_missing_legacy_and_changed_release_refuse_before_payloads(self):
        self.args.review = self.root / "arbitrary-review.json"
        with self.assertRaisesRegex(ValueError, "exact output/release.json"):
            campaign.run_retained(self.args)
        self.load.assert_not_called()
        self.validate.assert_not_called()
        self.runtime.assert_not_called()
        self.assert_unopened()
        self.args.review = self.output / "release.json"
        reviews = (
            None,
            {"status": "ACCEPTED_FOR_FROZEN_COMPARISON"},
            {**self.retained.release_bindings(self.preparation),
             "schema": "uor-r4.text-clause-retained-release/1", "issue": 1094,
             "reviewer": "synthetic fixture; no real approval",
             "status": "ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON",
             "assembly_sha256": "altered-identity"},
        )
        for review in reviews:
            with self.subTest(review=review):
                if review is None:
                    self.args.review.unlink(missing_ok=True)
                else:
                    self.args.review.write_bytes(contract.canonical(review))
                # Use the actual retained-release validator here. The assembly
                # validator alone is mocked; it cannot confer release.
                with self.assertRaises((ValueError, KeyError, FileNotFoundError)):
                    campaign.run_retained(self.args)
                self.assert_unopened()
                self.assertFalse((self.output / "admission-started.json").exists())
                self.validate.assert_not_called()
        self.args.review = None
        with self.assertRaisesRegex(ValueError, "requires assembly and independent release"):
            campaign.run_retained(self.args)
        self.assert_unopened()

    def test_source_failure_leaves_durable_attempt_and_refuses_reuse(self):
        with patch.object(self.retained, "verify_release", return_value=None):
            self.validate.side_effect = ValueError("synthetic committed-source drift")
            with self.assertRaisesRegex(ValueError, "committed-source drift"):
                campaign.run_retained(self.args)
            self.assert_unopened()
            self.runtime.assert_not_called()
            admission = json.loads((self.output / "admission-started.json").read_bytes())
            self.assertEqual(admission["historical_preparation_policy_debit_seconds"], 120)
            self.assertEqual(admission["historical_retained_bytes"], HISTORICAL_BYTES)
            self.assertEqual(admission["withheld_payload_reads"], 0)
            self.validate.side_effect = None
            self.validate.reset_mock()
            with self.assertRaisesRegex(ValueError, "already consumed"):
                campaign.run_retained(self.args)
            self.validate.assert_not_called()
            self.assert_unopened()

    def test_runtime_failure_leaves_durable_attempt_before_payloads(self):
        with patch.object(self.retained, "verify_release", return_value=None):
            self.runtime.side_effect = ValueError("synthetic runtime drift")
            with self.assertRaisesRegex(ValueError, "runtime drift"):
                campaign.run_retained(self.args)
            self.assert_unopened()
            self.assertTrue((self.output / "admission-started.json").exists())

    def test_prior_start_stop_progress_and_completion_refuse_reuse(self):
        markers = ("admission-started.json", "execution-started.json", "replay-started.json",
                   "run-retained-stopped.json", "run-stopped.json",
                   "execution-progress.jsonl", "execution.json",
                   "replay.json", "result.json", "completion.json")
        with patch.object(self.retained, "verify_release") as release:
            for marker in markers:
                with self.subTest(marker=marker):
                    path = self.write(marker, {"synthetic-prior-evidence": True})
                    before = path.read_bytes()
                    with self.assertRaisesRegex(ValueError, "already consumed"):
                        campaign.run_retained(self.args)
                    self.assertEqual(path.read_bytes(), before)
                    self.reader.assert_not_called()
                    self.launcher.assert_not_called()
                    self.assertEqual(self.payload_calls, [])
                    release.assert_not_called()
                    path.unlink()

    def test_start_receipt_precedes_first_payload_hash_and_checks_use_clock(self):
        class PayloadBoundary(Exception):
            pass

        def validate(*args, **kwargs):
            admission = json.loads((self.output / "admission-started.json").read_bytes())
            self.assertEqual(admission["fresh_identity_checks"], "NOT_RUN")
            self.assertEqual(admission["historical_preparation_policy_debit_seconds"], 120)
            self.assertFalse((self.output / "execution-started.json").exists())
            self.clock.return_value = 102.0
            return self.preparation

        def release(*args, **kwargs):
            self.assertFalse((self.output / "admission-started.json").exists())
            self.clock.return_value = 101.0

        def runtime(*args, **kwargs):
            self.clock.return_value = 103.0
            return {"synthetic": True}

        def at_payload_boundary(item):
            if not Path(item["path"]).is_relative_to(self.args.corpus):
                self.assertTrue(Path(item["path"]).is_relative_to(self.root))
                self.verify_synthetic_record(item)
                return
            self.payload_calls.append(item)
            start = json.loads((self.output / "execution-started.json").read_bytes())
            self.assertEqual(start["carried_seconds"], 120)
            self.assertEqual(start["historical_retained_bytes"], HISTORICAL_BYTES)
            self.assertEqual(start["withheld_payload_reads_before_receipt"], 0)
            self.assertEqual(item["path"], str(self.args.corpus / "withheld/synthetic.jsonl"))
            raise PayloadBoundary()

        self.validate.side_effect = validate
        self.runtime.side_effect = runtime
        self.hash_payload.side_effect = at_payload_boundary
        with patch.object(self.retained, "verify_release", side_effect=release):
            with self.assertRaises(PayloadBoundary):
                campaign.run_retained(self.args)
        snapshot = self.args._active_budget.snapshot()
        self.assertEqual(snapshot["phase_elapsed_seconds"], 3)
        self.assertEqual(snapshot["cumulative_elapsed_seconds"], 123)
        self.assertGreater(snapshot["new_bytes"], HISTORICAL_BYTES)
        self.reader.assert_not_called()
        self.launcher.assert_not_called()
        self.assertEqual(len(self.payload_calls), 1)


class RetainedLauncherTests(SyntheticCase):
    def setUp(self):
        super().setUp()
        self.bindings = {"reader_state_cid": "synthetic-reader-state",
                         "core_state_cid": "synthetic-core-state"}
        path = self.write("bindings.json", self.bindings)
        self.binding_sha = contract.record(path)["sha256"]
        self.environment = {
            "PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": "/var/empty",
            "PYTHONPATH": str(self.args.repo / "tools/r4-softmax-trainer/src"),
            "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1",
            "PYTHONUNBUFFERED": "1", "OMP_NUM_THREADS": "4",
            "VECLIB_MAXIMUM_THREADS": "4",
            "UOR_ISOLATION_PROBE": str(self.args.corpus / "isolation-probe.txt"),
        }
        self.args._retained_assembly = {"clean_environment": self.environment}
        self.identity = self.stack.enter_context(patch.object(
            campaign, "retained_runtime_check", return_value=None,
        ))

    def events(self):
        states = {"reader": self.bindings["reader_state_cid"],
                  "core": self.bindings["core_state_cid"]}
        common = {"bindings_sha256": self.binding_sha, "runtime": contract.RUNTIME,
                  "deterministic_algorithms": True, "model_loads": 2,
                  "row_forwards": 0, "batch_forwards": 0}
        return [
            {**common, "event": "ready", "isolation_denied": True, "states": states},
            {**common, "event": "done", "states_before": states, "states_after": states,
             "audit": {"isolation_denied": True, "optimizer_updates": 0, "rows": 0}},
        ]

    def assert_before_after(self, budget):
        self.assertEqual(self.identity.call_args_list, [
            call(self.args, budget, "oracle", "before-worker"),
            call(self.args, budget, "oracle", "after-worker"),
        ])

    def test_clean_environment_and_parent_identity_checks_surround_stub(self):
        process = SyntheticProcess(self.events())
        budget = self.budget()
        with patch.dict(os.environ, {"PYTHONHOME": "/injected", "DYLD_INSERT_LIBRARIES": "/injected",
                                     "PYTHONPATH": "/injected", "OMP_NUM_THREADS": "999"}):
            with patch.object(campaign.subprocess, "Popen", return_value=process) as launch:
                result = campaign.arm_process(self.args, "oracle", [], budget)
        self.assertEqual(result["event"], "done")
        self.assertEqual(launch.call_args.kwargs["env"], self.environment)
        self.assertEqual(launch.call_args.kwargs["cwd"], "/")
        self.assertNotIn("--readiness-only", launch.call_args.args[0])
        self.assert_before_after(budget)

    def test_launch_and_worker_failure_still_check_identity_and_keep_first_cause(self):
        for kind in ("launch", "worker", "worker-plus-postcheck"):
            with self.subTest(kind=kind):
                self.identity.reset_mock()
                self.identity.side_effect = ([None, ValueError("synthetic post-worker drift")]
                                             if kind == "worker-plus-postcheck" else None)
                budget = self.budget()
                failed = SyntheticProcess([{"event": "error", "status": "SYNTHETIC_WORKER_FAILURE",
                    "reason": "synthetic first worker failure", "row_forwards": 7}])
                with patch.object(campaign.subprocess, "Popen", return_value=failed,
                                  side_effect=OSError("synthetic launch failed") if kind == "launch" else None):
                    if kind == "launch":
                        with self.assertRaisesRegex(OSError, "synthetic launch failed"):
                            campaign.arm_process(self.args, "oracle", [], budget)
                    else:
                        with self.assertRaisesRegex(campaign.CampaignFailure, "first worker failure") as failure:
                            campaign.arm_process(self.args, "oracle", [], budget)
                        self.assertEqual(failure.exception.status, "SYNTHETIC_WORKER_FAILURE")
                        self.assertEqual(budget.row_forwards, 7)
                self.assert_before_after(budget)
                if kind == "worker-plus-postcheck":
                    self.assertTrue(any(event.get("event") == "post-worker-check-failed"
                                        and event["original_failure"] == "synthetic first worker failure"
                                        for event in budget.progress))


if __name__ == "__main__":
    unittest.main()
