"""Focused one-shot, provenance, schedule, and CLI checks for C1-SB5."""

from __future__ import annotations

from copy import deepcopy
from contextlib import ExitStack, contextmanager
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch

from r4_softmax_trainer import cli
from r4_softmax_trainer import paired_query_binding_campaign as campaign
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes, tree_cid


def _with_cid(value: dict[str, object], field: str) -> dict[str, object]:
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _pairs(*, records_per_width: int) -> list[dict[str, object]]:
    return [
        {
            "record_cid": cid_bytes(f"pair-{width}-{slot}".encode()),
            "source_width": width,
            "queries": [{"target_outcome": "answer"}, {"target_outcome": "abstain"}],
            "candidate_groups": [{"relation_group_cid": cid_bytes(b"group")}],
            "label_matrix": [[1], [0]],
        }
        for width in campaign.WIDTHS
        for slot in range(records_per_width)
    ]


def _fake_population() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    census = _with_cid(
        {
            "schema": campaign.CENSUS_SCHEMA,
            "policy": campaign.POLICY,
            "passed": True,
        },
        "census_cid",
    )
    tokenizer_census = _with_cid(
        {
            "schema": campaign.TOKENIZER_CENSUS_SCHEMA,
            "policy": campaign.POLICY,
            "tokenizer_cid": campaign.EXPECTED_TOKENIZER_CID,
            "partitions": {
                "fit": {"pairs": campaign.FIT_PAIRS, "passed": True},
                "sealed": {"pairs": campaign.SEALED_PAIRS, "passed": True},
                "product": {"pairs": 4, "passed": True},
            },
            "passed": True,
        },
        "tokenizer_census_cid",
    )
    preflight = _with_cid(
        {
            "schema": campaign.PREFLIGHT_SCHEMA,
            "policy": campaign.POLICY,
            "fit": _pairs(records_per_width=8),
            "sealed": _pairs(records_per_width=4),
            "census_cid": census["census_cid"],
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        },
        "preflight_cid",
    )
    commitments = [cid_bytes(f"product-{index}".encode()) for index in range(4)]
    products = _with_cid(
        {
            "schema": "uor-r4.paired-query-binding-products/1",
            "policy": campaign.POLICY,
            "records": [{"record_cid": commitment} for commitment in commitments],
        },
        "product_probes_cid",
    )
    split_policy_cid = cid_bytes(b"paired-query-split")
    dataset = _with_cid(
        {
            "schema": campaign.DATASET_SCHEMA,
            "policy": campaign.POLICY,
            "split_policy_cid": split_policy_cid,
            "census": census,
            "census_cid": census["census_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "tokenizer_census": tokenizer_census,
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "product_probes_cid": products["product_probes_cid"],
            "product_probe_commitments": commitments,
        },
        "dataset_cid",
    )
    return dataset, preflight, products


@contextmanager
def _patched_freeze(
    dataset: dict[str, object],
    preflight: dict[str, object],
    products: dict[str, object],
):
    with ExitStack() as stack:
        values = {
            "EXPECTED_DATASET_CID": dataset["dataset_cid"],
            "EXPECTED_PREFLIGHT_CID": preflight["preflight_cid"],
            "EXPECTED_CENSUS_CID": dataset["census_cid"],
            "EXPECTED_TOKENIZER_CENSUS_CID": dataset["tokenizer_census_cid"],
            "EXPECTED_PRODUCT_CID": products["product_probes_cid"],
            "EXPECTED_SPLIT_POLICY_CID": dataset["split_policy_cid"],
            "EXPECTED_PRODUCT_COMMITMENTS": tuple(
                dataset["product_probe_commitments"]  # type: ignore[arg-type]
            ),
            "EXPECTED_PREDECESSOR_MANIFEST_CID": cid_bytes(
                b"predecessor-manifest"
            ),
        }
        for name, value in values.items():
            stack.enter_context(patch.object(campaign, name, value))
        yield


def _metrics(
    *, pairs: int, exact: bool, mean_loss: float, identical_rows: bool = False
) -> dict[str, object]:
    if pairs == campaign.FIT_PAIRS:
        cells, flips, copies, duplicates = 532, 98, 42, 14
        outcomes = {"answer": 42, "abstain": 42, "conflict": 28}
    else:
        cells, flips, copies, duplicates = 266, 49, 21, 7
        outcomes = {"answer": 21, "abstain": 21, "conflict": 14}
    pair_correct = pairs if exact else 0
    pair_evaluations = []
    for index in range(pairs):
        group_cid = cid_bytes(f"group-{index}".encode())
        answer_supported = exact
        pair_evaluations.append(
            {
                "record_id": f"record-{index:02d}",
                "source_width": campaign.WIDTHS[index % len(campaign.WIDTHS)],
                "pair_slot": index % 4,
                "candidate_state_identity": True,
                "query_rows": [
                    {
                        "question": f"Where is subject-a-{index}?",
                        "target_outcome": "answer",
                        "predicted_outcome": "answer" if exact else "abstain",
                        "outcome_exact": exact,
                        "target_copy_candidate_index": 0,
                        "predicted_copy_candidate_index": 0 if exact else None,
                        "copy_exact": exact,
                        "positive_group_indices": [0],
                        "predicted_positive_group_indices": [0] if exact else [],
                        "cells_exact": exact,
                        "cells": [
                            {
                                "relation_group_cid": group_cid,
                                "label": 1,
                                "score": 2.0 if answer_supported else -2.0,
                                "supported": answer_supported,
                            }
                        ],
                    },
                    {
                        "question": f"Where is subject-b-{index}?",
                        "target_outcome": "abstain",
                        "predicted_outcome": "abstain",
                        "outcome_exact": True,
                        "target_copy_candidate_index": None,
                        "predicted_copy_candidate_index": None,
                        "copy_exact": True,
                        "positive_group_indices": [],
                        "predicted_positive_group_indices": [],
                        "cells_exact": True,
                        "cells": [
                            {
                                "relation_group_cid": group_cid,
                                "label": 0,
                                "score": -2.0,
                                "supported": False,
                            }
                        ],
                    },
                ],
                "flip_columns": [
                    {"relation_group_cid": group_cid, "exact": exact}
                ],
                "flip_exact": exact,
                "paired_rows_identical": identical_rows,
                "pair_exact": exact,
            }
        )
    return {
        "pairs": pairs,
        "query_rows": 2 * pairs,
        "matrix_cells": cells,
        "flip_columns": flips,
        "mean_row_margin": mean_loss / 2.0,
        "mean_flip_margin": mean_loss / 2.0,
        "mean_total_loss": mean_loss,
        "pair_exact": {"correct": pair_correct, "total": pairs},
        "row_exact": {"correct": 2 * pairs if exact else 0, "total": 2 * pairs},
        "cell_exact": {"correct": cells if exact else 0, "total": cells},
        "flip_exact": {"correct": flips if exact else 0, "total": flips},
        "candidate_copy_exact": {
            "correct": copies if exact else 0,
            "total": copies,
        },
        "duplicate_pair_exact": {
            "correct": duplicates if exact else 0,
            "total": duplicates,
        },
        "outcome": {
            name: {"correct": total if exact else 0, "total": total}
            for name, total in outcomes.items()
        },
        "candidate_state_bit_identity": {"correct": pairs, "total": pairs},
        "paired_rows_identical": {
            "correct": pairs if identical_rows else 0,
            "total": pairs,
        },
        "mean_loss": mean_loss,
        "attention_off": False,
        "mean_query_ablation": False,
        "row_swap": False,
        "candidate_state_identity_exact": True,
        "pair_evaluations": pair_evaluations,
    }


def _swapped_metrics(metrics: dict[str, object]) -> dict[str, object]:
    swapped = deepcopy(metrics)
    swapped["row_swap"] = True
    for pair in swapped["pair_evaluations"]:  # type: ignore[union-attr]
        pair["query_rows"] = list(reversed(pair["query_rows"]))
    return swapped


def _positive_delta() -> dict[str, object]:
    return {
        "target_tensor_count": 24,
        "changed_target_tensor_count": 24,
        "changed_target_tensors": [f"target-{index}" for index in range(24)],
        "changed_nontarget_tensors": [],
        "all_target_tensors_finite": True,
        "binding_head": {
            "tensor_count": 3,
            "changed_tensor_count": 3,
            "all_finite": True,
        },
        "passed": True,
    }


class _ScheduleDataset:
    def __init__(self, _records: object = None) -> None:
        group_counts = [5] * 42 + [4] * 14
        self.records = [
            SimpleNamespace(candidate_groups=[object()] * count)
            for count in group_counts
        ]

    def validate_fit_schedule(self) -> None:
        if len(self.records) != 56:
            raise ValueError("bad fixture schedule")

    def record_indices_for_step(self, step: int) -> tuple[int, ...]:
        base = ((step - 1) % campaign.STEPS_PER_EPOCH) * campaign.PAIRS_PER_STEP
        return tuple(range(base, base + campaign.PAIRS_PER_STEP))


class PairedQueryBindingCampaignTests(unittest.TestCase):
    def test_cli_registers_prepare_and_one_shot_train_commands(self) -> None:
        parser = cli.parser()
        prepare = parser.parse_args(["prepare-paired-query-binding"])
        train = parser.parse_args(["train-paired-query-binding-preflight"])
        self.assertEqual(prepare.command, "prepare-paired-query-binding")
        self.assertEqual(train.command, "train-paired-query-binding-preflight")
        help_text = parser.format_help()
        self.assertIn("optimizer under its 300-second", help_text)
        self.assertIn("ceiling, then the mandatory controls", help_text)
        self.assertNotIn("<=5 minute C1-SB5", help_text)
        self.assertEqual(
            campaign.CELL_PRESENTATIONS,
            campaign.EPOCHS * 532,
        )

    def test_one_shot_marker_is_exclusive_and_partial_marker_fails_closed(self) -> None:
        payload = {
            "schema": campaign.RUN_SCHEMA,
            "phase": "SOLE_C1_SB5_PREFLIGHT_STARTED",
            "run_contract_cid": cid_bytes(b"run-contract"),
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            marker = root / campaign.STARTED_FILE
            campaign._write_exclusive_started_marker(marker, payload)
            committed = marker.read_bytes()
            with self.assertRaisesRegex(FileExistsError, "already started"):
                campaign._write_exclusive_started_marker(marker, payload)
            self.assertEqual(marker.read_bytes(), committed)

            partial = root / "partial-started.json"
            partial.write_bytes(b"{")
            with self.assertRaisesRegex(FileExistsError, "already started"):
                campaign._write_exclusive_started_marker(partial, payload)
            self.assertEqual(partial.read_bytes(), b"{")

    def test_schedule_arithmetic_is_exact_and_drift_fails_closed(self) -> None:
        dataset = _ScheduleDataset()
        observation = campaign._schedule_observation(
            {"optimizer_steps": 120, "paired_records_per_step": 7}, dataset
        )
        self.assertEqual(observation["pair_presentations"], 840)
        self.assertEqual(observation["row_presentations"], 1_680)
        self.assertEqual(observation["cell_presentations"], 7_980)
        with self.assertRaisesRegex(ValueError, "frozen schedule"):
            campaign._schedule_observation(
                {"optimizer_steps": 119, "paired_records_per_step": 7}, dataset
            )

    def test_semantic_gate_binds_every_frozen_denominator(self) -> None:
        sealed = _metrics(pairs=28, exact=True, mean_loss=0.1)
        self.assertTrue(campaign._main_metrics_exact(sealed, expected_pairs=28))
        sealed["matrix_cells"] = 265
        with self.assertRaisesRegex(ValueError, "denominator drifted"):
            campaign._main_metrics_exact(sealed, expected_pairs=28)

    def test_population_failure_is_typed_and_leaves_no_partial_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            predecessor_manifest = {
                "manifest_cid": cid_bytes(b"predecessor-manifest"),
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            with (
                patch.object(
                    campaign,
                    "_validated_predecessor",
                    return_value=predecessor_manifest,
                ),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(campaign, "validate_tokenizer_contract"),
                patch.object(
                    campaign,
                    "build_paired_query_binding_population",
                    side_effect=ValueError("tokenizer census failed"),
                ),
                patch.object(campaign, "atomic_write_json") as write_json,
                patch.object(campaign, "write_bound_manifest") as write_manifest,
            ):
                result = campaign.prepare_paired_query_binding_data(
                    root, predecessor=predecessor
                )
            self.assertEqual(result["terminal"], "UNAVAILABLE_FRAME_OR_POPULATION")
            self.assertEqual(result["optimizer_steps"], 0)
            self.assertEqual(result["training"], "NOT_STARTED")
            self.assertEqual(result["artifacts"], "NOT_EMITTED")
            self.assertFalse(root.exists())
            write_json.assert_not_called()
            write_manifest.assert_not_called()

    def test_row_swap_requires_bit_exact_identity_aligned_trace(self) -> None:
        sealed = _metrics(pairs=28, exact=True, mean_loss=0.1)
        row_swap = _swapped_metrics(sealed)
        mean_query = _metrics(
            pairs=28, exact=False, mean_loss=1.0, identical_rows=True
        )
        mean_query["mean_query_ablation"] = True
        attention_off = _metrics(pairs=28, exact=False, mean_loss=1.1)
        attention_off["attention_off"] = True

        exact_gate = campaign._control_gate(
            sealed, row_swap, mean_query, attention_off
        )
        self.assertTrue(exact_gate["passed"])
        self.assertTrue(exact_gate["row_swap_equivariance"]["passed"])

        mutated = deepcopy(row_swap)
        # The raw swapped trace keeps query B in row zero and query A in row one.
        # Move query A's positive score without crossing zero: aggregate semantic
        # accuracy is unchanged, but exact row-swap equivariance must now fail.
        mutated["pair_evaluations"][0]["query_rows"][1]["cells"][0][  # type: ignore[index]
            "score"
        ] = 2.25
        self.assertTrue(
            campaign._main_metrics_exact(mutated, expected_pairs=campaign.SEALED_PAIRS)
        )
        mutated_gate = campaign._control_gate(
            sealed, mutated, mean_query, attention_off
        )
        self.assertFalse(mutated_gate["row_swap_exact"])
        self.assertFalse(mutated_gate["row_swap_equivariance"]["pair_trace_bit_exact"])
        self.assertFalse(mutated_gate["passed"])

    def test_training_view_rejects_product_path_before_binary_open(self) -> None:
        dataset, preflight, products = _fake_population()
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            predecessor_manifest = {
                "manifest_cid": cid_bytes(b"predecessor-manifest"),
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            with _patched_freeze(dataset, preflight, products):
                with (
                    patch.object(
                        campaign,
                        "_validated_predecessor",
                        return_value=predecessor_manifest,
                    ),
                    patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                    patch.object(campaign, "validate_tokenizer_contract"),
                    patch.object(
                        campaign,
                        "build_paired_query_binding_population",
                        return_value=(dataset, preflight, products),
                    ),
                ):
                    campaign.prepare_paired_query_binding_data(
                        root, predecessor=predecessor
                    )

            manifest_path = root / campaign.TRAINING_MANIFEST_FILE
            manifest = campaign.verify_manifest_envelope(manifest_path)
            product_path = root / campaign.PRODUCT_FILE
            manifest["artifacts"].append(
                {
                    "bytes": product_path.stat().st_size,
                    "cid": campaign.cid_file(product_path),
                    "path": campaign.PRODUCT_FILE,
                }
            )
            manifest["artifacts"].sort(key=lambda record: str(record["path"]))
            manifest["tree_cid"] = tree_cid(manifest["artifacts"])
            unsigned = dict(manifest)
            unsigned.pop("manifest_cid")
            manifest["manifest_cid"] = cid_bytes(canonical_json_bytes(unsigned))
            campaign.atomic_write_json(manifest_path, manifest)

            original_open = Path.open

            def guarded_open(
                path: Path, mode: str = "r", *args: object, **kwargs: object
            ) -> object:
                if path.name == campaign.PRODUCT_FILE and mode == "rb":
                    raise AssertionError("trainer opened product bytes before rejection")
                return original_open(path, mode, *args, **kwargs)

            with (
                patch.object(Path, "open", guarded_open),
                self.assertRaisesRegex(ValueError, "artifact whitelist drifted"),
            ):
                campaign._load_training_view(root)

    def test_positive_export_names_sb5_output_and_selects_exported_weights(self) -> None:
        class FixtureConfig:
            def validate(self) -> None:
                return None

            def as_hugging_face_config(self) -> dict[str, object]:
                return {"model_type": "c1-sb5-fixture"}

            def as_contract(self) -> dict[str, object]:
                return {"fixture": "c1-sb5"}

        class FixtureModel:
            config = FixtureConfig()

        class FixtureAdapter:
            def merged_model(self) -> FixtureModel:
                return FixtureModel()

            def binding_head_state_dict(self) -> dict[str, torch.Tensor]:
                return {
                    "bias": torch.tensor([0.0]),
                    "candidate_weight": torch.tensor([[1.0]]),
                    "query_weight": torch.tensor([[1.0]]),
                }

        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            tokenizer_bytes = b"fixture tokenizer bytes\n"
            (predecessor / "tokenizer.json").write_bytes(tokenizer_bytes)
            config_bytes = canonical_json_bytes({"model_type": "c1-sb5-fixture"})
            predecessor_weights_cid = cid_bytes(b"immutable predecessor weights")
            result = {"result_cid": cid_bytes(b"passing result")}
            dataset = {
                "dataset_cid": cid_bytes(b"dataset"),
                "split_policy_cid": cid_bytes(b"split"),
                "product_probe_commitments": [
                    cid_bytes(f"product-{index}".encode()) for index in range(4)
                ],
            }
            training_manifest = {"manifest_cid": cid_bytes(b"training manifest")}
            run_contract = {"run_contract_cid": cid_bytes(b"run contract")}
            with (
                patch(
                    "r4_softmax_trainer.export.export_state_dict",
                    return_value={"fixture.weight": torch.tensor([[2.0]])},
                ),
                patch.multiple(
                    campaign,
                    EXPECTED_CONFIG_CID=cid_bytes(config_bytes),
                    EXPECTED_TOKENIZER_CID=cid_bytes(tokenizer_bytes),
                    EXPECTED_WEIGHTS_CID=predecessor_weights_cid,
                ),
            ):
                delivery = campaign._write_positive_delivery(
                    root,
                    predecessor=predecessor,
                    adapter=FixtureAdapter(),  # type: ignore[arg-type]
                    result=result,
                    dataset=dataset,
                    training_manifest=training_manifest,
                    run_contract=run_contract,
                )
            checkpoint_manifest = delivery["checkpoint_manifest"]
            self.assertEqual(
                checkpoint_manifest["selected_checkpoint_identity"],
                "C1-SB5 merged LoRA training output",
            )
            self.assertEqual(
                checkpoint_manifest["selected_checkpoint_cid"],
                checkpoint_manifest["weights_cid"],
            )

    def test_prepare_and_negative_one_shot_never_read_or_emit_product(self) -> None:
        dataset, preflight, products = _fake_population()
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            predecessor_manifest = {
                "manifest_cid": cid_bytes(b"predecessor-manifest"),
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            with _patched_freeze(dataset, preflight, products):
                with (
                    patch.object(
                        campaign,
                        "_validated_predecessor",
                        return_value=predecessor_manifest,
                    ),
                    patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                    patch.object(campaign, "validate_tokenizer_contract"),
                    patch.object(
                        campaign,
                        "build_paired_query_binding_population",
                        return_value=(dataset, preflight, products),
                    ),
                ):
                    prepared = campaign.prepare_paired_query_binding_data(
                        root, predecessor=predecessor
                    )
            self.assertEqual(
                prepared["terminal"],
                "PAIRED_QUERY_BINDING_DATA_COMMITTED_NO_TRAINING",
            )
            with _patched_freeze(dataset, preflight, products):
                manifest_paths = {
                    row["path"]
                    for row in campaign._load_training_view(root)[3]["artifacts"]
                }
            self.assertEqual(manifest_paths, campaign.TRAINING_VIEW_ARTIFACTS)
            self.assertNotIn(campaign.PRODUCT_FILE, manifest_paths)

            original_read_text = Path.read_text

            def guarded_read_text(path: Path, *args: object, **kwargs: object) -> str:
                if path.name in {campaign.PRODUCT_FILE, campaign.PRODUCT_MANIFEST_FILE}:
                    raise AssertionError("optimizer opened product material")
                return original_read_text(path, *args, **kwargs)

            fit = _metrics(pairs=56, exact=True, mean_loss=0.1)
            sealed_negative = _metrics(pairs=28, exact=False, mean_loss=0.2)
            row_swap = _swapped_metrics(
                _metrics(pairs=28, exact=True, mean_loss=0.2)
            )
            mean_query = _metrics(
                pairs=28, exact=False, mean_loss=1.0, identical_rows=True
            )
            mean_query["mean_query_ablation"] = True
            attention_off = _metrics(pairs=28, exact=False, mean_loss=1.1)
            attention_off["attention_off"] = True
            evaluations = [
                fit,
                sealed_negative,
                sealed_negative,
                row_swap,
                mean_query,
                attention_off,
            ]

            class FakeAdapter:
                def to(self, _device: torch.device) -> "FakeAdapter":
                    return self

                def representation_audit(self, _base: object) -> dict[str, object]:
                    return _positive_delta()

            with _patched_freeze(dataset, preflight, products):
                with (
                    patch.object(Path, "read_text", guarded_read_text),
                    patch.object(
                        campaign,
                        "build_paired_query_binding_population",
                        side_effect=AssertionError("optimizer rebuilt product data"),
                    ),
                    patch.object(
                        campaign,
                        "_validated_predecessor",
                        return_value=predecessor_manifest,
                    ),
                    patch.object(campaign, "_run_contract") as run_contract,
                    patch.object(campaign, "require_mps", return_value=torch.device("cpu")),
                    patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                    patch.object(campaign, "validate_tokenizer_contract"),
                    patch.object(
                        campaign,
                        "EncodedPairedQueryBindingDataset",
                        side_effect=lambda records: _ScheduleDataset(records),
                    ),
                    patch.object(campaign, "_load_base_model", return_value=(object(), {})),
                    patch.object(
                        campaign,
                        "R4PairedQueryCandidateMatrix",
                        return_value=FakeAdapter(),
                    ),
                    patch.object(
                        campaign,
                        "fit_paired_query_binding",
                        return_value={
                            "optimizer_steps": 120,
                            "paired_records_per_step": 7,
                            "initial_loss": 3.0,
                            "final_loss": 0.1,
                        },
                    ),
                    patch.object(
                        campaign,
                        "evaluate_paired_query_binding",
                        side_effect=evaluations,
                    ),
                ):
                    run_contract.return_value = _with_cid(
                        {"schema": campaign.RUN_SCHEMA}, "run_contract_cid"
                    )
                    result = campaign.run_paired_query_binding_preflight(
                        root, predecessor=predecessor
                    )
            self.assertEqual(
                result["terminal"], "FAIL_PAIRED_QUERY_BINDING_PREFLIGHT"
            )
            self.assertEqual(result["research_checkpoint"], "NOT_EMITTED")
            self.assertEqual(result["binding_head"], "NOT_EMITTED")
            self.assertFalse((root / campaign.CHECKPOINT_DIRECTORY).exists())
            self.assertFalse((root / campaign.HEAD_DIRECTORY).exists())
            with self.assertRaisesRegex(FileExistsError, "already started"):
                with _patched_freeze(dataset, preflight, products):
                    with (
                        patch.object(
                            campaign,
                            "_validated_predecessor",
                            return_value=predecessor_manifest,
                        ),
                        patch.object(campaign, "_run_contract"),
                    ):
                        campaign.run_paired_query_binding_preflight(
                            root, predecessor=predecessor
                        )

    def test_positive_only_invokes_checkpoint_and_head_delivery(self) -> None:
        root = Path("/unused")
        common = {
            "predecessor": Path("/predecessor"),
            "dataset": {"dataset_cid": "dataset"},
            "preflight": {"preflight_cid": "preflight"},
            "tokenizer_census": {"tokenizer_census_cid": "tokenizer"},
            "training_manifest": {"manifest_cid": "training"},
            "run_contract": {"run_contract_cid": "run"},
        }
        passing = {"terminal": "PASS_PAIRED_QUERY_BINDING_PREFLIGHT_RESEARCH_ONLY"}
        negative = {"terminal": "FAIL_PAIRED_QUERY_BINDING_PREFLIGHT"}
        for value in (passing, negative):
            value["result_cid"] = cid_bytes(canonical_json_bytes(value))

        delivery = {
            "artifact": {"artifact_cid": "artifact"},
            "checkpoint_manifest": {"weights_cid": "weights"},
            "checkpoint_tree": {"checkpoint_tree_cid": "tree"},
            "head_manifest": {
                "manifest_cid": "head-manifest",
                "head_weights_cid": "head-weights",
            },
            "relative_paths": [],
        }
        with (
            patch.object(campaign, "_write_positive_delivery", return_value=delivery) as emit,
            patch.object(campaign, "atomic_write_json"),
            patch.object(
                campaign,
                "write_bound_manifest",
                return_value={"manifest_cid": "manifest"},
            ),
        ):
            campaign._finalize_result(
                root,
                adapter=object(),
                result=passing,
                **common,
            )
            emit.assert_called_once()
            emit.reset_mock()
            campaign._finalize_result(
                root,
                adapter=None,
                result=negative,
                **common,
            )
            emit.assert_not_called()


if __name__ == "__main__":
    unittest.main()
