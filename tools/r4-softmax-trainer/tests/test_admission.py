"""Focused fail-closed tests for the sole #1014 campaign admission."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from r4_softmax_trainer.admission import (
    _verify_rust_qualification,
    admit_rust_smoke_qualification,
    load_main_admission,
)
from r4_softmax_trainer.constants import (
    EXPORT_MANIFEST_SCHEMA,
    FROZEN_MODEL_CONFIG,
    PREFIX_LOGIT_ABS_TOLERANCE,
    PYTHON_PREFIX_LOGITS_SCHEMA,
    SMOKE_MANIFEST_SCHEMA,
    SMOKE_SCHEMA,
    TRAINING_VIEW_MANIFEST_SCHEMA,
)
from r4_softmax_trainer.data import (
    INDEX_RELATIVE_PATHS,
    TOKENIZER_RELATIVE_PATH,
    TOKEN_RELATIVE_PATHS,
)
from r4_softmax_trainer.provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    tree_cid,
    write_bound_manifest,
)
from r4_softmax_trainer.train import TrainConfig, reveal_sealed_test, train_main


def _cid(digit: str) -> str:
    return "blake3:" + digit * 64


def _rust_gate_fixture() -> tuple[dict[str, object], dict[str, object]]:
    python_logits = [0.0] * 4096
    python_logits[7] = 1.0
    rust_logits = list(python_logits)
    prefix = {
        "weights_cid": _cid("1"),
        "token_store_cid": _cid("2"),
        "prefix_token_ids": list(range(32)),
        "result_cid": _cid("3"),
        "enabled": {"top1_token_id": 7, "logits": python_logits},
        "attention_off": {"top1_token_id": 7, "logits": python_logits},
    }
    export = {
        "manifest_cid": _cid("4"),
        "tree_cid": _cid("5"),
        "dataset_manifest_cid": _cid("6"),
        "training_view_manifest_cid": _cid("7"),
        "split_policy_cid": _cid("8"),
        "run_contract_cid": _cid("9"),
        "training_result_cid": _cid("a"),
        "selected_checkpoint_cid": _cid("1"),
        "config_cid": _cid("b"),
        "tokenizer_cid": _cid("c"),
        "weights_cid": _cid("1"),
    }
    bundle = {
        "prefix": prefix,
        "prefix_file_cid": _cid("d"),
        "export": export,
    }

    def arm(policy: str, *, off: bool) -> dict[str, object]:
        applications = 32 * 6
        return {
            "attention_output_policy": policy,
            "policy_cid": _cid("e"),
            "top1_token_id": 7,
            "output_cid": _cid("f"),
            "audit_cid": _cid("0"),
            "audit": {
                "sessions": 1,
                "positions_per_session": 32,
                "total_positions": 32,
                "selected_layer_count": 6,
                "all_layers_selected": True,
                "causal_audits_exact": 1,
                "projection_audits_exact": 1,
                "r4_audits_exact": 1,
                "output_policy_audits_exact": 1,
                "future_reads": 0,
                "output_policy_applications": applications,
                "enabled_applications": 0 if off else applications,
                "zeroed_applications": applications if off else 0,
                "output_lanes": applications * 288,
                "nonzero_lanes_before_policy": 1,
                "nonzero_lanes_after_policy": 0 if off else 1,
                "applications_by_layer": [32] * 6,
                "state_ledger_cid": _cid("1"),
            },
        }

    def parity(policy: str) -> dict[str, object]:
        return {
            "attention_output_policy": policy,
            "python_top1_token_id": 7,
            "rust_top1_token_id": 7,
            "identical_top1": True,
            "maximum_absolute_logit_delta": 0.0,
            "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
            "maximum_absolute_logit_delta_within_limit": True,
            "python_logits": python_logits,
            "rust_logits": rust_logits,
            "passed": True,
        }

    enabled_policy = "causal-attention-output-enabled/1"
    off_policy = "causal-attention-output-zero-post-wo-before-residual/1"
    report = {
        "schema": "uor-r4.r4-softmax-local-qualification/1",
        "issue": 1014,
        "decision_cid": _cid("2"),
        "checkpoint": {
            "checkpoint_tree_cid": _cid("3"),
            "config_cid": export["config_cid"],
            "tokenizer_cid": export["tokenizer_cid"],
            "weights_cid": export["weights_cid"],
        },
        "provenance": {
            "export_manifest_cid": export["manifest_cid"],
            "export_tree_cid": export["tree_cid"],
            **{
                field: export[field]
                for field in (
                    "dataset_manifest_cid",
                    "training_view_manifest_cid",
                    "split_policy_cid",
                    "run_contract_cid",
                    "training_result_cid",
                    "selected_checkpoint_cid",
                    "config_cid",
                    "tokenizer_cid",
                    "weights_cid",
                )
            },
            "reveal_manifest_cid": None,
            "reveal_tree_cid": None,
        },
        "model_shape": {
            "dimension": 288,
            "hidden_dimension": 768,
            "layers": 6,
            "query_heads": 6,
            "key_value_heads": 6,
            "head_size": 48,
            "vocabulary": 4096,
            "sequence_capacity": 32,
        },
        "evaluation_input": {
            "token_store_cid": prefix["token_store_cid"],
            "python_prefix_logits_cid": bundle["prefix_file_cid"],
            "python_prefix_result_cid": prefix["result_cid"],
            "prefix_token_ids": prefix["prefix_token_ids"],
            "sources_unchanged_across_execution": True,
        },
        "enabled": arm(enabled_policy, off=False),
        "attention_off": arm(off_policy, off=True),
        "enabled_prefix_parity": parity(enabled_policy),
        "attention_off_prefix_parity": parity(off_policy),
        "qualification_passed": True,
        "source_read_audit": {
            "provider_calls": 0,
            "ollama_calls": 0,
            "prior_trace_reads": 0,
            "tree_unchanged_across_execution": True,
        },
    }
    return report, bundle


class RustAdmissionTests(unittest.TestCase):
    def test_two_arm_rust_pass_is_accepted(self) -> None:
        report, bundle = _rust_gate_fixture()
        # Python emits its f32 tensor value through a binary64 list while Rust
        # reserializes the parsed f32 with its shortest representation.
        bundle["prefix"]["enabled"]["logits"] = list(
            bundle["prefix"]["enabled"]["logits"]
        )
        report["enabled_prefix_parity"]["python_logits"] = list(
            report["enabled_prefix_parity"]["python_logits"]
        )
        report["enabled_prefix_parity"]["rust_logits"] = list(
            report["enabled_prefix_parity"]["rust_logits"]
        )
        bundle["prefix"]["enabled"]["logits"][0] = 0.10000000149011612
        report["enabled_prefix_parity"]["python_logits"][0] = 0.1
        report["enabled_prefix_parity"]["rust_logits"][0] = 0.1
        _verify_rust_qualification(report, report_file_cid=_cid("a"), bundle=bundle)

    def test_one_failed_arm_rejects_the_campaign(self) -> None:
        report, bundle = _rust_gate_fixture()
        report["attention_off_prefix_parity"]["passed"] = False
        with self.assertRaisesRegex(ValueError, "attention_off parity did not pass"):
            _verify_rust_qualification(report, report_file_cid=_cid("a"), bundle=bundle)

    def test_tampered_rust_logits_reject_even_with_pass_booleans(self) -> None:
        report, bundle = _rust_gate_fixture()
        report["enabled_prefix_parity"]["rust_logits"][0] = 0.001
        with self.assertRaisesRegex(ValueError, "maximum logit delta does not reproduce"):
            _verify_rust_qualification(report, report_file_cid=_cid("a"), bundle=bundle)

    def test_imported_pass_is_durable_and_cid_bound(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            training_paths = [
                TOKENIZER_RELATIVE_PATH,
                TOKEN_RELATIVE_PATHS["train"],
                TOKEN_RELATIVE_PATHS["dev"],
                INDEX_RELATIVE_PATHS["train"],
                INDEX_RELATIVE_PATHS["dev"],
            ]
            for relative in training_paths:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((relative + "\n").encode())
            training_view = write_bound_manifest(
                root / "training-view-manifest.json",
                {
                    "schema": TRAINING_VIEW_MANIFEST_SCHEMA,
                    "dataset_manifest_cid": _cid("6"),
                    "split_policy_cid": _cid("8"),
                    "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
                    "sealed_test_commitment": {"status": "UNOPENED"},
                },
                artifact_root=root,
                relative_paths=training_paths,
            )
            identity = {
                "dataset_manifest_cid": training_view["dataset_manifest_cid"],
                "training_view_manifest_cid": training_view["manifest_cid"],
                "split_policy_cid": training_view["split_policy_cid"],
            }
            smoke_contract = {
                "schema": "uor-r4-softmax-trainer-smoke-contract/1",
                **identity,
                "trainer_implementation": trainer_implementation_contract(),
                "sequences": 64,
                "context": 256,
                "required_loss_reduction_fraction": 0.80,
                "wall_ceiling_seconds": 300.0,
            }
            smoke_contract_cid = cid_bytes(canonical_json_bytes(smoke_contract))
            smoke_result = {
                "schema": SMOKE_SCHEMA,
                "terminal": "PASS",
                **identity,
                "smoke_contract": smoke_contract,
                "smoke_contract_cid": smoke_contract_cid,
                "sequences": 64,
                "context": 256,
                "initial_loss": 10.0,
                "final_loss": 1.9,
                "loss_reduction_fraction": 0.81,
                "required_reduction_fraction": 0.80,
                "elapsed_seconds": 299.0,
                "wall_ceiling_seconds": 300.0,
            }
            smoke_result["result_cid"] = cid_bytes(canonical_json_bytes(smoke_result))
            atomic_write_json(root / "smoke/smoke-result.json", smoke_result)

            export_root = root / "smoke/export"
            for name, contents in {
                "config.json": b"{}\n",
                "model.safetensors": b"fake weights",
                "tokenizer.json": b"{}\n",
            }.items():
                path = export_root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(contents)
            atomic_write_json(export_root / "training-result.json", smoke_result)
            export_manifest = write_bound_manifest(
                export_root / "export-manifest.json",
                {
                    "schema": EXPORT_MANIFEST_SCHEMA,
                    **identity,
                    "run_contract_cid": smoke_contract_cid,
                    "training_result_cid": smoke_result["result_cid"],
                    "selected_checkpoint_cid": cid_file(export_root / "model.safetensors"),
                    "config_cid": cid_file(export_root / "config.json"),
                    "tokenizer_cid": cid_file(export_root / "tokenizer.json"),
                    "weights_cid": cid_file(export_root / "model.safetensors"),
                },
                artifact_root=export_root,
                relative_paths=[
                    "config.json",
                    "model.safetensors",
                    "tokenizer.json",
                    "training-result.json",
                ],
            )

            report, _ = _rust_gate_fixture()
            logits = report["enabled_prefix_parity"]["python_logits"]
            prefix = {
                "schema": PYTHON_PREFIX_LOGITS_SCHEMA,
                "weights_cid": export_manifest["weights_cid"],
                "token_store_cid": cid_file(root / TOKEN_RELATIVE_PATHS["train"]),
                "prefix_token_ids": list(range(32)),
                "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
                "enabled": {"top1_token_id": 7, "logits": logits},
                "attention_off": {"top1_token_id": 7, "logits": logits},
            }
            prefix["result_cid"] = cid_bytes(canonical_json_bytes(prefix))
            atomic_write_json(root / "smoke/python-prefix-logits.json", prefix)
            smoke_manifest = write_bound_manifest(
                root / "smoke/smoke-manifest.json",
                {
                    "schema": SMOKE_MANIFEST_SCHEMA,
                    "terminal": "PASS_EXPORT_AWAITING_RUST_PARITY",
                    **identity,
                    "smoke_contract_cid": smoke_contract_cid,
                    "smoke_result_cid": smoke_result["result_cid"],
                    "export_manifest_cid": export_manifest["manifest_cid"],
                    "weights_cid": export_manifest["weights_cid"],
                    "prefix_result_cid": prefix["result_cid"],
                },
                artifact_root=root,
                relative_paths=[
                    "smoke/smoke-result.json",
                    "smoke/python-prefix-logits.json",
                    "smoke/export/config.json",
                    "smoke/export/model.safetensors",
                    "smoke/export/tokenizer.json",
                    "smoke/export/training-result.json",
                    "smoke/export/export-manifest.json",
                ],
            )

            report["checkpoint"].update(
                {
                    "config_cid": export_manifest["config_cid"],
                    "tokenizer_cid": export_manifest["tokenizer_cid"],
                    "weights_cid": export_manifest["weights_cid"],
                }
            )
            report["provenance"].update(
                {
                    "export_manifest_cid": export_manifest["manifest_cid"],
                    "export_tree_cid": export_manifest["tree_cid"],
                    **{
                        field: export_manifest[field]
                        for field in (
                            "dataset_manifest_cid",
                            "training_view_manifest_cid",
                            "split_policy_cid",
                            "run_contract_cid",
                            "training_result_cid",
                            "selected_checkpoint_cid",
                            "config_cid",
                            "tokenizer_cid",
                            "weights_cid",
                        )
                    },
                }
            )
            report["evaluation_input"].update(
                {
                    "token_store_cid": prefix["token_store_cid"],
                    "python_prefix_logits_cid": cid_file(
                        root / "smoke/python-prefix-logits.json"
                    ),
                    "python_prefix_result_cid": prefix["result_cid"],
                    "prefix_token_ids": prefix["prefix_token_ids"],
                }
            )
            qualifier_path = root / "rust-output.json"
            qualifier_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
            admission = admit_rust_smoke_qualification(root, qualifier_path)
            self.assertEqual(admission["terminal"], "PASS_SMOKE_AND_RUST_TWO_ARM_PARITY")
            self.assertEqual(admission["smoke_manifest_cid"], smoke_manifest["manifest_cid"])
            self.assertEqual(
                load_main_admission(root, require_current_trainer=True), admission
            )

            imported = root / "admission/rust-smoke-qualification.json"
            imported.write_bytes(imported.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "artifact records do not reproduce"):
                load_main_admission(root, require_current_trainer=True)


class CampaignImmutabilityTests(unittest.TestCase):
    def test_export_payload_file_cid_mismatches_stop_before_sealed_loader(self) -> None:
        for mismatched_field in ("config_cid", "tokenizer_cid", "weights_cid"):
            with self.subTest(field=mismatched_field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                export_root = root / "export"
                export_root.mkdir(parents=True)
                (export_root / "config.json").write_bytes(b"config\n")
                (export_root / "tokenizer.json").write_bytes(b"tokenizer\n")
                (export_root / "model.safetensors").write_bytes(b"weights\n")
                (root / "checkpoints").mkdir()
                (root / "checkpoints/best.pt").write_bytes(b"checkpoint\n")

                dataset_cid = _cid("1")
                training_view_cid = _cid("2")
                split_policy_cid = _cid("3")
                admission = {
                    "manifest_cid": _cid("4"),
                    "dataset_manifest_cid": dataset_cid,
                    "training_view_manifest_cid": training_view_cid,
                    "split_policy_cid": split_policy_cid,
                    "smoke_manifest_cid": _cid("5"),
                    "smoke_export_manifest_cid": _cid("6"),
                    "rust_qualification_report_cid": _cid("7"),
                    "rust_qualification_decision_cid": _cid("8"),
                    "smoke_trainer_implementation_tree_cid": _cid("9"),
                    "trainer_implementation_tree_cid": tree_cid([]),
                    "smoke_reuse_transition_cid": _cid("a"),
                }
                admission_identity = {
                    "manifest_cid": admission["manifest_cid"],
                    "smoke_manifest_cid": admission["smoke_manifest_cid"],
                    "smoke_export_manifest_cid": admission["smoke_export_manifest_cid"],
                    "rust_qualification_report_cid": admission[
                        "rust_qualification_report_cid"
                    ],
                    "rust_qualification_decision_cid": admission[
                        "rust_qualification_decision_cid"
                    ],
                    "smoke_trainer_implementation_tree_cid": admission[
                        "smoke_trainer_implementation_tree_cid"
                    ],
                    "campaign_trainer_implementation_tree_cid": admission[
                        "trainer_implementation_tree_cid"
                    ],
                    "smoke_reuse_transition_cid": admission[
                        "smoke_reuse_transition_cid"
                    ],
                }
                run_contract = {
                    "dataset_manifest_cid": dataset_cid,
                    "training_view_manifest_cid": training_view_cid,
                    "split_policy_cid": split_policy_cid,
                    "main_campaign_admission": admission_identity,
                    "trainer_implementation": {"files": [], "tree_cid": tree_cid([])},
                }
                run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
                training_result = {
                    "schema": "uor-r4-softmax-trainer-selection-result/1",
                    "terminal": "FINAL_CHECKPOINT_FROZEN_TEST_UNOPENED",
                    "dataset_manifest_cid": dataset_cid,
                    "training_view_manifest_cid": training_view_cid,
                    "run_contract": run_contract,
                    "run_contract_cid": run_contract_cid,
                    "selected_checkpoint_step": 10,
                    "selected_dev_loss": 1.0,
                    "sealed_test_status": "UNOPENED",
                }
                training_result["result_cid"] = cid_bytes(
                    canonical_json_bytes(training_result)
                )
                atomic_write_json(root / "training-result.json", training_result)

                file_cids = {
                    "config_cid": cid_file(export_root / "config.json"),
                    "tokenizer_cid": cid_file(export_root / "tokenizer.json"),
                    "weights_cid": cid_file(export_root / "model.safetensors"),
                }
                payload_cids = dict(file_cids)
                payload_cids[mismatched_field] = _cid("f")
                export_manifest = {
                    "schema": EXPORT_MANIFEST_SCHEMA,
                    "manifest_cid": _cid("b"),
                    "artifacts": [
                        {"path": "config.json", "cid": file_cids["config_cid"]},
                        {
                            "path": "model.safetensors",
                            "cid": file_cids["weights_cid"],
                        },
                        {"path": "tokenizer.json", "cid": file_cids["tokenizer_cid"]},
                        {"path": "training-result.json", "cid": _cid("c")},
                    ],
                    "dataset_manifest_cid": dataset_cid,
                    "training_view_manifest_cid": training_view_cid,
                    "split_policy_cid": split_policy_cid,
                    "run_contract_cid": run_contract_cid,
                    "selected_checkpoint_cid": cid_file(root / "checkpoints/best.pt"),
                    "training_result_cid": training_result["result_cid"],
                    **payload_cids,
                }
                selection = {
                    "schema": "uor-r4-softmax-trainer-selection/1",
                    "manifest_cid": _cid("d"),
                    "artifacts": [
                        {"path": path}
                        for path in (
                            "admission/main-admission-manifest.json",
                            "checkpoints/best.pt",
                            "training-result.json",
                            "export/config.json",
                            "export/model.safetensors",
                            "export/tokenizer.json",
                            "export/training-result.json",
                            "export/export-manifest.json",
                        )
                    ],
                    "sealed_test_status": "UNOPENED_BEFORE_THIS_MANIFEST",
                    "selected_checkpoint_cid": cid_file(root / "checkpoints/best.pt"),
                }

                def bound_manifest(path: Path, *, artifact_root: Path):
                    del artifact_root
                    if path.name == "selection-manifest.json":
                        return selection
                    if path.name == "export-manifest.json":
                        return export_manifest
                    raise AssertionError(f"unexpected manifest read: {path}")

                with (
                    mock.patch(
                        "r4_softmax_trainer.train.verify_bound_manifest",
                        side_effect=bound_manifest,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.train.load_main_admission",
                        return_value=admission,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.train.load_dataset_manifest"
                    ) as sealed_loader,
                ):
                    with self.assertRaisesRegex(
                        ValueError,
                        rf"export {mismatched_field} does not match verified",
                    ):
                        reveal_sealed_test(root)
                sealed_loader.assert_not_called()

    def test_main_requires_admission_before_mps_or_writes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            training_view = {
                "dataset_manifest_cid": _cid("1"),
                "manifest_cid": _cid("2"),
                "split_policy_cid": _cid("3"),
            }
            with (
                mock.patch(
                    "r4_softmax_trainer.train.load_training_view_manifest",
                    return_value=training_view,
                ),
                mock.patch(
                    "r4_softmax_trainer.train.load_main_admission",
                    side_effect=ValueError("no admitted Rust parity"),
                ),
                mock.patch("r4_softmax_trainer.train.require_mps") as require_mps,
            ):
                with self.assertRaisesRegex(ValueError, "no admitted Rust parity"):
                    train_main(root, config=TrainConfig())
            require_mps.assert_not_called()
            self.assertEqual(list(root.iterdir()), [])

    def test_resume_cannot_mutate_a_strictly_verified_selection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selection_path = root / "selection" / "selection-manifest.json"
            selection_path.parent.mkdir(parents=True)
            frozen_bytes = b"already frozen\n"
            selection_path.write_bytes(frozen_bytes)
            with (
                mock.patch(
                    "r4_softmax_trainer.train._load_frozen_selection",
                    return_value={"manifest_cid": _cid("1")},
                ) as verify_selection,
                mock.patch(
                    "r4_softmax_trainer.train.load_training_view_manifest"
                ) as load_training_view,
                mock.patch("r4_softmax_trainer.train.require_mps") as require_mps,
            ):
                with self.assertRaisesRegex(FileExistsError, "CID-frozen"):
                    train_main(root, config=TrainConfig(), resume=True)
            verify_selection.assert_called_once_with(root)
            load_training_view.assert_not_called()
            require_mps.assert_not_called()
            self.assertEqual(selection_path.read_bytes(), frozen_bytes)


if __name__ == "__main__":
    unittest.main()
