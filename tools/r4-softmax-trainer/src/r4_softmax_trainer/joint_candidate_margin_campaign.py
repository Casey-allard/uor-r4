"""Prepared, one-shot C1-SB4 joint-candidate margin preflight.

Preparation is the only phase allowed to construct the fresh population or
open the four product records.  The optimizer entrypoint consumes a narrower
manifest containing fit/sealed records, aggregate tokenizer evidence, and
opaque product commitments only.
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Any, Mapping, Sequence

import torch
from tokenizers import Tokenizer

from .constants import FROZEN_MODEL_CONFIG
from .joint_candidate_margin import (
    FIT_SEED,
    POLICY,
    REPRESENTATION_UPDATE,
    EncodedJointCandidateMarginDataset,
    JointCandidateMarginAdapterConfig,
    JointCandidateMarginFitConfig,
    JointCandidateMarginWallBudgetExceeded,
    R4JointCandidateMarginAdapter,
    evaluate_joint_candidate_margin_adapter,
    fit_joint_candidate_margin_adapter,
)
from .joint_candidate_margin_data import (
    JOINT_CENSUS_SCHEMA,
    JOINT_DATASET_SCHEMA,
    JOINT_INPUT_POLICY,
    JOINT_PREFLIGHT_SCHEMA,
    build_joint_candidate_margin_population,
    render_joint_candidate_input,
)
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    trainer_implementation_contract,
    verify_bound_manifest,
    write_bound_manifest,
)
from .source_relation_adapter import (
    NO_TOKEN_ID,
    YES_TOKEN_ID,
    export_merged_attended_relation_checkpoint,
    validate_tokenizer_contract,
)
from .source_relation_adapter_campaign import (
    _delta_contract,
    _load_base_model,
    _rust_checkpoint_tree_binding,
    _validate_roots,
    _validated_predecessor,
)
from .source_relation_data import split_sentence_spans
from .train import require_mps


ISSUE = 954
EXPECTED_WEIGHTS_CID = (
    "blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
EXPECTED_CONFIG_CID = (
    "blake3:1f1ddb6de22f5c81c04d3093eeff8e0991d63b79ee33bc8ff3cf7c68ef0a9497"
)
EXPECTED_DATASET_CID = (
    "blake3:46e95f83f05bd5a3bfd4ca0c39c4974f617ae9591ff083d3c6abb8f5593c0e51"
)
EXPECTED_PREFLIGHT_CID = (
    "blake3:b61a098a53fca1f30b69a0ef0d6e15c5b6fc5a310d0ac8f0f2df04d1fd208814"
)
EXPECTED_PRODUCT_CID = (
    "blake3:153f6075f165d6cf92aeb63c31f05c9a869cc94c4655f83b9fe036bf7c773e3e"
)
EXPECTED_CENSUS_CID = (
    "blake3:2531f2c19cc60570c3807aa117b97923d4ce0b0c8111a8c239861cbae4303a92"
)
EXPECTED_TOKENIZER_CENSUS_CID = (
    "blake3:ff5c25b70d0940e0b43ec8cacc60c2b576ba069106d28c8ce58cb6b17f2cbc10"
)
EXPECTED_PREDECESSOR_MANIFEST_CID = (
    "blake3:77d5735ccfb4f2ac8a89f2f42a7ad8663b96770ea23a0b4bfae87b3daea7d8f3"
)
EXPECTED_PRODUCT_MANIFEST_CID = (
    "blake3:886d931fd6dde610955c6eedff3181fbffa8d8590f300731387f225f0012197b"
)
TOKENIZER_CENSUS_SCHEMA = "uor-r4.joint-candidate-margin-tokenizer-census/1"
DATA_MANIFEST_SCHEMA = "uor-r4.joint-candidate-margin-training-view-manifest/1"
PRODUCT_MANIFEST_SCHEMA = "uor-r4.joint-candidate-margin-product-manifest/1"
RUN_SCHEMA = "uor-r4.joint-candidate-margin-run/1"
RESULT_SCHEMA = "uor-r4.joint-candidate-margin-preflight-result/1"
RESULT_MANIFEST_SCHEMA = "uor-r4.joint-candidate-margin-preflight-manifest/1"
ARTIFACT_SCHEMA = "uor-r4.attended-relation-adapter/2"
ARTIFACT_INPUT_TEMPLATE = "E:<source>\nQ:<question>\nC:<group>\nSupported:"
RESEARCH_ADMISSION = "research_only"
FROZEN_READOUT_UPDATE = "none_frozen_readout"
MAXIMUM_SOURCE_SPANS = 8
CHECKPOINT_DIRECTORY = "preflight-checkpoint"
ADAPTER_ARTIFACT = "joint-candidate-margin-adapter.json"

DATASET_FILE = "joint-candidate-margin-dataset.json"
PREFLIGHT_FILE = "joint-candidate-margin-preflight.json"
STRUCTURAL_CENSUS_FILE = "joint-candidate-margin-census.json"
TOKENIZER_CENSUS_FILE = "joint-candidate-margin-tokenizer-census.json"
PRODUCT_FILE = "product-probes.json"
PRODUCT_MANIFEST_FILE = "product-commitments-manifest.json"
TRAINING_MANIFEST_FILE = "training-view-manifest.json"

TRAINING_VIEW_ARTIFACTS = {
    DATASET_FILE,
    PREFLIGHT_FILE,
    STRUCTURAL_CENSUS_FILE,
    TOKENIZER_CENSUS_FILE,
}

EXPECTED_TOKENIZER_PARTITIONS = {
    "fit": {"records": 126, "groups": 604, "maximum_positions_including_bos": 221},
    "sealed": {"records": 63, "groups": 302, "maximum_positions_including_bos": 197},
    "product": {"records": 4, "groups": 11, "maximum_positions_including_bos": 114},
}


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if not isinstance(observed, str):
        raise ValueError(f"C1-SB4 value has no {field}")
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"C1-SB4 {field} does not reproduce")


def _write_or_verify(path: Path, value: Mapping[str, Any]) -> None:
    encoded = canonical_json_bytes(value)
    if path.exists():
        if path.read_bytes() != encoded:
            raise ValueError(f"existing C1-SB4 artifact differs: {path}")
        return
    atomic_write_json(path, value)


def _tokenizer_partition(
    records: Sequence[Mapping[str, Any]], tokenizer: Tokenizer
) -> dict[str, Any]:
    dataset = EncodedJointCandidateMarginDataset(records, tokenizer)
    content_lengths = [
        len(group.token_ids) for record in dataset.encoded for group in record.groups
    ]
    if not content_lengths:
        raise RuntimeError("C1-SB4 tokenizer partition contains no group prompts")
    positions = [length + 1 for length in content_lengths]
    maximum = max(positions)
    passed = (
        maximum <= FROZEN_MODEL_CONFIG.max_position_embeddings
        and all(length > 0 for length in content_lengths)
    )
    if not passed:
        raise RuntimeError("C1-SB4 tokenizer census exceeds the frozen context")
    return {
        "records": len(dataset.records),
        "groups": len(content_lengths),
        "minimum_content_tokens": min(content_lengths),
        "maximum_content_tokens": max(content_lengths),
        "minimum_positions_including_bos": min(positions),
        "maximum_positions_including_bos": maximum,
        "context_ceiling_including_bos": FROZEN_MODEL_CONFIG.max_position_embeddings,
        "terminal_token": "standalone colon",
        "truncation": "FORBIDDEN_NOT_USED",
        "passed": True,
    }


def _build_tokenizer_census(
    *,
    tokenizer: Tokenizer,
    preflight: Mapping[str, Any],
    products: Mapping[str, Any],
) -> dict[str, Any]:
    validate_tokenizer_contract(tokenizer)
    partitions = {
        "fit": _tokenizer_partition(list(preflight["fit"]), tokenizer),
        "sealed": _tokenizer_partition(list(preflight["sealed"]), tokenizer),
        "product": _tokenizer_partition(list(products["records"]), tokenizer),
    }
    compact = {
        name: {
            key: observed[key]
            for key in ("records", "groups", "maximum_positions_including_bos")
        }
        for name, observed in partitions.items()
    }
    if compact != EXPECTED_TOKENIZER_PARTITIONS:
        raise RuntimeError(f"C1-SB4 tokenizer census drifted: {compact}")
    value = {
        "schema": TOKENIZER_CENSUS_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "input_policy": JOINT_INPUT_POLICY,
        "partitions": partitions,
        "all_prompts_end_at_standalone_colon": True,
        "no_prompt_truncated": True,
        "passed": all(bool(partition["passed"]) for partition in partitions.values()),
    }
    if not value["passed"]:
        raise RuntimeError("C1-SB4 tokenizer census is not positive")
    return _canonical_with_cid(value, "tokenizer_census_cid")


def prepare_joint_candidate_margin_data(
    root: Path, *, predecessor: Path
) -> dict[str, Any]:
    """Commit fresh data and products without starting or constructing a model."""
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    dataset, preflight, products = build_joint_candidate_margin_population()
    observed_cids = (
        dataset["dataset_cid"],
        preflight["preflight_cid"],
        products["product_probes_cid"],
        dataset["census_cid"],
    )
    expected_cids = (
        EXPECTED_DATASET_CID,
        EXPECTED_PREFLIGHT_CID,
        EXPECTED_PRODUCT_CID,
        EXPECTED_CENSUS_CID,
    )
    if observed_cids != expected_cids or not dataset["census"]["passed"]:
        raise RuntimeError("C1-SB4 structural commitments do not match the freeze")
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    tokenizer_census = _build_tokenizer_census(
        tokenizer=tokenizer, preflight=preflight, products=products
    )

    # Nothing is written until every structural and tokenizer check has passed.
    root.mkdir(parents=True, exist_ok=True)
    _write_or_verify(root / DATASET_FILE, dataset)
    _write_or_verify(root / PREFLIGHT_FILE, preflight)
    _write_or_verify(root / STRUCTURAL_CENSUS_FILE, dataset["census"])
    _write_or_verify(root / TOKENIZER_CENSUS_FILE, tokenizer_census)
    _write_or_verify(root / PRODUCT_FILE, products)

    product_manifest_path = root / PRODUCT_MANIFEST_FILE
    if product_manifest_path.exists():
        product_manifest = verify_bound_manifest(product_manifest_path, artifact_root=root)
    else:
        product_manifest = write_bound_manifest(
            product_manifest_path,
            {
                "schema": PRODUCT_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "product_probes_cid": products["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "access_policy": "opaque to every optimizer and preflight evaluator",
            },
            artifact_root=root,
            relative_paths=[PRODUCT_FILE],
        )

    training_manifest_path = root / TRAINING_MANIFEST_FILE
    if training_manifest_path.exists():
        training_manifest = verify_bound_manifest(training_manifest_path, artifact_root=root)
    else:
        training_manifest = write_bound_manifest(
            training_manifest_path,
            {
                "schema": DATA_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "dataset_cid": dataset["dataset_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "split_policy_cid": dataset["split_policy_cid"],
                "census_cid": dataset["census_cid"],
                "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
                "product_probes_cid": products["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "product_probe_count": 4,
                "product_manifest_cid": product_manifest["manifest_cid"],
                "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
                "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
                "product_text_access": "DENIED_TO_TRAINING_VIEW",
            },
            artifact_root=root,
            relative_paths=[
                DATASET_FILE,
                PREFLIGHT_FILE,
                STRUCTURAL_CENSUS_FILE,
                TOKENIZER_CENSUS_FILE,
            ],
        )
    if any(
        record["path"] in {PRODUCT_FILE, PRODUCT_MANIFEST_FILE}
        for record in training_manifest["artifacts"]
    ):
        raise RuntimeError("C1-SB4 training view accidentally contains product material")
    return {
        "terminal": "JOINT_CANDIDATE_MARGIN_DATA_COMMITTED_NO_TRAINING",
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "census_cid": dataset["census_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": dataset["product_probe_commitments"],
        "training_view_manifest_cid": training_manifest["manifest_cid"],
        "product_manifest_cid": product_manifest["manifest_cid"],
        "product_text_status": "COMMITTED_UNOPENED_BY_TRAINER",
    }


def _load_training_view(
    root: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Load only artifacts explicitly admitted to the optimizer view."""
    manifest = verify_bound_manifest(root / TRAINING_MANIFEST_FILE, artifact_root=root)
    if manifest.get("schema") != DATA_MANIFEST_SCHEMA:
        raise ValueError("unexpected C1-SB4 training-view schema")
    dataset = json.loads((root / DATASET_FILE).read_text(encoding="utf-8"))
    preflight = json.loads((root / PREFLIGHT_FILE).read_text(encoding="utf-8"))
    structural = json.loads(
        (root / STRUCTURAL_CENSUS_FILE).read_text(encoding="utf-8")
    )
    tokenizer_census = json.loads(
        (root / TOKENIZER_CENSUS_FILE).read_text(encoding="utf-8")
    )
    _verify_self_cid(dataset, "dataset_cid")
    _verify_self_cid(preflight, "preflight_cid")
    _verify_self_cid(structural, "census_cid")
    _verify_self_cid(tokenizer_census, "tokenizer_census_cid")
    frozen_identity = {
        "dataset_cid": EXPECTED_DATASET_CID,
        "preflight_cid": EXPECTED_PREFLIGHT_CID,
        "census_cid": EXPECTED_CENSUS_CID,
        "tokenizer_census_cid": EXPECTED_TOKENIZER_CENSUS_CID,
        "product_probes_cid": EXPECTED_PRODUCT_CID,
        "product_manifest_cid": EXPECTED_PRODUCT_MANIFEST_CID,
        "predecessor_export_manifest_cid": EXPECTED_PREDECESSOR_MANIFEST_CID,
        "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
        "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
    }
    if any(manifest.get(key) != value for key, value in frozen_identity.items()):
        raise ValueError("C1-SB4 training view does not bind the frozen campaign")
    if (
        dataset.get("schema") != JOINT_DATASET_SCHEMA
        or dataset.get("policy") != POLICY
        or dataset.get("issue") != ISSUE
        or dataset.get("dataset_cid") != EXPECTED_DATASET_CID
        or dataset.get("preflight_cid") != EXPECTED_PREFLIGHT_CID
        or dataset.get("census_cid") != EXPECTED_CENSUS_CID
        or dataset.get("product_probes_cid") != EXPECTED_PRODUCT_CID
        or dataset.get("relation_input_policy") != JOINT_INPUT_POLICY
    ):
        raise ValueError("C1-SB4 dataset identity or policy drifted")
    if (
        preflight.get("schema") != JOINT_PREFLIGHT_SCHEMA
        or preflight.get("policy") != POLICY
        or preflight.get("issue") != ISSUE
        or preflight.get("preflight_cid") != EXPECTED_PREFLIGHT_CID
        or preflight.get("census_cid") != EXPECTED_CENSUS_CID
    ):
        raise ValueError("C1-SB4 preflight identity or policy drifted")
    if (
        structural.get("schema") != JOINT_CENSUS_SCHEMA
        or structural.get("policy") != POLICY
        or structural.get("census_cid") != EXPECTED_CENSUS_CID
        or not structural.get("passed")
    ):
        raise ValueError("C1-SB4 structural census identity or policy drifted")
    tokenizer_compact = {
        name: {
            key: tokenizer_census.get("partitions", {}).get(name, {}).get(key)
            for key in ("records", "groups", "maximum_positions_including_bos")
        }
        for name in EXPECTED_TOKENIZER_PARTITIONS
    }
    if (
        tokenizer_census.get("schema") != TOKENIZER_CENSUS_SCHEMA
        or tokenizer_census.get("policy") != POLICY
        or tokenizer_census.get("issue") != ISSUE
        or tokenizer_census.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
        or tokenizer_census.get("input_policy") != JOINT_INPUT_POLICY
        or tokenizer_census.get("tokenizer_census_cid")
        != EXPECTED_TOKENIZER_CENSUS_CID
        or tokenizer_compact != EXPECTED_TOKENIZER_PARTITIONS
        or not tokenizer_census.get("all_prompts_end_at_standalone_colon")
        or not tokenizer_census.get("no_prompt_truncated")
        or not tokenizer_census.get("passed")
    ):
        raise ValueError("C1-SB4 tokenizer census identity or policy drifted")
    expected = {
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "census_cid": structural["census_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "product_probes_cid": dataset["product_probes_cid"],
        "product_probe_commitments": dataset["product_probe_commitments"],
        "product_probe_count": 4,
    }
    if any(manifest.get(key) != value for key, value in expected.items()):
        raise ValueError("C1-SB4 training view differs from its frozen commitments")
    if dataset["census"] != structural or not structural["passed"]:
        raise ValueError("C1-SB4 structural census differs or is not positive")
    artifact_paths = [record.get("path") for record in manifest.get("artifacts", [])]
    if len(artifact_paths) != len(TRAINING_VIEW_ARTIFACTS) or set(
        artifact_paths
    ) != TRAINING_VIEW_ARTIFACTS:
        raise ValueError("C1-SB4 training view artifact whitelist drifted")
    return dataset, preflight, tokenizer_census, manifest


def _record_metrics(
    records: Sequence[Mapping[str, Any]],
    raw: Mapping[str, Any],
    *,
    include_records: bool,
) -> dict[str, Any]:
    evaluations = raw.get("record_evaluations")
    if not isinstance(evaluations, list) or len(evaluations) != len(records):
        raise RuntimeError("C1-SB4 evaluation/record cardinality differs")
    by_id: dict[str, Mapping[str, Any]] = {}
    for evaluation in evaluations:
        record_id = evaluation.get("record_id")
        if not isinstance(record_id, str) or record_id in by_id:
            raise ValueError("C1-SB4 evaluation identities are missing or duplicated")
        by_id[record_id] = evaluation

    outcome = {
        name: {"correct": 0, "total": 0, "accuracy": 0.0}
        for name in ("answer", "abstain", "conflict")
    }
    positive_group_correct = positive_group_total = 0
    negative_group_correct = negative_group_total = 0
    positive_occurrence_correct = positive_occurrence_total = 0
    negative_occurrence_correct = negative_occurrence_total = 0
    copy_correct = copy_total = 0
    record_correct = 0
    details: list[dict[str, Any]] = []
    all_scores: list[float] = []
    for record in records:
        record_id = str(record.get("record_id", record.get("record_cid", "")))
        evaluation = by_id.get(record_id)
        if evaluation is None:
            raise RuntimeError(f"C1-SB4 record {record_id} has no evaluation")
        expected_groups: dict[str, dict[str, Any]] = {}
        for span in record["sentence_spans"]:
            group_cid = str(span["relation_group_cid"])
            group = expected_groups.setdefault(
                group_cid,
                {
                    "text": str(span["text"]),
                    "label": int(span["relation_label"]),
                    "occurrences": [],
                },
            )
            if group["text"] != span["text"] or group["label"] != span["relation_label"]:
                raise ValueError("C1-SB4 committed duplicate group disagrees")
            group["occurrences"].append(int(span["candidate_index"]))
        observed_rows = evaluation.get("group_scores")
        if not isinstance(observed_rows, list) or len(observed_rows) != len(expected_groups):
            raise RuntimeError("C1-SB4 group score cardinality differs")
        observed_groups: dict[str, Mapping[str, Any]] = {}
        for row in observed_rows:
            group_cid = row.get("relation_group_cid")
            score = row.get("score")
            if (
                not isinstance(group_cid, str)
                or group_cid in observed_groups
                or not isinstance(score, (int, float))
                or not math.isfinite(float(score))
            ):
                raise ValueError("C1-SB4 group score is missing, duplicated, or nonfinite")
            observed_groups[group_cid] = row
            all_scores.append(float(score))
        if set(observed_groups) != set(expected_groups):
            raise RuntimeError("C1-SB4 scored group identities differ")

        group_exact = True
        positive_observed: list[str] = []
        for group_cid, expected_group in expected_groups.items():
            observed = observed_groups[group_cid]
            if (
                observed.get("text") != expected_group["text"]
                or int(observed.get("relation_label", -1)) != expected_group["label"]
                or list(observed.get("occurrence_indices", []))
                != expected_group["occurrences"]
            ):
                raise RuntimeError("C1-SB4 score row metadata differs from the record")
            score = float(observed["score"])
            predicted = score > 0.0
            expected_positive = bool(expected_group["label"])
            group_exact = group_exact and predicted == expected_positive
            occurrences = len(expected_group["occurrences"])
            if expected_positive:
                positive_group_total += 1
                positive_group_correct += int(predicted)
                positive_occurrence_total += occurrences
                positive_occurrence_correct += occurrences * int(predicted)
            else:
                negative_group_total += 1
                negative_group_correct += int(not predicted)
                negative_occurrence_total += occurrences
                negative_occurrence_correct += occurrences * int(not predicted)
            if predicted:
                positive_observed.append(group_cid)

        if not positive_observed:
            decision = "abstain"
            selected_index = None
        elif len(positive_observed) == 1:
            decision = "answer"
            selected_index = min(expected_groups[positive_observed[0]]["occurrences"])
        else:
            decision = "conflict"
            selected_index = None
        expected_outcome = str(record["target_outcome"])
        outcome[expected_outcome]["total"] += 1
        outcome_exact = decision == expected_outcome
        outcome[expected_outcome]["correct"] += int(outcome_exact)
        copy_exact = True
        if expected_outcome == "answer":
            copy_total += 1
            copy_exact = bool(
                decision == "answer"
                and selected_index == record["target_span_index"]
                and selected_index is not None
                and record["sentence_spans"][selected_index]["text"] == record["answer"]
            )
            copy_correct += int(copy_exact)
        exact = group_exact and outcome_exact and copy_exact
        record_correct += int(exact)
        details.append(
            {
                "record_id": record_id,
                "lexical_world": record.get("lexical_world"),
                "motif": record["motif"],
                "target_outcome": expected_outcome,
                "decision": decision,
                "selected_span_index": selected_index,
                "target_span_index": record["target_span_index"],
                "group_signs_exact": group_exact,
                "outcome_exact": outcome_exact,
                "copy_exact": copy_exact,
                "record_exact": exact,
            }
        )
    for cell in outcome.values():
        cell["accuracy"] = cell["correct"] / cell["total"] if cell["total"] else 0.0
    value: dict[str, Any] = {
        "records": len(records),
        "record_exact": {
            "correct": record_correct,
            "total": len(records),
            "accuracy": record_correct / len(records),
        },
        "outcome": outcome,
        "positive_group_recall": {
            "correct": positive_group_correct,
            "total": positive_group_total,
            "accuracy": positive_group_correct / positive_group_total,
        },
        "negative_group_specificity": {
            "correct": negative_group_correct,
            "total": negative_group_total,
            "accuracy": negative_group_correct / negative_group_total,
        },
        "positive_relation_recall": {
            "correct": positive_occurrence_correct,
            "total": positive_occurrence_total,
            "accuracy": positive_occurrence_correct / positive_occurrence_total,
        },
        "negative_relation_specificity": {
            "correct": negative_occurrence_correct,
            "total": negative_occurrence_total,
            "accuracy": negative_occurrence_correct / negative_occurrence_total,
        },
        "supported_copied_span": {
            "correct": copy_correct,
            "total": copy_total,
            "accuracy": copy_correct / copy_total,
        },
        "mean_structured_margin": float(raw["mean_structured_margin"]),
        "score_range": {"minimum": min(all_scores), "maximum": max(all_scores)},
    }
    if include_records:
        value["record_evaluations"] = details
    return value


def _evaluate_records(
    adapter: R4JointCandidateMarginAdapter,
    records: Sequence[Mapping[str, Any]],
    *,
    tokenizer: Tokenizer,
    device: torch.device,
    include_records: bool = False,
) -> dict[str, Any]:
    dataset = EncodedJointCandidateMarginDataset(records, tokenizer)
    raw = evaluate_joint_candidate_margin_adapter(
        adapter, dataset, device=device, record_batch_size=7
    )
    return _record_metrics(records, raw, include_records=include_records)


def _all_exact(metrics: Mapping[str, Any]) -> bool:
    return metrics["record_exact"]["correct"] == metrics["record_exact"]["total"]


def _without_records(metrics: Mapping[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in metrics.items() if key != "record_evaluations"}


def _named_controls(metrics: Mapping[str, Any]) -> dict[str, Any]:
    evaluations = list(metrics.get("record_evaluations", []))
    query_motifs = {
        "matched-primary-answer",
        "matched-secondary-answer",
        "primary-source-secondary-abstain",
        "secondary-source-primary-abstain",
        "primary-distinct-location-conflict",
        "secondary-distinct-location-conflict",
    }
    query = [row for row in evaluations if row["motif"] in query_motifs]
    duplicate = [row for row in evaluations if row["motif"] == "exact-duplicate-agreement"]
    conflict = [row for row in evaluations if row["target_outcome"] == "conflict"]
    return {
        "same_source_query_relocation_exact": bool(query)
        and all(bool(row["record_exact"]) for row in query),
        "same_source_query_relocation_records": len(query),
        "duplicate_agreement_exact": bool(duplicate)
        and all(bool(row["record_exact"]) for row in duplicate),
        "duplicate_agreement_records": len(duplicate),
        "distinct_conflict_exact": bool(conflict)
        and all(bool(row["record_exact"]) for row in conflict),
        "distinct_conflict_records": len(conflict),
    }


def _reversed_records(
    records: Sequence[Mapping[str, Any]],
) -> tuple[list[dict[str, Any]], dict[str, int]]:
    """Rebuild full-source reversals; the SB3 row-only helper is invalid here."""
    reversed_records: list[dict[str, Any]] = []
    identity = 0
    for record in records:
        old_spans = [dict(span) for span in reversed(record["sentence_spans"])]
        source = " ".join(str(span["text"]) for span in old_spans)
        parsed = split_sentence_spans(source)
        spans: list[dict[str, Any]] = []
        for index, (old_span, parsed_span) in enumerate(zip(old_spans, parsed)):
            span = dict(old_span)
            span["candidate_index"] = index
            span["byte_start"] = int(parsed_span["byte_start"])
            span["byte_end"] = int(parsed_span["byte_end"])
            relation_input = render_joint_candidate_input(
                source, str(record["question"]), str(span["text"])
            )
            span["relation_input"] = relation_input
            span["relation_input_cid"] = cid_bytes(relation_input.encode("utf-8"))
            spans.append(span)
        value = dict(record)
        value.pop("record_cid", None)
        value["record_id"] = f"{record['record_cid']}:source-reversed"
        value["population"] = "preflight-source-order-control"
        value["motif"] = f"source-reversed-{record['motif']}"
        value["source"] = source
        value["source_cid"] = cid_bytes(source.encode("utf-8"))
        value["sentence_spans"] = spans
        value["positive_span_indices"] = [
            index for index, span in enumerate(spans) if int(span["relation_label"]) == 1
        ]
        if record["target_outcome"] == "answer":
            value["target_span_index"] = min(
                index
                for index, span in enumerate(spans)
                if int(span["relation_label"]) == 1 and span["text"] == record["answer"]
            )
        else:
            value["target_span_index"] = None
        is_identity = source == record["source"]
        value["source_reversal_identity"] = is_identity
        value["candidate_original_indices"] = list(
            reversed(range(int(record["source_width"])))
        )
        identity += int(is_identity)
        reversed_records.append(value)
    return reversed_records, {
        "records": len(reversed_records),
        "nontrivial_reversals": len(reversed_records) - identity,
        "byte_identical_reversals": identity,
    }


def _run_contract(
    *,
    predecessor_manifest: Mapping[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    tokenizer_census: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
) -> dict[str, Any]:
    return _canonical_with_cid(
        {
            "schema": RUN_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
            "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
            "adapter_contract": JointCandidateMarginAdapterConfig().as_contract(),
            "optimizer_contract": JointCandidateMarginFitConfig().as_contract(),
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "census_cid": dataset["census_cid"],
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "product_probes_cid": dataset["product_probes_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
            "product_text_access": "DENIED_TO_PYTHON_TRAINER_AND_EVALUATOR",
            "mechanism": (
                "full-source/question/distinct-exact-text candidate prompts through "
                "rank-8 Q/K/V/O LoRA in all six attention layers; fixed tied yes/no "
                "readout; duplicate groups collapsed; no learned head"
            ),
            "objective": (
                "mean records of relu(1-min positive group score) plus "
                "relu(1+max negative group score); absent-set term zero"
            ),
            "reachability_ceiling": "189/189 fit+sealed records = 100%",
            "instrument": {
                "fit_maximum_positions_including_bos": 221,
                "sealed_maximum_positions_including_bos": 197,
                "frozen_context_ceiling": 256,
                "measured_warm_step_seconds": 0.7586764579173177,
                "projected_270_step_seconds": 204.8426436376478,
            },
            "preflight_gate": {
                "fit_records": 126,
                "sealed_records": 63,
                "all_group_signs_outcomes_and_copies": "exact",
                "sealed_source_reversal": "exact after main gate only",
                "base_selection": (
                    "if the untrained sealed partition is exact, retain the simpler "
                    "frozen readout; otherwise the learned arm must be exact on fit "
                    "and sealed"
                ),
                "wall_ceiling_seconds": 600.0,
            },
            "if_positive": (
                "emit one research-only merged checkpoint and adapter/2 descriptor "
                "for one Rust parity probe"
            ),
            "if_negative": (
                "stop before Rust parity, full fit, development, or product without retry"
            ),
            "implementation": trainer_implementation_contract(),
        },
        "run_contract_cid",
    )


def _sanitized_budget_failure(
    error: JointCandidateMarginWallBudgetExceeded,
) -> dict[str, Any]:
    """Record the binding budget verdict without timing-dependent CID bytes."""
    observation = error.as_result()
    observation.pop("stopped_after_step", None)
    observation.pop("elapsed_seconds", None)
    observation.pop("projected_seconds_at_eta_probe", None)
    observation.update(
        {
            "budget_gate_passed": False,
            "timing_values": "OMITTED_FROM_CONTENT_ADDRESS_FOR_DETERMINISM",
        }
    )
    return observation


def _sanitized_optimization(optimization: Mapping[str, Any]) -> dict[str, Any]:
    """Keep a deterministic success verdict while excluding wall-clock noise."""
    result = {
        key: value
        for key, value in optimization.items()
        if key not in {"elapsed_seconds", "projected_seconds_at_eta_probe"}
    }
    if "optimizer_steps" in optimization:
        fit_contract = JointCandidateMarginFitConfig()
        result["operational_budget_observation"] = {
            "eta_probe_step": optimization.get(
                "eta_probe_step", fit_contract.eta_probe_step
            ),
            "wall_ceiling_seconds": optimization.get(
                "wall_ceiling_seconds", fit_contract.wall_ceiling_seconds
            ),
            "eta_probe_passed": True,
            "completed_within_wall_ceiling": True,
            "timing_values": "OMITTED_FROM_CONTENT_ADDRESS_FOR_DETERMINISM",
        }
    return result


def _build_adapter_artifact(
    *,
    representation_update: str,
    checkpoint_manifest: Mapping[str, Any],
    checkpoint_tree_cid: str,
    dataset: Mapping[str, Any],
    run_contract: Mapping[str, Any],
    result: Mapping[str, Any],
) -> dict[str, Any]:
    return _canonical_with_cid(
        {
            "schema": ARTIFACT_SCHEMA,
            "policy": POLICY,
            "issue": ISSUE,
            "admission": RESEARCH_ADMISSION,
            "representation_update": representation_update,
            "predecessor_model_weights_cid": EXPECTED_WEIGHTS_CID,
            "model_weights_cid": checkpoint_manifest["weights_cid"],
            "checkpoint_tree_cid": checkpoint_tree_cid,
            "config_cid": checkpoint_manifest["config_cid"],
            "tokenizer_cid": checkpoint_manifest["tokenizer_cid"],
            "hidden_size": FROZEN_MODEL_CONFIG.hidden_size,
            "supported_token_id": YES_TOKEN_ID,
            "unsupported_token_id": NO_TOKEN_ID,
            "threshold": 0.0,
            "maximum_source_spans": MAXIMUM_SOURCE_SPANS,
            "relation_input_policy": ARTIFACT_INPUT_TEMPLATE,
            "dataset_cid": dataset["dataset_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "run_contract_cid": run_contract["run_contract_cid"],
            "training_result_cid": result["result_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
        },
        "artifact_cid",
    )


def _persist_positive_delivery(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: Mapping[str, Any],
    adapter: R4JointCandidateMarginAdapter,
    representation_update: str,
    result: Mapping[str, Any],
    dataset: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> dict[str, Any]:
    relative_paths: list[str] = []
    if representation_update == REPRESENTATION_UPDATE:
        checkpoint = root / CHECKPOINT_DIRECTORY
        if checkpoint.exists():
            raise FileExistsError(f"C1-SB4 checkpoint already exists: {checkpoint}")
        export_merged_attended_relation_checkpoint(
            adapter,
            output_dir=checkpoint,
            tokenizer_path=predecessor / "tokenizer.json",
            training_result=dict(result),
            dataset_manifest_cid=str(dataset["dataset_cid"]),
            training_view_manifest_cid=str(training_manifest["manifest_cid"]),
            split_policy_cid=str(dataset["split_policy_cid"]),
            run_contract_cid=str(run_contract["run_contract_cid"]),
            selected_checkpoint_cid=None,
        )
        checkpoint_manifest = verify_bound_manifest(
            checkpoint / "export-manifest.json", artifact_root=checkpoint
        )
        relative_paths.extend(
            f"{CHECKPOINT_DIRECTORY}/{record['path']}"
            for record in checkpoint_manifest["artifacts"]
        )
        relative_paths.append(f"{CHECKPOINT_DIRECTORY}/export-manifest.json")
    elif representation_update == FROZEN_READOUT_UPDATE:
        checkpoint = predecessor
        checkpoint_manifest = dict(predecessor_manifest)
    else:
        raise ValueError("C1-SB4 positive delivery has unknown representation update")
    if (
        checkpoint_manifest["config_cid"] != EXPECTED_CONFIG_CID
        or checkpoint_manifest["tokenizer_cid"] != EXPECTED_TOKENIZER_CID
        or checkpoint_manifest["model_contract"] != FROZEN_MODEL_CONFIG.as_contract()
    ):
        raise ValueError("C1-SB4 positive checkpoint identity drifted")
    if representation_update == REPRESENTATION_UPDATE:
        if checkpoint_manifest["weights_cid"] == EXPECTED_WEIGHTS_CID:
            raise ValueError("C1-SB4 LoRA checkpoint did not change weights")
        if checkpoint_manifest["training_result_cid"] != result["result_cid"]:
            raise ValueError("C1-SB4 checkpoint does not bind its result")
    elif checkpoint_manifest["weights_cid"] != EXPECTED_WEIGHTS_CID:
        raise ValueError("C1-SB4 frozen readout changed predecessor weights")
    checkpoint_tree = _rust_checkpoint_tree_binding(checkpoint)
    artifact = _build_adapter_artifact(
        representation_update=representation_update,
        checkpoint_manifest=checkpoint_manifest,
        checkpoint_tree_cid=checkpoint_tree["checkpoint_tree_cid"],
        dataset=dataset,
        run_contract=run_contract,
        result=result,
    )
    _write_or_verify(root / ADAPTER_ARTIFACT, artifact)
    relative_paths.append(ADAPTER_ARTIFACT)
    return {
        "checkpoint": checkpoint,
        "checkpoint_manifest": checkpoint_manifest,
        "checkpoint_tree": checkpoint_tree,
        "artifact": artifact,
        "relative_paths": relative_paths,
    }


def _finalize_result(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: Mapping[str, Any],
    adapter: R4JointCandidateMarginAdapter | None,
    representation_update: str,
    result: dict[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    tokenizer_census: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    delivery = None
    if str(result["terminal"]).startswith("PASS_"):
        if adapter is None:
            raise ValueError("C1-SB4 passing preflight has no adapter")
        delivery = _persist_positive_delivery(
            root,
            predecessor=predecessor,
            predecessor_manifest=predecessor_manifest,
            adapter=adapter,
            representation_update=representation_update,
            result=result,
            dataset=dataset,
            training_manifest=training_manifest,
            run_contract=run_contract,
        )
    atomic_write_json(root / "preflight-result.json", result)
    relative_paths = [
        DATASET_FILE,
        PREFLIGHT_FILE,
        STRUCTURAL_CENSUS_FILE,
        TOKENIZER_CENSUS_FILE,
        TRAINING_MANIFEST_FILE,
        "run-contract.json",
        "preflight-started.json",
        "preflight-result.json",
    ]
    payload: dict[str, Any] = {
        "schema": RESULT_MANIFEST_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "terminal": result["terminal"],
        "representation_update": representation_update,
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "run_contract_cid": run_contract["run_contract_cid"],
        "result_cid": result["result_cid"],
        "product_status": "UNOPENED_NOT_RUN",
    }
    if delivery is not None:
        payload.update(
            {
                "adapter_artifact_cid": delivery["artifact"]["artifact_cid"],
                "checkpoint_tree_cid": delivery["checkpoint_tree"]["checkpoint_tree_cid"],
                "model_weights_cid": delivery["checkpoint_manifest"]["weights_cid"],
                "config_cid": delivery["checkpoint_manifest"]["config_cid"],
                "tokenizer_cid": delivery["checkpoint_manifest"]["tokenizer_cid"],
            }
        )
        relative_paths.extend(delivery["relative_paths"])
    manifest = write_bound_manifest(
        root / "preflight-manifest.json",
        payload,
        artifact_root=root,
        relative_paths=relative_paths,
    )
    return manifest, delivery


def run_joint_candidate_margin_preflight(
    root: Path, *, predecessor: Path
) -> dict[str, Any]:
    """Run exactly one frozen C1-SB4 preflight without product access."""
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    dataset, preflight, tokenizer_census, training_manifest = _load_training_view(root)
    if (root / "preflight-result.json").exists() or (
        root / "preflight-started.json"
    ).exists():
        raise FileExistsError("the sole C1-SB4 preflight was already started")
    run_contract = _run_contract(
        predecessor_manifest=predecessor_manifest,
        dataset=dataset,
        preflight=preflight,
        tokenizer_census=tokenizer_census,
        training_manifest=training_manifest,
    )
    _write_or_verify(root / "run-contract.json", run_contract)
    atomic_write_json(
        root / "preflight-started.json",
        {
            "schema": RUN_SCHEMA,
            "phase": "SOLE_C1_SB4_PREFLIGHT_STARTED",
            "run_contract_cid": run_contract["run_contract_cid"],
        },
    )

    device = require_mps(FIT_SEED)
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    validate_tokenizer_contract(tokenizer)
    fit_records = list(preflight["fit"])
    sealed_records = list(preflight["sealed"])
    # Reproduce only the two admitted partitions. Product aggregate evidence is
    # bound from preparation and its text remains inaccessible here.
    for name, records in (("fit", fit_records), ("sealed", sealed_records)):
        observed = _tokenizer_partition(records, tokenizer)
        committed = tokenizer_census["partitions"][name]
        if observed != committed:
            raise ValueError(f"C1-SB4 {name} tokenizer census does not reproduce")

    model, base_state = _load_base_model(predecessor)
    adapter = R4JointCandidateMarginAdapter(model).to(device)
    untrained_sealed_detailed = _evaluate_records(
        adapter,
        sealed_records,
        tokenizer=tokenizer,
        device=device,
        include_records=True,
    )
    untrained_sealed = _without_records(untrained_sealed_detailed)
    representation_update = REPRESENTATION_UPDATE
    base_sealed_exact = _all_exact(untrained_sealed_detailed)
    untrained_fit_detailed = (
        _evaluate_records(
            adapter,
            fit_records,
            tokenizer=tokenizer,
            device=device,
            include_records=True,
        )
        if base_sealed_exact
        else None
    )
    if base_sealed_exact:
        representation_update = FROZEN_READOUT_UPDATE
        optimization: Mapping[str, Any] = {
            "status": "SKIPPED_SIMPLER_SEALED_READOUT_ALREADY_EXACT"
        }
        if untrained_fit_detailed is None:
            raise AssertionError("exact frozen gate has no fit evaluation")
        fit_detailed = untrained_fit_detailed
        sealed_detailed = untrained_sealed_detailed
        delta = {
            "status": "NOT_APPLICABLE_FROZEN_READOUT",
            "passed": True,
            "representation_update": FROZEN_READOUT_UPDATE,
        }
    else:
        fit_dataset = EncodedJointCandidateMarginDataset(fit_records, tokenizer)
        try:
            optimization = fit_joint_candidate_margin_adapter(adapter, fit_dataset)
        except JointCandidateMarginWallBudgetExceeded as error:
            result = _canonical_with_cid(
                {
                    "schema": RESULT_SCHEMA,
                    "issue": ISSUE,
                    "policy": POLICY,
                    "terminal": "UNAVAILABLE_JOINT_CANDIDATE_MARGIN_BUDGET",
                    "representation_update": REPRESENTATION_UPDATE,
                    "run_contract_cid": run_contract["run_contract_cid"],
                    "dataset_cid": dataset["dataset_cid"],
                    "preflight_cid": preflight["preflight_cid"],
                    "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
                    "training_view_manifest_cid": training_manifest["manifest_cid"],
                    "untrained_sealed_metrics": untrained_sealed,
                    "operational_budget_observation": _sanitized_budget_failure(
                        error
                    ),
                    "fit_metrics": "NOT_RUN",
                    "sealed_metrics": "NOT_RUN",
                    "controls": "NOT_RUN",
                    "delta_audit": _delta_contract(adapter, base_state),
                    "rust_parity": "NOT_RUN",
                    "full_fit": "NOT_RUN",
                    "development": "NOT_RUN",
                    "product": "UNOPENED_NOT_RUN",
                },
                "result_cid",
            )
            manifest, _ = _finalize_result(
                root,
                predecessor=predecessor,
                predecessor_manifest=predecessor_manifest,
                adapter=None,
                representation_update=REPRESENTATION_UPDATE,
                result=result,
                dataset=dataset,
                preflight=preflight,
                tokenizer_census=tokenizer_census,
                training_manifest=training_manifest,
                run_contract=run_contract,
            )
            return {**result, "manifest_cid": manifest["manifest_cid"]}
        fit_detailed = _evaluate_records(
            adapter,
            fit_records,
            tokenizer=tokenizer,
            device=device,
            include_records=True,
        )
        sealed_detailed = _evaluate_records(
            adapter,
            sealed_records,
            tokenizer=tokenizer,
            device=device,
            include_records=True,
        )
        delta = _delta_contract(adapter, base_state)

    if representation_update == FROZEN_READOUT_UPDATE:
        # The predeclared decision says an exact untrained sealed partition selects
        # the simpler mechanism.  Fit remains visible diagnostic evidence, not a
        # post-hoc extra gate on that frozen branch.
        main_passed = _all_exact(sealed_detailed) and bool(delta["passed"])
    else:
        main_passed = (
            not base_sealed_exact
            and _all_exact(fit_detailed)
            and _all_exact(sealed_detailed)
            and bool(delta["passed"])
        )
    named = {
        "fit": _named_controls(fit_detailed),
        "sealed": _named_controls(sealed_detailed),
    }
    if main_passed:
        reversed_sealed, reversal_scope = _reversed_records(sealed_records)
        if reversal_scope != {
            "records": 63,
            "nontrivial_reversals": 62,
            "byte_identical_reversals": 1,
        }:
            raise RuntimeError(f"C1-SB4 reversal scope drifted: {reversal_scope}")
        reversal_metrics = _evaluate_records(
            adapter,
            reversed_sealed,
            tokenizer=tokenizer,
            device=device,
        )
        reversal_exact = _all_exact(reversal_metrics)
        controls: Mapping[str, Any] = {
            "named_subsets": named,
            "sealed_source_reversal_scope": reversal_scope,
            "sealed_source_reversal_metrics": reversal_metrics,
            "sealed_source_reversal_exact": reversal_exact,
            "passed": reversal_exact,
        }
    else:
        controls = {
            "named_subsets": named,
            "sealed_source_reversal": "NOT_RUN_MAIN_GATE_NEGATIVE",
            "passed": False,
        }
    passed = main_passed and bool(controls["passed"])
    if passed and representation_update == FROZEN_READOUT_UPDATE:
        terminal = "PASS_FROZEN_JOINT_SOURCE_READOUT_AWAITING_RUST_PARITY"
    elif passed:
        terminal = "PASS_JOINT_CANDIDATE_MARGIN_PREFLIGHT_AWAITING_RUST_PARITY"
    else:
        terminal = "FAIL_JOINT_CANDIDATE_MARGIN_PREFLIGHT"

    sanitized_optimization = _sanitized_optimization(optimization)
    fit_metrics = _without_records(fit_detailed)
    sealed_metrics = _without_records(sealed_detailed)
    result = _canonical_with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": terminal,
            "representation_update": representation_update,
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "untrained_sealed_metrics": untrained_sealed,
            "optimization": sanitized_optimization,
            "fit_metrics": fit_metrics,
            "sealed_metrics": sealed_metrics,
            "controls": controls,
            "delta_audit": delta,
            "rust_parity": "NOT_RUN",
            "full_fit": "NOT_RUN",
            "development": "NOT_RUN",
            "product": "UNOPENED_NOT_RUN",
        },
        "result_cid",
    )
    manifest, delivery = _finalize_result(
        root,
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        adapter=adapter,
        representation_update=representation_update,
        result=result,
        dataset=dataset,
        preflight=preflight,
        tokenizer_census=tokenizer_census,
        training_manifest=training_manifest,
        run_contract=run_contract,
    )
    response: dict[str, Any] = {
        "terminal": terminal,
        "result_cid": result["result_cid"],
        "manifest_cid": manifest["manifest_cid"],
        "fit_metrics": fit_metrics,
        "sealed_metrics": sealed_metrics,
        "controls": controls,
        "delta_audit": delta,
        "rust_parity": "NOT_RUN",
        "full_fit": "NOT_RUN",
        "development": "NOT_RUN",
        "product": "UNOPENED_NOT_RUN",
    }
    if delivery is not None:
        response.update(
            {
                "adapter_artifact_cid": delivery["artifact"]["artifact_cid"],
                "checkpoint_tree_cid": delivery["checkpoint_tree"]["checkpoint_tree_cid"],
                "model_weights_cid": delivery["checkpoint_manifest"]["weights_cid"],
                "checkpoint": str(delivery["checkpoint"]),
            }
        )
    return response


__all__ = [
    "prepare_joint_candidate_margin_data",
    "run_joint_candidate_margin_preflight",
]
