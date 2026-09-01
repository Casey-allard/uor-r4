from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

import torch

from r4_softmax_trainer import learned_associative_readout_campaign as campaign


def _metric(*, nll: float, top1: int, attention_off: bool) -> dict[str, object]:
    return {
        "rows": campaign.FRESH_HELDOUT_DECISIONS,
        "ce_nats": nll,
        "top1_correct": top1,
        "top1_rate": top1 / campaign.FRESH_HELDOUT_DECISIONS,
        "forbidden_reads": 0,
        "attention_off": attention_off,
    }


def _plan_record(
    plan: campaign.ExecutionPlan,
    *,
    step_seconds: float,
    evaluation_seconds: float = 0.02,
    peak_bytes: int = 1_000,
    budget_bytes: int = 10_000,
    vector_delta: float = 0.0,
    available: bool = True,
) -> dict[str, object]:
    arms: dict[str, object] = {}
    for index, arm in enumerate(campaign.ARMS):
        if not available:
            arms[arm] = {"ok": False, "error": {"reason": "unavailable"}}
            continue
        vector = [float(index), 1.0 + vector_delta]
        arms[arm] = {
            "ok": True,
            "result": {
                "mean_train_step_seconds": step_seconds,
                "evaluation_batch_seconds": evaluation_seconds,
                "artifact_export_seconds": 0.01,
                "peak_memory_bytes": peak_bytes,
                "memory_budget_bytes": budget_bytes,
                "probe_vector": vector,
                "mechanics": {"passed": True},
                "sealed_prompt_reads": 0,
                "sealed_heldout_reads": 0,
            },
        }
    return {"plan": plan.identity(), "arms": arms}


class _FakeModel:
    def __init__(self) -> None:
        self.parameters_by_arm = {
            arm: torch.nn.Parameter(torch.zeros(campaign.QUERY_SHAPE))
            for arm in campaign.ARMS
        }

    def head_parameters(self, arm: str):
        return (self.parameters_by_arm[arm],)

    def export_head_artifact(self, arm: str) -> bytes:
        return self.parameters_by_arm[arm].detach().cpu().numpy().tobytes()


def _post_reveal_fixture(
    root: Path,
    *,
    geometric_elapsed: float = 100.0,
    pooled_elapsed: float = 90.0,
):
    artifacts: dict[str, dict[str, object]] = {}
    for arm in campaign.ARMS:
        path = campaign._artifact_path(root, arm)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(f"{arm}-head".encode())
        artifacts[arm] = {
            "path": str(path.relative_to(root)),
            "bytes": path.stat().st_size,
            "cid": campaign.cid_file(path),
        }
    reveal = campaign._with_cid(
        {
            "baseline_artifact_cid": campaign.PREDECESSOR_ARTIFACT_CID,
            "geometric_artifact_cid": artifacts["geometric"]["cid"],
            "pooled_artifact_cid": artifacts["pooled"]["cid"],
            "fresh_heldout_cid": campaign.FRESH_HELDOUT_CID,
        },
        "reveal_cid",
    )
    reveal_path = root / campaign.REVEAL_RELATIVE_PATH
    reveal_path.parent.mkdir(parents=True, exist_ok=True)
    reveal_path.write_bytes(campaign.canonical_json_bytes(reveal))
    predecessor = root / "qualified-v1.safetensors"
    predecessor.write_bytes(b"v1")
    preparation = Mock()
    preparation.root = root
    preparation.predecessor_artifact_path = predecessor
    preparation.manifest = {
        "preparation_cid": "preparation",
        "implementation": {"tree_cid": "implementation"},
    }
    probe = {"probe_cid": "probe"}
    started = {"started_cid": "started", "run_contract_cid": "contract"}
    arm_results = {
        "geometric": {
            "elapsed_seconds": geometric_elapsed,
            "artifact": artifacts["geometric"],
        },
        "pooled": {
            "elapsed_seconds": pooled_elapsed,
            "artifact": artifacts["pooled"],
        },
    }
    return preparation, probe, started, arm_results, reveal


class LearnedAssociativeReadoutCampaignTests(unittest.TestCase):
    def test_frozen_dose_population_and_execution_plans(self) -> None:
        self.assertEqual(campaign.OPTIMIZER_STEPS, 2_730)
        self.assertEqual(campaign.TRAIN_DECISIONS, 5_241_600)
        self.assertEqual(campaign.FRESH_HELDOUT_DECISIONS, 247_920)
        self.assertEqual(campaign.FRESH_HELDOUT_REACHABLE_DECISIONS, 245_854)
        self.assertEqual(campaign.FRESH_HELDOUT_TOKENS * 2, 499_972)
        self.assertEqual(
            [plan.name for plan in campaign.ELIGIBLE_PLANS],
            [
                "cpu-accelerate-4t-sequential",
                "cpu-accelerate-8t-sequential",
                "cpu-accelerate-2x4t-concurrent",
                "mps-deterministic-sequential",
            ],
        )
        self.assertEqual(campaign.ELIGIBLE_PLANS[2].threads_per_worker, 4)
        self.assertEqual(campaign.ELIGIBLE_PLANS[2].workers, 2)

    def test_plan_selection_uses_common_cpu4_scoring_and_fastest_eligible(self) -> None:
        records = [
            _plan_record(campaign.ELIGIBLE_PLANS[0], step_seconds=0.20),
            _plan_record(campaign.ELIGIBLE_PLANS[1], step_seconds=0.15),
            _plan_record(campaign.ELIGIBLE_PLANS[2], step_seconds=0.12),
            _plan_record(campaign.ELIGIBLE_PLANS[3], step_seconds=0.10, available=False),
        ]
        selection = campaign.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertEqual(
            selection["selected_plan"]["name"],
            "cpu-accelerate-2x4t-concurrent",
        )
        common = {
            record["projection"]["common_canonical_cpu4_scoring_seconds"]
            for record in selection["plans"]
        }
        self.assertEqual(len(common), 1)
        concurrent = next(
            record
            for record in selection["plans"]
            if record["plan"]["name"] == "cpu-accelerate-2x4t-concurrent"
        )
        self.assertAlmostEqual(
            concurrent["projection"]["plan_specific_training_seconds"],
            0.12 * campaign.OPTIMIZER_STEPS + 0.01,
        )

    def test_fresh_gates_apply_nonregression_and_state_load_bearing(self) -> None:
        predecessor = _metric(nll=2.0, top1=20_000, attention_off=False)
        candidate = _metric(nll=2.05, top1=17_521, attention_off=False)
        state_off = _metric(nll=2.15, top1=15_041, attention_off=True)
        gates = campaign.fresh_generalization_gates(
            candidate=candidate,
            predecessor=predecessor,
            state_off=state_off,
        )
        self.assertTrue(gates["passed"])
        self.assertAlmostEqual(gates["predecessor_nll_delta"], 0.05)
        self.assertEqual(gates["state_off_top1_decision_loss"], 2_480)

    def test_terminal_branches_preserve_independent_science_decisions(self) -> None:
        common = {
            "geometric_capacity_verdict": campaign.VERDICT_PASS,
            "pooled_capacity_verdict": campaign.VERDICT_PASS,
            "geometry_verdict": campaign.GEOMETRY_ATTRIBUTION_PASS,
            "geometric_fresh_nll": 2.1,
            "pooled_fresh_nll": 2.0,
            "mechanics_passed": True,
        }
        geometric = campaign.terminal_decision(
            **common,
            geometric_fresh_passed=True,
            pooled_fresh_passed=True,
        )
        self.assertEqual(geometric["verdict"], campaign.TERMINAL_GEOMETRIC_PASS)
        self.assertEqual(geometric["selected_arm"], "geometric")

        pooled_fallback = campaign.terminal_decision(
            **common,
            geometric_fresh_passed=False,
            pooled_fresh_passed=True,
        )
        self.assertEqual(pooled_fallback["verdict"], campaign.TERMINAL_CONTROL_ONLY)
        self.assertEqual(pooled_fallback["selected_arm"], "pooled")

        regression = campaign.terminal_decision(
            **common,
            geometric_fresh_passed=False,
            pooled_fresh_passed=False,
        )
        self.assertEqual(regression["verdict"], campaign.TERMINAL_FRESH_REGRESSION)

        failed = campaign.terminal_decision(
            **{
                **common,
                "geometric_capacity_verdict": "PROMPT_CONDITIONING_CAPACITY_FAIL",
                "pooled_capacity_verdict": "PROMPT_CONDITIONING_CAPACITY_FAIL",
                "geometry_verdict": "GEOMETRY_ATTRIBUTION_FAIL",
            },
            geometric_fresh_passed=False,
            pooled_fresh_passed=False,
        )
        self.assertEqual(failed["verdict"], campaign.TERMINAL_NO_CAPACITY)

    def test_missing_probe_cannot_be_created_after_reveal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reveal = root / campaign.REVEAL_RELATIVE_PATH
            reveal.parent.mkdir(parents=True)
            reveal.write_text("marker", encoding="utf-8")
            executor = Mock(side_effect=AssertionError("executor must not run"))
            with self.assertRaisesRegex(RuntimeError, "missing execution probe"):
                campaign._probe_learned_associative_readout(root, executor=executor)
            executor.assert_not_called()

    def test_post_reveal_resume_never_launches_training(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reveal = root / campaign.REVEAL_RELATIVE_PATH
            reveal.parent.mkdir(parents=True)
            reveal.write_text("marker", encoding="utf-8")
            preparation = Mock()
            preparation.root = root
            preparation.manifest = {"preparation_cid": "p"}
            probe = {
                "eligible": True,
                "probe_cid": "q",
                "selection": {"selected_plan": campaign.ELIGIBLE_PLANS[0].identity()},
            }
            started = {"run_contract_cid": "r", "started_cid": "s"}
            runner = Mock(side_effect=AssertionError("training must not launch"))
            with (
                patch.object(campaign, "load_learned_associative_readout_preparation", return_value=preparation),
                patch.object(campaign, "probe_learned_associative_readout", return_value=probe),
                patch.object(campaign, "_load_started", return_value=started),
                patch.object(campaign, "_load_arm_result", return_value={"status": "COMPLETE"}),
                patch.object(campaign, "_finalize_result", return_value={"verdict": "done"}),
                patch.object(campaign, "_spawned_arm_runner", runner),
            ):
                with self.assertRaisesRegex(RuntimeError, "requires --resume"):
                    campaign.run_learned_associative_readout(root, resume=False)
                self.assertEqual(
                    campaign.run_learned_associative_readout(root, resume=True),
                    {"verdict": "done"},
                )
            runner.assert_not_called()

    def test_missing_started_after_reveal_raises_without_creating_one(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reveal = root / campaign.REVEAL_RELATIVE_PATH
            reveal.parent.mkdir(parents=True)
            reveal.write_text("marker", encoding="utf-8")
            preparation = Mock()
            preparation.root = root
            preparation.manifest = {"preparation_cid": "p"}
            probe = {
                "eligible": True,
                "probe_cid": "q",
                "selection": {
                    "selected_plan": campaign.ELIGIBLE_PLANS[0].identity()
                },
            }
            with (
                patch.object(
                    campaign,
                    "load_learned_associative_readout_preparation",
                    return_value=preparation,
                ),
                patch.object(
                    campaign,
                    "probe_learned_associative_readout",
                    return_value=probe,
                ),
            ):
                with self.assertRaisesRegex(FileNotFoundError, "started envelope"):
                    campaign.run_learned_associative_readout(root, resume=True)
            self.assertFalse((root / campaign.STARTED_RELATIVE_PATH).exists())

    def test_verifier_rejects_result_writer_process(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(
                campaign,
                "_load_result",
                return_value={"writer_process_id": campaign.os.getpid()},
            ):
                with self.assertRaisesRegex(ValueError, "fresh process"):
                    campaign.verify_learned_associative_readout_result(
                        Path(directory)
                    )

    def test_cached_result_cannot_bypass_immutable_artifact_validation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            body = {
                "schema": campaign.RESULT_SCHEMA,
                "issue": campaign.ISSUE,
                "policy": campaign.POLICY,
                "decision": {"verdict": "x"},
                "verdict": "x",
            }
            result = campaign._with_cid(body, "result_cid")
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result_path.write_bytes(campaign.canonical_json_bytes(result))
            with patch.object(
                campaign,
                "_validate_result_bindings",
                side_effect=FileNotFoundError("deleted geometric head"),
            ):
                with self.assertRaisesRegex(
                    FileNotFoundError, "deleted geometric head"
                ):
                    campaign._load_result(root)

    def test_arm_result_rejects_a_deleted_head_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            arm = "geometric"
            artifact_path = campaign._artifact_path(root, arm)
            artifact_path.parent.mkdir(parents=True)
            artifact_path.write_bytes(b"head")
            body = {
                "schema": campaign.ARM_RESULT_SCHEMA,
                "issue": campaign.ISSUE,
                "policy": campaign.POLICY,
                "arm": arm,
                "status": "COMPLETE",
                "run_contract_cid": "run",
                "plan_cid": "plan",
                "completed_steps": campaign.OPTIMIZER_STEPS,
                "presentations": campaign.TRAIN_DECISIONS,
                "artifact": {
                    "path": str(artifact_path.relative_to(root)),
                    "bytes": artifact_path.stat().st_size,
                    "cid": campaign.cid_file(artifact_path),
                },
            }
            result = campaign._with_cid(body, "arm_result_cid")
            result_path = campaign._arm_result_path(root, arm)
            result_path.write_bytes(campaign.canonical_json_bytes(result))
            campaign._load_arm_result(
                root, arm, run_contract_cid="run", plan_cid="plan"
            )
            artifact_path.unlink()
            with self.assertRaises(FileNotFoundError):
                campaign._load_arm_result(
                    root, arm, run_contract_cid="run", plan_cid="plan"
                )

    def test_resume_wall_charges_progress_beyond_last_checkpoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            progress = {
                "schema": "uor-r4.learned-associative-readout-progress/1",
                "arm": "geometric",
                "completed_steps": 150,
                "elapsed_seconds": 25.0,
            }
            path = campaign._progress_path(root, "geometric")
            path.parent.mkdir(parents=True)
            path.write_bytes(campaign.canonical_json_bytes(progress))
            self.assertEqual(
                campaign._resume_elapsed(
                    root,
                    arms=("geometric",),
                    checkpoint_step=100,
                    checkpoint_elapsed=20.0,
                ),
                25.0,
            )

    def test_resume_wall_charges_same_step_shared_finalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            progress = {
                "schema": "uor-r4.learned-associative-readout-progress/1",
                "arm": "geometric",
                "completed_steps": campaign.OPTIMIZER_STEPS,
                "elapsed_seconds": 25.0,
            }
            path = campaign._progress_path(root, "geometric")
            path.parent.mkdir(parents=True)
            path.write_bytes(campaign.canonical_json_bytes(progress))
            self.assertEqual(
                campaign._resume_elapsed(
                    root,
                    arms=("geometric",),
                    checkpoint_step=campaign.OPTIMIZER_STEPS,
                    checkpoint_elapsed=20.0,
                ),
                25.0,
            )

    def test_shared_arm_elapsed_includes_artifact_replay_finalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = Mock()
            output.logits = torch.zeros((1, 1, 1))
            output.audit.forbidden_reads = 0
            model = Mock()
            model.export_head_artifact.return_value = b"head"
            model.forward_arm.return_value = output
            replay = Mock()
            replay.forward_arm.return_value = output
            preparation = Mock()
            preparation.root = root
            preparation.predecessor = Mock()
            with (
                patch.object(campaign, "_new_model", return_value=replay),
                patch.object(
                    campaign,
                    "_ordered_train_batch",
                    return_value=torch.zeros((1, 2), dtype=torch.long),
                ),
                patch.object(
                    campaign,
                    "_train_order_identity",
                    return_value={"order_cid": "order"},
                ),
                patch.object(campaign, "_write_progress"),
                patch.object(campaign.time, "monotonic", return_value=18.0),
            ):
                result = campaign._finalize_shared_arm(
                    root,
                    arm="geometric",
                    model=model,
                    preparation=preparation,
                    device=torch.device("cpu"),
                    backend={},
                    run_contract_cid="contract",
                    plan_cid="plan",
                    elapsed_before_seconds=5.0,
                    process_started=10.0,
                    wall_seconds=100.0,
                    last_loss=1.0,
                    base_initial=b"base",
                )
            self.assertEqual(result["elapsed_seconds"], 13.0)

    def test_shared_pair_stops_before_second_finalizer_after_wall(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = Mock()
            model.frozen_base_parameters.return_value = ()
            parameters = {
                arm: (torch.nn.Parameter(torch.zeros(1)),)
                for arm in campaign.ARMS
            }
            model.head_parameters.side_effect = lambda arm: parameters[arm]
            model.export_qualified_base_artifact.return_value = b"base"
            model.export_head_artifact.side_effect = [
                b"geometric-zero",
                b"pooled-zero",
                b"geometric-fit",
                b"pooled-fit",
            ]
            preparation = Mock()
            preparation.root = root
            finalizer = Mock(
                return_value={
                    "arm": "geometric",
                    "status": campaign.TERMINAL_UNAVAILABLE,
                    "progress": {"elapsed_seconds": 100.0},
                }
            )

            def progress(*_args, **kwargs):
                return {"elapsed_seconds": kwargs["elapsed_seconds"]}

            with (
                patch.object(campaign, "OPTIMIZER_STEPS", 0),
                patch.object(
                    campaign,
                    "_configure_device",
                    return_value=(torch.device("cpu"), {}),
                ),
                patch.object(
                    campaign,
                    "load_learned_associative_readout_preparation",
                    return_value=preparation,
                ),
                patch.object(campaign, "_new_model", return_value=model),
                patch.object(campaign, "_head_optimizer", return_value=Mock()),
                patch.object(campaign, "_save_shared_checkpoint"),
                patch.object(campaign, "_write_progress", side_effect=progress),
                patch.object(campaign, "_finalize_shared_arm", finalizer),
                patch.object(campaign.time, "monotonic", return_value=10.0),
            ):
                outcomes = campaign._train_shared(
                    root,
                    campaign.ELIGIBLE_PLANS[0],
                    run_contract_cid="contract",
                    resume=False,
                    wall_seconds=1_000.0,
                )
            finalizer.assert_called_once()
            self.assertEqual(
                outcomes["geometric"]["status"], campaign.TERMINAL_UNAVAILABLE
            )
            self.assertEqual(
                outcomes["pooled"]["status"], campaign.TERMINAL_UNAVAILABLE
            )
            self.assertEqual(
                outcomes["pooled"]["reason"],
                "SHARED_PAIR_FINALIZATION_WALL_EXHAUSTED",
            )

    def test_final_scoring_uses_residual_deadline_and_charges_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation, probe, started, arm_results, _reveal = (
                _post_reveal_fixture(root)
            )
            observed_timeouts: list[float] = []

            def scorer(_preparation, _reveal, _arm_results, timeout_seconds):
                observed_timeouts.append(timeout_seconds)
                return {
                    "ok": True,
                    "evidence": {},
                    "scoring_seconds": 2.0,
                    "cached": False,
                }

            with (
                patch.object(
                    campaign,
                    "_final_mechanics",
                    return_value={"passed": True},
                ),
                patch.object(
                    campaign,
                    "_decision_from_evidence",
                    return_value={
                        "verdict": campaign.TERMINAL_CONTROL_ONLY,
                        "action": "test",
                        "selected_arm": "pooled",
                    },
                ),
            ):
                result = campaign._finalize_result(
                    preparation,
                    probe,
                    started,
                    arm_results,
                    scoring_executor=scorer,
                )
            self.assertEqual(len(observed_timeouts), 1)
            self.assertAlmostEqual(
                observed_timeouts[0],
                campaign.HARD_WALL_CEILING_SECONDS
                - 100.0
                - campaign.RESULT_FINALIZATION_RESERVE_SECONDS,
            )
            self.assertEqual(result["timing"]["training_seconds"], 100.0)
            self.assertGreaterEqual(
                result["timing"]["scoring_seconds"],
                2.0 + campaign.RESULT_FINALIZATION_RESERVE_SECONDS,
            )
            self.assertLessEqual(
                result["timing"]["total_seconds"],
                campaign.HARD_WALL_CEILING_SECONDS,
            )

    def test_scoring_timeout_preserves_fixed_artifacts_for_scoring_only_resume(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation, probe, started, arm_results, _reveal = (
                _post_reveal_fixture(root)
            )
            before = {
                arm: campaign._artifact_path(root, arm).read_bytes()
                for arm in campaign.ARMS
            }

            def timeout(_preparation, _reveal, _arm_results, _timeout_seconds):
                return {
                    "ok": False,
                    "error": {"type": "TimeoutError", "reason": "deadline"},
                }

            unavailable = campaign._finalize_result(
                preparation,
                probe,
                started,
                arm_results,
                scoring_executor=timeout,
            )
            self.assertEqual(unavailable["verdict"], campaign.TERMINAL_UNAVAILABLE)
            self.assertEqual(unavailable["phase"], "POST_REVEAL_SCORING")
            self.assertTrue(unavailable["reveal_created"])
            self.assertFalse(unavailable["optimizer_created_after_reveal"])
            self.assertEqual(unavailable["optimizer_steps_after_reveal"], 0)
            self.assertFalse((root / campaign.RESULT_RELATIVE_PATH).exists())
            for arm in campaign.ARMS:
                self.assertEqual(campaign._artifact_path(root, arm).read_bytes(), before[arm])

    def test_inconsistent_terminal_timing_is_rejected(self) -> None:
        arm_results = {
            "geometric": {"elapsed_seconds": 10.0},
            "pooled": {"elapsed_seconds": 8.0},
        }
        result = {
            "timing": {
                "training_seconds": 10.0,
                "scoring_seconds": 2.0,
                "total_seconds": 12.5,
                "hard_wall_seconds": campaign.HARD_WALL_CEILING_SECONDS,
            }
        }
        with self.assertRaisesRegex(ValueError, "does not reproduce"):
            campaign._validate_result_timing(result, arm_results)

    def test_concurrent_checkpoint_recovers_from_interrupted_sidecar_publish(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = _FakeModel()
            arm = "geometric"
            with torch.no_grad():
                model.parameters_by_arm[arm].fill_(0.25)
            optimizer = torch.optim.AdamW(model.head_parameters(arm), lr=campaign.learning_rate(1))
            campaign._save_checkpoint(
                root,
                arm=arm,
                model=model,
                optimizer=optimizer,
                step=1,
                elapsed_seconds=1.0,
                run_contract_cid="run",
                plan_cid="plan",
                last_loss=2.0,
            )
            campaign._checkpoint_cid_path(root, arm).write_bytes(b"interrupted")
            replay = _FakeModel()
            replay_optimizer = torch.optim.AdamW(
                replay.head_parameters(arm), lr=campaign.learning_rate(1)
            )
            loaded = campaign._load_checkpoint(
                root,
                arm=arm,
                model=replay,
                optimizer=replay_optimizer,
                device=torch.device("cpu"),
                run_contract_cid="run",
                plan_cid="plan",
            )
            self.assertEqual(loaded["step"], 1)
            self.assertTrue(torch.equal(
                replay.parameters_by_arm[arm], model.parameters_by_arm[arm]
            ))
            campaign._verify_self_cid(
                campaign._read_json(campaign._checkpoint_cid_path(root, arm)),
                "sidecar_cid",
            )

    def test_shared_checkpoint_is_one_atomic_pair_and_repairs_sidecar(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = _FakeModel()
            with torch.no_grad():
                model.parameters_by_arm["geometric"].fill_(0.25)
                model.parameters_by_arm["pooled"].fill_(-0.5)
            optimizers = {
                arm: torch.optim.AdamW(
                    model.head_parameters(arm), lr=campaign.learning_rate(1)
                )
                for arm in campaign.ARMS
            }
            campaign._save_shared_checkpoint(
                root,
                model=model,
                optimizers=optimizers,
                step=1,
                elapsed_seconds=1.0,
                run_contract_cid="run",
                plan_cid="plan",
                last_losses={"geometric": 2.0, "pooled": 3.0},
            )
            campaign._shared_checkpoint_cid_path(root).write_bytes(b"interrupted")
            replay = _FakeModel()
            replay_optimizers = {
                arm: torch.optim.AdamW(
                    replay.head_parameters(arm), lr=campaign.learning_rate(1)
                )
                for arm in campaign.ARMS
            }
            loaded = campaign._load_shared_checkpoint(
                root,
                model=replay,
                optimizers=replay_optimizers,
                device=torch.device("cpu"),
                run_contract_cid="run",
                plan_cid="plan",
            )
            self.assertEqual(loaded["step"], 1)
            for arm in campaign.ARMS:
                self.assertTrue(torch.equal(
                    replay.parameters_by_arm[arm], model.parameters_by_arm[arm]
                ))
            repaired = campaign._read_json(campaign._shared_checkpoint_cid_path(root))
            campaign._verify_self_cid(repaired, "sidecar_cid")
            self.assertEqual(set(repaired["head_artifact_cids"]), set(campaign.ARMS))


if __name__ == "__main__":
    unittest.main()
