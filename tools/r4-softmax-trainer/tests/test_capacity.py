"""Focused frozen-contract tests for the #1019 capacity lifecycle."""

from __future__ import annotations

import json
import math
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import numpy as np
import torch
from blake3 import blake3

from r4_softmax_trainer.capacity import (
    CAPACITY_CHECKPOINT_MANIFEST_SCHEMA,
    CAPACITY_ELAPSED_LEDGER_SCHEMA,
    CAPACITY_HARDWARE_SCHEMA,
    ELAPSED_LEDGER_RELATIVE_PATH,
    MAXIMUM_BEST_CHECKPOINT_SAVES,
    OPTIMIZER_STEPS,
    PARAMETER_COUNT,
    PREFIX_LOGIT_ABS_TOLERANCE,
    PYTHON_PREFIX_SCHEMA,
    RUST_QUALIFICATION_SCHEMA,
    TOKENS_PER_OPTIMIZER_STEP,
    TRAIN_TOKENS,
    CapacityTrainConfig,
    R4_POLICY_IDENTITY,
    _capacity_decision_cid,
    _load_checkpoint,
    _load_elapsed_ledger,
    _qualification_output_cid,
    _runtime_environment_identity,
    _validate_backend_identity,
    _validate_capacity_hardware_evidence,
    _validate_development_candidates,
    _validate_rust_report,
    _write_signed,
    _write_training_unavailable,
    _write_python_prefix,
    capacity_learning_rate,
    hardware_checkpoint_relative_path,
    hardware_elapsed_sample_relative_path,
    hardware_result_relative_path,
    load_capacity_hardware_admission,
    train_capacity,
)
from r4_softmax_trainer.capacity_data import (
    PREDECESSOR_DATASET_MANIFEST_CID,
    PREDECESSOR_SPLIT_POLICY_CID,
    PREVIOUS_PROMPT_CIDS,
    _prior_boundaries,
    prepare_capacity_dataset,
)
from r4_softmax_trainer import capacity_data, cli
from r4_softmax_trainer.cli import parser
from r4_softmax_trainer.constants import CAPACITY_MODEL_CONFIG
from r4_softmax_trainer.model import expected_parameter_count
from r4_softmax_trainer.provenance import (
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
)
from r4_softmax_trainer.train import TokenStore


def _valid_capacity_dataset_envelope() -> dict[str, object]:
    split_policy = {
        "bucket": "big-endian integer(full 32-byte digest) mod 100",
        "dev": "90..94",
        "digest": "BLAKE3",
        "input": "canonical story bytes before UTF-8 decoding or tokenization",
        "test": "95..99",
        "train": "0..89",
    }
    artifact_paths = {
        capacity_data.TOKENIZER_RELATIVE_PATH,
        *capacity_data.TOKEN_RELATIVE_PATHS.values(),
        *capacity_data.INDEX_RELATIVE_PATHS.values(),
        capacity_data.SEALED_PROMPT_RELATIVE_PATH,
    }
    predecessor_ordinals = {
        "train": None,
        "dev": capacity_data.PREDECESSOR_DEV_LAST_SOURCE_STORY_ORDINAL,
        "test": capacity_data.PREDECESSOR_TEST_LAST_SOURCE_STORY_ORDINAL,
    }
    splits: dict[str, dict[str, object]] = {}
    for name, token_cap in capacity_data.TOKEN_CAPS.items():
        prior = predecessor_ordinals[name]
        first = 0 if prior is None else prior + 1
        splits[name] = {
            "tokens": token_cap,
            "token_cap": token_cap,
            "stories": 1,
            "first_source_story_ordinal": first,
            "last_source_story_ordinal": first,
            "predecessor_last_source_story_ordinal": prior,
            "source_ordinal_disjoint": True,
            "complete_context_scored_next_tokens": (
                (token_cap - 1) // CAPACITY_MODEL_CONFIG.max_position_embeddings
            )
            * CAPACITY_MODEL_CONFIG.max_position_embeddings,
        }
    return {
        "schema": capacity_data.CAPACITY_DATASET_MANIFEST_SCHEMA,
        "issue": 1019,
        "manifest_cid": "blake3:" + "a" * 64,
        "predecessor": {
            "issue": 1017,
            "dataset_manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
            "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
            "dev_last_source_story_ordinal": (
                capacity_data.PREDECESSOR_DEV_LAST_SOURCE_STORY_ORDINAL
            ),
            "test_last_source_story_ordinal": (
                capacity_data.PREDECESSOR_TEST_LAST_SOURCE_STORY_ORDINAL
            ),
            "sealed_artifact_reads": 0,
        },
        "source": {
            "repository": capacity_data.TINYSTORIES_REPOSITORY,
            "revision": capacity_data.TINYSTORIES_REVISION,
            "filename": capacity_data.TINYSTORIES_FILENAME,
            "url": capacity_data.TINYSTORIES_URL,
            "bytes": capacity_data.TINYSTORIES_BYTES,
            "sha256": capacity_data.TINYSTORIES_SHA256,
            "stories_examined": 1,
        },
        "split_policy": split_policy,
        "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
        "model_contract": CAPACITY_MODEL_CONFIG.as_contract(),
        "tokenizer_cid": capacity_data.TOKENIZER_CID,
        "splits": splits,
        "sealed_confirmation_budget": {
            "scored_store_token_ids": capacity_data.TEST_TOKEN_CAP,
            "prompt_token_ids": capacity_data.SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_token_ids": (
                capacity_data.TEST_TOKEN_CAP + capacity_data.SEALED_PROMPT_TOKEN_COUNT
            ),
            "hard_cap": 250_000,
        },
        "freshness": {
            "training_population": "canonical train split from its beginning",
            "development_and_test": (
                "strictly after content-bound #1017 source ordinals"
            ),
            "excluded_published_prompt_cids": sorted(PREVIOUS_PROMPT_CIDS),
            "excluded_published_prompt_story_count": len(PREVIOUS_PROMPT_CIDS),
            "predecessor_sealed_paths_opened": 0,
        },
        "artifacts": [{"path": path} for path in sorted(artifact_paths)],
    }


def _valid_capacity_training_view(
    dataset: dict[str, object],
) -> dict[str, object]:
    records = {
        str(record["path"]): record
        for record in dataset["artifacts"]  # type: ignore[union-attr]
    }
    sealed_paths = [
        capacity_data.TOKEN_RELATIVE_PATHS["test"],
        capacity_data.INDEX_RELATIVE_PATHS["test"],
        capacity_data.SEALED_PROMPT_RELATIVE_PATH,
    ]
    training_paths = {
        capacity_data.TOKENIZER_RELATIVE_PATH,
        capacity_data.TOKEN_RELATIVE_PATHS["train"],
        capacity_data.TOKEN_RELATIVE_PATHS["dev"],
        capacity_data.INDEX_RELATIVE_PATHS["train"],
        capacity_data.INDEX_RELATIVE_PATHS["dev"],
        capacity_data.SEALED_DENIAL_RELATIVE_PATH,
    }
    return {
        "schema": capacity_data.CAPACITY_TRAINING_VIEW_MANIFEST_SCHEMA,
        "issue": 1019,
        "dataset_manifest_cid": dataset["manifest_cid"],
        "predecessor_dataset_manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
        "split_policy": dataset["split_policy"],
        "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
        "model_contract": CAPACITY_MODEL_CONFIG.as_contract(),
        "tokenizer_cid": capacity_data.TOKENIZER_CID,
        "sealed_confirmation_commitment": {
            "tokens": capacity_data.TEST_TOKEN_CAP,
            "prompt_tokens": capacity_data.SEALED_PROMPT_TOKEN_COUNT,
            "total_reveal_tokens": (
                capacity_data.TEST_TOKEN_CAP + capacity_data.SEALED_PROMPT_TOKEN_COUNT
            ),
            "artifacts": [records[path] for path in sealed_paths],
            "access_policy": "directory mode 000 until create-once reveal marker",
            "denial_result_cid": "blake3:" + "b" * 64,
        },
        "artifacts": [{"path": path} for path in sorted(training_paths)],
    }


def _mps_identity() -> dict[str, object]:
    return {
        "backend": "mps",
        "device_count": 1,
        "device_name": "arm64",
        "recommended_max_memory_bytes": 10_000,
        "macos": "15.6",
        "deterministic_algorithms": True,
        "dtype": "float32",
    }


def _write_valid_mps_hardware_evidence(
    root: Path,
    *,
    environment: dict[str, object],
    trainer_tree: dict[str, object],
) -> tuple[dict[str, str], dict[str, str]]:
    training_view = {
        "dataset_manifest_cid": "blake3:" + "1" * 64,
        "manifest_cid": "blake3:" + "2" * 64,
    }
    smoke = {"manifest_cid": "blake3:" + "3" * 64}
    identity = _mps_identity()
    probe_contract = {
        "schema": "uor-r4-softmax-trainer-capacity-hardware-probe-contract/1",
        "issue": 1019,
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "smoke_admission_manifest_cid": smoke["manifest_cid"],
        "trainer_implementation": trainer_tree,
        "backend": identity,
        "environment": environment,
        "mps_failure_prerequisite": None,
        "model": CAPACITY_MODEL_CONFIG.as_contract(),
        "parameter_count": PARAMETER_COUNT,
        "optimizer_steps": 200,
        "tokens_per_optimizer_step": TOKENS_PER_OPTIMIZER_STEP,
        "checkpoint_interval": 100,
        "maximum_best_checkpoint_saves": MAXIMUM_BEST_CHECKPOINT_SAVES,
    }
    probe_contract_cid = cid_bytes(canonical_json_bytes(probe_contract))
    checkpoint_path = root / hardware_checkpoint_relative_path("mps")
    checkpoint_path.parent.mkdir(parents=True, exist_ok=True)
    checkpoint_path.write_bytes(b"probe checkpoint")
    checkpoint_manifest_path = checkpoint_path.with_suffix(".pt.manifest.json")
    _write_signed(
        checkpoint_manifest_path,
        {
            "schema": CAPACITY_CHECKPOINT_MANIFEST_SCHEMA,
            "issue": 1019,
            "checkpoint_filename": checkpoint_path.name,
            "checkpoint_cid": cid_file(checkpoint_path),
            "run_contract_cid": probe_contract_cid,
            "optimizer_step": 200,
            "tokens_seen": 200 * TOKENS_PER_OPTIMIZER_STEP,
            "backend": "mps",
        },
    )
    elapsed_sample_path = root / hardware_elapsed_sample_relative_path("mps")
    _write_signed(
        elapsed_sample_path,
        {
            "schema": "uor-r4-softmax-trainer-capacity-hardware-elapsed-sample/1",
            "issue": 1019,
            "backend": "mps",
            "probe_contract_cid": probe_contract_cid,
            "optimizer_step": 200,
            "train_tokens": 200 * TOKENS_PER_OPTIMIZER_STEP,
            "elapsed_seconds": 200.0,
        },
    )
    optimizer_seconds = 200.0
    checkpoint_seconds = 1.0
    development_seconds = 10.0
    projected_optimizer = optimizer_seconds / 200 * OPTIMIZER_STEPS
    projected_development = development_seconds * (2 + OPTIMIZER_STEPS // 400)
    checkpoint_samples = [1.0, 1.0]
    projected_best_checkpoints = max(checkpoint_samples) * MAXIMUM_BEST_CHECKPOINT_SAVES
    projected = (
        projected_optimizer
        + projected_development
        + projected_best_checkpoints
        + checkpoint_seconds
    )
    _write_signed(
        root / hardware_result_relative_path("mps"),
        {
            "schema": CAPACITY_HARDWARE_SCHEMA,
            "issue": 1019,
            "terminal": "PASS_HARDWARE_ADMISSION",
            "trainer_implementation": trainer_tree,
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "smoke_admission_manifest_cid": smoke["manifest_cid"],
            "backend": identity,
            "mps_failure_prerequisite": None,
            "probe_contract": probe_contract,
            "probe_contract_cid": probe_contract_cid,
            "probe_checkpoint_path": str(hardware_checkpoint_relative_path("mps")),
            "probe_checkpoint_cid": cid_file(checkpoint_path),
            "probe_checkpoint_manifest_path": (
                str(hardware_checkpoint_relative_path("mps")) + ".manifest.json"
            ),
            "probe_checkpoint_manifest_cid": cid_file(checkpoint_manifest_path),
            "checkpoint_interval": 100,
            "checkpoint_reload_passed": True,
            "elapsed_sample_path": str(hardware_elapsed_sample_relative_path("mps")),
            "elapsed_sample_cid": cid_file(elapsed_sample_path),
            "probe_optimizer_steps": 200,
            "probe_token_presentations": 200 * TOKENS_PER_OPTIMIZER_STEP,
            "elapsed_seconds": (
                optimizer_seconds + checkpoint_seconds + development_seconds
            ),
            "optimizer_loop_seconds": optimizer_seconds,
            "optimizer_steps_per_second": 200 / optimizer_seconds,
            "checkpoint_reload_seconds": checkpoint_seconds,
            "checkpoint_save_seconds_samples": checkpoint_samples,
            "maximum_measured_checkpoint_save_seconds": max(checkpoint_samples),
            "projected_best_checkpoint_saves": MAXIMUM_BEST_CHECKPOINT_SAVES,
            "projected_best_checkpoint_seconds": projected_best_checkpoints,
            "probe_complete_dev_loss": 2.0,
            "complete_dev_evaluation_seconds": development_seconds,
            "projected_complete_dev_evaluations": 2 + OPTIMIZER_STEPS // 400,
            "projected_optimizer_and_checkpoint_seconds": projected_optimizer,
            "projected_development_evaluation_seconds": projected_development,
            "projected_training_seconds": projected,
            "safety_factor": 1.25,
            "safety_projected_training_seconds": projected * 1.25,
            "maximum_safety_projected_seconds": 28_800,
            "peak_accelerator_memory_bytes": 1_000,
            "available_accelerator_memory_bytes": 10_000,
            "peak_memory_fraction": 0.1,
            "maximum_memory_fraction": 0.80,
            "time_passed": True,
            "memory_passed": True,
            "main_run_authorized": True,
            "partial_checkpoint_interpretable": False,
        },
    )
    return training_view, smoke


class CapacityArithmeticTests(unittest.TestCase):
    def test_hardware_admission_rejects_resigned_malformed_backend_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            malformed_identity = _mps_identity()
            malformed_identity["device_count"] = 2
            _write_signed(
                root / hardware_result_relative_path("mps"),
                {
                    "schema": CAPACITY_HARDWARE_SCHEMA,
                    "elapsed_seconds": 1.0,
                    "projected_training_seconds": 1.0,
                    "safety_projected_training_seconds": 1.25,
                    "optimizer_loop_seconds": 1.0,
                    "checkpoint_reload_seconds": 1.0,
                    "complete_dev_evaluation_seconds": 1.0,
                    "probe_complete_dev_loss": 2.0,
                    "peak_accelerator_memory_bytes": 1,
                    "available_accelerator_memory_bytes": 10,
                    "peak_memory_fraction": 0.1,
                    "probe_contract": {},
                    "backend": malformed_identity,
                },
            )
            with (
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_training_view_manifest",
                    return_value={},
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_smoke_admission",
                    return_value={},
                ),
            ):
                with self.assertRaisesRegex(ValueError, "backend identity differs"):
                    load_capacity_hardware_admission(root, backend="mps")

    def test_hardware_admission_rejects_dependency_version_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            identity = _mps_identity()
            _validate_backend_identity(identity, "mps")
            environment = _runtime_environment_identity(identity, "mps")
            trainer_tree = trainer_implementation_contract()
            training_view, smoke = _write_valid_mps_hardware_evidence(
                root,
                environment=environment,
                trainer_tree=trainer_tree,
            )
            common_patches = (
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_training_view_manifest",
                    return_value=training_view,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_smoke_admission",
                    return_value=smoke,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity.trainer_implementation_contract",
                    return_value=trainer_tree,
                ),
            )
            with common_patches[0], common_patches[1], common_patches[2]:
                self.assertEqual(
                    load_capacity_hardware_admission(root, backend="mps")["terminal"],
                    "PASS_HARDWARE_ADMISSION",
                )

            drifted_dependencies = dict(environment["dependencies"])
            drifted_dependencies["torch"] = "drifted-version"
            with (
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_training_view_manifest",
                    return_value=training_view,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity.load_capacity_smoke_admission",
                    return_value=smoke,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity.trainer_implementation_contract",
                    return_value=trainer_tree,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity._dependency_versions",
                    return_value=drifted_dependencies,
                ),
            ):
                with self.assertRaisesRegex(ValueError, "does not authorize"):
                    load_capacity_hardware_admission(root, backend="mps")

    def test_archived_hardware_evidence_recomputes_projection_and_bindings(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            identity = _mps_identity()
            archived_environment = _runtime_environment_identity(identity, "mps")
            archived_environment["dependencies"]["torch"] = "archived-version"
            trainer_tree = trainer_implementation_contract()
            training_view, smoke = _write_valid_mps_hardware_evidence(
                root,
                environment=archived_environment,
                trainer_tree=trainer_tree,
            )
            result = json.loads(
                (root / hardware_result_relative_path("mps")).read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(
                _validate_capacity_hardware_evidence(
                    root,
                    backend="mps",
                    training_view=training_view,
                    smoke_admission=smoke,
                    result=result,
                    require_pass=True,
                    require_current_environment=False,
                )["terminal"],
                "PASS_HARDWARE_ADMISSION",
            )

            def resign(value: dict[str, object]) -> dict[str, object]:
                value.pop("result_cid", None)
                value["result_cid"] = cid_bytes(canonical_json_bytes(value))
                return value

            fabricated_projection = json.loads(json.dumps(result))
            fabricated_projection["projected_training_seconds"] = 1.0
            fabricated_projection["safety_projected_training_seconds"] = 1.25
            broken_checkpoint_binding = json.loads(json.dumps(result))
            broken_checkpoint_binding["probe_checkpoint_cid"] = (
                "blake3:" + "0" * 64
            )
            for label, tampered in (
                ("fabricated projection", resign(fabricated_projection)),
                ("broken checkpoint binding", resign(broken_checkpoint_binding)),
            ):
                with self.subTest(tamper=label):
                    with self.assertRaisesRegex(ValueError, "does not authorize"):
                        _validate_capacity_hardware_evidence(
                            root,
                            backend="mps",
                            training_view=training_view,
                            smoke_admission=smoke,
                            result=tampered,
                            require_pass=True,
                            require_current_environment=False,
                        )

    def test_exact_frozen_capacity_and_training_arithmetic(self) -> None:
        config = CapacityTrainConfig()
        config.validate()
        self.assertEqual(expected_parameter_count(CAPACITY_MODEL_CONFIG), PARAMETER_COUNT)
        self.assertEqual(config.tokens_per_optimizer_step, TOKENS_PER_OPTIMIZER_STEP)
        self.assertEqual(OPTIMIZER_STEPS * TOKENS_PER_OPTIMIZER_STEP, TRAIN_TOKENS)
        self.assertEqual(config.train_tokens, 275_251_200)
        self.assertEqual(config.optimizer_steps, 16_800)

    def test_schedule_and_contract_are_closed(self) -> None:
        config = CapacityTrainConfig()
        self.assertEqual(capacity_learning_rate(0, config), 0.0)
        self.assertEqual(capacity_learning_rate(100, config), 3e-4)
        self.assertTrue(
            math.isclose(
                capacity_learning_rate(OPTIMIZER_STEPS, config),
                3e-5,
                rel_tol=0.0,
                abs_tol=1e-15,
            )
        )
        with self.assertRaisesRegex(ValueError, "exact frozen"):
            CapacityTrainConfig(seed=1020).validate()
        with self.assertRaises(ValueError):
            capacity_learning_rate(OPTIMIZER_STEPS + 1, config)

    def test_terminal_wall_budget_cannot_resume(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_training_unavailable(
                root,
                run_contract_cid="blake3:" + "c" * 64,
                optimizer_step=400,
                elapsed_seconds=28_800.0,
                wall_ceiling_seconds=28_800.0,
            )
            with mock.patch(
                "r4_softmax_trainer.capacity.require_backend",
                side_effect=AssertionError("terminal resume reached MPS"),
            ):
                with self.assertRaisesRegex(RuntimeError, "resume is prohibited"):
                    train_capacity(root, backend="mps", resume=True)

    def test_checkpoint_sidecar_rejects_changed_bytes_and_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint_path = root / "latest.pt"
            original_bytes = b"frozen checkpoint bytes"
            checkpoint_path.write_bytes(original_bytes)
            run_contract_cid = "blake3:" + "c" * 64
            manifest_path = checkpoint_path.with_suffix(".pt.manifest.json")

            def write_sidecar(*, optimizer_step: int, train_tokens: int) -> None:
                _write_signed(
                    manifest_path,
                    {
                        "schema": CAPACITY_CHECKPOINT_MANIFEST_SCHEMA,
                        "issue": 1019,
                        "checkpoint_filename": checkpoint_path.name,
                        "checkpoint_cid": cid_file(checkpoint_path),
                        "run_contract_cid": run_contract_cid,
                        "optimizer_step": optimizer_step,
                        "tokens_seen": train_tokens,
                        "elapsed_seconds": 1.0,
                        "best_dev_loss": 2.0,
                        "development_candidates": [],
                        "backend": "mps",
                    },
                )

            write_sidecar(optimizer_step=0, train_tokens=0)
            checkpoint_path.write_bytes(b"changed checkpoint bytes")
            with self.assertRaisesRegex(ValueError, "manifest identity differs"):
                _load_checkpoint(
                    checkpoint_path,
                    model=object(),  # type: ignore[arg-type]
                    optimizer=None,
                    device=torch.device("cpu"),
                    backend="mps",
                    run_contract_cid=run_contract_cid,
                )

            checkpoint_path.write_bytes(original_bytes)
            write_sidecar(optimizer_step=1, train_tokens=0)
            with self.assertRaisesRegex(ValueError, "manifest identity differs"):
                _load_checkpoint(
                    checkpoint_path,
                    model=object(),  # type: ignore[arg-type]
                    optimizer=None,
                    device=torch.device("cpu"),
                    backend="mps",
                    run_contract_cid=run_contract_cid,
                )

    def test_elapsed_ledger_rejects_wall_time_regression_without_mps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run_contract_cid = "blake3:" + "d" * 64
            _write_signed(
                root / ELAPSED_LEDGER_RELATIVE_PATH,
                {
                    "schema": CAPACITY_ELAPSED_LEDGER_SCHEMA,
                    "issue": 1019,
                    "run_contract_cid": run_contract_cid,
                    "backend": "mps",
                    "optimizer_step": 400,
                    "train_tokens": 400 * TOKENS_PER_OPTIMIZER_STEP,
                    "campaign_started_unix_seconds": 1_000.0,
                    "recorded_unix_seconds": 1_100.0,
                    "elapsed_seconds": 99.0,
                    "wall_ceiling_seconds": 28_800.0,
                },
            )
            with self.assertRaisesRegex(ValueError, "ledger identity differs"):
                _load_elapsed_ledger(
                    root, run_contract_cid=run_contract_cid, backend="mps"
                )

    def test_development_candidate_lattice_rejects_wrong_step_and_loss_shape(self) -> None:
        candidates = [
            {"optimizer_step": 0, "train_tokens": 0, "development_loss": 2.0},
            {
                "optimizer_step": 399,
                "train_tokens": 400 * TOKENS_PER_OPTIMIZER_STEP,
                "development_loss": 1.9,
            },
        ]
        with self.assertRaisesRegex(ValueError, "candidate identity differs"):
            _validate_development_candidates(candidates, optimizer_step=400)

        candidates[1]["optimizer_step"] = 400
        candidates[1]["development_loss"] = {"mean": 1.9}
        with self.assertRaisesRegex(ValueError, "must be a finite number"):
            _validate_development_candidates(candidates, optimizer_step=400)


class CapacityPopulationBoundaryTests(unittest.TestCase):
    def test_dataset_envelope_binds_exact_predecessor_dev_and_test_ordinals(self) -> None:
        for split_name, predecessor_field in (
            ("dev", "dev_last_source_story_ordinal"),
            ("test", "test_last_source_story_ordinal"),
        ):
            with self.subTest(split=split_name):
                dataset = _valid_capacity_dataset_envelope()
                predecessor = dataset["predecessor"]
                splits = dataset["splits"]
                assert isinstance(predecessor, dict)
                assert isinstance(splits, dict)
                changed = int(predecessor[predecessor_field]) - 1
                predecessor[predecessor_field] = changed
                split = splits[split_name]
                assert isinstance(split, dict)
                split["predecessor_last_source_story_ordinal"] = changed
                with self.assertRaisesRegex(ValueError, "dataset identity differs"):
                    capacity_data._validate_dataset_envelope(dataset)

    def test_pure_selection_semantics_reject_malformed_dataset_and_training_view(
        self,
    ) -> None:
        dataset = _valid_capacity_dataset_envelope()
        training_view = _valid_capacity_training_view(dataset)
        capacity_data._validate_training_view_envelope(training_view, dataset)

        malformed_dataset = _valid_capacity_dataset_envelope()
        source = malformed_dataset["source"]
        assert isinstance(source, dict)
        source["revision"] = "re-signed-but-wrong-revision"
        with self.assertRaisesRegex(ValueError, "dataset identity differs"):
            capacity_data._validate_training_view_envelope(
                _valid_capacity_training_view(malformed_dataset), malformed_dataset
            )

        malformed_training_view = _valid_capacity_training_view(dataset)
        malformed_training_view["tokenizer_cid"] = "blake3:" + "0" * 64
        with self.assertRaisesRegex(ValueError, "training-view identity differs"):
            capacity_data._validate_training_view_envelope(
                malformed_training_view, dataset
            )

    def test_training_view_shared_artifact_records_must_equal_dataset_records(
        self,
    ) -> None:
        for shared_path in (
            capacity_data.TOKENIZER_RELATIVE_PATH,
            capacity_data.TOKEN_RELATIVE_PATHS["train"],
            capacity_data.TOKEN_RELATIVE_PATHS["dev"],
        ):
            with self.subTest(path=shared_path):
                dataset = _valid_capacity_dataset_envelope()
                training_view = _valid_capacity_training_view(dataset)
                record = next(
                    record
                    for record in training_view["artifacts"]
                    if record["path"] == shared_path
                )
                record["kappa"] = "blake3:" + "0" * 64
                with self.assertRaisesRegex(
                    ValueError, "training-view identity differs"
                ):
                    capacity_data._validate_training_view_envelope(
                        training_view, dataset
                    )

    def test_population_is_create_once_and_force_is_never_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root = base / "capacity"
            predecessor_root = base / "predecessor"
            with self.assertRaisesRegex(ValueError, "create-once"):
                prepare_capacity_dataset(
                    root,
                    predecessor_root=predecessor_root,
                    force=True,
                )

            (root / "selection").mkdir(parents=True)
            with self.assertRaisesRegex(FileExistsError, "partial or downstream evidence"):
                prepare_capacity_dataset(root, predecessor_root=predecessor_root)

    def test_predecessor_boundaries_are_manifest_bound(self) -> None:
        predecessor = {
            "manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
            "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
            "splits": {
                "dev": {"last_source_story_ordinal": 47_293},
                "test": {"last_source_story_ordinal": 48_856},
            },
        }
        self.assertEqual(
            _prior_boundaries(predecessor),
            {"train": -1, "dev": 47_293, "test": 48_856},
        )
        predecessor["manifest_cid"] = "blake3:" + "0" * 64
        with self.assertRaisesRegex(ValueError, "frozen predecessor"):
            _prior_boundaries(predecessor)

    def test_all_ten_published_prompt_cids_are_excluded(self) -> None:
        self.assertEqual(len(PREVIOUS_PROMPT_CIDS), 10)
        self.assertTrue(all(value.startswith("blake3:") for value in PREVIOUS_PROMPT_CIDS))


class CapacityPrefixContractTests(unittest.TestCase):
    def test_capacity_decision_cid_matches_the_rust_cross_language_fixture(self) -> None:
        def raw(value: str) -> str:
            return f"blake3:{blake3(value.encode()).hexdigest()}"

        observed = _capacity_decision_cid(
            {"qualification_passed": True},
            checkpoint={"checkpoint_tree_cid": raw("tree"), "weights_cid": raw("weights")},
            provenance={
                "export_manifest_cid": raw("export-manifest"),
                "export_tree_cid": raw("export-tree"),
                "dataset_manifest_cid": raw("dataset"),
                "training_view_manifest_cid": raw("training-view"),
                "split_policy_cid": raw("split"),
                "run_contract_cid": raw("contract"),
                "training_result_cid": raw("training-result"),
                "selected_checkpoint_cid": raw("selected-checkpoint"),
                "config_cid": raw("config"),
                "tokenizer_cid": raw("tokenizer"),
                "weights_cid": raw("weights"),
                "reveal_manifest_cid": raw("reveal-manifest"),
                "reveal_tree_cid": raw("reveal-tree"),
            },
            shape={
                "dimension": 288,
                "hidden_dimension": 768,
                "layers": 12,
                "query_heads": 6,
                "key_value_heads": 6,
                "head_size": 48,
                "vocabulary": 4096,
                "sequence_capacity": 32,
            },
            evaluation_input={
                "token_store_cid": raw("token-store"),
                "python_prefix_logits_cid": raw("prefix-logits"),
                "python_prefix_result_cid": raw("prefix-result"),
                "prefix_token_ids": [0, 1, 2],
            },
            enabled={
                "policy_cid": raw("policy"),
                "output_cid": raw("output"),
                "audit_cid": raw("audit"),
            },
            parity={
                "python_top1_token_id": 2,
                "rust_top1_token_id": 2,
                "identical_top1": True,
                "maximum_absolute_logit_delta": 0.0,
                "maximum_absolute_logit_delta_limit": 0.005,
                "maximum_absolute_logit_delta_within_limit": True,
                "passed": True,
            },
        )
        self.assertEqual(
            observed,
            "blake3:b16624326eee695d5b0a39a3cf74866873e9b4537436c5627a15f2fd65bc8463",
        )

    def test_python_prefix_has_exact_rust_deny_unknown_fields_shape(self) -> None:
        class Model:
            def __call__(self, inputs: torch.Tensor) -> SimpleNamespace:
                logits = torch.zeros((1, inputs.shape[1], 4096), dtype=torch.float32)
                logits[0, -1, 17] = 2.0
                return SimpleNamespace(logits=logits)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            store_path = root / "dev.u16"
            store_path.write_bytes((np.arange(300, dtype=np.uint16) % 4096).astype("<u2").tobytes())
            store = TokenStore(store_path)
            result = _write_python_prefix(
                root / "prefix.json",
                model=Model(),  # type: ignore[arg-type]
                store=store,
                store_path=store_path,
                device=torch.device("cpu"),
                weights_cid="blake3:" + "1" * 64,
            )
            self.assertEqual(
                set(result),
                {
                    "schema",
                    "weights_cid",
                    "token_store_cid",
                    "prefix_token_ids",
                    "maximum_absolute_logit_delta_limit",
                    "enabled",
                    "result_cid",
                },
            )
            self.assertEqual(result["schema"], PYTHON_PREFIX_SCHEMA)
            unsigned = dict(result)
            result_cid = unsigned.pop("result_cid")
            self.assertEqual(result_cid, cid_bytes(canonical_json_bytes(unsigned)))

    @staticmethod
    def _rust_report(
        root: Path,
    ) -> tuple[dict[str, object], dict[str, object], Path, dict[str, object], Path]:
        export_root = root / "export"
        export_root.mkdir()
        for name, contents in {
            "config.json": b"config\n",
            "export-manifest.json": b"manifest\n",
            "model.safetensors": b"weights\n",
            "tokenizer.json": b"tokenizer\n",
            "training-result.json": b"training\n",
        }.items():
            (export_root / name).write_bytes(contents)
        export: dict[str, object] = {
            "manifest_cid": "blake3:" + "1" * 64,
            "tree_cid": "blake3:" + "2" * 64,
            "dataset_manifest_cid": "blake3:" + "3" * 64,
            "training_view_manifest_cid": "blake3:" + "4" * 64,
            "split_policy_cid": "blake3:" + "5" * 64,
            "run_contract_cid": "blake3:" + "6" * 64,
            "training_result_cid": "blake3:" + "7" * 64,
            "selected_checkpoint_cid": "blake3:" + "8" * 64,
            "config_cid": cid_file(export_root / "config.json"),
            "tokenizer_cid": cid_file(export_root / "tokenizer.json"),
            "weights_cid": cid_file(export_root / "model.safetensors"),
        }
        logits = [0.0] * 4096
        logits[17] = 2.0
        prefix: dict[str, object] = {
            "schema": PYTHON_PREFIX_SCHEMA,
            "weights_cid": export["weights_cid"],
            "token_store_cid": "blake3:" + "9" * 64,
            "prefix_token_ids": list(range(32)),
            "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
            "enabled": {"top1_token_id": 17, "logits": logits},
        }
        prefix["result_cid"] = cid_bytes(canonical_json_bytes(prefix))
        prefix_path = root / "prefix.json"
        prefix_path.write_bytes(canonical_json_bytes(prefix))
        provenance = {
            "export_manifest_cid": export["manifest_cid"],
            "export_tree_cid": export["tree_cid"],
            "dataset_manifest_cid": export["dataset_manifest_cid"],
            "training_view_manifest_cid": export["training_view_manifest_cid"],
            "split_policy_cid": export["split_policy_cid"],
            "run_contract_cid": export["run_contract_cid"],
            "training_result_cid": export["training_result_cid"],
            "selected_checkpoint_cid": export["selected_checkpoint_cid"],
            "config_cid": export["config_cid"],
            "tokenizer_cid": export["tokenizer_cid"],
            "weights_cid": export["weights_cid"],
            "reveal_manifest_cid": None,
            "reveal_tree_cid": None,
        }
        checkpoint_files = [
            {
                "path": name,
                "bytes": (export_root / name).stat().st_size,
                "kappa": cid_file(export_root / name),
            }
            for name in (
                "config.json",
                "export-manifest.json",
                "model.safetensors",
                "tokenizer.json",
                "training-result.json",
            )
        ]
        checkpoint = {
            "model_path": str(export_root),
            "checkpoint_tree_cid": "blake3:" + "a" * 64,
            "config_cid": export["config_cid"],
            "tokenizer_cid": export["tokenizer_cid"],
            "weights_cid": export["weights_cid"],
            "weights_cid_scope": (
                "Safetensors shard bytes in the loader's canonical shard order; "
                "not the checkpoint-tree CID"
            ),
            "files": checkpoint_files,
            "tokenizer": {"tokenizer_cid": export["tokenizer_cid"]},
            "bos_token_id": 0,
            "eos_token_id": 1,
            "exact_backend": {},
        }
        shape = {
            "dimension": 288,
            "hidden_dimension": 768,
            "layers": 12,
            "query_heads": 6,
            "key_value_heads": 6,
            "head_size": 48,
            "vocabulary": 4096,
            "sequence_capacity": 32,
        }
        evaluation_input = {
            "token_store_cid": prefix["token_store_cid"],
            "python_prefix_logits_path": str(prefix_path),
            "python_prefix_logits_cid": cid_file(prefix_path),
            "python_prefix_result_cid": prefix["result_cid"],
            "prefix_token_ids": prefix["prefix_token_ids"],
            "sources_unchanged_across_execution": True,
        }
        applications = 32 * 12
        audit = {
            "sessions": 1,
            "positions_per_session": 32,
            "total_positions": 32,
            "selected_layer_count": 12,
            "all_layers_selected": True,
            "causal_audits_exact": 1,
            "projection_audits_exact": 1,
            "r4_audits_exact": 1,
            "output_policy_audits_exact": 1,
            "future_reads": 0,
            "output_policy_applications": applications,
            "enabled_applications": applications,
            "zeroed_applications": 0,
            "output_lanes": applications * 288,
            "nonzero_lanes_before_policy": 100,
            "nonzero_lanes_after_policy": 100,
            "applications_by_layer": [32] * 12,
            "state_ledger_cid": "blake3:" + "b" * 64,
        }
        policy = "causal-attention-output-enabled/1"
        enabled = {
            "attention_output_policy": policy,
            "policy_cid": cid_bytes(
                canonical_json_bytes([R4_POLICY_IDENTITY, policy, "all-decoder-layers"])
            ),
            "top1_token_id": 17,
            "output_cid": _qualification_output_cid(policy, 17, logits),
            "audit_cid": cid_bytes(canonical_json_bytes(audit)),
            "audit": audit,
        }
        parity = {
            "attention_output_policy": policy,
            "python_top1_token_id": 17,
            "rust_top1_token_id": 17,
            "identical_top1": True,
            "maximum_absolute_logit_delta": 0.0,
            "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
            "maximum_absolute_logit_delta_within_limit": True,
            "python_logits": logits,
            "rust_logits": logits,
            "passed": True,
        }
        report: dict[str, object] = {
            "schema": RUST_QUALIFICATION_SCHEMA,
            "issue": 1019,
            "decision_cid": "",
            "checkpoint": checkpoint,
            "provenance": provenance,
            "model_shape": shape,
            "evaluation_input": evaluation_input,
            "enabled": enabled,
            "enabled_prefix_parity": parity,
            "attention_off_executions": 0,
            "qualification_passed": True,
            "source_read_audit": {
                "checkpoint_tree_scans": 2,
                "checkpoint_tree_file_reads": 10,
                "tokenizer_loads": 1,
                "oracle_loads": 1,
                "local_checkpoint_forward_steps": 32,
                "provider_calls": 0,
                "ollama_calls": 0,
                "prior_trace_reads": 0,
                "tree_unchanged_across_execution": True,
            },
            "execution": {},
            "timing": {},
            "nonclaims": [],
        }
        report["decision_cid"] = _capacity_decision_cid(
            report,  # type: ignore[arg-type]
            checkpoint=checkpoint,
            provenance=provenance,
            shape=shape,
            evaluation_input=evaluation_input,
            enabled=enabled,
            parity=parity,
        )
        return report, prefix, prefix_path, export, export_root

    def test_rust_admission_requires_all_twelve_layers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report, prefix, prefix_path, export, export_root = self._rust_report(
                Path(directory)
            )
            _validate_rust_report(
                report,  # type: ignore[arg-type]
                prefix=prefix,  # type: ignore[arg-type]
                prefix_path=prefix_path,
                export=export,
                export_root=export_root,
            )
            report["enabled"]["audit"]["selected_layer_count"] = 11  # type: ignore[index]
            with self.assertRaisesRegex(ValueError, "all-12-layer"):
                _validate_rust_report(
                    report,  # type: ignore[arg-type]
                    prefix=prefix,  # type: ignore[arg-type]
                    prefix_path=prefix_path,
                    export=export,
                    export_root=export_root,
                )

    def test_skeletal_or_tampered_rust_summary_cannot_pass(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report, prefix, prefix_path, export, export_root = self._rust_report(
                Path(directory)
            )
            report["decision_cid"] = "blake3:" + "f" * 64
            with self.assertRaisesRegex(ValueError, "decision CID"):
                _validate_rust_report(
                    report,  # type: ignore[arg-type]
                    prefix=prefix,  # type: ignore[arg-type]
                    prefix_path=prefix_path,
                    export=export,
                    export_root=export_root,
                )
            skeletal = {
                "schema": RUST_QUALIFICATION_SCHEMA,
                "issue": 1019,
                "qualification_passed": True,
            }
            with self.assertRaisesRegex(ValueError, "did not pass"):
                _validate_rust_report(
                    skeletal,
                    prefix=prefix,  # type: ignore[arg-type]
                    prefix_path=prefix_path,
                    export=export,
                    export_root=export_root,
                )


class CapacityCliTests(unittest.TestCase):
    def test_prepare_capacity_dispatch_is_create_once_without_force(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "capacity"
            predecessor = Path(directory) / "predecessor"
            source = Path(directory) / "source.txt"
            expected = {"status": "PREPARED"}
            argv = [
                "r4-softmax-trainer",
                "--root",
                str(root),
                "prepare-capacity",
                "--predecessor-root",
                str(predecessor),
                "--source",
                str(source),
            ]
            with (
                mock.patch("sys.argv", argv),
                mock.patch.object(
                    cli, "prepare_capacity_dataset", return_value=expected
                ) as prepare,
                mock.patch.object(cli, "_print_result") as print_result,
            ):
                cli.main()
            prepare.assert_called_once_with(
                root.resolve(),
                predecessor_root=predecessor.resolve(),
                source=source.resolve(),
                force=False,
            )
            print_result.assert_called_once_with(expected)

    def test_capacity_commands_require_mps_backend_and_closed_inputs(self) -> None:
        train = parser().parse_args(["train-capacity", "--backend", "mps"])
        self.assertEqual(train.command, "train-capacity")
        self.assertEqual(train.backend, "mps")
        qualify = parser().parse_args(
            ["admit-capacity-parity", "--rust-qualification", "/tmp/report.json"]
        )
        self.assertEqual(qualify.command, "admit-capacity-parity")
        rubric = parser().parse_args(
            ["prepare-capacity-rubric", "--output", "/tmp/rubric.json"]
        )
        self.assertEqual(rubric.command, "prepare-capacity-rubric")
        generation_ready = parser().parse_args(
            ["verify-capacity-generation-ready"]
        )
        self.assertEqual(
            generation_ready.command, "verify-capacity-generation-ready"
        )
        with self.assertRaises(SystemExit):
            parser().parse_args(["train-capacity", "--backend", "cpu"])


if __name__ == "__main__":
    unittest.main()
