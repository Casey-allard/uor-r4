#!/usr/bin/env python3
"""Host-only deterministic verifier for the K16-A vs K16-R (rows_v1) admission redirect.

Modes:
  --self-test                  Verify contract v1/v2/v3 pins, preserved audit backups,
                               companion README.md, repo/staged/pod-operator lineage
                               recomputation, and positive/negative qualification +
                               outcome decision unit tests ($0 GPU).
  --check-qualification <dir>  Verify single-session qualification receipts in native RUN
                               layout, explicit FilePin manifest, or flat layout:
                               rows-qualify positive (7 controls, 4-tuple rows_image_sha256)
                               + negative reducer gate (specific forced-terminal loss mismatch
                               & negative 4-tuple rows_image_sha256), 2-cell checkpoint0
                               100-batch rows_v1 k16-timing Ledger (Owner Waiver #2029
                               comment 6251027125; --legacy-4cell-timing optional), and
                               C-R1..C-R6 SuiteExecutionSummary + pinned control reports
                               (C-R5 14/1020/[641,73,256] + 15 status12 cases; C-R6 4 distinct
                               K16 A/R x seed 1/2 role lineages + K8 seed 1 role, G-Pre >= 4,894).
  --check-outcomes <dir>       Verify 4-trajectory K16-A vs K16-R (seeds {1,2}, 16,404
                               batches) checkpoints/evaluations, 4 hash domains (A..D),
                               admitted runtime source/kernel/executable identity binding,
                               checkpoint0 Domain-A/B storage/snapshot join, G-Adm, G-Occ,
                               Bar (i), Bar (ii), checkpoint0-rebased G-Guard (plus
                               structural-bias audit), paired delta_noise_M, and
                               per-qualifying-arm/seed G-Trans (2,000 rows).
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import pathlib
import subprocess
import sys
import tempfile
from typing import Any

CONSUMER_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = pathlib.Path("/Users/casey.allard/uor-r4")
CODEX_FASTPATH_DIR = pathlib.Path("/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath")

CONTRACT_V1_SHA256 = "126d6f623ec9b401459d5554d1984837867219cc72313635bcbe9e0031eca701"
CONTRACT_V2_SHA256 = "f06d8fd95069e7505d383863d798598f6be6b5d642d2358a298ce3061738484a"
CONTRACT_V3_SHA256 = "514ae2940710e66cdd00286dc4a143da7d8a7e84dfd45c79a0f03b0269ea0697"
README_V3_SHA256 = "28760a3211f28a73f8634881ea3eb0455997b40030410a82c3626863e4dee906"
VERIFIER_V3_SHA256 = "019eb5aa2551a507a840fe6124627253783b3a534ebea048d82c166a822f7ebe"
README_8E3A2B_SHA256 = "8e3a2b3ce32e02f63cf77d30376c4f4afa67fc871c016323c4fada06137783d1"
VERIFIER_C02D31_SHA256 = "c02d31d45bd81d0ee53860218fee453385cc67d26bdeed5555a5d72af8348e2c"
README_410FA4DE_SHA256 = "410fa4de6f1200300e41e366f7cae15eb255b2f913dc74c9e2c8ad45c710675c"
VERIFIER_5C3BC562_SHA256 = "5c3bc562df4ec1d2926980afeb08b308ac4578abbc00599966605de44449a6fd"
STAGE_A_SWEEP_SHA256 = "cc81ca6f76f3a38ac61ffe1d4665a1442d1e42961f4451691b327b91b438f472c".replace(
    "1b327b91", "1327b91"
)
ATTEMPT5_ROWS_PARITY_SHA256 = "27f38a7c24576da6eeabe1cf7e2fc219105756106383e32a3321113ded66cb71"
ATTEMPT5_ROWS_PARITY_NEG_SHA256 = "5ec82dc2eca43edc7a607be460268e517f1db88369a1000a7f4463d732722737"
Q4_ORIGINAL_DOMAIN_C_SHA256 = "ca27292ec2dd3f0ac075f85125327298eb8194a702d12881962911b4ec84b6d8"

# Current-authority producer constants at origin/main 21224b446db14efc8f9123cf8c43917a5337ea95
STAGED_K16_METADATA_DIR = pathlib.Path(
    "/Users/casey.allard/uor-r4-local/k16-training-metadata-stage-2020z"
)
ADMITTED_BASE_SOURCE = "2fa663b54"
ADMITTED_REVIEWED_SOURCE_COMMIT = "21224b446db14efc8f9123cf8c43917a5337ea95"
ADMITTED_NATIVE_EXECUTABLE_SHA256 = (
    "3add3e8d6942dc53d90682446914338e99141dc81ef08a1b79b8e71a6565c336"
)
ADMITTED_REDIRECT_PACKET_SHA256 = (
    "e367301901499c125521b15cc05acfc0bc0fa7eaaa4888e072cbe8ac1b1a047a"
)
ADMITTED_AMENDMENT7_SHEET_SHA256 = (
    "8be303f551b63ebcb9db5d733025ccca947ca99de81296915fe383202884f9c1"
)
ALLOWED_REDIRECT_SHEET_SHA256S = frozenset(
    {
        ADMITTED_AMENDMENT7_SHEET_SHA256,
        "e40034cd03a4e4c76b0f216e499d0f32a151a98fc370bb8939ea97ff67920884",
    }
)
DISPATCHED_K16_ROLE_RELEASE_PINS: dict[str, dict[str, str]] = {
    "k16-A-seed1": {
        "scope": "m1-admission-redirect-k16-a-seed1-checkpoint0..16404",
        "admission_sha256": "c09e50317a471a650e5534eb73dd447dc48cb015a0ba7bf712a6d9a1ec7a6459",
        "owner_sha256": "b6f264b2061141997d32aeaac0b8384c9c2aa3f375c59e36a573fab4b78597d9",
        "plan_sha256": "bdc7caec25de44c46ffec5e3a54aa717ed8fef795f0cfa05ceab04210a48f29e",
        "qualification_sha256": "b4afdefd68903cec294765157a3ae7b12dbee0cb901a2afbc5bcb0f17e9cd287",
        "pm_review_sha256": "6a64bcc93b0fd98ca344902eca6d23e3125b8334ef132a4e560474727d35382d",
    },
    "k16-R-seed1": {
        "scope": "m1-admission-redirect-k16-r-seed1-checkpoint0..16404",
        "admission_sha256": "670a4ecbf8c9eb20aa0ab0d7098c323491d5d0a4c104122e0c93888942945d21",
        "owner_sha256": "4f5bf7dcf190e92f76ccaf49a0351fbda4daf27473ead143efe4dfa8694d7954",
        "plan_sha256": "db3838605ced1ab0f738887810721cc6ea3fbfe7edba88b2fdbd5e45aef12fe3",
        "qualification_sha256": "6471d3395aba0c40a56031ca24b62d07cb84fec13790376ad9d1546d48566d53",
        "pm_review_sha256": "678d48cf0ea02026728cdc5149ab7e9a8acd336a9786ccded592ee3dc64f7d05",
    },
    "k16-A-seed2": {
        "scope": "m1-admission-redirect-k16-a-seed2-checkpoint0..16404",
        "admission_sha256": "ffa8745e6482bebf1a594cf67a6168611c69b9cea18f2dd1f9393bb3cc424967",
        "owner_sha256": "387a760f29378b31092bd9aa9b16ccbcab20cf41476c733a556d260d1b10bb2f",
        "plan_sha256": "7945917b0b94077449c2638e4033ce3a17f073ca53e50b8de4024b8c40c2882c",
        "qualification_sha256": "da552e8fca5e646968bddf3984c317c9a3b18f8cf6500dc6bb7387cdd7dece32",
        "pm_review_sha256": "a2541e679048234a3fe76850afa6228d3403fe0013b0de90b602ff08b08b284e",
    },
    "k16-R-seed2": {
        "scope": "m1-admission-redirect-k16-r-seed2-checkpoint0..16404",
        "admission_sha256": "308ed4c27bde1fd90c58d9e749966596d82266a2ad6ec66c9c6f0c4e8dd40a4c",
        "owner_sha256": "f74ddbee3927234ca5e6b3cec1348b8a31bc9eed66de45d66ffc4b087b5efe72",
        "plan_sha256": "392ea8df7cff48eb90ce57c1e9c60acd20c5bb5bba26310514e7b23d6e8433ab",
        "qualification_sha256": "4879fb4c4fd8b802d5d22939bec4f3c388d6a0a85e6a36503ecf4dc0906f56ba",
        "pm_review_sha256": "0562cf220b51811f7dd9b466622a6ed9e0acecd88058bfca6ccd473412909c35",
    },
}
SEALED_K16_CUSTODY_DIR = pathlib.Path("/Users/casey.allard/uor-r4-local/k16-four-role-data-custody-2324z")
SEALED_K16_MEMBER_MAP_DIR = (
    STAGED_K16_METADATA_DIR / "final-outcome-member-map-2337z"
)
SEALED_K16_FOUR_ROLE_CUSTODY_PINS: dict[str, dict[str, Any]] = {
    "member_manifest_receipt": {
        "path": str(SEALED_K16_CUSTODY_DIR / "metadata" / "manifests" / "training-root" / "part-000"),
        "bytes": 179398,
        "sha256": "5c4077b387592a15704e23fc79df003b222086ce64a2c267f1e8e92858428265",
    },
    "shard_index_receipt": {
        "path": str(SEALED_K16_CUSTODY_DIR / "metadata" / "SHARD-INDEX.tsv"),
        "bytes": 699,
        "sha256": "c21cd8b1591eab12c2b71ecff3f9fa8646b6c65c68d7a1b0a8be1205ca111127",
    },
    "custody_join_receipt": {
        "path": str(SEALED_K16_CUSTODY_DIR / "FINAL-INDEX-SHARD-CUSTODY-JOIN.json"),
        "bytes": 1823,
        "sha256": "44eafa157fdba6060ca6ee0d43db8fffa3f1b38db1514709e7590f04f4a90459",
    },
    "member_map_receipt": {
        "path": str(SEALED_K16_MEMBER_MAP_DIR / "MEMBER-MAP.json"),
        "bytes": 502425,
        "sha256": "8ad8ec3c4a88962ff89463f75ef14ca05693361e8c2b1b378868af9ede230b99",
    },
    "cloud_binding_receipt": {
        "path": str(SEALED_K16_MEMBER_MAP_DIR / "ACTUAL-CLOUD-BINDING.json"),
        "bytes": 7132,
        "sha256": "539e5348a4be0c5e31db3265ed9e31a0a78bdcfa96f8671d36ae844493af02ad",
    },
}
POSITIVE_ROWS_SOURCE_RS_SHA256 = "23d311bf815c458195a93c062a51eb9a24e91f17b90b9da2faec6fa7fc94d665"
POSITIVE_ROWS_CU_SHA256 = "48d8e6e05f2de03d2f9931812f606e5f923c2416dd4205696a6228d858446318"
NEGATIVE_ROWS_SOURCE_RS_SHA256 = "ef6c50fb4bf38efbe3a34176f052c44fa46e54c579d33e134b04921cd308434a"
NEGATIVE_ROWS_CU_SHA256 = "a863c8e8d9c1b1bd429741ac07d8367ca0216fd3ab35b9e9467aa3dc7f8efa3f"
NATIVE_COMPILED_SOURCE_SHA256 = "035b3c127bb5a0eb84e50901cabb09391922e40ec6ffb3b4f638abccf6930b85"
TIMING_COMPANION_SOURCE_SHA256 = "1327c052835fc0e99a264ae593cdf14542dbd6ca74f2d64186f783cd70b0444e"
ADMITTED_KERNEL_SHA256 = "5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6"
EXPECTED_NEGATIVE_REDUCER_ERROR_SUBSTRINGS = (
    "Gate(1): losses differ: ordinary [1344, 18446744073709551615, 18446744073709551615",
    "rows [1344, 0, 0, 18446744073709551615",
)

CR5_ROW_WORDS = 14
CR5_AGGREGATE_WORDS = 1020
CR5_HISTOGRAM_BINS = [641, 73, 256]
CR5_STATUS12_CASE_NAMES = [
    "raw_row_bounds_T_gt_640",
    "raw_row_bounds_A_gt_72",
    "raw_row_bounds_R_gt_255",
    "target_cell_rank_zero",
    "target_cell_rank_gt_256",
    "occupied_cells_zero",
    "occupied_cells_gt_256",
    "admitted_cells_zero",
    "admitted_cells_gt_16",
    "loss_ta_formula_mismatch",
    "loss_tr_formula_mismatch",
    "cross_row_occupied_cells_mismatch",
    "u64::MAX plus 1",
    "weighted R overflow",
    "quantile threshold overflow",
]

ACTIVE_TIMING_CELLS_CHECKPOINT0 = (
    ("checkpoint0", "nearest_address_margin_v1"),
    ("checkpoint0", "occupied_cell_rank_hinge_v1"),
)

K16_FOUR_ROLES_IN_ORDER = (
    ("k16-A-seed1", 1, "nearest_address_margin_v1"),
    ("k16-R-seed1", 1, "occupied_cell_rank_hinge_v1"),
    ("k16-A-seed2", 2, "nearest_address_margin_v1"),
    ("k16-R-seed2", 2, "occupied_cell_rank_hinge_v1"),
)

POD_OPERATOR_ARM_BINDINGS: dict[str, dict[str, str]] = {
    "k16-A-seed1": {
        "staged_raw_file_sha256": "71fcd892596672558770469553c8be06ca3e28b74d6c8ae101910eadce2ab53d",
        "rows_v1_typed_config_sha256": "d2be3b4e6703c37e4f074670ca79be15b36f4f898861d8eaccc05a55f6349c40",
        "rows_v1_lineage_sha256": "25f9a5880d66e4112edc321084ffb8595d740df94055ae92dd2db202ed1b86f3",
    },
    "k16-R-seed1": {
        "staged_raw_file_sha256": "c70f25f3a165b62b8efa5bc0ccf7b4f81693f2b6174649d29d38abf4a616e5e0",
        "rows_v1_typed_config_sha256": "fd3ad5d882576be4a0914b940a30955e5cacd0eedc63171d04a5164ebcf08f0f",
        "rows_v1_lineage_sha256": "e031d7d94795655a823e176c90befbdfcbaa50a088fbf1e7d6282c95aafc9b79",
    },
    "k16-A-seed2": {
        "staged_raw_file_sha256": "9adc0b607d4907fd336cf2caafb12f4386d771cbc57dcfbef56e45ef37c4841b",
        "rows_v1_typed_config_sha256": "3675090479c772c6ff7979d1e0ad0c5c5ffaf80ecaab96f332e4fe12936b3352",
        "rows_v1_lineage_sha256": "72109c0483653dafe1dac202dccf226f530c48d56b76bde548a96defb1d0af82",
    },
    "k16-R-seed2": {
        "staged_raw_file_sha256": "a6648d563b2bfa3e7d477650b99f70f8fed58e2012189f697309a831a41a7ef3",
        "rows_v1_typed_config_sha256": "6930676ba8f48c3cee6fddb4cb561687a4dfc0a4ea4ed90c90fb01ab6c3e3091",
        "rows_v1_lineage_sha256": "f638e3b1665007ebdedf4b04ba3b28fd779353700204123d4ea5957f4d0cbd2b",
    },
}

POD_OPERATOR_K8_BINDINGS: dict[str, dict[str, str]] = {
    "k8-A-seed1": {
        "staged_raw_file_sha256": "0d8ed5ec49a5ec89d75e1209d3c6881373ac3e10c5929eed827da197300a8817",
        "rows_v1_typed_config_sha256": "a8095677e4870bbd2a6d7a90f7d4431446ed5c8a780dfe8badc4c51f6c0385d1",
        "rows_v1_lineage_sha256": "295562ff0a150d1dc01f2a428a745e7952788f1c4f409b0f30edda6359b2af0c",
    },
    "k8-A-seed2": {
        "staged_raw_file_sha256": "07af21533ad011a8f276dd87639a10dc5f232186f80de4591a2e4287e2eb337f",
        "rows_v1_typed_config_sha256": "b4d07e6be915ebee15827e2c18b8b1d2830b61e19548a0850dd39993976a5834",
        "rows_v1_lineage_sha256": "c5552a058ad802aba0763fdde35b0024cbc7bfdcf96b9953600d95270c5bd5e5",
    },
}

K8_SEED1_ALLOWED_CFG_LIN: set[tuple[str, str]] = {
    (
        "ec2a622bd81dd9648e98967c3a8e99297e91dc009c9fa07a60837477a866f340",
        "87639a8e7966097dc1f4978137f57f1b4ff4e3018aa75e8a4e8da97092b150e2",
    ),
    (
        "b6116b2c04a221f2f76c827a4de6b15d74504a5ee8eddba79282155439e83943",
        "4d30bc9b57455b3a86558d5a07ac08dcd81619ad1fbbf31a66e2f21ad772d93f",
    ),
    (
        "58272296e72cf6e8a463689598d7bc32b8d6a360292582c8b02364463c5e25ed",
        "9ab6bf2e792fb8db58fc21931b3794de880c94efde429d478f197bf2c3755927",
    ),
    (
        "a8095677e4870bbd2a6d7a90f7d4431446ed5c8a780dfe8badc4c51f6c0385d1",
        "295562ff0a150d1dc01f2a428a745e7952788f1c4f409b0f30edda6359b2af0c",
    ),
}

BINDING_ORDER = [
    "source",
    "vocabulary",
    "bos",
    "payload_tokens",
    "mask_bytes",
    "mask_selected_positions",
    "eligible_targets",
    "documents",
    "window_policy",
    "identity_policy",
]


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_strict_int(val: Any) -> bool:
    return isinstance(val, int) and not isinstance(val, bool)


def is_hex64(val: Any) -> bool:
    if not isinstance(val, str) or len(val) != 64:
        return False
    return all(c in "0123456789abcdef" for c in val)


def is_rows_image_sha256_tuple(val: Any, expected_cu_sha256: str | None = None) -> bool:
    """Validate compiled_rows_image_sha256() -> [String; 4] ([rows.cu, rows.ptx, rows.cubin, rows-resources.txt])."""
    if not isinstance(val, list) or len(val) != 4 or not all(is_hex64(x) for x in val):
        return False
    if expected_cu_sha256 is not None and val[0] != expected_cu_sha256:
        return False
    return True


def compact_json_bytes(val: Any) -> bytes:
    return json.dumps(val, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def canonical_sorted_json_bytes(val: Any) -> bytes:
    return json.dumps(val, separators=(",", ":"), sort_keys=True, ensure_ascii=False).encode("utf-8")


def sort_json_value(val: Any) -> Any:
    if isinstance(val, dict):
        return {k: sort_json_value(val[k]) for k in sorted(val.keys())}
    if isinstance(val, list):
        return [sort_json_value(item) for item in val]
    return val


def native_driver_config_form(d: dict[str, Any]) -> bytes:
    """Serialize DriverConfig in exact Rust struct field order with skip_serializing_if rules."""

    def ordered(v: dict[str, Any], keys: list[str]) -> dict[str, Any]:
        return {k: v[k] for k in keys}

    def pin(v: dict[str, Any]) -> dict[str, Any]:
        return ordered(v, ["path", "sha256"])

    def corpus(v: dict[str, Any]) -> dict[str, Any]:
        return {
            "split": v["split"],
            "tokens": pin(v["tokens"]),
            "mask": pin(v["mask"]),
            "expected_payload_tokens": v["expected_payload_tokens"],
            "expected_eligible_targets": v["expected_eligible_targets"],
        }

    rs_keys = ["kind"]
    if d["read_spec"]["kind"] in ("flock", "relation"):
        rs_keys.append("kept")
    if d["read_spec"]["kind"] == "relation":
        rs_keys.append("sweeps")

    c: dict[str, Any] = {
        "schema": d["schema"],
        "spec": ordered(d["spec"], ["vocabulary", "blocks", "context", "kept_cells"]),
        "read_spec": ordered(d["read_spec"], rs_keys),
    }
    for k, default in (
        ("objective_policy", "nearest_address_margin_v1"),
        ("read_initialization", "zero_gates_v1"),
        ("relation_learning_policy", "learn_v1"),
        ("counterfactual_path", "lanes_v1"),
    ):
        if k in d and d[k] != default:
            c[k] = d[k]
    c.update(
        {
            "seed": d["seed"],
            "batch_rows": d["batch_rows"],
            "train": corpus(d["train"]),
            "dev": corpus(d["dev"]),
            "tokenizer": pin(d["tokenizer"]),
            "bos": ordered(d["bos"], ["token", "id"]),
            "schedule": d["schedule"],
        }
    )
    return compact_json_bytes(c)


def native_corpus_binding_dict(b: dict[str, Any], source_ordered: dict[str, Any]) -> dict[str, Any]:
    out = dict(b)
    out["source"] = source_ordered
    out["bos"] = {"token": b["bos"]["token"], "id": b["bos"]["id"]}
    return {k: out[k] for k in BINDING_ORDER}


def native_lineage_dict(lin: dict[str, Any]) -> dict[str, Any]:
    cfg_ordered = json.loads(native_driver_config_form(lin["config"]))
    return {
        "schema": lin["schema"],
        "driver_source_sha256": lin["driver_source_sha256"],
        "config": cfg_ordered,
        "train": native_corpus_binding_dict(lin["train"], cfg_ordered["train"]),
        "dev": native_corpus_binding_dict(lin["dev"], cfg_ordered["dev"]),
    }


EXECUTION_IDENTITY_ORDER = [
    "base_source",
    "reviewed_source_commit",
    "source_sha256",
    "kernel_sha256",
    "geometry_sha256",
    "executable_sha256",
    "driver_source_sha256",
    "config_sha256",
    "lineage_sha256",
    "sheet_sha256",
]

COMPLETION_REPORT_REQUIRED_KEYS = frozenset(
    {
        "schema",
        "status",
        "seed",
        "scope",
        "admission_sha256",
        "completed_batches",
        "eligible_train_target_exposures",
        "baseline_evaluation",
        "baseline_snapshot_metadata",
        "pilot_parent",
        "sensitivity",
        "evaluations",
        "attempt_elapsed_ns",
        "fresh_transfer",
        "next_seed",
        "m2",
    }
)


def native_file_pin_dict(pin: dict[str, Any]) -> dict[str, Any]:
    """Serialize corpus::FilePin in exact 2-key Rust struct order ({path, sha256})."""
    if not isinstance(pin, dict) or set(pin.keys()) != {"path", "sha256"}:
        raise ValueError(
            f"FilePin in native Rust struct must have exact keys {{'path', 'sha256'}}, got {pin!r}"
        )
    if not isinstance(pin.get("path"), str) or not pin["path"].strip() or not is_hex64(pin.get("sha256")):
        raise ValueError(f"Invalid FilePin path/sha256: {pin!r}")
    return {"path": pin["path"], "sha256": pin["sha256"]}


def native_execution_identity_dict(ex: dict[str, Any]) -> dict[str, Any]:
    """Serialize controller::ExecutionIdentity in exact 10-key Rust struct order (controller.rs:81-93)."""
    if not isinstance(ex, dict) or set(ex.keys()) != set(EXECUTION_IDENTITY_ORDER):
        raise ValueError(
            f"ExecutionIdentity keys mismatch: {sorted(ex.keys()) if isinstance(ex, dict) else ex!r}"
        )
    return {k: ex[k] for k in EXECUTION_IDENTITY_ORDER}


def native_redirect_evidence_dict(red: dict[str, Any]) -> dict[str, Any]:
    """Serialize redirect_admission::RedirectEvidence in exact 3-key Rust struct order (redirect_admission.rs:5-9)."""
    if not isinstance(red, dict) or set(red.keys()) != {"packet", "qualification", "pm_review"}:
        raise ValueError(f"RedirectEvidence keys mismatch: {red!r}")
    return {
        "packet": native_file_pin_dict(red["packet"]),
        "qualification": native_file_pin_dict(red["qualification"]),
        "pm_review": native_file_pin_dict(red["pm_review"]),
    }


def native_completion_plan_dict(plan: dict[str, Any]) -> dict[str, Any]:
    """Order completion::CompletionPlan fields in exact Rust struct declaration order (completion.rs:30-43)."""
    allowed = {
        "schema",
        "status",
        "scope",
        "execution",
        "config",
        "sheet",
        "claude_review",
        "run_state_directory",
        "pilot_parent",
        "redirect",
    }
    req = allowed - {"redirect"}
    if not isinstance(plan, dict) or not req.issubset(plan.keys()) or (set(plan.keys()) - allowed):
        raise ValueError(
            f"CompletionPlan keys mismatch: {sorted(plan.keys()) if isinstance(plan, dict) else plan!r}"
        )
    cfg_ordered = json.loads(native_driver_config_form(plan["config"]))
    out: dict[str, Any] = {
        "schema": plan["schema"],
        "status": plan["status"],
        "scope": plan["scope"],
        "execution": native_execution_identity_dict(plan["execution"]),
        "config": cfg_ordered,
        "sheet": native_file_pin_dict(plan["sheet"]),
        "claude_review": native_file_pin_dict(plan["claude_review"]),
        "run_state_directory": plan["run_state_directory"],
        "pilot_parent": (
            native_file_pin_dict(plan["pilot_parent"])
            if plan.get("pilot_parent") is not None
            else None
        ),
    }
    if plan.get("redirect") is not None:
        out["redirect"] = native_redirect_evidence_dict(plan["redirect"])
    return out


def native_completion_plan_form(plan: dict[str, Any]) -> bytes:
    """Serialize CompletionPlan in exact Rust encode(&plan) order (completion.rs:284, 334)."""
    return compact_json_bytes(native_completion_plan_dict(plan))


def native_completion_admission_form(adm: dict[str, Any]) -> bytes:
    """Serialize CompletionAdmission in exact Rust encode(&admission) order (completion.rs:46-50, 335)."""
    if not isinstance(adm, dict) or set(adm.keys()) != {"schema", "plan", "owner_release"}:
        raise ValueError(
            f"CompletionAdmission keys mismatch: {sorted(adm.keys()) if isinstance(adm, dict) else adm!r}"
        )
    out = {
        "schema": adm["schema"],
        "plan": native_completion_plan_dict(adm["plan"]),
        "owner_release": native_file_pin_dict(adm["owner_release"]),
    }
    return compact_json_bytes(out)


ALLOWED_SNAPSHOT_KEYS = frozenset({
    "schema",
    "spec",
    "read_spec",
    "objective_policy",
    "read_initialization",
    "seed",
    "epoch",
    "table",
    "routes",
    "gates",
    "operator_kappa",
    "composition_kappa",
    "kernel_sha256",
    "source_sha256",
    "geometry_sha256",
})
ALLOWED_SPEC_KEYS = frozenset({"vocabulary", "blocks", "context", "kept_cells"})
ALLOWED_READ_SPEC_KEYS = frozenset({"kind", "kept", "sweeps"})


def native_snapshot_form(model: dict[str, Any]) -> bytes:
    """Serialize uor_r4_discrete::device::Snapshot in exact Rust struct declaration order (Domain B),
    enforcing strict unknown-field rejection."""
    unknown_top = set(model.keys()) - ALLOWED_SNAPSHOT_KEYS
    if unknown_top:
        raise ValueError(f"Unknown field(s) in Snapshot: {sorted(unknown_top)}")
    spec = model["spec"]
    if not isinstance(spec, dict) or (set(spec.keys()) - ALLOWED_SPEC_KEYS):
        raise ValueError(f"Unknown or invalid field(s) in Snapshot.spec: {spec!r}")
    read_spec = model["read_spec"]
    if not isinstance(read_spec, dict) or (set(read_spec.keys()) - ALLOWED_READ_SPEC_KEYS):
        raise ValueError(f"Unknown or invalid field(s) in Snapshot.read_spec: {read_spec!r}")
    out: dict[str, Any] = {
        "schema": model["schema"],
        "spec": {
            "vocabulary": spec["vocabulary"],
            "blocks": spec["blocks"],
            "context": spec["context"],
            "kept_cells": spec["kept_cells"],
        },
        "read_spec": read_spec,
    }
    obj = model.get("objective_policy", "nearest_address_margin_v1")
    if obj != "nearest_address_margin_v1":
        out["objective_policy"] = obj
    read_init = model.get("read_initialization", "zero_gates_v1")
    if read_init != "zero_gates_v1":
        out["read_initialization"] = read_init
    for k in (
        "seed",
        "epoch",
        "table",
        "routes",
        "gates",
        "operator_kappa",
        "composition_kappa",
        "kernel_sha256",
        "source_sha256",
        "geometry_sha256",
    ):
        out[k] = model[k]
    return compact_json_bytes(out)


def native_driver_checkpoint_form(inner_ckpt: dict[str, Any]) -> bytes:
    """Serialize DriverCheckpointRecord in exact Rust struct declaration order (Domain C)."""
    cur = inner_ckpt["cursor"]
    out = {
        "schema": inner_ckpt["schema"],
        "lineage": native_lineage_dict(inner_ckpt["lineage"]),
        "cursor": {
            "completed_batches": cur["completed_batches"],
            "accepted_updates": cur["accepted_updates"],
            "corpus_pass": cur["corpus_pass"],
            "next_target": cur["next_target"],
            "next_coordinate": cur["next_coordinate"],
        },
        "model_sha256": inner_ckpt["model_sha256"],
        "model": sort_json_value(inner_ckpt["model"]),
    }
    return compact_json_bytes(out)


def compute_snapshot_hash_domains(
    checkpoint_obj: dict[str, Any], raw_file_bytes: bytes | None = None
) -> dict[str, str]:
    """Compute all 4 hash domains (Domains A, B, C, and D) for a DriverCheckpoint or CompletionCheckpoint."""
    inner_ckpt = checkpoint_obj.get("checkpoint", checkpoint_obj)
    model = inner_ckpt["model"]
    domain_a = sha256_bytes(canonical_sorted_json_bytes(model))
    domain_b = sha256_bytes(native_snapshot_form(model))
    domain_c = sha256_bytes(native_driver_checkpoint_form(inner_ckpt))
    res = {
        "domain_A_model_storage_sha256": domain_a,
        "domain_B_typed_model_snapshot_sha256": domain_b,
        "domain_C_driver_checkpoint_sha256": domain_c,
        "declared_model_sha256": inner_ckpt.get("model_sha256", ""),
    }
    if raw_file_bytes is not None:
        res["domain_D_checkpoint_file_sha256"] = sha256_bytes(raw_file_bytes)
    return res


def verify_zero_mutation_invariants(
    obj: dict[str, Any],
    digest_before_key: str,
    digest_after_key: str,
    epoch_before_key: str,
    epoch_after_key: str,
    *,
    expected_digest: str | None = None,
    expected_epoch: int | None = None,
    label: str,
    errors: list[str],
) -> None:
    """Enforce non-vacuous before/after snapshot digest (64-hex) and integer epoch equality (V3-S8)."""
    d_before = obj.get(digest_before_key)
    d_after = obj.get(digest_after_key)
    if not is_hex64(d_before) or not is_hex64(d_after):
        errors.append(
            f"{label}: {digest_before_key}={d_before!r} and {digest_after_key}={d_after!r} must be valid 64-hex SHA-256 strings"
        )
    elif d_before != d_after:
        errors.append(f"{label}: {digest_before_key} ({d_before}) != {digest_after_key} ({d_after})")
    elif expected_digest is not None and d_before != expected_digest:
        errors.append(
            f"{label}: {digest_before_key} ({d_before}) != expected digest ({expected_digest})"
        )

    e_before = obj.get(epoch_before_key)
    e_after = obj.get(epoch_after_key)
    if not is_strict_int(e_before) or not is_strict_int(e_after) or e_before < 0 or e_after < 0:
        errors.append(
            f"{label}: {epoch_before_key}={e_before!r} and {epoch_after_key}={e_after!r} must be non-negative integers"
        )
    elif e_before != e_after:
        errors.append(f"{label}: {epoch_before_key} ({e_before}) != {epoch_after_key} ({e_after})")
    elif expected_epoch is not None and e_before != expected_epoch:
        errors.append(
            f"{label}: {epoch_before_key} ({e_before}) != expected epoch ({expected_epoch})"
        )


def resolve_pin_path(
    rel_or_abs: str, base_dir: pathlib.Path, extra_roots: list[pathlib.Path] | None = None
) -> pathlib.Path:
    """Resolve a FilePin path against base_dir, extra_roots, or pod-to-host RUN suffix mapping."""
    p = pathlib.Path(rel_or_abs)
    roots = [base_dir] + (extra_roots or [])
    if not p.is_absolute():
        for r in roots:
            cand = r / p
            if cand.is_file():
                return cand
        return base_dir / p
    if p.is_file():
        return p
    # If p is a pod-absolute path (e.g. /workspace/uor-r4/local/.../RUN/c-r-fixtures/c-r1/report.json),
    # try matching its trailing path components under our local roots (still strictly verified by SHA-256).
    parts = p.parts
    for i in range(1, len(parts)):
        suffix = pathlib.Path(*parts[i:])
        for r in roots:
            cand = r / suffix
            if cand.is_file():
                return cand
    return p


def authenticate_file_pin(
    pin: Any,
    base_dir: pathlib.Path,
    label: str,
    errors: list[str],
    *,
    extra_roots: list[pathlib.Path] | None = None,
) -> tuple[bytes | None, dict[str, Any] | None]:
    """Authenticate a FilePin ({path, sha256, [bytes]}) against on-disk bytes (or authenticated
    ORIGINAL-RAW-FILEPINS.tsv + *-METADATA-ONLY.json host readback) and parse JSON."""
    if not isinstance(pin, dict):
        errors.append(f"{label}: missing or non-object FilePin: {pin!r}")
        return None, None
    rel_or_abs = pin.get("path")
    exp_sha = pin.get("sha256")
    if not isinstance(rel_or_abs, str) or not rel_or_abs.strip():
        errors.append(f"{label}: FilePin missing non-empty path")
        return None, None
    if not is_hex64(exp_sha):
        errors.append(f"{label}: FilePin sha256={exp_sha!r} is not a valid 64-hex SHA-256")
        return None, None
    p = resolve_pin_path(rel_or_abs, base_dir, extra_roots)
    if not p.is_file():
        roots = [base_dir] + (extra_roots or [])
        tsv_dirs = roots + [r / "gpre2670-metadata-only" for r in roots]
        req_tail = pathlib.Path(rel_or_abs).parts[-2:]
        for tdir in tsv_dirs:
            tsv_p = tdir / "ORIGINAL-RAW-FILEPINS.tsv"
            if not tsv_p.is_file():
                continue
            for line in tsv_p.read_text().splitlines():
                cols = line.strip().split("\t")
                if len(cols) != 4:
                    continue
                role_tag, raw_pod_path, raw_bytes_str, raw_sha = cols
                if raw_sha == exp_sha and (
                    raw_pod_path == rel_or_abs
                    or pathlib.Path(raw_pod_path).parts[-2:] == req_tail
                ):
                    if "bytes" in pin:
                        exp_bytes = pin["bytes"]
                        if not is_strict_int(exp_bytes) or int(raw_bytes_str) != exp_bytes:
                            errors.append(
                                f"{label}: FilePin byte length mismatch in ORIGINAL-RAW-FILEPINS.tsv: actual={raw_bytes_str} != expected={exp_bytes!r}"
                            )
                            return None, None
                    meta_p = tdir / f"{role_tag}-METADATA-ONLY.json"
                    if meta_p.is_file():
                        meta_raw = meta_p.read_bytes()
                        sums_p = tdir / "SHA256SUMS"
                        if sums_p.is_file():
                            expected_meta_sha: str | None = None
                            for sline in sums_p.read_text().splitlines():
                                sparts = sline.strip().split()
                                if len(sparts) >= 2 and pathlib.Path(sparts[-1]).name == meta_p.name:
                                    expected_meta_sha = sparts[0]
                                    break
                            if expected_meta_sha is not None and sha256_bytes(meta_raw) != expected_meta_sha:
                                errors.append(
                                    f"{label}: metadata-only readback SHA-256 mismatch on {meta_p}"
                                )
                                return None, None
                        try:
                            parsed_meta = json.loads(meta_raw)
                        except Exception as exc:
                            errors.append(f"{label}: invalid JSON in {meta_p}: {exc}")
                            return None, None
                        if not isinstance(parsed_meta, dict):
                            errors.append(f"{label}: top-level JSON in {meta_p} is not an object")
                            return None, None
                        return meta_raw, parsed_meta
        errors.append(f"{label}: FilePin target file does not exist: {p}")
        return None, None
    raw = p.read_bytes()
    act_sha = sha256_bytes(raw)
    if act_sha != exp_sha:
        errors.append(f"{label}: FilePin sha256 mismatch on {p}: actual={act_sha} != expected={exp_sha}")
        return None, None
    if "bytes" in pin:
        exp_bytes = pin["bytes"]
        if not is_strict_int(exp_bytes) or len(raw) != exp_bytes:
            errors.append(
                f"{label}: FilePin byte length mismatch on {p}: actual={len(raw)} != expected={exp_bytes!r}"
            )
            return None, None
    try:
        parsed = json.loads(raw)
    except Exception as exc:
        errors.append(f"{label}: invalid JSON in {p}: {exc}")
        return None, None
    if not isinstance(parsed, dict):
        errors.append(f"{label}: top-level JSON in {p} is not an object")
        return None, None
    return raw, parsed


def authenticate_raw_file_pin(
    pin: Any,
    base_dir: pathlib.Path,
    label: str,
    errors: list[str],
    *,
    extra_roots: list[pathlib.Path] | None = None,
) -> tuple[bytes | None, str | None]:
    """Authenticate a FilePin ({path, sha256, [bytes]}) against on-disk bytes for raw text/TSV/manifest files."""
    if not isinstance(pin, dict):
        errors.append(f"{label}: missing or non-object FilePin: {pin!r}")
        return None, None
    rel_or_abs = pin.get("path")
    exp_sha = pin.get("sha256")
    if not isinstance(rel_or_abs, str) or not rel_or_abs.strip():
        errors.append(f"{label}: FilePin missing non-empty path")
        return None, None
    if not is_hex64(exp_sha):
        errors.append(f"{label}: FilePin sha256={exp_sha!r} is not a valid 64-hex SHA-256")
        return None, None
    p = resolve_pin_path(rel_or_abs, base_dir, extra_roots)
    if not p.is_file():
        errors.append(f"{label}: FilePin target file does not exist: {p}")
        return None, None
    raw = p.read_bytes()
    act_sha = sha256_bytes(raw)
    if act_sha != exp_sha:
        errors.append(f"{label}: FilePin sha256 mismatch on {p}: actual={act_sha} != expected={exp_sha}")
        return None, None
    if "bytes" in pin:
        exp_bytes = pin["bytes"]
        if not is_strict_int(exp_bytes) or len(raw) != exp_bytes:
            errors.append(
                f"{label}: FilePin byte length mismatch on {p}: actual={len(raw)} != expected={exp_bytes!r}"
            )
            return None, None
    return raw, act_sha


def authenticate_off_host_custody_manifests(
    data: dict[str, Any],
    outcomes_dir: pathlib.Path,
    outcomes_extra_roots: list[pathlib.Path],
    errors: list[str],
) -> tuple[dict[str, set[str]], dict[str, dict[str, tuple[str, str]]]]:
    """Authenticate an explicit off_host_custody manifest bundle via FilePins and sealed-index/custody-join
    verification before honoring any off-host raw checkpoint-16404.json shortcut.
    Unauthenticated plaintext member manifests on disk are never auto-discovered."""
    cust_spec = data.get("off_host_custody")
    if cust_spec is None:
        return {}, {}
    if cust_spec == "SEALED_K16_FOUR_ROLE_CUSTODY_2324Z":
        cust_obj: dict[str, Any] = copy.deepcopy(SEALED_K16_FOUR_ROLE_CUSTODY_PINS)
    elif isinstance(cust_spec, dict):
        cust_obj = cust_spec
    else:
        errors.append(f"off_host_custody must be an object of FilePins or 'SEALED_K16_FOUR_ROLE_CUSTODY_2324Z', got {cust_spec!r}")
        return {}, {}

    local_errs: list[str] = []
    manifest_raw, manifest_sha = authenticate_raw_file_pin(
        cust_obj.get("member_manifest_receipt"),
        outcomes_dir,
        "off_host_custody.member_manifest_receipt",
        local_errs,
        extra_roots=outcomes_extra_roots,
    )
    index_raw, index_sha = authenticate_raw_file_pin(
        cust_obj.get("shard_index_receipt"),
        outcomes_dir,
        "off_host_custody.shard_index_receipt",
        local_errs,
        extra_roots=outcomes_extra_roots,
    )
    join_raw, join_obj = authenticate_file_pin(
        cust_obj.get("custody_join_receipt"),
        outcomes_dir,
        "off_host_custody.custody_join_receipt",
        local_errs,
        extra_roots=outcomes_extra_roots,
    )
    if local_errs or manifest_raw is None or manifest_sha is None or index_raw is None or index_sha is None or join_raw is None or join_obj is None:
        errors.extend(local_errs)
        return {}, {}

    join_sha = sha256_bytes(join_raw)

    # Parse SHARD-INDEX.tsv and locate the 'training-root' shard row
    training_row: dict[str, Any] | None = None
    try:
        index_lines = [ln for ln in index_raw.decode("utf-8").splitlines() if ln.strip()]
        if len(index_lines) < 2:
            local_errs.append("off_host_custody.shard_index_receipt has fewer than 2 non-empty lines")
        else:
            hdr = index_lines[0].split("\t")
            expected_hdr = [
                "group",
                "archive",
                "archive_sha256",
                "compressed_bytes",
                "regular_members",
                "manifest_sha256",
                "regular_payload_bytes",
            ]
            if hdr != expected_hdr:
                local_errs.append(
                    f"off_host_custody.shard_index_receipt header={hdr!r} != {expected_hdr!r}"
                )
            for rline in index_lines[1:]:
                cols = rline.split("\t")
                if len(cols) == 7 and cols[0] == "training-root":
                    training_row = {
                        "group": cols[0],
                        "archive": cols[1],
                        "archive_sha256": cols[2],
                        "compressed_bytes": int(cols[3]),
                        "regular_members": int(cols[4]),
                        "manifest_sha256": cols[5],
                        "regular_payload_bytes": int(cols[6]),
                    }
                    break
    except Exception as exc:
        local_errs.append(f"off_host_custody.shard_index_receipt parse failed: {exc}")

    if training_row is None:
        local_errs.append("off_host_custody.shard_index_receipt missing 'training-root' row")
    else:
        if training_row["manifest_sha256"] != manifest_sha:
            local_errs.append(
                f"off_host_custody.member_manifest_receipt sha256 ({manifest_sha}) != "
                f"SHARD-INDEX.tsv training-root manifest_sha256 ({training_row['manifest_sha256']})"
            )
        if not is_hex64(training_row["archive_sha256"]):
            local_errs.append(
                f"off_host_custody.shard_index_receipt training-root archive_sha256 invalid: {training_row['archive_sha256']!r}"
            )
        if (
            training_row["compressed_bytes"] <= 0
            or training_row["regular_members"] <= 0
            or training_row["regular_payload_bytes"] <= 0
        ):
            local_errs.append("off_host_custody.shard_index_receipt training-root counts must be positive")

    # Join SHARD-INDEX.tsv against FINAL-INDEX-SHARD-CUSTODY-JOIN.json
    sel = join_obj.get("selection") if isinstance(join_obj.get("selection"), dict) else {}
    if sel.get("index_sha256") != index_sha:
        local_errs.append(
            f"off_host_custody.custody_join_receipt selection.index_sha256 ({sel.get('index_sha256')!r}) != "
            f"shard_index_receipt sha256 ({index_sha})"
        )
    shards_list = join_obj.get("shards") if isinstance(join_obj.get("shards"), list) else []
    join_train_shard: dict[str, Any] | None = None
    for sh in shards_list:
        if isinstance(sh, dict) and sh.get("id") in ("training-root-part-000", "training-root"):
            join_train_shard = sh
            break
    if join_train_shard is None:
        local_errs.append("off_host_custody.custody_join_receipt missing training-root shard entry")
    elif training_row is not None:
        if (
            join_train_shard.get("archive_sha256") != training_row["archive_sha256"]
            or join_train_shard.get("compressed_bytes") != training_row["compressed_bytes"]
            or join_train_shard.get("regular_members") != training_row["regular_members"]
            or join_train_shard.get("raw_bytes") != training_row["regular_payload_bytes"]
            or not isinstance(join_train_shard.get("cloud_object"), str)
            or not join_train_shard.get("cloud_object", "").strip()
            or not is_strict_int(join_train_shard.get("cloud_bytes"))
            or join_train_shard.get("cloud_bytes", 0) <= 0
            or not isinstance(join_train_shard.get("cloud_MD5"), str)
            or len(join_train_shard.get("cloud_MD5", "")) != 32
        ):
            local_errs.append(
                "off_host_custody.custody_join_receipt training-root shard tuple mismatch against SHARD-INDEX.tsv"
            )

    # Parse authenticated member manifest lines
    parsed_manifest_shas: dict[str, set[str]] = {}
    try:
        for mline in manifest_raw.decode("utf-8").splitlines():
            if len(mline) >= 67 and mline[64:66] == "  " and is_hex64(mline[:64]):
                d_sha = mline[:64]
                rel_m = mline[66:].strip().lstrip("/")
                parsed_manifest_shas.setdefault(rel_m, set()).add(d_sha)
    except Exception as exc:
        local_errs.append(f"off_host_custody.member_manifest_receipt decode failed: {exc}")

    role_map_pins: dict[str, dict[str, tuple[str, str]]] = {}
    if "member_map_receipt" in cust_obj or "cloud_binding_receipt" in cust_obj:
        mm_raw, mm_obj = authenticate_file_pin(
            cust_obj.get("member_map_receipt"),
            outcomes_dir,
            "off_host_custody.member_map_receipt",
            local_errs,
            extra_roots=outcomes_extra_roots,
        )
        cb_raw, cb_obj = authenticate_file_pin(
            cust_obj.get("cloud_binding_receipt"),
            outcomes_dir,
            "off_host_custody.cloud_binding_receipt",
            local_errs,
            extra_roots=outcomes_extra_roots,
        )
        if mm_raw is not None and mm_obj is not None and cb_raw is not None and cb_obj is not None and training_row is not None:
            mm_sha = sha256_bytes(mm_raw)
            if mm_obj.get("schema") != "uor-r4.final-outcome-artifact-member-map/1":
                local_errs.append(
                    f"off_host_custody.member_map_receipt schema={mm_obj.get('schema')!r} != 'uor-r4.final-outcome-artifact-member-map/1'"
                )
            mm_idx = mm_obj.get("new_DATA_original_index") if isinstance(mm_obj.get("new_DATA_original_index"), dict) else {}
            mm_tr = mm_obj.get("training_root_shard") if isinstance(mm_obj.get("training_root_shard"), dict) else {}
            if mm_idx.get("sha256") != index_sha:
                local_errs.append("off_host_custody.member_map_receipt new_DATA_original_index.sha256 != shard_index_receipt.sha256")
            if (
                mm_tr.get("archive_sha256") != training_row["archive_sha256"]
                or mm_tr.get("manifest_sha256") != manifest_sha
            ):
                local_errs.append("off_host_custody.member_map_receipt training_root_shard mismatch against SHARD-INDEX.tsv")

            if (
                cb_obj.get("base_member_map_sha256") != mm_sha
                or cb_obj.get("actual_final_join", {}).get("sha256") != join_sha
                or cb_obj.get("metadata_tuple_joins") != "ALL_3_PASS"
            ):
                local_errs.append("off_host_custody.cloud_binding_receipt failed member_map/custody_join SHA binding checks")
            tr_bind = (
                cb_obj.get("training_role_members_cloud_binding")
                if isinstance(cb_obj.get("training_role_members_cloud_binding"), dict)
                else {}
            )
            if tr_bind.get("original_gzip_sha256") != training_row["archive_sha256"]:
                local_errs.append("off_host_custody.cloud_binding_receipt original_gzip_sha256 != training-root archive_sha256")
            if "same_reader_receipt" in tr_bind:
                authenticate_file_pin(
                    tr_bind.get("same_reader_receipt"),
                    outcomes_dir,
                    "off_host_custody.cloud_binding.same_reader_receipt",
                    local_errs,
                    extra_roots=outcomes_extra_roots,
                )
            for raw_k in ("cloud_index", "native_full_readback_log", "remote_postidentity"):
                if raw_k in tr_bind:
                    authenticate_raw_file_pin(
                        tr_bind.get(raw_k),
                        outcomes_dir,
                        f"off_host_custody.cloud_binding.{raw_k}",
                        local_errs,
                        extra_roots=outcomes_extra_roots,
                    )

            mm_roles = mm_obj.get("roles") if isinstance(mm_obj.get("roles"), list) else []
            for r_entry in mm_roles:
                if not isinstance(r_entry, dict) or not isinstance(r_entry.get("role"), str):
                    continue
                r_name = r_entry["role"]
                role_map_pins[r_name] = {}
                for k_field in ("final_report", "checkpoint_16404", "evaluation_16404", "checkpoint_00000"):
                    f_obj = r_entry.get(k_field)
                    if not isinstance(f_obj, dict):
                        local_errs.append(f"off_host_custody.member_map_receipt role {r_name} missing {k_field}")
                        continue
                    m_path = str(f_obj.get("member", "")).strip().lstrip("/")
                    m_sha = f_obj.get("raw_sha256")
                    if not m_path or not is_hex64(m_sha) or m_sha not in parsed_manifest_shas.get(m_path, set()):
                        local_errs.append(
                            f"off_host_custody.member_map_receipt role {r_name} {k_field} ({m_path}, {m_sha}) not in authenticated member_manifest_receipt"
                        )
                    else:
                        role_map_pins[r_name][k_field] = (m_path, m_sha)

    if local_errs:
        errors.extend(local_errs)
        return {}, {}
    return parsed_manifest_shas, role_map_pins


def extract_driver_eval_and_flags(
    val: dict[str, Any], label: str, errors: list[str]
) -> tuple[dict[str, Any] | None, dict[str, Any] | None]:
    """Match admission-rank-fixture-runner's driver_evaluation and driver_zero_update helpers."""
    driver_schema = "uor-r4.discrete-corpus-driver/1"
    objective_schema = "uor-r4.discrete-objective-evaluation/1"
    completion_schema = "uor-r4.completion-evaluation/1"
    qualification_schema = "uor-r4.redirect-qualification/1"
    schema = val.get("schema")
    if schema == qualification_schema:
        drv = val.get("initial_evaluation")
        if not isinstance(drv, dict):
            errors.append(f"{label}: redirect-qualification receipt has no initial_evaluation object")
            return None, None
        return extract_driver_eval_and_flags(drv, f"{label}.initial_evaluation", errors)
    if schema == completion_schema:
        drv = val.get("evaluation")
        if not isinstance(drv, dict):
            errors.append(f"{label}: completion envelope has no evaluation object")
            return None, None
        if drv.get("schema") != driver_schema:
            errors.append(f"{label}: nested evaluation schema={drv.get('schema')!r} != {driver_schema!r}")
            return None, None
        if drv.get("objective_evaluation_schema") != objective_schema:
            errors.append(
                f"{label}: nested objective_evaluation_schema={drv.get('objective_evaluation_schema')!r} != {objective_schema!r}"
            )
            return None, None
        agg = drv.get("aggregate")
        if not isinstance(agg, dict):
            errors.append(f"{label}: completion envelope has no evaluation.aggregate object")
            return None, None
        return agg, drv
    if schema == driver_schema:
        if val.get("objective_evaluation_schema") != objective_schema:
            errors.append(
                f"{label}: objective_evaluation_schema={val.get('objective_evaluation_schema')!r} != {objective_schema!r}"
            )
            return None, None
        agg = val.get("aggregate")
        if not isinstance(agg, dict):
            errors.append(f"{label}: driver receipt has no aggregate object")
            return None, None
        return agg, val
    errors.append(
        f"{label}: unrecognized evaluation schema {schema!r}; expected {driver_schema!r}, {completion_schema!r}, or {qualification_schema!r}"
    )
    return None, None


def verify_eval_zero_update_flags(
    flags: dict[str, Any],
    *,
    expected_epoch: int,
    expected_domain_a: str | None = None,
    expected_domain_b: str | None = None,
    label: str,
    errors: list[str],
) -> None:
    """Verify strict zero-update flags and snapshot/epoch invariants on an evaluation receipt (V3-S8)."""
    if "updates_applied_by_evaluation" not in flags or not is_strict_int(flags.get("updates_applied_by_evaluation")) or flags.get("updates_applied_by_evaluation") != 0:
        errors.append(
            f"{label}: updates_applied_by_evaluation={flags.get('updates_applied_by_evaluation')!r} must be present and == 0"
        )
    if "accepted_updates_in_model" not in flags or not is_strict_int(flags.get("accepted_updates_in_model")) or flags.get("accepted_updates_in_model") != expected_epoch:
        errors.append(
            f"{label}: accepted_updates_in_model={flags.get('accepted_updates_in_model')!r} must be present and == {expected_epoch}"
        )
    if flags.get("evaluation_mutates_model") is not False:
        errors.append(
            f"{label}: evaluation_mutates_model={flags.get('evaluation_mutates_model')!r} must be false"
        )
    if "model_epoch" in flags:
        if not is_strict_int(flags.get("model_epoch")) or flags.get("model_epoch") != expected_epoch:
            errors.append(f"{label}: model_epoch={flags.get('model_epoch')!r} != {expected_epoch}")
    if not is_hex64(flags.get("model_storage_sha256")):
        errors.append(f"{label}: model_storage_sha256={flags.get('model_storage_sha256')!r} is not valid 64-hex")
    elif expected_domain_a is not None and flags.get("model_storage_sha256") != expected_domain_a:
        errors.append(
            f"{label}: model_storage_sha256 ({flags.get('model_storage_sha256')}) != Domain A ({expected_domain_a})"
        )
    if not is_hex64(flags.get("model_snapshot_sha256")):
        errors.append(f"{label}: model_snapshot_sha256={flags.get('model_snapshot_sha256')!r} is not valid 64-hex")
    elif expected_domain_b is not None and flags.get("model_snapshot_sha256") != expected_domain_b:
        errors.append(
            f"{label}: model_snapshot_sha256 ({flags.get('model_snapshot_sha256')}) != Domain B ({expected_domain_b})"
        )
    if "before_snapshot_sha256" in flags or "after_snapshot_sha256" in flags:
        verify_zero_mutation_invariants(
            flags,
            "before_snapshot_sha256",
            "after_snapshot_sha256",
            "before_epoch" if "before_epoch" in flags else "model_epoch",
            "after_epoch" if "after_epoch" in flags else "accepted_updates_in_model",
            expected_digest=expected_domain_b,
            expected_epoch=expected_epoch,
            label=label,
            errors=errors,
        )


def load_contract(contract_path: pathlib.Path | None = None) -> dict[str, Any]:
    path = contract_path or (CONSUMER_DIR / "k16-rows-redirect-consumer-contract-v3.json")
    raw = path.read_bytes()
    return json.loads(raw)


def get_allowed_k16_arm_cfg_lin(contract: dict[str, Any], arm: str) -> set[tuple[str, str]]:
    arm_contract = contract["driver_source_and_arm_bindings_at_fb9e004"]["arms"][arm]
    allowed = {
        (
            arm_contract[dom]["rows_v1_typed_config_sha256"],
            arm_contract[dom]["rows_v1_lineage_sha256"],
        )
        for dom in ("repo_relative_paths", "codex_local_physical_paths", "pod_workspace_absolute_paths")
        if dom in arm_contract
    }
    if arm in POD_OPERATOR_ARM_BINDINGS:
        pod_b = POD_OPERATOR_ARM_BINDINGS[arm]
        allowed.add((pod_b["rows_v1_typed_config_sha256"], pod_b["rows_v1_lineage_sha256"]))
    return allowed


def verify_qualification_packet(
    qual_dir: pathlib.Path,
    contract: dict[str, Any],
    *,
    legacy_4cell_timing: bool = False,
) -> dict[str, Any]:
    """Verify rows-qualify (7 controls + negative reducer gate), k16-timing Ledger (2 checkpoint0 cells x 100 batches
    on rows_v1 under Owner Waiver 6251027125, or 4 cells with legacy_4cell_timing=True), and C-R1..C-R6
    SuiteExecutionSummary + pinned control/G-Pre receipts in qual_dir (native RUN layout, explicit FilePin
    manifest, or flat layout)."""
    errors: list[str] = []
    checks: list[str] = []

    base_qual_dir = qual_dir if qual_dir.is_dir() else qual_dir.parent
    if (
        (base_qual_dir / "private").is_dir()
        and (base_qual_dir / "private" / "finite2670-six-native-reports").is_dir()
        and not (base_qual_dir / "rows-positive").is_dir()
    ):
        base_qual_dir = base_qual_dir / "private"
    manifest_obj: dict[str, Any] | None = None
    manifest_path: pathlib.Path | None = None
    if qual_dir.is_file():
        manifest_path = qual_dir
    else:
        for cand_name in ("qualification-packet.json", "redirect-qualification.json"):
            cand_p = base_qual_dir / cand_name
            if cand_p.is_file():
                manifest_path = cand_p
                break
    if manifest_path is not None:
        try:
            manifest_obj = json.loads(manifest_path.read_text())
        except Exception as exc:
            errors.append(f"Invalid qualification manifest JSON at {manifest_path}: {exc}")

    def load_qual_artifact(
        manifest_keys: tuple[str, ...],
        rel_candidates: tuple[str, ...],
        label: str,
    ) -> tuple[pathlib.Path | None, dict[str, Any] | None]:
        if isinstance(manifest_obj, dict):
            q_sec = manifest_obj.get("qualification") if isinstance(manifest_obj.get("qualification"), dict) else {}
            for mk in manifest_keys:
                pin = manifest_obj.get(mk) if mk in manifest_obj else q_sec.get(mk)
                if isinstance(pin, dict) and "path" in pin and "sha256" in pin:
                    p = resolve_pin_path(pin["path"], base_qual_dir)
                    _, parsed = authenticate_file_pin(pin, base_qual_dir, f"manifest {label}", errors)
                    return p, parsed
        for rel in rel_candidates:
            p = base_qual_dir / rel
            if p.is_file():
                try:
                    parsed = json.loads(p.read_text())
                except Exception as exc:
                    errors.append(f"{label}: invalid JSON in {p}: {exc}")
                    return p, None
                if not isinstance(parsed, dict):
                    errors.append(f"{label}: top-level JSON in {p} is not an object")
                    return p, None
                return p, parsed
        errors.append(
            f"Missing {label}: checked {[ str(base_qual_dir / r) for r in rel_candidates ]}"
        )
        return None, None

    expected_driver_src = contract["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_sha256"]
    frozen_binds = contract["frozen_data_and_geometry_bindings"]
    native_runtime_identities: list[tuple[str, str, str, str]] = []
    timing_runtime_identities: list[tuple[str, str, str, str]] = []
    positive_rows_image_tuple: list[str] | None = None
    timing_rows_image_tuple: list[str] | None = None
    expected_timing_rows_image_tuple: list[str] | None = None
    timing_cell_digests: dict[tuple[str, str], str] = {}
    cr5_ptx_sha256: str | None = None
    native_k16_a_seed1_model: dict[str, Any] | None = None
    native_k16_r_seed1_model: dict[str, Any] | None = None
    native_seed1_eval_domain_b: dict[str, str] = {}

    if isinstance(manifest_obj, dict) and isinstance(manifest_obj.get("admitted_execution"), dict):
        adm_ex = manifest_obj["admitted_execution"]
        if (
            not is_hex64(adm_ex.get("source_sha256"))
            or not is_hex64(adm_ex.get("kernel_sha256"))
            or adm_ex.get("geometry_sha256") != frozen_binds["geometry_sha256"]
            or adm_ex.get("driver_source_sha256") != expected_driver_src
            or ("executable_sha256" in adm_ex and not is_hex64(adm_ex.get("executable_sha256")))
        ):
            errors.append("manifest admitted_execution failed source/kernel/geometry/driver_source/executable checks")
        else:
            native_runtime_identities.append(
                (
                    "manifest.admitted_execution",
                    adm_ex["source_sha256"],
                    adm_ex["kernel_sha256"],
                    adm_ex["geometry_sha256"],
                )
            )

    if isinstance(manifest_obj, dict) and isinstance(manifest_obj.get("timing_admitted_execution"), dict):
        t_adm = manifest_obj["timing_admitted_execution"]
        if (
            not is_hex64(t_adm.get("source_sha256"))
            or not is_hex64(t_adm.get("kernel_sha256"))
            or t_adm.get("geometry_sha256") != frozen_binds["geometry_sha256"]
        ):
            errors.append("manifest timing_admitted_execution failed source/kernel/geometry checks")
        else:
            timing_runtime_identities.append(
                (
                    "manifest.timing_admitted_execution",
                    t_adm["source_sha256"],
                    t_adm["kernel_sha256"],
                    t_adm["geometry_sha256"],
                )
            )

    native_id_log = base_qual_dir / "logs" / "compiled-identity.stdout.log"
    if native_id_log.is_file():
        try:
            nid = json.loads(native_id_log.read_text())
            n_comp = nid.get("compiled", {})
            if (
                nid.get("cuda_context_created") is not False
                or nid.get("training") != "HELD"
                or nid.get("driver_source_sha256") != expected_driver_src
                or not is_hex64(nid.get("executable_sha256"))
                or not isinstance(n_comp, dict)
                or not is_hex64(n_comp.get("source_sha256"))
                or not is_hex64(n_comp.get("kernel_sha256"))
                or n_comp.get("geometry_sha256") != frozen_binds["geometry_sha256"]
            ):
                errors.append("logs/compiled-identity.stdout.log failed native build-identity checks")
            else:
                native_runtime_identities.append(
                    (
                        "logs.compiled_identity",
                        n_comp["source_sha256"],
                        n_comp["kernel_sha256"],
                        n_comp["geometry_sha256"],
                    )
                )
        except Exception as exc:
            errors.append(f"Invalid logs/compiled-identity.stdout.log: {exc}")

    # 1. rows-qualify positive receipt (CURRENT-2 & CURRENT-7)
    rq_bind = contract["rows_qualify_consumer_binding"]
    _, rq = load_qual_artifact(
        ("rows_positive", "rows_parity", "rows_qualify_positive"),
        (
            "rows-positive/rows-parity.json",
            "finite2670-six-native-reports/rows-positive/rows-parity.json",
            "rows-parity.json",
        ),
        "rows-qualify positive receipt",
    )
    if rq is not None:
        for k_contract, k_json in [
            ("required_schema", "schema"),
            ("required_suite", "suite"),
            ("required_status", "status"),
            ("required_training_status", "training"),
            ("required_applied_updates", "applied_updates"),
        ]:
            if rq.get(k_json) != rq_bind[k_contract] or (
                k_json == "applied_updates" and not is_strict_int(rq.get(k_json))
            ):
                errors.append(
                    f"rows-parity.json {k_json}={rq.get(k_json)!r} != {rq_bind[k_contract]!r}"
                )
        rows_imgs = rq.get("rows_image_sha256")
        if not is_rows_image_sha256_tuple(rows_imgs, POSITIVE_ROWS_CU_SHA256):
            errors.append(
                f"rows-parity.json rows_image_sha256={rows_imgs!r} must be a 4-element list of 64-hex digests with [0]=={POSITIVE_ROWS_CU_SHA256!r}"
            )
        else:
            positive_rows_image_tuple = rows_imgs
            if isinstance(manifest_obj, dict) and "expected_rows_positive_image_sha256" in manifest_obj:
                if rows_imgs != manifest_obj["expected_rows_positive_image_sha256"]:
                    errors.append(
                        f"rows-parity.json rows_image_sha256={rows_imgs!r} != manifest expected_rows_positive_image_sha256"
                    )
        ctrls = rq.get("controls", [])
        if not isinstance(ctrls, list) or len(ctrls) != rq_bind["required_controls_count"]:
            errors.append(
                f"rows-parity.json controls len != {rq_bind['required_controls_count']}"
            )
        else:
            for exp_c, act_c in zip(rq_bind["required_controls_in_order"], ctrls):
                if not isinstance(act_c, dict):
                    errors.append(f"rows-parity.json control {exp_c['name']} is not an object")
                    continue
                if act_c.get("status") != "PASS":
                    errors.append(f"control {exp_c['name']} status={act_c.get('status')!r} != 'PASS'")
                for field in ("index", "name", "cases", "successes", "racy_refusals"):
                    if act_c.get(field) != exp_c[field] or (
                        field in ("index", "cases", "successes", "racy_refusals")
                        and not is_strict_int(act_c.get(field))
                    ):
                        errors.append(
                            f"control {exp_c['name']} {field}={act_c.get(field)!r} != {exp_c[field]!r}"
                        )
                if "refusals_len" in exp_c:
                    if not isinstance(act_c.get("refusals"), list) or len(act_c.get("refusals", [])) != exp_c["refusals_len"]:
                        errors.append(
                            f"control {exp_c['name']} refusals len != {exp_c['refusals_len']}"
                        )
                if exp_c["name"] == "rows_rank_hinge_active_parity":
                    if not is_strict_int(act_c.get("rank_occupied_cells")) or act_c.get("rank_occupied_cells", 0) < exp_c["rank_occupied_cells_min"]:
                        errors.append("rows_rank_hinge_active_parity rank_occupied_cells < 17")
                    if not is_strict_int(act_c.get("rank_active_rows")) or act_c.get("rank_active_rows", 0) < exp_c["rank_active_rows_min"]:
                        errors.append("rows_rank_hinge_active_parity rank_active_rows < 1 (vacuous R=0)")
                    if (
                        not is_strict_int(act_c.get("objective_sensitive_parameters"))
                        or act_c.get("objective_sensitive_parameters", 0)
                        < exp_c["objective_sensitive_parameters_min"]
                    ):
                        errors.append(
                            "rows_rank_hinge_active_parity objective_sensitive_parameters < 1"
                        )
        checks.append("rows-parity.json 7-control + 4-tuple rows_image_sha256 check completed")

    # 2. rows-qualify negative gate (V3-S7 & CURRENT-4 & CURRENT-7)
    _, rq_neg = load_qual_artifact(
        ("rows_negative", "rows_parity_negative", "rows_qualify_negative"),
        (
            "rows-negative/rows-parity.json",
            "finite2670-six-native-reports/rows-negative/rows-parity.json",
            "rows-negative/rows-parity-negative.json",
            "rows-parity-negative.json",
        ),
        "rows-qualify negative receipt",
    )
    if rq_neg is not None:
        if rq_neg.get("schema") != rq_bind["required_schema"]:
            errors.append(
                f"rows-parity-negative.json schema={rq_neg.get('schema')!r} != {rq_bind['required_schema']!r}"
            )
        if rq_neg.get("suite") != rq_bind["required_suite"]:
            errors.append(
                f"rows-parity-negative.json suite={rq_neg.get('suite')!r} != {rq_bind['required_suite']!r}"
            )
        if rq_neg.get("status") != "FAILED":
            errors.append(
                f"rows-parity-negative.json status={rq_neg.get('status')!r} != 'FAILED'"
            )
        if rq_neg.get("training") != "HELD":
            errors.append(
                f"rows-parity-negative.json training={rq_neg.get('training')!r} != 'HELD'"
            )
        if not is_strict_int(rq_neg.get("applied_updates")) or rq_neg.get("applied_updates") != 0:
            errors.append(
                f"rows-parity-negative.json applied_updates={rq_neg.get('applied_updates')!r} != 0"
            )
        if rq_neg.get("timing") != "NOT_RUN: parity failed":
            errors.append(
                f"rows-parity-negative.json timing={rq_neg.get('timing')!r} != 'NOT_RUN: parity failed'"
            )
        neg_imgs = rq_neg.get("rows_image_sha256")
        if not is_rows_image_sha256_tuple(neg_imgs, NEGATIVE_ROWS_CU_SHA256):
            errors.append(
                f"rows-parity-negative.json rows_image_sha256={neg_imgs!r} must be a 4-element list of 64-hex digests with [0]=={NEGATIVE_ROWS_CU_SHA256!r}"
            )
        elif positive_rows_image_tuple is not None and neg_imgs == positive_rows_image_tuple:
            errors.append(
                "rows-parity-negative.json rows_image_sha256 must differ from positive rows_image_sha256"
            )
        neg_list = rq_neg.get("controls", [])
        if not isinstance(neg_list, list) or len(neg_list) != rq_bind["required_controls_count"]:
            errors.append("rows-parity-negative.json controls len != 7")
        else:
            for exp_c, act_c in zip(rq_bind["required_controls_in_order"], neg_list):
                if not isinstance(act_c, dict):
                    errors.append(f"rows-parity-negative.json control {exp_c['name']} is not an object")
                    continue
                if act_c.get("index") != exp_c["index"] or not is_strict_int(act_c.get("index")):
                    errors.append(
                        f"rows-parity-negative.json control {exp_c['name']} index={act_c.get('index')!r} != {exp_c['index']}"
                    )
                if act_c.get("name") != exp_c["name"]:
                    errors.append(
                        f"rows-parity-negative.json control index {exp_c['index']} name={act_c.get('name')!r} != {exp_c['name']!r}"
                    )
                if exp_c["name"] == "rows_forced_terminal_parity":
                    if act_c.get("status") != "FAIL":
                        errors.append(
                            f"rows-parity-negative.json rows_forced_terminal_parity status={act_c.get('status')!r} != 'FAIL'"
                        )
                    if not is_strict_int(act_c.get("cases")) or act_c.get("cases") != exp_c["cases"]:
                        errors.append(
                            f"rows-parity-negative.json rows_forced_terminal_parity cases={act_c.get('cases')!r} != {exp_c['cases']}"
                        )
                    if "successes" in act_c:
                        errors.append(
                            f"rows-parity-negative.json rows_forced_terminal_parity must omit 'successes' on FAIL, got {act_c.get('successes')!r}"
                        )
                    err_str = act_c.get("error")
                    if not isinstance(err_str, str) or not all(
                        sub in err_str for sub in EXPECTED_NEGATIVE_REDUCER_ERROR_SUBSTRINGS
                    ):
                        errors.append(
                            f"rows-parity-negative.json rows_forced_terminal_parity error={err_str!r} missing required forced-terminal reducer loss mismatch substrings {EXPECTED_NEGATIVE_REDUCER_ERROR_SUBSTRINGS!r}"
                        )
                else:
                    if act_c.get("status") != "PASS":
                        errors.append(
                            f"rows-parity-negative.json control {exp_c['name']} status={act_c.get('status')!r} != 'PASS'"
                        )
                    for field in ("cases", "successes", "racy_refusals"):
                        if act_c.get(field) != exp_c[field] or not is_strict_int(act_c.get(field)):
                            errors.append(
                                f"rows-parity-negative.json control {exp_c['name']} {field}={act_c.get(field)!r} != {exp_c[field]!r}"
                            )
                    if "refusals_len" in exp_c:
                        if not isinstance(act_c.get("refusals"), list) or len(act_c.get("refusals", [])) != exp_c["refusals_len"]:
                            errors.append(
                                f"rows-parity-negative.json control {exp_c['name']} refusals len != {exp_c['refusals_len']}"
                            )
                    if exp_c["name"] == "rows_rank_hinge_active_parity":
                        if not is_strict_int(act_c.get("rank_occupied_cells")) or act_c.get("rank_occupied_cells", 0) < exp_c["rank_occupied_cells_min"]:
                            errors.append("rows-parity-negative.json rows_rank_hinge_active_parity rank_occupied_cells < 17")
                        if not is_strict_int(act_c.get("rank_active_rows")) or act_c.get("rank_active_rows", 0) < exp_c["rank_active_rows_min"]:
                            errors.append("rows-parity-negative.json rows_rank_hinge_active_parity rank_active_rows < 1")
                        if (
                            not is_strict_int(act_c.get("objective_sensitive_parameters"))
                            or act_c.get("objective_sensitive_parameters", 0)
                            < exp_c["objective_sensitive_parameters_min"]
                        ):
                            errors.append("rows-parity-negative.json rows_rank_hinge_active_parity objective_sensitive_parameters < 1")

        neg_mod_bind_path = base_qual_dir / "negative-module-binding.json"
        if neg_mod_bind_path.is_file():
            try:
                nmb = json.loads(neg_mod_bind_path.read_text())
                if (
                    nmb.get("expected_negative_module_sha256") != NEGATIVE_ROWS_SOURCE_RS_SHA256
                    or nmb.get("expected_positive_module_sha256") != POSITIVE_ROWS_SOURCE_RS_SHA256
                ):
                    errors.append("negative-module-binding.json positive/negative module sha256 mismatch")
            except Exception as exc:
                errors.append(f"Invalid negative-module-binding.json: {exc}")

        neg_exit_path = base_qual_dir / "logs" / "rows-negative.exit"
        if neg_exit_path.is_file() and neg_exit_path.read_text().strip() != "1":
            errors.append(
                f"logs/rows-negative.exit={neg_exit_path.read_text().strip()!r} != '1'"
            )
        pristine_exit_path = base_qual_dir / "logs" / "source-still-pristine.exit"
        if pristine_exit_path.is_file() and pristine_exit_path.read_text().strip() != "0":
            errors.append(
                f"logs/source-still-pristine.exit={pristine_exit_path.read_text().strip()!r} != '0'"
            )
        bins_log_path = base_qual_dir / "logs" / "binaries-sha256.stdout.log"
        if bins_log_path.is_file():
            pos_rq_sha: str | None = None
            neg_rq_sha: str | None = None
            for line in bins_log_path.read_text().splitlines():
                parts = line.strip().split()
                if len(parts) >= 2:
                    digest, bpath = parts[0], parts[-1]
                    if bpath.endswith("targets/native/release/rows-qualify"):
                        pos_rq_sha = digest
                    elif bpath.endswith("targets/negative/release/rows-qualify"):
                        neg_rq_sha = digest
            if not is_hex64(pos_rq_sha) or not is_hex64(neg_rq_sha) or pos_rq_sha == neg_rq_sha:
                errors.append(
                    "logs/binaries-sha256.stdout.log failed distinct 64-hex positive/negative rows-qualify binary check"
                )
        if isinstance(manifest_obj, dict) and isinstance(manifest_obj.get("negative_provenance"), dict):
            np = manifest_obj["negative_provenance"]
            if (
                np.get("exit_code") != 1
                or np.get("source_still_pristine") is not True
                or np.get("negative_rows_source_rs_sha256") != NEGATIVE_ROWS_SOURCE_RS_SHA256
                or np.get("positive_rows_source_rs_sha256") != POSITIVE_ROWS_SOURCE_RS_SHA256
                or not is_hex64(np.get("negative_executable_sha256"))
                or (
                    "positive_rows_qualify_executable_sha256" in np
                    and np.get("negative_executable_sha256") == np.get("positive_rows_qualify_executable_sha256")
                )
            ):
                errors.append("manifest negative_provenance failed exit_code/pristine/module/executable checks")
        checks.append("rows-parity-negative.json reducer negative gate + provenance check completed")

    # 3. 100-batch K16 timing receipt (rows_v1 Ledger schema, CURRENT-1, CURRENT-2, CURRENT-7, V3-S1 & V3-S8)
    tm_bind = contract["timing_receipt_consumer_binding"]
    tm_path, tm = load_qual_artifact(
        ("timing", "k16_timing"),
        (
            "timing/k16-timing-record.json",
            "timing-companion-measurement/k16-timing-record.json",
            "timing2670-native-metadata/timing-companion-measurement/k16-timing-record.json",
            "timing2670-native-metadata/k16-timing-record.json",
            "k16-timing-record.json",
            "k16-timing-report.json",
        ),
        "100-batch K16 timing Ledger receipt",
    )

    for cand_exp_img in (
        base_qual_dir / "timing-companion-checkpoint0" / "expected-rows-image.json",
        base_qual_dir / "timing2670-native-metadata" / "timing-companion-checkpoint0" / "expected-rows-image.json",
    ):
        if cand_exp_img.is_file():
            try:
                et_imgs = json.loads(cand_exp_img.read_text())
                if not is_rows_image_sha256_tuple(et_imgs, POSITIVE_ROWS_CU_SHA256):
                    errors.append(f"{cand_exp_img} is not a valid 4-element rows_image_sha256 tuple")
                else:
                    expected_timing_rows_image_tuple = et_imgs
            except Exception as exc:
                errors.append(f"Invalid {cand_exp_img}: {exc}")
            break
    if isinstance(manifest_obj, dict) and "timing_expected_rows_image_sha256" in manifest_obj:
        m_et_imgs = manifest_obj["timing_expected_rows_image_sha256"]
        if not is_rows_image_sha256_tuple(m_et_imgs, POSITIVE_ROWS_CU_SHA256):
            errors.append("manifest timing_expected_rows_image_sha256 is not a valid 4-element rows_image_sha256 tuple")
        elif expected_timing_rows_image_tuple is not None and expected_timing_rows_image_tuple != m_et_imgs:
            errors.append("manifest timing_expected_rows_image_sha256 != timing-companion-checkpoint0/expected-rows-image.json")
        else:
            expected_timing_rows_image_tuple = m_et_imgs

    for comp_id_log in (
        base_qual_dir / "timing-companion-metadata-continuation" / "companion-identity.stdout.log",
        base_qual_dir / "companion2670-metadata-continuation" / "companion-identity.stdout.log",
        base_qual_dir / "timing-companion-checkpoint0" / "companion-identity.stdout.log",
    ):
        if comp_id_log.is_file():
            try:
                cid = json.loads(comp_id_log.read_text())
                c_comp = cid.get("compiled", {})
                if (
                    cid.get("cuda_context_created") is not False
                    or cid.get("training") != "HELD"
                    or cid.get("driver_source_sha256") != expected_driver_src
                    or not is_hex64(cid.get("executable_sha256"))
                    or not isinstance(c_comp, dict)
                    or not is_hex64(c_comp.get("source_sha256"))
                    or not is_hex64(c_comp.get("kernel_sha256"))
                    or c_comp.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                ):
                    errors.append(f"{comp_id_log} failed companion build-identity checks")
                else:
                    timing_runtime_identities.append(
                        (
                            "timing_companion.companion_identity",
                            c_comp["source_sha256"],
                            c_comp["kernel_sha256"],
                            c_comp["geometry_sha256"],
                        )
                    )
            except Exception as exc:
                errors.append(f"Invalid {comp_id_log}: {exc}")
            break

    for comp_adm_path in (
        base_qual_dir / "timing-companion-checkpoint0" / "compiled-admitted.json",
        base_qual_dir / "timing-init2670-METADATA-ONLY" / "compiled-admitted.json",
    ):
        if comp_adm_path.is_file():
            try:
                cadm = json.loads(comp_adm_path.read_text())
                if (
                    not isinstance(cadm, dict)
                    or not is_hex64(cadm.get("source_sha256"))
                    or not is_hex64(cadm.get("kernel_sha256"))
                    or cadm.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                ):
                    errors.append(f"{comp_adm_path} failed checks")
                else:
                    timing_runtime_identities.append(
                        (
                            "timing_companion.compiled_admitted",
                            cadm["source_sha256"],
                            cadm["kernel_sha256"],
                            cadm["geometry_sha256"],
                        )
                    )
            except Exception as exc:
                errors.append(f"Invalid {comp_adm_path}: {exc}")
            break

    if tm is not None:
        if tm.get("schema") != tm_bind["required_record_schema"]:
            errors.append(
                f"k16-timing schema={tm.get('schema')!r} != {tm_bind['required_record_schema']!r}"
            )
        if tm.get("status") != tm_bind["required_status"]:
            errors.append(
                f"k16-timing status={tm.get('status')!r} != {tm_bind['required_status']!r}"
            )
        faults = tm.get("faults")
        if not isinstance(faults, list) or len(faults) != 0:
            errors.append(f"k16-timing faults must be empty list, got: {faults!r}")

        expected_cell_list = (
            [(c["source"], c["policy"]) for c in tm_bind["required_cells"]]
            if legacy_4cell_timing
            else list(ACTIVE_TIMING_CELLS_CHECKPOINT0)
        )
        expected_pairs = set(expected_cell_list)

        header = tm.get("header")
        if not isinstance(header, dict):
            errors.append("k16-timing missing Ledger header object")
        else:
            if header.get("schema") != tm_bind["required_plan_schema"]:
                errors.append(
                    f"k16-timing header.schema={header.get('schema')!r} != {tm_bind['required_plan_schema']!r}"
                )
            if header.get("counterfactual_path") != "rows_v1":
                errors.append(
                    f"k16-timing header.counterfactual_path={header.get('counterfactual_path')!r} != 'rows_v1'"
                )
            if not is_strict_int(header.get("batches_per_cell")) or header.get("batches_per_cell") != 100:
                errors.append(
                    f"k16-timing header.batches_per_cell={header.get('batches_per_cell')!r} != 100"
                )
            if (
                not is_strict_int(header.get("excluded_warmups_per_cell"))
                or header.get("excluded_warmups_per_cell") != 1
            ):
                errors.append(
                    f"k16-timing header.excluded_warmups_per_cell={header.get('excluded_warmups_per_cell')!r} != 1"
                )
            if header.get("apply") is not False:
                errors.append(f"k16-timing header.apply={header.get('apply')!r} != false")
            if "cells" in header:
                hdr_cells = header.get("cells")
                if not isinstance(hdr_cells, list) or [
                    (c.get("source"), c.get("policy")) for c in hdr_cells if isinstance(c, dict)
                ] != expected_cell_list:
                    errors.append(
                        f"k16-timing header.cells={hdr_cells!r} does not match expected cell list {expected_cell_list!r}"
                    )

        records = tm.get("records")
        if not isinstance(records, list):
            errors.append("k16-timing missing Ledger records[] list")
        else:
            cell_records = [
                r for r in records if isinstance(r, dict) and r.get("record_kind") == "cell"
            ]
            batch_records = [
                r for r in records if isinstance(r, dict) and r.get("record_kind") == "batch"
            ]
            if len(cell_records) != len(expected_cell_list):
                errors.append(
                    f"k16-timing cell records count={len(cell_records)} != {len(expected_cell_list)}"
                )
            else:
                seen_pairs: set[tuple[str | None, str | None]] = set()
                for cell_rec in cell_records:
                    cell_obj = cell_rec.get("cell")
                    if not isinstance(cell_obj, dict):
                        errors.append(f"k16-timing cell record missing cell object: {cell_rec!r}")
                        continue
                    pair = (cell_obj.get("source"), cell_obj.get("policy"))
                    seen_pairs.add(pair)
                    if cell_rec.get("counterfactual_path") != "rows_v1":
                        errors.append(f"cell {pair} counterfactual_path != 'rows_v1'")
                    if not is_strict_int(cell_rec.get("timed_batches")) or cell_rec.get("timed_batches") != 100:
                        errors.append(
                            f"cell {pair} timed_batches={cell_rec.get('timed_batches')!r} != 100"
                        )
                    if (
                        not is_strict_int(cell_rec.get("excluded_warmups"))
                        or cell_rec.get("excluded_warmups") != 1
                    ):
                        errors.append(
                            f"cell {pair} excluded_warmups={cell_rec.get('excluded_warmups')!r} != 1"
                        )
                    if cell_rec.get("applied") is not False:
                        errors.append(f"cell {pair} applied != false")
                    verify_zero_mutation_invariants(
                        cell_rec,
                        "snapshot_digest_before",
                        "snapshot_digest_after",
                        "epoch_before",
                        "epoch_after",
                        expected_epoch=0 if pair[0] == "checkpoint0" else None,
                        label=f"k16-timing cell {pair}",
                        errors=errors,
                    )
                    if isinstance(pair[0], str) and isinstance(pair[1], str) and is_hex64(cell_rec.get("snapshot_digest_before")):
                        timing_cell_digests[(pair[0], pair[1])] = cell_rec["snapshot_digest_before"]

                    identities = cell_rec.get("identities")
                    if not isinstance(identities, dict):
                        errors.append(f"cell {pair} missing identities object")
                    else:
                        c_rows_imgs = identities.get("rows_image_sha256")
                        if not is_rows_image_sha256_tuple(c_rows_imgs, POSITIVE_ROWS_CU_SHA256):
                            errors.append(
                                f"cell {pair} identities.rows_image_sha256={c_rows_imgs!r} must be a 4-element list of 64-hex digests with [0]=={POSITIVE_ROWS_CU_SHA256!r}"
                            )
                        else:
                            if timing_rows_image_tuple is None:
                                timing_rows_image_tuple = c_rows_imgs
                            elif c_rows_imgs != timing_rows_image_tuple:
                                errors.append(
                                    f"cell {pair} identities.rows_image_sha256={c_rows_imgs!r} != prior timing cell rows_image_sha256={timing_rows_image_tuple!r}"
                                )
                            if (
                                expected_timing_rows_image_tuple is not None
                                and c_rows_imgs != expected_timing_rows_image_tuple
                            ):
                                errors.append(
                                    f"cell {pair} identities.rows_image_sha256={c_rows_imgs!r} != expected timing rows_image_sha256={expected_timing_rows_image_tuple!r}"
                                )
                        comp_id = identities.get("compiled")
                        if not isinstance(comp_id, dict):
                            errors.append(f"cell {pair} identities missing compiled identity object")
                        elif (
                            not is_hex64(comp_id.get("source_sha256"))
                            or not is_hex64(comp_id.get("kernel_sha256"))
                            or comp_id.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                        ):
                            errors.append(
                                f"cell {pair} identities.compiled={comp_id!r} failed source/kernel/geometry checks"
                            )
                        else:
                            timing_runtime_identities.append(
                                (
                                    f"timing.cell.{pair[0]}.{pair[1]}",
                                    comp_id["source_sha256"],
                                    comp_id["kernel_sha256"],
                                    comp_id["geometry_sha256"],
                                )
                            )

                    cell_batches = [
                        b
                        for b in batch_records
                        if isinstance(b.get("cell"), dict)
                        and (b["cell"].get("source"), b["cell"].get("policy")) == pair
                    ]
                    if len(cell_batches) != 100:
                        errors.append(
                            f"cell {pair} batch record count={len(cell_batches)} != 100"
                        )
                    else:
                        for b in cell_batches:
                            if b.get("counterfactual_path") != "rows_v1":
                                errors.append(f"cell {pair} batch counterfactual_path != 'rows_v1'")
                                break
                            if b.get("applied") is not False:
                                errors.append(f"cell {pair} batch applied != false")
                                break
                            props = b.get("proposals", [])
                            if not isinstance(props, list) or len(props) == 0:
                                errors.append(f"cell {pair} batch proposals list empty")
                                break
                            if any(
                                p.get("counterfactual_path") != "rows_v1" or p.get("applied") is not False
                                for p in props
                                if isinstance(p, dict)
                            ):
                                errors.append(
                                    f"cell {pair} batch proposal counterfactual_path != 'rows_v1' or applied != false"
                                )
                                break
                if seen_pairs != expected_pairs:
                    errors.append(f"k16-timing cell pairs {seen_pairs} != {expected_pairs}")

        # Optional timing/checkpoint0-seed1-route-k16.snapshot.json cross-check
        if tm_path is not None:
            snap_cands = [
                tm_path.parent / "checkpoint0-seed1-route-k16.snapshot.json",
                base_qual_dir / "timing" / "checkpoint0-seed1-route-k16.snapshot.json",
            ]
            for sc in snap_cands:
                if sc.is_file():
                    try:
                        sc_model = json.loads(sc.read_text())
                        if isinstance(sc_model, dict) and "spec" in sc_model:
                            for pol in ("nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"):
                                m_copy = copy.deepcopy(sc_model)
                                m_copy["objective_policy"] = pol
                                exp_dom_b = sha256_bytes(native_snapshot_form(m_copy))
                                act_dig = timing_cell_digests.get(("checkpoint0", pol))
                                if act_dig is not None and act_dig != exp_dom_b:
                                    errors.append(
                                        f"k16-timing ('checkpoint0', {pol!r}) snapshot_digest_before={act_dig} != Domain B of {sc} ({exp_dom_b})"
                                    )
                    except Exception as exc:
                        errors.append(f"Failed to cross-check {sc}: {exc}")
                    break

        checks.append(
            f"k16-timing Ledger {len(expected_cell_list)}-cell x 100-batch rows_v1 check completed"
        )

    # 4. C-R1..C-R6 & G-Pre SuiteExecutionSummary report (CURRENT-3, CURRENT-5, CURRENT-7, V3-S2 & V3-S8)
    cr_bind = contract["c_r1_to_c_r6_and_g_pre_consumer_binding"]
    cr_path, cr = load_qual_artifact(
        ("c_r_suite", "c_r1_to_c_r6_suite"),
        (
            "c-r-fixtures/c-r-suite-summary.json",
            "cr2670-seven-native-reports/c-r-suite-summary.json",
            "c-r-fixtures/c-r-suite-report.json",
            "c-r-suite-summary.json",
            "c-r-suite-report.json",
        ),
        "C-R1..C-R6 suite summary",
    )
    if cr is not None and cr_path is not None:
        cr_extra_roots = [base_qual_dir, cr_path.parent]
        if cr.get("schema") != cr_bind["suite_schema"]:
            errors.append(f"c-r-suite schema={cr.get('schema')!r} != {cr_bind['suite_schema']!r}")
        for k, v in cr_bind["required_suite_fields"].items():
            if cr.get(k) != v or (isinstance(v, bool) and type(cr.get(k)) is not bool) or (is_strict_int(v) and not is_strict_int(cr.get(k))):
                errors.append(f"c-r-suite {k}={cr.get(k)!r} != {v!r}")
        cr_controls = cr.get("controls")
        if not isinstance(cr_controls, list) or len(cr_controls) != 6:
            errors.append(
                f"c-r-suite controls must be Vec<ControlReceipt> of length 6, got: {type(cr_controls).__name__}"
            )
        else:
            expected_ids = ["C-R1", "C-R2", "C-R3", "C-R4", "C-R5", "C-R6"]
            actual_ids = [c.get("id") if isinstance(c, dict) else None for c in cr_controls]
            if actual_ids != expected_ids:
                errors.append(f"c-r-suite control IDs {actual_ids} != {expected_ids}")
            for entry in cr_controls:
                if not isinstance(entry, dict):
                    errors.append("c-r-suite control entry is not an object")
                    continue
                cid = entry.get("id")
                if entry.get("status") != "PASS":
                    errors.append(f"c-r-suite control {cid} summary status={entry.get('status')!r} != 'PASS'")
                _, c_rep = authenticate_file_pin(
                    entry.get("receipt"),
                    cr_path.parent,
                    f"c-r-suite {cid} receipt",
                    errors,
                    extra_roots=cr_extra_roots,
                )
                if c_rep is None:
                    continue
                if c_rep.get("schema") != cr_bind["control_report_schema"]:
                    errors.append(
                        f"{cid} report schema={c_rep.get('schema')!r} != {cr_bind['control_report_schema']!r}"
                    )
                if c_rep.get("id") != cid:
                    errors.append(f"{cid} report id={c_rep.get('id')!r} != {cid!r}")
                if c_rep.get("status") != "PASS":
                    errors.append(f"{cid} report status={c_rep.get('status')!r} != 'PASS'")
                if not is_strict_int(c_rep.get("apply_calls")) or c_rep.get("apply_calls") != 0:
                    errors.append(f"{cid} report apply_calls={c_rep.get('apply_calls')!r} != 0")
                if c_rep.get("cpu_model_reference_used") is not False:
                    errors.append(
                        f"{cid} report cpu_model_reference_used={c_rep.get('cpu_model_reference_used')!r} != false"
                    )

                # Per-control registered predicates (C-R1..C-R6)
                if cid == "C-R1":
                    exp_hinges = [2, 1] + [0] * 14
                    exp_adm = [False, False] + [True] * 14
                    for fk in ("fixture_1a_mixed_distance", "fixture_1a_equal_distance_tie"):
                        f1a = c_rep.get(fk, {})
                        if (
                            not isinstance(f1a, dict)
                            or f1a.get("occupied_cells") != 5
                            or f1a.get("target_cell_rank") != 3
                            or f1a.get("cell_rank_hinge_k1_to_16") != exp_hinges
                            or f1a.get("target_admitted_k1_to_16") != exp_adm
                        ):
                            errors.append(f"C-R1 {fk} failed registered K=1..16 hinge/admission predicates")
                    f1b = c_rep.get("fixture_1b_all_256_cells_k16", {})
                    if (
                        not isinstance(f1b, dict)
                        or f1b.get("occupied_cells") != 256
                        or f1b.get("admitted_cells") != 16
                        or f1b.get("target_cell_rank") != 256
                        or f1b.get("cell_rank_hinge") != 240
                        or f1b.get("weighted_cell_rank_hinge_sum") != 7680
                    ):
                        errors.append("C-R1 fixture_1b_all_256_cells_k16 failed registered 256-cell boundary predicates")

                elif cid == "C-R2":
                    p2a = c_rep.get("part_2a_crowded_rival_cell", {})
                    if (
                        not isinstance(p2a, dict)
                        or p2a.get("multiplicities") != [1, 2, 4]
                        or p2a.get("target_cell_rank") != [3, 3, 3]
                        or p2a.get("cell_rank_hinge_k1") != [2, 2, 2]
                        or p2a.get("cell_rank_hinge_k2") != [1, 1, 1]
                        or p2a.get("cell_rank_hinge_k3") != [0, 0, 0]
                        or p2a.get("admitted_token_count_k3") != [3, 4, 6]
                    ):
                        errors.append("C-R2 part_2a_crowded_rival_cell failed registered crowding invariance predicates")
                    p2b = c_rep.get("part_2b_new_preceding_cell", {})
                    if (
                        not isinstance(p2b, dict)
                        or p2b.get("occupied_cells_before") != 5
                        or p2b.get("occupied_cells_after") != 6
                        or p2b.get("target_cell_rank_before") != 3
                        or p2b.get("target_cell_rank_after") != 4
                        or p2b.get("cell_rank_hinge_after_k1_to_k4") != [3, 2, 1, 0]
                    ):
                        errors.append("C-R2 part_2b_new_preceding_cell failed registered new-cell shift predicates")

                elif cid == "C-R3":
                    if c_rep.get("total_candidate_evaluations_verified") != 160:
                        errors.append(
                            f"C-R3 total_candidate_evaluations_verified={c_rep.get('total_candidate_evaluations_verified')!r} != 160"
                        )
                    muts = c_rep.get("mutations", [])
                    exp_muts = [
                        "create_cell",
                        "empty_cell",
                        "move_target_cell",
                        "interior_block_only",
                        "identity_candidate",
                    ]
                    if not isinstance(muts, list) or [m.get("mutation") for m in muts if isinstance(m, dict)] != exp_muts:
                        errors.append("C-R3 mutations list does not match the 5 registered mutation classes")
                    elif any(
                        m.get("passed") is not True or m.get("candidates_verified_per_policy") != 16
                        for m in muts
                    ):
                        errors.append("C-R3 mutation entry failed passed==true or candidates_verified_per_policy==16")

                elif cid == "C-R4":
                    occ_u = c_rep.get("occupancy_under_k", {})
                    m1 = occ_u.get("m1_at_k16", {}) if isinstance(occ_u, dict) else {}
                    m16 = occ_u.get("m16_at_k16", {}) if isinstance(occ_u, dict) else {}
                    abs_r = c_rep.get("absent_rival_checks", {})
                    if (
                        m1.get("occupied_cells") != 1
                        or m1.get("target_cell_rank") != 1
                        or m1.get("cell_rank_hinge") != 0
                        or m1.get("target_admitted") is not True
                    ):
                        errors.append("C-R4 occupancy_under_k.m1_at_k16 failed")
                    if (
                        m16.get("occupied_cells") != 16
                        or m16.get("target_admitted_rows") != 16
                        or m16.get("cell_rank_hinge_sum") != 0
                    ):
                        errors.append("C-R4 occupancy_under_k.m16_at_k16 failed")
                    if (
                        not isinstance(abs_r, dict)
                        or abs_r.get("wholly_ineligible_refusal_status") != 8
                        or abs_r.get("mixed_batch_skips_tr_zero_identically") is not True
                    ):
                        errors.append("C-R4 absent_rival_checks failed status 8 or tr==0 skip predicate")

                elif cid == "C-R5":
                    # CURRENT-3: Native objective.rs & cuda_aggregate.rs constants + 15 status-12 overflow cases
                    if (
                        c_rep.get("evaluation_schema") != "uor-r4.discrete-objective-evaluation/1"
                        or not is_strict_int(c_rep.get("row_words"))
                        or c_rep.get("row_words") != CR5_ROW_WORDS
                        or not is_strict_int(c_rep.get("aggregate_words"))
                        or c_rep.get("aggregate_words") != CR5_AGGREGATE_WORDS
                        or c_rep.get("histogram_bins") != CR5_HISTOGRAM_BINS
                        or c_rep.get("dual_loss_and_summary_aggregation_verified") is not True
                    ):
                        errors.append(
                            f"C-R5 failed bounded integer arithmetic / distribution constants (expected row_words={CR5_ROW_WORDS}, aggregate_words={CR5_AGGREGATE_WORDS}, histogram_bins={CR5_HISTOGRAM_BINS})"
                        )
                    s12 = c_rep.get("status12_overflow_checks")
                    if not isinstance(s12, dict):
                        errors.append("C-R5 status12_overflow_checks must be a CudaAggregateOverflowReport object")
                    else:
                        if not is_hex64(s12.get("ptx_sha256")):
                            errors.append(
                                f"C-R5 status12_overflow_checks.ptx_sha256={s12.get('ptx_sha256')!r} is not valid 64-hex"
                            )
                        else:
                            cr5_ptx_sha256 = s12["ptx_sha256"]
                        if s12.get("matches_device_compiled_kernel_sha256") is not True:
                            errors.append(
                                "C-R5 status12_overflow_checks.matches_device_compiled_kernel_sha256 != true"
                            )
                        if (
                            not is_strict_int(s12.get("baseline_valid_row_status"))
                            or s12.get("baseline_valid_row_status") != 0
                        ):
                            errors.append(
                                f"C-R5 status12_overflow_checks.baseline_valid_row_status={s12.get('baseline_valid_row_status')!r} != 0"
                            )
                        if s12.get("all_passed") is not True:
                            errors.append("C-R5 status12_overflow_checks.all_passed != true")
                        s12_cases = s12.get("cases")
                        if not isinstance(s12_cases, list) or len(s12_cases) != len(CR5_STATUS12_CASE_NAMES):
                            errors.append(
                                f"C-R5 status12_overflow_checks.cases must have {len(CR5_STATUS12_CASE_NAMES)} entries"
                            )
                        else:
                            act_case_names = [
                                (x.get("case") or x.get("name")) if isinstance(x, dict) else None
                                for x in s12_cases
                            ]
                            if act_case_names != CR5_STATUS12_CASE_NAMES:
                                errors.append(
                                    f"C-R5 status12_overflow_checks.cases names {act_case_names} != {CR5_STATUS12_CASE_NAMES}"
                                )
                            for sc in s12_cases:
                                if (
                                    not isinstance(sc, dict)
                                    or not is_strict_int(sc.get("observed_status"))
                                    or sc.get("observed_status") != 12
                                    or not is_strict_int(sc.get("expected_status"))
                                    or sc.get("expected_status") != 12
                                    or sc.get("passed") is not True
                                ):
                                    errors.append(
                                        f"C-R5 status12_overflow_checks case failed status==12/passed==true: {sc!r}"
                                    )

                elif cid == "C-R6":
                    if c_rep.get("stage_a_sweep_sha256") != STAGE_A_SWEEP_SHA256:
                        errors.append(
                            f"C-R6 stage_a_sweep_sha256={c_rep.get('stage_a_sweep_sha256')!r} != {STAGE_A_SWEEP_SHA256!r}"
                        )
                    if c_rep.get("baseline_admitted_cells_guard_range") != [1, 16]:
                        errors.append("C-R6 baseline_admitted_cells_guard_range != [1, 16]")
                    if c_rep.get("g_pre_k16_minimum") != 4894:
                        errors.append(f"C-R6 g_pre_k16_minimum={c_rep.get('g_pre_k16_minimum')!r} != 4894")
                    dev_checks = c_rep.get("v4096_checkpoint0_device_checks", [])
                    if not isinstance(dev_checks, list) or len(dev_checks) != 4:
                        errors.append("C-R6 v4096_checkpoint0_device_checks must have 4 entries")
                    else:
                        for dc in dev_checks:
                            if not isinstance(dc, dict) or dc.get("occupied_cells") != 16:
                                errors.append(f"C-R6 device check occupied_cells != 16: {dc!r}")
                            if dc.get("kept_cells") == 16 and (
                                dc.get("admitted_cells") != 16
                                or dc.get("target_admitted_rows") != 1
                                or dc.get("cell_rank_hinge_sum") != 0
                            ):
                                errors.append(f"C-R6 K=16 device check failed 100% admission: {dc!r}")

                    # Authenticate g_pre_k8_seed1_receipt FilePin & role lineage (CURRENT-5)
                    _, k8_val = authenticate_file_pin(
                        c_rep.get("g_pre_k8_seed1_receipt"),
                        cr_path.parent,
                        "C-R6 g_pre_k8_seed1_receipt",
                        errors,
                        extra_roots=cr_extra_roots,
                    )
                    if k8_val is not None:
                        k8_agg, k8_flags = extract_driver_eval_and_flags(
                            k8_val, "C-R6 g_pre_k8_seed1_receipt", errors
                        )
                        if k8_agg is not None and k8_flags is not None:
                            k8_init_ckpt_p = base_qual_dir / "g-pre-k8-A-seed1-init" / "checkpoint-00000.json"
                            k8_init_rep_cands = (
                                base_qual_dir / "g-pre-k8-A-seed1-init" / "report.json",
                                base_qual_dir / "gpre2670-metadata-only" / "k8-A-seed1-init-METADATA-ONLY.json",
                                base_qual_dir / "k8-A-seed1-init-METADATA-ONLY.json",
                            )
                            k8_exp_dom_a: str | None = None
                            k8_exp_dom_b: str | None = None
                            if k8_init_ckpt_p.is_file():
                                k8_init_raw = k8_init_ckpt_p.read_bytes()
                                k8_init_doms = compute_snapshot_hash_domains(
                                    json.loads(k8_init_raw), k8_init_raw
                                )
                                k8_exp_dom_a = k8_init_doms["domain_A_model_storage_sha256"]
                                k8_exp_dom_b = k8_init_doms["domain_B_typed_model_snapshot_sha256"]
                                if "driver_checkpoint_sha256" in k8_flags and k8_flags.get("driver_checkpoint_sha256") != k8_init_doms["domain_C_driver_checkpoint_sha256"]:
                                    errors.append("C-R6 g_pre_k8_seed1_receipt driver_checkpoint_sha256 != init Domain C")
                                if "checkpoint_file_sha256" in k8_flags and k8_flags.get("checkpoint_file_sha256") != k8_init_doms["domain_D_checkpoint_file_sha256"]:
                                    errors.append("C-R6 g_pre_k8_seed1_receipt checkpoint_file_sha256 != init Domain D")
                            else:
                                for k8_init_rep_p in k8_init_rep_cands:
                                    if k8_init_rep_p.is_file():
                                        k8_rep_raw = k8_init_rep_p.read_bytes()
                                        k8_rep_obj = json.loads(k8_rep_raw)
                                        k8_inner_m = k8_rep_obj.get("checkpoint", k8_rep_obj).get("model")
                                        if isinstance(k8_inner_m, dict) and "table" in k8_inner_m:
                                            k8_init_doms = compute_snapshot_hash_domains(k8_rep_obj, k8_rep_raw)
                                            k8_exp_dom_a = k8_init_doms["domain_A_model_storage_sha256"]
                                            k8_exp_dom_b = k8_init_doms["domain_B_typed_model_snapshot_sha256"]
                                        else:
                                            k8_exp_dom_a = k8_rep_obj.get("model_sha256") or k8_rep_obj.get("model_storage_sha256")
                                            k8_exp_dom_b = k8_rep_obj.get("model_snapshot_sha256")
                                        break
                            verify_eval_zero_update_flags(
                                k8_flags,
                                expected_epoch=0,
                                expected_domain_a=k8_exp_dom_a,
                                expected_domain_b=k8_exp_dom_b,
                                label="C-R6 g_pre_k8_seed1_receipt",
                                errors=errors,
                            )
                            if (
                                k8_agg.get("target_admitted_rows") != 5680
                                or k8_agg.get("top1_correct") != 17
                                or k8_agg.get("top8_correct") != 173
                                or k8_agg.get("token_strict_win_eligible_rows") != 105
                                or k8_agg.get("address_hinge_sum") != 261824
                                or k8_agg.get("occupied_cells") != 16
                            ):
                                errors.append(
                                    "C-R6 g_pre_k8_seed1_receipt aggregate failed to match sealed K=8 seed=1 baseline counts"
                                )
                            k8_lin = k8_flags.get("evaluation_lineage")
                            k8_mb = k8_flags.get("model_bindings")
                            if not isinstance(k8_lin, dict) or not isinstance(k8_lin.get("config"), dict):
                                errors.append("C-R6 g_pre_k8_seed1_receipt missing evaluation_lineage.config")
                            else:
                                k8_cfg = k8_lin["config"]
                                if (
                                    k8_lin.get("driver_source_sha256") != expected_driver_src
                                    or k8_cfg.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8}
                                    or k8_cfg.get("read_spec") != {"kind": "route"}
                                    or k8_cfg.get("seed") != 1
                                    or k8_cfg.get("objective_policy", "nearest_address_margin_v1") != "nearest_address_margin_v1"
                                ):
                                    errors.append("C-R6 g_pre_k8_seed1_receipt evaluation_lineage role mismatch (expected K=8, Route, seed=1, A)")
                                try:
                                    k8_cfg_sha = sha256_bytes(native_driver_config_form(k8_cfg))
                                    k8_lin_sha = sha256_bytes(compact_json_bytes(native_lineage_dict(k8_lin)))
                                    if k8_flags.get("evaluation_lineage_sha256") != k8_lin_sha:
                                        errors.append("C-R6 g_pre_k8_seed1_receipt evaluation_lineage_sha256 recompute mismatch")
                                    if (k8_cfg_sha, k8_lin_sha) not in K8_SEED1_ALLOWED_CFG_LIN:
                                        errors.append(
                                            f"C-R6 g_pre_k8_seed1_receipt (config_sha256={k8_cfg_sha}, lineage_sha256={k8_lin_sha}) not in registered K8 seed1 bindings"
                                        )
                                except Exception as exc:
                                    errors.append(f"C-R6 g_pre_k8_seed1_receipt lineage hash recompute failed: {exc}")
                            if not isinstance(k8_mb, dict):
                                errors.append("C-R6 g_pre_k8_seed1_receipt missing model_bindings object")
                            elif (
                                k8_mb.get("seed") != 1
                                or k8_mb.get("objective_policy", "nearest_address_margin_v1") != "nearest_address_margin_v1"
                                or k8_mb.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8}
                                or k8_mb.get("read_spec") != {"kind": "route"}
                                or k8_mb.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                                or not is_hex64(k8_mb.get("source_sha256"))
                                or not is_hex64(k8_mb.get("kernel_sha256"))
                            ):
                                errors.append("C-R6 g_pre_k8_seed1_receipt model_bindings failed K8 seed1 role/identity checks")
                            else:
                                native_runtime_identities.append(
                                    (
                                        "c_r6.g_pre_k8_seed1",
                                        k8_mb["source_sha256"],
                                        k8_mb["kernel_sha256"],
                                        k8_mb["geometry_sha256"],
                                    )
                                )

                    # Authenticate g_pre_k16_receipts (all 4 distinct K16 A/R x seed 1/2 roles, CURRENT-5)
                    k16_list = c_rep.get("g_pre_k16_receipts")
                    if not isinstance(k16_list, list) or len(k16_list) != 4:
                        errors.append(
                            f"C-R6 g_pre_k16_receipts must be a 4-element list, got: {k16_list!r}"
                        )
                    else:
                        seen_pin_paths: set[str] = set()
                        seen_pin_shas: set[str] = set()
                        for idx, (item, (arm_name, exp_seed, exp_policy)) in enumerate(
                            zip(k16_list, K16_FOUR_ROLES_IN_ORDER)
                        ):
                            if not isinstance(item, dict):
                                errors.append(f"C-R6 g_pre_k16_receipts[{idx}] is not an object")
                                continue
                            r_pin = item.get("receipt", item if "path" in item else None)
                            if isinstance(r_pin, dict):
                                p_str = str(r_pin.get("path", ""))
                                s_str = str(r_pin.get("sha256", ""))
                                if p_str in seen_pin_paths or s_str in seen_pin_shas:
                                    errors.append(
                                        f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) duplicates a previous FilePin path or sha256"
                                    )
                                seen_pin_paths.add(p_str)
                                seen_pin_shas.add(s_str)
                            _, g_val = authenticate_file_pin(
                                r_pin,
                                cr_path.parent,
                                f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name})",
                                errors,
                                extra_roots=cr_extra_roots,
                            )
                            if g_val is None:
                                continue
                            g_agg, g_flags = extract_driver_eval_and_flags(
                                g_val, f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name})", errors
                            )
                            if g_agg is None or g_flags is None:
                                continue

                            g_init_ckpt_p = base_qual_dir / f"g-pre-{arm_name}-init" / "checkpoint-00000.json"
                            g_init_rep_cands = (
                                base_qual_dir / f"g-pre-{arm_name}-init" / "report.json",
                                base_qual_dir / "gpre2670-metadata-only" / f"{arm_name}-init-METADATA-ONLY.json",
                                base_qual_dir / f"{arm_name}-init-METADATA-ONLY.json",
                            )
                            g_exp_dom_a: str | None = None
                            g_exp_dom_b: str | None = None
                            if g_init_ckpt_p.is_file():
                                g_init_raw = g_init_ckpt_p.read_bytes()
                                g_init_obj = json.loads(g_init_raw)
                                g_init_doms = compute_snapshot_hash_domains(g_init_obj, g_init_raw)
                                g_exp_dom_a = g_init_doms["domain_A_model_storage_sha256"]
                                g_exp_dom_b = g_init_doms["domain_B_typed_model_snapshot_sha256"]
                                if arm_name == "k16-A-seed1":
                                    native_k16_a_seed1_model = g_init_obj.get("checkpoint", g_init_obj).get("model")
                                elif arm_name == "k16-R-seed1":
                                    native_k16_r_seed1_model = g_init_obj.get("checkpoint", g_init_obj).get("model")
                                if "driver_checkpoint_sha256" in g_flags and g_flags.get("driver_checkpoint_sha256") != g_init_doms["domain_C_driver_checkpoint_sha256"]:
                                    errors.append(f"C-R6 {arm_name} driver_checkpoint_sha256 != init Domain C")
                                if "checkpoint_file_sha256" in g_flags and g_flags.get("checkpoint_file_sha256") != g_init_doms["domain_D_checkpoint_file_sha256"]:
                                    errors.append(f"C-R6 {arm_name} checkpoint_file_sha256 != init Domain D")
                            else:
                                for g_init_rep_p in g_init_rep_cands:
                                    if g_init_rep_p.is_file():
                                        g_rep_raw = g_init_rep_p.read_bytes()
                                        g_rep_obj = json.loads(g_rep_raw)
                                        g_inner_m = g_rep_obj.get("checkpoint", g_rep_obj).get("model")
                                        if isinstance(g_inner_m, dict) and "table" in g_inner_m:
                                            g_init_doms = compute_snapshot_hash_domains(g_rep_obj, g_rep_raw)
                                            g_exp_dom_a = g_init_doms["domain_A_model_storage_sha256"]
                                            g_exp_dom_b = g_init_doms["domain_B_typed_model_snapshot_sha256"]
                                            if arm_name == "k16-A-seed1":
                                                native_k16_a_seed1_model = g_inner_m
                                            elif arm_name == "k16-R-seed1":
                                                native_k16_r_seed1_model = g_inner_m
                                        else:
                                            g_exp_dom_a = g_rep_obj.get("model_sha256") or g_rep_obj.get("model_storage_sha256")
                                            g_exp_dom_b = g_rep_obj.get("model_snapshot_sha256")
                                        break

                            verify_eval_zero_update_flags(
                                g_flags,
                                expected_epoch=0,
                                expected_domain_a=g_exp_dom_a,
                                expected_domain_b=g_exp_dom_b,
                                label=f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name})",
                                errors=errors,
                            )

                            # Verify role lineage & model_bindings for this specific K16 arm (CURRENT-5)
                            g_lin = g_flags.get("evaluation_lineage")
                            g_mb = g_flags.get("model_bindings")
                            if not isinstance(g_lin, dict) or not isinstance(g_lin.get("config"), dict):
                                errors.append(
                                    f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) missing evaluation_lineage.config"
                                )
                            else:
                                g_cfg = g_lin["config"]
                                if (
                                    g_lin.get("driver_source_sha256") != expected_driver_src
                                    or g_cfg.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16}
                                    or g_cfg.get("read_spec") != {"kind": "route"}
                                    or g_cfg.get("counterfactual_path") != "rows_v1"
                                    or g_cfg.get("seed") != exp_seed
                                    or g_cfg.get("objective_policy", "nearest_address_margin_v1") != exp_policy
                                    or g_cfg.get("schedule") != "canonical_vector_then_read_v1"
                                ):
                                    errors.append(
                                        f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) evaluation_lineage role mismatch: seed={g_cfg.get('seed')!r} (expected {exp_seed}), policy={g_cfg.get('objective_policy', 'nearest_address_margin_v1')!r} (expected {exp_policy!r}), path={g_cfg.get('counterfactual_path')!r}"
                                    )
                                try:
                                    act_cfg_sha = sha256_bytes(native_driver_config_form(g_cfg))
                                    act_lin_sha = sha256_bytes(compact_json_bytes(native_lineage_dict(g_lin)))
                                    if g_flags.get("evaluation_lineage_sha256") != act_lin_sha:
                                        errors.append(
                                            f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) evaluation_lineage_sha256 != recomputed {act_lin_sha}"
                                        )
                                    if "training_lineage_sha256" in g_flags and g_flags.get("training_lineage_sha256") != act_lin_sha:
                                        errors.append(
                                            f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) training_lineage_sha256 != {act_lin_sha}"
                                        )
                                    allowed_set = get_allowed_k16_arm_cfg_lin(contract, arm_name)
                                    if (act_cfg_sha, act_lin_sha) not in allowed_set:
                                        errors.append(
                                            f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) (config_sha256={act_cfg_sha}, lineage_sha256={act_lin_sha}) does not match any registered rows_v1 binding for {arm_name}"
                                        )
                                except Exception as exc:
                                    errors.append(
                                        f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) lineage recompute failed: {exc}"
                                    )

                            if not isinstance(g_mb, dict):
                                errors.append(
                                    f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) missing model_bindings object"
                                )
                            elif (
                                g_mb.get("seed") != exp_seed
                                or g_mb.get("objective_policy", "nearest_address_margin_v1") != exp_policy
                                or g_mb.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16}
                                or g_mb.get("read_spec") != {"kind": "route"}
                                or g_mb.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                                or not is_hex64(g_mb.get("source_sha256"))
                                or not is_hex64(g_mb.get("kernel_sha256"))
                            ):
                                errors.append(
                                    f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) model_bindings failed role/identity checks"
                                )
                            else:
                                native_runtime_identities.append(
                                    (
                                        f"c_r6.g_pre_{arm_name}",
                                        g_mb["source_sha256"],
                                        g_mb["kernel_sha256"],
                                        g_mb["geometry_sha256"],
                                    )
                                )

                            if exp_seed == 1 and is_hex64(g_flags.get("model_snapshot_sha256")):
                                native_seed1_eval_domain_b[exp_policy] = g_flags["model_snapshot_sha256"]

                            t_adm = g_agg.get("target_admitted_rows")
                            if not is_strict_int(t_adm) or t_adm < 4894:
                                errors.append(
                                    f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) target_admitted_rows={t_adm!r} < 4894"
                                )
                            if "target_admitted_rows" in item and item.get("target_admitted_rows") != t_adm:
                                errors.append(
                                    f"C-R6 g_pre_k16_receipts[{idx}] ({arm_name}) summary target_admitted_rows={item.get('target_admitted_rows')!r} != receipt {t_adm}"
                                )
        checks.append("c-r-suite-summary.json C-R1..C-R6 + G-Pre 4-role FilePin & predicate check completed")

    # 5. Per-graph and cross-graph runtime identity, rows_image_sha256 4-tuple, and checkpoint0 Domain B checks
    native_src: str | None = None
    native_krn: str | None = None
    native_geo: str | None = None
    timing_src: str | None = None
    timing_krn: str | None = None
    timing_geo: str | None = None

    if native_runtime_identities:
        ref_n_lbl, native_src, native_krn, native_geo = native_runtime_identities[0]
        for lbl, s_sha, k_sha, g_sha in native_runtime_identities[1:]:
            if (s_sha, k_sha, g_sha) != (native_src, native_krn, native_geo):
                errors.append(
                    f"Native build-graph identity mismatch between {ref_n_lbl} ({native_src}, {native_krn}, {native_geo}) and {lbl} ({s_sha}, {k_sha}, {g_sha})"
                )

    if timing_runtime_identities:
        ref_t_lbl, timing_src, timing_krn, timing_geo = timing_runtime_identities[0]
        for lbl, s_sha, k_sha, g_sha in timing_runtime_identities[1:]:
            if (s_sha, k_sha, g_sha) != (timing_src, timing_krn, timing_geo):
                errors.append(
                    f"Timing build-graph identity mismatch between {ref_t_lbl} ({timing_src}, {timing_krn}, {timing_geo}) and {lbl} ({s_sha}, {k_sha}, {g_sha})"
                )

    if native_krn is not None and timing_krn is not None:
        if (native_krn, native_geo) != (timing_krn, timing_geo):
            errors.append(
                f"Cross-graph kernel/geometry mismatch between native ({native_krn}, {native_geo}) and timing ({timing_krn}, {timing_geo})"
            )

    admitted_krn = native_krn or timing_krn
    if cr5_ptx_sha256 is not None and admitted_krn is not None and cr5_ptx_sha256 != admitted_krn:
        errors.append(
            f"C-R5 status12_overflow_checks.ptx_sha256 ({cr5_ptx_sha256}) != admitted runtime kernel_sha256 ({admitted_krn})"
        )

    rows_image_component_equality: dict[str, bool] | None = None
    if positive_rows_image_tuple is not None and timing_rows_image_tuple is not None:
        rows_image_component_equality = {
            "rows_cu": positive_rows_image_tuple[0] == timing_rows_image_tuple[0],
            "rows_ptx": positive_rows_image_tuple[1] == timing_rows_image_tuple[1],
            "rows_cubin": positive_rows_image_tuple[2] == timing_rows_image_tuple[2],
            "rows_resources_report": positive_rows_image_tuple[3] == timing_rows_image_tuple[3],
        }
        same_build_graph = (
            native_src is not None
            and timing_src is not None
            and native_src == timing_src
            and expected_timing_rows_image_tuple is None
        )
        if same_build_graph:
            if timing_rows_image_tuple != positive_rows_image_tuple:
                errors.append(
                    f"Same-graph k16-timing rows_image_sha256={timing_rows_image_tuple!r} != rows-positive rows_image_sha256={positive_rows_image_tuple!r}"
                )
        else:
            if expected_timing_rows_image_tuple is None:
                errors.append(
                    "Cross-graph k16-timing build requires timing-companion-checkpoint0/expected-rows-image.json or manifest timing_expected_rows_image_sha256 to authenticate timing rows_image_sha256 4-tuple"
                )
            elif timing_rows_image_tuple != expected_timing_rows_image_tuple:
                errors.append(
                    f"Cross-graph k16-timing rows_image_sha256={timing_rows_image_tuple!r} != expected timing rows_image_sha256={expected_timing_rows_image_tuple!r}"
                )
            if timing_rows_image_tuple[:3] != positive_rows_image_tuple[:3]:
                errors.append(
                    f"Cross-graph k16-timing rows_image_sha256[:3]={timing_rows_image_tuple[:3]!r} != rows-positive rows_image_sha256[:3]={positive_rows_image_tuple[:3]!r} (rows.cu, rows.ptx, rows.cubin must match)"
                )

    def verify_shared_rlib_link_proof() -> None:
        for mdir in (
            base_qual_dir / "timing-companion-metadata-continuation",
            base_qual_dir / "companion2670-metadata-continuation",
        ):
            if not mdir.is_dir():
                continue
            lib_paths_p = mdir / "library-paths.txt"
            t_link_p = mdir / "timing-library-link.txt"
            c_link_p = mdir / "companion-library-link.txt"
            lib_sha_p = mdir / "library-sha256.stdout.log"
            if lib_paths_p.is_file() and t_link_p.is_file() and c_link_p.is_file():
                lines = [ln.strip() for ln in lib_paths_p.read_text().splitlines() if ln.strip()]
                rlib_path = lines[0] if lines else ""
                needle = f"--extern uor_r4_discrete={rlib_path}"
                if not rlib_path or needle not in t_link_p.read_text() or needle not in c_link_p.read_text():
                    errors.append(
                        f"{mdir.name} failed shared uor_r4_discrete rlib link proof"
                    )
                if lib_sha_p.is_file():
                    sha_parts = lib_sha_p.read_text().strip().split()
                    rlib_sha = sha_parts[0] if len(sha_parts) >= 2 else ""
                    sha_path = sha_parts[-1] if len(sha_parts) >= 2 else ""
                    if not is_hex64(rlib_sha) or sha_path != rlib_path:
                        errors.append(
                            f"{mdir.name}/library-sha256.stdout.log failed 64-hex rlib SHA-256 / path match with library-paths.txt"
                        )
                return
            r_deps_p = mdir / "release-deps-uor-r4-discrete.stdout.log"
            c_out_p = mdir / "companion-build-script-output.stdout.log"
            if r_deps_p.is_file() and c_out_p.is_file():
                parts = r_deps_p.read_text().strip().split()
                rlib_sha = parts[0] if len(parts) >= 2 else ""
                rlib_name = pathlib.Path(parts[-1]).name if len(parts) >= 2 else ""
                if not is_hex64(rlib_sha) or not rlib_name.endswith(".rlib") or rlib_name not in c_out_p.read_text():
                    errors.append(
                        f"{mdir.name} failed release-deps / companion-build-script shared rlib check"
                    )
                return
        for cdir in (
            base_qual_dir / "timing-companion-checkpoint0",
            base_qual_dir / "timing-init2670-METADATA-ONLY",
        ):
            t_proof_p = cdir / "timing-library-proof.txt"
            c_proof_p = cdir / "companion-library-proof.txt"
            if t_proof_p.is_file() and c_proof_p.is_file():
                t_ext = next(
                    (tok for tok in t_proof_p.read_text().split() if tok.startswith("uor_r4_discrete=")),
                    None,
                )
                c_ext = next(
                    (tok for tok in c_proof_p.read_text().split() if tok.startswith("uor_r4_discrete=")),
                    None,
                )
                if not t_ext or t_ext != c_ext:
                    errors.append(
                        f"{cdir.name} timing-library-proof.txt and companion-library-proof.txt failed shared uor_r4_discrete rlib link proof"
                    )
                return
        errors.append(
            "Cross-graph timing-companion witness missing shared uor_r4_discrete .rlib link proof"
        )

    if native_seed1_eval_domain_b and timing_cell_digests:
        if native_src == timing_src:
            for pol, n_dom_b in native_seed1_eval_domain_b.items():
                if ("checkpoint0", pol) in timing_cell_digests:
                    t_dig = timing_cell_digests[("checkpoint0", pol)]
                    if n_dom_b != t_dig:
                        errors.append(
                            f"Same-graph C-R6 seed1 {pol} model_snapshot_sha256 ({n_dom_b}) != k16-timing ('checkpoint0', {pol!r}) snapshot_digest_before ({t_dig})"
                        )
        else:
            fresh_snap_path = base_qual_dir / "timing-companion-checkpoint0" / "checkpoint0-seed1-route-k16.snapshot.json"
            state_eq_exit = base_qual_dir / "timing-companion-checkpoint0" / "state-equality.exit"
            timing_pin_p = base_qual_dir / "timing-companion-checkpoint0" / "timing-pin.json"
            meta_snap_cands = (
                base_qual_dir / "timing-companion-checkpoint0" / "METADATA-ONLY-fresh-snapshot.json",
                base_qual_dir / "timing-init2670-METADATA-ONLY" / "METADATA-ONLY-fresh-snapshot.json",
            )
            meta_snap_path = next((p for p in meta_snap_cands if p.is_file()), None)
            tcw = manifest_obj.get("timing_companion_witness") if isinstance(manifest_obj, dict) else None

            if fresh_snap_path.is_file():
                try:
                    fresh_raw = fresh_snap_path.read_bytes()
                    fresh_snap = json.loads(fresh_raw)
                    if (
                        fresh_snap.get("source_sha256") != timing_src
                        or fresh_snap.get("kernel_sha256") != timing_krn
                        or fresh_snap.get("geometry_sha256") != timing_geo
                    ):
                        errors.append(
                            "timing-companion-checkpoint0/checkpoint0-seed1-route-k16.snapshot.json source/kernel/geometry mismatch with timing build graph"
                        )
                    if timing_pin_p.is_file():
                        tpin = json.loads(timing_pin_p.read_text())
                        if tpin.get("checkpoint0", {}).get("snapshot_sha256") != sha256_bytes(fresh_raw):
                            errors.append(
                                "timing-companion-checkpoint0/timing-pin.json checkpoint0.snapshot_sha256 != fresh snapshot file SHA-256"
                            )
                    elif native_k16_a_seed1_model is None:
                        errors.append(
                            "timing-companion-checkpoint0/timing-pin.json required when native k16-A-seed1 full model is absent"
                        )
                    if state_eq_exit.is_file() and state_eq_exit.read_text().strip() != "0":
                        errors.append("timing-companion-checkpoint0/state-equality.exit != '0'")
                    if native_k16_a_seed1_model is not None:
                        f_form = json.loads(native_snapshot_form(fresh_snap))
                        n_form = json.loads(native_snapshot_form(native_k16_a_seed1_model))
                        f_form.pop("source_sha256", None)
                        n_form.pop("source_sha256", None)
                        if f_form != n_form:
                            errors.append(
                                "timing-companion fresh checkpoint0 snapshot differs from native G-Pre k16-A-seed1 model outside source_sha256"
                            )
                    elif not (state_eq_exit.is_file() and state_eq_exit.read_text().strip() == "0"):
                        errors.append(
                            "timing-companion fresh checkpoint0 snapshot requires either native k16-A-seed1 model state or state-equality.exit == '0'"
                        )
                    verify_shared_rlib_link_proof()
                    for pol in ("nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"):
                        if ("checkpoint0", pol) in timing_cell_digests:
                            snap_pol = copy.deepcopy(fresh_snap)
                            snap_pol["objective_policy"] = pol
                            exp_t_dom_b = sha256_bytes(native_snapshot_form(snap_pol))
                            if timing_cell_digests[("checkpoint0", pol)] != exp_t_dom_b:
                                errors.append(
                                    f"k16-timing ('checkpoint0', {pol!r}) snapshot_digest_before ({timing_cell_digests[('checkpoint0', pol)]}) != timing-companion fresh snapshot Domain B ({exp_t_dom_b})"
                                )
                except Exception as exc:
                    errors.append(f"Invalid timing-companion-checkpoint0 witness: {exc}")
            elif meta_snap_path is not None:
                try:
                    ms_dir = meta_snap_path.parent
                    ms_obj = json.loads(meta_snap_path.read_text())
                    if (
                        ms_obj.get("source_sha256") != timing_src
                        or ms_obj.get("kernel_sha256") != timing_krn
                        or ms_obj.get("geometry_sha256") != timing_geo
                        or ms_obj.get("seed") != 1
                        or ms_obj.get("epoch") != 0
                        or ms_obj.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16}
                        or ms_obj.get("read_spec") != {"kind": "route"}
                        or ms_obj.get("table_length") != 131072
                        or ms_obj.get("routes_length") != 20
                        or ms_obj.get("gates_length") != 20
                    ):
                        errors.append(f"{meta_snap_path} failed timing-companion checkpoint0 metadata checks")
                    ms_state_eq = ms_dir / "state-equality.exit"
                    if not ms_state_eq.is_file() or ms_state_eq.read_text().strip() != "0":
                        errors.append(f"{ms_state_eq} missing or != '0' (state-except-source equality failed)")
                    ms_tpin_p = ms_dir / "timing-pin.json"
                    ms_fpins_p = ms_dir / "fresh-pins.stdout.log"
                    if not ms_tpin_p.is_file() or not ms_fpins_p.is_file():
                        errors.append(f"{ms_dir} missing timing-pin.json or fresh-pins.stdout.log")
                    else:
                        ms_tpin = json.loads(ms_tpin_p.read_text())
                        c0_snap_sha = ms_tpin.get("checkpoint0", {}).get("snapshot_sha256")
                        if not is_hex64(c0_snap_sha) or c0_snap_sha != timing_cell_digests.get(("checkpoint0", "nearest_address_margin_v1")):
                            errors.append(
                                f"{ms_tpin_p} checkpoint0.snapshot_sha256 ({c0_snap_sha}) != k16-timing ('checkpoint0', 'nearest_address_margin_v1') snapshot_digest_before"
                            )
                        fpins_text = ms_fpins_p.read_text()
                        if not is_hex64(c0_snap_sha) or c0_snap_sha not in fpins_text:
                            errors.append(f"{ms_fpins_p} does not pin {c0_snap_sha}")
                        r_snap_sha = timing_cell_digests.get(("checkpoint0", "occupied_cell_rank_hinge_v1"))
                        if not is_hex64(r_snap_sha) or r_snap_sha == c0_snap_sha:
                            errors.append(
                                f"k16-timing ('checkpoint0', 'occupied_cell_rank_hinge_v1') snapshot_digest_before ({r_snap_sha}) invalid or not distinct from A digest"
                            )
                    if native_k16_a_seed1_model is not None:
                        for pol in ("nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"):
                            if ("checkpoint0", pol) in timing_cell_digests:
                                snap_pol = copy.deepcopy(native_k16_a_seed1_model)
                                snap_pol["source_sha256"] = timing_src
                                snap_pol["objective_policy"] = pol
                                exp_t_dom_b = sha256_bytes(native_snapshot_form(snap_pol))
                                if timing_cell_digests[("checkpoint0", pol)] != exp_t_dom_b:
                                    errors.append(
                                        f"k16-timing ('checkpoint0', {pol!r}) snapshot_digest_before ({timing_cell_digests[('checkpoint0', pol)]}) != state-except-source timing Domain B ({exp_t_dom_b})"
                                    )
                    verify_shared_rlib_link_proof()
                except Exception as exc:
                    errors.append(f"Invalid metadata-only timing-companion checkpoint0 witness at {meta_snap_path}: {exc}")
            elif isinstance(tcw, dict):
                if (
                    tcw.get("state_except_source_equal") is not True
                    or not is_hex64(tcw.get("shared_rlib_sha256"))
                    or tcw.get("native_source_sha256") != native_src
                    or tcw.get("timing_source_sha256") != timing_src
                ):
                    errors.append(
                        "manifest timing_companion_witness failed state_except_source_equal/shared_rlib_sha256/source_sha256 checks"
                    )
                verify_shared_rlib_link_proof()
                for pol in ("nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"):
                    if ("checkpoint0", pol) in timing_cell_digests:
                        t_dig = timing_cell_digests[("checkpoint0", pol)]
                        ref_model = native_k16_a_seed1_model if pol == "nearest_address_margin_v1" else (native_k16_r_seed1_model or native_k16_a_seed1_model)
                        if ref_model is not None:
                            snap_pol = copy.deepcopy(ref_model)
                            snap_pol["source_sha256"] = timing_src
                            snap_pol["objective_policy"] = pol
                            exp_t_dom_b = sha256_bytes(native_snapshot_form(snap_pol))
                            if t_dig != exp_t_dom_b:
                                errors.append(
                                    f"k16-timing ('checkpoint0', {pol!r}) snapshot_digest_before ({t_dig}) != state-except-source timing Domain B ({exp_t_dom_b})"
                                )
                        else:
                            w_t_map = tcw.get("timing_checkpoint0_domain_b_by_policy", {})
                            w_n_map = tcw.get("native_checkpoint0_domain_b_by_policy", {})
                            if (
                                not isinstance(w_t_map, dict)
                                or not isinstance(w_n_map, dict)
                                or w_t_map.get(pol) != t_dig
                                or w_n_map.get(pol) != native_seed1_eval_domain_b.get(pol)
                            ):
                                errors.append(
                                    f"timing_companion_witness failed Domain B policy map check for {pol!r}"
                                )
            else:
                errors.append(
                    f"Cross-graph timing source_sha256 ({timing_src}) differs from native G-Pre source_sha256 ({native_src}) without authenticated timing-companion-checkpoint0 state-except-source witness"
                )

    # 6. Optional finite control suites in native RUN layout (qualify-route, qualify-flock, qualify-driver, qualify-relation)
    for rel_ctrl in (
        "qualify-route/controls.json",
        "qualify-flock/flock-controls.json",
        "qualify-driver/controls.json",
        "qualify-relation/relation-controls.json",
    ):
        cp = base_qual_dir / rel_ctrl
        if not cp.is_file() and (base_qual_dir / "finite2670-six-native-reports" / rel_ctrl).is_file():
            cp = base_qual_dir / "finite2670-six-native-reports" / rel_ctrl
        if cp.is_file():
            try:
                co = json.loads(cp.read_text())
                if rel_ctrl == "qualify-driver/controls.json":
                    driver_updates_ok = (
                        is_strict_int(co.get("applied_updates"))
                        and (
                            co.get("applied_updates") == 0
                            or (
                                co.get("applied_updates") == 8
                                and co.get("corpus_updates") == 0
                                and co.get("expected_applied_updates_on_complete_pass") == 8
                                and co.get("apply_dose_matches") is True
                                and co.get("update_count_complete") is True
                            )
                        )
                    )
                    if co.get("status") != "PASS" or co.get("training") != "HELD" or not driver_updates_ok:
                        errors.append(
                            f"{rel_ctrl} failed status=='PASS'/training=='HELD'/(applied_updates==8 with corpus_updates==0 or applied_updates==0)"
                        )
                else:
                    if (
                        co.get("status") != "PASS"
                        or co.get("training") != "HELD"
                        or not is_strict_int(co.get("applied_updates"))
                        or co.get("applied_updates") != 0
                    ):
                        errors.append(f"{rel_ctrl} failed status=='PASS'/training=='HELD'/applied_updates==0")
            except Exception as exc:
                errors.append(f"Invalid JSON in {cp}: {exc}")

    return {
        "status": "PASS" if not errors else "FAIL",
        "checks": checks,
        "errors": errors,
        "per_graph_identities": {
            "native": {
                "source_sha256": native_src,
                "kernel_sha256": native_krn,
                "geometry_sha256": native_geo,
                "rows_image_sha256": positive_rows_image_tuple,
            },
            "timing": {
                "source_sha256": timing_src,
                "kernel_sha256": timing_krn,
                "geometry_sha256": timing_geo,
                "rows_image_sha256": timing_rows_image_tuple,
            },
            "rows_image_component_equality": rows_image_component_equality,
        },
    }


def evaluate_outcomes(outcomes_dir: pathlib.Path, contract: dict[str, Any]) -> dict[str, Any]:
    """Evaluate 4 trained trajectories (K16-A and K16-R x seeds {1, 2}) at cursor 16,404
    by authenticating per-role CompletionAdmission manifests (uor-r4.redirect-owner-admission/1),
    per-role Qualification / PMReview / CompletionOwner receipts, per-role CompletionReport
    receipts (uor-r4.completion-report/1), bound checkpoint + evaluation receipts across all 4
    hash domains (A..D), runtime source/kernel/executable identities, and checkpoint0 Domain A/B
    digests, then emitting the pre-registered outcome decision from
    m1-discrete-rung0-admission-redirect-2026-10-15.md."""
    summary_path = outcomes_dir / "trajectories-summary.json"
    if not summary_path.exists():
        return {
            "status": "FAIL",
            "decision": "INSTRUMENT_OR_RECEIPT_FAULT",
            "errors": [f"Missing {summary_path}"],
        }
    data = json.loads(summary_path.read_text())
    errors: list[str] = []
    arms = data.get("arms", {})
    required_arms = ("k16-A-seed1", "k16-A-seed2", "k16-R-seed1", "k16-R-seed2")

    outcomes_extra_roots: list[pathlib.Path] = [
        outcomes_dir,
        outcomes_dir / "metadata",
        outcomes_dir / "source-releases-private",
        outcomes_dir.parent,
    ]
    if STAGED_K16_METADATA_DIR.is_dir():
        outcomes_extra_roots.extend(
            [
                STAGED_K16_METADATA_DIR,
                STAGED_K16_METADATA_DIR / "metadata",
                STAGED_K16_METADATA_DIR / "source-releases-private",
            ]
        )

    if SEALED_K16_CUSTODY_DIR.is_dir():
        outcomes_extra_roots.append(SEALED_K16_CUSTODY_DIR)
    if SEALED_K16_MEMBER_MAP_DIR.is_dir():
        outcomes_extra_roots.append(SEALED_K16_MEMBER_MAP_DIR)

    # Authenticate explicit off_host_custody FilePin bundle + SHARD-INDEX.tsv / custody-join / member-map
    # provenance before honoring any off-host raw checkpoint-16404.json shortcut.
    member_manifest_shas, role_member_map_pins = authenticate_off_host_custody_manifests(
        data, outcomes_dir, outcomes_extra_roots, errors
    )

    arm_q4_agg: dict[str, dict[str, Any]] = {}
    arm_c0_agg: dict[str, dict[str, Any]] = {}
    arm_domains: dict[str, dict[str, str]] = {}
    arm_epochs: dict[str, int] = {}
    arm_lineages: dict[str, str] = {}
    arm_admission_shas: dict[str, str] = {}
    seen_role_admission_shas: set[str] = set()

    expected_driver_src = contract["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_sha256"]
    frozen_binds = contract["frozen_data_and_geometry_bindings"]

    # Resolve optional top-level runtime execution identity (CURRENT-6 & CURRENT-8);
    # authoritative per-role execution identity is authenticated per arm from admission_receipt.plan.execution.
    admitted_source_sha256: str | None = None
    admitted_kernel_sha256: str | None = None
    admitted_executable_sha256: str | None = None

    if "qualification_receipt" in data and isinstance(data.get("qualification_receipt"), dict):
        _, top_q_obj = authenticate_file_pin(
            data.get("qualification_receipt"),
            outcomes_dir,
            "top-level qualification_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if top_q_obj is not None:
            if top_q_obj.get("schema") != "uor-r4.redirect-qualification/1":
                errors.append("top-level qualification_receipt schema != 'uor-r4.redirect-qualification/1'")
            top_ex = top_q_obj.get("execution") or top_q_obj.get("admitted_execution")
            if isinstance(top_ex, dict):
                admitted_source_sha256 = top_ex.get("source_sha256")
                admitted_kernel_sha256 = top_ex.get("kernel_sha256")
                admitted_executable_sha256 = top_ex.get("executable_sha256")
                if top_ex.get("geometry_sha256") != frozen_binds["geometry_sha256"]:
                    errors.append("top-level qualification_receipt execution.geometry_sha256 mismatch")
                if top_ex.get("driver_source_sha256") != expected_driver_src:
                    errors.append("top-level qualification_receipt execution.driver_source_sha256 mismatch")

    if "build_identity_receipt" in data:
        _, b_obj = authenticate_file_pin(
            data.get("build_identity_receipt"),
            outcomes_dir,
            "build_identity_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if b_obj is not None:
            b_src = b_obj.get("source_sha256")
            b_krn = b_obj.get("kernel_sha256")
            b_exe = b_obj.get("executable_sha256")
            if b_obj.get("geometry_sha256") != frozen_binds["geometry_sha256"]:
                errors.append("build_identity_receipt geometry_sha256 mismatch")
            if "driver_source_sha256" in b_obj and b_obj.get("driver_source_sha256") != expected_driver_src:
                errors.append("build_identity_receipt driver_source_sha256 mismatch")
            if admitted_source_sha256 is not None and b_src != admitted_source_sha256:
                errors.append("build_identity_receipt source_sha256 != admitted source_sha256")
            if admitted_kernel_sha256 is not None and b_krn != admitted_kernel_sha256:
                errors.append("build_identity_receipt kernel_sha256 != admitted kernel_sha256")
            if admitted_executable_sha256 is not None and b_exe is not None and b_exe != admitted_executable_sha256:
                errors.append("build_identity_receipt executable_sha256 != admitted executable_sha256")
            admitted_source_sha256 = admitted_source_sha256 or b_src
            admitted_kernel_sha256 = admitted_kernel_sha256 or b_krn
            admitted_executable_sha256 = admitted_executable_sha256 or b_exe

    if isinstance(data.get("admitted_execution"), dict):
        adm_ex = data["admitted_execution"]
        a_src = adm_ex.get("source_sha256")
        a_krn = adm_ex.get("kernel_sha256")
        a_exe = adm_ex.get("executable_sha256")
        if adm_ex.get("geometry_sha256", frozen_binds["geometry_sha256"]) != frozen_binds["geometry_sha256"]:
            errors.append("admitted_execution.geometry_sha256 mismatch")
        if adm_ex.get("driver_source_sha256", expected_driver_src) != expected_driver_src:
            errors.append("admitted_execution.driver_source_sha256 mismatch")
        if admitted_source_sha256 is not None and a_src != admitted_source_sha256:
            errors.append("admitted_execution.source_sha256 mismatch")
        if admitted_kernel_sha256 is not None and a_krn != admitted_kernel_sha256:
            errors.append("admitted_execution.kernel_sha256 mismatch")
        if admitted_executable_sha256 is not None and a_exe is not None and a_exe != admitted_executable_sha256:
            errors.append("admitted_execution.executable_sha256 mismatch")
        admitted_source_sha256 = admitted_source_sha256 or a_src
        admitted_kernel_sha256 = admitted_kernel_sha256 or a_krn
        admitted_executable_sha256 = admitted_executable_sha256 or a_exe

    top_admission_map = data.get("admission_receipts") if isinstance(data.get("admission_receipts"), dict) else {}

    for arm in required_arms:
        if arm not in arms or not isinstance(arms[arm], dict):
            errors.append(f"Missing arm {arm}")
            continue
        a = arms[arm]
        if a.get("faults"):
            errors.append(f"{arm} faults: {a.get('faults')}")

        expected_seed = 1 if arm.endswith("seed1") else 2
        expected_policy = (
            "nearest_address_margin_v1" if "-A-" in arm else "occupied_cell_rank_hinge_v1"
        )
        expected_scope = f"m1-admission-redirect-{arm.lower()}-checkpoint0..16404"
        allowed_cfg_lin = get_allowed_k16_arm_cfg_lin(contract, arm)

        # 0A. Require & authenticate per-role CompletionAdmission manifest (CURRENT-8)
        adm_pin = (
            a.get("admission_receipt")
            or a.get("release_admission_receipt")
            or a.get("owner_admission_receipt")
            or top_admission_map.get(arm)
        )
        if adm_pin is None:
            errors.append(
                f"{arm} missing required per-role admission_receipt FilePin (uor-r4.redirect-owner-admission/1) "
                "and admitted runtime identity"
            )
            continue
        adm_raw, adm_obj = authenticate_file_pin(
            adm_pin,
            outcomes_dir,
            f"{arm} admission_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if adm_raw is None or adm_obj is None:
            continue

        if adm_obj.get("schema") != "uor-r4.redirect-owner-admission/1":
            errors.append(
                f"{arm} admission_receipt schema={adm_obj.get('schema')!r} != 'uor-r4.redirect-owner-admission/1'"
            )
            continue
        try:
            role_admission_bytes = native_completion_admission_form(adm_obj)
            role_admission_sha256 = sha256_bytes(role_admission_bytes)
            role_plan_bytes = native_completion_plan_form(adm_obj["plan"])
            role_plan_sha256 = sha256_bytes(role_plan_bytes)
        except Exception as exc:
            errors.append(f"{arm} admission_receipt native serialization failed: {exc}")
            continue

        if role_admission_sha256 in seen_role_admission_shas:
            errors.append(
                f"{arm} duplicate per-role admission_sha256={role_admission_sha256} (each K16 arm/seed requires a distinct CompletionAdmission manifest)"
            )
        seen_role_admission_shas.add(role_admission_sha256)
        arm_admission_shas[arm] = role_admission_sha256

        if "admission_sha256" in a and a.get("admission_sha256") != role_admission_sha256:
            errors.append(
                f"{arm} summary admission_sha256={a.get('admission_sha256')!r} != authenticated role_admission_sha256 ({role_admission_sha256})"
            )

        plan = adm_obj["plan"]
        if plan.get("schema") != "uor-r4.redirect-owner-release-plan/1":
            errors.append(
                f"{arm} admission.plan.schema={plan.get('schema')!r} != 'uor-r4.redirect-owner-release-plan/1'"
            )
        if plan.get("status") != "HELD_OWNER_ACTION_REQUIRED":
            errors.append(
                f"{arm} admission.plan.status={plan.get('status')!r} != 'HELD_OWNER_ACTION_REQUIRED'"
            )
        if plan.get("scope") != expected_scope:
            errors.append(
                f"{arm} admission.plan.scope={plan.get('scope')!r} != {expected_scope!r}"
            )
        if plan.get("pilot_parent") is not None:
            errors.append(f"{arm} admission.plan.pilot_parent must be null for K16")
        if not isinstance(plan.get("run_state_directory"), str) or not plan.get("run_state_directory", "").strip():
            errors.append(f"{arm} admission.plan.run_state_directory must be a non-empty path")

        plan_cfg = plan.get("config", {})
        plan_cfg_sha = sha256_bytes(native_driver_config_form(plan_cfg))
        if plan_cfg.get("counterfactual_path") != "rows_v1":
            errors.append(f"{arm} admission.plan.config.counterfactual_path != 'rows_v1'")
        if plan_cfg.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16}:
            errors.append(f"{arm} admission.plan.config.spec mismatch: {plan_cfg.get('spec')!r}")
        if plan_cfg.get("read_spec") != {"kind": "route"}:
            errors.append(f"{arm} admission.plan.config.read_spec != {{'kind': 'route'}}")
        if plan_cfg.get("seed") != expected_seed:
            errors.append(f"{arm} admission.plan.config.seed={plan_cfg.get('seed')!r} != {expected_seed}")
        if plan_cfg.get("objective_policy", "nearest_address_margin_v1") != expected_policy:
            errors.append(f"{arm} admission.plan.config.objective_policy != {expected_policy!r}")
        if plan_cfg.get("schedule") != "canonical_vector_then_read_v1":
            errors.append(f"{arm} admission.plan.config.schedule != 'canonical_vector_then_read_v1'")

        plan_ex = plan.get("execution", {})
        role_src = plan_ex.get("source_sha256")
        role_krn = plan_ex.get("kernel_sha256")
        role_exe = plan_ex.get("executable_sha256")
        role_cfg_sha = plan_ex.get("config_sha256")
        role_lin_sha = plan_ex.get("lineage_sha256")
        role_sheet_sha = plan_ex.get("sheet_sha256")

        if plan_ex.get("base_source") != ADMITTED_BASE_SOURCE:
            errors.append(
                f"{arm} admission.plan.execution.base_source={plan_ex.get('base_source')!r} != {ADMITTED_BASE_SOURCE!r}"
            )
        rev_commit = plan_ex.get("reviewed_source_commit")
        if not isinstance(rev_commit, str) or len(rev_commit) != 40 or not all(c in "0123456789abcdef" for c in rev_commit):
            errors.append(f"{arm} admission.plan.execution.reviewed_source_commit={rev_commit!r} invalid")
        if not is_hex64(role_src) or not is_hex64(role_krn) or not is_hex64(role_exe):
            errors.append(
                f"{arm} admission.plan.execution missing valid 64-hex source/kernel/executable sha256"
            )
        if plan_ex.get("geometry_sha256") != frozen_binds["geometry_sha256"]:
            errors.append(f"{arm} admission.plan.execution.geometry_sha256 mismatch")
        if plan_ex.get("driver_source_sha256") != expected_driver_src:
            errors.append(f"{arm} admission.plan.execution.driver_source_sha256 mismatch")
        if role_cfg_sha != plan_cfg_sha:
            errors.append(
                f"{arm} admission.plan.execution.config_sha256 ({role_cfg_sha}) != serialized plan.config sha256 ({plan_cfg_sha})"
            )
        if (role_cfg_sha, role_lin_sha) not in allowed_cfg_lin:
            errors.append(
                f"{arm} admission.plan.execution (config_sha256={role_cfg_sha}, lineage_sha256={role_lin_sha}) does not match any registered rows_v1 binding for {arm}"
            )
        if role_sheet_sha != plan.get("sheet", {}).get("sha256") or role_sheet_sha not in ALLOWED_REDIRECT_SHEET_SHA256S:
            errors.append(
                f"{arm} admission.plan.execution.sheet_sha256={role_sheet_sha!r} mismatch or not in allowed redirect sheet SHA-256 set"
            )

        if admitted_source_sha256 is not None and role_src != admitted_source_sha256:
            errors.append(f"{arm} admission.plan.execution.source_sha256 != shared admitted source_sha256")
        if admitted_kernel_sha256 is not None and role_krn != admitted_kernel_sha256:
            errors.append(f"{arm} admission.plan.execution.kernel_sha256 != shared admitted kernel_sha256")
        if admitted_executable_sha256 is not None and role_exe != admitted_executable_sha256:
            errors.append(f"{arm} admission.plan.execution.executable_sha256 != shared admitted executable_sha256")
        admitted_source_sha256 = admitted_source_sha256 or role_src
        admitted_kernel_sha256 = admitted_kernel_sha256 or role_krn
        admitted_executable_sha256 = admitted_executable_sha256 or role_exe

        red = plan.get("redirect")
        if not isinstance(red, dict):
            errors.append(f"{arm} admission.plan.redirect missing")
            continue
        if red.get("packet", {}).get("sha256") != ADMITTED_REDIRECT_PACKET_SHA256:
            errors.append(
                f"{arm} admission.plan.redirect.packet.sha256={red.get('packet', {}).get('sha256')!r} != {ADMITTED_REDIRECT_PACKET_SHA256!r}"
            )
        if plan.get("claude_review") != red.get("pm_review"):
            errors.append(f"{arm} admission.plan.claude_review != admission.plan.redirect.pm_review")
        if red.get("qualification", {}).get("sha256") == role_admission_sha256:
            errors.append(f"{arm} redirect.qualification.sha256 must be distinct from role_admission_sha256")

        # Authenticate per-role Qualification receipt (uor-r4.redirect-qualification/1)
        qual_obj: dict[str, Any] | None = None
        role_qual_pin = a.get("qualification_receipt") or red.get("qualification")
        if a.get("qualification_receipt") is not None and isinstance(a.get("qualification_receipt"), dict):
            if a["qualification_receipt"].get("sha256") != red.get("qualification", {}).get("sha256"):
                errors.append(
                    f"{arm} qualification_receipt.sha256 != admission.plan.redirect.qualification.sha256"
                )
        _, qual_obj = authenticate_file_pin(
            role_qual_pin,
            outcomes_dir,
            f"{arm} qualification_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if qual_obj is not None:
            if qual_obj.get("schema") != "uor-r4.redirect-qualification/1":
                errors.append(f"{arm} qualification_receipt schema != 'uor-r4.redirect-qualification/1'")
            if qual_obj.get("scope") != expected_scope:
                errors.append(f"{arm} qualification_receipt scope={qual_obj.get('scope')!r} != {expected_scope!r}")
            if qual_obj.get("execution") != plan_ex:
                errors.append(f"{arm} qualification_receipt execution != admission.plan.execution")
            if qual_obj.get("packet_sha256") != red.get("packet", {}).get("sha256"):
                errors.append(f"{arm} qualification_receipt packet_sha256 mismatch")
            if qual_obj.get("cuda_build_exit") != 0 or qual_obj.get("cuda_clippy_exit") != 0:
                errors.append(f"{arm} qualification_receipt cuda_build_exit/cuda_clippy_exit != 0")
            q_ctrls = qual_obj.get("controls", [])
            if (
                not isinstance(q_ctrls, list)
                or [c.get("id") for c in q_ctrls if isinstance(c, dict)]
                != ["C-R1", "C-R2", "C-R3", "C-R4", "C-R5", "C-R6"]
                or any(not isinstance(c, dict) or c.get("status") != "PASS" for c in q_ctrls)
            ):
                errors.append(f"{arm} qualification_receipt controls C-R1..C-R6 failed PASS check")
            else:
                for c_item in q_ctrls:
                    try:
                        native_file_pin_dict(c_item.get("receipt", {}))
                    except Exception as exc:
                        errors.append(f"{arm} qualification_receipt control {c_item.get('id')} invalid receipt FilePin: {exc}")
            for q_pin_key in ("cuda_build", "cuda_clippy", "timing", "initial_checkpoint", "initial_evaluation"):
                try:
                    native_file_pin_dict(qual_obj.get(q_pin_key, {}))
                except Exception as exc:
                    errors.append(f"{arm} qualification_receipt.{q_pin_key} invalid FilePin: {exc}")

        # Authenticate per-role PM review receipt (QualificationReview)
        role_pm_pin = a.get("pm_review_receipt") or red.get("pm_review")
        if a.get("pm_review_receipt") is not None and isinstance(a.get("pm_review_receipt"), dict):
            if a["pm_review_receipt"].get("sha256") != red.get("pm_review", {}).get("sha256"):
                errors.append(
                    f"{arm} pm_review_receipt.sha256 != admission.plan.redirect.pm_review.sha256"
                )
        _, pm_obj = authenticate_file_pin(
            role_pm_pin,
            outcomes_dir,
            f"{arm} pm_review_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if pm_obj is not None:
            if (
                pm_obj.get("reviewer") != "Kimi"
                or pm_obj.get("verdict") != "APPROVE_K16_REDIRECT_QUALIFIED_ADMISSION"
                or pm_obj.get("scope") != expected_scope
                or pm_obj.get("execution") != plan_ex
                or pm_obj.get("qualification_sha256") != red.get("qualification", {}).get("sha256")
                or pm_obj.get("packet_sha256") != red.get("packet", {}).get("sha256")
                or not isinstance(pm_obj.get("exact_source_review"), str)
                or not pm_obj["exact_source_review"].startswith("https://github.com/UOR-Foundation/uor-r4/")
                or "#issuecomment-" not in pm_obj["exact_source_review"]
                or not isinstance(pm_obj.get("receipt_url"), str)
                or not pm_obj["receipt_url"].startswith("https://github.com/UOR-Foundation/uor-r4/")
                or "#issuecomment-" not in pm_obj["receipt_url"]
            ):
                errors.append(f"{arm} pm_review_receipt failed QualificationReview contract checks")

        # Authenticate per-role CompletionOwner receipt (owner.json)
        role_owner_pin = a.get("owner_release_receipt") or a.get("owner_receipt") or adm_obj.get("owner_release")
        if (a.get("owner_release_receipt") or a.get("owner_receipt")) is not None:
            expl_ow = a.get("owner_release_receipt") or a.get("owner_receipt")
            if isinstance(expl_ow, dict) and expl_ow.get("sha256") != adm_obj.get("owner_release", {}).get("sha256"):
                errors.append(f"{arm} owner_release_receipt.sha256 != admission.owner_release.sha256")
        _, owner_obj = authenticate_file_pin(
            role_owner_pin,
            outcomes_dir,
            f"{arm} owner_release_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if owner_obj is not None:
            expected_owner_url = f"owner-terminal:{adm_obj['owner_release']['path']}"
            if (
                owner_obj.get("owner") != "Casey"
                or owner_obj.get("decision") != "RELEASE_D4_FOR_THIS_K16_REDIRECT_ARM_SEED_ONLY"
                or owner_obj.get("scope") != expected_scope
                or owner_obj.get("plan_sha256") != role_plan_sha256
                or owner_obj.get("receipt_url") != expected_owner_url
            ):
                errors.append(
                    f"{arm} owner_release_receipt failed CompletionOwner contract checks (plan_sha256={owner_obj.get('plan_sha256')!r} vs {role_plan_sha256!r})"
                )

        # 0B. Require & authenticate per-role CompletionReport receipt (uor-r4.completion-report/1, CURRENT-8)
        comp_pin = a.get("completion_report_receipt") or a.get("completion_report")
        if comp_pin is None:
            errors.append(
                f"{arm} missing required completion_report_receipt FilePin (uor-r4.completion-report/1)"
            )
            continue
        _, comp_rep = authenticate_file_pin(
            comp_pin,
            outcomes_dir,
            f"{arm} completion_report_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if comp_rep is None:
            continue

        comp_keys = set(comp_rep.keys())
        if comp_keys != COMPLETION_REPORT_REQUIRED_KEYS:
            missing_k = sorted(COMPLETION_REPORT_REQUIRED_KEYS - comp_keys)
            extra_k = sorted(comp_keys - COMPLETION_REPORT_REQUIRED_KEYS)
            errors.append(
                f"{arm} completion_report_receipt key set mismatch: missing={missing_k}, extra/invented={extra_k}"
            )
        el_ns = comp_rep.get("attempt_elapsed_ns")
        valid_elapsed_ns = (isinstance(el_ns, str) and len(el_ns) > 0 and el_ns.isdigit()) or (
            is_strict_int(el_ns) and el_ns >= 0
        )
        if (
            comp_rep.get("schema") != "uor-r4.completion-report/1"
            or comp_rep.get("status") != "COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION"
            or not is_strict_int(comp_rep.get("seed"))
            or comp_rep.get("seed") != expected_seed
            or comp_rep.get("scope") != expected_scope
            or comp_rep.get("admission_sha256") != role_admission_sha256
            or not is_strict_int(comp_rep.get("completed_batches"))
            or comp_rep.get("completed_batches") != 16404
            or not is_strict_int(comp_rep.get("eligible_train_target_exposures"))
            or comp_rep.get("eligible_train_target_exposures") != 1049856
            or comp_rep.get("pilot_parent") is not None
            or not valid_elapsed_ns
            or comp_rep.get("fresh_transfer") != "UNAVAILABLE_NOT_RUN"
            or comp_rep.get("next_seed") != "SEPARATE_OWNER_RELEASE_REQUIRED"
            or comp_rep.get("m2") != "HELD"
        ):
            errors.append(
                f"{arm} completion_report_receipt failed uor-r4.completion-report/1 field or role_admission_sha256 checks"
            )
        comp_evals = comp_rep.get("evaluations")
        if (
            not isinstance(comp_evals, list)
            or len(comp_evals) != 4
            or any(
                not isinstance(ep, dict)
                or not isinstance(ep.get("path"), str)
                or not ep.get("path", "").strip()
                or not is_hex64(ep.get("sha256"))
                for ep in comp_evals
            )
            or len({ep["sha256"] for ep in comp_evals if isinstance(ep, dict) and "sha256" in ep}) != 4
        ):
            errors.append(
                f"{arm} completion_report_receipt.evaluations must be a 4-element list of distinct valid FilePins (04101, 08202, 12303, 16404)"
            )

        # 1. Authenticate cursor-16404 evaluation & checkpoint receipts across all 4 hash domains (V3-S3, CURRENT-6, CURRENT-8)
        eval_pin = a.get("q4_evaluation_receipt") or a.get("q4_16404_evaluation_receipt")
        _, eval_obj = authenticate_file_pin(
            eval_pin,
            outcomes_dir,
            f"{arm} q4_evaluation_receipt",
            errors,
            extra_roots=outcomes_extra_roots,
        )
        if eval_obj is None:
            continue

        if (
            isinstance(comp_evals, list)
            and len(comp_evals) == 4
            and isinstance(comp_evals[3], dict)
            and isinstance(eval_pin, dict)
            and comp_evals[3].get("sha256") != eval_pin.get("sha256")
        ):
            errors.append(
                f"{arm} completion_report_receipt.evaluations[3].sha256 ({comp_evals[3].get('sha256')}) != q4_evaluation_receipt.sha256 ({eval_pin.get('sha256')})"
            )

        ckpt_pin = (
            a.get("q4_checkpoint_receipt")
            or a.get("q4_16404_checkpoint_receipt")
            or (eval_obj.get("checkpoint") if isinstance(eval_obj.get("checkpoint"), dict) else None)
        )
        ckpt_raw: bytes | None = None
        ckpt_obj: dict[str, Any] | None = None
        off_host_ckpt_verified = False
        if (
            bool(member_manifest_shas)
            and isinstance(ckpt_pin, dict)
            and isinstance(ckpt_pin.get("path"), str)
            and is_hex64(ckpt_pin.get("sha256"))
        ):
            resolved_ckpt = resolve_pin_path(ckpt_pin["path"], outcomes_dir, outcomes_extra_roots)
            if not resolved_ckpt.is_file():
                rel_ckpt = ckpt_pin["path"].strip().lstrip("/")
                eval_ckpt_pin = eval_obj.get("checkpoint") if isinstance(eval_obj.get("checkpoint"), dict) else {}
                eval_rel_ckpt = str(eval_ckpt_pin.get("path", "")).strip().lstrip("/")
                map_ok = True
                if arm in role_member_map_pins and "checkpoint_16404" in role_member_map_pins[arm]:
                    map_ok = role_member_map_pins[arm]["checkpoint_16404"] == (rel_ckpt, ckpt_pin["sha256"])
                if (
                    map_ok
                    and ckpt_pin["sha256"] in member_manifest_shas.get(rel_ckpt, set())
                    and eval_obj.get("schema") == "uor-r4.completion-evaluation/1"
                    and eval_ckpt_pin.get("sha256") == ckpt_pin["sha256"]
                    and eval_rel_ckpt == rel_ckpt
                ):
                    off_host_ckpt_verified = True

        if not off_host_ckpt_verified:
            ckpt_raw, ckpt_obj = authenticate_file_pin(
                ckpt_pin,
                outcomes_dir,
                f"{arm} q4_checkpoint_receipt",
                errors,
                extra_roots=outcomes_extra_roots,
            )
            if ckpt_raw is None or ckpt_obj is None:
                continue

            try:
                domains = compute_snapshot_hash_domains(ckpt_obj, ckpt_raw)
            except Exception as exc:
                errors.append(f"{arm} q4_checkpoint_receipt hash domain computation failed: {exc}")
                continue

            if (
                not is_hex64(domains["declared_model_sha256"])
                or domains["declared_model_sha256"] != domains["domain_A_model_storage_sha256"]
            ):
                errors.append(
                    f"{arm} checkpoint model_sha256 ({domains['declared_model_sha256']!r}) != Domain A ({domains['domain_A_model_storage_sha256']})"
                )

            # Optional standalone ModelSnapshot receipt FilePin cross-check
            snap_pin = a.get("q4_model_snapshot_receipt")
            if snap_pin is not None:
                _, snap_obj = authenticate_file_pin(
                    snap_pin,
                    outcomes_dir,
                    f"{arm} q4_model_snapshot_receipt",
                    errors,
                    extra_roots=outcomes_extra_roots,
                )
                if snap_obj is not None:
                    snap_dom_a = sha256_bytes(canonical_sorted_json_bytes(snap_obj))
                    snap_dom_b = sha256_bytes(native_snapshot_form(snap_obj))
                    if snap_dom_a != domains["domain_A_model_storage_sha256"]:
                        errors.append(f"{arm} standalone ModelSnapshot Domain A != checkpoint Domain A")
                    if snap_dom_b != domains["domain_B_typed_model_snapshot_sha256"]:
                        errors.append(f"{arm} standalone ModelSnapshot Domain B != checkpoint Domain B")

            if ckpt_obj.get("schema") == "uor-r4.completion-checkpoint/1":
                inner_ckpt = ckpt_obj.get("checkpoint", {})
                if ckpt_obj.get("pilot_parent") is not None:
                    errors.append(f"{arm} completion-checkpoint pilot_parent must be null for K16")
                if ckpt_obj.get("admission_sha256") != role_admission_sha256:
                    errors.append(
                        f"{arm} completion-checkpoint admission_sha256 ({ckpt_obj.get('admission_sha256')!r}) != role_admission_sha256 ({role_admission_sha256})"
                    )
                vec_c = ckpt_obj.get("vector", {})
                rte_c = ckpt_obj.get("route", {})
                if vec_c.get("attempted") != 16384 or rte_c.get("attempted") != 20:
                    errors.append(
                        f"{arm} completion-checkpoint vector/route attempted=({vec_c.get('attempted')}, {rte_c.get('attempted')}) != (16384, 20)"
                    )
                if (
                    not is_strict_int(vec_c.get("accepted_updates"))
                    or not is_strict_int(rte_c.get("accepted_updates"))
                    or vec_c.get("accepted_updates") + rte_c.get("accepted_updates")
                    != inner_ckpt.get("cursor", {}).get("accepted_updates")
                ):
                    errors.append(f"{arm} completion-checkpoint split-family accepted_updates sum mismatch")
                if comp_rep.get("sensitivity") != {"vector_block": vec_c, "route": rte_c}:
                    errors.append(
                        f"{arm} completion_report_receipt.sensitivity != completion-checkpoint vector/route counters"
                    )
                if comp_rep.get("baseline_snapshot_metadata") != ckpt_obj.get("baseline_snapshot_metadata"):
                    errors.append(
                        f"{arm} completion_report_receipt.baseline_snapshot_metadata != completion-checkpoint baseline_snapshot_metadata"
                    )
                if comp_rep.get("baseline_evaluation") != ckpt_obj.get("baseline_evaluation"):
                    errors.append(
                        f"{arm} completion_report_receipt.baseline_evaluation != completion-checkpoint baseline_evaluation"
                    )
            elif ckpt_obj.get("schema") == "uor-r4.discrete-driver-checkpoint/1":
                inner_ckpt = ckpt_obj
            else:
                errors.append(f"{arm} unsupported checkpoint schema: {ckpt_obj.get('schema')!r}")
                continue

            if inner_ckpt.get("schema") != "uor-r4.discrete-driver-checkpoint/1":
                errors.append(f"{arm} inner checkpoint schema != 'uor-r4.discrete-driver-checkpoint/1'")
                continue

            lineage = inner_ckpt.get("lineage", {})
            cfg = lineage.get("config", {})
            if lineage.get("driver_source_sha256") != expected_driver_src:
                errors.append(
                    f"{arm} lineage.driver_source_sha256={lineage.get('driver_source_sha256')!r} != {expected_driver_src!r}"
                )
            act_cfg_sha = sha256_bytes(native_driver_config_form(cfg))
            act_lin_sha = sha256_bytes(compact_json_bytes(native_lineage_dict(lineage)))
            if act_cfg_sha != role_cfg_sha or act_lin_sha != role_lin_sha:
                errors.append(
                    f"{arm} checkpoint (config_sha256={act_cfg_sha}, lineage_sha256={act_lin_sha}) != admission.plan.execution ({role_cfg_sha}, {role_lin_sha})"
                )
            if (act_cfg_sha, act_lin_sha) not in allowed_cfg_lin:
                errors.append(
                    f"{arm} (config_sha256={act_cfg_sha}, lineage_sha256={act_lin_sha}) does not match any registered rows_v1 binding for {arm}"
                )
            if cfg.get("counterfactual_path") != "rows_v1":
                errors.append(f"{arm} checkpoint config.counterfactual_path != 'rows_v1'")
            if cfg.get("spec") != {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16}:
                errors.append(f"{arm} checkpoint config.spec mismatch: {cfg.get('spec')!r}")
            if cfg.get("read_spec") != {"kind": "route"}:
                errors.append(f"{arm} checkpoint config.read_spec != {{'kind': 'route'}}")
            if cfg.get("seed") != expected_seed:
                errors.append(f"{arm} checkpoint config.seed={cfg.get('seed')!r} != {expected_seed}")
            if cfg.get("objective_policy", "nearest_address_margin_v1") != expected_policy:
                errors.append(f"{arm} checkpoint config.objective_policy != {expected_policy!r}")
            if cfg.get("schedule") != "canonical_vector_then_read_v1":
                errors.append(f"{arm} checkpoint config.schedule != 'canonical_vector_then_read_v1'")

            cursor = inner_ckpt.get("cursor", {})
            model = inner_ckpt.get("model", {})
            epoch = model.get("epoch")
            if (
                not is_strict_int(cursor.get("completed_batches"))
                or cursor.get("completed_batches") != 16404
                or not is_strict_int(cursor.get("accepted_updates"))
                or not is_strict_int(epoch)
                or cursor.get("accepted_updates") != epoch
                or epoch < 0
                or epoch > 16404
            ):
                errors.append(f"{arm} invalid Q4 cursor/model.epoch: cursor={cursor!r}, epoch={epoch!r}")
                continue

            if model.get("schema") not in ("uor-r4.discrete-spiralcore-fbs/2", "uor-r4.discrete-snapshot/1"):
                errors.append(f"{arm} model.schema={model.get('schema')!r} unrecognized")
            if not is_hex64(model.get("source_sha256")) or model.get("source_sha256") != role_src:
                errors.append(
                    f"{arm} model.source_sha256={model.get('source_sha256')!r} != admitted source_sha256={role_src!r}"
                )
            if not is_hex64(model.get("kernel_sha256")) or model.get("kernel_sha256") != role_krn:
                errors.append(
                    f"{arm} model.kernel_sha256={model.get('kernel_sha256')!r} != admitted kernel_sha256={role_krn!r}"
                )
            if model.get("geometry_sha256") != frozen_binds["geometry_sha256"]:
                errors.append(f"{arm} model.geometry_sha256 mismatch")
            if model.get("operator_kappa") != frozen_binds["operator_kappa"]:
                errors.append(f"{arm} model.operator_kappa mismatch")
            if model.get("composition_kappa") != frozen_binds["composition_kappa"]:
                errors.append(f"{arm} model.composition_kappa mismatch")
        else:
            # Off-host raw checkpoint-16404.json verified via authenticated member manifest + evaluation-16404.json + report.json
            snap_meta = eval_obj.get("snapshot_metadata", {})
            drv_ev = eval_obj.get("evaluation", {})
            dom_a = snap_meta.get("model_storage_sha256") or drv_ev.get("model_storage_sha256")
            dom_b = snap_meta.get("model_snapshot_sha256") or drv_ev.get("model_snapshot_sha256")
            dom_c = eval_obj.get("driver_checkpoint_sha256")
            dom_d = ckpt_pin["sha256"]
            if not (is_hex64(dom_a) and is_hex64(dom_b) and is_hex64(dom_c) and is_hex64(dom_d)):
                errors.append(f"{arm} off-host checkpoint domain A..D digests incomplete in evaluation-16404.json")
                continue
            domains = {
                "domain_A_model_storage_sha256": dom_a,
                "domain_B_typed_model_snapshot_sha256": dom_b,
                "domain_C_driver_checkpoint_sha256": dom_c,
                "domain_D_checkpoint_file_sha256": dom_d,
                "declared_model_sha256": dom_a,
            }
            act_cfg_sha = role_cfg_sha
            act_lin_sha = role_lin_sha
            cursor = eval_obj.get("cursor", {})
            epoch = cursor.get("accepted_updates")
            if (
                not is_strict_int(cursor.get("completed_batches"))
                or cursor.get("completed_batches") != 16404
                or not is_strict_int(epoch)
                or epoch < 0
                or epoch > 16404
            ):
                errors.append(f"{arm} invalid Q4 cursor in evaluation-16404.json: {cursor!r}")
                continue
            ckpt_obj = {
                "schema": "uor-r4.completion-checkpoint/1",
                "baseline_snapshot_metadata": comp_rep.get("baseline_snapshot_metadata"),
                "baseline_evaluation": comp_rep.get("baseline_evaluation"),
                "vector": comp_rep.get("sensitivity", {}).get("vector_block", {}),
                "route": comp_rep.get("sensitivity", {}).get("route", {}),
            }

        if eval_obj.get("schema") == "uor-r4.completion-evaluation/1":
            if eval_obj.get("admission_sha256") != role_admission_sha256:
                errors.append(
                    f"{arm} completion-evaluation admission_sha256 ({eval_obj.get('admission_sha256')!r}) != role_admission_sha256 ({role_admission_sha256})"
                )
            if eval_obj.get("completed_batches") != 16404:
                errors.append(f"{arm} completion-evaluation completed_batches != 16404")
            if eval_obj.get("cursor") != cursor:
                errors.append(f"{arm} completion-evaluation cursor != checkpoint cursor")
            if eval_obj.get("checkpoint", {}).get("sha256") != domains["domain_D_checkpoint_file_sha256"]:
                errors.append(
                    f"{arm} completion-evaluation checkpoint.sha256 != Domain D ({domains['domain_D_checkpoint_file_sha256']})"
                )
            if eval_obj.get("driver_checkpoint_sha256") != domains["domain_C_driver_checkpoint_sha256"]:
                errors.append(
                    f"{arm} completion-evaluation driver_checkpoint_sha256 != Domain C ({domains['domain_C_driver_checkpoint_sha256']})"
                )
            sens = eval_obj.get("sensitivity", {})
            vec_s = sens.get("vector_block", {})
            rte_s = sens.get("route", {})
            if vec_s.get("attempted") != 16384 or rte_s.get("attempted") != 20:
                errors.append(f"{arm} completion-evaluation sensitivity attempted != (16384, 20)")
            if (
                not is_strict_int(vec_s.get("accepted_updates"))
                or not is_strict_int(rte_s.get("accepted_updates"))
                or vec_s.get("accepted_updates") + rte_s.get("accepted_updates") != epoch
            ):
                errors.append(f"{arm} completion-evaluation split-family accepted_updates sum != {epoch}")
            if comp_rep.get("sensitivity") != sens:
                errors.append(
                    f"{arm} completion_report_receipt.sensitivity != completion-evaluation sensitivity"
                )
            if isinstance(eval_obj.get("snapshot_metadata"), dict):
                sm = eval_obj["snapshot_metadata"]
                if (
                    sm.get("model_storage_sha256") != domains["domain_A_model_storage_sha256"]
                    or sm.get("model_snapshot_sha256") != domains["domain_B_typed_model_snapshot_sha256"]
                    or sm.get("epoch") != epoch
                    or sm.get("source_sha256") != role_src
                    or sm.get("kernel_sha256") != role_krn
                    or sm.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                ):
                    errors.append(f"{arm} completion-evaluation snapshot_metadata mismatch")

        q4_agg, q4_flags = extract_driver_eval_and_flags(
            eval_obj, f"{arm} q4_evaluation_receipt", errors
        )
        if q4_agg is None or q4_flags is None:
            continue

        if q4_flags.get("evaluation_lineage_sha256") != act_lin_sha:
            errors.append(
                f"{arm} q4_evaluation_receipt evaluation_lineage_sha256 != {act_lin_sha}"
            )
        verify_eval_zero_update_flags(
            q4_flags,
            expected_epoch=epoch,
            expected_domain_a=domains["domain_A_model_storage_sha256"],
            expected_domain_b=domains["domain_B_typed_model_snapshot_sha256"],
            label=f"{arm} q4_evaluation_receipt",
            errors=errors,
        )
        if "model_bindings" in q4_flags:
            q4_mb = q4_flags.get("model_bindings")
            if (
                not isinstance(q4_mb, dict)
                or q4_mb.get("source_sha256") != role_src
                or q4_mb.get("kernel_sha256") != role_krn
                or q4_mb.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                or q4_mb.get("seed") != expected_seed
                or q4_mb.get("objective_policy", "nearest_address_margin_v1") != expected_policy
            ):
                errors.append(f"{arm} q4_evaluation_receipt model_bindings mismatch")
        if q4_agg.get("scored_rows") != 14287 or q4_agg.get("token_eligible_rows") != 14287:
            errors.append(
                f"{arm} q4_evaluation_receipt scored_rows/token_eligible_rows != 14287"
            )

        # If summary also repeated q4_16404 metrics, verify they match the authenticated receipt
        if isinstance(a.get("q4_16404"), dict):
            for mk, mv in a["q4_16404"].items():
                if mk in q4_agg and q4_agg[mk] != mv:
                    errors.append(
                        f"{arm} summary q4_16404[{mk!r}]={mv!r} != receipt aggregate {q4_agg[mk]!r}"
                    )

        # 3. Authenticate checkpoint-0 state & evaluation receipt FilePin (V3-S3, V3-S8, CURRENT-6 & CURRENT-8)
        c0_exp_dom_a: str | None = None
        c0_exp_dom_b: str | None = None
        c0_exp_dom_c: str | None = None
        c0_exp_dom_d: str | None = None

        c0_ckpt_pin = a.get("checkpoint0_checkpoint_receipt") or a.get("checkpoint0_model_snapshot_receipt")
        if c0_ckpt_pin is not None:
            c0_ckpt_raw, c0_ckpt_obj = authenticate_file_pin(
                c0_ckpt_pin,
                outcomes_dir,
                f"{arm} checkpoint0_checkpoint_receipt",
                errors,
                extra_roots=outcomes_extra_roots,
            )
            if c0_ckpt_raw is not None and c0_ckpt_obj is not None:
                if "checkpoint" in c0_ckpt_obj or "model" in c0_ckpt_obj:
                    c0_doms = compute_snapshot_hash_domains(c0_ckpt_obj, c0_ckpt_raw)
                    c0_inner = c0_ckpt_obj.get("checkpoint", c0_ckpt_obj)
                    c0_model = c0_inner.get("model", {})
                    c0_cur = c0_inner.get("cursor", {})
                    if c0_ckpt_obj.get("schema") == "uor-r4.completion-checkpoint/1":
                        if c0_ckpt_obj.get("admission_sha256") != role_admission_sha256:
                            errors.append(
                                f"{arm} checkpoint0_checkpoint_receipt admission_sha256 != role_admission_sha256 ({role_admission_sha256})"
                            )
                    if (
                        c0_doms["declared_model_sha256"] != c0_doms["domain_A_model_storage_sha256"]
                        or c0_cur.get("completed_batches") != 0
                        or c0_cur.get("accepted_updates") != 0
                        or c0_model.get("epoch") != 0
                        or c0_model.get("seed") != expected_seed
                        or c0_model.get("objective_policy", "nearest_address_margin_v1") != expected_policy
                        or c0_model.get("source_sha256") != role_src
                        or c0_model.get("kernel_sha256") != role_krn
                        or c0_model.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                    ):
                        errors.append(
                            f"{arm} checkpoint0_checkpoint_receipt failed epoch==0/seed/policy/runtime identity checks"
                        )
                    c0_exp_dom_a = c0_doms["domain_A_model_storage_sha256"]
                    c0_exp_dom_b = c0_doms["domain_B_typed_model_snapshot_sha256"]
                    c0_exp_dom_c = c0_doms["domain_C_driver_checkpoint_sha256"]
                    c0_exp_dom_d = c0_doms["domain_D_checkpoint_file_sha256"]
                else:
                    # Standalone checkpoint0 Snapshot JSON
                    if (
                        c0_ckpt_obj.get("epoch") != 0
                        or c0_ckpt_obj.get("seed") != expected_seed
                        or c0_ckpt_obj.get("objective_policy", "nearest_address_margin_v1") != expected_policy
                        or c0_ckpt_obj.get("source_sha256") != role_src
                        or c0_ckpt_obj.get("kernel_sha256") != role_krn
                        or c0_ckpt_obj.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                    ):
                        errors.append(
                            f"{arm} checkpoint0_model_snapshot_receipt failed epoch==0/seed/policy/runtime identity checks"
                        )
                    c0_exp_dom_a = sha256_bytes(canonical_sorted_json_bytes(c0_ckpt_obj))
                    c0_exp_dom_b = sha256_bytes(native_snapshot_form(c0_ckpt_obj))

        base_meta = ckpt_obj.get("baseline_snapshot_metadata") if isinstance(ckpt_obj, dict) else None
        if isinstance(base_meta, dict) and base_meta:
            bm_a = base_meta.get("model_storage_sha256")
            bm_b = base_meta.get("model_snapshot_sha256")
            if (
                not is_hex64(bm_a)
                or not is_hex64(bm_b)
                or not is_strict_int(base_meta.get("epoch"))
                or base_meta.get("epoch") != 0
                or base_meta.get("source_sha256") != role_src
                or base_meta.get("kernel_sha256") != role_krn
                or base_meta.get("geometry_sha256") != frozen_binds["geometry_sha256"]
            ):
                errors.append(
                    f"{arm} completion-checkpoint baseline_snapshot_metadata failed Domain A/B or runtime identity checks"
                )
            else:
                if c0_exp_dom_a is not None and c0_exp_dom_a != bm_a:
                    errors.append(
                        f"{arm} baseline_snapshot_metadata Domain A ({bm_a}) != checkpoint0_checkpoint_receipt Domain A ({c0_exp_dom_a})"
                    )
                if c0_exp_dom_b is not None and c0_exp_dom_b != bm_b:
                    errors.append(
                        f"{arm} baseline_snapshot_metadata Domain B ({bm_b}) != checkpoint0_checkpoint_receipt Domain B ({c0_exp_dom_b})"
                    )
                c0_exp_dom_a = c0_exp_dom_a or bm_a
                c0_exp_dom_b = c0_exp_dom_b or bm_b

        if not is_hex64(c0_exp_dom_a) or not is_hex64(c0_exp_dom_b):
            errors.append(
                f"{arm} missing authenticated checkpoint0 model/storage domain binding (provide checkpoint0_checkpoint_receipt or completion-checkpoint baseline_snapshot_metadata)"
            )

        c0_pin = a.get("checkpoint0_evaluation_receipt") or a.get("checkpoint0_receipt")
        c0_obj: dict[str, Any] | None = None
        if c0_pin is not None:
            _, c0_obj = authenticate_file_pin(
                c0_pin,
                outcomes_dir,
                f"{arm} checkpoint0_evaluation_receipt",
                errors,
                extra_roots=outcomes_extra_roots,
            )
        elif (
            isinstance(ckpt_obj.get("baseline_evaluation"), dict)
            and ckpt_obj["baseline_evaluation"].get("schema") == "uor-r4.discrete-corpus-driver/1"
        ):
            c0_obj = ckpt_obj["baseline_evaluation"]
        else:
            errors.append(f"{arm} checkpoint0_evaluation_receipt: missing or non-object FilePin")
            continue

        if c0_obj is None:
            continue
        c0_agg, c0_flags = extract_driver_eval_and_flags(
            c0_obj, f"{arm} checkpoint0_evaluation_receipt", errors
        )
        if c0_agg is None or c0_flags is None:
            continue
        if c0_flags.get("evaluation_lineage_sha256") != act_lin_sha:
            errors.append(
                f"{arm} checkpoint0_evaluation_receipt evaluation_lineage_sha256 != {act_lin_sha}"
            )
        if c0_exp_dom_c is not None and "driver_checkpoint_sha256" in c0_flags:
            if c0_flags.get("driver_checkpoint_sha256") != c0_exp_dom_c:
                errors.append(
                    f"{arm} checkpoint0_evaluation_receipt driver_checkpoint_sha256 != checkpoint0 Domain C ({c0_exp_dom_c})"
                )
        if c0_exp_dom_d is not None and "checkpoint_file_sha256" in c0_flags:
            if c0_flags.get("checkpoint_file_sha256") != c0_exp_dom_d:
                errors.append(
                    f"{arm} checkpoint0_evaluation_receipt checkpoint_file_sha256 != checkpoint0 Domain D ({c0_exp_dom_d})"
                )
        if "model_bindings" in c0_flags:
            c0_mb = c0_flags.get("model_bindings")
            if (
                not isinstance(c0_mb, dict)
                or c0_mb.get("source_sha256") != role_src
                or c0_mb.get("kernel_sha256") != role_krn
                or c0_mb.get("geometry_sha256") != frozen_binds["geometry_sha256"]
                or c0_mb.get("seed") != expected_seed
                or c0_mb.get("objective_policy", "nearest_address_margin_v1") != expected_policy
            ):
                errors.append(f"{arm} checkpoint0_evaluation_receipt model_bindings mismatch")
        verify_eval_zero_update_flags(
            c0_flags,
            expected_epoch=0,
            expected_domain_a=c0_exp_dom_a,
            expected_domain_b=c0_exp_dom_b,
            label=f"{arm} checkpoint0_evaluation_receipt",
            errors=errors,
        )
        if (
            c0_agg.get("scored_rows") != 14287
            or c0_agg.get("token_eligible_rows") != 14287
            or not is_strict_int(c0_agg.get("target_admitted_rows"))
            or c0_agg.get("target_admitted_rows") < 4894
        ):
            errors.append(
                f"{arm} checkpoint0_evaluation_receipt failed scored_rows==14287 or G-Pre target_admitted_rows >= 4894"
            )
        base_eval = ckpt_obj.get("baseline_evaluation") if isinstance(ckpt_obj, dict) else None
        if isinstance(base_eval, dict) and base_eval:
            if base_eval.get("schema") == "uor-r4.discrete-corpus-driver/1":
                if base_eval != c0_flags:
                    errors.append(
                        f"{arm} completion-checkpoint baseline_evaluation != authenticated checkpoint0 driver evaluation"
                    )
                if (
                    qual_obj is not None
                    and isinstance(qual_obj.get("initial_evaluation"), dict)
                    and qual_obj["initial_evaluation"].get("schema") == "uor-r4.discrete-corpus-driver/1"
                ):
                    if base_eval != qual_obj["initial_evaluation"]:
                        errors.append(
                            f"{arm} completion-checkpoint baseline_evaluation != qualification_receipt.initial_evaluation"
                        )
            else:
                for mk, mv in base_eval.items():
                    if mk in c0_agg and c0_agg[mk] != mv:
                        errors.append(
                            f"{arm} completion-checkpoint baseline_evaluation[{mk!r}]={mv!r} != checkpoint0_evaluation_receipt {c0_agg[mk]!r}"
                        )
        if isinstance(a.get("checkpoint0"), dict):
            for mk, mv in a["checkpoint0"].items():
                if mk in c0_agg and c0_agg[mk] != mv:
                    errors.append(
                        f"{arm} summary checkpoint0[{mk!r}]={mv!r} != receipt aggregate {c0_agg[mk]!r}"
                    )

        arm_q4_agg[arm] = q4_agg
        arm_c0_agg[arm] = c0_agg
        arm_domains[arm] = domains
        arm_epochs[arm] = epoch
        arm_lineages[arm] = act_lin_sha

    if (
        not is_hex64(admitted_source_sha256)
        or not is_hex64(admitted_kernel_sha256)
        or not is_hex64(admitted_executable_sha256)
    ):
        errors.append(
            "Missing or invalid admitted runtime identity across per-role admission_receipts"
        )

    if errors:
        return {
            "status": "FAIL",
            "decision": "INSTRUMENT_OR_RECEIPT_FAULT",
            "errors": errors,
        }

    # Extract final cursor 16,404 metrics and checkpoint-0 strict-win baseline u from authenticated receipts
    def m(arm: str, key: str) -> float:
        return float(arm_q4_agg[arm][key])

    def u0(arm: str) -> float:
        c0 = arm_c0_agg[arm]
        return float(c0["token_strict_win_eligible_rows"]) / float(c0["token_eligible_rows"])

    def strict_rate_q4(arm: str) -> float:
        q4 = arm_q4_agg[arm]
        return float(q4["token_strict_win_eligible_rows"]) / float(q4["token_eligible_rows"])

    def guard_passes(arm: str) -> tuple[bool, bool]:
        u = u0(arm)
        if u > 0.95:
            return True, True  # INSTRUMENT_INVALID -> bars decide alone
        req = max(0.01, min(5.0 * u, u + 0.05))
        return strict_rate_q4(arm) >= req - 1e-12, False

    # 1. G-Occ at cursor 16,404 for K16-R vs K16-A on each seed
    g_occ_triggered = False
    for s in (1, 2):
        occ_a = m(f"k16-A-seed{s}", "occupied_cells")
        occ_r = m(f"k16-R-seed{s}", "occupied_cells")
        n_a = m(f"k16-A-seed{s}", "admitted_token_count_sum") / 14287.0
        n_r = m(f"k16-R-seed{s}", "admitted_token_count_sum") / 14287.0
        min_occ = max(32, math.ceil(0.50 * occ_a))
        if occ_r < min_occ or n_r > 2.0 * n_a + 1e-12:
            g_occ_triggered = True

    # 2. G-Adm (>= 4894 on both seeds)
    r_g_adm = all(m(f"k16-R-seed{s}", "target_admitted_rows") >= 4894 for s in (1, 2))
    a_g_adm = all(m(f"k16-A-seed{s}", "target_admitted_rows") >= 4894 for s in (1, 2))

    # 3. Ranking Bars: Bar (i) top1 >= 1941, Bar (ii) top8 >= 4775 on both seeds
    r_bar_i = all(m(f"k16-R-seed{s}", "top1_correct") >= 1941 for s in (1, 2))
    r_bar_ii = all(m(f"k16-R-seed{s}", "top8_correct") >= 4775 for s in (1, 2))
    r_bars = r_bar_i and r_bar_ii

    a_bar_i = all(m(f"k16-A-seed{s}", "top1_correct") >= 1941 for s in (1, 2))
    a_bar_ii = all(m(f"k16-A-seed{s}", "top8_correct") >= 4775 for s in (1, 2))
    a_bars = a_bar_i and a_bar_ii

    # 4. G-Guard (rebased at own K16 checkpoint 0)
    r_guard = all(guard_passes(f"k16-R-seed{s}")[0] for s in (1, 2))
    a_guard = all(guard_passes(f"k16-A-seed{s}")[0] for s in (1, 2))

    # 5. Dynamic noise floors delta_noise_M (V3-S6: ONLY the 3 registered metrics from sheet lines 120-121)
    floors = {
        "target_admitted_rows": 87.0,
        "top1_correct": 20.0,
        "top8_correct": 63.0,
    }
    delta_noise: dict[str, float] = {}
    mean_diff_r_minus_a: dict[str, float] = {}
    for metric, fl in floors.items():
        a1, a2 = m("k16-A-seed1", metric), m("k16-A-seed2", metric)
        r1, r2 = m("k16-R-seed1", metric), m("k16-R-seed2", metric)
        delta_noise[metric] = max(abs(a1 - a2), abs(r1 - r2), fl)
        mean_diff_r_minus_a[metric] = 0.5 * (r1 + r2) - 0.5 * (a1 + a2)

    a1_sw = m("k16-A-seed1", "token_strict_win_eligible_rows")
    a2_sw = m("k16-A-seed2", "token_strict_win_eligible_rows")
    r1_sw = m("k16-R-seed1", "token_strict_win_eligible_rows")
    r2_sw = m("k16-R-seed2", "token_strict_win_eligible_rows")
    delta_strict_win = 0.5 * (r1_sw + r2_sw) - 0.5 * (a1_sw + a2_sw)
    strict_win_spread = max(abs(a1_sw - a2_sw), abs(r1_sw - r2_sw))
    strict_win_no_regression = delta_strict_win >= 0.0
    mean_diff_r_minus_a["token_strict_win_eligible_rows"] = delta_strict_win

    r_beats_a_adm = mean_diff_r_minus_a["target_admitted_rows"] > delta_noise["target_admitted_rows"]
    r_beats_a_rank = (
        mean_diff_r_minus_a["top1_correct"] > delta_noise["top1_correct"]
        or mean_diff_r_minus_a["top8_correct"] > delta_noise["top8_correct"]
    )
    r_no_regression = (
        mean_diff_r_minus_a["top1_correct"] >= -delta_noise["top1_correct"]
        and mean_diff_r_minus_a["top8_correct"] >= -delta_noise["top8_correct"]
        and strict_win_no_regression
    )
    r_causal_advantage = (r_beats_a_adm or (r_g_adm and a_g_adm and r_beats_a_rank)) and r_no_regression

    # Decision tree per sheet lines 119-123 & 204-212 (V3-S4: do not suppress A-KEEP when R collapses)
    qualifying_arm: str | None = None
    a_keep_r_disposition: str | None = None
    if r_g_adm and r_bars and not g_occ_triggered and r_causal_advantage:
        qualifying_arm = "K16-R"
        if r_guard:
            decision = "KEEP_K16_R"
        else:
            decision = "GUARD_STRUCTURAL_BIAS_DIAGNOSIS"
    elif a_g_adm and a_bars:
        qualifying_arm = "K16-A"
        a_keep_r_disposition = (
            "A_KEEP_R_NEGATIVE"
            if (g_occ_triggered or not r_g_adm or not r_bars)
            else "A_KEEP_R_NULL_WITHIN_NOISE"
        )
        if a_guard:
            decision = "K16_A_KEEP_R_NULL_AT_SCOPE"
        else:
            decision = "GUARD_STRUCTURAL_BIAS_DIAGNOSIS"
    elif g_occ_triggered:
        decision = "OCCUPANCY_COLLAPSE_TRIGGERED"
    elif (r_g_adm and not g_occ_triggered) or a_g_adm:
        decision = "RANK_BOTTLENECK_ISOLATED"
    else:
        decision = "ADMISSION_STILL_CAPPED_AT_K16"

    # 6. Per-qualifying-arm/seed G-Trans fresh transfer & G-Guard structural-bias audit (V3-S5 & V3-S8)
    fresh_ok = True
    guard_audit_status: str | None = None
    status = "PASS"

    if decision in ("KEEP_K16_R", "K16_A_KEEP_R_NULL_AT_SCOPE", "GUARD_STRUCTURAL_BIAS_DIAGNOSIS"):
        assert qualifying_arm in ("K16-R", "K16-A")
        qual_seeds = (
            ("k16-R-seed1", "k16-R-seed2")
            if qualifying_arm == "K16-R"
            else ("k16-A-seed1", "k16-A-seed2")
        )
        ft_map = data.get("fresh_transfer_receipts", {})
        for q_arm in qual_seeds:
            ft_pin = arms[q_arm].get("fresh_transfer_receipt") or (
                ft_map.get(q_arm) if isinstance(ft_map, dict) else None
            )
            if ft_pin is None:
                fresh_ok = False
                errors.append(
                    f"Decision {decision} requires bound fresh-transfer receipt FilePin for qualifying arm/seed {q_arm}"
                )
                continue
            _, ft_rep = authenticate_file_pin(
                ft_pin, outcomes_dir, f"{q_arm} fresh_transfer_receipt", errors
            )
            if ft_rep is None:
                fresh_ok = False
                continue
            if ft_rep.get("schema") != "uor-r4.fresh-scoring-report/1":
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt schema != 'uor-r4.fresh-scoring-report/1'")
            if ft_rep.get("status") != "FRESH_SCORED_EVAL_ONLY":
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt status != 'FRESH_SCORED_EVAL_ONLY'")
            if ft_rep.get("model_execution") != "CUDA_EVALUATION_ONLY":
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt model_execution != 'CUDA_EVALUATION_ONLY'")
            if (
                "updates_applied_by_evaluation" not in ft_rep
                or not is_strict_int(ft_rep.get("updates_applied_by_evaluation"))
                or ft_rep.get("updates_applied_by_evaluation") != 0
            ):
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt updates_applied_by_evaluation != 0")
            if ft_rep.get("evaluation_mutates_model") is not False:
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt evaluation_mutates_model != false")
            prev_err_count = len(errors)
            verify_zero_mutation_invariants(
                ft_rep,
                "before_typed_snapshot_sha256",
                "after_typed_snapshot_sha256",
                "before_epoch",
                "after_epoch",
                expected_digest=arm_domains[q_arm]["domain_B_typed_model_snapshot_sha256"],
                expected_epoch=arm_epochs[q_arm],
                label=f"{q_arm} fresh_transfer_receipt",
                errors=errors,
            )
            if len(errors) > prev_err_count:
                fresh_ok = False
            if (
                ft_rep.get("checkpoint_model_storage_sha256")
                != arm_domains[q_arm]["domain_A_model_storage_sha256"]
            ):
                fresh_ok = False
                errors.append(
                    f"{q_arm} fresh_transfer_receipt checkpoint_model_storage_sha256 != Domain A"
                )
            if ft_rep.get("training_lineage_sha256") != arm_lineages[q_arm]:
                fresh_ok = False
                errors.append(
                    f"{q_arm} fresh_transfer_receipt training_lineage_sha256 != arm lineage_sha256"
                )
            if ft_rep.get("driver_source_sha256") != expected_driver_src:
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt driver_source_sha256 mismatch")
            if ft_rep.get("rows_sha256") != frozen_binds["fresh_rows_sha256"]:
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt rows_sha256 mismatch")
            if ft_rep.get("documents_sha256") != frozen_binds["fresh_documents_sha256"]:
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt documents_sha256 mismatch")
            fb_src = ft_rep.get("fresh_binding", {}).get("source", {})
            if (
                fb_src.get("tokens", {}).get("sha256") != frozen_binds["fresh_tokens_sha256"]
                or fb_src.get("mask", {}).get("sha256") != frozen_binds["fresh_mask_sha256"]
            ):
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt fresh_binding tokens/mask sha256 mismatch")
            ft_agg = ft_rep.get("aggregate", {})
            if (
                not isinstance(ft_agg, dict)
                or ft_agg.get("scored_rows") != 2000
                or ft_rep.get("scored_rows_expected") != 2000
                or ft_rep.get("frozen_row_inventory") != 2000
            ):
                fresh_ok = False
                errors.append(f"{q_arm} fresh_transfer_receipt scored_rows != 2000")

        if decision == "GUARD_STRUCTURAL_BIAS_DIAGNOSIS":
            audit_pin = data.get("structural_bias_audit_receipt")
            if audit_pin is None:
                guard_audit_status = "DIAGNOSE_STRUCTURAL_BIAS_PENDING_AUDIT"
                errors.append(
                    "Decision GUARD_STRUCTURAL_BIAS_DIAGNOSIS requires bound structural_bias_audit_receipt FilePin"
                )
            else:
                _, audit_obj = authenticate_file_pin(
                    audit_pin, outcomes_dir, "structural_bias_audit_receipt", errors
                )
                if audit_obj is None:
                    guard_audit_status = "DIAGNOSE_STRUCTURAL_BIAS_PENDING_AUDIT"
                else:
                    audit_ok = True
                    if (
                        audit_obj.get("schema") != "uor-r4.guard-structural-bias-audit/1"
                        or audit_obj.get("status") != "PASS"
                        or audit_obj.get("qualifying_arm") != qualifying_arm
                    ):
                        audit_ok = False
                        errors.append(
                            "structural_bias_audit_receipt schema/status/qualifying_arm mismatch"
                        )
                    seeds_audit = audit_obj.get("seeds", {})
                    for q_arm in qual_seeds:
                        sa = seeds_audit.get(q_arm, {}) if isinstance(seeds_audit, dict) else {}
                        q4_adm_sw = arm_q4_agg[q_arm].get("admitted_strict_win_rows", 0)
                        if (
                            not is_strict_int(q4_adm_sw)
                            or q4_adm_sw <= 0
                            or sa.get("admitted_strict_win_rows") != q4_adm_sw
                            or sa.get("model_snapshot_sha256")
                            != arm_domains[q_arm]["domain_B_typed_model_snapshot_sha256"]
                            or sa.get("tie_free_top1_top8_margins_verified") is not True
                        ):
                            audit_ok = False
                            errors.append(
                                f"structural_bias_audit_receipt failed non-vacuity/margin audit for {q_arm}"
                            )
                    guard_audit_status = (
                        "DIAGNOSE_STRUCTURAL_BIAS_VERIFIED"
                        if audit_ok
                        else "DIAGNOSE_STRUCTURAL_BIAS_PENDING_AUDIT"
                    )

        if decision == "GUARD_STRUCTURAL_BIAS_DIAGNOSIS" and guard_audit_status != "DIAGNOSE_STRUCTURAL_BIAS_VERIFIED":
            status = "DIAGNOSE_STRUCTURAL_BIAS_PENDING_AUDIT"
        elif not fresh_ok:
            status = "PENDING_FRESH_TRANSFER"

    return {
        "status": status,
        "decision": decision,
        "qualifying_arm": qualifying_arm,
        "a_keep_r_disposition": a_keep_r_disposition,
        "r_occupancy_collapse_recorded": g_occ_triggered,
        "guard_audit_status": guard_audit_status,
        "g_occ_triggered": g_occ_triggered,
        "r_g_adm": r_g_adm,
        "a_g_adm": a_g_adm,
        "r_bars": r_bars,
        "a_bars": a_bars,
        "r_guard": r_guard,
        "a_guard": a_guard,
        "delta_noise": delta_noise,
        "delta_strict_win": delta_strict_win,
        "strict_win_spread": strict_win_spread,
        "strict_win_no_regression": strict_win_no_regression,
        "mean_diff_r_minus_a": mean_diff_r_minus_a,
        "fresh_transfer_verified": fresh_ok,
        "errors": errors,
    }


def write_pinned_json(path: pathlib.Path, obj: dict[str, Any]) -> dict[str, Any]:
    path.parent.mkdir(parents=True, exist_ok=True)
    raw = (json.dumps(obj, indent=2) + "\n").encode("utf-8")
    path.write_bytes(raw)
    return {
        "path": str(path),
        "sha256": sha256_bytes(raw),
        "bytes": len(raw),
    }


def run_self_test() -> int:
    print("[1/4] Verifying contract v1, v2, v3 and preserved audit backup SHA-256 pins...")
    v1_path = CONSUMER_DIR / "k16-rows-redirect-consumer-contract.json"
    v2_path = CONSUMER_DIR / "k16-rows-redirect-consumer-contract-v2.json"
    v3_path = CONSUMER_DIR / "k16-rows-redirect-consumer-contract-v3.json"
    readme_v3_path = CONSUMER_DIR / "README-v3.md"
    verifier_v3_path = CONSUMER_DIR / "verify_k16_rows_redirect-v3.py"
    readme_8e3a2b_path = CONSUMER_DIR / "README-8e3a2b.md"
    verifier_c02d31_path = CONSUMER_DIR / "verify_k16_rows_redirect-c02d31.py"
    readme_410fa4de_path = CONSUMER_DIR / "README-410fa4de-superseded.md"
    verifier_5c3bc562_path = CONSUMER_DIR / "verify_k16_rows_redirect-5c3bc562-superseded.py"
    readme_path = CONSUMER_DIR / "README.md"

    assert sha256_bytes(v1_path.read_bytes()) == CONTRACT_V1_SHA256, "v1 SHA mismatch"
    assert sha256_bytes(v2_path.read_bytes()) == CONTRACT_V2_SHA256, "v2 SHA mismatch"
    assert sha256_bytes(v3_path.read_bytes()) == CONTRACT_V3_SHA256, "v3 SHA mismatch"
    if readme_v3_path.exists():
        assert sha256_bytes(readme_v3_path.read_bytes()) == README_V3_SHA256, "README-v3.md SHA mismatch"
    if verifier_v3_path.exists():
        assert (
            sha256_bytes(verifier_v3_path.read_bytes()) == VERIFIER_V3_SHA256
        ), "verify_k16_rows_redirect-v3.py SHA mismatch"
    if readme_8e3a2b_path.exists():
        assert (
            sha256_bytes(readme_8e3a2b_path.read_bytes()) == README_8E3A2B_SHA256
        ), "README-8e3a2b.md SHA mismatch"
    if verifier_c02d31_path.exists():
        assert (
            sha256_bytes(verifier_c02d31_path.read_bytes()) == VERIFIER_C02D31_SHA256
        ), "verify_k16_rows_redirect-c02d31.py SHA mismatch"
    if readme_410fa4de_path.exists():
        assert (
            sha256_bytes(readme_410fa4de_path.read_bytes()) == README_410FA4DE_SHA256
        ), "README-410fa4de-superseded.md SHA mismatch"
    if verifier_5c3bc562_path.exists():
        assert (
            sha256_bytes(verifier_5c3bc562_path.read_bytes()) == VERIFIER_5C3BC562_SHA256
        ), "verify_k16_rows_redirect-5c3bc562-superseded.py SHA mismatch"
    c = json.loads(v3_path.read_bytes())

    print("[2/4] Auditing v3 contract, pod-operator lineages, staged K16 per-role admissions, and companion README.md...")
    assertions_v1_path = CODEX_FASTPATH_DIR / "agy-consumer-review" / "assertions.json"
    if assertions_v1_path.exists():
        v1_assertions = json.loads(assertions_v1_path.read_text())["assertions"]

        def at(obj: Any, ptr: str) -> Any:
            for k in ptr.strip("/").split("/"):
                obj = obj[int(k)] if isinstance(obj, list) else obj[k]
            return obj

        for item in v1_assertions:
            p = item["json_pointer"]
            if (
                p.startswith("/frozen_data_and_geometry_bindings/")
                or p
                in (
                    "/owner_and_pm_authority/sheet_sha256",
                    "/owner_and_pm_authority/fit_review_comment",
                    "/driver_source_and_arm_bindings_at_fb9e004/driver_source_files_hashed_in_order",
                    "/driver_source_and_arm_bindings_at_fb9e004/driver_source_sha256",
                    "/producer_consumer_compatibility_audit/fresh_scoring_eval/details/1",
                )
                or p.startswith("/acceptance_and_decision_bars/trained_q4_16404_batches/")
            ):
                actual = at(c, p)
                expected = item["expected"]
                assert actual == expected, f"Mismatch at {p}: actual={actual!r} != expected={expected!r}"

    # Verify 11-file driver_source_sha256 from git HEAD
    rev = c["ownership_boundaries"]["shared_repo_head_inspected"]
    h_driver = hashlib.sha256()
    for rel_path in c["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_files_hashed_in_order"]:
        blob = subprocess.check_output(["git", "show", f"{rev}:{rel_path}"], cwd=REPO_ROOT)
        h_driver.update(blob)
    assert (
        h_driver.hexdigest()
        == c["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_sha256"]
    ), "driver_source_sha256 recompute mismatch"

    # Recompute all repo-relative, Codex-staged, and pod-operator config_sha256 and lineage_sha256 digests
    cov_blob = subprocess.check_output(
        ["git", "show", f"{rev}:docs/labs/discrete-driver-cost-2026-10-13/receipts/coverage-plan.json"],
        cwd=REPO_ROOT,
    )
    cov_meta = json.loads(cov_blob)["lineage"]
    arm_Configs: dict[str, dict[str, Any]] = {}
    arm_Lineages: dict[str, dict[str, Any]] = {}
    for arm, values in c["driver_source_and_arm_bindings_at_fb9e004"]["arms"].items():
        for domain in ("repo_relative_paths", "codex_local_physical_paths", "pod_operator_paths"):
            if domain == "repo_relative_paths":
                raw = subprocess.check_output(
                    ["git", "show", f"{rev}:docs/labs/admission-redirect-packet-2026-10-15/configs/{arm}.json"],
                    cwd=REPO_ROOT,
                )
                expected_cfg_sha = values[domain]["rows_v1_typed_config_sha256"]
                expected_lin_sha = values[domain]["rows_v1_lineage_sha256"]
            elif domain == "codex_local_physical_paths":
                raw = (CODEX_FASTPATH_DIR / "configs" / f"{arm}.json").read_bytes()
                assert (
                    sha256_bytes(raw) == values[domain]["staged_raw_file_sha256"]
                ), f"{arm} staged_raw_file_sha256 mismatch"
                expected_cfg_sha = values[domain]["rows_v1_typed_config_sha256"]
                expected_lin_sha = values[domain]["rows_v1_lineage_sha256"]
            else:
                pod_raw_path = (
                    CODEX_FASTPATH_DIR / "qualification-prep" / "operator" / "configs" / f"{arm}.json"
                )
                raw = pod_raw_path.read_bytes()
                assert (
                    sha256_bytes(raw) == POD_OPERATOR_ARM_BINDINGS[arm]["staged_raw_file_sha256"]
                ), f"{arm} pod-operator staged_raw_file_sha256 mismatch"
                expected_cfg_sha = POD_OPERATOR_ARM_BINDINGS[arm]["rows_v1_typed_config_sha256"]
                expected_lin_sha = POD_OPERATOR_ARM_BINDINGS[arm]["rows_v1_lineage_sha256"]
            cfg = json.loads(raw)
            cfg["counterfactual_path"] = "rows_v1"
            cfg_bytes = native_driver_config_form(cfg)
            assert (
                sha256_bytes(cfg_bytes) == expected_cfg_sha
            ), f"{arm} {domain} config_sha256 mismatch"
            cfg_ordered = json.loads(cfg_bytes)
            lineage: dict[str, Any] = {
                "schema": cfg_ordered["schema"],
                "driver_source_sha256": c["driver_source_and_arm_bindings_at_fb9e004"][
                    "driver_source_sha256"
                ],
                "config": cfg_ordered,
            }
            for split in ("train", "dev"):
                b = copy.deepcopy(cov_meta[split])
                b["source"] = cfg_ordered[split]
                b["bos"] = cfg_ordered["bos"]
                lineage[split] = {k: b[k] for k in BINDING_ORDER}
            assert (
                sha256_bytes(compact_json_bytes(lineage)) == expected_lin_sha
            ), f"{arm} {domain} lineage_sha256 mismatch"
            if domain == "repo_relative_paths":
                arm_Configs[arm] = cfg_ordered
                arm_Lineages[arm] = lineage

    # Verify pod-operator K8 control config/lineage bindings
    for k8_arm, k8_pins in POD_OPERATOR_K8_BINDINGS.items():
        k8_raw = (
            CODEX_FASTPATH_DIR / "qualification-prep" / "operator" / "configs" / f"{k8_arm}.json"
        ).read_bytes()
        assert (
            sha256_bytes(k8_raw) == k8_pins["staged_raw_file_sha256"]
        ), f"{k8_arm} pod-operator staged_raw_file_sha256 mismatch"
        k8_cfg = json.loads(k8_raw)
        k8_cfg_bytes = native_driver_config_form(k8_cfg)
        assert (
            sha256_bytes(k8_cfg_bytes) == k8_pins["rows_v1_typed_config_sha256"]
        ), f"{k8_arm} pod-operator rows_v1_typed_config_sha256 mismatch"
        k8_cfg_ordered = json.loads(k8_cfg_bytes)
        k8_lin_obj: dict[str, Any] = {
            "schema": k8_cfg_ordered["schema"],
            "driver_source_sha256": c["driver_source_and_arm_bindings_at_fb9e004"][
                "driver_source_sha256"
            ],
            "config": k8_cfg_ordered,
        }
        for split in ("train", "dev"):
            b = copy.deepcopy(cov_meta[split])
            b["source"] = k8_cfg_ordered[split]
            b["bos"] = k8_cfg_ordered["bos"]
            k8_lin_obj[split] = {k: b[k] for k in BINDING_ORDER}
        assert (
            sha256_bytes(compact_json_bytes(k8_lin_obj)) == k8_pins["rows_v1_lineage_sha256"]
        ), f"{k8_arm} pod-operator rows_v1_lineage_sha256 mismatch"

    # Verify real staged K16 per-role CompletionAdmission / CompletionPlan / Qualification / PMReview / Owner artifacts when present
    if STAGED_K16_METADATA_DIR.is_dir():
        for arm_name, r_pins in DISPATCHED_K16_ROLE_RELEASE_PINS.items():
            adm_file = (
                STAGED_K16_METADATA_DIR
                / "source-releases-private"
                / f"release-{arm_name}"
                / "admission.json"
            )
            plan_file = STAGED_K16_METADATA_DIR / "metadata" / f"PLAN-{arm_name}.json"
            qual_file = STAGED_K16_METADATA_DIR / "metadata" / f"QUALIFICATION-{arm_name}.json"
            pm_file = STAGED_K16_METADATA_DIR / "metadata" / f"PMREVIEW-{arm_name}.json"
            owner_file = (
                STAGED_K16_METADATA_DIR
                / "source-releases-private"
                / f"release-{arm_name}"
                / "owner.json"
            )
            assert adm_file.exists(), f"Missing staged {adm_file}"
            adm_raw = adm_file.read_bytes()
            assert (
                sha256_bytes(adm_raw) == r_pins["admission_sha256"]
            ), f"{arm_name} admission.json file_sha256 mismatch"
            adm_obj = json.loads(adm_raw)
            assert (
                sha256_bytes(native_completion_admission_form(adm_obj))
                == r_pins["admission_sha256"]
            ), f"{arm_name} native_completion_admission_form sha256 mismatch"
            assert (
                sha256_bytes(native_completion_plan_form(adm_obj["plan"]))
                == r_pins["plan_sha256"]
            ), f"{arm_name} native_completion_plan_form sha256 mismatch"
            assert plan_file.exists(), f"Missing staged {plan_file}"
            plan_obj_staged = json.loads(plan_file.read_bytes())
            assert (
                sha256_bytes(native_completion_plan_form(plan_obj_staged))
                == r_pins["plan_sha256"]
            ), f"{arm_name} PLAN file native_completion_plan_form sha256 mismatch"
            assert qual_file.exists(), f"Missing staged {qual_file}"
            assert (
                sha256_bytes(qual_file.read_bytes()) == r_pins["qualification_sha256"]
            ), f"{arm_name} QUALIFICATION file_sha256 mismatch"
            assert pm_file.exists(), f"Missing staged {pm_file}"
            assert (
                sha256_bytes(pm_file.read_bytes()) == r_pins["pm_review_sha256"]
            ), f"{arm_name} PMREVIEW file_sha256 mismatch"
            assert owner_file.exists(), f"Missing staged {owner_file}"
            assert (
                sha256_bytes(owner_file.read_bytes()) == r_pins["owner_sha256"]
            ), f"{arm_name} owner.json file_sha256 mismatch"

    if SEALED_K16_CUSTODY_DIR.is_dir() and SEALED_K16_MEMBER_MAP_DIR.is_dir():
        cust_errs: list[str] = []
        cust_shas, cust_role_pins = authenticate_off_host_custody_manifests(
            {"off_host_custody": "SEALED_K16_FOUR_ROLE_CUSTODY_2324Z"},
            SEALED_K16_CUSTODY_DIR,
            [SEALED_K16_CUSTODY_DIR, SEALED_K16_MEMBER_MAP_DIR, STAGED_K16_METADATA_DIR],
            cust_errs,
        )
        assert not cust_errs and len(cust_shas) >= 1130 and len(cust_role_pins) == 4, (
            f"Sealed K16 four-role custody authentication failed: {cust_errs}"
        )

    # Verify Q4 wrapper domain A, B, C, and D pins against authenticated Q4 checkpoint if present
    q4_ckpt_path = CODEX_FASTPATH_DIR / "q4-original-authenticated" / "checkpoint-16404.json"
    if q4_ckpt_path.exists():
        q4_raw = q4_ckpt_path.read_bytes()
        assert (
            sha256_bytes(q4_raw)
            == c["frozen_data_and_geometry_bindings"]["sealed_seed1_q4_wrapper"]["wrapper_file_sha256"]
        )
        q4_obj = json.loads(q4_raw)
        domains = compute_snapshot_hash_domains(q4_obj, q4_raw)
        assert (
            domains["domain_A_model_storage_sha256"]
            == c["frozen_data_and_geometry_bindings"]["sealed_seed1_q4_wrapper"][
                "inner_model_storage_sha256"
            ]
        )
        assert (
            domains["domain_B_typed_model_snapshot_sha256"]
            == c["frozen_data_and_geometry_bindings"]["sealed_seed1_q4_wrapper"][
                "inner_model_typed_snapshot_sha256_source_derived"
            ]
        )
        assert domains["domain_C_driver_checkpoint_sha256"] == Q4_ORIGINAL_DOMAIN_C_SHA256
        assert (
            domains["domain_D_checkpoint_file_sha256"]
            == c["frozen_data_and_geometry_bindings"]["sealed_seed1_q4_wrapper"]["wrapper_file_sha256"]
        )
        assert (
            domains["declared_model_sha256"]
            == c["timing_receipt_consumer_binding"]["pending_same_unit_dependency"][
                "inner_storage_pin"
            ]
        )

    # Verify companion README.md pins (including current-authority amendment pins)
    readme_text = readme_path.read_text()
    inv_sha = sha256_bytes((CODEX_FASTPATH_DIR / "configs" / "inventory.json").read_bytes())
    assert inv_sha in readme_text, "README.md missing current inventory.json SHA-256"
    for arm, values in c["driver_source_and_arm_bindings_at_fb9e004"]["arms"].items():
        assert (
            values["codex_local_physical_paths"]["staged_raw_file_sha256"] in readme_text
        ), f"README.md missing staged_raw_file_sha256 for {arm}"
        assert (
            POD_OPERATOR_ARM_BINDINGS[arm]["rows_v1_typed_config_sha256"] in readme_text
        ), f"README.md missing pod-operator config_sha256 for {arm}"
        assert (
            POD_OPERATOR_ARM_BINDINGS[arm]["rows_v1_lineage_sha256"] in readme_text
        ), f"README.md missing pod-operator lineage_sha256 for {arm}"
        assert (
            DISPATCHED_K16_ROLE_RELEASE_PINS[arm]["admission_sha256"] in readme_text
        ), f"README.md missing per-role admission_sha256 for {arm}"
    assert "historical/unverified template only" in readme_text
    assert "Domain A and Domain B hashes are distinct and are never compared to each other" in readme_text
    assert "https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6199263447" in readme_text
    assert "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125" in readme_text
    assert ATTEMPT5_ROWS_PARITY_SHA256 in readme_text, "README.md missing attempt5 rows-parity.json SHA-256"
    assert (
        ATTEMPT5_ROWS_PARITY_NEG_SHA256 in readme_text
    ), "README.md missing attempt5 rows-parity-negative.json raw SHA-256"
    assert POSITIVE_ROWS_CU_SHA256 in readme_text, "README.md missing positive rows.cu SHA-256"
    assert NEGATIVE_ROWS_CU_SHA256 in readme_text, "README.md missing negative rows.cu SHA-256"
    assert VERIFIER_5C3BC562_SHA256 in readme_text, "README.md missing preserved 5c3bc562 verifier SHA-256"
    assert README_410FA4DE_SHA256 in readme_text, "README.md missing preserved 410fa4de README SHA-256"
    assert "CURRENT-8" in readme_text, "README.md missing CURRENT-8 section"
    assert (
        "0009096d642fd8cc450882610363f954b76e7f7c4f24b756847bc058c0b4bd1b" not in readme_text
    ), "README.md still contains wrong rows-parity-negative SHA-256"

    print(
        "[3/4] Running qualification verifier positive + negative unit tests "
        "(V3-S1, V3-S2, V3-S7, V3-S8, CURRENT-1..CURRENT-7)..."
    )
    with tempfile.TemporaryDirectory() as tmp:
        tdir = pathlib.Path(tmp)
        parity_blob = subprocess.check_output(
            [
                "git",
                "show",
                f"{rev}:docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt5/rows-parity.json",
            ],
            cwd=REPO_ROOT,
        )
        parity_neg_blob = subprocess.check_output(
            [
                "git",
                "show",
                f"{rev}:docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt5/rows-parity-negative.json",
            ],
            cwd=REPO_ROOT,
        )
        assert sha256_bytes(parity_blob) == ATTEMPT5_ROWS_PARITY_SHA256
        assert sha256_bytes(parity_neg_blob) == ATTEMPT5_ROWS_PARITY_NEG_SHA256
        # Write in native RUN layout (rows-positive/rows-parity.json, rows-negative/rows-parity-negative.json, c-r-fixtures/...)
        (tdir / "rows-positive").mkdir(parents=True, exist_ok=True)
        (tdir / "rows-negative").mkdir(parents=True, exist_ok=True)
        (tdir / "c-r-fixtures").mkdir(parents=True, exist_ok=True)
        (tdir / "rows-positive" / "rows-parity.json").write_bytes(parity_blob)
        (tdir / "rows-negative" / "rows-parity.json").write_bytes(parity_neg_blob)
        parity_obj = json.loads(parity_blob)
        pos_rows_img = parity_obj["rows_image_sha256"]
        pos_source_sha = NATIVE_COMPILED_SOURCE_SHA256
        pos_kernel_sha = ADMITTED_KERNEL_SHA256

        # Build authentic checkpoint0 models for each K16 role so Domain A/B/C/D and timing digests cross-verify
        frozen = c["frozen_data_and_geometry_bindings"]
        drv_src = c["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_sha256"]
        role_c0_models: dict[str, dict[str, Any]] = {}
        role_c0_dom_a: dict[str, str] = {}
        role_c0_dom_b: dict[str, str] = {}
        for arm_name, seed_num, pol_name in K16_FOUR_ROLES_IN_ORDER:
            m_c0: dict[str, Any] = {
                "schema": "uor-r4.discrete-spiralcore-fbs/2",
                "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16},
                "read_spec": {"kind": "route"},
            }
            if pol_name != "nearest_address_margin_v1":
                m_c0["objective_policy"] = pol_name
            m_c0.update(
                {
                    "seed": seed_num,
                    "epoch": 0,
                    "table": [seed_num] * 32,
                    "routes": [seed_num] * 8,
                    "gates": [seed_num] * 8,
                    "operator_kappa": frozen["operator_kappa"],
                    "composition_kappa": frozen["composition_kappa"],
                    "kernel_sha256": pos_kernel_sha,
                    "source_sha256": pos_source_sha,
                    "geometry_sha256": frozen["geometry_sha256"],
                }
            )
            role_c0_models[arm_name] = m_c0
            role_c0_dom_a[arm_name] = sha256_bytes(canonical_sorted_json_bytes(m_c0))
            role_c0_dom_b[arm_name] = sha256_bytes(native_snapshot_form(m_c0))

        # Write optional g-pre-*-init/report.json and checkpoint-00000.json in native RUN layout
        for arm_name, seed_num, pol_name in K16_FOUR_ROLES_IN_ORDER:
            init_dir = tdir / f"g-pre-{arm_name}-init"
            init_dir.mkdir(parents=True, exist_ok=True)
            lin = arm_Lineages[arm_name]
            lin_sha = sha256_bytes(compact_json_bytes(lin))
            cfg_sha = sha256_bytes(native_driver_config_form(arm_Configs[arm_name]))
            c0_ckpt_obj = {
                "schema": "uor-r4.discrete-driver-checkpoint/1",
                "lineage": lin,
                "cursor": {
                    "completed_batches": 0,
                    "accepted_updates": 0,
                    "corpus_pass": 0,
                    "next_target": 0,
                    "next_coordinate": 0,
                },
                "model_sha256": role_c0_dom_a[arm_name],
                "model": role_c0_models[arm_name],
            }
            c0_dom_c = sha256_bytes(native_driver_checkpoint_form(c0_ckpt_obj))
            c0_ckpt_pin = write_pinned_json(init_dir / "checkpoint-00000.json", c0_ckpt_obj)
            write_pinned_json(
                init_dir / "report.json",
                {
                    "schema": "uor-r4.discrete-corpus-driver/1",
                    "driver_source_sha256": drv_src,
                    "config_sha256": cfg_sha,
                    "lineage_sha256": lin_sha,
                    "checkpoint_sha256": c0_dom_c,
                    "checkpoint_file": {
                        "path": str(init_dir / "checkpoint-00000.json"),
                        "sha256": c0_ckpt_pin["sha256"],
                        "bytes": c0_ckpt_pin["bytes"],
                    },
                    "model_sha256": role_c0_dom_a[arm_name],
                    "model_snapshot_sha256": role_c0_dom_b[arm_name],
                },
            )

        # Genuine 2-cell checkpoint0 k16-timing Ledger schema (Owner Waiver #2029 comment 6251027125)
        timing_records: list[dict[str, Any]] = []
        for src, pol in ACTIVE_TIMING_CELLS_CHECKPOINT0:
            arm_for_cell = (
                "k16-A-seed1" if pol == "nearest_address_margin_v1" else "k16-R-seed1"
            )
            ep = 0
            for b_idx in range(1, 101):
                timing_records.append(
                    {
                        "record_kind": "batch",
                        "counterfactual_path": "rows_v1",
                        "cell": {"source": src, "policy": pol},
                        "batch": b_idx,
                        "rows": 64,
                        "carry_ns": 1000,
                        "proposals": [
                            {
                                "operation": "route",
                                "counterfactual_path": "rows_v1",
                                "applied": False,
                            },
                            {
                                "operation": "vector_block",
                                "counterfactual_path": "rows_v1",
                                "applied": False,
                            },
                        ],
                        "epoch": ep,
                        "applied": False,
                    }
                )
            timing_records.append(
                {
                    "record_kind": "cell",
                    "counterfactual_path": "rows_v1",
                    "cell": {"source": src, "policy": pol},
                    "timed_batches": 100,
                    "excluded_warmups": 1,
                    "applied": False,
                    "snapshot_digest_before": role_c0_dom_b[arm_for_cell],
                    "snapshot_digest_after": role_c0_dom_b[arm_for_cell],
                    "epoch_before": ep,
                    "epoch_after": ep,
                    "identities": {
                        "rows_image_sha256": pos_rows_img,
                        "compiled": {
                            "source_sha256": pos_source_sha,
                            "kernel_sha256": pos_kernel_sha,
                            "geometry_sha256": frozen["geometry_sha256"],
                        },
                    },
                }
            )

        valid_timing = {
            "schema": "k16-timing-record/v1",
            "status": "complete",
            "header": {
                "schema": "k16-timing-plan/v1",
                "counterfactual_path": "rows_v1",
                "batches_per_cell": 100,
                "excluded_warmups_per_cell": 1,
                "apply": False,
            },
            "records": timing_records,
            "faults": [],
        }
        (tdir / "k16-timing-record.json").write_text(json.dumps(valid_timing))

        # Genuine C-R1..C-R6 control reports and G-Pre receipts pinned via FilePin
        k8_lin = copy.deepcopy(arm_Lineages["k16-A-seed1"])
        k8_lin["config"]["spec"]["kept_cells"] = 8
        k8_lin["config"]["counterfactual_path"] = "rows_v1"
        k8_lin_ordered = native_lineage_dict(k8_lin)
        k8_lin_sha = sha256_bytes(compact_json_bytes(k8_lin_ordered))
        k8_gpre_pin = write_pinned_json(
            tdir / "g-pre-k8-A-seed1-eval/report.json",
            {
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "evaluation_lineage_sha256": k8_lin_sha,
                "evaluation_lineage": k8_lin_ordered,
                "model_epoch": 0,
                "model_storage_sha256": "1" * 64,
                "model_snapshot_sha256": "2" * 64,
                "evaluation_mutates_model": False,
                "accepted_updates_in_model": 0,
                "updates_applied_by_evaluation": 0,
                "model_bindings": {
                    "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
                    "read_spec": {"kind": "route"},
                    "source_sha256": pos_source_sha,
                    "kernel_sha256": pos_kernel_sha,
                    "geometry_sha256": frozen["geometry_sha256"],
                    "operator_kappa": frozen["operator_kappa"],
                    "composition_kappa": frozen["composition_kappa"],
                    "objective_policy": "nearest_address_margin_v1",
                    "kept_cells": 8,
                    "seed": 1,
                },
                "aggregate": {
                    "scored_rows": 14287,
                    "target_admitted_rows": 5680,
                    "top1_correct": 17,
                    "top8_correct": 173,
                    "token_strict_win_eligible_rows": 105,
                    "address_hinge_sum": 261824,
                    "occupied_cells": 16,
                },
            },
        )
        k16_gpre_entries = []
        for arm_name, seed_num, pol_name in K16_FOUR_ROLES_IN_ORDER:
            run_rel = f"g-pre-{arm_name}-eval/report.json"
            lin = arm_Lineages[arm_name]
            lin_sha = sha256_bytes(compact_json_bytes(lin))
            pin = write_pinned_json(
                tdir / run_rel,
                {
                    "schema": "uor-r4.discrete-corpus-driver/1",
                    "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                    "evaluation_lineage_sha256": lin_sha,
                    "evaluation_lineage": lin,
                    "model_epoch": 0,
                    "model_storage_sha256": role_c0_dom_a[arm_name],
                    "model_snapshot_sha256": role_c0_dom_b[arm_name],
                    "evaluation_mutates_model": False,
                    "accepted_updates_in_model": 0,
                    "updates_applied_by_evaluation": 0,
                    "model_bindings": {
                        "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16},
                        "read_spec": {"kind": "route"},
                        "source_sha256": pos_source_sha,
                        "kernel_sha256": pos_kernel_sha,
                        "geometry_sha256": frozen["geometry_sha256"],
                        "operator_kappa": frozen["operator_kappa"],
                        "composition_kappa": frozen["composition_kappa"],
                        "objective_policy": pol_name,
                        "kept_cells": 16,
                        "seed": seed_num,
                    },
                    "aggregate": {
                        "scored_rows": 14287,
                        "target_admitted_rows": 14287,
                        "occupied_cells": 16,
                        "admitted_cells": 16,
                        "cell_rank_hinge_sum": 0,
                    },
                },
            )
            k16_gpre_entries.append(
                {
                    "arm": arm_name,
                    "seed": seed_num,
                    "objective_policy": pol_name,
                    "kept_cells": 16,
                    "receipt": pin,
                    "target_admitted_rows": 14287,
                    "occupied_cells": 16,
                    "admitted_cells": 16,
                    "cell_rank_hinge_sum": 0,
                }
            )

        cr_reports = {
            "C-R1": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R1",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "fixture_1a_mixed_distance": {
                    "occupied_cells": 5,
                    "target_cell": 15,
                    "target_cell_rank": 3,
                    "cell_rank_hinge_k1_to_16": [2, 1] + [0] * 14,
                    "target_admitted_k1_to_16": [False, False] + [True] * 14,
                },
                "fixture_1a_equal_distance_tie": {
                    "occupied_cells": 5,
                    "target_cell": 15,
                    "target_cell_rank": 3,
                    "cell_rank_hinge_k1_to_16": [2, 1] + [0] * 14,
                    "target_admitted_k1_to_16": [False, False] + [True] * 14,
                },
                "fixture_1b_all_256_cells_k16": {
                    "occupied_cells": 256,
                    "admitted_cells": 16,
                    "target_cell_rank": 256,
                    "cell_rank_hinge": 240,
                    "weighted_cell_rank_hinge_sum": 7680,
                },
            },
            "C-R2": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R2",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "part_2a_crowded_rival_cell": {
                    "rival_cell": 1,
                    "multiplicities": [1, 2, 4],
                    "target_cell_rank": [3, 3, 3],
                    "cell_rank_hinge_k1": [2, 2, 2],
                    "cell_rank_hinge_k2": [1, 1, 1],
                    "cell_rank_hinge_k3": [0, 0, 0],
                    "admitted_token_count_k3": [3, 4, 6],
                },
                "part_2b_new_preceding_cell": {
                    "new_preceding_cell": 2,
                    "kept_cells_swept": [1, 2, 3, 4],
                    "occupied_cells_before": 5,
                    "occupied_cells_after": 6,
                    "target_cell_rank_before": 3,
                    "target_cell_rank_after": 4,
                    "cell_rank_hinge_before_k1_to_k4": [2, 1, 0, 0],
                    "cell_rank_hinge_after_k1_to_k4": [3, 2, 1, 0],
                },
            },
            "C-R3": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R3",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "total_candidate_evaluations_verified": 160,
                "mutations": [
                    {
                        "mutation": m_name,
                        "candidates_verified_per_policy": 16,
                        "policies_verified": [
                            "nearest_address_margin_v1",
                            "occupied_cell_rank_hinge_v1",
                        ],
                        "passed": True,
                    }
                    for m_name in (
                        "create_cell",
                        "empty_cell",
                        "move_target_cell",
                        "interior_block_only",
                        "identity_candidate",
                    )
                ],
            },
            "C-R4": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R4",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "occupancy_under_k": {
                    "m1_at_k16": {
                        "occupied_cells": 1,
                        "target_cell_rank": 1,
                        "cell_rank_hinge": 0,
                        "target_admitted": True,
                    },
                    "m16_at_k16": {
                        "occupied_cells": 16,
                        "target_admitted_rows": 16,
                        "cell_rank_hinge_sum": 0,
                    },
                },
                "absent_rival_checks": {
                    "wholly_ineligible_refusal_status": 8,
                    "mixed_batch_skips_tr_zero_identically": True,
                    "policies_verified": [
                        "nearest_address_margin_v1",
                        "occupied_cell_rank_hinge_v1",
                    ],
                },
            },
            "C-R5": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R5",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "row_words": CR5_ROW_WORDS,
                "aggregate_words": CR5_AGGREGATE_WORDS,
                "histogram_bins": CR5_HISTOGRAM_BINS,
                "dual_loss_and_summary_aggregation_verified": True,
                "status12_overflow_checks": {
                    "ptx_sha256": pos_kernel_sha,
                    "matches_device_compiled_kernel_sha256": True,
                    "baseline_valid_row_status": 0,
                    "all_passed": True,
                    "cases": [
                        {
                            "case": cname,
                            "observed_status": 12,
                            "expected_status": 12,
                            "passed": True,
                        }
                        for cname in CR5_STATUS12_CASE_NAMES
                    ],
                },
            },
            "C-R6": {
                "schema": "uor-r4.admission-rank-control-report/1",
                "id": "C-R6",
                "status": "PASS",
                "apply_calls": 0,
                "cpu_model_reference_used": False,
                "stage_a_sweep_sha256": STAGE_A_SWEEP_SHA256,
                "baseline_admitted_cells_guard_range": [1, 16],
                "g_pre_k16_minimum": 4894,
                "v4096_checkpoint0_device_checks": [
                    {
                        "seed": s,
                        "kept_cells": k,
                        "occupied_cells": 16,
                        "admitted_cells": k,
                        "target_admitted_rows": 1 if k == 16 else 0,
                        "cell_rank_hinge_sum": 0 if k == 16 else 4,
                    }
                    for s in (1, 2)
                    for k in (8, 16)
                ],
                "g_pre_k8_seed1_receipt": k8_gpre_pin,
                "g_pre_k16_receipts": k16_gpre_entries,
            },
        }
        control_receipts = []
        for cid in ("C-R1", "C-R2", "C-R3", "C-R4", "C-R5", "C-R6"):
            pin = write_pinned_json(
                tdir / "c-r-fixtures" / f"{cid.lower()}/report.json", cr_reports[cid]
            )
            control_receipts.append({"id": cid, "status": "PASS", "receipt": pin})

        valid_cr = {
            "schema": "uor-r4.admission-rank-fixture-runner/1",
            "status": "PASS",
            "model_compute_executed": True,
            "apply_calls": 0,
            "cpu_model_reference_used": False,
            "controls": control_receipts,
        }
        (tdir / "c-r-fixtures" / "c-r-suite-summary.json").write_text(json.dumps(valid_cr))

        res_pos = verify_qualification_packet(tdir, c)
        assert res_pos["status"] == "PASS", f"Expected PASS, got {res_pos}"

        # Negative test CURRENT-1: adding a sealed_seed1_q4 cell when legacy_4cell_timing=False must fail
        bad_timing_4c = copy.deepcopy(valid_timing)
        extra_cell = copy.deepcopy(bad_timing_4c["records"][-1])
        extra_cell["cell"] = {"source": "sealed_seed1_q4", "policy": "nearest_address_margin_v1"}
        bad_timing_4c["records"].append(extra_cell)
        (tdir / "k16-timing-record.json").write_text(json.dumps(bad_timing_4c))
        res_neg_c1 = verify_qualification_packet(tdir, c)
        assert res_neg_c1["status"] == "FAIL" and any(
            "cell records count" in e or "cell pairs" in e for e in res_neg_c1["errors"]
        ), f"Expected CURRENT-1 failure on unexpected sealed_seed1_q4 cell, got {res_neg_c1}"

        # Negative test CURRENT-2: scalar string rows_image_sha256 in k16-timing must fail
        bad_timing_img = copy.deepcopy(valid_timing)
        cell_idx = next(
            i for i, r in enumerate(bad_timing_img["records"]) if r.get("record_kind") == "cell"
        )
        bad_timing_img["records"][cell_idx]["identities"]["rows_image_sha256"] = "a" * 64
        (tdir / "k16-timing-record.json").write_text(json.dumps(bad_timing_img))
        res_neg_c2 = verify_qualification_packet(tdir, c)
        assert res_neg_c2["status"] == "FAIL" and any(
            "rows_image_sha256" in e for e in res_neg_c2["errors"]
        ), f"Expected CURRENT-2 failure on scalar rows_image_sha256, got {res_neg_c2}"

        # V3-S1/V3-S8 negative test: missing snapshot_digest_before/after (None == None) must fail
        bad_timing = copy.deepcopy(valid_timing)
        bad_timing["records"][cell_idx]["snapshot_digest_before"] = None
        bad_timing["records"][cell_idx]["snapshot_digest_after"] = None
        (tdir / "k16-timing-record.json").write_text(json.dumps(bad_timing))
        res_neg_s8 = verify_qualification_packet(tdir, c)
        assert res_neg_s8["status"] == "FAIL" and any(
            "64-hex" in e for e in res_neg_s8["errors"]
        ), f"Expected V3-S8 failure, got {res_neg_s8}"
        (tdir / "k16-timing-record.json").write_text(json.dumps(valid_timing))

        # Negative test CURRENT-3: stale C-R5 constants (25 / 1126 / [513,257,241]) or missing 15 cases must fail
        bad_cr5 = copy.deepcopy(cr_reports["C-R5"])
        bad_cr5["row_words"] = 25
        bad_cr5["aggregate_words"] = 1126
        bad_cr5["histogram_bins"] = [513, 257, 241]
        bad_cr5["status12_overflow_checks"] = {"verified": True}
        bad_cr5_pin = write_pinned_json(tdir / "c-r-fixtures" / "c-r5/report.json", bad_cr5)
        for entry in valid_cr["controls"]:
            if entry["id"] == "C-R5":
                entry["receipt"] = bad_cr5_pin
        (tdir / "c-r-fixtures" / "c-r-suite-summary.json").write_text(json.dumps(valid_cr))
        res_neg_c3 = verify_qualification_packet(tdir, c)
        assert res_neg_c3["status"] == "FAIL" and any(
            "C-R5" in e for e in res_neg_c3["errors"]
        ), f"Expected CURRENT-3 failure on stale C-R5 report, got {res_neg_c3}"
        # Restore valid C-R5 and update its FilePin in c-r-suite-summary.json
        cr5_pin_restored = write_pinned_json(
            tdir / "c-r-fixtures" / "c-r5/report.json", cr_reports["C-R5"]
        )
        for entry in valid_cr["controls"]:
            if entry["id"] == "C-R5":
                entry["receipt"] = cr5_pin_restored
        (tdir / "c-r-fixtures" / "c-r-suite-summary.json").write_text(json.dumps(valid_cr))

        # Negative test CURRENT-4 / V3-S7: wrong error message or positive rows.cu in rows-parity.json must fail
        bad_neg = json.loads(parity_neg_blob)
        bad_neg["rows_image_sha256"][0] = POSITIVE_ROWS_CU_SHA256
        (tdir / "rows-negative" / "rows-parity.json").write_text(json.dumps(bad_neg))
        res_neg_c4 = verify_qualification_packet(tdir, c)
        assert res_neg_c4["status"] == "FAIL" and any(
            NEGATIVE_ROWS_CU_SHA256 in e or "rows_image_sha256" in e for e in res_neg_c4["errors"]
        ), f"Expected CURRENT-4 failure on positive rows.cu in negative receipt, got {res_neg_c4}"

        bad_neg2 = json.loads(parity_neg_blob)
        bad_neg2["controls"] = [
            ctrl for ctrl in bad_neg2["controls"] if ctrl["name"] != "rows_forced_terminal_parity"
        ]
        (tdir / "rows-negative" / "rows-parity.json").write_text(json.dumps(bad_neg2))
        res_neg_s7 = verify_qualification_packet(tdir, c)
        assert res_neg_s7["status"] == "FAIL" and any(
            "rows_forced_terminal_parity" in e or "controls len" in e for e in res_neg_s7["errors"]
        )
        (tdir / "rows-negative" / "rows-parity.json").write_bytes(parity_neg_blob)

        # Negative test CURRENT-5: duplicate G-Pre K16 receipt repeated 4 times must fail
        bad_cr6 = copy.deepcopy(cr_reports["C-R6"])
        bad_cr6["g_pre_k16_receipts"] = [copy.deepcopy(k16_gpre_entries[0]) for _ in range(4)]
        bad_cr6_pin = write_pinned_json(tdir / "c-r-fixtures" / "c-r6/report.json", bad_cr6)
        for entry in valid_cr["controls"]:
            if entry["id"] == "C-R6":
                entry["receipt"] = bad_cr6_pin
        (tdir / "c-r-fixtures" / "c-r-suite-summary.json").write_text(json.dumps(valid_cr))
        res_neg_c5 = verify_qualification_packet(tdir, c)
        assert res_neg_c5["status"] == "FAIL" and any(
            "duplicates a previous FilePin" in e or "role mismatch" in e for e in res_neg_c5["errors"]
        ), f"Expected CURRENT-5 failure on duplicate K16 role receipts, got {res_neg_c5}"
        cr6_pin_restored = write_pinned_json(
            tdir / "c-r-fixtures" / "c-r6/report.json", cr_reports["C-R6"]
        )
        for entry in valid_cr["controls"]:
            if entry["id"] == "C-R6":
                entry["receipt"] = cr6_pin_restored
        (tdir / "c-r-fixtures" / "c-r-suite-summary.json").write_text(json.dumps(valid_cr))

        # Cross-graph timing-companion positive + negative tests (CURRENT-2, CURRENT-5, CURRENT-6)
        companion_rows_img = [*pos_rows_img[:3], "9" * 64]
        companion_c0_a = copy.deepcopy(role_c0_models["k16-A-seed1"])
        companion_c0_a["source_sha256"] = TIMING_COMPANION_SOURCE_SHA256
        companion_c0_r = copy.deepcopy(role_c0_models["k16-R-seed1"])
        companion_c0_r["source_sha256"] = TIMING_COMPANION_SOURCE_SHA256
        companion_dom_b_a = sha256_bytes(native_snapshot_form(companion_c0_a))
        companion_dom_b_r = sha256_bytes(native_snapshot_form(companion_c0_r))

        companion_timing = copy.deepcopy(valid_timing)
        for rec in companion_timing["records"]:
            if rec.get("record_kind") == "cell":
                pol = rec["cell"]["policy"]
                c_dom_b = (
                    companion_dom_b_a
                    if pol == "nearest_address_margin_v1"
                    else companion_dom_b_r
                )
                rec["snapshot_digest_before"] = c_dom_b
                rec["snapshot_digest_after"] = c_dom_b
                rec["identities"]["rows_image_sha256"] = companion_rows_img
                rec["identities"]["compiled"]["source_sha256"] = TIMING_COMPANION_SOURCE_SHA256
        (tdir / "k16-timing-record.json").write_text(json.dumps(companion_timing))

        # Without timing-companion witness, cross-graph timing must fail
        res_neg_comp_missing = verify_qualification_packet(tdir, c)
        assert res_neg_comp_missing["status"] == "FAIL" and any(
            "expected-rows-image.json" in e or "timing-companion" in e or "snapshot_digest_before" in e
            for e in res_neg_comp_missing["errors"]
        ), f"Expected failure when timing-companion witness is missing, got {res_neg_comp_missing}"

        # Write timing-companion-checkpoint0 and timing-companion-metadata-continuation witnesses
        comp_c0_dir = tdir / "timing-companion-checkpoint0"
        comp_meta_dir = tdir / "timing-companion-metadata-continuation"
        comp_c0_dir.mkdir(parents=True, exist_ok=True)
        comp_meta_dir.mkdir(parents=True, exist_ok=True)
        (comp_c0_dir / "expected-rows-image.json").write_text(json.dumps(companion_rows_img))
        (comp_c0_dir / "checkpoint0-seed1-route-k16.snapshot.json").write_bytes(
            native_snapshot_form(companion_c0_a)
        )
        (comp_meta_dir / "companion-identity.stdout.log").write_text(
            json.dumps(
                {
                    "cuda_context_created": False,
                    "training": "HELD",
                    "driver_source_sha256": drv_src,
                    "executable_sha256": "e" * 64,
                    "compiled": {
                        "source_sha256": TIMING_COMPANION_SOURCE_SHA256,
                        "kernel_sha256": pos_kernel_sha,
                        "geometry_sha256": frozen["geometry_sha256"],
                    },
                }
            )
        )
        rlib_rel = "/tmp/target/release/deps/libuor_r4_discrete-1111222233334444.rlib"
        (comp_meta_dir / "library-paths.txt").write_text(f"{rlib_rel}\n")
        (comp_meta_dir / "timing-library-link.txt").write_text(
            f"rustc --extern uor_r4_discrete={rlib_rel}\n"
        )
        (comp_meta_dir / "companion-library-link.txt").write_text(
            f"rustc --extern uor_r4_discrete={rlib_rel}\n"
        )

        res_pos_comp = verify_qualification_packet(tdir, c)
        assert (
            res_pos_comp["status"] == "PASS"
        ), f"Expected PASS with valid timing-companion witness, got {res_pos_comp}"
        assert (
            res_pos_comp["per_graph_identities"]["native"]["source_sha256"]
            == NATIVE_COMPILED_SOURCE_SHA256
            and res_pos_comp["per_graph_identities"]["timing"]["source_sha256"]
            == TIMING_COMPANION_SOURCE_SHA256
            and res_pos_comp["per_graph_identities"]["rows_image_component_equality"]
            == {
                "rows_cu": True,
                "rows_ptx": True,
                "rows_cubin": True,
                "rows_resources_report": False,
            }
        ), f"Expected separate native vs timing source_sha256 and rows_image_component_equality, got {res_pos_comp}"

        # Negative test: mutating shared rlib link proof in companion-library-link.txt must fail
        (comp_meta_dir / "companion-library-link.txt").write_text(
            "rustc --extern uor_r4_discrete=/tmp/target/release/deps/libuor_r4_discrete-different.rlib\n"
        )
        res_neg_rlib = verify_qualification_packet(tdir, c)
        assert res_neg_rlib["status"] == "FAIL" and any(
            "rlib link proof" in e for e in res_neg_rlib["errors"]
        ), f"Expected failure on mismatched shared rlib link proof, got {res_neg_rlib}"
        (comp_meta_dir / "companion-library-link.txt").write_text(
            f"rustc --extern uor_r4_discrete={rlib_rel}\n"
        )

        # Negative test: mutating state outside source_sha256 in companion checkpoint0 must fail
        bad_comp_c0_a = copy.deepcopy(companion_c0_a)
        bad_comp_c0_a["table"][0] = 999
        (comp_c0_dir / "checkpoint0-seed1-route-k16.snapshot.json").write_bytes(
            native_snapshot_form(bad_comp_c0_a)
        )
        res_neg_comp_state = verify_qualification_packet(tdir, c)
        assert res_neg_comp_state["status"] == "FAIL" and any(
            "outside source_sha256" in e or "Domain B" in e
            for e in res_neg_comp_state["errors"]
        ), f"Expected failure on mutated companion checkpoint0 state, got {res_neg_comp_state}"
        (comp_c0_dir / "checkpoint0-seed1-route-k16.snapshot.json").write_bytes(
            native_snapshot_form(companion_c0_a)
        )

        # Negative test: mutating rows_image_sha256[:3] (e.g., rows.cubin [2]) across graphs must fail
        bad_comp_img = [companion_rows_img[0], companion_rows_img[1], "8" * 64, companion_rows_img[3]]
        (comp_c0_dir / "expected-rows-image.json").write_text(json.dumps(bad_comp_img))
        bad_comp_timing = copy.deepcopy(companion_timing)
        for rec in bad_comp_timing["records"]:
            if rec.get("record_kind") == "cell":
                rec["identities"]["rows_image_sha256"] = bad_comp_img
        (tdir / "k16-timing-record.json").write_text(json.dumps(bad_comp_timing))
        res_neg_comp_img3 = verify_qualification_packet(tdir, c)
        assert res_neg_comp_img3["status"] == "FAIL" and any(
            "rows_image_sha256[:3]" in e or "companion-identity" in e
            for e in res_neg_comp_img3["errors"]
        ), f"Expected failure on cross-graph rows_image_sha256[:3] mismatch, got {res_neg_comp_img3}"

    # Verify against actual pod qualification readback packet in qualification-prep/private if present
    priv_dir = CODEX_FASTPATH_DIR / "qualification-prep" / "private"
    if (
        (priv_dir / "finite2670-six-native-reports").is_dir()
        and (priv_dir / "cr2670-seven-native-reports").is_dir()
        and (priv_dir / "gpre2670-metadata-only").is_dir()
        and (priv_dir / "timing2670-native-metadata").is_dir()
        and (priv_dir / "timing-init2670-METADATA-ONLY").is_dir()
        and (priv_dir / "companion2670-metadata-continuation").is_dir()
    ):
        with tempfile.TemporaryDirectory() as tmp_real:
            rdir = pathlib.Path(tmp_real)
            for sub in ("rows-positive", "rows-negative", "qualify-route", "qualify-flock", "qualify-driver", "qualify-relation"):
                src_sub = priv_dir / "finite2670-six-native-reports" / sub
                if src_sub.exists():
                    (rdir / sub).symlink_to(src_sub)
            (rdir / "c-r-fixtures").symlink_to(priv_dir / "cr2670-seven-native-reports")
            (rdir / "gpre2670-metadata-only").symlink_to(priv_dir / "gpre2670-metadata-only")
            (rdir / "timing-companion-measurement").symlink_to(
                priv_dir / "timing2670-native-metadata" / "timing-companion-measurement"
            )
            (rdir / "timing2670-native-metadata").symlink_to(priv_dir / "timing2670-native-metadata")
            (rdir / "timing-init2670-METADATA-ONLY").symlink_to(priv_dir / "timing-init2670-METADATA-ONLY")
            (rdir / "companion2670-metadata-continuation").symlink_to(
                priv_dir / "companion2670-metadata-continuation"
            )
            res_real_pod = verify_qualification_packet(rdir, c)
            assert (
                res_real_pod["status"] == "PASS"
            ), f"Expected PASS on actual pod qualification readback packet, got {res_real_pod}"
            assert (
                res_real_pod["per_graph_identities"]["native"]["source_sha256"]
                == NATIVE_COMPILED_SOURCE_SHA256
                and res_real_pod["per_graph_identities"]["timing"]["source_sha256"]
                == TIMING_COMPANION_SOURCE_SHA256
                and res_real_pod["per_graph_identities"]["native"]["kernel_sha256"]
                == ADMITTED_KERNEL_SHA256
                and res_real_pod["per_graph_identities"]["timing"]["kernel_sha256"]
                == ADMITTED_KERNEL_SHA256
                and res_real_pod["per_graph_identities"]["rows_image_component_equality"]
                == {
                    "rows_cu": True,
                    "rows_ptx": True,
                    "rows_cubin": True,
                    "rows_resources_report": False,
                }
            ), f"Unexpected per_graph_identities on actual pod qualification readback packet: {res_real_pod['per_graph_identities']}"

    print(
        "[4/4] Running outcome decision verifier unit tests across all registered branches "
        "(V3-S3..V3-S6, V3-S8, CURRENT-6, CURRENT-8)..."
    )
    with tempfile.TemporaryDirectory() as tmp:
        tdir = pathlib.Path(tmp)
        frozen = c["frozen_data_and_geometry_bindings"]
        drv_src = c["driver_source_and_arm_bindings_at_fb9e004"]["driver_source_sha256"]
        admitted_src = POSITIVE_ROWS_SOURCE_RS_SHA256
        admitted_krn = "f" * 64
        admitted_exe = "e" * 64
        admitted_rows_img = [POSITIVE_ROWS_CU_SHA256, "d" * 64, "c" * 64, "b" * 64]

        def to_2key_pin(pin: dict[str, Any]) -> dict[str, str]:
            return {"path": str(pin["path"]), "sha256": str(pin["sha256"])}

        def build_bound_arm(
            case_name: str,
            arm: str,
            adm: int,
            top1: int,
            top8: int,
            sw: int,
            occ: int = 180,
            tokens: int = 142870,
            with_fresh: bool = True,
            adm_sw: int = 500,
        ) -> dict[str, Any]:
            seed = 1 if arm.endswith("seed1") else 2
            pol = "nearest_address_margin_v1" if "-A-" in arm else "occupied_cell_rank_hinge_v1"
            cfg = arm_Configs[arm]
            cfg_sha = sha256_bytes(native_driver_config_form(cfg))
            lin = arm_Lineages[arm]
            lin_sha = sha256_bytes(compact_json_bytes(lin))
            epoch = 320 + seed
            scope_str = f"m1-admission-redirect-{arm.lower()}-checkpoint0..16404"

            c0_model: dict[str, Any] = {
                "schema": "uor-r4.discrete-spiralcore-fbs/2",
                "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16},
                "read_spec": {"kind": "route"},
            }
            if pol != "nearest_address_margin_v1":
                c0_model["objective_policy"] = pol
            c0_model.update(
                {
                    "seed": seed,
                    "epoch": 0,
                    "table": [seed] * 32,
                    "routes": [seed] * 8,
                    "gates": [seed] * 8,
                    "operator_kappa": frozen["operator_kappa"],
                    "composition_kappa": frozen["composition_kappa"],
                    "kernel_sha256": admitted_krn,
                    "source_sha256": admitted_src,
                    "geometry_sha256": frozen["geometry_sha256"],
                }
            )
            c0_dom_a = sha256_bytes(canonical_sorted_json_bytes(c0_model))
            c0_dom_b = sha256_bytes(native_snapshot_form(c0_model))
            c0_inner_ckpt = {
                "schema": "uor-r4.discrete-driver-checkpoint/1",
                "lineage": lin,
                "cursor": {
                    "completed_batches": 0,
                    "accepted_updates": 0,
                    "corpus_pass": 0,
                    "next_target": 0,
                    "next_coordinate": 0,
                },
                "model_sha256": c0_dom_a,
                "model": c0_model,
            }
            c0_ckpt_pin = write_pinned_json(
                tdir / case_name / arm / "checkpoint-00000.json", c0_inner_ckpt
            )

            c0_eval = {
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "evaluation_lineage_sha256": lin_sha,
                "model_epoch": 0,
                "model_storage_sha256": c0_dom_a,
                "model_snapshot_sha256": c0_dom_b,
                "evaluation_mutates_model": False,
                "accepted_updates_in_model": 0,
                "updates_applied_by_evaluation": 0,
                "aggregate": {
                    "scored_rows": 14287,
                    "token_eligible_rows": 14287,
                    "target_admitted_rows": 14287,
                    "token_strict_win_eligible_rows": 100,
                },
            }
            c0_pin = write_pinned_json(
                tdir / case_name / arm / "evaluation-00000.json", c0_eval
            )

            exec_id = {
                "base_source": ADMITTED_BASE_SOURCE,
                "reviewed_source_commit": ADMITTED_REVIEWED_SOURCE_COMMIT,
                "source_sha256": admitted_src,
                "kernel_sha256": admitted_krn,
                "geometry_sha256": frozen["geometry_sha256"],
                "executable_sha256": admitted_exe,
                "driver_source_sha256": drv_src,
                "config_sha256": cfg_sha,
                "lineage_sha256": lin_sha,
                "sheet_sha256": ADMITTED_AMENDMENT7_SHEET_SHA256,
            }
            role_qual_obj = {
                "schema": "uor-r4.redirect-qualification/1",
                "scope": scope_str,
                "execution": exec_id,
                "packet_sha256": ADMITTED_REDIRECT_PACKET_SHA256,
                "controls": [
                    {
                        "id": f"C-R{i}",
                        "status": "PASS",
                        "receipt": {"path": f"/workspace/c-r{i}/report.json", "sha256": f"{i}" * 64},
                    }
                    for i in range(1, 7)
                ],
                "cuda_build": {"path": "/workspace/logs/build-native.stderr.log", "sha256": "7" * 64},
                "cuda_build_exit": 0,
                "cuda_clippy": {"path": "/workspace/logs/clippy-native.stderr.log", "sha256": "8" * 64},
                "cuda_clippy_exit": 0,
                "timing": {"path": "/workspace/timing/k16-timing-record.json", "sha256": "9" * 64},
                "initial_checkpoint": to_2key_pin(c0_ckpt_pin),
                "initial_evaluation": to_2key_pin(c0_pin),
            }
            role_qual_pin = write_pinned_json(
                tdir / case_name / arm / f"QUALIFICATION-{arm}.json", role_qual_obj
            )
            role_pm_obj = {
                "reviewer": "Kimi",
                "verdict": "APPROVE_K16_REDIRECT_QUALIFIED_ADMISSION",
                "scope": scope_str,
                "execution": exec_id,
                "qualification_sha256": role_qual_pin["sha256"],
                "packet_sha256": ADMITTED_REDIRECT_PACKET_SHA256,
                "exact_source_review": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254417234",
                "receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254457591",
            }
            role_pm_pin = write_pinned_json(
                tdir / case_name / arm / f"PMREVIEW-{arm}.json", role_pm_obj
            )
            redirect_ev = {
                "packet": {
                    "path": "/workspace/metadata/accepted-packet/SHA256SUMS",
                    "sha256": ADMITTED_REDIRECT_PACKET_SHA256,
                },
                "qualification": to_2key_pin(role_qual_pin),
                "pm_review": to_2key_pin(role_pm_pin),
            }
            plan_obj = {
                "schema": "uor-r4.redirect-owner-release-plan/1",
                "status": "HELD_OWNER_ACTION_REQUIRED",
                "scope": scope_str,
                "execution": exec_id,
                "config": cfg,
                "sheet": {
                    "path": "/workspace/metadata/FIT-sheet.md",
                    "sha256": ADMITTED_AMENDMENT7_SHEET_SHA256,
                },
                "claude_review": to_2key_pin(role_pm_pin),
                "run_state_directory": f"/workspace/state-{arm}",
                "pilot_parent": None,
                "redirect": redirect_ev,
            }
            plan_bytes = native_completion_plan_form(plan_obj)
            role_plan_sha = sha256_bytes(plan_bytes)
            owner_path = tdir / case_name / arm / "owner.json"
            role_owner_obj = {
                "owner": "Casey",
                "decision": "RELEASE_D4_FOR_THIS_K16_REDIRECT_ARM_SEED_ONLY",
                "scope": scope_str,
                "plan_sha256": role_plan_sha,
                "receipt_url": f"owner-terminal:{owner_path}",
            }
            role_owner_pin = write_pinned_json(owner_path, role_owner_obj)
            role_adm_obj = {
                "schema": "uor-r4.redirect-owner-admission/1",
                "plan": json.loads(plan_bytes),
                "owner_release": to_2key_pin(role_owner_pin),
            }
            role_adm_bytes = native_completion_admission_form(role_adm_obj)
            role_adm_sha = sha256_bytes(role_adm_bytes)
            role_adm_path = tdir / case_name / arm / f"ADMISSION-{arm}.json"
            role_adm_path.parent.mkdir(parents=True, exist_ok=True)
            role_adm_path.write_bytes(role_adm_bytes)
            role_adm_pin = {
                "path": str(role_adm_path),
                "sha256": role_adm_sha,
                "bytes": len(role_adm_bytes),
            }

            model: dict[str, Any] = {
                "schema": "uor-r4.discrete-spiralcore-fbs/2",
                "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16},
                "read_spec": {"kind": "route"},
            }
            if pol != "nearest_address_margin_v1":
                model["objective_policy"] = pol
            model.update(
                {
                    "seed": seed,
                    "epoch": epoch,
                    "table": [0] * 32,
                    "routes": [0] * 8,
                    "gates": [0] * 8,
                    "operator_kappa": frozen["operator_kappa"],
                    "composition_kappa": frozen["composition_kappa"],
                    "kernel_sha256": admitted_krn,
                    "source_sha256": admitted_src,
                    "geometry_sha256": frozen["geometry_sha256"],
                }
            )
            dom_a = sha256_bytes(canonical_sorted_json_bytes(model))
            dom_b = sha256_bytes(native_snapshot_form(model))
            cursor = {
                "completed_batches": 16404,
                "accepted_updates": epoch,
                "corpus_pass": 0,
                "next_target": 1049856,
                "next_coordinate": 0,
            }
            inner_ckpt = {
                "schema": "uor-r4.discrete-driver-checkpoint/1",
                "lineage": lin,
                "cursor": cursor,
                "model_sha256": dom_a,
                "model": model,
            }
            dom_c = sha256_bytes(native_driver_checkpoint_form(inner_ckpt))
            comp_ckpt = {
                "schema": "uor-r4.completion-checkpoint/1",
                "admission_sha256": role_adm_sha,
                "pilot_parent": None,
                "checkpoint": inner_ckpt,
                "baseline_evaluation": c0_eval,
                "baseline_snapshot_metadata": {
                    "model_storage_sha256": c0_dom_a,
                    "model_snapshot_sha256": c0_dom_b,
                    "epoch": 0,
                    "source_sha256": admitted_src,
                    "kernel_sha256": admitted_krn,
                    "geometry_sha256": frozen["geometry_sha256"],
                },
                "vector": {
                    "attempted": 16384,
                    "any_candidate_cost_differs_from_identity": 1000,
                    "accepted_updates": epoch - 2,
                },
                "route": {
                    "attempted": 20,
                    "any_candidate_cost_differs_from_identity": 10,
                    "accepted_updates": 2,
                },
            }
            ckpt_pin = write_pinned_json(
                tdir / case_name / arm / "checkpoint-16404.json", comp_ckpt
            )
            snap_pin = write_pinned_json(
                tdir / case_name / arm / "snapshot-16404.json", model
            )

            q4_drv_eval = {
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "evaluation_lineage_sha256": lin_sha,
                "model_epoch": epoch,
                "model_storage_sha256": dom_a,
                "model_snapshot_sha256": dom_b,
                "evaluation_mutates_model": False,
                "accepted_updates_in_model": epoch,
                "updates_applied_by_evaluation": 0,
                "aggregate": {
                    "scored_rows": 14287,
                    "token_eligible_rows": 14287,
                    "target_admitted_rows": adm,
                    "top1_correct": top1,
                    "top8_correct": top8,
                    "token_strict_win_eligible_rows": sw,
                    "admitted_strict_win_rows": adm_sw,
                    "occupied_cells": occ,
                    "admitted_token_count_sum": tokens,
                },
            }
            sens_obj = {
                "vector_block": comp_ckpt["vector"],
                "route": comp_ckpt["route"],
            }
            q4_eval_env = {
                "schema": "uor-r4.completion-evaluation/1",
                "admission_sha256": role_adm_sha,
                "completed_batches": 16404,
                "cursor": cursor,
                "checkpoint": to_2key_pin(ckpt_pin),
                "driver_checkpoint_sha256": dom_c,
                "snapshot_metadata": {
                    "model_storage_sha256": dom_a,
                    "model_snapshot_sha256": dom_b,
                    "epoch": epoch,
                    "source_sha256": admitted_src,
                    "kernel_sha256": admitted_krn,
                    "geometry_sha256": frozen["geometry_sha256"],
                },
                "evaluation": q4_drv_eval,
                "sensitivity": sens_obj,
            }
            q4_eval_pin = write_pinned_json(
                tdir / case_name / arm / "evaluation-16404.json", q4_eval_env
            )

            q_dummy_pins = [
                {"path": f"/workspace/{arm}/evaluation-0{b}.json", "sha256": f"{idx}" * 64}
                for idx, b in enumerate((4101, 8202, 12303), start=3)
            ]
            comp_report_obj = {
                "schema": "uor-r4.completion-report/1",
                "status": "COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION",
                "seed": seed,
                "scope": scope_str,
                "admission_sha256": role_adm_sha,
                "pilot_parent": None,
                "completed_batches": 16404,
                "eligible_train_target_exposures": 1049856,
                "attempt_elapsed_ns": "1000",
                "baseline_snapshot_metadata": comp_ckpt["baseline_snapshot_metadata"],
                "baseline_evaluation": c0_eval,
                "evaluations": [*q_dummy_pins, to_2key_pin(q4_eval_pin)],
                "sensitivity": sens_obj,
                "fresh_transfer": "UNAVAILABLE_NOT_RUN",
                "next_seed": "SEPARATE_OWNER_RELEASE_REQUIRED",
                "m2": "HELD",
            }
            comp_report_pin = write_pinned_json(
                tdir / case_name / arm / "report.json", comp_report_obj
            )

            out_arm: dict[str, Any] = {
                "admission_receipt": role_adm_pin,
                "qualification_receipt": role_qual_pin,
                "pm_review_receipt": role_pm_pin,
                "owner_release_receipt": role_owner_pin,
                "completion_report_receipt": comp_report_pin,
                "q4_checkpoint_receipt": ckpt_pin,
                "q4_model_snapshot_receipt": snap_pin,
                "q4_evaluation_receipt": q4_eval_pin,
                "checkpoint0_evaluation_receipt": c0_pin,
                "checkpoint0_checkpoint_receipt": c0_ckpt_pin,
                "_domain_b": dom_b,
                "_role_adm_sha": role_adm_sha,
                "_role_qual_sha": role_qual_pin["sha256"],
            }
            if with_fresh:
                ft_report = {
                    "schema": "uor-r4.fresh-scoring-report/1",
                    "status": "FRESH_SCORED_EVAL_ONLY",
                    "model_execution": "CUDA_EVALUATION_ONLY",
                    "driver_source_sha256": drv_src,
                    "checkpoint_model_storage_sha256": dom_a,
                    "training_lineage_sha256": lin_sha,
                    "before_typed_snapshot_sha256": dom_b,
                    "after_typed_snapshot_sha256": dom_b,
                    "before_epoch": epoch,
                    "after_epoch": epoch,
                    "evaluation_mutates_model": False,
                    "updates_applied_by_evaluation": 0,
                    "rows_sha256": frozen["fresh_rows_sha256"],
                    "documents_sha256": frozen["fresh_documents_sha256"],
                    "scored_rows_expected": 2000,
                    "frozen_row_inventory": 2000,
                    "fresh_binding": {
                        "source": {
                            "tokens": {"sha256": frozen["fresh_tokens_sha256"]},
                            "mask": {"sha256": frozen["fresh_mask_sha256"]},
                        }
                    },
                    "aggregate": {
                        "scored_rows": 2000,
                        "target_admitted_rows": 800,
                    },
                }
                out_arm["fresh_transfer_receipt"] = write_pinned_json(
                    tdir / case_name / arm / "fresh-report.json", ft_report
                )
            return out_arm

        def build_summary(arms_dict: dict[str, Any]) -> dict[str, Any]:
            return {
                "arms": arms_dict,
            }

        # Case 1: KEEP_K16_R (both R seeds have authenticated fresh transfer receipts)
        c1_dir = tdir / "case1"
        c1_dir.mkdir(parents=True, exist_ok=True)
        summary1 = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case1", "k16-A-seed1", 3500, 1800, 4200, 600, occ=180, with_fresh=False),
                "k16-A-seed2": build_bound_arm("case1", "k16-A-seed2", 3520, 1810, 4220, 610, occ=180, with_fresh=False),
                "k16-R-seed1": build_bound_arm("case1", "k16-R-seed1", 5200, 2050, 4900, 700, occ=150, with_fresh=True),
                "k16-R-seed2": build_bound_arm("case1", "k16-R-seed2", 5220, 2060, 4920, 710, occ=150, with_fresh=True),
            }
        )
        (c1_dir / "trajectories-summary.json").write_text(json.dumps(summary1))
        out1 = evaluate_outcomes(c1_dir, c)
        assert out1["decision"] == "KEEP_K16_R" and out1["status"] == "PASS", f"Got {out1}"
        assert "token_strict_win_eligible_rows" not in out1["delta_noise"], "V3-S6: no invented floor in delta_noise"

        # Case 1b (V3-S5): KEEP_K16_R with only ONE seed having fresh transfer must be PENDING_FRESH_TRANSFER
        c1b_dir = tdir / "case1b"
        c1b_dir.mkdir(parents=True, exist_ok=True)
        summary1b = copy.deepcopy(summary1)
        del summary1b["arms"]["k16-R-seed2"]["fresh_transfer_receipt"]
        (c1b_dir / "trajectories-summary.json").write_text(json.dumps(summary1b))
        out1b = evaluate_outcomes(c1b_dir, c)
        assert (
            out1b["decision"] == "KEEP_K16_R"
            and out1b["status"] == "PENDING_FRESH_TRANSFER"
            and out1b["fresh_transfer_verified"] is False
        ), f"Expected PENDING_FRESH_TRANSFER on missing seed2 fresh receipt, got {out1b}"

        # Case 1c (V3-S6): strict-win regression of -5 (even < 20) blocks R causal advantage
        c1c_dir = tdir / "case1c"
        c1c_dir.mkdir(parents=True, exist_ok=True)
        summary1c = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case1c", "k16-A-seed1", 5000, 1950, 4800, 705, occ=180, with_fresh=True),
                "k16-A-seed2": build_bound_arm("case1c", "k16-A-seed2", 5010, 1955, 4810, 705, occ=180, with_fresh=True),
                "k16-R-seed1": build_bound_arm("case1c", "k16-R-seed1", 5200, 2050, 4900, 700, occ=150, with_fresh=True),
                "k16-R-seed2": build_bound_arm("case1c", "k16-R-seed2", 5220, 2060, 4920, 700, occ=150, with_fresh=True),
            }
        )
        (c1c_dir / "trajectories-summary.json").write_text(json.dumps(summary1c))
        out1c = evaluate_outcomes(c1c_dir, c)
        assert (
            out1c["delta_strict_win"] == -5.0
            and out1c["strict_win_no_regression"] is False
            and out1c["decision"] == "K16_A_KEEP_R_NULL_AT_SCOPE"
        ), f"Expected V3-S6 strict-win regression to block KEEP_K16_R, got {out1c}"

        # Case 2 (V3-S5): GUARD_STRUCTURAL_BIAS_DIAGNOSIS pending vs verified audit receipt
        c2_dir = tdir / "case2"
        c2_dir.mkdir(parents=True, exist_ok=True)
        r1_arm = build_bound_arm("case2", "k16-R-seed1", 5200, 2050, 4900, 400, occ=150, with_fresh=True, adm_sw=380)
        r2_arm = build_bound_arm("case2", "k16-R-seed2", 5220, 2060, 4920, 400, occ=150, with_fresh=True, adm_sw=385)
        summary2 = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case2", "k16-A-seed1", 3500, 1800, 4200, 390, occ=180, with_fresh=False),
                "k16-A-seed2": build_bound_arm("case2", "k16-A-seed2", 3520, 1810, 4220, 390, occ=180, with_fresh=False),
                "k16-R-seed1": r1_arm,
                "k16-R-seed2": r2_arm,
            }
        )
        (c2_dir / "trajectories-summary.json").write_text(json.dumps(summary2))
        out2_pending = evaluate_outcomes(c2_dir, c)
        assert (
            out2_pending["decision"] == "GUARD_STRUCTURAL_BIAS_DIAGNOSIS"
            and out2_pending["status"] == "DIAGNOSE_STRUCTURAL_BIAS_PENDING_AUDIT"
        ), f"Got {out2_pending}"

        audit_pin = write_pinned_json(
            c2_dir / "structural-bias-audit.json",
            {
                "schema": "uor-r4.guard-structural-bias-audit/1",
                "status": "PASS",
                "qualifying_arm": "K16-R",
                "seeds": {
                    "k16-R-seed1": {
                        "model_snapshot_sha256": r1_arm["_domain_b"],
                        "admitted_strict_win_rows": 380,
                        "tie_free_top1_top8_margins_verified": True,
                    },
                    "k16-R-seed2": {
                        "model_snapshot_sha256": r2_arm["_domain_b"],
                        "admitted_strict_win_rows": 385,
                        "tie_free_top1_top8_margins_verified": True,
                    },
                },
            },
        )
        summary2["structural_bias_audit_receipt"] = audit_pin
        (c2_dir / "trajectories-summary.json").write_text(json.dumps(summary2))
        out2_verified = evaluate_outcomes(c2_dir, c)
        assert (
            out2_verified["decision"] == "GUARD_STRUCTURAL_BIAS_DIAGNOSIS"
            and out2_verified["guard_audit_status"] == "DIAGNOSE_STRUCTURAL_BIAS_VERIFIED"
            and out2_verified["status"] == "PASS"
        ), f"Got {out2_verified}"

        # Case 3a: OCCUPANCY_COLLAPSE_TRIGGERED when R collapses and A does NOT pass bars
        c3a_dir = tdir / "case3a"
        c3a_dir.mkdir(parents=True, exist_ok=True)
        summary3a = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case3a", "k16-A-seed1", 3500, 1800, 4200, 600, occ=180, with_fresh=False),
                "k16-A-seed2": build_bound_arm("case3a", "k16-A-seed2", 3520, 1810, 4220, 610, occ=180, with_fresh=False),
                "k16-R-seed1": build_bound_arm("case3a", "k16-R-seed1", 5200, 2050, 4900, 700, occ=80, with_fresh=False),
                "k16-R-seed2": build_bound_arm("case3a", "k16-R-seed2", 5220, 2060, 4920, 710, occ=150, with_fresh=False),
            }
        )
        (c3a_dir / "trajectories-summary.json").write_text(json.dumps(summary3a))
        out3a = evaluate_outcomes(c3a_dir, c)
        assert out3a["decision"] == "OCCUPANCY_COLLAPSE_TRIGGERED" and out3a["status"] == "PASS", f"Got {out3a}"

        # Case 3b (V3-S4): R collapses (occ=80 < 90), but A passes both seeds -> retains K16_A_KEEP_R_NULL_AT_SCOPE (A_KEEP_R_NEGATIVE)
        c3b_dir = tdir / "case3b"
        c3b_dir.mkdir(parents=True, exist_ok=True)
        summary3b = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case3b", "k16-A-seed1", 5200, 2050, 4900, 700, occ=180, with_fresh=True),
                "k16-A-seed2": build_bound_arm("case3b", "k16-A-seed2", 5220, 2060, 4920, 710, occ=180, with_fresh=True),
                "k16-R-seed1": build_bound_arm("case3b", "k16-R-seed1", 5300, 2100, 5000, 720, occ=80, with_fresh=False),
                "k16-R-seed2": build_bound_arm("case3b", "k16-R-seed2", 5320, 2110, 5020, 730, occ=80, with_fresh=False),
            }
        )
        (c3b_dir / "trajectories-summary.json").write_text(json.dumps(summary3b))
        out3b = evaluate_outcomes(c3b_dir, c)
        assert (
            out3b["decision"] == "K16_A_KEEP_R_NULL_AT_SCOPE"
            and out3b["a_keep_r_disposition"] == "A_KEEP_R_NEGATIVE"
            and out3b["r_occupancy_collapse_recorded"] is True
            and out3b["status"] == "PASS"
        ), f"Expected V3-S4 A-KEEP retained when R collapses, got {out3b}"

        # Case 4: K16_A_KEEP_R_NULL_AT_SCOPE (both pass, R within noise of A)
        c4_dir = tdir / "case4"
        c4_dir.mkdir(parents=True, exist_ok=True)
        summary4 = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case4", "k16-A-seed1", 5200, 2050, 4900, 700, occ=180, with_fresh=True),
                "k16-A-seed2": build_bound_arm("case4", "k16-A-seed2", 5220, 2060, 4920, 710, occ=180, with_fresh=True),
                "k16-R-seed1": build_bound_arm("case4", "k16-R-seed1", 5210, 2055, 4905, 705, occ=150, with_fresh=False),
                "k16-R-seed2": build_bound_arm("case4", "k16-R-seed2", 5230, 2065, 4925, 715, occ=150, with_fresh=False),
            }
        )
        (c4_dir / "trajectories-summary.json").write_text(json.dumps(summary4))
        out4 = evaluate_outcomes(c4_dir, c)
        assert (
            out4["decision"] == "K16_A_KEEP_R_NULL_AT_SCOPE"
            and out4["a_keep_r_disposition"] == "A_KEEP_R_NULL_WITHIN_NOISE"
            and out4["status"] == "PASS"
        ), f"Got {out4}"

        # Case 5: RANK_BOTTLENECK_ISOLATED (G-Adm >= 4894 and G-Occ pass, Bar(i) or Bar(ii) fails)
        c5_dir = tdir / "case5"
        c5_dir.mkdir(parents=True, exist_ok=True)
        summary5 = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case5", "k16-A-seed1", 3500, 1500, 4000, 500, occ=180, with_fresh=False),
                "k16-A-seed2": build_bound_arm("case5", "k16-A-seed2", 3520, 1510, 4020, 510, occ=180, with_fresh=False),
                "k16-R-seed1": build_bound_arm("case5", "k16-R-seed1", 5200, 1800, 4600, 600, occ=150, with_fresh=False),
                "k16-R-seed2": build_bound_arm("case5", "k16-R-seed2", 5220, 1810, 4620, 610, occ=150, with_fresh=False),
            }
        )
        (c5_dir / "trajectories-summary.json").write_text(json.dumps(summary5))
        out5 = evaluate_outcomes(c5_dir, c)
        assert out5["decision"] == "RANK_BOTTLENECK_ISOLATED" and out5["status"] == "PASS", f"Got {out5}"

        # Case 6: ADMISSION_STILL_CAPPED_AT_K16 (neither arm reaches 4894)
        c6_dir = tdir / "case6"
        c6_dir.mkdir(parents=True, exist_ok=True)
        summary6 = build_summary(
            {
                "k16-A-seed1": build_bound_arm("case6", "k16-A-seed1", 3500, 1500, 4000, 500, occ=180, with_fresh=False),
                "k16-A-seed2": build_bound_arm("case6", "k16-A-seed2", 3520, 1510, 4020, 510, occ=180, with_fresh=False),
                "k16-R-seed1": build_bound_arm("case6", "k16-R-seed1", 4200, 1700, 4400, 550, occ=150, with_fresh=False),
                "k16-R-seed2": build_bound_arm("case6", "k16-R-seed2", 4220, 1710, 4420, 560, occ=150, with_fresh=False),
            }
        )
        (c6_dir / "trajectories-summary.json").write_text(json.dumps(summary6))
        out6 = evaluate_outcomes(c6_dir, c)
        assert out6["decision"] == "ADMISSION_STILL_CAPPED_AT_K16" and out6["status"] == "PASS", f"Got {out6}"

        # Case 7 (V3-S3 negative test): unbound summary with raw numbers and no receipt FilePins must be refused
        c7_dir = tdir / "case7"
        c7_dir.mkdir(parents=True, exist_ok=True)
        (c7_dir / "trajectories-summary.json").write_text(
            json.dumps(
                {
                    "arms": {
                        arm: {
                            "counterfactual_path": "rows_v1",
                            "completed_batches": 16404,
                            "checkpoint0": {"token_strict_win_eligible_rows": 100, "token_eligible_rows": 14287},
                            "q4_16404": {"target_admitted_rows": 5200, "top1_correct": 2050, "top8_correct": 4900},
                        }
                        for arm in ("k16-A-seed1", "k16-A-seed2", "k16-R-seed1", "k16-R-seed2")
                    }
                }
            )
        )
        out7 = evaluate_outcomes(c7_dir, c)
        assert (
            out7["status"] == "FAIL" and out7["decision"] == "INSTRUMENT_OR_RECEIPT_FAULT"
        ), f"Expected V3-S3 refusal on unbound summary, got {out7}"

        # Case 8 (CURRENT-6 & CURRENT-8 negative + off-host member-manifest tests):
        c8_dir = tdir / "case8"
        c8_dir.mkdir(parents=True, exist_ok=True)

        # 8a: missing per-role admission_receipt must fail
        summary8_no_adm = copy.deepcopy(summary1)
        del summary8_no_adm["arms"]["k16-R-seed1"]["admission_receipt"]
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_no_adm))
        out8_no_adm = evaluate_outcomes(c8_dir, c)
        assert out8_no_adm["status"] == "FAIL" and any(
            "admission_receipt" in e for e in out8_no_adm["errors"]
        ), f"Expected CURRENT-8 failure on missing per-role admission_receipt, got {out8_no_adm}"

        # 8b: admission_sha256 in checkpoint-16404.json set to qualification.sha256 instead of role_admission_sha256 must fail
        summary8_qual_sha = copy.deepcopy(summary1)
        r1_ckpt_obj = json.loads(
            pathlib.Path(
                summary8_qual_sha["arms"]["k16-R-seed1"]["q4_checkpoint_receipt"]["path"]
            ).read_text()
        )
        r1_ckpt_obj["admission_sha256"] = summary8_qual_sha["arms"]["k16-R-seed1"]["_role_qual_sha"]
        bad_ckpt_pin = write_pinned_json(c8_dir / "bad-ckpt-qual-sha.json", r1_ckpt_obj)
        summary8_qual_sha["arms"]["k16-R-seed1"]["q4_checkpoint_receipt"] = bad_ckpt_pin
        r1_eval_obj = json.loads(
            pathlib.Path(
                summary8_qual_sha["arms"]["k16-R-seed1"]["q4_evaluation_receipt"]["path"]
            ).read_text()
        )
        r1_eval_obj["checkpoint"] = to_2key_pin(bad_ckpt_pin)
        bad_eval_pin = write_pinned_json(c8_dir / "bad-eval-qual-sha.json", r1_eval_obj)
        summary8_qual_sha["arms"]["k16-R-seed1"]["q4_evaluation_receipt"] = bad_eval_pin
        r1_rep_obj = json.loads(
            pathlib.Path(
                summary8_qual_sha["arms"]["k16-R-seed1"]["completion_report_receipt"]["path"]
            ).read_text()
        )
        r1_rep_obj["evaluations"][3] = to_2key_pin(bad_eval_pin)
        summary8_qual_sha["arms"]["k16-R-seed1"]["completion_report_receipt"] = write_pinned_json(
            c8_dir / "bad-rep-qual-sha.json", r1_rep_obj
        )
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_qual_sha))
        out8_qual_sha = evaluate_outcomes(c8_dir, c)
        assert out8_qual_sha["status"] == "FAIL" and any(
            "role_admission_sha256" in e for e in out8_qual_sha["errors"]
        ), f"Expected CURRENT-8 failure when admission_sha256 == qualification.sha256, got {out8_qual_sha}"

        # 8c: reusing one role's admission_receipt for another role must fail
        summary8_dup_adm = copy.deepcopy(summary1)
        summary8_dup_adm["arms"]["k16-R-seed2"]["admission_receipt"] = summary8_dup_adm["arms"][
            "k16-R-seed1"
        ]["admission_receipt"]
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_dup_adm))
        out8_dup_adm = evaluate_outcomes(c8_dir, c)
        assert out8_dup_adm["status"] == "FAIL" and any(
            "duplicate per-role admission_sha256=" in e for e in out8_dup_adm["errors"]
        ), f"Expected CURRENT-8 failure on duplicate per-role admission_receipt, got {out8_dup_adm}"

        # 8d: missing completion_report_receipt or invented top-level runtime keys in completion-report/1 must fail
        summary8_no_rep = copy.deepcopy(summary1)
        del summary8_no_rep["arms"]["k16-R-seed1"]["completion_report_receipt"]
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_no_rep))
        out8_no_rep = evaluate_outcomes(c8_dir, c)
        assert out8_no_rep["status"] == "FAIL" and any(
            "completion_report_receipt" in e for e in out8_no_rep["errors"]
        ), f"Expected CURRENT-8 failure on missing completion_report_receipt, got {out8_no_rep}"

        summary8_extra_key = copy.deepcopy(summary1)
        bad_rep_extra = json.loads(
            pathlib.Path(
                summary8_extra_key["arms"]["k16-R-seed1"]["completion_report_receipt"]["path"]
            ).read_text()
        )
        bad_rep_extra["source_sha256"] = admitted_src
        summary8_extra_key["arms"]["k16-R-seed1"]["completion_report_receipt"] = write_pinned_json(
            c8_dir / "bad-rep-extra-key.json", bad_rep_extra
        )
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_extra_key))
        out8_extra_key = evaluate_outcomes(c8_dir, c)
        assert out8_extra_key["status"] == "FAIL" and any(
            "completion_report_receipt key set mismatch:" in e for e in out8_extra_key["errors"]
        ), f"Expected CURRENT-8 failure on invented top-level key in completion-report/1, got {out8_extra_key}"

        # 8e: mismatched checkpoint0 storage/snapshot join must fail
        summary8_bad_c0 = copy.deepcopy(summary1)
        bad_c0_eval = json.loads(
            pathlib.Path(
                summary8_bad_c0["arms"]["k16-R-seed1"]["checkpoint0_evaluation_receipt"]["path"]
            ).read_text()
        )
        bad_c0_eval["model_storage_sha256"] = "9" * 64
        summary8_bad_c0["arms"]["k16-R-seed1"]["checkpoint0_evaluation_receipt"] = write_pinned_json(
            c8_dir / "bad-c0-eval.json", bad_c0_eval
        )
        (c8_dir / "trajectories-summary.json").write_text(json.dumps(summary8_bad_c0))
        out8_bad_c0 = evaluate_outcomes(c8_dir, c)
        assert out8_bad_c0["status"] == "FAIL" and any(
            "checkpoint0_evaluation_receipt" in e and "model_storage_sha256" in e
            for e in out8_bad_c0["errors"]
        ), f"Expected CURRENT-6 failure on mismatched checkpoint0 storage join, got {out8_bad_c0}"

        # 8f: off-host checkpoint-16404.json requires explicit FilePin-authenticated off_host_custody
        # joined to SHARD-INDEX.tsv and FINAL-INDEX-SHARD-CUSTODY-JOIN.json (never unauthenticated auto-discovery)
        c8f_dir = tdir / "case8f"
        c8f_dir.mkdir(parents=True, exist_ok=True)
        summary8f = copy.deepcopy(summary1)
        r1_orig_ckpt_pin = summary1["arms"]["k16-R-seed1"]["q4_checkpoint_receipt"]
        offhost_ckpt_path = "/workspace/offhost-only/k16-R-seed1/checkpoint-16404.json"
        r1_off_eval = json.loads(
            pathlib.Path(
                summary8f["arms"]["k16-R-seed1"]["q4_evaluation_receipt"]["path"]
            ).read_text()
        )
        r1_off_eval["checkpoint"] = {
            "path": offhost_ckpt_path,
            "sha256": r1_orig_ckpt_pin["sha256"],
        }
        r1_off_eval_pin = write_pinned_json(c8f_dir / "eval-16404-offhost.json", r1_off_eval)
        summary8f["arms"]["k16-R-seed1"]["q4_evaluation_receipt"] = r1_off_eval_pin
        r1_off_rep = json.loads(
            pathlib.Path(
                summary8f["arms"]["k16-R-seed1"]["completion_report_receipt"]["path"]
            ).read_text()
        )
        r1_off_rep["evaluations"][3] = to_2key_pin(r1_off_eval_pin)
        summary8f["arms"]["k16-R-seed1"]["completion_report_receipt"] = write_pinned_json(
            c8f_dir / "report-offhost.json", r1_off_rep
        )
        del summary8f["arms"]["k16-R-seed1"]["q4_checkpoint_receipt"]
        del summary8f["arms"]["k16-R-seed1"]["q4_model_snapshot_receipt"]

        manifest_file = c8f_dir / "ORIGINAL-MEMBER-SHA256SUMS"
        manifest_bytes = f"{r1_orig_ckpt_pin['sha256']}  {offhost_ckpt_path.lstrip('/')}\n".encode("utf-8")
        manifest_file.write_bytes(manifest_bytes)
        manifest_sha = sha256_bytes(manifest_bytes)

        # Planted ORIGINAL-MEMBER-SHA256SUMS without explicit off_host_custody FilePins must be rejected
        (c8f_dir / "trajectories-summary.json").write_text(json.dumps(summary8f))
        out8f_unauth = evaluate_outcomes(c8f_dir, c)
        assert out8f_unauth["status"] == "FAIL" and any(
            "q4_checkpoint_receipt" in e for e in out8f_unauth["errors"]
        ), f"Expected failure on unauthenticated planted ORIGINAL-MEMBER-SHA256SUMS, got {out8f_unauth}"

        # Construct FilePin-authenticated SHARD-INDEX.tsv and FINAL-INDEX-SHARD-CUSTODY-JOIN.json
        archive_sha = "1ca211b4975c61e8aaaf21a2b01911d7842f213ae4f1cd0ba1997e41f6fdfe77"
        shard_idx_file = c8f_dir / "SHARD-INDEX.tsv"
        shard_idx_bytes = (
            "group\tarchive\tarchive_sha256\tcompressed_bytes\tregular_members\tmanifest_sha256\tregular_payload_bytes\n"
            f"training-root\tshards/training-root-part-000.tar.gz\t{archive_sha}\t351480539\t1130\t{manifest_sha}\t7387870556\n"
        ).encode("utf-8")
        shard_idx_file.write_bytes(shard_idx_bytes)
        shard_idx_sha = sha256_bytes(shard_idx_bytes)

        cust_join_pin = write_pinned_json(
            c8f_dir / "FINAL-INDEX-SHARD-CUSTODY-JOIN.json",
            {
                "scope": "FINAL_FOUR_ROLE_REMOTE_LOCAL_SNAPSHOT",
                "selection": {
                    "remote_root": "/workspace/codex-cf1-k16-four-role-final-local-seal-2322z",
                    "index_sha256": shard_idx_sha,
                    "transport_manifest_sha256": "6d01b44120f9d4c07db487622fe7eff369aaea08b4a86f40d5ec0b4005b2e7b9",
                    "actual_index_shards": 1,
                    "metadata_fetch_not_payload_transfer": True,
                },
                "shards": [
                    {
                        "id": "training-root-part-000",
                        "archive_sha256": archive_sha,
                        "compressed_bytes": 351480539,
                        "regular_members": 1130,
                        "raw_bytes": 7387870556,
                        "cloud_object": "results/codex/codex-cf1-k16-four-role-data1-training-root-part-000-2324z.tar",
                        "cloud_bytes": 351675392,
                        "cloud_MD5": "948cd0b9bb4eb22bcda3470e001abd00",
                    }
                ],
                "role_interpretation": "NO_SCIENTIFIC_VERDICT",
                "lateOP_cloud": "NOT_IN_THIS_CONTROLLER",
                "safe_to_down": False,
            },
        )
        summary8f["off_host_custody"] = {
            "member_manifest_receipt": {
                "path": str(manifest_file),
                "bytes": len(manifest_bytes),
                "sha256": manifest_sha,
            },
            "shard_index_receipt": {
                "path": str(shard_idx_file),
                "bytes": len(shard_idx_bytes),
                "sha256": shard_idx_sha,
            },
            "custody_join_receipt": cust_join_pin,
        }
        (c8f_dir / "trajectories-summary.json").write_text(json.dumps(summary8f))
        out8f_pass = evaluate_outcomes(c8f_dir, c)
        assert (
            out8f_pass["status"] == "PASS" and out8f_pass["decision"] == "KEEP_K16_R"
        ), f"Expected PASS with FilePin-authenticated off-host custody join, got {out8f_pass}"

        # Valid FilePin on a manifest whose SHA-256 does NOT match SHARD-INDEX.tsv training-root manifest_sha256 must fail
        alt_manifest_file = c8f_dir / "ALT-MEMBER-SHA256SUMS"
        alt_manifest_bytes = (
            f"{r1_orig_ckpt_pin['sha256']}  {offhost_ckpt_path.lstrip('/')}\n"
            f"{'a' * 64}  workspace/extra/dummy.json\n"
        ).encode("utf-8")
        alt_manifest_file.write_bytes(alt_manifest_bytes)
        summary8f_bad_idx = copy.deepcopy(summary8f)
        summary8f_bad_idx["off_host_custody"]["member_manifest_receipt"] = {
            "path": str(alt_manifest_file),
            "bytes": len(alt_manifest_bytes),
            "sha256": sha256_bytes(alt_manifest_bytes),
        }
        (c8f_dir / "trajectories-summary.json").write_text(json.dumps(summary8f_bad_idx))
        out8f_bad_idx = evaluate_outcomes(c8f_dir, c)
        assert out8f_bad_idx["status"] == "FAIL" and any(
            "SHARD-INDEX.tsv training-root manifest_sha256" in e for e in out8f_bad_idx["errors"]
        ), f"Expected failure when member_manifest_receipt SHA != SHARD-INDEX.tsv manifest_sha256, got {out8f_bad_idx}"

        # Tampering with ORIGINAL-MEMBER-SHA256SUMS on disk (breaking FilePin SHA) must fail
        manifest_file.write_text(f"{'0' * 64}  {offhost_ckpt_path.lstrip('/')}\n")
        (c8f_dir / "trajectories-summary.json").write_text(json.dumps(summary8f))
        out8f_fail = evaluate_outcomes(c8f_dir, c)
        assert out8f_fail["status"] == "FAIL" and any(
            "off_host_custody.member_manifest_receipt" in e or "q4_checkpoint_receipt" in e
            for e in out8f_fail["errors"]
        ), f"Expected failure on tampered ORIGINAL-MEMBER-SHA256SUMS, got {out8f_fail}"

    print(
        "SELF_TEST_PASS: contract v1/v2/v3 pins, preserved audit backups, 11-file driver recipe, "
        "12 config/lineage hashes (repo/staged/pod-operator), staged K16 per-role admissions, Q4 Domain-A..D pins, README.md, "
        "and all V3-S1..V3-S8 + CURRENT-1..CURRENT-8 qualification/outcome unit tests passed."
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", type=pathlib.Path, default=None, help="Override contract JSON path")
    parser.add_argument("--self-test", action="store_true", help="Run deterministic host self-tests")
    parser.add_argument("--check-qualification", type=pathlib.Path, default=None, help="Verify qualification directory")
    parser.add_argument(
        "--legacy-4cell-timing",
        action="store_true",
        help="Verify legacy 4-cell timing (including waived sealed_seed1_q4 cells) instead of active 2-cell checkpoint0 timing",
    )
    parser.add_argument("--check-outcomes", type=pathlib.Path, default=None, help="Verify trained trajectories directory")
    args = parser.parse_args()

    if args.self_test:
        return run_self_test()
    contract = load_contract(args.contract)
    if args.check_qualification is not None:
        res = verify_qualification_packet(
            args.check_qualification,
            contract,
            legacy_4cell_timing=args.legacy_4cell_timing,
        )
        print(json.dumps(res, indent=2))
        return 0 if res["status"] == "PASS" else 1
    if args.check_outcomes is not None:
        res = evaluate_outcomes(args.check_outcomes, contract)
        print(json.dumps(res, indent=2))
        return 0 if res["status"] == "PASS" else 1

    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
