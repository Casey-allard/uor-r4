"""Frozen C1-SB3 representation-transfer campaign.

The campaign has two deliberately separate entry points.  ``prepare`` writes
the independently committed data, including the product envelope.  ``run``
reads only the training-view manifest, so product text is not opened by the
Python optimizer or evaluator.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Mapping, Sequence

import torch
from safetensors.torch import load_file
from tokenizers import Tokenizer

from .constants import FROZEN_MODEL_CONFIG
from .model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    verify_bound_manifest,
    write_bound_manifest,
)
from .source_relation_adapter import (
    ARTIFACT_SCHEMA,
    NO_TOKEN_ID,
    POLICY,
    RELATION_INPUT_TEMPLATE,
    REPRESENTATION_UPDATE,
    YES_TOKEN_ID,
    AdapterFitConfig,
    AttendedRelationWallBudgetExceeded,
    AttendedRelationAdapterConfig,
    EncodedRelationDataset,
    R4AttendedRelationAdapter,
    evaluate_attended_relation_adapter,
    export_merged_attended_relation_checkpoint,
    fit_attended_relation_adapter,
    relation_examples_from_records,
    validate_tokenizer_contract,
)
from .source_relation_adapter_data import (
    OUTCOMES,
    build_source_relation_adapter_population,
)
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
DATA_MANIFEST_SCHEMA = "uor-r4.attended-relation-training-view-manifest/1"
PRODUCT_MANIFEST_SCHEMA = "uor-r4.attended-relation-product-commitments-manifest/1"
RUN_SCHEMA = "uor-r4.attended-relation-run/1"
PREFLIGHT_RESULT_SCHEMA = "uor-r4.attended-relation-preflight-result/1"
PREFLIGHT_MANIFEST_SCHEMA = "uor-r4.attended-relation-preflight-manifest/1"
RUST_CHECKPOINT_TREE_SCHEMA = "uor-r4.r4-softmax-local-checkpoint-tree/1"
RESEARCH_ADMISSION = "research_only"
FROZEN_READOUT_UPDATE = "none_frozen_readout"
MAXIMUM_SOURCE_SPANS = 8
POSITIVE_CHECKPOINT_DIRECTORY = "preflight-checkpoint"
ATTENDED_ADAPTER_ARTIFACT = "attended-relation-adapter.json"


@dataclass(frozen=True, slots=True)
class AttendedRelationCampaignConfig:
    """The sole frozen C1-SB3 work budget."""

    seed: int = 9_543
    preflight_optimizer_steps: int = 192
    full_optimizer_steps: int = 256
    candidate_batch_size: int = 64
    evaluation_batch_size: int = 64
    learning_rate: float = 0.001
    adam_beta1: float = 0.9
    adam_beta2: float = 0.999
    adam_epsilon: float = 1e-8
    weight_decay: float = 0.0
    gradient_clip: float = 1.0
    eta_probe_step: int = 8
    preflight_wall_ceiling_seconds: float = 600.0
    full_wall_ceiling_seconds: float = 1_200.0
    development_floor: float = 0.95

    def validate(self) -> None:
        if self != AttendedRelationCampaignConfig():
            raise ValueError("C1-SB3 exposes one frozen campaign, not a sweep")
        if (
            self.seed != 9_543
            or self.preflight_optimizer_steps != 192
            or self.full_optimizer_steps != 256
            or self.candidate_batch_size != 64
        ):
            raise AssertionError("C1-SB3 work budget drifted")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        return asdict(self)


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _write_or_verify(path: Path, value: dict[str, Any]) -> None:
    encoded = canonical_json_bytes(value)
    if path.exists():
        if path.read_bytes() != encoded:
            raise ValueError(f"existing C1-SB3 artifact differs: {path}")
        return
    atomic_write_json(path, value)


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if not isinstance(observed, str):
        raise ValueError(f"C1-SB3 value has no {field}")
    expected = cid_bytes(canonical_json_bytes(unsigned))
    if observed != expected:
        raise ValueError(f"C1-SB3 {field} does not reproduce")


def _validate_roots(root: Path, predecessor: Path) -> tuple[Path, Path]:
    root = root.expanduser().resolve()
    predecessor = predecessor.expanduser().resolve()
    if root == predecessor or root in predecessor.parents or predecessor in root.parents:
        raise ValueError("adapter output and immutable #1017 predecessor must be disjoint")
    return root, predecessor


def _validated_predecessor(predecessor: Path) -> dict[str, Any]:
    manifest = verify_bound_manifest(
        predecessor / "export-manifest.json", artifact_root=predecessor
    )
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("adapter predecessor is not the exact six-layer architecture")
    if manifest.get("weights_cid") != EXPECTED_WEIGHTS_CID:
        raise ValueError("adapter predecessor weights differ from #1017")
    if manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
        raise ValueError("adapter predecessor tokenizer differs from #1017")
    if manifest.get("config_cid") != EXPECTED_CONFIG_CID:
        raise ValueError("adapter predecessor config differs from #1017")
    if cid_file(predecessor / "model.safetensors") != EXPECTED_WEIGHTS_CID:
        raise ValueError("#1017 weights file CID does not reproduce")
    if cid_file(predecessor / "tokenizer.json") != EXPECTED_TOKENIZER_CID:
        raise ValueError("#1017 tokenizer file CID does not reproduce")
    if cid_file(predecessor / "config.json") != EXPECTED_CONFIG_CID:
        raise ValueError("#1017 config file CID does not reproduce")
    return manifest


def prepare_attended_relation_data(
    root: Path,
    *,
    predecessor: Path,
) -> dict[str, Any]:
    """Commit data and products before optimization, then return without training."""
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    dataset, preflight, products = build_source_relation_adapter_population()
    if dataset.get("policy") != POLICY or preflight.get("policy") != POLICY:
        raise ValueError("C1-SB3 data and mechanism policies differ")
    if not dataset.get("census", {}).get("passed"):
        raise RuntimeError("C1-SB3 zero-training census is not positive")

    root.mkdir(parents=True, exist_ok=True)
    _write_or_verify(root / "attended-relation-dataset.json", dataset)
    _write_or_verify(root / "attended-relation-preflight.json", preflight)
    _write_or_verify(root / "attended-relation-census.json", dataset["census"])
    _write_or_verify(root / "product-probes.json", products)

    product_manifest_path = root / "product-commitments-manifest.json"
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
                "access_policy": (
                    "Python training receives only commitment CIDs/count and must not "
                    "open this product artifact"
                ),
            },
            artifact_root=root,
            relative_paths=["product-probes.json"],
        )

    training_manifest_path = root / "training-view-manifest.json"
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
                "product_probes_cid": products["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "product_probe_count": len(dataset["product_probe_commitments"]),
                "product_manifest_cid": product_manifest["manifest_cid"],
                "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
                "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
            },
            artifact_root=root,
            relative_paths=[
                "attended-relation-dataset.json",
                "attended-relation-preflight.json",
                "attended-relation-census.json",
            ],
        )
    return {
        "terminal": "ATTENDED_RELATION_DATA_COMMITTED_NO_TRAINING",
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "census_cid": dataset["census_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": dataset["product_probe_commitments"],
        "training_view_manifest_cid": training_manifest["manifest_cid"],
        "product_manifest_cid": product_manifest["manifest_cid"],
        "product_text_status": "COMMITTED_UNOPENED_BY_TRAINER",
    }


def _load_training_view(root: Path) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    # This manifest intentionally excludes product-probes.json from its files.
    manifest = verify_bound_manifest(root / "training-view-manifest.json", artifact_root=root)
    if manifest.get("schema") != DATA_MANIFEST_SCHEMA:
        raise ValueError("unexpected C1-SB3 training-view schema")
    dataset = json.loads(
        (root / "attended-relation-dataset.json").read_text(encoding="utf-8")
    )
    preflight = json.loads(
        (root / "attended-relation-preflight.json").read_text(encoding="utf-8")
    )
    _verify_self_cid(dataset, "dataset_cid")
    _verify_self_cid(preflight, "preflight_cid")
    if (
        manifest.get("dataset_cid") != dataset["dataset_cid"]
        or manifest.get("preflight_cid") != preflight["preflight_cid"]
        or manifest.get("census_cid") != dataset["census_cid"]
        or manifest.get("product_probe_commitments")
        != dataset["product_probe_commitments"]
        or manifest.get("product_probe_count") != 4
    ):
        raise ValueError("C1-SB3 training view does not match its frozen commitments")
    if not dataset.get("census", {}).get("passed"):
        raise RuntimeError("C1-SB3 census is not positive")
    return dataset, preflight, manifest


def _load_base_model(predecessor: Path) -> tuple[R4SoftmaxForCausalLM, dict[str, torch.Tensor]]:
    base_state = load_file(str(predecessor / "model.safetensors"), device="cpu")
    if set(base_state) != expected_hf_tensor_names():
        raise ValueError("#1017 tensor inventory differs from the frozen model")
    model = R4SoftmaxForCausalLM()
    model.load_state_dict(base_state, strict=True)
    model.requires_grad_(False)
    model.eval()
    return model, base_state


def _record_metrics(
    records: Sequence[Mapping[str, Any]],
    *,
    raw: Mapping[str, Any],
    examples: Sequence[Any],
    include_records: bool,
) -> dict[str, Any]:
    scores = list(raw["scores"])
    if len(scores) != len(examples):
        raise RuntimeError("C1-SB3 score/example cardinality differs")
    raw_labels = [int(label) for label in raw["labels"]]
    expected_labels = [int(example.relation_label) for example in examples]
    if raw_labels != expected_labels:
        raise RuntimeError("C1-SB3 score labels do not follow the ordered examples")
    candidate_identities = [
        (example.record_id, int(example.candidate_index)) for example in examples
    ]
    if len(set(candidate_identities)) != len(candidate_identities):
        raise ValueError("C1-SB3 candidate identities are not unique")
    score_by_candidate = {
        identity: float(score)
        for identity, score in zip(candidate_identities, scores)
    }
    outcome = {
        name: {"correct": 0, "total": 0, "accuracy": 0.0} for name in OUTCOMES
    }
    positive_correct = positive_total = 0
    negative_correct = negative_total = 0
    copy_correct = copy_total = 0
    evaluations: list[dict[str, Any]] = []
    for record in records:
        record_id_value = record.get("record_id") or record.get("record_cid")
        if not isinstance(record_id_value, str) or not record_id_value:
            raise ValueError("C1-SB3 record has no stable identity")
        record_id = record_id_value
        candidate_scores = [
            score_by_candidate[(record_id, int(span["candidate_index"]))]
            for span in record["sentence_spans"]
        ]
        representatives: dict[str, int] = {}
        relation_exact = True
        for index, (span, score) in enumerate(zip(record["sentence_spans"], candidate_scores)):
            expected = bool(span["relation_label"])
            observed = score > 0.0
            relation_exact = relation_exact and observed == expected
            if expected:
                positive_total += 1
                positive_correct += int(observed)
            else:
                negative_total += 1
                negative_correct += int(not observed)
            if not observed:
                continue
            text = str(span["text"])
            prior = representatives.get(text)
            if prior is None or (score, -int(span["byte_start"]), -index) > (
                candidate_scores[prior],
                -int(record["sentence_spans"][prior]["byte_start"]),
                -prior,
            ):
                representatives[text] = index
        if not representatives:
            decision = "abstain"
            selected = None
        elif len(representatives) == 1:
            decision = "answer"
            selected = next(iter(representatives.values()))
        else:
            decision = "conflict"
            selected = None
        expected_outcome = str(record["target_outcome"])
        outcome[expected_outcome]["total"] += 1
        outcome[expected_outcome]["correct"] += int(decision == expected_outcome)
        copy_exact = True
        if expected_outcome == "answer":
            copy_total += 1
            copy_exact = bool(
                decision == "answer"
                and selected is not None
                and str(record["sentence_spans"][selected]["text"])
                == str(record["answer"])
                and selected == record["target_span_index"]
            )
            copy_correct += int(copy_exact)
        evaluations.append(
            {
                "record_cid": record_id,
                "lexical_world": record.get("lexical_world"),
                "source_cid": record.get("source_cid"),
                "question_cid": record.get("question_cid"),
                "motif": record["motif"],
                "target_outcome": expected_outcome,
                "candidate_scores": candidate_scores,
                "decision": decision,
                "selected_span_index": selected,
                "target_span_index": record["target_span_index"],
                "relation_exact": relation_exact,
                "outcome_exact": decision == expected_outcome,
                "copy_exact": copy_exact,
                "record_exact": relation_exact
                and decision == expected_outcome
                and copy_exact,
            }
        )
    for cell in outcome.values():
        cell["accuracy"] = cell["correct"] / cell["total"] if cell["total"] else 0.0
    value: dict[str, Any] = {
        "records": len(records),
        "mean_binary_cross_entropy": raw["mean_binary_cross_entropy"],
        "outcome": outcome,
        "positive_relation_recall": {
            "correct": positive_correct,
            "total": positive_total,
            "accuracy": positive_correct / positive_total,
        },
        "negative_relation_specificity": {
            "correct": negative_correct,
            "total": negative_total,
            "accuracy": negative_correct / negative_total,
        },
        "supported_copied_span": {
            "correct": copy_correct,
            "total": copy_total,
            "accuracy": copy_correct / copy_total,
        },
    }
    if include_records:
        value["record_evaluations"] = evaluations
    return value


def _evaluate_records(
    adapter: R4AttendedRelationAdapter,
    records: Sequence[Mapping[str, Any]],
    *,
    tokenizer: Tokenizer,
    device: torch.device,
    batch_size: int,
    include_records: bool = False,
) -> dict[str, Any]:
    examples = relation_examples_from_records(records)
    encoded = EncodedRelationDataset(examples, tokenizer)
    raw = evaluate_attended_relation_adapter(
        adapter, encoded, device=device, batch_size=batch_size
    )
    return _record_metrics(
        records,
        raw=raw,
        # EncodedRelationDataset sorts by stable record/candidate identity before
        # batching. Scores are emitted in that order, so aggregation must use
        # the same ordered example inventory rather than the caller's order.
        examples=encoded.examples,
        include_records=include_records,
    )


def _all_exact(metrics: Mapping[str, Any]) -> bool:
    return (
        all(
            metrics["outcome"][name]["correct"]
            == metrics["outcome"][name]["total"]
            for name in OUTCOMES
        )
        and metrics["positive_relation_recall"]["correct"]
        == metrics["positive_relation_recall"]["total"]
        and metrics["negative_relation_specificity"]["correct"]
        == metrics["negative_relation_specificity"]["total"]
        and metrics["supported_copied_span"]["correct"]
        == metrics["supported_copied_span"]["total"]
    )


def _without_record_evaluations(metrics: Mapping[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in metrics.items() if key != "record_evaluations"}


def _named_control_metrics(
    fit_metrics: Mapping[str, Any], sealed_metrics: Mapping[str, Any]
) -> dict[str, Any]:
    evaluations = [
        *fit_metrics.get("record_evaluations", []),
        *sealed_metrics.get("record_evaluations", []),
    ]

    query_motifs = {
        "matched-primary-answer",
        "matched-secondary-answer",
        "primary-source-secondary-abstain",
        "secondary-source-primary-abstain",
        "primary-distinct-location-conflict",
        "secondary-distinct-location-conflict",
    }
    query_records = [
        evaluation
        for evaluation in evaluations
        if evaluation["motif"] in query_motifs
    ]
    query_pairs: dict[tuple[Any, Any], list[Mapping[str, Any]]] = {}
    for evaluation in query_records:
        key = (evaluation["lexical_world"], evaluation["source_cid"])
        query_pairs.setdefault(key, []).append(evaluation)
    query_isolated = bool(query_pairs) and all(
        len(pair) == 2
        and len({evaluation["question_cid"] for evaluation in pair}) == 2
        for pair in query_pairs.values()
    )

    duplicate_records = [
        evaluation
        for evaluation in evaluations
        if evaluation["motif"] == "exact-duplicate-agreement"
    ]
    conflict_records = [
        evaluation
        for evaluation in evaluations
        if evaluation["target_outcome"] == "conflict"
    ]

    def exact_or_not_isolated(
        records: Sequence[Mapping[str, Any]], *, isolated: bool = True
    ) -> bool | str:
        if not records or not isolated:
            return "NOT_ISOLATED"
        return all(bool(record["record_exact"]) for record in records)

    return {
        "same_source_query_relocation_exact": exact_or_not_isolated(
            query_records, isolated=query_isolated
        ),
        "same_source_query_relocation_records": len(query_records),
        "same_source_query_relocation_pairs": len(query_pairs) if query_isolated else 0,
        "duplicate_agreement_exact": exact_or_not_isolated(duplicate_records),
        "duplicate_agreement_records": len(duplicate_records),
        "distinct_conflict_exact": exact_or_not_isolated(conflict_records),
        "distinct_conflict_records": len(conflict_records),
    }


def _reversed_records(records: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    reversed_records: list[dict[str, Any]] = []
    for record in records:
        spans = [dict(span) for span in reversed(record["sentence_spans"])]
        for index, span in enumerate(spans):
            span["candidate_index"] = index
        target = record["target_span_index"]
        value = dict(record)
        value.pop("record_cid", None)
        value["record_id"] = f"{record['record_cid']}:reversed"
        value["population"] = "preflight-order-control"
        value["motif"] = f"reversed-{record['motif']}"
        value["sentence_spans"] = spans
        value["target_span_index"] = None if target is None else len(spans) - 1 - int(target)
        value["positive_span_indices"] = [
            index for index, span in enumerate(spans) if int(span["relation_label"]) == 1
        ]
        reversed_records.append(value)
    return reversed_records


def _delta_contract(
    adapter: R4AttendedRelationAdapter,
    base_state: Mapping[str, torch.Tensor],
) -> dict[str, Any]:
    merged = adapter.merged_state_dict()
    target_names = {
        f"model.layers.{layer}.self_attn.{projection}.weight"
        for layer in range(6)
        for projection in ("q_proj", "k_proj", "v_proj", "o_proj")
    }
    changed_targets = sorted(
        name for name in target_names if not torch.equal(merged[name], base_state[name])
    )
    changed_nontargets = sorted(
        name
        for name in merged
        if name not in target_names and not torch.equal(merged[name], base_state[name])
    )
    finite = all(bool(torch.isfinite(merged[name]).all()) for name in target_names)
    return {
        "target_tensor_count": len(target_names),
        "changed_target_tensor_count": len(changed_targets),
        "changed_target_tensors": changed_targets,
        "changed_nontarget_tensors": changed_nontargets,
        "all_target_tensors_finite": finite,
        "passed": len(changed_targets) == 24 and not changed_nontargets and finite,
    }


def _rust_checkpoint_tree_binding(checkpoint: Path) -> dict[str, Any]:
    """Reproduce the Rust local-checkpoint tree identity byte for byte."""
    checkpoint = checkpoint.resolve()
    if not checkpoint.is_dir():
        raise ValueError(f"C1-SB3 checkpoint is not a directory: {checkpoint}")
    records: list[dict[str, Any]] = []
    for path in checkpoint.rglob("*"):
        relative = path.relative_to(checkpoint)
        if any(part.startswith(".") for part in relative.parts):
            continue
        if path.is_symlink():
            raise ValueError(f"C1-SB3 checkpoint contains a symlink: {relative}")
        if not path.is_file():
            continue
        name = path.name
        admitted = (
            name != "source_manifest.json"
            and (
                path.suffix in {".safetensors", ".json", ".model"}
                or name == "merges.txt"
                or name.startswith("LICENSE")
                or name.startswith("README")
            )
        )
        if admitted:
            records.append(
                {
                    "path": relative.as_posix(),
                    "bytes": path.stat().st_size,
                    "kappa": cid_file(path),
                }
            )
    records.sort(key=lambda record: str(record["path"]).encode("utf-8"))
    if not records:
        raise ValueError("C1-SB3 checkpoint tree has no Rust-admitted files")
    identity = {"schema": RUST_CHECKPOINT_TREE_SCHEMA, "files": records}
    encoded = json.dumps(
        identity,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
    ).encode("utf-8")
    return {
        "schema": RUST_CHECKPOINT_TREE_SCHEMA,
        "checkpoint_tree_cid": cid_bytes(encoded),
        "files": records,
    }


def _build_attended_adapter_artifact(
    *,
    representation_update: str,
    checkpoint_manifest: Mapping[str, Any],
    checkpoint_tree_cid: str,
    dataset: Mapping[str, Any],
    run_contract: Mapping[str, Any],
    result: Mapping[str, Any],
) -> dict[str, Any]:
    commitments = list(dataset["product_probe_commitments"])
    if len(commitments) != 4 or len(set(commitments)) != 4:
        raise ValueError("C1-SB3 requires exactly four distinct product commitments")
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
            "relation_input_policy": RELATION_INPUT_TEMPLATE,
            "dataset_cid": dataset["dataset_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "run_contract_cid": run_contract["run_contract_cid"],
            "training_result_cid": result["result_cid"],
            "product_probe_commitments": commitments,
        },
        "artifact_cid",
    )


def _persist_positive_preflight_delivery(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: Mapping[str, Any],
    adapter: R4AttendedRelationAdapter,
    representation_update: str,
    result: Mapping[str, Any],
    dataset: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> dict[str, Any]:
    if not str(result.get("terminal", "")).startswith("PASS_"):
        raise ValueError("C1-SB3 positive delivery requires a passing preflight result")

    relative_paths: list[str] = []
    if representation_update == REPRESENTATION_UPDATE:
        checkpoint = root / POSITIVE_CHECKPOINT_DIRECTORY
        if checkpoint.exists():
            raise FileExistsError(f"C1-SB3 positive checkpoint already exists: {checkpoint}")
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
            f"{POSITIVE_CHECKPOINT_DIRECTORY}/{record['path']}"
            for record in checkpoint_manifest["artifacts"]
        )
        relative_paths.append(
            f"{POSITIVE_CHECKPOINT_DIRECTORY}/export-manifest.json"
        )
    elif representation_update == FROZEN_READOUT_UPDATE:
        checkpoint = predecessor
        checkpoint_manifest = dict(predecessor_manifest)
    else:
        raise ValueError("C1-SB3 positive delivery has an unknown representation update")

    if (
        checkpoint_manifest.get("config_cid") != EXPECTED_CONFIG_CID
        or checkpoint_manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
        or checkpoint_manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract()
    ):
        raise ValueError("C1-SB3 positive checkpoint identity drifted")
    if representation_update == REPRESENTATION_UPDATE:
        if checkpoint_manifest.get("weights_cid") == EXPECTED_WEIGHTS_CID:
            raise ValueError("C1-SB3 LoRA checkpoint did not change the predecessor weights")
        if checkpoint_manifest.get("training_result_cid") != result["result_cid"]:
            raise ValueError("C1-SB3 checkpoint does not bind the preflight result")
    elif checkpoint_manifest.get("weights_cid") != EXPECTED_WEIGHTS_CID:
        raise ValueError("C1-SB3 frozen readout changed the predecessor weights")

    checkpoint_tree = _rust_checkpoint_tree_binding(checkpoint)
    artifact = _build_attended_adapter_artifact(
        representation_update=representation_update,
        checkpoint_manifest=checkpoint_manifest,
        checkpoint_tree_cid=checkpoint_tree["checkpoint_tree_cid"],
        dataset=dataset,
        run_contract=run_contract,
        result=result,
    )
    _write_or_verify(root / ATTENDED_ADAPTER_ARTIFACT, artifact)
    relative_paths.append(ATTENDED_ADAPTER_ARTIFACT)
    return {
        "artifact": artifact,
        "checkpoint": checkpoint,
        "checkpoint_manifest": checkpoint_manifest,
        "checkpoint_tree": checkpoint_tree,
        "relative_paths": relative_paths,
    }


def _finalize_preflight_result(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: Mapping[str, Any],
    adapter: R4AttendedRelationAdapter | None,
    representation_update: str,
    result: dict[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    delivery = None
    if str(result["terminal"]).startswith("PASS_"):
        if adapter is None:
            raise ValueError("C1-SB3 passing preflight has no adapter to persist")
        delivery = _persist_positive_preflight_delivery(
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
        "attended-relation-dataset.json",
        "attended-relation-preflight.json",
        "attended-relation-census.json",
        "training-view-manifest.json",
        "run-contract.json",
        "preflight-started.json",
        "preflight-result.json",
    ]
    payload: dict[str, Any] = {
        "schema": PREFLIGHT_MANIFEST_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "terminal": result["terminal"],
        "representation_update": representation_update,
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "run_contract_cid": run_contract["run_contract_cid"],
        "result_cid": result["result_cid"],
        "product_status": "UNOPENED_NOT_RUN",
    }
    if delivery is not None:
        artifact = delivery["artifact"]
        checkpoint_manifest = delivery["checkpoint_manifest"]
        checkpoint_tree = delivery["checkpoint_tree"]
        payload.update(
            {
                "adapter_artifact_cid": artifact["artifact_cid"],
                "checkpoint_tree_cid": checkpoint_tree["checkpoint_tree_cid"],
                "model_weights_cid": checkpoint_manifest["weights_cid"],
                "config_cid": checkpoint_manifest["config_cid"],
                "tokenizer_cid": checkpoint_manifest["tokenizer_cid"],
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


def _run_contract(
    *,
    predecessor_manifest: Mapping[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    config: AttendedRelationCampaignConfig,
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
            "adapter_contract": AttendedRelationAdapterConfig().as_contract(),
            "optimizer_contract": config.as_contract(),
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "census_cid": dataset["census_cid"],
            "product_probes_cid": dataset["product_probes_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
            "product_text_access": "DENIED_TO_PYTHON_TRAINER_AND_EVALUATOR",
            "mechanism": (
                "rank-8 alpha-8 dropout-zero LoRA on q/k/v/o in all six layers; "
                "fixed tied yes/no verbalizer; no learned head"
            ),
            "preflight_gate": {
                "fit_records": 126,
                "sealed_records": 63,
                "all_semantic_metrics": "exact",
                "candidate_order_mapping": "exact",
                "requires_corrected_untrained_sealed_miss": True,
                "wall_ceiling_seconds": 600.0,
            },
            "if_positive": "emit research-only merged checkpoint for Rust parity",
            "if_negative": "stop before Rust parity/full/development/product without retry",
            "implementation": trainer_implementation_contract(),
        },
        "run_contract_cid",
    )


def run_attended_relation_preflight(
    root: Path,
    *,
    predecessor: Path,
    config: AttendedRelationCampaignConfig = AttendedRelationCampaignConfig(),
) -> dict[str, Any]:
    """Run exactly one C1-SB3 cheap gate without opening product text."""
    config.validate()
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    dataset, preflight, training_manifest = _load_training_view(root)
    result_path = root / "preflight-result.json"
    if result_path.exists() or (root / "preflight-started.json").exists():
        raise FileExistsError("the sole C1-SB3 preflight was already started")
    run_contract = _run_contract(
        predecessor_manifest=predecessor_manifest,
        dataset=dataset,
        preflight=preflight,
        training_manifest=training_manifest,
        config=config,
    )
    _write_or_verify(root / "run-contract.json", run_contract)
    atomic_write_json(
        root / "preflight-started.json",
        {
            "schema": RUN_SCHEMA,
            "phase": "SOLE_C1_SB3_PREFLIGHT_STARTED",
            "run_contract_cid": run_contract["run_contract_cid"],
        },
    )

    device = require_mps(config.seed)
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    validate_tokenizer_contract(tokenizer)
    model, base_state = _load_base_model(predecessor)
    adapter = R4AttendedRelationAdapter(model).to(device)
    fit_records = list(preflight["fit"])
    sealed_records = list(preflight["sealed"])
    untrained_sealed_detailed = _evaluate_records(
        adapter,
        sealed_records,
        tokenizer=tokenizer,
        device=device,
        batch_size=config.evaluation_batch_size,
        include_records=True,
    )
    untrained_sealed = _without_record_evaluations(untrained_sealed_detailed)

    if _all_exact(untrained_sealed):
        terminal = "PASS_FROZEN_RELATION_READOUT"
        optimization: dict[str, Any] = {"status": "SKIPPED_SIMPLER_MECHANISM_ALREADY_EXACT"}
        fit_metrics_detailed = _evaluate_records(
            adapter,
            fit_records,
            tokenizer=tokenizer,
            device=device,
            batch_size=config.evaluation_batch_size,
            include_records=True,
        )
        sealed_metrics_detailed = untrained_sealed_detailed
        delta = {
            "status": "NOT_APPLICABLE_FROZEN_READOUT",
            "passed": True,
            "representation_update": FROZEN_READOUT_UPDATE,
        }
        representation_update = FROZEN_READOUT_UPDATE
    else:
        fit_examples = relation_examples_from_records(fit_records)
        fit_dataset = EncodedRelationDataset(fit_examples, tokenizer)
        try:
            optimization = fit_attended_relation_adapter(
                adapter,
                fit_dataset,
                config=AdapterFitConfig(
                    seed=config.seed,
                    optimizer_steps=config.preflight_optimizer_steps,
                    batch_size=config.candidate_batch_size,
                    learning_rate=config.learning_rate,
                    adam_beta1=config.adam_beta1,
                    adam_beta2=config.adam_beta2,
                    adam_epsilon=config.adam_epsilon,
                    weight_decay=config.weight_decay,
                    gradient_clip=config.gradient_clip,
                    eta_probe_step=config.eta_probe_step,
                    wall_ceiling_seconds=config.preflight_wall_ceiling_seconds,
                ),
            )
        except AttendedRelationWallBudgetExceeded as error:
            delta = _delta_contract(adapter, base_state)
            result = _canonical_with_cid(
                {
                    "schema": PREFLIGHT_RESULT_SCHEMA,
                    "issue": ISSUE,
                    "policy": POLICY,
                    "terminal": "UNAVAILABLE_PREFLIGHT_BUDGET",
                    "representation_update": REPRESENTATION_UPDATE,
                    "run_contract_cid": run_contract["run_contract_cid"],
                    "dataset_cid": dataset["dataset_cid"],
                    "preflight_cid": preflight["preflight_cid"],
                    "training_view_manifest_cid": training_manifest["manifest_cid"],
                    "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                    "tokenizer_cid": EXPECTED_TOKENIZER_CID,
                    "untrained_sealed_metrics": untrained_sealed,
                    "optimization": error.as_result(),
                    "fit_metrics": "NOT_RUN",
                    "sealed_metrics": "NOT_RUN",
                    "controls": "NOT_RUN",
                    "delta_audit": delta,
                    "rust_parity": "NOT_RUN",
                    "full_fit": "NOT_RUN",
                    "development": "NOT_RUN",
                    "product": "NOT_RUN",
                },
                "result_cid",
            )
            manifest, _ = _finalize_preflight_result(
                root,
                predecessor=predecessor,
                predecessor_manifest=predecessor_manifest,
                adapter=None,
                representation_update=REPRESENTATION_UPDATE,
                result=result,
                dataset=dataset,
                preflight=preflight,
                training_manifest=training_manifest,
                run_contract=run_contract,
            )
            return {
                **result,
                "manifest_cid": manifest["manifest_cid"],
                "product": "UNOPENED_NOT_RUN",
            }
        fit_metrics_detailed = _evaluate_records(
            adapter,
            fit_records,
            tokenizer=tokenizer,
            device=device,
            batch_size=config.evaluation_batch_size,
            include_records=True,
        )
        sealed_metrics_detailed = _evaluate_records(
            adapter,
            sealed_records,
            tokenizer=tokenizer,
            device=device,
            batch_size=config.evaluation_batch_size,
            include_records=True,
        )
        delta = _delta_contract(adapter, base_state)
        corrected_miss = not _all_exact(untrained_sealed) and _all_exact(
            sealed_metrics_detailed
        )
        passed = (
            _all_exact(fit_metrics_detailed)
            and _all_exact(sealed_metrics_detailed)
            and corrected_miss
            and delta["passed"]
        )
        terminal = (
            "PASS_REPRESENTATION_TRANSFER_PREFLIGHT_AWAITING_RUST_PARITY"
            if passed
            else "FAIL_REPRESENTATION_TRANSFER_PREFLIGHT"
        )
        representation_update = REPRESENTATION_UPDATE

    fit_metrics = _without_record_evaluations(fit_metrics_detailed)
    sealed_metrics = _without_record_evaluations(sealed_metrics_detailed)
    reverse_metrics = _evaluate_records(
        adapter,
        _reversed_records([*fit_records, *sealed_records]),
        tokenizer=tokenizer,
        device=device,
        batch_size=config.evaluation_batch_size,
    )
    controls: dict[str, Any] = {
        "candidate_order_mapping_exact": _all_exact(reverse_metrics),
        **_named_control_metrics(fit_metrics_detailed, sealed_metrics_detailed),
    }
    controls["passed"] = all(
        value is True for key, value in controls.items() if key.endswith("_exact")
    )
    if terminal.startswith("PASS_") and not controls["passed"]:
        terminal = "FAIL_REPRESENTATION_TRANSFER_PREFLIGHT"

    result = _canonical_with_cid(
        {
            "schema": PREFLIGHT_RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": terminal,
            "representation_update": representation_update,
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "untrained_sealed_metrics": untrained_sealed,
            "optimization": {
                key: value for key, value in optimization.items() if key != "elapsed_seconds"
            },
            "fit_metrics": fit_metrics,
            "sealed_metrics": sealed_metrics,
            "controls": controls,
            "delta_audit": delta,
            "rust_parity": "NOT_RUN",
            "full_fit": "NOT_RUN",
            "development": "NOT_RUN",
            "product": "NOT_RUN",
        },
        "result_cid",
    )
    manifest, delivery = _finalize_preflight_result(
        root,
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        adapter=adapter,
        representation_update=representation_update,
        result=result,
        dataset=dataset,
        preflight=preflight,
        training_manifest=training_manifest,
        run_contract=run_contract,
    )
    response = {
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
                "checkpoint_tree_cid": delivery["checkpoint_tree"][
                    "checkpoint_tree_cid"
                ],
                "model_weights_cid": delivery["checkpoint_manifest"]["weights_cid"],
                "checkpoint": str(delivery["checkpoint"]),
            }
        )
    return response
