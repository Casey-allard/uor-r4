"""Durable pre-campaign admission for the one #1014 main training run."""

from __future__ import annotations

import json
import math
import struct
from pathlib import Path
from typing import Any

from .constants import (
    EXPORT_MANIFEST_SCHEMA,
    FROZEN_MODEL_CONFIG,
    MAIN_ADMISSION_MANIFEST_SCHEMA,
    PREFIX_LOGIT_ABS_TOLERANCE,
    PREFIX_PARITY_TOKENS,
    PYTHON_PREFIX_LOGITS_SCHEMA,
    RUST_QUALIFICATION_REPORT_SCHEMA,
    SMOKE_MANIFEST_SCHEMA,
    SMOKE_SCHEMA,
)
from .data import TOKEN_RELATIVE_PATHS, load_training_view_manifest
from .provenance import (
    atomic_write,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    tree_cid,
    verify_bound_manifest,
    write_bound_manifest,
)


SMOKE_MANIFEST_PATH = Path("smoke/smoke-manifest.json")
SMOKE_RESULT_PATH = Path("smoke/smoke-result.json")
SMOKE_EXPORT_MANIFEST_PATH = Path("smoke/export/export-manifest.json")
SMOKE_PREFIX_PATH = Path("smoke/python-prefix-logits.json")
IMPORTED_RUST_QUALIFICATION_PATH = Path("admission/rust-smoke-qualification.json")
MAIN_ADMISSION_MANIFEST_PATH = Path("admission/main-admission-manifest.json")

_SMOKE_ARTIFACT_PATHS = {
    "smoke/smoke-result.json",
    "smoke/python-prefix-logits.json",
    "smoke/export/config.json",
    "smoke/export/model.safetensors",
    "smoke/export/tokenizer.json",
    "smoke/export/training-result.json",
    "smoke/export/export-manifest.json",
}
_SMOKE_EXPORT_ARTIFACT_PATHS = {
    "config.json",
    "model.safetensors",
    "tokenizer.json",
    "training-result.json",
}
_SMOKE_REUSE_INVARIANT_PATHS = {
    "pyproject.toml",
    "uv.lock",
    "src/r4_softmax_trainer/__init__.py",
    "src/r4_softmax_trainer/__main__.py",
    "src/r4_softmax_trainer/constants.py",
    "src/r4_softmax_trainer/data.py",
    "src/r4_softmax_trainer/export.py",
    "src/r4_softmax_trainer/model.py",
    "src/r4_softmax_trainer/paths.py",
    "src/r4_softmax_trainer/provenance.py",
}
_POST_SMOKE_LIFECYCLE_PATHS = {
    "src/r4_softmax_trainer/admission.py",
    "src/r4_softmax_trainer/capacity.py",
    "src/r4_softmax_trainer/capacity_data.py",
    "src/r4_softmax_trainer/capacity_finalize.py",
    "src/r4_softmax_trainer/cli.py",
    "src/r4_softmax_trainer/continuation.py",
    "src/r4_softmax_trainer/continuation_data.py",
    "src/r4_softmax_trainer/finalize.py",
    "src/r4_softmax_trainer/train.py",
}


def _load_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {label} as UTF-8 JSON: {path}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be a JSON object")
    return value


def _require_exact_bool(value: object, *, label: str) -> bool:
    if type(value) is not bool:
        raise ValueError(f"{label} must be a JSON boolean")
    return value


def _require_cid(value: object, *, label: str) -> str:
    if not isinstance(value, str) or not value.startswith("blake3:") or len(value) != 71:
        raise ValueError(f"{label} must be a BLAKE3 CID")
    try:
        int(value[7:], 16)
    except ValueError as error:
        raise ValueError(f"{label} must be a BLAKE3 CID") from error
    return value


def _require_self_cid(value: dict[str, Any], *, field: str, label: str) -> str:
    expected = _require_cid(value.get(field), label=f"{label}.{field}")
    unsigned = dict(value)
    unsigned.pop(field, None)
    actual = cid_bytes(canonical_json_bytes(unsigned))
    if expected != actual:
        raise ValueError(f"{label} {field} does not reproduce")
    return expected


def _as_f32(value: int | float) -> float:
    """Reproduce Rust's JSON-f32 parse before comparing emitted parity evidence."""
    return struct.unpack("<f", struct.pack("<f", float(value)))[0]


def _artifact_paths(manifest: dict[str, Any], *, label: str) -> set[str]:
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError(f"{label} has no artifact records")
    paths: set[str] = set()
    for record in records:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            raise ValueError(f"{label} has an invalid artifact record")
        path = str(record["path"])
        if path in paths:
            raise ValueError(f"{label} repeats artifact path {path}")
        paths.add(path)
    return paths


def _trainer_records(contract: dict[str, Any], *, label: str) -> dict[str, dict[str, Any]]:
    records = contract.get("files")
    if not isinstance(records, list):
        raise ValueError(f"{label} has no implementation-file records")
    by_path: dict[str, dict[str, Any]] = {}
    for record in records:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            raise ValueError(f"{label} has an invalid implementation-file record")
        path = str(record["path"])
        if path in by_path:
            raise ValueError(f"{label} repeats implementation path {path}")
        by_path[path] = record
    if contract.get("tree_cid") != tree_cid(list(by_path.values())):
        raise ValueError(f"{label} implementation tree CID does not reproduce")
    return by_path


def _smoke_reuse_transition(
    smoke_trainer: dict[str, Any], campaign_trainer: dict[str, Any]
) -> dict[str, Any]:
    """Permit only the reviewed post-smoke admission/lifecycle delta."""
    smoke_records = _trainer_records(smoke_trainer, label="smoke trainer")
    campaign_records = _trainer_records(campaign_trainer, label="campaign trainer")
    known_paths = _SMOKE_REUSE_INVARIANT_PATHS | _POST_SMOKE_LIFECYCLE_PATHS
    observed_paths = set(smoke_records) | set(campaign_records)
    unknown = sorted(observed_paths - known_paths)
    if unknown:
        raise ValueError(f"unclassified trainer files prevent smoke reuse: {unknown}")
    for path in sorted(_SMOKE_REUSE_INVARIANT_PATHS):
        if smoke_records.get(path) != campaign_records.get(path):
            raise ValueError(f"smoke-reuse invariant changed after smoke: {path}")
    changed_paths = sorted(
        path
        for path in observed_paths
        if smoke_records.get(path) != campaign_records.get(path)
    )
    if not set(changed_paths).issubset(_POST_SMOKE_LIFECYCLE_PATHS):
        raise ValueError("post-smoke changes exceed the reviewed lifecycle boundary")
    invariant_records = [campaign_records[path] for path in sorted(_SMOKE_REUSE_INVARIANT_PATHS)]
    transition: dict[str, Any] = {
        "schema": "uor-r4-softmax-trainer-smoke-reuse-transition/1",
        "smoke_trainer_tree_cid": smoke_trainer["tree_cid"],
        "campaign_trainer_tree_cid": campaign_trainer["tree_cid"],
        "invariant_paths": sorted(_SMOKE_REUSE_INVARIANT_PATHS),
        "invariant_tree_cid": tree_cid(invariant_records),
        "allowed_lifecycle_delta_paths": sorted(_POST_SMOKE_LIFECYCLE_PATHS),
        "observed_delta_paths": changed_paths,
        "scope": "existing smoke reused only across reviewed admission and lifecycle hardening; model, data, export, dependency, and provenance files are byte-identical",
    }
    transition["transition_cid"] = cid_bytes(canonical_json_bytes(transition))
    return transition


def _require_identity(
    observed: dict[str, Any], expected: dict[str, Any], fields: tuple[str, ...], *, label: str
) -> None:
    for field in fields:
        if observed.get(field) != expected.get(field):
            raise ValueError(f"{label} {field} identity mismatch")


def _verify_prefix_fixture(path: Path) -> dict[str, Any]:
    fixture = _load_json_object(path, "Python prefix fixture")
    if fixture.get("schema") != PYTHON_PREFIX_LOGITS_SCHEMA:
        raise ValueError("unsupported Python prefix fixture schema")
    _require_self_cid(fixture, field="result_cid", label="Python prefix fixture")
    _require_cid(fixture.get("weights_cid"), label="Python prefix fixture.weights_cid")
    _require_cid(
        fixture.get("token_store_cid"), label="Python prefix fixture.token_store_cid"
    )
    token_ids = fixture.get("prefix_token_ids")
    if (
        not isinstance(token_ids, list)
        or len(token_ids) != PREFIX_PARITY_TOKENS
        or any(type(token) is not int or not 0 <= token < FROZEN_MODEL_CONFIG.vocab_size for token in token_ids)
    ):
        raise ValueError("Python prefix fixture must contain exactly 32 valid token IDs")
    if fixture.get("maximum_absolute_logit_delta_limit") != PREFIX_LOGIT_ABS_TOLERANCE:
        raise ValueError("Python prefix fixture has a non-frozen logit tolerance")
    for arm_name in ("enabled", "attention_off"):
        arm = fixture.get(arm_name)
        if not isinstance(arm, dict):
            raise ValueError(f"Python prefix fixture has no {arm_name} arm")
        logits = arm.get("logits")
        if (
            not isinstance(logits, list)
            or len(logits) != FROZEN_MODEL_CONFIG.vocab_size
            or any(
                isinstance(value, bool)
                or not isinstance(value, (int, float))
                or not math.isfinite(float(value))
                for value in logits
            )
        ):
            raise ValueError(f"Python {arm_name} logits must be 4096 finite numbers")
        reproduced_top1 = max(range(len(logits)), key=lambda index: (float(logits[index]), -index))
        if arm.get("top1_token_id") != reproduced_top1:
            raise ValueError(f"Python {arm_name} top-1 does not reproduce")
    return fixture


def _verify_smoke_bundle(root: Path, training_view: dict[str, Any]) -> dict[str, Any]:
    smoke_manifest = verify_bound_manifest(root / SMOKE_MANIFEST_PATH, artifact_root=root)
    if smoke_manifest.get("schema") != SMOKE_MANIFEST_SCHEMA:
        raise ValueError("unsupported smoke manifest schema")
    if smoke_manifest.get("terminal") != "PASS_EXPORT_AWAITING_RUST_PARITY":
        raise ValueError("smoke export has not reached its Rust-parity boundary")
    if _artifact_paths(smoke_manifest, label="smoke manifest") != _SMOKE_ARTIFACT_PATHS:
        raise ValueError("smoke manifest does not bind the exact required artifact set")

    smoke_result = _load_json_object(root / SMOKE_RESULT_PATH, "smoke result")
    if smoke_result.get("schema") != SMOKE_SCHEMA or smoke_result.get("terminal") != "PASS":
        raise ValueError("64-sequence trainer smoke did not pass")
    _require_self_cid(smoke_result, field="result_cid", label="smoke result")
    if smoke_result.get("sequences") != 64 or smoke_result.get("context") != 256:
        raise ValueError("smoke did not use exactly 64 context-256 sequences")
    if (
        smoke_result.get("required_reduction_fraction") != 0.80
        or smoke_result.get("wall_ceiling_seconds") != 300.0
    ):
        raise ValueError("smoke result does not carry the frozen admission thresholds")
    reduction = smoke_result.get("loss_reduction_fraction")
    if (
        isinstance(reduction, bool)
        or not isinstance(reduction, (int, float))
        or not math.isfinite(float(reduction))
        or float(reduction) < 0.80
    ):
        raise ValueError("smoke loss reduction is below the required 80 percent")
    initial_loss = smoke_result.get("initial_loss")
    final_loss = smoke_result.get("final_loss")
    if (
        isinstance(initial_loss, bool)
        or isinstance(final_loss, bool)
        or not isinstance(initial_loss, (int, float))
        or not isinstance(final_loss, (int, float))
        or not math.isfinite(float(initial_loss))
        or not math.isfinite(float(final_loss))
        or float(initial_loss) <= 0
        or float(final_loss) < 0
        or not math.isclose(
            float(reduction),
            1.0 - float(final_loss) / float(initial_loss),
            rel_tol=0.0,
            abs_tol=1e-12,
        )
    ):
        raise ValueError("smoke loss reduction does not reproduce from its losses")
    elapsed = smoke_result.get("elapsed_seconds")
    if (
        isinstance(elapsed, bool)
        or not isinstance(elapsed, (int, float))
        or not math.isfinite(float(elapsed))
        or not 0 <= float(elapsed) <= 300.0
    ):
        raise ValueError("smoke did not complete inside its five-minute wall")
    smoke_contract = smoke_result.get("smoke_contract")
    if not isinstance(smoke_contract, dict):
        raise ValueError("smoke result has no frozen smoke contract")
    smoke_contract_cid = cid_bytes(canonical_json_bytes(smoke_contract))
    if smoke_result.get("smoke_contract_cid") != smoke_contract_cid:
        raise ValueError("smoke contract CID does not reproduce")
    contract_wall = smoke_contract.get("wall_ceiling_seconds")
    if (
        smoke_contract.get("sequences") != 64
        or smoke_contract.get("context") != 256
        or smoke_contract.get("required_loss_reduction_fraction") != 0.80
        or isinstance(contract_wall, bool)
        or not isinstance(contract_wall, (int, float))
        or not 0 < float(contract_wall) <= 300.0
        or float(elapsed) > float(contract_wall)
    ):
        raise ValueError("smoke contract does not freeze the exact admission gate")
    trainer_contract = smoke_contract.get("trainer_implementation")
    if not isinstance(trainer_contract, dict):
        raise ValueError("smoke contract does not bind the trainer implementation")
    trainer_tree_cid = _require_cid(
        trainer_contract.get("tree_cid"), label="smoke trainer implementation tree"
    )

    export_manifest = verify_bound_manifest(
        root / SMOKE_EXPORT_MANIFEST_PATH, artifact_root=root / "smoke" / "export"
    )
    if export_manifest.get("schema") != EXPORT_MANIFEST_SCHEMA:
        raise ValueError("unsupported smoke export manifest schema")
    if (
        _artifact_paths(export_manifest, label="smoke export manifest")
        != _SMOKE_EXPORT_ARTIFACT_PATHS
    ):
        raise ValueError("smoke export manifest does not bind the exact HF snapshot")
    prefix = _verify_prefix_fixture(root / SMOKE_PREFIX_PATH)

    common_fields = ("dataset_manifest_cid", "training_view_manifest_cid", "split_policy_cid")
    view_identity = {
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
    }
    _require_identity(smoke_manifest, view_identity, common_fields, label="smoke/training view")
    _require_identity(smoke_result, view_identity, common_fields, label="smoke result/training view")
    _require_identity(smoke_contract, view_identity, common_fields, label="smoke contract/training view")
    _require_identity(export_manifest, view_identity, common_fields, label="smoke export/training view")
    if smoke_manifest.get("smoke_result_cid") != smoke_result.get("result_cid"):
        raise ValueError("smoke manifest/result identity mismatch")
    if smoke_manifest.get("smoke_contract_cid") != smoke_contract_cid:
        raise ValueError("smoke manifest/contract identity mismatch")
    if smoke_manifest.get("export_manifest_cid") != export_manifest.get("manifest_cid"):
        raise ValueError("smoke manifest/export identity mismatch")
    if smoke_manifest.get("weights_cid") != export_manifest.get("weights_cid"):
        raise ValueError("smoke manifest/weights identity mismatch")
    if smoke_manifest.get("prefix_result_cid") != prefix.get("result_cid"):
        raise ValueError("smoke manifest/prefix identity mismatch")
    if export_manifest.get("run_contract_cid") != smoke_contract_cid:
        raise ValueError("smoke export does not bind the smoke contract")
    if export_manifest.get("training_result_cid") != smoke_result.get("result_cid"):
        raise ValueError("smoke export does not bind the smoke result")
    if export_manifest.get("selected_checkpoint_cid") != export_manifest.get("weights_cid"):
        raise ValueError("smoke checkpoint identity must be its exported weights")
    if prefix.get("weights_cid") != export_manifest.get("weights_cid"):
        raise ValueError("smoke prefix fixture belongs to different weights")
    if prefix.get("token_store_cid") != cid_file(root / TOKEN_RELATIVE_PATHS["train"]):
        raise ValueError("smoke prefix fixture belongs to a different train token store")
    return {
        "manifest": smoke_manifest,
        "result": smoke_result,
        "contract": smoke_contract,
        "contract_cid": smoke_contract_cid,
        "trainer_tree_cid": trainer_tree_cid,
        "trainer_contract": trainer_contract,
        "export": export_manifest,
        "prefix": prefix,
        "prefix_file_cid": cid_file(root / SMOKE_PREFIX_PATH),
    }


def _require_passed_parity(
    report: dict[str, Any], bundle: dict[str, Any], *, arm_name: str, policy: str
) -> None:
    fixture_arm = bundle["prefix"][arm_name]
    report_arm = report.get(arm_name)
    parity = report.get(f"{arm_name}_prefix_parity")
    if not isinstance(report_arm, dict) or not isinstance(parity, dict):
        raise ValueError(f"Rust qualification is missing the {arm_name} arm")
    if report_arm.get("attention_output_policy") != policy:
        raise ValueError(f"Rust {arm_name} arm used the wrong output policy")
    if parity.get("attention_output_policy") != policy:
        raise ValueError(f"Rust {arm_name} parity used the wrong output policy")
    for field in ("identical_top1", "maximum_absolute_logit_delta_within_limit", "passed"):
        if not _require_exact_bool(parity.get(field), label=f"Rust {arm_name} parity.{field}"):
            raise ValueError(f"Rust {arm_name} parity did not pass {field}")
    delta = parity.get("maximum_absolute_logit_delta")
    if (
        isinstance(delta, bool)
        or not isinstance(delta, (int, float))
        or not math.isfinite(float(delta))
        or float(delta) > PREFIX_LOGIT_ABS_TOLERANCE
        or parity.get("maximum_absolute_logit_delta_limit") != PREFIX_LOGIT_ABS_TOLERANCE
    ):
        raise ValueError(f"Rust {arm_name} parity exceeds the frozen tolerance")
    if parity.get("python_top1_token_id") != fixture_arm.get("top1_token_id"):
        raise ValueError(f"Rust {arm_name} parity binds a different Python top-1")
    python_logits = parity.get("python_logits")
    rust_logits = parity.get("rust_logits")
    fixture_logits = fixture_arm.get("logits")
    if (
        not isinstance(python_logits, list)
        or not isinstance(fixture_logits, list)
        or len(python_logits) != len(fixture_logits)
        or any(
            _as_f32(observed) != _as_f32(expected)
            for observed, expected in zip(python_logits, fixture_logits, strict=True)
        )
    ):
        raise ValueError(f"Rust {arm_name} parity binds different Python logits")
    if (
        not isinstance(rust_logits, list)
        or len(rust_logits) != FROZEN_MODEL_CONFIG.vocab_size
        or any(
            isinstance(value, bool)
            or not isinstance(value, (int, float))
            or not math.isfinite(float(value))
            for value in rust_logits
        )
    ):
        raise ValueError(f"Rust {arm_name} parity logits must be 4096 finite numbers")
    rust_top1 = max(
        range(len(rust_logits)), key=lambda index: (float(rust_logits[index]), -index)
    )
    if parity.get("rust_top1_token_id") != rust_top1:
        raise ValueError(f"Rust {arm_name} parity top-1 does not reproduce")
    reproduced_delta = max(
        abs(_as_f32(python) - _as_f32(rust))
        for python, rust in zip(python_logits, rust_logits, strict=True)
    )
    if float(delta) != reproduced_delta:
        raise ValueError(f"Rust {arm_name} maximum logit delta does not reproduce")
    if parity.get("identical_top1") != (
        parity.get("python_top1_token_id") == rust_top1
    ):
        raise ValueError(f"Rust {arm_name} identical-top1 flag does not reproduce")
    if report_arm.get("top1_token_id") != parity.get("rust_top1_token_id"):
        raise ValueError(f"Rust {arm_name} arm/parity top-1 mismatch")
    for field in ("policy_cid", "output_cid", "audit_cid"):
        _require_cid(report_arm.get(field), label=f"Rust {arm_name}.{field}")
    audit = report_arm.get("audit")
    if not isinstance(audit, dict):
        raise ValueError(f"Rust {arm_name} arm has no audit")
    if (
        audit.get("sessions") != 1
        or audit.get("positions_per_session") != PREFIX_PARITY_TOKENS
        or audit.get("total_positions") != PREFIX_PARITY_TOKENS
        or audit.get("selected_layer_count") != FROZEN_MODEL_CONFIG.num_hidden_layers
        or audit.get("future_reads") != 0
    ):
        raise ValueError(f"Rust {arm_name} arm did not audit the exact 32-token, six-layer path")
    if not _require_exact_bool(
        audit.get("all_layers_selected"), label=f"Rust {arm_name}.all_layers_selected"
    ):
        raise ValueError(f"Rust {arm_name} arm audit failed all_layers_selected")
    for field in (
        "causal_audits_exact",
        "projection_audits_exact",
        "r4_audits_exact",
        "output_policy_audits_exact",
    ):
        if type(audit.get(field)) is not int or audit.get(field) != 1:
            raise ValueError(f"Rust {arm_name} arm audit failed {field}")
    applications = PREFIX_PARITY_TOKENS * FROZEN_MODEL_CONFIG.num_hidden_layers
    if (
        audit.get("output_policy_applications") != applications
        or audit.get("output_lanes") != applications * FROZEN_MODEL_CONFIG.hidden_size
        or audit.get("applications_by_layer")
        != [PREFIX_PARITY_TOKENS] * FROZEN_MODEL_CONFIG.num_hidden_layers
    ):
        raise ValueError(f"Rust {arm_name} arm has incomplete output-policy coverage")
    _require_cid(audit.get("state_ledger_cid"), label=f"Rust {arm_name}.state_ledger_cid")
    if arm_name == "enabled":
        if audit.get("enabled_applications") != applications or audit.get("zeroed_applications") != 0:
            raise ValueError("Rust enabled arm did not retain every post-Wo output")
    elif (
        audit.get("enabled_applications") != 0
        or audit.get("zeroed_applications") != applications
        or audit.get("nonzero_lanes_after_policy") != 0
    ):
        raise ValueError("Rust attention-off arm did not zero every post-Wo output")


def _verify_rust_qualification(
    report: dict[str, Any], *, report_file_cid: str, bundle: dict[str, Any]
) -> None:
    if report.get("schema") != RUST_QUALIFICATION_REPORT_SCHEMA or report.get("issue") != 1014:
        raise ValueError("unsupported Rust qualification report")
    _require_cid(report.get("decision_cid"), label="Rust qualification.decision_cid")
    if not _require_exact_bool(
        report.get("qualification_passed"), label="Rust qualification.qualification_passed"
    ):
        raise ValueError("Rust qualification did not pass")
    checkpoint = report.get("checkpoint")
    provenance = report.get("provenance")
    evaluation = report.get("evaluation_input")
    shape = report.get("model_shape")
    if not all(isinstance(value, dict) for value in (checkpoint, provenance, evaluation, shape)):
        raise ValueError("Rust qualification lacks checkpoint/provenance/input/shape evidence")
    export = bundle["export"]
    _require_identity(
        provenance,
        export,
        (
            "dataset_manifest_cid",
            "training_view_manifest_cid",
            "split_policy_cid",
            "run_contract_cid",
            "training_result_cid",
            "selected_checkpoint_cid",
            "config_cid",
            "tokenizer_cid",
            "weights_cid",
        ),
        label="Rust qualification/smoke export",
    )
    if provenance.get("export_manifest_cid") != export.get("manifest_cid"):
        raise ValueError("Rust qualification binds a different export manifest")
    if provenance.get("export_tree_cid") != export.get("tree_cid"):
        raise ValueError("Rust qualification binds a different export tree")
    if provenance.get("reveal_manifest_cid") is not None or provenance.get("reveal_tree_cid") is not None:
        raise ValueError("pre-campaign Rust qualification must not open the sealed reveal")
    _require_identity(
        checkpoint,
        export,
        ("config_cid", "tokenizer_cid", "weights_cid"),
        label="Rust checkpoint/smoke export",
    )
    _require_cid(
        checkpoint.get("checkpoint_tree_cid"), label="Rust checkpoint.checkpoint_tree_cid"
    )
    expected_shape = {
        "dimension": 288,
        "hidden_dimension": 768,
        "layers": 6,
        "query_heads": 6,
        "key_value_heads": 6,
        "head_size": 48,
        "vocabulary": 4096,
        "sequence_capacity": PREFIX_PARITY_TOKENS,
    }
    if shape != expected_shape:
        raise ValueError("Rust qualification used a non-frozen model shape")
    prefix = bundle["prefix"]
    if (
        evaluation.get("token_store_cid") != prefix.get("token_store_cid")
        or evaluation.get("python_prefix_logits_cid") != bundle["prefix_file_cid"]
        or evaluation.get("python_prefix_result_cid") != prefix.get("result_cid")
        or evaluation.get("prefix_token_ids") != prefix.get("prefix_token_ids")
        or not _require_exact_bool(
            evaluation.get("sources_unchanged_across_execution"),
            label="Rust qualification.evaluation_input.sources_unchanged_across_execution",
        )
    ):
        raise ValueError("Rust qualification evaluated different or mutable prefix inputs")
    _require_passed_parity(
        report,
        bundle,
        arm_name="enabled",
        policy="causal-attention-output-enabled/1",
    )
    _require_passed_parity(
        report,
        bundle,
        arm_name="attention_off",
        policy="causal-attention-output-zero-post-wo-before-residual/1",
    )
    source_audit = report.get("source_read_audit")
    if not isinstance(source_audit, dict):
        raise ValueError("Rust qualification has no source-read audit")
    if (
        source_audit.get("provider_calls") != 0
        or source_audit.get("ollama_calls") != 0
        or source_audit.get("prior_trace_reads") != 0
        or not _require_exact_bool(
            source_audit.get("tree_unchanged_across_execution"),
            label="Rust qualification.source_read_audit.tree_unchanged_across_execution",
        )
    ):
        raise ValueError("Rust qualification did not remain local and immutable")
    _require_cid(report_file_cid, label="Rust qualification file CID")


def _admission_artifact_paths() -> list[str]:
    return sorted(
        {
            "training-view-manifest.json",
            str(SMOKE_MANIFEST_PATH),
            *_SMOKE_ARTIFACT_PATHS,
            str(IMPORTED_RUST_QUALIFICATION_PATH),
        }
    )


def load_main_admission(
    root: Path,
    *,
    training_view: dict[str, Any] | None = None,
    require_current_trainer: bool = False,
) -> dict[str, Any]:
    """Reproduce the durable smoke-plus-Rust gate without opening sealed test data."""
    view = training_view if training_view is not None else load_training_view_manifest(root)
    admission = verify_bound_manifest(root / MAIN_ADMISSION_MANIFEST_PATH, artifact_root=root)
    if admission.get("schema") != MAIN_ADMISSION_MANIFEST_SCHEMA:
        raise ValueError("unsupported main-admission manifest schema")
    if admission.get("terminal") != "PASS_SMOKE_AND_RUST_TWO_ARM_PARITY":
        raise ValueError("main campaign has no PASS admission")
    if _artifact_paths(admission, label="main admission") != set(_admission_artifact_paths()):
        raise ValueError("main admission does not bind the exact required evidence set")
    bundle = _verify_smoke_bundle(root, view)
    report_path = root / IMPORTED_RUST_QUALIFICATION_PATH
    report = _load_json_object(report_path, "imported Rust qualification")
    report_file_cid = cid_file(report_path)
    _verify_rust_qualification(report, report_file_cid=report_file_cid, bundle=bundle)
    common_fields = ("dataset_manifest_cid", "training_view_manifest_cid", "split_policy_cid")
    view_identity = {
        "dataset_manifest_cid": view["dataset_manifest_cid"],
        "training_view_manifest_cid": view["manifest_cid"],
        "split_policy_cid": view["split_policy_cid"],
    }
    _require_identity(admission, view_identity, common_fields, label="admission/training view")
    expected_fields = {
        "smoke_trainer_implementation_tree_cid": bundle["trainer_tree_cid"],
        "smoke_manifest_cid": bundle["manifest"]["manifest_cid"],
        "smoke_contract_cid": bundle["contract_cid"],
        "smoke_result_cid": bundle["result"]["result_cid"],
        "smoke_export_manifest_cid": bundle["export"]["manifest_cid"],
        "smoke_weights_cid": bundle["export"]["weights_cid"],
        "python_prefix_result_cid": bundle["prefix"]["result_cid"],
        "python_prefix_logits_cid": bundle["prefix_file_cid"],
        "rust_qualification_report_cid": report_file_cid,
        "rust_qualification_decision_cid": report["decision_cid"],
    }
    for field, expected in expected_fields.items():
        if admission.get(field) != expected:
            raise ValueError(f"main admission {field} does not reproduce")
    transition = admission.get("smoke_reuse_transition")
    if not isinstance(transition, dict):
        raise ValueError("main admission has no smoke-reuse transition")
    unsigned_transition = dict(transition)
    expected_transition_cid = unsigned_transition.pop("transition_cid", None)
    if expected_transition_cid != cid_bytes(canonical_json_bytes(unsigned_transition)):
        raise ValueError("smoke-reuse transition CID does not reproduce")
    if admission.get("smoke_reuse_transition_cid") != expected_transition_cid:
        raise ValueError("main admission binds a different smoke-reuse transition")
    if transition.get("smoke_trainer_tree_cid") != bundle["trainer_tree_cid"]:
        raise ValueError("smoke-reuse transition binds a different historical smoke")
    if admission.get("trainer_implementation_tree_cid") != transition.get(
        "campaign_trainer_tree_cid"
    ):
        raise ValueError("main admission campaign trainer identity is inconsistent")
    if require_current_trainer:
        current_trainer = trainer_implementation_contract()
        reproduced_transition = _smoke_reuse_transition(
            bundle["trainer_contract"], current_trainer
        )
        if reproduced_transition != transition:
            raise ValueError("campaign trainer changed after the main admission was frozen")
    return admission


def admit_rust_smoke_qualification(root: Path, rust_qualification: Path) -> dict[str, Any]:
    """Import one real Rust PASS and freeze the sole main-campaign admission."""
    training_view = load_training_view_manifest(root)
    bundle = _verify_smoke_bundle(root, training_view)
    current_trainer = trainer_implementation_contract()
    smoke_reuse_transition = _smoke_reuse_transition(
        bundle["trainer_contract"], current_trainer
    )
    source_bytes = rust_qualification.read_bytes()
    try:
        report = json.loads(source_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("Rust qualification is not valid UTF-8 JSON") from error
    if not isinstance(report, dict):
        raise ValueError("Rust qualification must be a JSON object")
    source_cid = cid_bytes(source_bytes)
    _verify_rust_qualification(report, report_file_cid=source_cid, bundle=bundle)

    manifest_path = root / MAIN_ADMISSION_MANIFEST_PATH
    if manifest_path.exists():
        existing = load_main_admission(
            root, training_view=training_view, require_current_trainer=True
        )
        if existing.get("rust_qualification_report_cid") != source_cid:
            raise FileExistsError("a different immutable main admission already exists")
        return existing

    atomic_write(root / IMPORTED_RUST_QUALIFICATION_PATH, source_bytes)
    enabled_delta = report["enabled_prefix_parity"]["maximum_absolute_logit_delta"]
    attention_off_delta = report["attention_off_prefix_parity"][
        "maximum_absolute_logit_delta"
    ]
    return write_bound_manifest(
        manifest_path,
        {
            "schema": MAIN_ADMISSION_MANIFEST_SCHEMA,
            "terminal": "PASS_SMOKE_AND_RUST_TWO_ARM_PARITY",
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "split_policy_cid": training_view["split_policy_cid"],
            "smoke_trainer_implementation_tree_cid": bundle["trainer_tree_cid"],
            "trainer_implementation_tree_cid": current_trainer["tree_cid"],
            "smoke_reuse_transition": smoke_reuse_transition,
            "smoke_reuse_transition_cid": smoke_reuse_transition["transition_cid"],
            "smoke_manifest_cid": bundle["manifest"]["manifest_cid"],
            "smoke_contract_cid": bundle["contract_cid"],
            "smoke_result_cid": bundle["result"]["result_cid"],
            "smoke_export_manifest_cid": bundle["export"]["manifest_cid"],
            "smoke_weights_cid": bundle["export"]["weights_cid"],
            "python_prefix_result_cid": bundle["prefix"]["result_cid"],
            "python_prefix_logits_cid": bundle["prefix_file_cid"],
            "rust_qualification_report_cid": source_cid,
            "rust_qualification_decision_cid": report["decision_cid"],
            "rust_qualification_passed": True,
            "enabled_maximum_absolute_logit_delta": enabled_delta,
            "attention_off_maximum_absolute_logit_delta": attention_off_delta,
            "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
            "sealed_test_status": "UNOPENED",
        },
        artifact_root=root,
        relative_paths=_admission_artifact_paths(),
    )
