"""Focused phase, decision, and frozen-contract tests for issue #1043."""

from __future__ import annotations

import inspect
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch

from r4_softmax_trainer import position_kv_binding_campaign as campaign


IMPLEMENTATION = {
    "files": [{"path": "test", "bytes": 1, "cid": "blake3:" + "1" * 64}],
    "tree_cid": "blake3:" + "2" * 64,
    "torch": torch.__version__,
    "platform": "test",
}


def _score(decisions: int, correct: int, *, nll: float = 1.0, **extra: object) -> dict:
    execution = str(extra.pop("execution", "r4"))
    return {
        "decisions": decisions,
        "top1_correct": correct,
        "top1_rate": correct / decisions,
        "nll_nats": nll,
        "selected_logits_cid": "blake3:" + "3" * 64,
        "work": _work(decisions, execution=execution),
        **extra,
    }


def _work(
    target_reads: int,
    *,
    execution: str,
    token_steps: int | None = None,
    transported_materialized: int | None = None,
) -> dict[str, int]:
    token_steps = target_reads if token_steps is None else token_steps
    materialized = token_steps
    transported_source = (
        materialized if transported_materialized is None else transported_materialized
    )
    return {
        "token_steps": token_steps,
        "cache_writes": token_steps
        * campaign.LAYERS
        * 2
        * campaign.HEADS
        * campaign.HEAD_DIM,
        "materialized_attention_scores": materialized,
        "admitted_attention_scores": materialized,
        "transported_r4_blocks": (
            transported_source * 2 * (campaign.HEAD_DIM // 4)
            if execution == "r4"
            else 0
        ),
        "value_reads": materialized * campaign.HEAD_DIM,
        "vocabulary_scores": token_steps * campaign.VOCAB_SIZE,
        "target_reads": target_reads,
        "source_reads": token_steps,
        "provider_calls": 0,
        "teacher_calls": 0,
        "future_reads": 0,
        "forbidden_reads": 0,
    }


def _passing_metrics() -> dict:
    metrics = {
        "mqar": {
            "native": _score(campaign.TERMINAL_MQAR_DECISIONS, 8_192),
            "current_only": _score(campaign.TERMINAL_MQAR_DECISIONS, 2_000),
            "value_permuted": _score(campaign.TERMINAL_MQAR_DECISIONS, 2_000),
            "binding_permuted": _score(campaign.TERMINAL_MQAR_DECISIONS, 2_000),
            "transport_mismatch": _score(campaign.TERMINAL_MQAR_DECISIONS, 5_000),
        },
        "english": {
            "history": _score(campaign.TERMINAL_ENGLISH_HISTORY_DECISIONS, 500),
            "binding_permuted": _score(
                campaign.TERMINAL_ENGLISH_HISTORY_DECISIONS, 100
            ),
            "no_history": _score(
                campaign.TERMINAL_ENGLISH_NO_HISTORY_DECISIONS,
                500,
                assigned_answer_top1_correct=0,
                assigned_answer_top1_rate=0.0,
                unsupported_assigned_value_top1=0,
            ),
        },
        "language": {
            "initialization": _score(
                campaign.TERMINAL_NATURAL_DECISIONS,
                50_000,
                nll=2.0,
                execution="plain",
            ),
            "fitted": _score(
                campaign.TERMINAL_NATURAL_DECISIONS,
                50_000,
                nll=2.01,
                execution="plain",
            ),
        },
        "parity": {
            "decisions": campaign.TERMINAL_PARITY_DECISIONS,
            "r4_plain_attention_weight_max_delta": 1.0e-7,
            "r4_plain_logit_max_delta": 1.0e-6,
            "r4_plain_top1_identical": True,
            "full_incremental_logit_max_delta": 1.0e-6,
            "full_incremental_top1_identical": True,
            "work": _work(
                3 * campaign.TERMINAL_PARITY_DECISIONS,
                execution="r4",
                transported_materialized=2 * campaign.TERMINAL_PARITY_DECISIONS,
            ),
        },
        "replay": {
            "artifact_bytes_identical": True,
            "logits_identical": True,
            "attention_weights_identical": True,
            "artifact_cid": "blake3:" + "4" * 64,
            "decisions": campaign.TERMINAL_REPLAY_DECISIONS,
            "replay_logits_cid": "blake3:" + "5" * 64,
            "passed": True,
            "work": _work(
                2 * campaign.TERMINAL_REPLAY_DECISIONS,
                execution="r4",
            ),
        },
    }
    work = campaign._aggregate_evaluation_work(metrics)
    metrics["work"] = work
    metrics["leakage"] = {
        name: work[name]
        for name in (
            "target_reads",
            "source_reads",
            "provider_calls",
            "teacher_calls",
            "future_reads",
            "forbidden_reads",
        )
    }
    return metrics


def _phase_file(root: Path, relative: str, body: dict, cid_field: str) -> dict:
    value = campaign._with_cid(body, cid_field)
    campaign._write_exclusive_json(root / relative, value)
    return value


def _preparation(root: Path, manifest: dict) -> dict:
    return _phase_file(
        root,
        campaign.PREPARATION_RELATIVE_PATH,
        {
            "schema": campaign.PREPARATION_SCHEMA,
            "issue": campaign.ISSUE,
            "policy": campaign.POLICY,
            "implementation": IMPLEMENTATION,
            "data_manifest": manifest,
            "data_manifest_cid": campaign._manifest_cid(manifest),
        },
        "preparation_cid",
    )


class _FakeDataAPI:
    def __init__(self, construction: object, *, terminal: object | None = None) -> None:
        self.construction = construction
        self.terminal = terminal
        self.events: list[str] = []

    def load_position_kv_binding_construction(self, _root: Path) -> object:
        return self.construction

    def reveal_position_kv_binding_terminal(
        self, root: Path, *, final_artifact_path: Path
    ) -> object:
        self.events.append("reveal")
        if not final_artifact_path.is_file():
            raise AssertionError("artifact must exist before reveal")
        assert self.terminal is not None
        campaign._write_exclusive_json(
            root / campaign.REVEAL_RELATIVE_PATH, self.terminal.reveal
        )
        return self.terminal

    def load_revealed_position_kv_binding_terminal(
        self, _root: Path, *, final_artifact_path: Path
    ) -> object:
        self.events.append("reload-reveal")
        if not final_artifact_path.is_file():
            raise AssertionError("artifact must exist before reveal replay")
        assert self.terminal is not None
        return self.terminal


class _FakeModel:
    def __init__(self, artifact: bytes = b"init") -> None:
        self.artifact = artifact

    def train(self) -> _FakeModel:
        return self

    def export_learned_artifact(self) -> bytes:
        return self.artifact

    def eval(self) -> _FakeModel:
        return self


class PositionKVBindingCampaignTests(unittest.TestCase):
    def test_prepare_rejects_imported_source_drift_before_root_mutation(self) -> None:
        drifted = {**IMPLEMENTATION, "tree_cid": "blake3:" + "9" * 64}
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory) / "not-created"
            root = parent / "campaign"
            with (
                patch.object(
                    campaign,
                    "_IMPORTED_IMPLEMENTATION_CONTRACT",
                    IMPLEMENTATION,
                ),
                patch.object(
                    campaign,
                    "_implementation_contract",
                    return_value=drifted,
                ),
                self.assertRaisesRegex(ValueError, "process-imported source"),
            ):
                campaign.prepare_position_kv_binding_campaign(
                    root,
                    retained_language_root=Path("unused-retained"),
                    source_root=Path("unused-source"),
                    tokenizer_path=Path("unused-tokenizer"),
                    geometry_path=Path("unused-geometry"),
                    h4_sidecar_path=Path("unused-frames"),
                    excluded_story_cids=(),
                )
            self.assertFalse(parent.exists())

    def test_prepare_refuses_mid_materialization_source_drift_without_envelope(self) -> None:
        drifted = {**IMPLEMENTATION, "tree_cid": "blake3:" + "8" * 64}

        def prepare_data(*, output_root: Path, **_kwargs: object) -> object:
            output_root.mkdir(parents=True)
            return SimpleNamespace(
                manifest={"manifest_cid": "blake3:" + "7" * 64}
            )

        data_api = SimpleNamespace(
            prepare_position_kv_binding_data=prepare_data,
        )
        geometry = SimpleNamespace(artifact_cid=campaign.GEOMETRY_ARTIFACT_CID)
        frames = SimpleNamespace(artifact_cid=campaign.H4_FRAME_ARTIFACT_CID)
        copied = {
            "path": "fixture",
            "bytes": 1,
            "cid": "blake3:" + "6" * 64,
        }
        exclusion = "blake3:" + "5" * 64

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "campaign"
            with (
                patch.object(
                    campaign,
                    "_IMPORTED_IMPLEMENTATION_CONTRACT",
                    IMPLEMENTATION,
                ),
                patch.object(
                    campaign,
                    "_implementation_contract",
                    side_effect=(IMPLEMENTATION, drifted),
                ),
                patch.object(
                    campaign,
                    "_validate_complete_story_exclusions",
                    return_value=(exclusion,),
                ),
                patch.object(campaign, "_data_module", return_value=data_api),
                patch.object(
                    campaign,
                    "_copy_verified_input",
                    return_value=copied,
                ),
                patch.object(
                    campaign,
                    "load_group_geometry_artifacts",
                    return_value=geometry,
                ),
                patch.object(
                    campaign.H4SpinFrameArtifactV1,
                    "load",
                    return_value=frames,
                ),
                self.assertRaisesRegex(ValueError, "frozen phase"),
            ):
                campaign.prepare_position_kv_binding_campaign(
                    root,
                    retained_language_root=Path("unused-retained"),
                    source_root=Path("unused-source"),
                    tokenizer_path=Path("unused-tokenizer"),
                    geometry_path=Path("unused-geometry"),
                    h4_sidecar_path=Path("unused-frames"),
                    excluded_story_cids=(exclusion,),
                )
            self.assertTrue(root.is_dir())
            self.assertFalse((root / campaign.PREPARATION_RELATIVE_PATH).exists())

    def test_public_phase_apis_do_not_accept_runner_injection(self) -> None:
        self.assertNotIn(
            "probe_runner",
            inspect.signature(
                campaign.preflight_position_kv_binding_campaign
            ).parameters,
        )
        self.assertNotIn(
            "scoring_runner",
            inspect.signature(
                campaign.finalize_position_kv_binding_campaign
            ).parameters,
        )

    def test_learning_rate_hits_all_frozen_endpoints(self) -> None:
        self.assertAlmostEqual(campaign.learning_rate(1), 1.0e-6)
        self.assertEqual(
            campaign.learning_rate(campaign.WARMUP_STEPS),
            campaign.PEAK_LEARNING_RATE,
        )
        self.assertEqual(
            campaign.learning_rate(campaign.OPTIMIZER_STEPS),
            campaign.FINAL_LEARNING_RATE,
        )
        with self.assertRaises(ValueError):
            campaign.learning_rate(0)

    def test_execution_selection_uses_fastest_eligible_cpu_plan(self) -> None:
        records = []
        totals = (100.0, 80.0, 90.0)
        for plan, total in zip(campaign.ELIGIBLE_PLANS, totals, strict=True):
            records.append(
                {
                    "plan": plan.identity(),
                    "deterministic_replay": True,
                    "projection": {"total_seconds": total},
                    "peak_memory_bytes": 100,
                }
            )
        selection = campaign.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertEqual(selection["selected_plan"]["threads"], 4)

        records[1]["projection"]["total_seconds"] = campaign.HARD_WALL_SECONDS + 1
        selection = campaign.select_execution_plan(records)
        self.assertEqual(selection["selected_plan"]["threads"], 8)

    def test_scoring_projection_counts_every_full_r4_and_incremental_batch(self) -> None:
        self.assertEqual(campaign.NATURAL_SCORE_BATCHES, 130)
        self.assertEqual(campaign.MQAR_SCORE_BATCHES, 64)
        self.assertEqual(campaign.ENGLISH_HISTORY_SCORE_BATCHES, 32)
        self.assertEqual(campaign.ENGLISH_NO_HISTORY_SCORE_BATCHES, 32)
        self.assertEqual(campaign.PROJECTED_PLAIN_FULL_BATCHES, 518)
        self.assertEqual(campaign.PROJECTED_R4_FULL_BATCHES, 676)
        self.assertEqual(campaign.PROJECTED_INCREMENTAL_BATCHES, 258)
        self.assertEqual(campaign.ENGLISH_BINDING_PERMUTED_DROP, 0.35)

    def test_oracle_normalization_uses_the_sealed_overlength_witness(self) -> None:
        result = campaign._normalize_oracle(
            {
                "mqar_correct": 8_192,
                "mqar_total": 8_192,
                "english_correct": 512,
                "english_total": 512,
                "ambiguous_bindings": 0,
                "missing_bindings": 0,
                "overlength_sequences": 0,
            }
        )
        self.assertTrue(result["passed"])
        self.assertEqual(result["maximum_context"], campaign.CONTEXT)

    def test_terminal_decision_has_one_nonoverlapping_branch_per_failure(self) -> None:
        passing = _passing_metrics()
        self.assertEqual(
            campaign.decide_position_kv_binding(passing)["verdict"],
            campaign.TERMINAL_PASS,
        )

        not_learned = _passing_metrics()
        not_learned["mqar"]["native"] = _score(8_192, 8_000)
        self.assertEqual(
            campaign.decide_position_kv_binding(not_learned)["verdict"],
            campaign.TERMINAL_NOT_LEARNED,
        )

        unattributed = _passing_metrics()
        unattributed["mqar"]["current_only"] = _score(8_192, 7_000)
        self.assertEqual(
            campaign.decide_position_kv_binding(unattributed)["verdict"],
            campaign.TERMINAL_UNATTRIBUTED,
        )

        synthetic = _passing_metrics()
        synthetic["english"]["history"] = _score(512, 400)
        self.assertEqual(
            campaign.decide_position_kv_binding(synthetic)["verdict"],
            campaign.TERMINAL_SYNTHETIC_ONLY,
        )

        english_unattributed = _passing_metrics()
        english_unattributed["english"]["binding_permuted"] = _score(512, 400)
        self.assertEqual(
            campaign.decide_position_kv_binding(english_unattributed)["verdict"],
            campaign.TERMINAL_SYNTHETIC_ONLY,
        )

        geometry_unattributed = _passing_metrics()
        geometry_unattributed["mqar"]["transport_mismatch"] = _score(8_192, 7_000)
        self.assertEqual(
            campaign.decide_position_kv_binding(geometry_unattributed)["verdict"],
            campaign.TERMINAL_GEOMETRY_UNATTRIBUTED,
        )

        regression = _passing_metrics()
        regression["language"]["fitted"] = _score(
            247_920, 40_000, nll=2.2, execution="plain"
        )
        self.assertEqual(
            campaign.decide_position_kv_binding(regression)["verdict"],
            campaign.TERMINAL_LANGUAGE_REGRESSION,
        )

        invalid = _passing_metrics()
        invalid["leakage"]["future_reads"] = 1
        self.assertEqual(
            campaign.decide_position_kv_binding(invalid)["verdict"],
            campaign.TERMINAL_INVALID,
        )

        nonfinite = _passing_metrics()
        nonfinite["language"]["fitted"]["nll_nats"] = float("nan")
        self.assertEqual(
            campaign.decide_position_kv_binding(nonfinite)["verdict"],
            campaign.TERMINAL_INVALID,
        )

        boolean_ledger = _passing_metrics()
        boolean_ledger["leakage"]["future_reads"] = False
        self.assertEqual(
            campaign.decide_position_kv_binding(boolean_ledger)["verdict"],
            campaign.TERMINAL_INVALID,
        )

        wrong_work = _passing_metrics()
        wrong_work["mqar"]["native"]["work"]["cache_writes"] += 1
        self.assertEqual(
            campaign.decide_position_kv_binding(wrong_work)["verdict"],
            campaign.TERMINAL_INVALID,
        )

    def test_preflight_consumes_public_oracle_without_reveal(self) -> None:
        manifest = {"schema": "test-data", "manifest_cid": "manifest"}
        commitment = {
            "direct_serialization_oracle": {
                "mqar_correct": 8_192,
                "mqar_total": 8_192,
                "english_correct": 512,
                "english_total": 512,
                "ambiguous_bindings": 0,
                "missing_bindings": 0,
                "overlength_sequences": 0,
            }
        }
        construction = SimpleNamespace(manifest=manifest, commitment=commitment)
        data_api = _FakeDataAPI(construction)

        def runner(_root: Path, plan: campaign.ExecutionPlan) -> dict:
            return {
                "plan": plan.identity(),
                "deterministic_replay": True,
                "projection": {"total_seconds": 100.0 + plan.threads},
                "peak_memory_bytes": 100,
            }

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation = _preparation(root, manifest)
            with (
                patch.object(campaign, "_implementation_contract", return_value=IMPLEMENTATION),
                patch.object(campaign, "_data_module", return_value=data_api),
                patch.object(
                    campaign,
                    "_configure_cpu",
                    return_value=torch.device("cpu"),
                ),
                patch.object(
                    campaign,
                    "_mechanics_preflight",
                    return_value={"passed": True},
                ),
                patch.object(
                    campaign,
                    "_spawned_probe_runner",
                    side_effect=runner,
                ),
            ):
                result = campaign.preflight_position_kv_binding_campaign(root)
        self.assertTrue(result["passed"])
        self.assertEqual(result["preparation_cid"], preparation["preparation_cid"])
        self.assertEqual(data_api.events, [])
        self.assertEqual(result["terminal_payload_reads"], 0)

    def test_fit_is_exactly_once_and_never_reveals_terminal_data(self) -> None:
        manifest = {"schema": "test-data", "manifest_cid": "manifest"}
        construction = SimpleNamespace(manifest=manifest)
        data_api = _FakeDataAPI(construction)
        plan = campaign.ELIGIBLE_PLANS[0]
        fake_model = _FakeModel()
        train_calls: list[int] = []

        def train_step(
            model: _FakeModel, _optimizer: object, _batch: object, *, step: int
        ) -> dict:
            train_calls.append(step)
            if step == 2:
                model.artifact = b"fitted"
            return {
                "total": 1.0,
                "natural": 1.0,
                "mqar": 1.0,
                "english": 1.0,
                "gradient_norm_before_clip": 1.0,
                "learning_rate": 1.0e-4,
                "construction_top1": {
                    "natural": {"decisions": 960, "top1_correct": 1},
                    "mqar": {"decisions": 32, "top1_correct": 1},
                    "english": {"decisions": 4, "top1_correct": 1},
                },
                "audits": {
                    "natural": {"target_reads": 960},
                    "mqar": {"target_reads": 32},
                    "english": {
                        "target_reads": 4,
                        "provider_calls": 0,
                        "teacher_calls": 0,
                        "future_reads": 0,
                        "forbidden_reads": 0,
                    },
                },
            }

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation = _preparation(root, manifest)
            preflight = _phase_file(
                root,
                campaign.PREFLIGHT_RELATIVE_PATH,
                {
                    "schema": campaign.PREFLIGHT_SCHEMA,
                    "preparation_cid": preparation["preparation_cid"],
                    "data_manifest_cid": preparation["data_manifest_cid"],
                    "implementation": IMPLEMENTATION,
                    "passed": True,
                    "selection": {
                        "selected_plan": plan.identity(),
                        "selected_projection": {
                            "training_seconds": 100.0,
                            "scoring_seconds": 100.0,
                            "total_seconds": 200.0,
                        },
                    },
                },
                "preflight_cid",
            )
            del preflight
            (root / campaign.INPUT_INITIAL_ARTIFACT).parent.mkdir(
                parents=True, exist_ok=True
            )
            (root / campaign.INPUT_INITIAL_ARTIFACT).write_bytes(b"init")
            with (
                patch.object(campaign, "OPTIMIZER_STEPS", 2),
                patch.object(campaign, "NATURAL_CONSTRUCTION_ROWS", 16),
                patch.object(campaign, "MQAR_CONSTRUCTION_ROWS", 8),
                patch.object(campaign, "ENGLISH_HISTORY_CONSTRUCTION_ROWS", 6),
                patch.object(campaign, "ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS", 2),
                patch.object(campaign, "_implementation_contract", return_value=IMPLEMENTATION),
                patch.object(campaign, "_data_module", return_value=data_api),
                patch.object(campaign, "_construction_parts", return_value=((), (), (), ())),
                patch.object(campaign, "_mixed_batch", return_value=object()),
                patch.object(campaign, "_configure_cpu", return_value=torch.device("cpu")),
                patch.object(campaign, "_build_model", return_value=fake_model),
                patch.object(campaign, "_optimizer", return_value=object()) as optimizer,
                patch.object(campaign, "_train_step", side_effect=train_step),
            ):
                fit = campaign.fit_position_kv_binding_campaign(root)
                cached = campaign.fit_position_kv_binding_campaign(root)
            self.assertEqual(fit, cached)
            self.assertEqual(fit["completed_steps"], 2)
            self.assertEqual(fit["artifact"]["cid"], campaign.cid_file(root / campaign.ARTIFACT_RELATIVE_PATH))
            self.assertFalse((root / campaign.REVEAL_RELATIVE_PATH).exists())
        self.assertEqual(train_calls, [1, 2])
        optimizer.assert_called_once()
        self.assertEqual(data_api.events, [])

    def test_terminal_binding_and_controls_execute_through_r4(self) -> None:
        calls: list[tuple[str, str]] = []
        score = _score(1, 1)
        terminal = SimpleNamespace(
            natural_windows=(),
            mqar=(),
            mqar_binding_permuted=(),
            english_history=(),
            english_binding_permuted=(),
            english_no_history=(),
        )
        data_api = _FakeDataAPI(SimpleNamespace(), terminal=terminal)

        def score_examples(*_args: object, **kwargs: object) -> dict:
            calls.append(
                (
                    str(kwargs.get("execution", "plain")),
                    str(kwargs.get("intervention", "native")),
                )
            )
            return score

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _phase_file(
                root,
                campaign.FIT_RELATIVE_PATH,
                {"schema": campaign.FIT_SCHEMA, "elapsed_seconds": 0.0},
                "fit_cid",
            )
            artifact = root / campaign.ARTIFACT_RELATIVE_PATH
            artifact.parent.mkdir(parents=True)
            artifact.write_bytes(b"fitted")
            with (
                patch.object(campaign, "_data_module", return_value=data_api),
                patch.object(
                    campaign,
                    "_configure_cpu",
                    return_value=torch.device("cpu"),
                ),
                patch.object(campaign, "_load_fitted_model", return_value=_FakeModel()),
                patch.object(campaign, "_build_model", return_value=_FakeModel()),
                patch.object(campaign, "_world_assignments", return_value={}),
                patch.object(campaign, "_score_examples", side_effect=score_examples),
                patch.object(campaign, "_score_natural", return_value=score),
                patch.object(campaign, "_parity_natural", return_value={
                    "decisions": 1,
                    "r4_plain_attention_weight_max_delta": 0.0,
                    "r4_plain_logit_max_delta": 0.0,
                    "r4_plain_top1_identical": True,
                    "full_incremental_logit_max_delta": 0.0,
                    "full_incremental_top1_identical": True,
                    "work": {},
                }),
                patch.object(campaign, "_parity_examples", return_value={
                    "decisions": 1,
                    "r4_plain_attention_weight_max_delta": 0.0,
                    "r4_plain_logit_max_delta": 0.0,
                    "r4_plain_top1_identical": True,
                    "full_incremental_logit_max_delta": 0.0,
                    "full_incremental_top1_identical": True,
                    "work": {},
                }),
                patch.object(campaign, "_artifact_replay", return_value={
                    "passed": True,
                    "work": {},
                }),
                patch.object(campaign, "_validate_terminal_metrics"),
            ):
                campaign._default_scoring_runner(
                    root,
                    artifact,
                    terminal=terminal,
                    plan=campaign.ELIGIBLE_PLANS[0],
                    deadline=float("inf"),
                )
        self.assertEqual(len(calls), 8)
        self.assertTrue(all(execution == "r4" for execution, _ in calls))
        self.assertEqual(
            [intervention for _, intervention in calls],
            [
                "native",
                "current_only",
                "value_permuted",
                "native",
                "transport_mismatch",
                "native",
                "native",
                "native",
            ],
        )

    def test_finalize_reveals_only_after_artifact_and_is_create_once(self) -> None:
        manifest = {"schema": "test-data", "manifest_cid": "manifest"}
        metrics = _passing_metrics()
        events: list[str] = []
        plan = campaign.ELIGIBLE_PLANS[0]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation = _preparation(root, manifest)
            preflight = _phase_file(
                root,
                campaign.PREFLIGHT_RELATIVE_PATH,
                {
                    "schema": campaign.PREFLIGHT_SCHEMA,
                    "preparation_cid": preparation["preparation_cid"],
                    "data_manifest_cid": preparation["data_manifest_cid"],
                    "implementation": IMPLEMENTATION,
                    "passed": True,
                    "selection": {
                        "selected_plan": plan.identity(),
                    },
                },
                "preflight_cid",
            )
            artifact_path = root / campaign.ARTIFACT_RELATIVE_PATH
            artifact_path.parent.mkdir(parents=True)
            artifact_path.write_bytes(b"fitted")
            artifact = {
                "path": campaign.ARTIFACT_RELATIVE_PATH,
                "bytes": artifact_path.stat().st_size,
                "cid": campaign.cid_file(artifact_path),
            }
            fit = _phase_file(
                root,
                campaign.FIT_RELATIVE_PATH,
                {
                    "schema": campaign.FIT_SCHEMA,
                    "preparation_cid": preparation["preparation_cid"],
                    "preflight_cid": preflight["preflight_cid"],
                    "implementation": IMPLEMENTATION,
                    "plan": plan.identity(),
                    "completed_steps": campaign.OPTIMIZER_STEPS,
                    "optimizer_steps_after_reveal": 0,
                    "artifact": artifact,
                    "presentations": {},
                    "work": {},
                    "loss_trace_cid": "blake3:" + "4" * 64,
                },
                "fit_cid",
            )
            reveal = campaign._with_cid(
                {
                    "schema": "uor-r4.position-kv-binding-data-reveal/1",
                    "final_artifact_cid": artifact["cid"],
                    "fit_cid": fit["fit_cid"],
                    "reveal_count": 1,
                },
                "reveal_cid",
            )
            terminal = SimpleNamespace(
                reveal=reveal, final_artifact_cid=artifact["cid"]
            )
            data_api = _FakeDataAPI(SimpleNamespace(), terminal=terminal)

            def score(
                _root: Path,
                path: Path,
                **_kwargs: object,
            ) -> dict:
                self.assertTrue(path.is_file())
                self.assertTrue((root / campaign.REVEAL_RELATIVE_PATH).is_file())
                events.append("score")
                return metrics

            with (
                patch.object(campaign, "_implementation_contract", return_value=IMPLEMENTATION),
                patch.object(campaign, "_data_module", return_value=data_api),
                patch.object(campaign, "_default_scoring_runner", side_effect=score),
            ):
                result = campaign.finalize_position_kv_binding_campaign(root)
                cached = campaign.finalize_position_kv_binding_campaign(root)
        self.assertEqual(result, cached)
        self.assertEqual(result["verdict"], campaign.TERMINAL_PASS)
        self.assertEqual(data_api.events, ["reveal"])
        self.assertEqual(events, ["score"])
        self.assertEqual(result["optimizer_steps_after_reveal"], 0)


if __name__ == "__main__":
    unittest.main()
