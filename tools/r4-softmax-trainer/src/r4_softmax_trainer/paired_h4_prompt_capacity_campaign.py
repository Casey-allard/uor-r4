"""One bounded paired-H4 prompt-conditioning capacity campaign for issue #973.

The predecessor ``R4RetainedLanguagePathV1`` is already qualified.  This
campaign trains exactly one successor from the same initialization, examples,
order, and optimizer schedule while changing only the per-layer exact-H4 token
address.  The independently selected prompt-swap population remains mode-000
until the frozen predecessor and newly fitted successor artifact CIDs exist.

There is deliberately no backend sweep, second fitted control, generation,
lowering, or retry-with-new-hyperparameters path here.  The only resumable work
is the same deterministic CPU-four-thread trajectory.
"""

from __future__ import annotations

import json
import math
import os
import shutil
import statistics
import tempfile
import time
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .language_path_generalization import (
    CONTEXT,
    INITIALIZATION_SEED,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)
from .language_path_generalization_campaign import (
    ADAM_BETA1,
    ADAM_BETA2,
    ADAM_EPSILON,
    BATCH_SIZE,
    CHECKPOINT_INTERVAL,
    GRADIENT_CLIP,
    MAXIMUM_LEARNING_RATE,
    MINIMUM_LEARNING_RATE,
    OPTIMIZER_STEPS,
    PROGRESS_INTERVAL,
    TRAIN_DECISIONS,
    TRAIN_WINDOWS,
    WARMUP_STEPS,
    WEIGHT_DECAY,
    ExecutionPlan,
    _configure_device,
    _exact_geometry,
    _optimizer,
    _optimizer_to_device,
    _ordered_train_batch,
    _peak_rss_bytes,
    _preparation_manifest,
    _sync,
    _train_order_identity,
    _train_step,
    _train_windows,
    _window_batch,
    learning_rate,
)
from .language_path_generalization_data import (
    DATA_MANIFEST_NAME as PREDECESSOR_DATA_MANIFEST_NAME,
)
from .language_path_generalization_data import (
    TOKENIZER_RELATIVE_PATH as PREDECESSOR_TOKENIZER_RELATIVE_PATH,
)
from .language_path_generalization_data import (
    LanguagePathData,
    LanguagePathWindowStore,
    load_language_path_preparation,
)
from .paired_h4_language_path import (
    POLICY as MODEL_POLICY,
)
from .paired_h4_language_path import (
    R4PairedH4LanguagePathV1,
    canonical_layer_token_leaves,
)
from .prompt_conditioning import (
    COMMITMENT_RELATIVE_PATH,
    POPULATION_RELATIVE_PATH,
    REVEAL_RELATIVE_PATH,
    VERDICT_ABSOLUTE_NO_CAPACITY_GAIN,
    VERDICT_FAIL,
    VERDICT_INVALID,
    VERDICT_PARTIAL,
    VERDICT_PASS,
    evaluate_prompt_conditioning,
    load_prompt_conditioning_commitment,
    load_revealed_prompt_conditioning_population,
    reveal_prompt_conditioning_population,
    seal_prompt_conditioning_population,
    select_prompt_conditioning_population_from_source,
)
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
)

ISSUE = 973
POLICY = "R4PairedH4PromptCapacityV1"

PREDECESSOR_POLICY = "R4RetainedLanguagePathV1"
PREDECESSOR_PREPARATION_MANIFEST_CID = (
    "blake3:daef3fc9c7f6ccb3e6c4803140adba547c6a5dfa25abc1d974c4705306e2c207"
)
PREDECESSOR_RESULT_CID = (
    "blake3:cf23d03a8809bb713774704630ef7e90129dd6224856ffd0cd4515554ed5eb95"
)
PREDECESSOR_ARM_RESULT_CID = (
    "blake3:45fe48555ea5f18bfe2e9acc7ba53569101c599fc806513ea5ddc17934565d91"
)
PREDECESSOR_ARTIFACT_CID = (
    "blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d"
)
PREDECESSOR_ARTIFACT_BYTES = 1_010_792
PREDECESSOR_RESULT_RELATIVE_PATH = "run/language-path-result.json"
PREDECESSOR_ARTIFACT_RELATIVE_PATH = "arms/retained/model.safetensors"

EXPECTED_PROMPT_POPULATION_CID = (
    "blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340"
)
EXPECTED_PROMPT_LAST_SOURCE_STORY_ORDINAL = 153_977
EXPECTED_PROMPT_ELIGIBLE_STORIES = 4_200

FRESH_HELDOUT_SOURCE_OFFSET_TOKENS = 155_281_967
FRESH_HELDOUT_TOKENS = 249_986
FRESH_HELDOUT_WINDOWS = 2_066
FRESH_HELDOUT_DECISIONS = FRESH_HELDOUT_WINDOWS * CONTEXT
FRESH_HELDOUT_CID = (
    "blake3:9de081d4a639dfebe885854ce7fbf850fe9a0b0a658307d88e3f9b49d579024d"
)
FRESH_HELDOUT_FIRST_SOURCE_STORY = 760_379
PREDECESSOR_LAST_OVERLAPPING_SOURCE_STORY = 760_378

EXPECTED_SHARED_REPEAT_TOTAL = 1_937_864
EXPECTED_SHARED_REPEAT_MEDIAN = 44.0
EXPECTED_SHARED_COLLISION_FREE = 0
EXPECTED_SHARED_REPEAT_MAXIMUM = 60
EXPECTED_PAIRED_REPEAT_TOTAL = 47_522
EXPECTED_PAIRED_REPEAT_MEDIAN = 1.0
EXPECTED_PAIRED_REPEAT_P95 = 3
EXPECTED_PAIRED_COLLISION_FREE = 11_148
EXPECTED_PAIRED_REPEAT_MAXIMUM = 13

PROBE_STEPS = 5
PROJECTION_SAFETY_FACTOR = 1.25
PROJECTION_CEILING_SECONDS = 3_000.0
HARD_WALL_CEILING_SECONDS = 3_600.0
MEMORY_FRACTION_CEILING = 0.80
PROMPT_EVALUATION_ESTIMATE_SECONDS = 55.0
PROMPT_EVALUATION_CEILING_SECONDS = 300.0

REQUIRED_NLL_IMPROVEMENT = 1.0
REQUIRED_TOP1_POINT_IMPROVEMENT = 5.0
REQUIRED_FINAL_NLL_CEILING = 4.0
PREDECESSOR_NLL_TOLERANCE = 0.05
PREDECESSOR_TOP1_POINT_TOLERANCE = 1.0

CPU_PLAN = ExecutionPlan(
    name="cpu-accelerate-4t-single-candidate",
    backend="cpu",
    threads_per_worker=4,
    workers=1,
    concurrent_arms=False,
)

PREPARATION_RELATIVE_PATH = "paired-h4-prompt-capacity-preparation.json"
HELDOUT_RELATIVE_PATH = "data/fresh-heldout.u16"
PROBE_RELATIVE_PATH = "preflight/paired-h4-prompt-capacity-probe.json"
STARTED_RELATIVE_PATH = "run/paired-h4-prompt-capacity-started.json"
RESULT_RELATIVE_PATH = "run/paired-h4-prompt-capacity-result.json"
CHECKPOINT_RELATIVE_PATH = "candidate/checkpoint.pt"
CHECKPOINT_CID_RELATIVE_PATH = "candidate/checkpoint.pt.cid.json"
PROGRESS_RELATIVE_PATH = "candidate/progress.json"
CANDIDATE_ARTIFACT_RELATIVE_PATH = "candidate/model.safetensors"

PREPARATION_SCHEMA = "uor-r4.paired-h4-prompt-capacity-preparation/1"
PROBE_SCHEMA = "uor-r4.paired-h4-prompt-capacity-probe/1"
STARTED_SCHEMA = "uor-r4.paired-h4-prompt-capacity-started/1"
CHECKPOINT_SCHEMA = "uor-r4.paired-h4-prompt-capacity-checkpoint/1"
RESULT_SCHEMA = "uor-r4.paired-h4-prompt-capacity-result/1"

TERMINAL_PASS = "PAIRED_H4_PROMPT_CAPACITY_PASS"
TERMINAL_GENERAL_LANGUAGE_REGRESSION = "PAIRED_H4_GENERAL_LANGUAGE_REGRESSION"
TERMINAL_ABSOLUTE_NO_CAPACITY_GAIN = "PAIRED_H4_ABSOLUTE_NO_CAPACITY_GAIN"
TERMINAL_PARTIAL = "PAIRED_H4_PROMPT_CAPACITY_PARTIAL"
TERMINAL_FAIL = "PAIRED_H4_PROMPT_CAPACITY_FAIL"
TERMINAL_INVALID = "INVALID_PAIRED_H4_PROMPT_CAPACITY"
TERMINAL_UNAVAILABLE = "UNAVAILABLE_PAIRED_H4_PROMPT_CAPACITY"


@dataclass(frozen=True, slots=True)
class CampaignPreparation:
    """Verified inputs without opening the sealed prompt population."""

    root: Path
    manifest: dict[str, Any]
    predecessor: LanguagePathData
    fresh_heldout: LanguagePathWindowStore
    predecessor_artifact_path: Path
    commitment: dict[str, Any]


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    observed = value.get(field)
    unsigned = dict(value)
    unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_canonical_json(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"expected a regular non-symlink JSON file: {path}")
    payload = path.read_bytes()
    try:
        value = json.loads(payload.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot decode canonical JSON: {path}") from error
    if not isinstance(value, dict) or canonical_json_bytes(value) != payload:
        raise ValueError(f"JSON file is not canonical: {path}")
    return value


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(canonical_json_bytes(value))
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def _read_u16_slice(path: Path, *, offset_tokens: int, token_count: int) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError("fresh-heldout source must be a regular non-symlink file")
    if offset_tokens < 0 or token_count < 1:
        raise ValueError("fresh-heldout slice coordinates are invalid")
    byte_offset = offset_tokens * 2
    byte_count = token_count * 2
    if byte_offset + byte_count > path.stat().st_size:
        raise ValueError("fresh-heldout slice crosses its source store")
    with path.open("rb") as source:
        source.seek(byte_offset)
        payload = source.read(byte_count)
    if len(payload) != byte_count:
        raise ValueError("fresh-heldout source ended inside the frozen slice")
    return payload


def _subgroup_size(
    support: Sequence[int], left_actions: Tensor, *, identity_index: int
) -> int:
    """Return closure size under the exact multiplication table."""

    known = {identity_index}
    generators = {int(value) for value in support}
    frontier = [identity_index]
    while frontier:
        left = frontier.pop()
        for right in generators | known:
            for product in (
                int(left_actions[left, right]),
                int(left_actions[right, left]),
            ):
                if product not in known:
                    known.add(product)
                    frontier.append(product)
    return len(known)


def _route_repeats(
    token_ids: Tensor,
    *,
    layer_token_leaves: Tensor,
    left_actions: Tensor,
    identity_index: int,
) -> tuple[int, ...]:
    """Count repeated cumulative joint addresses without reading targets.

    For a left-regular action, composing ``old[new[h]]`` advances the group
    element as ``old_element * new_element``.  Tracking that element is exactly
    equivalent to carrying all 120 permutation columns and is much cheaper.
    """

    if token_ids.ndim != 2 or token_ids.dtype != torch.long:
        raise ValueError("route census tokens must be int64 [batch,time]")
    if tuple(layer_token_leaves.shape) != (2, VOCAB_SIZE):
        raise ValueError("route census requires two complete token codebooks")
    batch, time_steps = token_ids.shape
    addresses = torch.full(
        (2, batch), identity_index, dtype=torch.long, device=token_ids.device
    )
    encoded: list[Tensor] = []
    for position in range(time_steps):
        token = token_ids[:, position]
        for layer in range(2):
            leaf = layer_token_leaves[layer].index_select(0, token)
            addresses[layer] = left_actions[addresses[layer], leaf]
        encoded.append(addresses[0] * 120 + addresses[1])
    routes = torch.stack(encoded, dim=1)
    return tuple(
        time_steps - int(torch.unique(routes[row]).numel()) for row in range(batch)
    )


def _repeat_summary(repeats: Sequence[int]) -> dict[str, Any]:
    if len(repeats) != TRAIN_WINDOWS:
        raise ValueError("route census did not cover the frozen training population")
    ordered = sorted(int(value) for value in repeats)
    return {
        "windows": len(ordered),
        "repeated_joint_addresses": sum(ordered),
        "mean_repeated_joint_addresses": sum(ordered) / len(ordered),
        "median_repeated_joint_addresses": statistics.median(ordered),
        "p95_repeated_joint_addresses": ordered[math.ceil(0.95 * len(ordered)) - 1],
        "maximum_repeated_joint_addresses": ordered[-1],
        "collision_free_windows": sum(value == 0 for value in ordered),
    }


def structural_collision_census(predecessor: LanguagePathData) -> dict[str, Any]:
    """Reproduce the construction-only representation admission evidence."""

    geometry = _exact_geometry(predecessor)
    train = _train_windows(predecessor)
    paired_leaves = canonical_layer_token_leaves(
        identity_index=geometry.identity_offset
    )
    shared_leaves = torch.stack((geometry.token_leaves, geometry.token_leaves))
    shared_repeats: list[int] = []
    paired_repeats: list[int] = []
    for start in range(0, TRAIN_WINDOWS, 512):
        count = min(512, TRAIN_WINDOWS - start)
        inputs = _window_batch(train, start, count, torch.device("cpu"))[:, :-1]
        shared_repeats.extend(
            _route_repeats(
                inputs,
                layer_token_leaves=shared_leaves,
                left_actions=geometry.left_actions,
                identity_index=geometry.identity_offset,
            )
        )
        paired_repeats.extend(
            _route_repeats(
                inputs,
                layer_token_leaves=paired_leaves,
                left_actions=geometry.left_actions,
                identity_index=geometry.identity_offset,
            )
        )
    shared = _repeat_summary(shared_repeats)
    paired = _repeat_summary(paired_repeats)
    expected_shared = {
        "repeated_joint_addresses": EXPECTED_SHARED_REPEAT_TOTAL,
        "median_repeated_joint_addresses": EXPECTED_SHARED_REPEAT_MEDIAN,
        "maximum_repeated_joint_addresses": EXPECTED_SHARED_REPEAT_MAXIMUM,
        "collision_free_windows": EXPECTED_SHARED_COLLISION_FREE,
    }
    expected_paired = {
        "repeated_joint_addresses": EXPECTED_PAIRED_REPEAT_TOTAL,
        "median_repeated_joint_addresses": EXPECTED_PAIRED_REPEAT_MEDIAN,
        "p95_repeated_joint_addresses": EXPECTED_PAIRED_REPEAT_P95,
        "maximum_repeated_joint_addresses": EXPECTED_PAIRED_REPEAT_MAXIMUM,
        "collision_free_windows": EXPECTED_PAIRED_COLLISION_FREE,
    }
    if any(shared[field] != value for field, value in expected_shared.items()):
        raise ValueError("shared-coordinate collision census differs from the freeze")
    if any(paired[field] != value for field, value in expected_paired.items()):
        raise ValueError("paired-coordinate collision census differs from the freeze")
    pair_ids = paired_leaves[0] * 120 + paired_leaves[1]
    first_support = sorted({int(value) for value in paired_leaves[0].tolist()})
    second_support = sorted({int(value) for value in paired_leaves[1].tolist()})
    support = {
        "unique_token_pairs": int(torch.unique(pair_ids).numel()),
        "first_coordinate_support": len(first_support),
        "second_coordinate_support": len(second_support),
        "first_coordinate_generated_subgroup": _subgroup_size(
            first_support,
            geometry.left_actions,
            identity_index=geometry.identity_offset,
        ),
        "second_coordinate_generated_subgroup": _subgroup_size(
            second_support,
            geometry.left_actions,
            identity_index=geometry.identity_offset,
        ),
    }
    if support != {
        "unique_token_pairs": VOCAB_SIZE,
        "first_coordinate_support": 120,
        "second_coordinate_support": 36,
        "first_coordinate_generated_subgroup": 120,
        "second_coordinate_generated_subgroup": 120,
    }:
        raise ValueError("paired-H4 codebook support differs from the freeze")
    return {
        "role": "CONSTRUCTION_ONLY_NO_TARGET_OR_MODEL_READS",
        "shared_single_coordinate": shared,
        "paired_coordinates": paired,
        "support": support,
        "repeat_reduction_fraction": 1.0
        - paired["repeated_joint_addresses"] / shared["repeated_joint_addresses"],
    }


def _verify_predecessor(predecessor_root: Path) -> tuple[LanguagePathData, Path]:
    predecessor = load_language_path_preparation(predecessor_root)
    if (
        _preparation_manifest(predecessor).get("manifest_cid")
        != PREDECESSOR_PREPARATION_MANIFEST_CID
    ):
        raise ValueError("predecessor preparation differs from the qualified V1 freeze")
    result = _read_canonical_json(predecessor_root / PREDECESSOR_RESULT_RELATIVE_PATH)
    _verify_self_cid(result, "result_cid")
    retained = result.get("arms", {}).get("retained", {})
    artifact_record = (
        retained.get("artifact", {}) if isinstance(retained, Mapping) else {}
    )
    if (
        result.get("policy") != PREDECESSOR_POLICY
        or result.get("result_cid") != PREDECESSOR_RESULT_CID
        or result.get("verdict") != "RETAINED_LANGUAGE_PATH_PASS"
        or retained.get("arm_result_cid") != PREDECESSOR_ARM_RESULT_CID
        or artifact_record
        != {
            "path": PREDECESSOR_ARTIFACT_RELATIVE_PATH,
            "bytes": PREDECESSOR_ARTIFACT_BYTES,
            "cid": PREDECESSOR_ARTIFACT_CID,
        }
    ):
        raise ValueError("predecessor result differs from the qualified V1 freeze")
    artifact = predecessor_root / PREDECESSOR_ARTIFACT_RELATIVE_PATH
    if (
        artifact.is_symlink()
        or not artifact.is_file()
        or artifact.stat().st_size != PREDECESSOR_ARTIFACT_BYTES
        or cid_file(artifact) != PREDECESSOR_ARTIFACT_CID
    ):
        raise ValueError("qualified predecessor artifact does not reproduce")
    return predecessor, artifact.resolve()


def _cleanup_staging(staging: Path) -> None:
    sealed = staging / POPULATION_RELATIVE_PATH
    if sealed.parent.exists() and not sealed.parent.is_symlink():
        sealed.parent.chmod(0o700)
    if staging.exists() and not staging.is_symlink():
        shutil.rmtree(staging)


def prepare_paired_h4_prompt_capacity(
    *,
    root: Path,
    predecessor_root: Path,
    source_train_path: Path,
    raw_source_path: Path,
) -> CampaignPreparation:
    """Create the heldout slice and seal the prompt population exactly once."""

    root = root.resolve()
    predecessor_root = predecessor_root.resolve()
    if root.exists() or root.is_symlink():
        raise FileExistsError("paired-H4 prompt-capacity root is create-once")
    predecessor, artifact = _verify_predecessor(predecessor_root)
    if FRESH_HELDOUT_TOKENS != FRESH_HELDOUT_WINDOWS * (CONTEXT + 1):
        raise RuntimeError("fresh-heldout arithmetic differs from the freeze")
    heldout_payload = _read_u16_slice(
        source_train_path,
        offset_tokens=FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
        token_count=FRESH_HELDOUT_TOKENS,
    )
    if cid_bytes(heldout_payload) != FRESH_HELDOUT_CID:
        raise ValueError("fresh-heldout slice CID differs from the independent freeze")
    tokenizer_path = predecessor_root / PREDECESSOR_TOKENIZER_RELATIVE_PATH
    population = select_prompt_conditioning_population_from_source(
        raw_source_path,
        tokenizer_path,
    )
    if (
        population.population_cid != EXPECTED_PROMPT_POPULATION_CID
        or population.last_source_story_ordinal
        != EXPECTED_PROMPT_LAST_SOURCE_STORY_ORDINAL
        or population.eligible_stories_examined != EXPECTED_PROMPT_ELIGIBLE_STORIES
    ):
        raise ValueError(
            "prompt-conditioning population differs from the independent freeze: "
            f"cid={population.population_cid}, "
            f"last_source_story_ordinal={population.last_source_story_ordinal}, "
            f"eligible_stories_examined={population.eligible_stories_examined}"
        )
    collision_census = structural_collision_census(predecessor)
    implementation = trainer_implementation_contract()

    root.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{root.name}.preparing-", dir=root.parent))
    try:
        atomic_write(staging / HELDOUT_RELATIVE_PATH, heldout_payload)
        commitment = seal_prompt_conditioning_population(staging, population)
        body = {
            "schema": PREPARATION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "implementation": implementation,
            "predecessor": {
                "root": str(predecessor_root),
                "preparation_manifest": PREDECESSOR_DATA_MANIFEST_NAME,
                "preparation_manifest_cid": PREDECESSOR_PREPARATION_MANIFEST_CID,
                "result_cid": PREDECESSOR_RESULT_CID,
                "retained_arm_result_cid": PREDECESSOR_ARM_RESULT_CID,
                "artifact": {
                    "path": str(artifact),
                    "bytes": PREDECESSOR_ARTIFACT_BYTES,
                    "cid": PREDECESSOR_ARTIFACT_CID,
                },
            },
            "training": {
                "reuse": "exact predecessor train slice and deterministic order",
                "windows": TRAIN_WINDOWS,
                "decisions": TRAIN_DECISIONS,
                "optimizer_steps": OPTIMIZER_STEPS,
                "seed": INITIALIZATION_SEED,
                "changed_variable": "LAYER_PAIRED_EXACT_H4_TOKEN_ADDRESS_ONLY",
            },
            "fresh_heldout": {
                "path": HELDOUT_RELATIVE_PATH,
                "source_path": str(source_train_path.resolve()),
                "source_offset_tokens": FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
                "tokens": FRESH_HELDOUT_TOKENS,
                "windows": FRESH_HELDOUT_WINDOWS,
                "decisions": FRESH_HELDOUT_DECISIONS,
                "bytes": len(heldout_payload),
                "cid": FRESH_HELDOUT_CID,
                "first_source_story": FRESH_HELDOUT_FIRST_SOURCE_STORY,
                "predecessor_last_overlapping_source_story": (
                    PREDECESSOR_LAST_OVERLAPPING_SOURCE_STORY
                ),
            },
            "prompt_population": {
                "commitment_cid": commitment["commitment_cid"],
                "population_cid": EXPECTED_PROMPT_POPULATION_CID,
                "access": "SEALED_MODE_000",
                "reveal_after": "predecessor and candidate artifact CIDs are fixed",
            },
            "construction_admission": collision_census,
            "scope": {
                "trained_arms": ["paired_h4_candidate"],
                "predecessor_retrained": False,
                "generation": "NOT_RUN",
                "reasoning": "NOT_RUN",
                "lowering": "NOT_RUN",
                "h4_specific_superiority": "NOT_EVALUATED",
                "cuda": "FORBIDDEN",
            },
        }
        manifest = _with_cid(body, "preparation_cid")
        _write_exclusive_json(staging / PREPARATION_RELATIVE_PATH, manifest)
        if root.exists() or root.is_symlink():
            raise FileExistsError("paired-H4 root appeared during preparation")
        staging.rename(root)
    except BaseException:
        _cleanup_staging(staging)
        raise
    return load_paired_h4_prompt_capacity_preparation(root)


def _load_commitment_in_any_state(root: Path) -> dict[str, Any]:
    if (root / REVEAL_RELATIVE_PATH).exists():
        load_revealed_prompt_conditioning_population(root)
        commitment = _read_canonical_json(root / COMMITMENT_RELATIVE_PATH)
        _verify_self_cid(commitment, "commitment_cid")
        return commitment
    return load_prompt_conditioning_commitment(root)


def load_paired_h4_prompt_capacity_preparation(root: Path) -> CampaignPreparation:
    """Verify prepared inputs while respecting sealed/revealed population state."""

    if root.is_symlink() or not root.is_dir():
        raise ValueError("paired-H4 prompt-capacity root must be a regular directory")
    root = root.resolve()
    manifest = _read_canonical_json(root / PREPARATION_RELATIVE_PATH)
    _verify_self_cid(manifest, "preparation_cid")
    if (
        manifest.get("schema") != PREPARATION_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("policy") != POLICY
        or manifest.get("model_policy") != MODEL_POLICY
    ):
        raise ValueError("paired-H4 preparation envelope differs")
    predecessor_record = manifest.get("predecessor")
    heldout_record = manifest.get("fresh_heldout")
    prompt_record = manifest.get("prompt_population")
    if not all(
        isinstance(value, Mapping)
        for value in (predecessor_record, heldout_record, prompt_record)
    ):
        raise ValueError("paired-H4 preparation records are malformed")
    predecessor_root = Path(str(predecessor_record["root"]))
    predecessor, artifact = _verify_predecessor(predecessor_root)
    if (
        predecessor_record.get("preparation_manifest_cid")
        != PREDECESSOR_PREPARATION_MANIFEST_CID
        or predecessor_record.get("result_cid") != PREDECESSOR_RESULT_CID
        or predecessor_record.get("retained_arm_result_cid")
        != PREDECESSOR_ARM_RESULT_CID
        or predecessor_record.get("artifact")
        != {
            "path": str(artifact),
            "bytes": PREDECESSOR_ARTIFACT_BYTES,
            "cid": PREDECESSOR_ARTIFACT_CID,
        }
    ):
        raise ValueError("paired-H4 predecessor binding differs")
    heldout_path = root / HELDOUT_RELATIVE_PATH
    if (
        heldout_path.is_symlink()
        or not heldout_path.is_file()
        or heldout_path.stat().st_size != FRESH_HELDOUT_TOKENS * 2
        or cid_file(heldout_path) != FRESH_HELDOUT_CID
        or heldout_record.get("path") != HELDOUT_RELATIVE_PATH
        or heldout_record.get("cid") != FRESH_HELDOUT_CID
        or heldout_record.get("windows") != FRESH_HELDOUT_WINDOWS
    ):
        raise ValueError("paired-H4 fresh-heldout artifact differs")
    commitment = _load_commitment_in_any_state(root)
    if (
        prompt_record.get("commitment_cid") != commitment.get("commitment_cid")
        or prompt_record.get("population_cid") != EXPECTED_PROMPT_POPULATION_CID
        or commitment.get("population", {}).get("cid") != EXPECTED_PROMPT_POPULATION_CID
    ):
        raise ValueError("paired-H4 prompt commitment differs")
    census = manifest.get("construction_admission", {})
    if (
        census.get("shared_single_coordinate", {}).get("repeated_joint_addresses")
        != EXPECTED_SHARED_REPEAT_TOTAL
        or census.get("paired_coordinates", {}).get("repeated_joint_addresses")
        != EXPECTED_PAIRED_REPEAT_TOTAL
        or census.get("paired_coordinates", {}).get("collision_free_windows")
        != EXPECTED_PAIRED_COLLISION_FREE
    ):
        raise ValueError("paired-H4 construction admission differs")
    return CampaignPreparation(
        root=root,
        manifest=manifest,
        predecessor=predecessor,
        fresh_heldout=LanguagePathWindowStore(
            heldout_path,
            window_count=FRESH_HELDOUT_WINDOWS,
        ),
        predecessor_artifact_path=artifact,
        commitment=commitment,
    )


def _require_implementation(manifest: Mapping[str, Any]) -> dict[str, Any]:
    current = trainer_implementation_contract()
    if manifest.get("implementation") != current:
        raise ValueError("trainer implementation differs from the preparation freeze")
    return current


def project_candidate_execution(
    *,
    mean_train_step_seconds: float,
    evaluation_batch_seconds: float,
    checkpoint_seconds: float,
    artifact_seconds: float,
    replay_seconds: float,
    peak_memory_bytes: int,
    memory_budget_bytes: int,
) -> dict[str, Any]:
    """Project the one candidate run under the predeclared conservative budget."""

    values = (
        mean_train_step_seconds,
        evaluation_batch_seconds,
        checkpoint_seconds,
        artifact_seconds,
        replay_seconds,
    )
    if any(not math.isfinite(value) or value < 0.0 for value in values):
        raise ValueError("execution timings must be finite and nonnegative")
    if peak_memory_bytes < 0 or memory_budget_bytes <= 0:
        raise ValueError("execution memory values are invalid")
    evaluation_batches = math.ceil(FRESH_HELDOUT_WINDOWS / BATCH_SIZE)
    checkpoint_writes = 1 + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL)
    raw_seconds = (
        mean_train_step_seconds * OPTIMIZER_STEPS
        + evaluation_batch_seconds * evaluation_batches * 3
        + checkpoint_seconds * checkpoint_writes
        + artifact_seconds
        + replay_seconds
        + PROMPT_EVALUATION_ESTIMATE_SECONDS
    )
    projected = raw_seconds * PROJECTION_SAFETY_FACTOR
    memory_fraction = peak_memory_bytes / memory_budget_bytes
    eligible = bool(
        projected <= PROJECTION_CEILING_SECONDS
        and memory_fraction <= MEMORY_FRACTION_CEILING
    )
    return {
        "eligible": eligible,
        "raw_seconds": raw_seconds,
        "safety_factor": PROJECTION_SAFETY_FACTOR,
        "projected_seconds": projected,
        "projection_ceiling_seconds": PROJECTION_CEILING_SECONDS,
        "hard_wall_ceiling_seconds": HARD_WALL_CEILING_SECONDS,
        "evaluation_batches": evaluation_batches,
        "general_language_evaluations": 3,
        "checkpoint_writes": checkpoint_writes,
        "prompt_evaluation_estimate_seconds": PROMPT_EVALUATION_ESTIMATE_SECONDS,
        "prompt_evaluation_ceiling_seconds": PROMPT_EVALUATION_CEILING_SECONDS,
        "peak_memory_bytes": peak_memory_bytes,
        "memory_budget_bytes": memory_budget_bytes,
        "memory_fraction": memory_fraction,
        "memory_fraction_ceiling": MEMORY_FRACTION_CEILING,
        "reason": (
            "ELIGIBLE"
            if eligible
            else "WALL_PROJECTION"
            if projected > PROJECTION_CEILING_SECONDS
            else "MEMORY"
        ),
    }


def _maximum_state_delta(left: Any, right: Any) -> float:
    return max(
        float((left.keys - right.keys).abs().max()),
        float((left.values - right.values).abs().max()),
        float((left.occupied != right.occupied).any()),
    )


def _initialization_identity(geometry: Any) -> dict[str, Any]:
    predecessor = R4RetainedLanguagePathV1(geometry)
    candidate = R4PairedH4LanguagePathV1(geometry)
    left = dict(predecessor.named_parameters())
    right = dict(candidate.named_parameters())
    if set(left) != set(right):
        raise RuntimeError("paired-H4 candidate changed learned parameter names")
    digest = blake3()
    for name in sorted(left):
        if not torch.equal(left[name], right[name]):
            raise RuntimeError(f"paired-H4 initialization differs at {name}")
        digest.update(name.encode("utf-8"))
        digest.update(left[name].detach().cpu().contiguous().numpy().tobytes())
    return {
        "seed": INITIALIZATION_SEED,
        "learned_parameters_byte_identical": True,
        "learned_initialization_cid": f"blake3:{digest.hexdigest()}",
        "parameters": candidate.parameter_count(),
        "state_values": candidate.state_value_count(),
        "state_bytes_f32": STATE_BYTES_F32,
        "validity_bits": candidate.validity_bit_count(),
    }


def _mechanical_admission(preparation: CampaignPreparation) -> dict[str, Any]:
    geometry = _exact_geometry(preparation.predecessor)
    candidate = R4PairedH4LanguagePathV1(geometry)
    if (
        candidate.parameter_count() != PARAMETER_COUNT
        or candidate.state_value_count() != STATE_VALUES
        or candidate.validity_bit_count() != VALIDITY_BITS
        or candidate.output_weight.data_ptr()
        != candidate.token_embedding.weight.data_ptr()
    ):
        raise RuntimeError(
            "paired-H4 candidate changed the qualified architecture ledger"
        )
    batch = _window_batch(
        _train_windows(preparation.predecessor), 0, 2, torch.device("cpu")
    )
    candidate.zero_grad(set_to_none=True)
    output = candidate(batch[:, :-1], batch[:, 1:])
    if output.loss is None:
        raise RuntimeError("paired-H4 admission produced no training loss")
    output.loss.backward()
    inactive = [
        name
        for name, parameter in candidate.named_parameters()
        if parameter.grad is None
        or not bool(torch.isfinite(parameter.grad).all())
        or not bool((parameter.grad != 0).any())
    ]
    if inactive:
        raise RuntimeError(f"paired-H4 admission has inactive gradients: {inactive}")
    original = batch[:1, :-1]
    altered = original.clone()
    altered[:, 61:] = (altered[:, 61:] + 1) % VOCAB_SIZE
    with torch.no_grad():
        shared = candidate(original).logits[:, :61]
        changed = candidate(altered).logits[:, :61]
        stationary = candidate(original, implementation="stationary")
        direct = candidate(original, implementation="direct")
    causal_delta = float((shared - changed).abs().max())
    parity_logits = float((stationary.logits - direct.logits).abs().max())
    parity_state = _maximum_state_delta(stationary.final_state, direct.final_state)
    if causal_delta != 0.0 or parity_logits > 2e-5 or parity_state > 2e-5:
        raise RuntimeError("paired-H4 causal or stationary/direct admission failed")
    artifact = candidate.export_learned_artifact()
    replay = R4PairedH4LanguagePathV1(geometry)
    replay.load_learned_artifact(artifact)
    with torch.no_grad():
        replay_logits = replay(original).logits
    replay_delta = float((stationary.logits - replay_logits).abs().max())
    forbidden_reads = sum(
        int(getattr(value.audit, "forbidden_reads", -1))
        for value in (stationary, direct)
    )
    if replay_delta != 0.0 or forbidden_reads != 0:
        raise RuntimeError(
            "paired-H4 artifact replay or forbidden-read admission failed"
        )
    return {
        "passed": True,
        "initialization": _initialization_identity(geometry),
        "finite_nonzero_gradient_parameters": len(list(candidate.parameters())),
        "causal_shared_prefix_maximum_logits_delta": causal_delta,
        "stationary_direct_maximum_logits_delta": parity_logits,
        "stationary_direct_maximum_state_delta": parity_state,
        "artifact_reload_maximum_logits_delta": replay_delta,
        "forbidden_reads": forbidden_reads,
    }


def _execute_probe(preparation: CampaignPreparation) -> dict[str, Any]:
    device, backend = _configure_device(CPU_PLAN)
    if device.type != "cpu":
        raise RuntimeError("paired-H4 probe selected a non-CPU device")
    mechanics = _mechanical_admission(preparation)
    geometry = _exact_geometry(preparation.predecessor)
    candidate = R4PairedH4LanguagePathV1(geometry).to(device)
    optimizer = _optimizer(candidate)
    measured: list[float] = []
    losses: list[float] = []
    gradient_norms: list[float] = []
    for step in range(1, PROBE_STEPS + 1):
        batch = _ordered_train_batch(preparation.predecessor, step, device)
        _sync(device)
        started = time.perf_counter()
        loss, gradient_norm = _train_step(candidate, optimizer, batch, step=step)
        _sync(device)
        measured.append(time.perf_counter() - started)
        losses.append(loss)
        gradient_norms.append(gradient_norm)
    evaluation_batch = _window_batch(preparation.fresh_heldout, 0, BATCH_SIZE, device)
    candidate.eval()
    _sync(device)
    started = time.perf_counter()
    with torch.no_grad():
        evaluation = candidate(evaluation_batch[:, :-1], evaluation_batch[:, 1:])
    _sync(device)
    evaluation_seconds = time.perf_counter() - started
    if evaluation.loss is None or int(evaluation.audit.forbidden_reads) != 0:
        raise RuntimeError("paired-H4 timing evaluation failed its causal audit")
    artifact_started = time.perf_counter()
    artifact = candidate.export_learned_artifact()
    artifact_seconds = time.perf_counter() - artifact_started
    (preparation.root / "preflight").mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=preparation.root / "preflight") as directory:
        checkpoint_path = Path(directory) / "probe.pt"
        checkpoint_started = time.perf_counter()
        torch.save(
            {
                "model": candidate.state_dict(),
                "optimizer": optimizer.state_dict(),
                "step": PROBE_STEPS,
            },
            checkpoint_path,
        )
        checkpoint_seconds = time.perf_counter() - checkpoint_started
    replay_started = time.perf_counter()
    replay = R4PairedH4LanguagePathV1(geometry).to(device)
    replay.load_learned_artifact(artifact)
    with torch.no_grad():
        expected = candidate(evaluation_batch[:1, :-1]).logits
        observed = replay(evaluation_batch[:1, :-1]).logits
    _sync(device)
    replay_seconds = time.perf_counter() - replay_started
    replay_delta = float((expected - observed).abs().max())
    if replay_delta != 0.0:
        raise RuntimeError("paired-H4 probe replay differs")
    projection = project_candidate_execution(
        mean_train_step_seconds=statistics.fmean(measured),
        evaluation_batch_seconds=evaluation_seconds,
        checkpoint_seconds=checkpoint_seconds,
        artifact_seconds=artifact_seconds,
        replay_seconds=replay_seconds,
        peak_memory_bytes=_peak_rss_bytes(),
        memory_budget_bytes=int(backend["memory_budget_bytes"]),
    )
    return {
        "backend": backend,
        "probe_steps": PROBE_STEPS,
        "measured_train_step_seconds": measured,
        "mean_train_step_seconds": statistics.fmean(measured),
        "losses": losses,
        "gradient_norms": gradient_norms,
        "evaluation_batch_seconds": evaluation_seconds,
        "checkpoint_seconds": checkpoint_seconds,
        "artifact_seconds": artifact_seconds,
        "replay_seconds": replay_seconds,
        "replay_maximum_logits_delta": replay_delta,
        "mechanics": mechanics,
        "projection": projection,
    }


ProbeExecutor = Callable[[CampaignPreparation], Mapping[str, Any]]


def probe_paired_h4_prompt_capacity(
    root: Path, *, _executor: ProbeExecutor | None = None
) -> dict[str, Any]:
    """Run the sole five-step CPU-four-thread admission/projection probe."""

    resolved_root = root.resolve()
    path = resolved_root / PROBE_RELATIVE_PATH
    if path.exists():
        result = _read_canonical_json(path)
        _verify_self_cid(result, "probe_cid")
        if result.get("implementation") != trainer_implementation_contract():
            raise ValueError("paired-H4 probe implementation differs")
        return result
    if (resolved_root / REVEAL_RELATIVE_PATH).exists():
        raise ValueError("paired-H4 probe cannot begin after prompt reveal")
    preparation = load_paired_h4_prompt_capacity_preparation(root)
    implementation = _require_implementation(preparation.manifest)
    executor = _execute_probe if _executor is None else _executor
    execution = dict(executor(preparation))
    projection = execution.get("projection")
    mechanics = execution.get("mechanics")
    eligible = bool(
        isinstance(projection, Mapping)
        and projection.get("eligible") is True
        and isinstance(mechanics, Mapping)
        and mechanics.get("passed") is True
        and execution.get("probe_steps") == PROBE_STEPS
    )
    body = {
        "schema": PROBE_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation.manifest["preparation_cid"],
        "prompt_commitment_cid": preparation.commitment["commitment_cid"],
        "implementation": implementation,
        "plan": CPU_PLAN.identity(),
        "execution": execution,
        "eligible": eligible,
        "verdict": "PAIRED_H4_EXECUTION_ADMITTED" if eligible else TERMINAL_UNAVAILABLE,
        "cuda": "FORBIDDEN",
        "mps": "NOT_USED",
    }
    result = _with_cid(body, "probe_cid")
    _write_exclusive_json(path, result)
    return result


@torch.no_grad()
def _evaluate_language(
    model: torch.nn.Module,
    windows: LanguagePathWindowStore,
    device: torch.device,
) -> dict[str, Any]:
    model.eval()
    loss_sum = 0.0
    top1 = 0
    rows = 0
    forbidden_reads = 0
    digest = blake3()
    for start in range(0, FRESH_HELDOUT_WINDOWS, BATCH_SIZE):
        count = min(BATCH_SIZE, FRESH_HELDOUT_WINDOWS - start)
        batch = _window_batch(windows, start, count, device)
        output = model(batch[:, :-1])
        logits = output.logits.float()
        targets = batch[:, 1:]
        loss_sum += float(
            F.cross_entropy(
                logits.reshape(-1, logits.shape[-1]),
                targets.reshape(-1),
                reduction="sum",
            ).cpu()
        )
        top1 += int((logits.argmax(dim=-1) == targets).sum().cpu())
        rows += int(targets.numel())
        forbidden_reads += int(getattr(output.audit, "forbidden_reads", -1))
        digest.update(logits.cpu().contiguous().numpy().tobytes())
    if rows != FRESH_HELDOUT_DECISIONS or forbidden_reads != 0:
        raise RuntimeError("fresh-heldout evaluation failed its row/causal audit")
    return {
        "rows": rows,
        "ce_nats": loss_sum / rows,
        "top1_correct": top1,
        "top1_rate": top1 / rows,
        "logits_cid": f"blake3:{digest.hexdigest()}",
        "forbidden_reads": forbidden_reads,
    }


def fresh_generalization_gates(
    *,
    candidate_initial: Mapping[str, Any],
    candidate_final: Mapping[str, Any],
    predecessor: Mapping[str, Any],
) -> dict[str, Any]:
    """Apply the frozen fresh general-language learning/nonregression gates."""

    if any(
        int(value.get("rows", -1)) != FRESH_HELDOUT_DECISIONS
        for value in (candidate_initial, candidate_final, predecessor)
    ):
        raise ValueError("fresh-heldout gate rows differ")
    nll_improvement = float(candidate_initial["ce_nats"]) - float(
        candidate_final["ce_nats"]
    )
    top1_point_improvement = 100.0 * (
        float(candidate_final["top1_rate"]) - float(candidate_initial["top1_rate"])
    )
    predecessor_nll_delta = float(candidate_final["ce_nats"]) - float(
        predecessor["ce_nats"]
    )
    predecessor_top1_point_delta = 100.0 * (
        float(candidate_final["top1_rate"]) - float(predecessor["top1_rate"])
    )
    gates = {
        "candidate_nll_improvement": nll_improvement >= REQUIRED_NLL_IMPROVEMENT,
        "candidate_top1_point_improvement": (
            top1_point_improvement >= REQUIRED_TOP1_POINT_IMPROVEMENT
        ),
        "candidate_final_nll_ceiling": (
            float(candidate_final["ce_nats"]) <= REQUIRED_FINAL_NLL_CEILING
        ),
        "predecessor_nll_nonregression": (
            predecessor_nll_delta <= PREDECESSOR_NLL_TOLERANCE
        ),
        "predecessor_top1_nonregression": (
            predecessor_top1_point_delta >= -PREDECESSOR_TOP1_POINT_TOLERANCE
        ),
        "forbidden_reads_zero": all(
            int(value.get("forbidden_reads", -1)) == 0
            for value in (candidate_initial, candidate_final, predecessor)
        ),
    }
    return {
        "passed": all(gates.values()),
        "gates": gates,
        "nll_improvement": nll_improvement,
        "top1_point_improvement": top1_point_improvement,
        "predecessor_nll_delta": predecessor_nll_delta,
        "predecessor_top1_point_delta": predecessor_top1_point_delta,
        "thresholds": {
            "nll_improvement": REQUIRED_NLL_IMPROVEMENT,
            "top1_point_improvement": REQUIRED_TOP1_POINT_IMPROVEMENT,
            "final_nll_ceiling": REQUIRED_FINAL_NLL_CEILING,
            "predecessor_nll_tolerance": PREDECESSOR_NLL_TOLERANCE,
            "predecessor_top1_point_tolerance": PREDECESSOR_TOP1_POINT_TOLERANCE,
        },
    }


def combine_terminal_verdict(
    *, prompt_verdict: str, language_passed: bool, mechanics_passed: bool
) -> dict[str, str]:
    """Map every predeclared outcome to one distinct next action."""

    if not mechanics_passed or prompt_verdict == VERDICT_INVALID:
        return {
            "verdict": TERMINAL_INVALID,
            "action": "repair only the failed causal, replay, seal, or control mechanic; do not interpret model metrics",
        }
    if prompt_verdict == VERDICT_PASS:
        if language_passed:
            return {
                "verdict": TERMINAL_PASS,
                "action": "freeze the paired-H4 candidate; run one disjoint autonomous subject/scene-retention smoke",
            }
        return {
            "verdict": TERMINAL_GENERAL_LANGUAGE_REGRESSION,
            "action": "reject the paired-H4 candidate and preserve qualified V1 despite prompt-contrast success",
        }
    if prompt_verdict == VERDICT_ABSOLUTE_NO_CAPACITY_GAIN:
        return {
            "verdict": TERMINAL_ABSOLUTE_NO_CAPACITY_GAIN,
            "action": "reject the paired-capacity change and preserve qualified V1",
        }
    if prompt_verdict == VERDICT_PARTIAL:
        return {
            "verdict": TERMINAL_PARTIAL,
            "action": "record partial prompt conditioning; do not generate, retry, widen, or lower",
        }
    if prompt_verdict == VERDICT_FAIL:
        return {
            "verdict": TERMINAL_FAIL,
            "action": "stop this capacity seam and revisit prompt-state-to-logit readout",
        }
    raise ValueError(f"unknown prompt-conditioning verdict: {prompt_verdict}")


def _atomic_torch_save(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    torch.save(dict(value), temporary)
    with temporary.open("rb+") as target:
        os.fsync(target.fileno())
    os.replace(temporary, path)


def _save_checkpoint(
    root: Path,
    *,
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    step: int,
    elapsed_seconds: float,
    initial_heldout: Mapping[str, Any],
    run_contract_cid: str,
    last_loss: float | None,
) -> dict[str, Any]:
    path = root / CHECKPOINT_RELATIVE_PATH
    checkpoint = {
        "schema": CHECKPOINT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "step": step,
        "elapsed_seconds": elapsed_seconds,
        "initial_heldout": dict(initial_heldout),
        "run_contract_cid": run_contract_cid,
        "plan_cid": CPU_PLAN.identity()["plan_cid"],
        "last_loss": last_loss,
        "model_state": model.state_dict(),
        "optimizer_state": optimizer.state_dict(),
        "cpu_rng_state": torch.get_rng_state(),
    }
    _atomic_torch_save(path, checkpoint)
    sidecar = {
        "schema": "uor-r4.paired-h4-prompt-capacity-checkpoint-cid/1",
        "step": step,
        "bytes": path.stat().st_size,
        "cid": cid_file(path),
        "run_contract_cid": run_contract_cid,
        "plan_cid": CPU_PLAN.identity()["plan_cid"],
    }
    atomic_write_json(root / CHECKPOINT_CID_RELATIVE_PATH, sidecar)
    return sidecar


def _load_checkpoint(
    root: Path,
    *,
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    run_contract_cid: str,
) -> dict[str, Any]:
    path = root / CHECKPOINT_RELATIVE_PATH
    sidecar = _read_canonical_json(root / CHECKPOINT_CID_RELATIVE_PATH)
    if (
        sidecar.get("schema") != "uor-r4.paired-h4-prompt-capacity-checkpoint-cid/1"
        or sidecar.get("bytes") != path.stat().st_size
        or sidecar.get("cid") != cid_file(path)
        or sidecar.get("run_contract_cid") != run_contract_cid
        or sidecar.get("plan_cid") != CPU_PLAN.identity()["plan_cid"]
    ):
        raise ValueError("paired-H4 checkpoint CID sidecar differs")
    checkpoint = torch.load(path, map_location="cpu", weights_only=True)
    if (
        not isinstance(checkpoint, dict)
        or checkpoint.get("schema") != CHECKPOINT_SCHEMA
        or checkpoint.get("policy") != POLICY
        or checkpoint.get("run_contract_cid") != run_contract_cid
        or checkpoint.get("plan_cid") != CPU_PLAN.identity()["plan_cid"]
        or checkpoint.get("step") != sidecar.get("step")
    ):
        raise ValueError("paired-H4 checkpoint envelope differs")
    step = checkpoint["step"]
    if (
        isinstance(step, bool)
        or not isinstance(step, int)
        or not 0 <= step <= OPTIMIZER_STEPS
    ):
        raise ValueError("paired-H4 checkpoint step differs")
    model.load_state_dict(checkpoint["model_state"], strict=True)
    optimizer.load_state_dict(checkpoint["optimizer_state"])
    _optimizer_to_device(optimizer, device)
    rng = checkpoint.get("cpu_rng_state")
    if not isinstance(rng, Tensor):
        raise TypeError("paired-H4 checkpoint omitted CPU RNG state")
    torch.set_rng_state(rng)
    expected_rate = learning_rate(step)
    if any(
        not math.isclose(float(group["lr"]), expected_rate, rel_tol=0.0, abs_tol=1e-15)
        for group in optimizer.param_groups
    ):
        raise ValueError("paired-H4 checkpoint learning rate differs")
    return checkpoint


def _write_progress(
    root: Path,
    *,
    step: int,
    elapsed_seconds: float,
    last_loss: float | None,
    status: str,
) -> dict[str, Any]:
    sidecar = _read_canonical_json(root / CHECKPOINT_CID_RELATIVE_PATH)
    rate = step / elapsed_seconds if elapsed_seconds > 0.0 else 0.0
    progress = {
        "schema": "uor-r4.paired-h4-prompt-capacity-progress/1",
        "issue": ISSUE,
        "policy": POLICY,
        "status": status,
        "completed_steps": step,
        "total_steps": OPTIMIZER_STEPS,
        "completed_presentations": step * BATCH_SIZE * CONTEXT,
        "total_presentations": TRAIN_DECISIONS,
        "elapsed_seconds": elapsed_seconds,
        "steps_per_second": rate,
        "eta_seconds": (OPTIMIZER_STEPS - step) / rate if rate > 0.0 else None,
        "last_loss": last_loss,
        "checkpoint": dict(sidecar),
        "resume": "run-paired-h4-prompt-capacity --resume",
    }
    atomic_write_json(root / PROGRESS_RELATIVE_PATH, progress)
    return progress


def _resume_elapsed(root: Path, checkpoint: Mapping[str, Any]) -> float:
    checkpoint_elapsed = float(checkpoint["elapsed_seconds"])
    progress_path = root / PROGRESS_RELATIVE_PATH
    if not progress_path.exists():
        return checkpoint_elapsed
    progress = _read_canonical_json(progress_path)
    progress_elapsed = float(progress.get("elapsed_seconds", -1.0))
    progress_step = progress.get("completed_steps")
    if (
        progress.get("schema") != "uor-r4.paired-h4-prompt-capacity-progress/1"
        or isinstance(progress_step, bool)
        or not isinstance(progress_step, int)
        or not 0 <= progress_step <= OPTIMIZER_STEPS
        or progress_elapsed < checkpoint_elapsed
        or progress_step < int(checkpoint["step"])
    ):
        raise ValueError("paired-H4 durable progress contradicts its checkpoint")
    return progress_elapsed


def _candidate_artifact_replay(
    *,
    model: R4PairedH4LanguagePathV1,
    geometry: Any,
    artifact: bytes,
    heldout: LanguagePathWindowStore,
    device: torch.device,
) -> dict[str, Any]:
    prefix = _window_batch(heldout, 0, 1, device)[:, :65]
    model.eval()
    with torch.no_grad():
        expected = model(prefix[:, :-1]).logits
    replay = R4PairedH4LanguagePathV1(geometry).to(device)
    replay.load_learned_artifact(artifact)
    replay.eval()
    with torch.no_grad():
        observed = replay(prefix[:, :-1]).logits
        direct = replay(prefix[:, :-1], implementation="direct").logits
    reload_delta = float((expected - observed).abs().max())
    direct_delta = float((observed - direct).abs().max())
    return {
        "prefix_tokens": int(prefix.shape[1] - 1),
        "artifact_reload_maximum_logits_delta": reload_delta,
        "direct_maximum_logits_delta": direct_delta,
        "passed": reload_delta == 0.0 and direct_delta <= 2e-5,
    }


def _run_contract(
    preparation: CampaignPreparation, probe: Mapping[str, Any]
) -> dict[str, Any]:
    return {
        "policy": POLICY,
        "model_policy": MODEL_POLICY,
        "preparation_cid": preparation.manifest["preparation_cid"],
        "probe_cid": probe["probe_cid"],
        "implementation": preparation.manifest["implementation"],
        "plan": CPU_PLAN.identity(),
        "trained_arms": ["paired_h4_candidate"],
        "predecessor_retrained": False,
        "model": {
            "parameters": PARAMETER_COUNT,
            "state_values": STATE_VALUES,
            "state_bytes_f32": STATE_BYTES_F32,
            "validity_bits": VALIDITY_BITS,
        },
        "training": {
            "windows": TRAIN_WINDOWS,
            "decisions": TRAIN_DECISIONS,
            "batch_size": BATCH_SIZE,
            "steps": OPTIMIZER_STEPS,
            "seed": INITIALIZATION_SEED,
            "train_order": _train_order_identity(preparation.predecessor),
            "optimizer": "AdamW",
            "betas": [ADAM_BETA1, ADAM_BETA2],
            "epsilon": ADAM_EPSILON,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip": GRADIENT_CLIP,
            "warmup_steps": WARMUP_STEPS,
            "maximum_learning_rate": MAXIMUM_LEARNING_RATE,
            "minimum_learning_rate": MINIMUM_LEARNING_RATE,
            "one_epoch_without_replacement": True,
        },
        "fresh_heldout_cid": FRESH_HELDOUT_CID,
        "prompt_population_cid": EXPECTED_PROMPT_POPULATION_CID,
        "prompt_reveal": "only after both artifact CIDs are fixed",
        "prompt_evaluation_ceiling_seconds": PROMPT_EVALUATION_CEILING_SECONDS,
        "hard_wall_ceiling_seconds": HARD_WALL_CEILING_SECONDS,
        "retry": "same checkpoint trajectory only",
        "cuda": "FORBIDDEN",
        "mps": "NOT_USED",
    }


def _factories(
    *,
    geometry: Any,
    predecessor_artifact: bytes,
    candidate_artifact: bytes,
    device: torch.device,
) -> tuple[Callable[[], Any], Callable[[], Any]]:
    def predecessor_factory() -> R4RetainedLanguagePathV1:
        model = R4RetainedLanguagePathV1(geometry).to(device)
        model.load_learned_artifact(predecessor_artifact)
        return model

    def candidate_factory() -> R4PairedH4LanguagePathV1:
        model = R4PairedH4LanguagePathV1(geometry).to(device)
        model.load_learned_artifact(candidate_artifact)
        return model

    return predecessor_factory, candidate_factory


def _load_terminal_result(root: Path) -> dict[str, Any]:
    """Verify a terminal envelope and every artifact/reveal binding it claims."""

    result = _read_canonical_json(root / RESULT_RELATIVE_PATH)
    _verify_self_cid(result, "result_cid")
    if (
        result.get("schema") != RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
    ):
        raise ValueError("paired-H4 terminal result envelope differs")
    candidate = result.get("candidate_artifact")
    if isinstance(candidate, Mapping):
        candidate_path = root / str(candidate.get("path"))
        if (
            candidate_path.is_symlink()
            or not candidate_path.is_file()
            or candidate.get("bytes") != candidate_path.stat().st_size
            or candidate.get("cid") != cid_file(candidate_path)
        ):
            raise ValueError("paired-H4 terminal candidate artifact differs")
    reveal_binding = result.get("prompt_reveal")
    if isinstance(reveal_binding, Mapping):
        reveal = _read_canonical_json(root / REVEAL_RELATIVE_PATH)
        _verify_self_cid(reveal, "reveal_cid")
        expected = {
            "cid": reveal.get("reveal_cid"),
            "population_cid": reveal.get("population_cid"),
            "baseline_artifact_cid": reveal.get("baseline_artifact_cid"),
            "candidate_artifact_cid": reveal.get("candidate_artifact_cid"),
        }
        if reveal_binding != expected:
            raise ValueError("paired-H4 terminal prompt reveal binding differs")
        if (
            expected["baseline_artifact_cid"] != PREDECESSOR_ARTIFACT_CID
            or not isinstance(candidate, Mapping)
            or expected["candidate_artifact_cid"] != candidate.get("cid")
        ):
            raise ValueError("paired-H4 terminal artifacts differ from prompt reveal")
    return result


def run_paired_h4_prompt_capacity(root: Path, resume: bool = False) -> dict[str, Any]:
    """Run or resume the sole candidate fit, reveal, and frozen decision."""

    process_started = time.monotonic()
    resolved_root = root.resolve()
    if (resolved_root / RESULT_RELATIVE_PATH).exists():
        return _load_terminal_result(resolved_root)
    if not resume and (resolved_root / REVEAL_RELATIVE_PATH).exists():
        raise ValueError("paired-H4 population was revealed before candidate fitting")
    preparation = load_paired_h4_prompt_capacity_preparation(root)
    implementation = _require_implementation(preparation.manifest)
    result_path = preparation.root / RESULT_RELATIVE_PATH
    probe = _read_canonical_json(preparation.root / PROBE_RELATIVE_PATH)
    _verify_self_cid(probe, "probe_cid")
    if (
        probe.get("eligible") is not True
        or probe.get("implementation") != implementation
        or probe.get("preparation_cid") != preparation.manifest["preparation_cid"]
    ):
        raise ValueError("paired-H4 run requires its eligible current probe")
    contract = _run_contract(preparation, probe)
    contract_cid = cid_bytes(canonical_json_bytes(contract))
    started_path = preparation.root / STARTED_RELATIVE_PATH
    if resume:
        started = _read_canonical_json(started_path)
        _verify_self_cid(started, "started_cid")
        if (
            started.get("schema") != STARTED_SCHEMA
            or started.get("issue") != ISSUE
            or started.get("policy") != POLICY
            or started.get("run_contract_cid") != contract_cid
            or started.get("run_contract") != contract
        ):
            raise ValueError("paired-H4 resume run contract differs")
    else:
        if (
            started_path.exists()
            or (preparation.root / CHECKPOINT_RELATIVE_PATH).exists()
        ):
            raise FileExistsError("paired-H4 run already started; resume is required")
        started = _with_cid(
            {
                "schema": STARTED_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "preparation_cid": preparation.manifest["preparation_cid"],
                "probe_cid": probe["probe_cid"],
                "prompt_commitment_cid": preparation.commitment["commitment_cid"],
                "implementation": implementation,
                "run_contract": contract,
                "run_contract_cid": contract_cid,
                "prompt_population": {"status": "SEALED", "reads": 0},
            },
            "started_cid",
        )
        _write_exclusive_json(started_path, started)

    device, backend = _configure_device(CPU_PLAN)
    geometry = _exact_geometry(preparation.predecessor)
    candidate = R4PairedH4LanguagePathV1(geometry).to(device)
    optimizer = _optimizer(candidate)
    elapsed_before = 0.0
    step = 0
    last_loss: float | None = None
    if resume and (preparation.root / CHECKPOINT_RELATIVE_PATH).exists():
        checkpoint = _load_checkpoint(
            preparation.root,
            model=candidate,
            optimizer=optimizer,
            device=device,
            run_contract_cid=contract_cid,
        )
        step = int(checkpoint["step"])
        elapsed_before = _resume_elapsed(preparation.root, checkpoint)
        initial_heldout = dict(checkpoint["initial_heldout"])
        last_loss = checkpoint.get("last_loss")
    else:
        if resume and (preparation.root / PROGRESS_RELATIVE_PATH).exists():
            raise ValueError("paired-H4 progress exists without a resume checkpoint")
        initial_heldout = _evaluate_language(
            candidate, preparation.fresh_heldout, device
        )
        elapsed = time.monotonic() - process_started
        _save_checkpoint(
            preparation.root,
            model=candidate,
            optimizer=optimizer,
            step=0,
            elapsed_seconds=elapsed,
            initial_heldout=initial_heldout,
            run_contract_cid=contract_cid,
            last_loss=None,
        )
        _write_progress(
            preparation.root,
            step=0,
            elapsed_seconds=elapsed,
            last_loss=None,
            status="RUNNING",
        )
    for next_step in range(step + 1, OPTIMIZER_STEPS + 1):
        batch = _ordered_train_batch(preparation.predecessor, next_step, device)
        last_loss, _ = _train_step(candidate, optimizer, batch, step=next_step)
        step = next_step
        elapsed = elapsed_before + (time.monotonic() - process_started)
        checkpoint_due = step % CHECKPOINT_INTERVAL == 0 or step == OPTIMIZER_STEPS
        progress_due = step % PROGRESS_INTERVAL == 0 or step == OPTIMIZER_STEPS
        if checkpoint_due:
            _save_checkpoint(
                preparation.root,
                model=candidate,
                optimizer=optimizer,
                step=step,
                elapsed_seconds=elapsed,
                initial_heldout=initial_heldout,
                run_contract_cid=contract_cid,
                last_loss=last_loss,
            )
        if progress_due:
            elapsed = elapsed_before + (time.monotonic() - process_started)
            progress = _write_progress(
                preparation.root,
                step=step,
                elapsed_seconds=elapsed,
                last_loss=last_loss,
                status="RUNNING",
            )
            print(
                f"paired_h4_prompt_capacity step={step}/{OPTIMIZER_STEPS} "
                f"loss={last_loss:.6f} eta={progress['eta_seconds']}",
                flush=True,
            )
        elapsed = elapsed_before + (time.monotonic() - process_started)
        if elapsed >= HARD_WALL_CEILING_SECONDS:
            if not checkpoint_due:
                _save_checkpoint(
                    preparation.root,
                    model=candidate,
                    optimizer=optimizer,
                    step=step,
                    elapsed_seconds=elapsed,
                    initial_heldout=initial_heldout,
                    run_contract_cid=contract_cid,
                    last_loss=last_loss,
                )
            progress = _write_progress(
                preparation.root,
                step=step,
                elapsed_seconds=elapsed,
                last_loss=last_loss,
                status=TERMINAL_UNAVAILABLE,
            )
            result = _with_cid(
                {
                    "schema": RESULT_SCHEMA,
                    "issue": ISSUE,
                    "policy": POLICY,
                    "started_cid": started["started_cid"],
                    "run_contract_cid": contract_cid,
                    "verdict": TERMINAL_UNAVAILABLE,
                    "action": "stop; the frozen compute contract is unavailable",
                    "progress": progress,
                    "prompt_population": {"status": "SEALED", "reads": 0},
                    "h4_specific_superiority": "NOT_EVALUATED",
                },
                "result_cid",
            )
            _write_exclusive_json(result_path, result)
            return result

    candidate_final = _evaluate_language(candidate, preparation.fresh_heldout, device)
    predecessor_artifact = preparation.predecessor_artifact_path.read_bytes()
    predecessor_model = R4RetainedLanguagePathV1(geometry).to(device)
    predecessor_model.load_learned_artifact(predecessor_artifact)
    predecessor_heldout = _evaluate_language(
        predecessor_model, preparation.fresh_heldout, device
    )
    candidate_artifact = candidate.export_learned_artifact()
    candidate_artifact_path = preparation.root / CANDIDATE_ARTIFACT_RELATIVE_PATH
    if candidate_artifact_path.exists():
        if cid_bytes(candidate_artifact) != cid_file(candidate_artifact_path):
            raise ValueError("resumed paired-H4 candidate artifact differs")
    else:
        atomic_write(candidate_artifact_path, candidate_artifact)
    candidate_artifact_cid = cid_file(candidate_artifact_path)
    replay = _candidate_artifact_replay(
        model=candidate,
        geometry=geometry,
        artifact=candidate_artifact,
        heldout=preparation.fresh_heldout,
        device=device,
    )
    if not replay["passed"]:
        raise RuntimeError("paired-H4 candidate artifact replay differs")

    elapsed = elapsed_before + (time.monotonic() - process_started)
    if (
        elapsed + PROMPT_EVALUATION_CEILING_SECONDS
        > HARD_WALL_CEILING_SECONDS
    ):
        progress = _write_progress(
            preparation.root,
            step=OPTIMIZER_STEPS,
            elapsed_seconds=elapsed,
            last_loss=last_loss,
            status=TERMINAL_UNAVAILABLE,
        )
        result = _with_cid(
            {
                "schema": RESULT_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "started_cid": started["started_cid"],
                "run_contract_cid": contract_cid,
                "verdict": TERMINAL_UNAVAILABLE,
                "action": (
                    "stop before reveal; the frozen whole-process wall cannot "
                    "reserve the complete prompt-evaluation ceiling"
                ),
                "progress": progress,
                "candidate_artifact": {
                    "path": CANDIDATE_ARTIFACT_RELATIVE_PATH,
                    "bytes": candidate_artifact_path.stat().st_size,
                    "cid": candidate_artifact_cid,
                    "fixed_before_prompt_reveal": True,
                },
                "predecessor_artifact_cid": PREDECESSOR_ARTIFACT_CID,
                "prompt_population": {"status": "SEALED", "reads": 0},
                "h4_specific_superiority": "NOT_EVALUATED",
            },
            "result_cid",
        )
        _write_exclusive_json(result_path, result)
        return result

    # This call is the first operation allowed to open the prompt population.
    if (preparation.root / REVEAL_RELATIVE_PATH).exists():
        population = load_revealed_prompt_conditioning_population(preparation.root)
        reveal = _read_canonical_json(preparation.root / REVEAL_RELATIVE_PATH)
        if (
            reveal.get("baseline_artifact_cid") != PREDECESSOR_ARTIFACT_CID
            or reveal.get("candidate_artifact_cid") != candidate_artifact_cid
        ):
            raise ValueError("resumed prompt reveal binds different artifacts")
    else:
        population = reveal_prompt_conditioning_population(
            preparation.root,
            baseline_artifact_cid=PREDECESSOR_ARTIFACT_CID,
            candidate_artifact_cid=candidate_artifact_cid,
        )
    baseline_factory, candidate_factory = _factories(
        geometry=geometry,
        predecessor_artifact=predecessor_artifact,
        candidate_artifact=candidate_artifact,
        device=device,
    )
    prompt_started = time.perf_counter()
    reveal_record = _read_canonical_json(preparation.root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal_record, "reveal_cid")
    if (
        reveal_record.get("baseline_artifact_cid") != PREDECESSOR_ARTIFACT_CID
        or reveal_record.get("candidate_artifact_cid") != candidate_artifact_cid
    ):
        raise ValueError("prompt reveal does not bind the fitted artifact files")
    prompt_decision = evaluate_prompt_conditioning(
        population=population,
        reveal_cid=str(reveal_record["reveal_cid"]),
        baseline_artifact_cid=PREDECESSOR_ARTIFACT_CID,
        candidate_artifact_cid=candidate_artifact_cid,
        baseline_factory=baseline_factory,
        candidate_factory=candidate_factory,
        device=device,
    )
    prompt_seconds = time.perf_counter() - prompt_started
    prompt_record = prompt_decision.record()
    language = fresh_generalization_gates(
        candidate_initial=initial_heldout,
        candidate_final=candidate_final,
        predecessor=predecessor_heldout,
    )
    control_mechanics_passed = bool(
        probe.get("execution", {}).get("mechanics", {}).get("passed") is True
        and replay["passed"] is True
        and prompt_record.get("population_cid") == EXPECTED_PROMPT_POPULATION_CID
        and prompt_record.get("reveal_cid") == reveal_record["reveal_cid"]
        and prompt_record.get("artifacts")
        == {
            "baseline": PREDECESSOR_ARTIFACT_CID,
            "candidate": candidate_artifact_cid,
        }
        and all(
            int(score.get("forbidden_reads", -1)) == 0
            for name, score in prompt_record.items()
            if name
            in {"baseline", "candidate", "baseline_state_off", "candidate_state_off"}
        )
    )
    prompt_within_ceiling = prompt_seconds <= PROMPT_EVALUATION_CEILING_SECONDS
    elapsed = elapsed_before + (time.monotonic() - process_started)
    prompt_verdict = str(prompt_record["verdict"])
    if not control_mechanics_passed or prompt_verdict == VERDICT_INVALID:
        decision = combine_terminal_verdict(
            prompt_verdict=prompt_verdict,
            language_passed=bool(language["passed"]),
            mechanics_passed=control_mechanics_passed,
        )
    elif not prompt_within_ceiling or elapsed > HARD_WALL_CEILING_SECONDS:
        decision = {
            "verdict": TERMINAL_UNAVAILABLE,
            "action": "stop; the frozen evaluation compute ceiling was exceeded",
        }
    else:
        decision = combine_terminal_verdict(
            prompt_verdict=prompt_verdict,
            language_passed=bool(language["passed"]),
            mechanics_passed=True,
        )
    artifact_record = {
        "path": CANDIDATE_ARTIFACT_RELATIVE_PATH,
        "bytes": candidate_artifact_path.stat().st_size,
        "cid": candidate_artifact_cid,
        "fixed_before_prompt_reveal": True,
    }
    body = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "started_cid": started["started_cid"],
        "run_contract_cid": contract_cid,
        "probe_cid": probe["probe_cid"],
        "implementation": implementation,
        "backend": backend,
        "completed_steps": OPTIMIZER_STEPS,
        "presentations": TRAIN_DECISIONS,
        "elapsed_seconds": elapsed,
        "candidate_artifact": artifact_record,
        "predecessor_artifact_cid": PREDECESSOR_ARTIFACT_CID,
        "prompt_reveal": {
            "cid": reveal_record["reveal_cid"],
            "population_cid": reveal_record["population_cid"],
            "baseline_artifact_cid": reveal_record["baseline_artifact_cid"],
            "candidate_artifact_cid": reveal_record["candidate_artifact_cid"],
        },
        "artifact_replay": replay,
        "fresh_heldout": {
            "candidate_initial": initial_heldout,
            "candidate_final": candidate_final,
            "predecessor": predecessor_heldout,
            "decision": language,
        },
        "prompt_evaluation_seconds": prompt_seconds,
        "prompt_evaluation_within_ceiling": prompt_within_ceiling,
        "prompt_decision": prompt_record,
        "mechanics_passed": control_mechanics_passed,
        "decision": decision,
        "verdict": decision["verdict"],
        "h4_specific_superiority": "NOT_EVALUATED",
        "generation": "NOT_RUN",
        "reasoning": "NOT_RUN",
        "lowering": "NOT_RUN",
    }
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(result_path, result)
    _write_progress(
        preparation.root,
        step=OPTIMIZER_STEPS,
        elapsed_seconds=elapsed,
        last_loss=last_loss,
        status="COMPLETE",
    )
    return result


__all__ = [
    "POLICY",
    "CampaignPreparation",
    "combine_terminal_verdict",
    "fresh_generalization_gates",
    "load_paired_h4_prompt_capacity_preparation",
    "prepare_paired_h4_prompt_capacity",
    "probe_paired_h4_prompt_capacity",
    "project_candidate_execution",
    "run_paired_h4_prompt_capacity",
    "structural_collision_census",
]
