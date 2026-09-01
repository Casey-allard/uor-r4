"""Focused synthetic contracts for the bounded layerwise-normalized-readout capacity campaign."""

from __future__ import annotations

import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch
from r4_softmax_trainer import layerwise_normalized_retained_readout_campaign as subject


def _metric(
    *, ce: float, top1_rate: float, state_off: bool = False
) -> dict[str, object]:
    return {
        "rows": subject.FRESH_HELDOUT_DECISIONS,
        "ce_nats": ce,
        "top1_correct": round(top1_rate * subject.FRESH_HELDOUT_DECISIONS),
        "top1_rate": top1_rate,
        "forbidden_reads": 0,
        "state_off": state_off,
    }


class FrozenContractTests(unittest.TestCase):
    def test_population_training_and_fresh_heldout_arithmetic(self) -> None:
        self.assertEqual(
            subject.PRIOR_REVEALED_PROMPT_LAST_SOURCE_STORY_ORDINAL, 241_074
        )
        self.assertEqual(subject.OPTIMIZER_STEPS, 2_730)
        self.assertEqual(subject.TRAIN_WINDOWS, 43_680)
        self.assertEqual(subject.TRAIN_DECISIONS, 5_241_600)
        self.assertEqual(subject.FRESH_HELDOUT_WINDOWS, 2_066)
        self.assertEqual(subject.FRESH_HELDOUT_TOKENS, 2_066 * 121)
        self.assertEqual(subject.FRESH_HELDOUT_DECISIONS, 247_920)
        self.assertEqual(subject.FRESH_HELDOUT_SOURCE_OFFSET_TOKENS, 155_782_142)
        self.assertEqual(
            subject.FRESH_HELDOUT_CID,
            "blake3:79e5e74e3e85f10ed8eb44ea7c37fca7fceba4e2cb2c227db0f37340fcf4d0f3",
        )
        self.assertEqual(subject.FRESH_HELDOUT_FIRST_CAPACITY_STORY, 762_819)
        self.assertEqual(subject.FRESH_HELDOUT_FIRST_SOURCE_STORY, 847_141)
        self.assertEqual(subject.FRESH_HELDOUT_LAST_CAPACITY_STORY, 764_049)
        self.assertEqual(subject.FRESH_HELDOUT_LAST_SOURCE_STORY, 848_492)
        self.assertEqual(subject.FRESH_HELDOUT_STORY_CIDS, 1_231)
        self.assertLess(
            subject.PRIOR_REVEALED_PROMPT_LAST_SOURCE_STORY_ORDINAL,
            subject.FRESH_HELDOUT_FIRST_SOURCE_STORY,
        )
        self.assertEqual(subject.CPU_PLAN.backend, "cpu")
        self.assertEqual(subject.CPU_PLAN.threads_per_worker, 4)
        self.assertEqual(subject.CPU_PLAN.workers, 1)
        self.assertFalse(subject.CPU_PLAN.concurrent_arms)
        self.assertEqual(subject.REQUIRED_STATE_OFF_NLL_COST, 0.10)
        self.assertEqual(subject.REQUIRED_STATE_OFF_TOP1_DECISION_LOSS, 2_480)

    def test_projection_admits_only_bounded_time_and_memory(self) -> None:
        admitted = subject.project_candidate_execution(
            mean_train_step_seconds=0.4,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=100,
            memory_budget_bytes=1_000,
        )
        self.assertTrue(admitted["eligible"])
        self.assertEqual(admitted["reason"], "ELIGIBLE")
        self.assertEqual(admitted["safety_factor"], 1.25)
        self.assertEqual(admitted["projection_ceiling_seconds"], 3_000.0)
        self.assertEqual(admitted["hard_wall_ceiling_seconds"], 3_600.0)

        wall = subject.project_candidate_execution(
            mean_train_step_seconds=1.0,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=100,
            memory_budget_bytes=1_000,
        )
        self.assertFalse(wall["eligible"])
        self.assertEqual(wall["reason"], "WALL_PROJECTION")

        memory = subject.project_candidate_execution(
            mean_train_step_seconds=0.4,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=801,
            memory_budget_bytes=1_000,
        )
        self.assertFalse(memory["eligible"])
        self.assertEqual(memory["reason"], "MEMORY")


class ScientificDecisionTests(unittest.TestCase):
    def test_fresh_generalization_passes_learning_and_predecessor_nonregression(
        self,
    ) -> None:
        decision = subject.fresh_generalization_gates(
            candidate_initial=_metric(ce=8.3, top1_rate=0.001),
            candidate_final=_metric(ce=3.9, top1_rate=0.30),
            predecessor=_metric(ce=3.88, top1_rate=0.305),
            candidate_state_off=_metric(ce=4.1, top1_rate=0.28, state_off=True),
            predecessor_state_off=_metric(ce=4.1, top1_rate=0.28, state_off=True),
        )
        self.assertTrue(decision["passed"])
        self.assertTrue(all(decision["gates"].values()))

    def test_fresh_generalization_rejects_a_prompt_only_regression(self) -> None:
        decision = subject.fresh_generalization_gates(
            candidate_initial=_metric(ce=8.3, top1_rate=0.001),
            candidate_final=_metric(ce=4.2, top1_rate=0.25),
            predecessor=_metric(ce=3.8, top1_rate=0.30),
            candidate_state_off=_metric(ce=4.4, top1_rate=0.23, state_off=True),
            predecessor_state_off=_metric(ce=4.1, top1_rate=0.28, state_off=True),
        )
        self.assertFalse(decision["passed"])
        self.assertFalse(decision["gates"]["candidate_final_nll_ceiling"])
        self.assertFalse(decision["gates"]["predecessor_nll_nonregression"])

    def test_fresh_generalization_requires_load_bearing_retained_state(self) -> None:
        decision = subject.fresh_generalization_gates(
            candidate_initial=_metric(ce=8.3, top1_rate=0.001),
            candidate_final=_metric(ce=3.9, top1_rate=0.30),
            predecessor=_metric(ce=3.88, top1_rate=0.305),
            candidate_state_off=_metric(ce=3.95, top1_rate=0.295, state_off=True),
            predecessor_state_off=_metric(ce=4.1, top1_rate=0.28, state_off=True),
        )
        self.assertFalse(decision["passed"])
        self.assertFalse(decision["gates"]["candidate_state_off_nll_load_bearing"])
        self.assertFalse(decision["gates"]["candidate_state_off_top1_load_bearing"])

    def test_terminal_outcomes_close_parameter_free_ladder_on_every_valid_miss(
        self,
    ) -> None:
        cases = {
            subject.VERDICT_PASS: subject.TERMINAL_PASS,
            subject.VERDICT_ABSOLUTE_NO_CAPACITY_GAIN: (
                subject.TERMINAL_ABSOLUTE_NO_CAPACITY_GAIN
            ),
            subject.VERDICT_PARTIAL: subject.TERMINAL_PARTIAL,
            subject.VERDICT_FAIL: subject.TERMINAL_FAIL,
            subject.VERDICT_INVALID: subject.TERMINAL_INVALID,
        }
        actions: dict[str, str] = {}
        for prompt, expected in cases.items():
            result = subject.combine_terminal_verdict(
                prompt_verdict=prompt,
                language_passed=True,
                mechanics_passed=True,
            )
            self.assertEqual(result["verdict"], expected)
            actions[prompt] = result["action"]
        pivot = (
            "end the parameter-free readout ladder and pivot to a freshly "
            "frozen learned associative binding/readout"
        )
        for prompt in (
            subject.VERDICT_ABSOLUTE_NO_CAPACITY_GAIN,
            subject.VERDICT_PARTIAL,
            subject.VERDICT_FAIL,
        ):
            self.assertEqual(actions[prompt], pivot)
        self.assertNotEqual(actions[subject.VERDICT_PASS], pivot)
        self.assertNotEqual(actions[subject.VERDICT_INVALID], pivot)

        regression = subject.combine_terminal_verdict(
            prompt_verdict=subject.VERDICT_PASS,
            language_passed=False,
            mechanics_passed=True,
        )
        self.assertEqual(
            regression["verdict"], subject.TERMINAL_GENERAL_LANGUAGE_REGRESSION
        )
        self.assertEqual(regression["action"], pivot)


class RecoveryAndProbeTests(unittest.TestCase):
    def test_reveal_closes_fitting_but_allows_explicit_finalization_recovery(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reveal = root / subject.REVEAL_RELATIVE_PATH
            reveal.parent.mkdir(parents=True)
            reveal.write_text("revealed", encoding="utf-8")
            with mock.patch.object(
                subject, "load_layerwise_normalized_retained_readout_preparation"
            ) as loader:
                with self.assertRaisesRegex(ValueError, "use --resume"):
                    subject.run_layerwise_normalized_retained_readout(root)
                loader.assert_not_called()
            with (
                mock.patch.object(
                    subject,
                    "_recover_revealed_prompt_access",
                    return_value={},
                ) as recover,
                mock.patch.object(
                    subject,
                    "load_layerwise_normalized_retained_readout_preparation",
                    side_effect=RuntimeError("entered finalization"),
                ) as loader,
                self.assertRaisesRegex(RuntimeError, "entered finalization"),
            ):
                subject.run_layerwise_normalized_retained_readout(root, resume=True)
            recover.assert_called_once_with(root.resolve())
            loader.assert_called_once_with(root)

    def test_finalization_loader_requires_a_completed_checkpoint_without_optimizer(
        self,
    ) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=0.0)
        initial = _metric(ce=8.3, top1_rate=0.0)
        contract_cid = "blake3:" + "1" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subject._save_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_seconds=1.0,
                initial_heldout=initial,
                run_contract_cid=contract_cid,
                last_loss=None,
            )
            with self.assertRaisesRegex(ValueError, "completed frozen checkpoint"):
                subject._load_completed_checkpoint_for_finalization(
                    root,
                    model=torch.nn.Linear(2, 2, bias=False),
                    run_contract_cid=contract_cid,
                )

            subject._save_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                step=subject.OPTIMIZER_STEPS,
                elapsed_seconds=2.0,
                initial_heldout=initial,
                run_contract_cid=contract_cid,
                last_loss=3.0,
            )
            replay = torch.nn.Linear(2, 2, bias=False)
            checkpoint = subject._load_completed_checkpoint_for_finalization(
                root,
                model=replay,
                run_contract_cid=contract_cid,
            )
            self.assertEqual(checkpoint["step"], subject.OPTIMIZER_STEPS)
            self.assertTrue(
                all(
                    torch.equal(left, right)
                    for left, right in zip(model.parameters(), replay.parameters())
                )
            )

    def test_checkpoint_round_trip_and_tamper_rejection(self) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=0.0)
        initial = _metric(ce=8.3, top1_rate=0.0)
        contract_cid = "blake3:" + "1" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subject._save_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_seconds=1.0,
                initial_heldout=initial,
                run_contract_cid=contract_cid,
                last_loss=None,
            )
            checkpoint = subject._load_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                device=torch.device("cpu"),
                run_contract_cid=contract_cid,
            )
            self.assertEqual(checkpoint["step"], 0)
            with (root / subject.CHECKPOINT_RELATIVE_PATH).open("ab") as target:
                target.write(b"tamper")
            with mock.patch.object(subject.torch, "load") as deserialize:
                with self.assertRaisesRegex(ValueError, "checkpoint CID"):
                    subject._load_checkpoint(
                        root,
                        model=model,
                        optimizer=optimizer,
                        device=torch.device("cpu"),
                        run_contract_cid=contract_cid,
                    )
                deserialize.assert_not_called()

    def test_probe_is_one_candidate_cpu4_five_step_gate(self) -> None:
        execution = {
            "probe_steps": subject.PROBE_STEPS,
            "mechanics": {"passed": True},
            "projection": {"eligible": True},
        }
        implementation = {"files": [], "tree_cid": "blake3:" + "2" * 64}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation = SimpleNamespace(
                root=root,
                manifest={
                    "preparation_cid": "blake3:" + "3" * 64,
                    "implementation": implementation,
                },
                commitment={"commitment_cid": "blake3:" + "4" * 64},
            )
            with (
                mock.patch.object(
                    subject,
                    "load_layerwise_normalized_retained_readout_preparation",
                    return_value=preparation,
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value=implementation,
                ),
            ):
                result = subject.probe_layerwise_normalized_retained_readout(
                    root,
                    _executor=lambda prepared: execution,
                )
        self.assertTrue(result["eligible"])
        self.assertEqual(result["plan"], subject.CPU_PLAN.identity())
        self.assertEqual(result["execution"]["probe_steps"], 5)
        self.assertEqual(result["cuda"], "FORBIDDEN")
        self.assertEqual(result["mps"], "NOT_USED")

    def test_verifier_rejects_same_process_as_terminal_writer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                mock.patch.object(
                    subject,
                    "_load_terminal_result",
                    return_value={"writer_process_id": subject.os.getpid()},
                ),
                self.assertRaisesRegex(ValueError, "fresh process"),
            ):
                subject.verify_layerwise_normalized_retained_readout_result(root)

    def test_verifier_recomputes_every_terminal_input_and_rejects_mismatch(
        self,
    ) -> None:
        def verify_once(
            root: Path, *, mismatch: bool
        ) -> tuple[dict[str, object], mock.Mock]:
            root = root.resolve()
            implementation = {
                "files": [],
                "tree_cid": "blake3:" + "1" * 64,
            }
            preparation_cid = "blake3:" + "2" * 64
            population_cid = "blake3:" + "3" * 64
            predecessor_path = root / "predecessor.safetensors"
            predecessor_path.write_bytes(b"frozen predecessor")
            candidate_path = root / subject.CANDIDATE_ARTIFACT_RELATIVE_PATH
            candidate_path.parent.mkdir(parents=True)
            candidate_artifact = b"fitted candidate"
            candidate_path.write_bytes(candidate_artifact)
            candidate_cid = subject.cid_bytes(candidate_artifact)
            geometry = object()
            heldout = object()
            preparation = SimpleNamespace(
                manifest={
                    "preparation_cid": preparation_cid,
                    "implementation": implementation,
                    "prompt_population": {"population_cid": population_cid},
                },
                predecessor=object(),
                predecessor_artifact_path=predecessor_path,
                fresh_heldout=heldout,
            )
            probe = subject._with_cid(
                {
                    "preparation_cid": preparation_cid,
                    "implementation": implementation,
                    "eligible": True,
                },
                "probe_cid",
            )
            run_contract = {"frozen": "run contract"}
            run_contract_cid = subject.cid_bytes(
                subject.canonical_json_bytes(run_contract)
            )
            started = subject._with_cid(
                {
                    "run_contract": run_contract,
                    "run_contract_cid": run_contract_cid,
                },
                "started_cid",
            )
            reveal = subject._with_cid(
                {
                    "population_cid": population_cid,
                    "baseline_artifact_cid": subject.PREDECESSOR_ARTIFACT_CID,
                    "candidate_artifact_cid": candidate_cid,
                },
                "reveal_cid",
            )
            prompt = {"verdict": subject.VERDICT_PASS, "score": "replayed"}
            initial = {"metric": "fresh-untrained-initial"}
            candidate = {"metric": "fresh-fitted-candidate"}
            predecessor = {"metric": "fresh-frozen-predecessor"}
            candidate_off = {"metric": "fresh-candidate-state-off"}
            predecessor_off = {"metric": "fresh-predecessor-state-off"}
            language = {"passed": True, "gates": {"all": True}}
            replay = {"passed": True, "artifact": "fresh-replay"}
            terminal = {
                "verdict": subject.TERMINAL_PASS,
                "action": "freeze candidate",
            }
            stored_terminal = (
                {"verdict": "STALE", "action": "wrong branch"} if mismatch else terminal
            )
            result = subject._with_cid(
                {
                    "writer_process_id": subject.os.getpid() + 1,
                    "implementation": implementation,
                    "probe_cid": probe["probe_cid"],
                    "run_contract_cid": run_contract_cid,
                    "started_cid": started["started_cid"],
                    "completed_steps": subject.OPTIMIZER_STEPS,
                    "presentations": subject.TRAIN_DECISIONS,
                    "backend": {"device": "cpu", "threads": 4},
                    "candidate_artifact": {
                        "path": subject.CANDIDATE_ARTIFACT_RELATIVE_PATH,
                        "cid": candidate_cid,
                    },
                    "prompt_reveal": {
                        "cid": reveal["reveal_cid"],
                        "population_cid": population_cid,
                    },
                    "prompt_decision": prompt,
                    "fresh_heldout": {
                        "candidate_initial": initial,
                        "candidate_final": candidate,
                        "predecessor": predecessor,
                        "candidate_state_off": candidate_off,
                        "predecessor_state_off": predecessor_off,
                        "decision": language,
                    },
                    "artifact_replay": replay,
                    "mechanics_passed": True,
                    "prompt_evaluation_seconds": 1.0,
                    "prompt_evaluation_within_ceiling": True,
                    "elapsed_seconds": 2.0,
                    "decision": stored_terminal,
                    "verdict": stored_terminal["verdict"],
                },
                "result_cid",
            )

            def read_envelope(path: Path) -> dict[str, object]:
                envelopes = {
                    root / subject.PROBE_RELATIVE_PATH: probe,
                    root / subject.STARTED_RELATIVE_PATH: started,
                    root / subject.REVEAL_RELATIVE_PATH: reveal,
                }
                return envelopes[path]

            initial_model = mock.Mock()
            initial_model.to.return_value = "fresh-untrained-model"
            model_type = mock.Mock(return_value=initial_model)
            baseline_factory = mock.Mock(side_effect=lambda: object())
            candidate_factory = mock.Mock(side_effect=lambda: object())
            language_evaluator = mock.Mock(
                side_effect=[
                    initial,
                    candidate,
                    predecessor,
                    candidate_off,
                    predecessor_off,
                ]
            )
            prompt_evaluator = mock.Mock(
                return_value=SimpleNamespace(record=lambda: dict(prompt))
            )
            terminal_evaluator = mock.Mock(return_value=terminal)
            with ExitStack() as stack:
                stack.enter_context(
                    mock.patch.object(
                        subject, "_load_terminal_result", return_value=result
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject,
                        "load_layerwise_normalized_retained_readout_preparation",
                        return_value=preparation,
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "_require_implementation", return_value=implementation
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject,
                        "_configure_device",
                        return_value=(
                            torch.device("cpu"),
                            {"device": "cpu", "threads": 4},
                        ),
                    )
                )
                stack.enter_context(
                    mock.patch.object(subject, "_exact_geometry", return_value=geometry)
                )
                stack.enter_context(
                    mock.patch.object(subject, "_read_canonical_json", read_envelope)
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "_run_contract", return_value=run_contract
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject,
                        "load_revealed_prompt_conditioning_population",
                        return_value=object(),
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject,
                        "_factories",
                        return_value=(baseline_factory, candidate_factory),
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "evaluate_prompt_conditioning", prompt_evaluator
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject,
                        "R4LayerwiseNormalizedRetainedReadoutLanguagePathV1",
                        model_type,
                    )
                )
                stack.enter_context(
                    mock.patch.object(subject, "_evaluate_language", language_evaluator)
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "fresh_generalization_gates", return_value=language
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "_candidate_artifact_replay", return_value=replay
                    )
                )
                stack.enter_context(
                    mock.patch.object(
                        subject, "_control_mechanics_passed", return_value=True
                    )
                )
                stack.enter_context(
                    mock.patch.object(subject, "_terminal_decision", terminal_evaluator)
                )
                verification = (
                    subject.verify_layerwise_normalized_retained_readout_result(root)
                )

            self.assertEqual(language_evaluator.call_count, 5)
            self.assertEqual(
                language_evaluator.call_args_list[0].args[0],
                "fresh-untrained-model",
            )
            self.assertEqual(baseline_factory.call_count, 2)
            self.assertEqual(candidate_factory.call_count, 3)
            prompt_evaluator.assert_called_once()
            terminal_evaluator.assert_called_once_with(
                prompt_verdict=subject.VERDICT_PASS,
                language_passed=True,
                mechanics_passed=True,
                prompt_evaluation_seconds=1.0,
                elapsed_seconds=2.0,
            )
            return verification, language_evaluator

        with tempfile.TemporaryDirectory() as directory:
            verification, evaluator = verify_once(Path(directory), mismatch=False)
            self.assertTrue(verification["passed"])
            self.assertTrue(all(verification["comparisons"].values()))
            self.assertEqual(evaluator.call_count, 5)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "re-score differs"):
                verify_once(root, mismatch=True)
            self.assertFalse((root / subject.VERIFICATION_RELATIVE_PATH).exists())


if __name__ == "__main__":
    unittest.main()
