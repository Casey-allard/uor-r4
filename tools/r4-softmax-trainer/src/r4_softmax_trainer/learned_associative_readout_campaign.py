"""Bounded learned-associative-readout campaign for issue #973.

The campaign keeps the qualified retained-language-path V1 backbone frozen and
fits two independent candidate-query tables: one reads the exact geometric
leaf, while the matched control reads the occupied-address mean.  The V4
prompt population and a disjoint fresh-heldout continuation are created once
and sealed together before either query table is fitted.

This module deliberately owns the complete lifecycle.  A probe may read only
the predecessor training population.  Both final head artifact CIDs and the
qualified V1 CID are fixed before the one-time reveal.  No optimizer can be
constructed after that marker exists, and the independent verifier must run
in a different process from the result writer.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import shutil
import statistics
import tempfile
import time
import traceback
from collections.abc import Callable, Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from queue import Empty
from typing import Any, Literal

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .language_path_generalization import (
    CONTEXT,
    INITIALIZATION_SEED,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    R4RetainedLanguagePathV1,
)
from .language_path_generalization_campaign import (
    ADAM_BETA1,
    ADAM_BETA2,
    ADAM_EPSILON,
    BATCH_SIZE,
    CHECKPOINT_INTERVAL,
    EQUIVALENCE_ABS_TOLERANCE,
    GRADIENT_CLIP,
    MAXIMUM_LEARNING_RATE,
    MEMORY_FRACTION_CEILING,
    MINIMUM_LEARNING_RATE,
    OPTIMIZER_STEPS,
    PROBE_MEASURED_STEPS,
    PROBE_WARMUP_STEPS,
    PROGRESS_INTERVAL,
    PROJECTION_SAFETY_FACTOR,
    TRAIN_DECISIONS,
    TRAIN_WINDOWS,
    WARMUP_STEPS,
    WEIGHT_DECAY,
    ExecutionPlan,
    _configure_device,
    _exact_geometry,
    _ordered_train_batch,
    _peak_rss_bytes,
    _sync,
    _train_order_identity,
    _window_batch,
    learning_rate,
)
from .language_path_generalization_data import LanguagePathData, LanguagePathWindowStore
from .layerwise_normalized_retained_readout_campaign import (
    PREDECESSOR_ARTIFACT_BYTES,
    PREDECESSOR_ARTIFACT_CID,
    PREDECESSOR_ARM_RESULT_CID,
    PREDECESSOR_POLICY,
    PREDECESSOR_PREPARATION_MANIFEST_CID,
    PREDECESSOR_RESULT_CID,
    _verify_predecessor,
)
from .learned_associative_readout import (
    EFFECTIVE_ARM_PARAMETER_COUNT,
    HEAD_PARAMETER_COUNT,
    POLICY as MODEL_POLICY,
    QUERY_SHAPE,
    R4LearnedCandidateLeafAssociativeReadoutV1,
)
from .prompt_conditioning_v4 import (
    COMMITMENT_RELATIVE_PATH,
    GEOMETRY_ATTRIBUTION_INVALID,
    GEOMETRY_ATTRIBUTION_PASS,
    POPULATION_RELATIVE_PATH,
    REVEAL_RELATIVE_PATH,
    VERDICT_INVALID,
    VERDICT_PASS,
    associative_capacity_decision,
    geometry_attribution_decision,
    load_prompt_conditioning_commitment,
    load_required_prior_story_cids,
    load_revealed_prompt_conditioning_population,
    reveal_prompt_conditioning_population,
    score_prompt_conditioning,
    seal_prompt_conditioning_population,
    select_prompt_conditioning_population_from_source,
    stage_prompt_conditioning_population,
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
POLICY = "R4LearnedAssociativeReadoutPromptCapacityV1"
ARMS = ("geometric", "pooled")

# The training dose and order are inherited exactly from the qualified V1 fit.
if OPTIMIZER_STEPS != 2_730 or TRAIN_WINDOWS != 43_680 or TRAIN_DECISIONS != 5_241_600:
    raise RuntimeError("learned-associative training dose drifted")

FRESH_HELDOUT_SOURCE_OFFSET_TOKENS = 156_032_138
FRESH_HELDOUT_TOKENS = 249_986
FRESH_HELDOUT_WINDOWS = 2_066
FRESH_HELDOUT_DECISIONS = FRESH_HELDOUT_WINDOWS * CONTEXT
FRESH_HELDOUT_REACHABLE_DECISIONS = FRESH_HELDOUT_DECISIONS - FRESH_HELDOUT_WINDOWS
FRESH_HELDOUT_CID = (
    "blake3:77dfa0744c140e5affe9be233244e616c940dbff469f786deadeb87768e3c752"
)
FRESH_HELDOUT_TRAIN_INDEX_CID = (
    "blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e"
)
FRESH_HELDOUT_FIRST_CAPACITY_STORY = 764_050
FRESH_HELDOUT_FIRST_SOURCE_STORY = 848_493
FRESH_HELDOUT_LAST_CAPACITY_STORY = 765_247
FRESH_HELDOUT_LAST_SOURCE_STORY = 849_802
FRESH_HELDOUT_STORY_CIDS = 1_198
FRESH_HELDOUT_STORY_CIDS_CID = (
    "blake3:c112790145657c771cf72d63d8e1f055b3b2d772f1cd3485c7cacb74dbb1e4a0"
)

PREPARATION_RELATIVE_PATH = "learned-associative-readout-preparation.json"
HELDOUT_RELATIVE_PATH = "evaluation/sealed/fresh-heldout.u16"
PROBE_RELATIVE_PATH = "preflight/learned-associative-readout-probe.json"
STARTED_RELATIVE_PATH = "run/learned-associative-readout-started.json"
RESULT_RELATIVE_PATH = "run/learned-associative-readout-result.json"
UNAVAILABLE_RELATIVE_PATH = "run/learned-associative-readout-unavailable.json"
SCORING_EVIDENCE_RELATIVE_PATH = (
    "run/learned-associative-readout-scoring-evidence.json"
)
VERIFICATION_RELATIVE_PATH = "run/learned-associative-readout-independent-verification.json"

PREPARATION_SCHEMA = "uor-r4.learned-associative-readout-preparation/1"
PROBE_SCHEMA = "uor-r4.learned-associative-readout-probe/1"
STARTED_SCHEMA = "uor-r4.learned-associative-readout-started/1"
CHECKPOINT_SCHEMA = "uor-r4.learned-associative-readout-checkpoint/1"
ARM_RESULT_SCHEMA = "uor-r4.learned-associative-readout-arm-result/1"
RESULT_SCHEMA = "uor-r4.learned-associative-readout-result/1"
VERIFICATION_SCHEMA = "uor-r4.learned-associative-readout-independent-verification/1"

TERMINAL_GEOMETRIC_PASS = "GEOMETRIC_ASSOCIATIVE_CAPACITY_AND_ATTRIBUTION_PASS"
TERMINAL_CONTROL_ONLY = "ASSOCIATIVE_CAPACITY_WITHOUT_GEOMETRY_ATTRIBUTION"
TERMINAL_FRESH_REGRESSION = "ASSOCIATIVE_PROMPT_CAPACITY_FRESH_LANGUAGE_REGRESSION"
TERMINAL_NO_CAPACITY = "LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY"
TERMINAL_INVALID = "INVALID_LEARNED_ASSOCIATIVE_READOUT_MECHANICS"
TERMINAL_UNAVAILABLE = "UNAVAILABLE_LEARNED_ASSOCIATIVE_READOUT_COMPUTE"

PREDECESSOR_NLL_TOLERANCE = 0.05
PREDECESSOR_TOP1_POINT_TOLERANCE = 1.0
REQUIRED_STATE_OFF_NLL_COST = 0.10
REQUIRED_STATE_OFF_TOP1_DECISION_LOSS = 2_480
HARD_WALL_CEILING_SECONDS = 7_200.0
# Leave enough residual wall to terminate a timed-out scorer and publish the
# resulting durable state.  The charged scoring time includes this reserve.
RESULT_FINALIZATION_RESERVE_SECONDS = 15.0
DIRECTION_BATCH_SIZE = 8
PROMPT_SCORE_CALLS = 10
PROMPT_BATCHES_PER_SCORE = 512 // DIRECTION_BATCH_SIZE
FRESH_SCORE_CALLS = 5
FRESH_BATCHES_PER_SCORE = math.ceil(FRESH_HELDOUT_WINDOWS / BATCH_SIZE)
CANONICAL_SCORING_EQUIVALENT_BATCHES = (
    math.ceil(
        PROMPT_SCORE_CALLS
        * PROMPT_BATCHES_PER_SCORE
        * 64
        / CONTEXT
    )
    + FRESH_SCORE_CALLS * FRESH_BATCHES_PER_SCORE
)

ELIGIBLE_PLANS = (
    ExecutionPlan("cpu-accelerate-4t-sequential", "cpu", 4, 1, False),
    ExecutionPlan("cpu-accelerate-8t-sequential", "cpu", 8, 1, False),
    ExecutionPlan("cpu-accelerate-2x4t-concurrent", "cpu", 4, 2, True),
    ExecutionPlan("mps-deterministic-sequential", "mps", 1, 1, False),
)


@dataclass(frozen=True, slots=True)
class CampaignPreparation:
    """Verified nonsealed inputs plus a commitment, never a sealed-data view."""

    root: Path
    manifest: dict[str, Any]
    predecessor: LanguagePathData
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


def _read_json(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"expected a regular non-symlink JSON file: {path}")
    payload = path.read_bytes()
    try:
        value = json.loads(payload.decode("utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read canonical JSON: {path}") from error
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
    byte_offset = offset_tokens * 2
    byte_count = token_count * 2
    if offset_tokens < 0 or token_count < 1 or byte_offset + byte_count > path.stat().st_size:
        raise ValueError("fresh-heldout slice coordinates cross the source store")
    with path.open("rb") as source:
        source.seek(byte_offset)
        payload = source.read(byte_count)
    if len(payload) != byte_count:
        raise ValueError("fresh-heldout source ended inside the frozen slice")
    return payload


def _verify_fresh_heldout_index(path: Path) -> tuple[dict[str, Any], frozenset[str]]:
    """Bind the fixed slice to the canonical story-index boundaries."""

    if path.is_symlink() or not path.is_file() or cid_file(path) != FRESH_HELDOUT_TRAIN_INDEX_CID:
        raise ValueError("fresh-heldout train index differs from the freeze")
    wanted = set(range(FRESH_HELDOUT_FIRST_CAPACITY_STORY, FRESH_HELDOUT_LAST_CAPACITY_STORY + 1))
    records: dict[int, dict[str, Any]] = {}
    with path.open("rb") as source:
        for ordinal, line in enumerate(source):
            if ordinal not in wanted:
                continue
            try:
                record = json.loads(line.decode("utf-8"))
            except (UnicodeError, json.JSONDecodeError) as error:
                raise ValueError("fresh-heldout train index is malformed") from error
            if not isinstance(record, dict) or canonical_json_bytes(record) != line:
                raise ValueError("fresh-heldout train index record is not canonical")
            records[ordinal] = record
            if len(records) == len(wanted):
                break
    if set(records) != wanted:
        raise ValueError("fresh-heldout story boundaries are absent from the index")
    first = records[FRESH_HELDOUT_FIRST_CAPACITY_STORY]
    last = records[FRESH_HELDOUT_LAST_CAPACITY_STORY]
    end_offset = FRESH_HELDOUT_SOURCE_OFFSET_TOKENS + FRESH_HELDOUT_TOKENS
    if (
        first.get("capacity_story_ordinal") != FRESH_HELDOUT_FIRST_CAPACITY_STORY
        or first.get("source_story_ordinal") != FRESH_HELDOUT_FIRST_SOURCE_STORY
        or first.get("story_token_offset") != FRESH_HELDOUT_SOURCE_OFFSET_TOKENS
        or last.get("capacity_story_ordinal") != FRESH_HELDOUT_LAST_CAPACITY_STORY
        or last.get("source_story_ordinal") != FRESH_HELDOUT_LAST_SOURCE_STORY
        or not isinstance(last.get("story_token_offset"), int)
        or not isinstance(last.get("story_token_count"), int)
        or not int(last["story_token_offset"]) < end_offset
        <= int(last["story_token_offset"]) + int(last["story_token_count"])
    ):
        raise ValueError("fresh-heldout story boundaries differ from the freeze")
    story_cids = frozenset(str(record.get("story_cid")) for record in records.values())
    if (
        len(story_cids) != FRESH_HELDOUT_STORY_CIDS
        or cid_bytes(canonical_json_bytes(sorted(story_cids))) != FRESH_HELDOUT_STORY_CIDS_CID
        or any(not value.startswith("blake3:") for value in story_cids)
    ):
        raise ValueError("fresh-heldout story-CID witness differs from the freeze")
    return {
        "path": str(path.resolve()),
        "cid": FRESH_HELDOUT_TRAIN_INDEX_CID,
        "first_capacity_story_ordinal": FRESH_HELDOUT_FIRST_CAPACITY_STORY,
        "first_source_story_ordinal": FRESH_HELDOUT_FIRST_SOURCE_STORY,
        "last_capacity_story_ordinal": FRESH_HELDOUT_LAST_CAPACITY_STORY,
        "last_source_story_ordinal": FRESH_HELDOUT_LAST_SOURCE_STORY,
        "story_cids": len(story_cids),
        "story_cids_cid": FRESH_HELDOUT_STORY_CIDS_CID,
    }, story_cids


def _cleanup_staging(staging: Path) -> None:
    sealed = staging / POPULATION_RELATIVE_PATH
    if sealed.parent.exists() and not sealed.parent.is_symlink():
        sealed.parent.chmod(0o700)
    if staging.exists() and not staging.is_symlink():
        shutil.rmtree(staging)


def prepare_learned_associative_readout(
    *,
    root: Path,
    predecessor_root: Path,
    source_train_path: Path,
    source_train_index_path: Path,
    raw_source_path: Path,
    prior_v1_prompt_population_path: Path,
    prior_v2_prompt_population_path: Path,
    prior_v3_prompt_population_path: Path,
) -> CampaignPreparation:
    """Create V4 and its disjoint continuation, then seal both exactly once."""

    root = root.resolve()
    predecessor_root = predecessor_root.resolve()
    if root.exists() or root.is_symlink():
        raise FileExistsError("learned-associative campaign root is create-once")
    predecessor, artifact = _verify_predecessor(predecessor_root)
    if (
        FRESH_HELDOUT_TOKENS != FRESH_HELDOUT_WINDOWS * (CONTEXT + 1)
        or FRESH_HELDOUT_DECISIONS != 247_920
        or FRESH_HELDOUT_REACHABLE_DECISIONS != 245_854
    ):
        raise RuntimeError("fresh-heldout arithmetic differs from the freeze")
    heldout_payload = _read_u16_slice(
        source_train_path,
        offset_tokens=FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
        token_count=FRESH_HELDOUT_TOKENS,
    )
    if cid_bytes(heldout_payload) != FRESH_HELDOUT_CID:
        raise ValueError("fresh-heldout continuation differs from the freeze")
    index_record, heldout_story_cids = _verify_fresh_heldout_index(source_train_index_path)
    excluded_story_cids = load_required_prior_story_cids(
        prior_v1_prompt_population_path,
        prior_v2_prompt_population_path,
        prior_v3_prompt_population_path,
    )
    population = select_prompt_conditioning_population_from_source(
        raw_source_path,
        predecessor.tokenizer_path,
        excluded_story_cids=excluded_story_cids,
    )
    prompt_story_cids = frozenset(
        record.story_cid
        for pair in population.pairs
        for record in (pair.left, pair.right)
    )
    if (
        population.last_source_story_ordinal >= FRESH_HELDOUT_FIRST_SOURCE_STORY
        or prompt_story_cids & heldout_story_cids
    ):
        raise ValueError("V4 prompt population overlaps the fresh-heldout stories")

    implementation = trainer_implementation_contract()
    root.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{root.name}.preparing-", dir=root.parent))
    try:
        stage_prompt_conditioning_population(staging, population)
        atomic_write(staging / HELDOUT_RELATIVE_PATH, heldout_payload)
        commitment = seal_prompt_conditioning_population(
            staging,
            population,
            heldout_relative_path=HELDOUT_RELATIVE_PATH,
            heldout_bytes=len(heldout_payload),
            heldout_cid=FRESH_HELDOUT_CID,
        )
        body = {
            "schema": PREPARATION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "implementation": implementation,
            "predecessor": {
                "root": str(predecessor_root),
                "policy": PREDECESSOR_POLICY,
                "preparation_manifest_cid": PREDECESSOR_PREPARATION_MANIFEST_CID,
                "result_cid": PREDECESSOR_RESULT_CID,
                "arm_result_cid": PREDECESSOR_ARM_RESULT_CID,
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
                "reachable_decisions": TRAIN_DECISIONS - TRAIN_WINDOWS,
                "batch_size": BATCH_SIZE,
                "optimizer_steps": OPTIMIZER_STEPS,
                "seed": INITIALIZATION_SEED,
                "optimizer": {
                    "name": "AdamW",
                    "betas": [ADAM_BETA1, ADAM_BETA2],
                    "epsilon": ADAM_EPSILON,
                    "weight_decay": WEIGHT_DECAY,
                    "gradient_clip": GRADIENT_CLIP,
                    "warmup_steps": WARMUP_STEPS,
                    "maximum_learning_rate": MAXIMUM_LEARNING_RATE,
                    "minimum_learning_rate": MINIMUM_LEARNING_RATE,
                    "schedule": "linear warmup then cosine decay",
                },
                "arms": list(ARMS),
                "trajectories_per_arm": 1,
                "retry": "FORBIDDEN",
            },
            "model": {
                "query_shape": list(QUERY_SHAPE),
                "head_parameters_per_arm": HEAD_PARAMETER_COUNT,
                "effective_parameters_per_arm": EFFECTIVE_ARM_PARAMETER_COUNT,
                "persistent_state_values": STATE_VALUES,
                "persistent_state_bytes_f32": STATE_BYTES_F32,
                "validity_bits": VALIDITY_BITS,
                "base_frozen": True,
                "heads_disjoint": True,
                "initialization": "INDEPENDENT_EXACT_ZERO",
            },
            "fresh_heldout": {
                "path": HELDOUT_RELATIVE_PATH,
                "source_path": str(source_train_path.resolve()),
                "source_offset_tokens": FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
                "tokens": FRESH_HELDOUT_TOKENS,
                "windows": FRESH_HELDOUT_WINDOWS,
                "decisions": FRESH_HELDOUT_DECISIONS,
                "reachable_decisions": FRESH_HELDOUT_REACHABLE_DECISIONS,
                "bytes": len(heldout_payload),
                "cid": FRESH_HELDOUT_CID,
                "train_index": index_record,
            },
            "prompt_population": {
                "policy": "R4RetainedPromptSwapContrastV4",
                "commitment_cid": commitment["commitment_cid"],
                "population_cid": population.population_cid,
                "last_source_story_ordinal": population.last_source_story_ordinal,
                "eligible_stories_examined": population.eligible_stories_examined,
                "prior_population_paths": {
                    "v1": str(prior_v1_prompt_population_path.resolve()),
                    "v2": str(prior_v2_prompt_population_path.resolve()),
                    "v3": str(prior_v3_prompt_population_path.resolve()),
                },
                "prior_story_cids": len(excluded_story_cids),
                "fresh_story_cids": len(heldout_story_cids),
                "prompt_fresh_story_overlap": 0,
                "access": "SEALED_MODE_000",
                "reveal_after": "V1, geometric, and pooled artifact CIDs are fixed",
            },
            "scope": {
                "cuda": "FORBIDDEN",
                "generation": "NOT_RUN",
                "reasoning": "NOT_RUN",
                "lowering": "NOT_RUN",
                "geometry_native_lowering": "NOT_RUN",
                "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
            },
        }
        manifest = _with_cid(body, "preparation_cid")
        _write_exclusive_json(staging / PREPARATION_RELATIVE_PATH, manifest)
        if root.exists() or root.is_symlink():
            raise FileExistsError("campaign root appeared during preparation")
        staging.rename(root)
    except BaseException:
        _cleanup_staging(staging)
        raise
    return load_learned_associative_readout_preparation(root)


def _load_commitment_in_any_state(root: Path) -> dict[str, Any]:
    if (root / REVEAL_RELATIVE_PATH).exists():
        load_revealed_prompt_conditioning_population(root)
        commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
        _verify_self_cid(commitment, "commitment_cid")
        return commitment
    return load_prompt_conditioning_commitment(root)


def load_learned_associative_readout_preparation(root: Path) -> CampaignPreparation:
    """Verify the public freeze without opening either sealed artifact."""

    if root.is_symlink() or not root.is_dir():
        raise ValueError("learned-associative root must be a regular directory")
    root = root.resolve()
    manifest = _read_json(root / PREPARATION_RELATIVE_PATH)
    _verify_self_cid(manifest, "preparation_cid")
    if (
        manifest.get("schema") != PREPARATION_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("policy") != POLICY
        or manifest.get("model_policy") != MODEL_POLICY
        or manifest.get("implementation") != trainer_implementation_contract()
    ):
        raise ValueError("learned-associative preparation envelope differs")
    predecessor_record = manifest.get("predecessor")
    fresh_record = manifest.get("fresh_heldout")
    prompt_record = manifest.get("prompt_population")
    if not all(isinstance(value, Mapping) for value in (predecessor_record, fresh_record, prompt_record)):
        raise ValueError("learned-associative preparation records are malformed")
    predecessor, artifact = _verify_predecessor(Path(str(predecessor_record["root"])))
    if (
        predecessor_record.get("preparation_manifest_cid") != PREDECESSOR_PREPARATION_MANIFEST_CID
        or predecessor_record.get("result_cid") != PREDECESSOR_RESULT_CID
        or predecessor_record.get("arm_result_cid") != PREDECESSOR_ARM_RESULT_CID
        or predecessor_record.get("artifact")
        != {"path": str(artifact), "bytes": PREDECESSOR_ARTIFACT_BYTES, "cid": PREDECESSOR_ARTIFACT_CID}
    ):
        raise ValueError("learned-associative predecessor binding differs")
    expected_fresh = {
        "path": HELDOUT_RELATIVE_PATH,
        "bytes": FRESH_HELDOUT_TOKENS * 2,
        "cid": FRESH_HELDOUT_CID,
    }
    if any(fresh_record.get(key) != value for key, value in expected_fresh.items()):
        raise ValueError("learned-associative fresh-heldout binding differs")
    commitment = _load_commitment_in_any_state(root)
    committed_fresh = commitment.get("fresh_heldout")
    if (
        prompt_record.get("commitment_cid") != commitment.get("commitment_cid")
        or prompt_record.get("population_cid") != commitment.get("population", {}).get("cid")
        or committed_fresh != expected_fresh
        or prompt_record.get("prior_story_cids") != 1_536
        or prompt_record.get("prompt_fresh_story_overlap") != 0
    ):
        raise ValueError("learned-associative sealed commitment differs")
    return CampaignPreparation(
        root=root,
        manifest=manifest,
        predecessor=predecessor,
        predecessor_artifact_path=artifact,
        commitment=commitment,
    )


def _new_model(preparation: CampaignPreparation, device: torch.device) -> R4LearnedCandidateLeafAssociativeReadoutV1:
    model = R4LearnedCandidateLeafAssociativeReadoutV1(
        _exact_geometry(preparation.predecessor)
    )
    model.load_qualified_base_artifact(preparation.predecessor_artifact_path.read_bytes())
    return model.to(device)


def _select_trainable_head(
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    arm: Literal["geometric", "pooled"],
) -> tuple[torch.nn.Parameter, ...]:
    own = tuple(model.head_parameters(arm))
    peer = "pooled" if arm == "geometric" else "geometric"
    for parameter in model.head_parameters(peer):
        parameter.requires_grad_(False)
    for parameter in model.frozen_base_parameters():
        parameter.requires_grad_(False)
    for parameter in own:
        parameter.requires_grad_(True)
    if (
        len(own) != 1
        or sum(parameter.numel() for parameter in own) != HEAD_PARAMETER_COUNT
        or {id(parameter) for parameter in own}
        & {id(parameter) for parameter in model.head_parameters(peer)}
    ):
        raise RuntimeError("learned-associative trainable-head partition differs")
    return own


def _head_optimizer(parameters: Sequence[torch.nn.Parameter]) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        parameters,
        lr=learning_rate(0),
        betas=(ADAM_BETA1, ADAM_BETA2),
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
    )


def _train_step(
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    arm: Literal["geometric", "pooled"],
    optimizer: torch.optim.Optimizer,
    parameters: Sequence[torch.nn.Parameter],
    batch: Tensor,
    *,
    step: int,
) -> tuple[float, float]:
    model.train()
    optimizer.zero_grad(set_to_none=True)
    output = model.forward_arm(arm, batch[:, :-1], batch[:, 1:])
    if output.loss is None or not bool(torch.isfinite(output.loss)):
        raise RuntimeError(f"{arm} training loss is nonfinite")
    output.loss.backward()
    if any(parameter.grad is None for parameter in parameters):
        raise RuntimeError(f"{arm} trainable head did not receive a gradient")
    gradient_norm = torch.nn.utils.clip_grad_norm_(parameters, GRADIENT_CLIP)
    if not bool(torch.isfinite(gradient_norm)):
        raise RuntimeError(f"{arm} gradient norm is nonfinite")
    rate = learning_rate(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    optimizer.step()
    return float(output.loss.detach().cpu()), float(gradient_norm.detach().cpu())


def _shared_train_step(
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    optimizers: Mapping[str, torch.optim.Optimizer],
    parameters: Mapping[str, Sequence[torch.nn.Parameter]],
    batch: Tensor,
    *,
    step: int,
) -> tuple[dict[str, float], dict[str, float]]:
    """Fit both disjoint heads from one shared frozen-backbone feature pass."""

    model.train()
    for optimizer in optimizers.values():
        optimizer.zero_grad(set_to_none=True)
    bundle = model(batch[:, :-1], batch[:, 1:])
    losses = {"geometric": bundle.geometric.loss, "pooled": bundle.pooled.loss}
    if any(loss is None or not bool(torch.isfinite(loss)) for loss in losses.values()):
        raise RuntimeError("shared learned-associative training loss is nonfinite")
    assert losses["geometric"] is not None and losses["pooled"] is not None
    (losses["geometric"] + losses["pooled"]).backward()
    norms: dict[str, float] = {}
    for arm in ARMS:
        if any(parameter.grad is None for parameter in parameters[arm]):
            raise RuntimeError(f"shared {arm} head did not receive a gradient")
        norm = torch.nn.utils.clip_grad_norm_(parameters[arm], GRADIENT_CLIP)
        if not bool(torch.isfinite(norm)):
            raise RuntimeError(f"shared {arm} gradient norm is nonfinite")
        norms[arm] = float(norm.detach().cpu())
        for group in optimizers[arm].param_groups:
            group["lr"] = learning_rate(step)
        optimizers[arm].step()
    return (
        {arm: float(losses[arm].detach().cpu()) for arm in ARMS},
        norms,
    )


def _probe_vector(model: R4LearnedCandidateLeafAssociativeReadoutV1, logits: Tensor, arm: str) -> list[float]:
    values = logits.detach().float().reshape(-1)[:64].cpu().tolist()
    parameter = next(iter(model.head_parameters(arm)))
    values.extend(parameter.detach().float().reshape(-1)[:64].cpu().tolist())
    return values


def _probe_arm(root: Path, arm: Literal["geometric", "pooled"], plan: ExecutionPlan) -> dict[str, Any]:
    """Exercise one head without loading the prompt or fresh-heldout files."""

    device, backend = _configure_device(plan)
    preparation = load_learned_associative_readout_preparation(root)
    model = _new_model(preparation, device)
    parameters = _select_trainable_head(model, arm)
    peer = "pooled" if arm == "geometric" else "geometric"
    base_before = model.export_qualified_base_artifact()
    own_before = model.export_head_artifact(arm)
    peer_before = model.export_head_artifact(peer)
    batch = _ordered_train_batch(preparation.predecessor, 1, device)

    baseline = R4RetainedLanguagePathV1(_exact_geometry(preparation.predecessor)).to(device)
    baseline.load_learned_artifact(base_before)
    model.eval()
    baseline.eval()
    with torch.no_grad():
        zero = model.forward_arm(arm, batch[:1, :-1])
        base = baseline(batch[:1, :-1])
        head_off = model.forward_arm(arm, batch[:1, :-1], head_off=True)
        state_off = model.forward_arm(arm, batch[:1, :-1], attention_off=True)
        state_off_head_off = model.forward_arm(
            arm, batch[:1, :-1], attention_off=True, head_off=True
        )
        direct = model.forward_arm(arm, batch[:1, :-1], implementation="direct")
        prefix = model.forward_arm(arm, batch[:1, :63])
    initialization_delta = float((zero.logits - base.logits).abs().max().cpu())
    head_off_delta = float((head_off.logits - base.logits).abs().max().cpu())
    state_off_delta = float((state_off.logits - state_off_head_off.logits).abs().max().cpu())
    direct_delta = float((zero.logits - direct.logits).abs().max().cpu())
    causal_delta = float((zero.logits[:, :63] - prefix.logits).abs().max().cpu())
    work_equal = (
        zero.audit.work_signature() == head_off.audit.work_signature()
        == state_off.audit.work_signature() == state_off_head_off.audit.work_signature()
    )
    del baseline, base, head_off, state_off, state_off_head_off, direct, prefix

    optimizer = _head_optimizer(parameters)
    measured: list[float] = []
    final_loss = math.nan
    final_gradient_norm = math.nan
    for offset in range(PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS):
        _sync(device)
        started = time.perf_counter()
        train_batch = _ordered_train_batch(preparation.predecessor, offset + 1, device)
        final_loss, final_gradient_norm = _train_step(
            model, arm, optimizer, parameters, train_batch, step=offset + 1
        )
        _sync(device)
        elapsed = time.perf_counter() - started
        if offset >= PROBE_WARMUP_STEPS:
            measured.append(elapsed)

    evaluation_batch = _ordered_train_batch(
        preparation.predecessor, PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS + 1, device
    )
    model.eval()
    _sync(device)
    evaluation_started = time.perf_counter()
    with torch.no_grad():
        evaluation = model.forward_arm(arm, evaluation_batch[:, :-1], evaluation_batch[:, 1:])
    _sync(device)
    evaluation_seconds = time.perf_counter() - evaluation_started
    if evaluation.loss is None or int(evaluation.audit.forbidden_reads) != 0:
        raise RuntimeError(f"{arm} training-only probe evaluation audit failed")

    active_baseline = R4RetainedLanguagePathV1(
        _exact_geometry(preparation.predecessor)
    ).to(device)
    active_baseline.load_learned_artifact(base_before)
    active_baseline.eval()
    with torch.no_grad():
        active_head_off = model.forward_arm(
            arm, evaluation_batch[:, :-1], head_off=True
        )
        active_state_off = model.forward_arm(
            arm, evaluation_batch[:, :-1], attention_off=True
        )
        active_state_off_head_off = model.forward_arm(
            arm,
            evaluation_batch[:, :-1],
            attention_off=True,
            head_off=True,
        )
        active_direct = model.forward_arm(
            arm, evaluation_batch[:, :-1], implementation="direct"
        )
        active_prefix = model.forward_arm(arm, evaluation_batch[:, :63])
        active_base = active_baseline(evaluation_batch[:, :-1])
    active_head_effect = float(
        (evaluation.logits - active_head_off.logits).abs().max().cpu()
    )
    active_head_off_delta = float(
        (active_head_off.logits - active_base.logits).abs().max().cpu()
    )
    active_state_off_delta = float(
        (active_state_off.logits - active_state_off_head_off.logits)
        .abs()
        .max()
        .cpu()
    )
    active_direct_delta = float(
        (evaluation.logits - active_direct.logits).abs().max().cpu()
    )
    active_causal_delta = float(
        (evaluation.logits[:, :63] - active_prefix.logits).abs().max().cpu()
    )
    active_work_equal = (
        evaluation.audit.work_signature()
        == active_head_off.audit.work_signature()
        == active_state_off.audit.work_signature()
        == active_state_off_head_off.audit.work_signature()
    )

    artifact_started = time.perf_counter()
    artifact = model.export_head_artifact(arm)
    artifact_seconds = time.perf_counter() - artifact_started
    replay = _new_model(preparation, device)
    replay.load_head_artifact(arm, artifact)
    replay.eval()
    with torch.no_grad():
        replay_output = replay.forward_arm(arm, evaluation_batch[:, :-1])
    replay_delta = float((evaluation.logits - replay_output.logits).abs().max().cpu())
    base_unchanged = model.export_qualified_base_artifact() == base_before
    peer_unchanged = model.export_head_artifact(peer) == peer_before
    own_changed = artifact != own_before
    if device.type == "mps":
        peak_bytes = max(
            int(torch.mps.current_allocated_memory()),
            int(torch.mps.driver_allocated_memory()),
        )
    else:
        peak_bytes = _peak_rss_bytes()
    mechanics = {
        "zero_head_v1_maximum_logits_delta": initialization_delta,
        "head_off_v1_maximum_logits_delta": head_off_delta,
        "state_off_head_maximum_logits_delta": state_off_delta,
        "stationary_direct_maximum_logits_delta": direct_delta,
        "strict_causal_prefix_maximum_logits_delta": causal_delta,
        "artifact_replay_maximum_logits_delta": replay_delta,
        "active_head_effect_maximum_logits_delta": active_head_effect,
        "active_head_off_v1_maximum_logits_delta": active_head_off_delta,
        "active_state_off_head_maximum_logits_delta": active_state_off_delta,
        "active_stationary_direct_maximum_logits_delta": active_direct_delta,
        "active_strict_causal_prefix_maximum_logits_delta": active_causal_delta,
        "active_work_signatures_equal": active_work_equal,
        "base_artifact_unchanged": base_unchanged,
        "peer_head_unchanged": peer_unchanged,
        "own_head_changed": own_changed,
        "work_signatures_equal": work_equal,
        "forbidden_reads": int(evaluation.audit.forbidden_reads),
    }
    mechanics["passed"] = bool(
        initialization_delta == 0.0
        and head_off_delta == 0.0
        and state_off_delta == 0.0
        and direct_delta <= 2e-5
        and causal_delta <= 2e-5
        and replay_delta == 0.0
        and active_head_effect > 0.0
        and active_head_off_delta == 0.0
        and active_state_off_delta == 0.0
        and active_direct_delta <= 2e-5
        and active_causal_delta <= 2e-5
        and active_work_equal
        and base_unchanged
        and peer_unchanged
        and own_changed
        and work_equal
        and mechanics["forbidden_reads"] == 0
    )
    return {
        "arm": arm,
        "backend": backend,
        "mean_train_step_seconds": statistics.fmean(measured),
        "measured_train_step_seconds": measured,
        "evaluation_batch_seconds": evaluation_seconds,
        "artifact_export_seconds": artifact_seconds,
        "peak_memory_bytes": peak_bytes,
        "memory_budget_bytes": int(backend["memory_budget_bytes"]),
        "final_probe_train_loss": final_loss,
        "final_probe_gradient_norm": final_gradient_norm,
        "probe_vector": _probe_vector(model, evaluation.logits, arm),
        "mechanics": mechanics,
        "sealed_prompt_reads": 0,
        "sealed_heldout_reads": 0,
    }


def _probe_shared(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    """Benchmark both heads through one shared backbone for sequential plans."""

    if plan.concurrent_arms:
        raise ValueError("shared probe requires a sequential execution plan")
    device, backend = _configure_device(plan)
    preparation = load_learned_associative_readout_preparation(root)
    model = _new_model(preparation, device)
    parameters = {
        arm: tuple(model.head_parameters(arm))
        for arm in ARMS
    }
    if {id(value) for value in parameters["geometric"]} & {
        id(value) for value in parameters["pooled"]
    }:
        raise RuntimeError("shared probe head parameters overlap")
    for parameter in model.frozen_base_parameters():
        parameter.requires_grad_(False)
    optimizers = {arm: _head_optimizer(parameters[arm]) for arm in ARMS}
    head_parameter_storage_disjoint = all(
        left.data_ptr() != right.data_ptr()
        for left in parameters["geometric"]
        for right in parameters["pooled"]
    )
    optimizer_parameter_sets = {
        arm: {
            id(parameter)
            for group in optimizers[arm].param_groups
            for parameter in group["params"]
        }
        for arm in ARMS
    }
    optimizer_parameter_sets_disjoint = not (
        optimizer_parameter_sets["geometric"] & optimizer_parameter_sets["pooled"]
    )
    if not head_parameter_storage_disjoint or not optimizer_parameter_sets_disjoint:
        raise RuntimeError("shared probe optimizer/head storage overlaps")
    base_before = model.export_qualified_base_artifact()
    head_before = {arm: model.export_head_artifact(arm) for arm in ARMS}
    batch = _ordered_train_batch(preparation.predecessor, 1, device)
    baseline = R4RetainedLanguagePathV1(_exact_geometry(preparation.predecessor)).to(device)
    baseline.load_learned_artifact(base_before)
    model.eval()
    baseline.eval()
    with torch.no_grad():
        zero = model(batch[:1, :-1])
        base = baseline(batch[:1, :-1])
        head_off = model(batch[:1, :-1], head_off=True)
        state_off = model(batch[:1, :-1], attention_off=True)
        state_off_head_off = model(
            batch[:1, :-1], attention_off=True, head_off=True
        )
        direct = model(batch[:1, :-1], implementation="direct")
        prefix = model(batch[:1, :63])
    mechanics: dict[str, dict[str, Any]] = {}
    cross_arm_work_signatures_equal = (
        zero.geometric.audit.work_signature()
        == zero.pooled.audit.work_signature()
    )
    for arm in ARMS:
        zero_output = getattr(zero, arm)
        head_off_output = getattr(head_off, arm)
        state_output = getattr(state_off, arm)
        state_off_output = getattr(state_off_head_off, arm)
        direct_output = getattr(direct, arm)
        prefix_output = getattr(prefix, arm)
        initialization_delta = float((zero_output.logits - base.logits).abs().max().cpu())
        head_off_delta = float((head_off_output.logits - base.logits).abs().max().cpu())
        state_delta = float((state_output.logits - state_off_output.logits).abs().max().cpu())
        direct_delta = float((zero_output.logits - direct_output.logits).abs().max().cpu())
        causal_delta = float((zero_output.logits[:, :63] - prefix_output.logits).abs().max().cpu())
        work_equal = (
            zero_output.audit.work_signature()
            == head_off_output.audit.work_signature()
            == state_output.audit.work_signature()
            == state_off_output.audit.work_signature()
        )
        mechanics[arm] = {
            "zero_head_v1_maximum_logits_delta": initialization_delta,
            "head_off_v1_maximum_logits_delta": head_off_delta,
            "state_off_head_maximum_logits_delta": state_delta,
            "stationary_direct_maximum_logits_delta": direct_delta,
            "strict_causal_prefix_maximum_logits_delta": causal_delta,
            "work_signatures_equal": work_equal,
            "cross_arm_work_signatures_equal": cross_arm_work_signatures_equal,
            "head_parameter_storage_disjoint": head_parameter_storage_disjoint,
            "optimizer_parameter_sets_disjoint": optimizer_parameter_sets_disjoint,
            "forbidden_reads": int(zero_output.audit.forbidden_reads),
            "passed": initialization_delta == 0.0
            and head_off_delta == 0.0
            and state_delta == 0.0
            and direct_delta <= 2e-5
            and causal_delta <= 2e-5
            and work_equal
            and cross_arm_work_signatures_equal
            and head_parameter_storage_disjoint
            and optimizer_parameter_sets_disjoint
            and int(zero_output.audit.forbidden_reads) == 0,
        }
    del baseline, base, head_off, state_off, state_off_head_off, direct, prefix

    measured: list[float] = []
    losses: dict[str, float] = {arm: math.nan for arm in ARMS}
    norms: dict[str, float] = {arm: math.nan for arm in ARMS}
    for offset in range(PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS):
        _sync(device)
        started = time.perf_counter()
        train_batch = _ordered_train_batch(preparation.predecessor, offset + 1, device)
        losses, norms = _shared_train_step(
            model, optimizers, parameters, train_batch, step=offset + 1
        )
        _sync(device)
        elapsed = time.perf_counter() - started
        if offset >= PROBE_WARMUP_STEPS:
            measured.append(elapsed)
    evaluation_batch = _ordered_train_batch(
        preparation.predecessor, PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS + 1, device
    )
    model.eval()
    _sync(device)
    evaluation_started = time.perf_counter()
    with torch.no_grad():
        evaluation = model(evaluation_batch[:, :-1], evaluation_batch[:, 1:])
    _sync(device)
    evaluation_seconds = time.perf_counter() - evaluation_started
    active_baseline = R4RetainedLanguagePathV1(
        _exact_geometry(preparation.predecessor)
    ).to(device)
    active_baseline.load_learned_artifact(base_before)
    active_baseline.eval()
    with torch.no_grad():
        active_head_off = model(evaluation_batch[:, :-1], head_off=True)
        active_state_off = model(evaluation_batch[:, :-1], attention_off=True)
        active_state_off_head_off = model(
            evaluation_batch[:, :-1], attention_off=True, head_off=True
        )
        active_direct = model(
            evaluation_batch[:, :-1], implementation="direct"
        )
        active_prefix = model(evaluation_batch[:, :63])
        active_base = active_baseline(evaluation_batch[:, :-1])
    active_cross_arm_work_equal = (
        evaluation.geometric.audit.work_signature()
        == evaluation.pooled.audit.work_signature()
    )
    replay_deltas: dict[str, float] = {}
    artifact_seconds: dict[str, float] = {}
    outputs: dict[str, Any] = {}
    for arm in ARMS:
        artifact_started = time.perf_counter()
        artifact = model.export_head_artifact(arm)
        artifact_seconds[arm] = time.perf_counter() - artifact_started
        replay = _new_model(preparation, device)
        replay.load_head_artifact(arm, artifact)
        replay.eval()
        with torch.no_grad():
            replay_output = replay.forward_arm(arm, evaluation_batch[:, :-1])
        output = getattr(evaluation, arm)
        active_head_off_output = getattr(active_head_off, arm)
        active_state_output = getattr(active_state_off, arm)
        active_state_off_output = getattr(active_state_off_head_off, arm)
        active_direct_output = getattr(active_direct, arm)
        active_prefix_output = getattr(active_prefix, arm)
        active_head_effect = float(
            (output.logits - active_head_off_output.logits).abs().max().cpu()
        )
        active_head_off_delta = float(
            (active_head_off_output.logits - active_base.logits).abs().max().cpu()
        )
        active_state_delta = float(
            (active_state_output.logits - active_state_off_output.logits)
            .abs()
            .max()
            .cpu()
        )
        active_direct_delta = float(
            (output.logits - active_direct_output.logits).abs().max().cpu()
        )
        active_causal_delta = float(
            (output.logits[:, :63] - active_prefix_output.logits)
            .abs()
            .max()
            .cpu()
        )
        active_work_equal = (
            output.audit.work_signature()
            == active_head_off_output.audit.work_signature()
            == active_state_output.audit.work_signature()
            == active_state_off_output.audit.work_signature()
        )
        replay_deltas[arm] = float((output.logits - replay_output.logits).abs().max().cpu())
        mechanics[arm].update(
            {
                "artifact_replay_maximum_logits_delta": replay_deltas[arm],
                "base_artifact_unchanged": model.export_qualified_base_artifact() == base_before,
                "own_head_changed": artifact != head_before[arm],
                "active_head_effect_maximum_logits_delta": active_head_effect,
                "active_head_off_v1_maximum_logits_delta": active_head_off_delta,
                "active_state_off_head_maximum_logits_delta": active_state_delta,
                "active_stationary_direct_maximum_logits_delta": active_direct_delta,
                "active_strict_causal_prefix_maximum_logits_delta": active_causal_delta,
                "active_work_signatures_equal": active_work_equal,
                "active_cross_arm_work_signatures_equal": active_cross_arm_work_equal,
            }
        )
        mechanics[arm]["passed"] = bool(
            mechanics[arm]["passed"]
            and replay_deltas[arm] == 0.0
            and mechanics[arm]["base_artifact_unchanged"]
            and mechanics[arm]["own_head_changed"]
            and active_head_effect > 0.0
            and active_head_off_delta == 0.0
            and active_state_delta == 0.0
            and active_direct_delta <= 2e-5
            and active_causal_delta <= 2e-5
            and active_work_equal
            and active_cross_arm_work_equal
        )
        outputs[arm] = output
    if device.type == "mps":
        peak_bytes = max(
            int(torch.mps.current_allocated_memory()),
            int(torch.mps.driver_allocated_memory()),
        )
    else:
        peak_bytes = _peak_rss_bytes()
    shared_step_seconds = statistics.fmean(measured)
    return {
        "plan": plan.identity(),
        "shared_feature_pass": True,
        "arms": {
            arm: {
                "ok": True,
                "result": {
                    "arm": arm,
                    "backend": backend,
                    # Two half-cost ledger entries sum to the measured shared pass.
                    "mean_train_step_seconds": shared_step_seconds / 2.0,
                    "measured_shared_train_step_seconds": measured,
                    "evaluation_batch_seconds": evaluation_seconds / 2.0,
                    "artifact_export_seconds": artifact_seconds[arm],
                    "peak_memory_bytes": peak_bytes,
                    "memory_budget_bytes": int(backend["memory_budget_bytes"]),
                    "final_probe_train_loss": losses[arm],
                    "final_probe_gradient_norm": norms[arm],
                    "probe_vector": _probe_vector(model, outputs[arm].logits, arm),
                    "mechanics": mechanics[arm],
                    "sealed_prompt_reads": 0,
                    "sealed_heldout_reads": 0,
                },
            }
            for arm in ARMS
        },
    }


def _probe_shared_worker(root: str, plan_value: Mapping[str, Any], queue: Any) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        queue.put({"ok": True, "result": _probe_shared(Path(root), plan)})
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _probe_worker(root: str, arm: str, plan_value: Mapping[str, Any], queue: Any) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        queue.put({"ok": True, "result": _probe_arm(Path(root), arm, plan)})
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _collect_worker(process: Any, queue: Any, *, timeout: float = 900.0) -> dict[str, Any]:
    process.join(timeout)
    if process.is_alive():
        process.terminate()
        process.join(10.0)
        return {"ok": False, "error": {"type": "TimeoutError", "reason": "worker timed out"}}
    try:
        message = queue.get(timeout=5.0)
    except Empty:
        return {
            "ok": False,
            "error": {
                "type": "WorkerExitError",
                "reason": f"worker exited {process.exitcode} without evidence",
            },
        }
    if not isinstance(message, dict):
        raise RuntimeError("worker returned a non-object")
    return message


def _spawned_probe_executor(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    context = mp.get_context("spawn")
    if not plan.concurrent_arms:
        queue = context.Queue()
        process = context.Process(
            target=_probe_shared_worker,
            args=(str(root), asdict(plan), queue),
            name=f"learned-associative-probe-{plan.name}-shared",
        )
        process.start()
        outcome = _collect_worker(process, queue)
        if not outcome.get("ok"):
            return {
                "plan": plan.identity(),
                "arms": {arm: dict(outcome) for arm in ARMS},
            }
        return dict(outcome["result"])
    outcomes: dict[str, Any] = {}
    active: dict[str, tuple[Any, Any]] = {}
    for arm in ARMS:
        queue = context.Queue()
        process = context.Process(
            target=_probe_worker,
            args=(str(root), arm, asdict(plan), queue),
            name=f"learned-associative-probe-{plan.name}-{arm}",
        )
        process.start()
        if plan.concurrent_arms:
            active[arm] = (process, queue)
        else:
            outcomes[arm] = _collect_worker(process, queue)
    for arm, (process, queue) in active.items():
        outcomes[arm] = _collect_worker(process, queue)
    return {"plan": plan.identity(), "arms": outcomes}


ProbeExecutor = Callable[[Path, ExecutionPlan], Mapping[str, Any]]


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Bind CPU4 equivalence, mechanics, memory, wall, and measured-fastest."""

    reference_name = "cpu-accelerate-4t-sequential"
    by_name = {str(record.get("plan", {}).get("name")): record for record in records}
    reference = by_name.get(reference_name)
    if reference is None:
        raise ValueError("CPU-four-thread reference probe is absent")
    reference_arms = reference.get("arms")
    if not isinstance(reference_arms, Mapping):
        raise ValueError("CPU-four-thread reference has no arm evidence")
    reference_available = all(
        bool(reference_arms.get(arm, {}).get("ok")) for arm in ARMS
    )
    # Final V4/fresh scoring and replay always execute once on canonical CPU4,
    # irrespective of which backend wins the training-only comparison.
    common_scoring_seconds = (
        sum(
            float(reference_arms[arm]["result"]["evaluation_batch_seconds"])
            for arm in ARMS
        )
        * CANONICAL_SCORING_EQUIVALENT_BATCHES
        if reference_available
        else math.inf
    )
    projected: list[dict[str, Any]] = []
    for raw in records:
        record = dict(raw)
        plan = record.get("plan", {})
        arms = record.get("arms", {})
        available = isinstance(arms, Mapping) and all(
            bool(arms.get(arm, {}).get("ok")) for arm in ARMS
        )
        deltas: dict[str, float | None] = {}
        equivalent = available
        mechanics = available
        for arm in ARMS:
            observed = arms.get(arm, {}) if isinstance(arms, Mapping) else {}
            expected = reference_arms.get(arm, {})
            if not observed.get("ok") or not expected.get("ok"):
                deltas[arm] = None
                equivalent = False
                mechanics = False
                continue
            vector = observed["result"].get("probe_vector", [])
            reference_vector = expected["result"].get("probe_vector", [])
            if not vector or len(vector) != len(reference_vector):
                deltas[arm] = None
                equivalent = False
            else:
                delta = max(abs(float(left) - float(right)) for left, right in zip(vector, reference_vector, strict=True))
                deltas[arm] = delta
                equivalent = equivalent and delta <= EQUIVALENCE_ABS_TOLERANCE
            mechanics = mechanics and bool(observed["result"].get("mechanics", {}).get("passed"))
            mechanics = mechanics and observed["result"].get("sealed_prompt_reads") == 0
            mechanics = mechanics and observed["result"].get("sealed_heldout_reads") == 0
        per_arm_seconds = {
            arm: float(arms[arm]["result"]["mean_train_step_seconds"]) * OPTIMIZER_STEPS
            + float(arms[arm]["result"]["artifact_export_seconds"])
            if available
            else math.inf
            for arm in ARMS
        }
        training_seconds = (
            max(per_arm_seconds.values())
            if bool(plan.get("concurrent_arms"))
            else sum(per_arm_seconds.values())
        )
        raw_seconds = training_seconds + common_scoring_seconds
        projected_seconds = raw_seconds * PROJECTION_SAFETY_FACTOR
        peak_bytes = (
            sum(int(arms[arm]["result"]["peak_memory_bytes"]) for arm in ARMS)
            if available and bool(plan.get("concurrent_arms"))
            else max(
                (int(arms[arm]["result"]["peak_memory_bytes"]) for arm in ARMS),
                default=0,
            )
            if available
            else 0
        )
        memory_budget = (
            min(int(arms[arm]["result"]["memory_budget_bytes"]) for arm in ARMS)
            if available
            else 0
        )
        memory_fraction = peak_bytes / memory_budget if memory_budget else math.inf
        eligible = bool(
            available
            and equivalent
            and mechanics
            and projected_seconds <= HARD_WALL_CEILING_SECONDS
            and memory_fraction <= MEMORY_FRACTION_CEILING
        )
        record["equivalence"] = {
            "reference_plan": reference_name,
            "absolute_tolerance": EQUIVALENCE_ABS_TOLERANCE,
            "maximum_deltas": deltas,
            "passed": equivalent,
        }
        record["projection"] = {
            "eligible": eligible,
            "per_arm_seconds": per_arm_seconds,
            "plan_specific_training_seconds": training_seconds,
            "common_canonical_cpu4_scoring_seconds": common_scoring_seconds,
            "common_canonical_cpu4_scoring_equivalent_batches": (
                CANONICAL_SCORING_EQUIVALENT_BATCHES
            ),
            "raw_aggregate_seconds": raw_seconds,
            "safety_factor": PROJECTION_SAFETY_FACTOR,
            "projected_aggregate_seconds": projected_seconds,
            "wall_ceiling_seconds": HARD_WALL_CEILING_SECONDS,
            "peak_memory_bytes": peak_bytes,
            "memory_budget_bytes": memory_budget,
            "memory_fraction": memory_fraction,
            "memory_fraction_ceiling": MEMORY_FRACTION_CEILING,
            "mechanics_passed": mechanics,
            "reason": "ELIGIBLE" if eligible else "PROBE_UNAVAILABLE" if not available else "EQUIVALENCE" if not equivalent else "MECHANICS" if not mechanics else "WALL" if projected_seconds > HARD_WALL_CEILING_SECONDS else "MEMORY",
        }
        projected.append(record)
    eligible_records = [record for record in projected if record["projection"]["eligible"]]
    selected = min(
        eligible_records,
        key=lambda record: (
            float(record["projection"]["projected_aggregate_seconds"]),
            str(record["plan"]["name"]),
        ),
        default=None,
    )
    return {
        "plans": projected,
        "selected_plan": selected["plan"] if selected is not None else None,
        "selected_projection": selected["projection"] if selected is not None else None,
        "available": selected is not None,
    }


def _probe_learned_associative_readout(root: Path, *, executor: ProbeExecutor) -> dict[str, Any]:
    root = root.resolve()
    path = root / PROBE_RELATIVE_PATH
    if path.exists():
        probe = _read_json(path)
        _verify_self_cid(probe, "probe_cid")
        preparation = _read_json(root / PREPARATION_RELATIVE_PATH)
        _verify_self_cid(preparation, "preparation_cid")
        commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
        _verify_self_cid(commitment, "commitment_cid")
        contract = {
            "plans": [plan.identity() for plan in ELIGIBLE_PLANS],
            "warmup_steps_per_arm": PROBE_WARMUP_STEPS,
            "measured_steps_per_arm": PROBE_MEASURED_STEPS,
            "training_population_only": True,
            "prompt_population_reads": 0,
            "fresh_heldout_reads": 0,
            "cuda": "FORBIDDEN",
            "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
        }
        selection = probe.get("selection")
        if (
            probe.get("schema") != PROBE_SCHEMA
            or probe.get("issue") != ISSUE
            or probe.get("policy") != POLICY
            or probe.get("preparation_cid") != preparation.get("preparation_cid")
            or probe.get("prompt_commitment_cid") != commitment.get("commitment_cid")
            or probe.get("implementation") != trainer_implementation_contract()
            or probe.get("contract") != contract
            or not isinstance(selection, Mapping)
            or select_execution_plan(selection.get("plans", [])) != selection
            or probe.get("eligible") is not selection.get("available")
            or probe.get("verdict")
            != (
                "LEARNED_ASSOCIATIVE_EXECUTION_ADMITTED"
                if selection.get("available")
                else TERMINAL_UNAVAILABLE
            )
        ):
            raise ValueError("existing learned-associative execution probe differs")
        return probe
    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("a missing execution probe cannot be created after reveal")
    preparation = load_learned_associative_readout_preparation(root)
    implementation = trainer_implementation_contract()
    records = [dict(executor(root, plan)) for plan in ELIGIBLE_PLANS]
    if implementation != trainer_implementation_contract():
        raise ValueError("trainer implementation changed during execution probe")
    selection = select_execution_plan(records)
    body = {
        "schema": PROBE_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation.manifest["preparation_cid"],
        "prompt_commitment_cid": preparation.commitment["commitment_cid"],
        "implementation": implementation,
        "contract": {
            "plans": [plan.identity() for plan in ELIGIBLE_PLANS],
            "warmup_steps_per_arm": PROBE_WARMUP_STEPS,
            "measured_steps_per_arm": PROBE_MEASURED_STEPS,
            "training_population_only": True,
            "prompt_population_reads": 0,
            "fresh_heldout_reads": 0,
            "cuda": "FORBIDDEN",
            "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
        },
        "selection": selection,
        "eligible": selection["available"],
        "verdict": "LEARNED_ASSOCIATIVE_EXECUTION_ADMITTED" if selection["available"] else TERMINAL_UNAVAILABLE,
    }
    probe = _with_cid(body, "probe_cid")
    _write_exclusive_json(path, probe)
    return probe


def probe_learned_associative_readout(root: Path) -> dict[str, Any]:
    return _probe_learned_associative_readout(root, executor=_spawned_probe_executor)


def _arm_directory(root: Path, arm: str) -> Path:
    if arm not in ARMS:
        raise ValueError(f"unknown learned-associative arm: {arm}")
    return root / "arms" / arm


def _checkpoint_path(root: Path, arm: str) -> Path:
    return _arm_directory(root, arm) / "checkpoint.pt"


def _shared_checkpoint_path(root: Path) -> Path:
    return root / "run" / "shared-heads-checkpoint.pt"


def _shared_checkpoint_cid_path(root: Path) -> Path:
    return root / "run" / "shared-heads-checkpoint.pt.cid.json"


def _checkpoint_cid_path(root: Path, arm: str) -> Path:
    return _arm_directory(root, arm) / "checkpoint.pt.cid.json"


def _progress_path(root: Path, arm: str) -> Path:
    return _arm_directory(root, arm) / "progress.json"


def _arm_result_path(root: Path, arm: str) -> Path:
    return _arm_directory(root, arm) / "result.json"


def _artifact_path(root: Path, arm: str) -> Path:
    return _arm_directory(root, arm) / "head.safetensors"


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
    arm: Literal["geometric", "pooled"],
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    optimizer: torch.optim.Optimizer,
    step: int,
    elapsed_seconds: float,
    run_contract_cid: str,
    plan_cid: str,
    last_loss: float | None,
) -> dict[str, Any]:
    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("checkpoint mutation is forbidden after outcome reveal")
    path = _checkpoint_path(root, arm)
    parameter = next(iter(model.head_parameters(arm)))
    head_artifact_cid = cid_bytes(model.export_head_artifact(arm))
    checkpoint = {
        "schema": CHECKPOINT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "step": step,
        "elapsed_seconds": elapsed_seconds,
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "head": parameter.detach().cpu(),
        "head_artifact_cid": head_artifact_cid,
        "optimizer": optimizer.state_dict(),
        "cpu_rng_state": torch.get_rng_state(),
        "last_loss": last_loss,
    }
    _atomic_torch_save(path, checkpoint)
    sidecar = _with_cid(
        {
            "schema": "uor-r4.learned-associative-readout-checkpoint-cid/1",
            "issue": ISSUE,
            "policy": POLICY,
            "arm": arm,
            "step": step,
            "bytes": path.stat().st_size,
            "checkpoint_cid": cid_file(path),
            "head_artifact_cid": head_artifact_cid,
            "run_contract_cid": run_contract_cid,
            "plan_cid": plan_cid,
        },
        "sidecar_cid",
    )
    atomic_write_json(_checkpoint_cid_path(root, arm), sidecar)
    return sidecar


def _load_checkpoint(
    root: Path,
    *,
    arm: Literal["geometric", "pooled"],
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    run_contract_cid: str,
    plan_cid: str,
) -> dict[str, Any]:
    path = _checkpoint_path(root, arm)
    # The atomically replaced torch envelope is authoritative.  The JSON
    # sidecar is a regenerable index, so interruption between the two writes
    # cannot strand the sole trajectory.
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    if (
        not isinstance(checkpoint, dict)
        or checkpoint.get("schema") != CHECKPOINT_SCHEMA
        or checkpoint.get("arm") != arm
        or checkpoint.get("run_contract_cid") != run_contract_cid
        or checkpoint.get("plan_cid") != plan_cid
    ):
        raise ValueError(f"{arm} checkpoint envelope differs")
    step = checkpoint["step"]
    head = checkpoint.get("head")
    if (
        isinstance(step, bool)
        or not isinstance(step, int)
        or not 0 <= step <= OPTIMIZER_STEPS
        or not isinstance(head, Tensor)
        or tuple(head.shape) != QUERY_SHAPE
    ):
        raise ValueError(f"{arm} checkpoint state differs")
    parameter = next(iter(model.head_parameters(arm)))
    with torch.no_grad():
        parameter.copy_(head.to(device=device, dtype=parameter.dtype))
    head_artifact_cid = cid_bytes(model.export_head_artifact(arm))
    if checkpoint.get("head_artifact_cid") != head_artifact_cid:
        raise ValueError(f"{arm} checkpoint head CID differs")
    optimizer.load_state_dict(checkpoint["optimizer"])
    for state in optimizer.state.values():
        for key, value in state.items():
            if isinstance(value, Tensor):
                state[key] = value.to(device=device)
    rng = checkpoint.get("cpu_rng_state")
    if not isinstance(rng, Tensor):
        raise ValueError(f"{arm} checkpoint has no CPU RNG state")
    torch.set_rng_state(rng)
    expected_rate = learning_rate(step)
    if any(
        not math.isclose(float(group["lr"]), expected_rate, rel_tol=0.0, abs_tol=1e-15)
        for group in optimizer.param_groups
    ):
        raise ValueError(f"{arm} checkpoint learning rate differs")
    sidecar = _with_cid(
        {
            "schema": "uor-r4.learned-associative-readout-checkpoint-cid/1",
            "issue": ISSUE,
            "policy": POLICY,
            "arm": arm,
            "step": step,
            "bytes": path.stat().st_size,
            "checkpoint_cid": cid_file(path),
            "head_artifact_cid": head_artifact_cid,
            "run_contract_cid": run_contract_cid,
            "plan_cid": plan_cid,
        },
        "sidecar_cid",
    )
    sidecar_path = _checkpoint_cid_path(root, arm)
    try:
        observed_sidecar = _read_json(sidecar_path)
    except (ValueError, OSError):
        observed_sidecar = None
    if observed_sidecar != sidecar:
        atomic_write_json(sidecar_path, sidecar)
    return checkpoint


def _write_progress(
    root: Path,
    *,
    arm: str,
    step: int,
    elapsed_seconds: float,
    last_loss: float | None,
    status: str,
) -> dict[str, Any]:
    arm_sidecar_path = _checkpoint_cid_path(root, arm)
    shared = not arm_sidecar_path.exists()
    sidecar = _read_json(
        _shared_checkpoint_cid_path(root) if shared else arm_sidecar_path
    )
    rate = step / elapsed_seconds if elapsed_seconds > 0.0 else 0.0
    progress = {
        "schema": "uor-r4.learned-associative-readout-progress/1",
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "status": status,
        "completed_steps": step,
        "total_steps": OPTIMIZER_STEPS,
        "completed_presentations": step * BATCH_SIZE * CONTEXT,
        "total_presentations": TRAIN_DECISIONS,
        "elapsed_seconds": elapsed_seconds,
        "steps_per_second": rate,
        "eta_seconds": (OPTIMIZER_STEPS - step) / rate if rate else None,
        "last_loss": last_loss,
        "learning_rate": learning_rate(step),
        "checkpoint": {
            "path": str(
                (
                    _shared_checkpoint_path(root)
                    if shared
                    else _checkpoint_path(root, arm)
                ).relative_to(root)
            ),
            "step": sidecar["step"],
            "bytes": sidecar["bytes"],
            "cid": sidecar["checkpoint_cid"],
            "shared_pair": shared,
        },
        "resume": "run-learned-associative-readout --resume",
    }
    atomic_write_json(_progress_path(root, arm), progress)
    return progress


def _resume_elapsed(
    root: Path,
    *,
    arms: Sequence[str],
    checkpoint_step: int,
    checkpoint_elapsed: float,
) -> float:
    """Charge durable post-checkpoint work again rather than making replay free."""

    elapsed = checkpoint_elapsed
    for arm in arms:
        path = _progress_path(root, arm)
        if not path.exists():
            continue
        progress = _read_json(path)
        progress_step = progress.get("completed_steps")
        progress_elapsed = progress.get("elapsed_seconds")
        if (
            progress.get("schema") != "uor-r4.learned-associative-readout-progress/1"
            or progress.get("arm") != arm
            or isinstance(progress_step, bool)
            or not isinstance(progress_step, int)
            or not 0 <= progress_step <= OPTIMIZER_STEPS
            or isinstance(progress_elapsed, bool)
            or not isinstance(progress_elapsed, (int, float))
            or not math.isfinite(float(progress_elapsed))
        ):
            raise ValueError(f"{arm} durable progress differs from its checkpoint")
        progress_elapsed_float = float(progress_elapsed)
        if progress_step < checkpoint_step:
            if progress_elapsed_float > checkpoint_elapsed:
                raise ValueError(f"{arm} stale progress is newer than its checkpoint")
            continue
        if progress_step == checkpoint_step:
            # Artifact export/replay happens after the final checkpoint at the
            # same optimizer step.  Charge that durable work on every resume.
            elapsed = max(elapsed, progress_elapsed_float)
            continue
        if progress_elapsed_float < checkpoint_elapsed:
            raise ValueError(f"{arm} progress/checkpoint time ordering differs")
        elapsed = max(elapsed, progress_elapsed_float)
    return elapsed


def _save_shared_checkpoint(
    root: Path,
    *,
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    optimizers: Mapping[str, torch.optim.Optimizer],
    step: int,
    elapsed_seconds: float,
    run_contract_cid: str,
    plan_cid: str,
    last_losses: Mapping[str, float | None],
) -> dict[str, Any]:
    """Publish one atomic pair while keeping head/optimizer subrecords disjoint."""

    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("shared checkpoint mutation is forbidden after reveal")
    head_payloads = {arm: model.export_head_artifact(arm) for arm in ARMS}
    checkpoint = {
        "schema": "uor-r4.learned-associative-readout-shared-checkpoint/1",
        "issue": ISSUE,
        "policy": POLICY,
        "step": step,
        "elapsed_seconds": elapsed_seconds,
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "arms": {
            arm: {
                "head": next(iter(model.head_parameters(arm))).detach().cpu(),
                "head_artifact_cid": cid_bytes(head_payloads[arm]),
                "head_parameters": HEAD_PARAMETER_COUNT,
                "optimizer": optimizers[arm].state_dict(),
                "last_loss": last_losses[arm],
            }
            for arm in ARMS
        },
        "optimizer_parameter_sets_disjoint": True,
        "cpu_rng_state": torch.get_rng_state(),
    }
    path = _shared_checkpoint_path(root)
    _atomic_torch_save(path, checkpoint)
    sidecar = _with_cid(
        {
            "schema": "uor-r4.learned-associative-readout-shared-checkpoint-cid/1",
            "issue": ISSUE,
            "policy": POLICY,
            "step": step,
            "bytes": path.stat().st_size,
            "checkpoint_cid": cid_file(path),
            "run_contract_cid": run_contract_cid,
            "plan_cid": plan_cid,
            "head_artifact_cids": {
                arm: cid_bytes(head_payloads[arm]) for arm in ARMS
            },
            "head_parameters": {arm: HEAD_PARAMETER_COUNT for arm in ARMS},
            "optimizer_parameter_sets_disjoint": True,
        },
        "sidecar_cid",
    )
    atomic_write_json(_shared_checkpoint_cid_path(root), sidecar)
    return sidecar


def _load_shared_checkpoint(
    root: Path,
    *,
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    optimizers: Mapping[str, torch.optim.Optimizer],
    device: torch.device,
    run_contract_cid: str,
    plan_cid: str,
) -> dict[str, Any]:
    path = _shared_checkpoint_path(root)
    # As above, the atomic paired envelope is authoritative and the sidecar is
    # repaired only after all disjoint subrecords reproduce.
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    arms = checkpoint.get("arms") if isinstance(checkpoint, Mapping) else None
    if (
        not isinstance(checkpoint, dict)
        or checkpoint.get("schema")
        != "uor-r4.learned-associative-readout-shared-checkpoint/1"
        or checkpoint.get("run_contract_cid") != run_contract_cid
        or checkpoint.get("plan_cid") != plan_cid
        or checkpoint.get("optimizer_parameter_sets_disjoint") is not True
        or not isinstance(arms, Mapping)
        or set(arms) != set(ARMS)
    ):
        raise ValueError("shared checkpoint envelope differs")
    for arm in ARMS:
        arm_record = arms[arm]
        head = arm_record.get("head") if isinstance(arm_record, Mapping) else None
        if (
            not isinstance(head, Tensor)
            or tuple(head.shape) != QUERY_SHAPE
            or arm_record.get("head_parameters") != HEAD_PARAMETER_COUNT
        ):
            raise ValueError(f"shared {arm} checkpoint subrecord differs")
        parameter = next(iter(model.head_parameters(arm)))
        with torch.no_grad():
            parameter.copy_(head.to(device=device, dtype=parameter.dtype))
        if cid_bytes(model.export_head_artifact(arm)) != arm_record.get("head_artifact_cid"):
            raise ValueError(f"shared {arm} checkpoint head CID differs")
        optimizers[arm].load_state_dict(arm_record["optimizer"])
        for state in optimizers[arm].state.values():
            for key, value in state.items():
                if isinstance(value, Tensor):
                    state[key] = value.to(device=device)
    rng = checkpoint.get("cpu_rng_state")
    if not isinstance(rng, Tensor):
        raise ValueError("shared checkpoint has no CPU RNG state")
    torch.set_rng_state(rng)
    step = checkpoint.get("step")
    if isinstance(step, bool) or not isinstance(step, int) or not 0 <= step <= OPTIMIZER_STEPS:
        raise ValueError("shared checkpoint step differs")
    expected_rate = learning_rate(step)
    if any(
        not math.isclose(float(group["lr"]), expected_rate, rel_tol=0.0, abs_tol=1e-15)
        for optimizer in optimizers.values()
        for group in optimizer.param_groups
    ):
        raise ValueError("shared checkpoint learning rate differs")
    sidecar = _with_cid(
        {
            "schema": "uor-r4.learned-associative-readout-shared-checkpoint-cid/1",
            "issue": ISSUE,
            "policy": POLICY,
            "step": step,
            "bytes": path.stat().st_size,
            "checkpoint_cid": cid_file(path),
            "run_contract_cid": run_contract_cid,
            "plan_cid": plan_cid,
            "head_artifact_cids": {
                arm: str(checkpoint["arms"][arm]["head_artifact_cid"])
                for arm in ARMS
            },
            "head_parameters": {arm: HEAD_PARAMETER_COUNT for arm in ARMS},
            "optimizer_parameter_sets_disjoint": True,
        },
        "sidecar_cid",
    )
    sidecar_path = _shared_checkpoint_cid_path(root)
    try:
        observed_sidecar = _read_json(sidecar_path)
    except (ValueError, OSError):
        observed_sidecar = None
    if observed_sidecar != sidecar:
        atomic_write_json(sidecar_path, sidecar)
    return checkpoint


def _load_arm_result(
    root: Path,
    arm: str,
    *,
    run_contract_cid: str,
    plan_cid: str,
) -> dict[str, Any]:
    result = _read_json(_arm_result_path(root, arm))
    _verify_self_cid(result, "arm_result_cid")
    artifact = result.get("artifact")
    path = _artifact_path(root, arm)
    if (
        result.get("schema") != ARM_RESULT_SCHEMA
        or result.get("arm") != arm
        or result.get("status") != "COMPLETE"
        or result.get("run_contract_cid") != run_contract_cid
        or result.get("plan_cid") != plan_cid
        or result.get("completed_steps") != OPTIMIZER_STEPS
        or result.get("presentations") != TRAIN_DECISIONS
        or not isinstance(artifact, Mapping)
        or artifact.get("path") != str(path.relative_to(root))
        or artifact.get("bytes") != path.stat().st_size
        or artifact.get("cid") != cid_file(path)
    ):
        raise ValueError(f"{arm} completed result differs")
    return result


def _train_arm(
    root: Path,
    arm: Literal["geometric", "pooled"],
    plan: ExecutionPlan,
    *,
    run_contract_cid: str,
    resume: bool,
    wall_seconds: float,
) -> dict[str, Any]:
    """Fit exactly one disjoint head; this function cannot run after reveal."""

    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("optimizer construction is forbidden after outcome reveal")
    process_started = time.monotonic()
    plan_cid = plan.identity()["plan_cid"]
    if _arm_result_path(root, arm).exists():
        return _load_arm_result(root, arm, run_contract_cid=run_contract_cid, plan_cid=plan_cid)
    device, backend = _configure_device(plan)
    preparation = load_learned_associative_readout_preparation(root)
    model = _new_model(preparation, device)
    parameters = _select_trainable_head(model, arm)
    peer = "pooled" if arm == "geometric" else "geometric"
    peer_initial = model.export_head_artifact(peer)
    base_initial = model.export_qualified_base_artifact()
    optimizer = _head_optimizer(parameters)
    checkpoint_path = _checkpoint_path(root, arm)
    step = 0
    elapsed_before = 0.0
    last_loss: float | None = None
    if checkpoint_path.exists():
        if not resume:
            raise FileExistsError(f"{arm} checkpoint exists; --resume is required")
        checkpoint = _load_checkpoint(
            root,
            arm=arm,
            model=model,
            optimizer=optimizer,
            device=device,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
        )
        step = int(checkpoint["step"])
        elapsed_before = _resume_elapsed(
            root,
            arms=(arm,),
            checkpoint_step=step,
            checkpoint_elapsed=float(checkpoint["elapsed_seconds"]),
        )
        last_loss = checkpoint.get("last_loss")
    else:
        _save_checkpoint(
            root,
            arm=arm,
            model=model,
            optimizer=optimizer,
            step=0,
            elapsed_seconds=0.0,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
            last_loss=None,
        )
        _write_progress(root, arm=arm, step=0, elapsed_seconds=0.0, last_loss=None, status="RUNNING")

    elapsed_now = elapsed_before + (time.monotonic() - process_started)
    if elapsed_now >= wall_seconds:
        progress = _write_progress(
            root,
            arm=arm,
            step=step,
            elapsed_seconds=elapsed_now,
            last_loss=last_loss,
            status=TERMINAL_UNAVAILABLE,
        )
        return {"arm": arm, "status": TERMINAL_UNAVAILABLE, "progress": progress}

    for next_step in range(step + 1, OPTIMIZER_STEPS + 1):
        batch = _ordered_train_batch(preparation.predecessor, next_step, device)
        last_loss, _gradient_norm = _train_step(
            model, arm, optimizer, parameters, batch, step=next_step
        )
        step = next_step
        elapsed = elapsed_before + (time.monotonic() - process_started)
        checkpoint_due = step % CHECKPOINT_INTERVAL == 0 or step == OPTIMIZER_STEPS
        if checkpoint_due:
            _save_checkpoint(
                root,
                arm=arm,
                model=model,
                optimizer=optimizer,
                step=step,
                elapsed_seconds=elapsed,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
                last_loss=last_loss,
            )
        if step % PROGRESS_INTERVAL == 0 or step == OPTIMIZER_STEPS:
            progress = _write_progress(
                root,
                arm=arm,
                step=step,
                elapsed_seconds=elapsed,
                last_loss=last_loss,
                status="RUNNING",
            )
            if step % max(PROGRESS_INTERVAL, 100) == 0 or step == OPTIMIZER_STEPS:
                print(
                    f"learned_associative arm={arm} step={step}/{OPTIMIZER_STEPS} "
                    f"loss={last_loss:.6f} eta={progress['eta_seconds']}",
                    flush=True,
                )
        elapsed = elapsed_before + (time.monotonic() - process_started)
        if elapsed >= wall_seconds:
            if not checkpoint_due:
                _save_checkpoint(
                    root,
                    arm=arm,
                    model=model,
                    optimizer=optimizer,
                    step=step,
                    elapsed_seconds=elapsed,
                    run_contract_cid=run_contract_cid,
                    plan_cid=plan_cid,
                    last_loss=last_loss,
                )
            progress = _write_progress(
                root,
                arm=arm,
                step=step,
                elapsed_seconds=elapsed,
                last_loss=last_loss,
                status=TERMINAL_UNAVAILABLE,
            )
            return {"arm": arm, "status": TERMINAL_UNAVAILABLE, "progress": progress}

    elapsed = elapsed_before + (time.monotonic() - process_started)
    if elapsed >= wall_seconds:
        progress = _write_progress(
            root,
            arm=arm,
            step=step,
            elapsed_seconds=elapsed,
            last_loss=last_loss,
            status=TERMINAL_UNAVAILABLE,
        )
        return {"arm": arm, "status": TERMINAL_UNAVAILABLE, "progress": progress}
    if model.export_qualified_base_artifact() != base_initial or model.export_head_artifact(peer) != peer_initial:
        raise RuntimeError(f"{arm} fit mutated frozen or peer parameters")
    artifact = model.export_head_artifact(arm)
    artifact_path = _artifact_path(root, arm)
    if artifact_path.exists():
        if artifact_path.read_bytes() != artifact:
            raise ValueError(f"{arm} pre-existing artifact differs")
    else:
        atomic_write(artifact_path, artifact)
    replay = _new_model(preparation, device)
    replay.load_head_artifact(arm, artifact)
    fixed_batch = _ordered_train_batch(preparation.predecessor, OPTIMIZER_STEPS, device)[:1]
    model.eval()
    replay.eval()
    with torch.no_grad():
        expected = model.forward_arm(arm, fixed_batch[:, :-1])
        observed = replay.forward_arm(arm, fixed_batch[:, :-1])
    replay_delta = float((expected.logits - observed.logits).abs().max().cpu())
    if replay_delta != 0.0:
        raise RuntimeError(f"{arm} artifact replay differs")
    elapsed = elapsed_before + (time.monotonic() - process_started)
    body = {
        "schema": ARM_RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "status": "COMPLETE",
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "backend": backend,
        "completed_steps": OPTIMIZER_STEPS,
        "presentations": TRAIN_DECISIONS,
        "train_order_cid": _train_order_identity(preparation.predecessor)["order_cid"],
        "elapsed_seconds": elapsed,
        "last_loss": last_loss,
        "artifact": {
            "path": str(artifact_path.relative_to(root)),
            "bytes": artifact_path.stat().st_size,
            "cid": cid_file(artifact_path),
        },
        "base_artifact_cid": cid_bytes(base_initial),
        "peer_head_unchanged": True,
        "artifact_replay_maximum_logits_delta": replay_delta,
        "forbidden_reads": int(expected.audit.forbidden_reads) + int(observed.audit.forbidden_reads),
    }
    result = _with_cid(body, "arm_result_cid")
    _write_exclusive_json(_arm_result_path(root, arm), result)
    _write_progress(
        root,
        arm=arm,
        step=OPTIMIZER_STEPS,
        elapsed_seconds=elapsed,
        last_loss=last_loss,
        status="COMPLETE",
    )
    return result


def _finalize_shared_arm(
    root: Path,
    *,
    arm: Literal["geometric", "pooled"],
    model: R4LearnedCandidateLeafAssociativeReadoutV1,
    preparation: CampaignPreparation,
    device: torch.device,
    backend: Mapping[str, Any],
    run_contract_cid: str,
    plan_cid: str,
    elapsed_before_seconds: float,
    process_started: float,
    wall_seconds: float,
    last_loss: float,
    base_initial: bytes,
) -> dict[str, Any]:
    artifact = model.export_head_artifact(arm)
    artifact_path = _artifact_path(root, arm)
    if artifact_path.exists():
        if artifact_path.read_bytes() != artifact:
            raise ValueError(f"{arm} pre-existing shared artifact differs")
    else:
        atomic_write(artifact_path, artifact)
    replay = _new_model(preparation, device)
    replay.load_head_artifact(arm, artifact)
    fixed_batch = _ordered_train_batch(preparation.predecessor, OPTIMIZER_STEPS, device)[:1]
    model.eval()
    replay.eval()
    with torch.no_grad():
        expected = model.forward_arm(arm, fixed_batch[:, :-1])
        observed = replay.forward_arm(arm, fixed_batch[:, :-1])
    replay_delta = float((expected.logits - observed.logits).abs().max().cpu())
    if replay_delta != 0.0:
        raise RuntimeError(f"{arm} shared artifact replay differs")
    elapsed_seconds = elapsed_before_seconds + (time.monotonic() - process_started)
    if elapsed_seconds >= wall_seconds:
        progress = _write_progress(
            root,
            arm=arm,
            step=OPTIMIZER_STEPS,
            elapsed_seconds=elapsed_seconds,
            last_loss=last_loss,
            status=TERMINAL_UNAVAILABLE,
        )
        return {"arm": arm, "status": TERMINAL_UNAVAILABLE, "progress": progress}
    body = {
        "schema": ARM_RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "status": "COMPLETE",
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "backend": dict(backend),
        "shared_feature_pass": True,
        "completed_steps": OPTIMIZER_STEPS,
        "presentations": TRAIN_DECISIONS,
        "train_order_cid": _train_order_identity(preparation.predecessor)["order_cid"],
        "elapsed_seconds": elapsed_seconds,
        "last_loss": last_loss,
        "artifact": {
            "path": str(artifact_path.relative_to(root)),
            "bytes": artifact_path.stat().st_size,
            "cid": cid_file(artifact_path),
        },
        "base_artifact_cid": cid_bytes(base_initial),
        "optimizers_disjoint": True,
        "artifact_replay_maximum_logits_delta": replay_delta,
        "forbidden_reads": int(expected.audit.forbidden_reads) + int(observed.audit.forbidden_reads),
    }
    result = _with_cid(body, "arm_result_cid")
    _write_exclusive_json(_arm_result_path(root, arm), result)
    _write_progress(
        root,
        arm=arm,
        step=OPTIMIZER_STEPS,
        elapsed_seconds=elapsed_seconds,
        last_loss=last_loss,
        status="COMPLETE",
    )
    return result


def _train_shared(
    root: Path,
    plan: ExecutionPlan,
    *,
    run_contract_cid: str,
    resume: bool,
    wall_seconds: float,
) -> dict[str, Any]:
    """Fit both heads with one backbone pass and independent optimizer slots."""

    if plan.concurrent_arms:
        raise ValueError("shared fit requires a sequential plan")
    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("optimizer construction is forbidden after outcome reveal")
    process_started = time.monotonic()
    plan_cid = plan.identity()["plan_cid"]
    completed = {
        arm: _arm_result_path(root, arm).exists()
        for arm in ARMS
    }
    if all(completed.values()):
        return {
            arm: _load_arm_result(root, arm, run_contract_cid=run_contract_cid, plan_cid=plan_cid)
            for arm in ARMS
        }
    device, backend = _configure_device(plan)
    preparation = load_learned_associative_readout_preparation(root)
    model = _new_model(preparation, device)
    for parameter in model.frozen_base_parameters():
        parameter.requires_grad_(False)
    parameters = {arm: tuple(model.head_parameters(arm)) for arm in ARMS}
    if {id(value) for value in parameters["geometric"]} & {id(value) for value in parameters["pooled"]}:
        raise RuntimeError("shared fit head parameters overlap")
    optimizers = {arm: _head_optimizer(parameters[arm]) for arm in ARMS}
    base_initial = model.export_qualified_base_artifact()
    heads_initial = {arm: model.export_head_artifact(arm) for arm in ARMS}
    checkpoint_path = _shared_checkpoint_path(root)
    step = 0
    elapsed_before = 0.0
    last_losses = {arm: math.nan for arm in ARMS}
    if checkpoint_path.exists():
        if not resume:
            raise FileExistsError("shared checkpoint exists; --resume is required")
        loaded = _load_shared_checkpoint(
            root,
            model=model,
            optimizers=optimizers,
            device=device,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
        )
        step = int(loaded["step"])
        elapsed_before = _resume_elapsed(
            root,
            arms=ARMS,
            checkpoint_step=step,
            checkpoint_elapsed=float(loaded["elapsed_seconds"]),
        )
        for arm in ARMS:
            if loaded["arms"][arm].get("last_loss") is not None:
                last_losses[arm] = float(loaded["arms"][arm]["last_loss"])
    else:
        _save_shared_checkpoint(
            root,
            model=model,
            optimizers=optimizers,
            step=0,
            elapsed_seconds=0.0,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
            last_losses={arm: None for arm in ARMS},
        )
        for arm in ARMS:
            _write_progress(root, arm=arm, step=0, elapsed_seconds=0.0, last_loss=None, status="RUNNING")

    elapsed_now = elapsed_before + (time.monotonic() - process_started)
    if elapsed_now >= wall_seconds:
        return {
            arm: {
                "arm": arm,
                "status": TERMINAL_UNAVAILABLE,
                "progress": _write_progress(
                    root,
                    arm=arm,
                    step=step,
                    elapsed_seconds=elapsed_now,
                    last_loss=None if math.isnan(last_losses[arm]) else last_losses[arm],
                    status=TERMINAL_UNAVAILABLE,
                ),
            }
            for arm in ARMS
        }

    for next_step in range(step + 1, OPTIMIZER_STEPS + 1):
        batch = _ordered_train_batch(preparation.predecessor, next_step, device)
        last_losses, _norms = _shared_train_step(
            model, optimizers, parameters, batch, step=next_step
        )
        step = next_step
        elapsed = elapsed_before + (time.monotonic() - process_started)
        checkpoint_due = step % CHECKPOINT_INTERVAL == 0 or step == OPTIMIZER_STEPS
        if checkpoint_due:
            _save_shared_checkpoint(
                root,
                model=model,
                optimizers=optimizers,
                step=step,
                elapsed_seconds=elapsed,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
                last_losses=last_losses,
            )
        if step % PROGRESS_INTERVAL == 0 or step == OPTIMIZER_STEPS:
            for arm in ARMS:
                _write_progress(
                    root,
                    arm=arm,
                    step=step,
                    elapsed_seconds=elapsed,
                    last_loss=last_losses[arm],
                    status="RUNNING",
                )
            if step % max(PROGRESS_INTERVAL, 100) == 0 or step == OPTIMIZER_STEPS:
                print(
                    f"learned_associative shared step={step}/{OPTIMIZER_STEPS} "
                    f"geometric_loss={last_losses['geometric']:.6f} "
                    f"pooled_loss={last_losses['pooled']:.6f}",
                    flush=True,
                )
        elapsed = elapsed_before + (time.monotonic() - process_started)
        if elapsed >= wall_seconds:
            if not checkpoint_due:
                _save_shared_checkpoint(
                    root,
                    model=model,
                    optimizers=optimizers,
                    step=step,
                    elapsed_seconds=elapsed,
                    run_contract_cid=run_contract_cid,
                    plan_cid=plan_cid,
                    last_losses=last_losses,
                )
            return {
                arm: {
                    "arm": arm,
                    "status": TERMINAL_UNAVAILABLE,
                    "progress": _write_progress(
                        root,
                        arm=arm,
                        step=step,
                        elapsed_seconds=elapsed,
                        last_loss=last_losses[arm],
                        status=TERMINAL_UNAVAILABLE,
                    ),
                }
                for arm in ARMS
            }

    elapsed = elapsed_before + (time.monotonic() - process_started)
    if elapsed >= wall_seconds:
        return {
            arm: {
                "arm": arm,
                "status": TERMINAL_UNAVAILABLE,
                "progress": _write_progress(
                    root,
                    arm=arm,
                    step=step,
                    elapsed_seconds=elapsed,
                    last_loss=last_losses[arm],
                    status=TERMINAL_UNAVAILABLE,
                ),
            }
            for arm in ARMS
        }
    if model.export_qualified_base_artifact() != base_initial:
        raise RuntimeError("shared fit mutated the frozen base")
    if any(model.export_head_artifact(arm) == heads_initial[arm] for arm in ARMS):
        raise RuntimeError("shared fit left a learned head at exact zero")
    outcomes: dict[str, Any] = {}
    for index, arm in enumerate(ARMS):
        outcome = (
            _load_arm_result(
                root,
                arm,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
            )
            if completed[arm]
            else _finalize_shared_arm(
                root,
                arm=arm,  # type: ignore[arg-type]
                model=model,
                preparation=preparation,
                device=device,
                backend=backend,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
                elapsed_before_seconds=elapsed_before,
                process_started=process_started,
                wall_seconds=wall_seconds,
                last_loss=last_losses[arm],
                base_initial=base_initial,
            )
        )
        outcomes[arm] = outcome
        if outcome.get("status") == "COMPLETE":
            continue
        # The shared model has one aggregate wall.  Once one arm's artifact
        # replay exhausts it, do not begin the peer's export/replay.  Mark the
        # unfinalized remainder unavailable at the same durable elapsed time.
        pair_elapsed = float(
            outcome.get("progress", {}).get(
                "elapsed_seconds",
                elapsed_before + (time.monotonic() - process_started),
            )
        )
        for remaining_arm in ARMS[index + 1 :]:
            progress = _write_progress(
                root,
                arm=remaining_arm,
                step=OPTIMIZER_STEPS,
                elapsed_seconds=pair_elapsed,
                last_loss=(
                    None
                    if math.isnan(last_losses[remaining_arm])
                    else last_losses[remaining_arm]
                ),
                status=TERMINAL_UNAVAILABLE,
            )
            outcomes[remaining_arm] = {
                "arm": remaining_arm,
                "status": TERMINAL_UNAVAILABLE,
                "reason": "SHARED_PAIR_FINALIZATION_WALL_EXHAUSTED",
                "progress": progress,
            }
        return outcomes
    return outcomes


def _shared_arm_worker(
    root: str,
    plan_value: Mapping[str, Any],
    run_contract_cid: str,
    resume: bool,
    wall_seconds: float,
    queue: Any,
) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        queue.put(
            {
                "ok": True,
                "result": _train_shared(
                    Path(root),
                    plan,
                    run_contract_cid=run_contract_cid,
                    resume=resume,
                    wall_seconds=wall_seconds,
                ),
            }
        )
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _arm_worker(
    root: str,
    arm: str,
    plan_value: Mapping[str, Any],
    run_contract_cid: str,
    resume: bool,
    wall_seconds: float,
    queue: Any,
) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        result = _train_arm(
            Path(root),
            arm,  # type: ignore[arg-type]
            plan,
            run_contract_cid=run_contract_cid,
            resume=resume,
            wall_seconds=wall_seconds,
        )
        queue.put({"ok": True, "result": result})
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _spawned_arm_runner(
    root: Path,
    plan: ExecutionPlan,
    *,
    run_contract_cid: str,
    resume: bool,
    wall_seconds: float,
) -> dict[str, Any]:
    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("training workers cannot launch after reveal")
    context = mp.get_context("spawn")
    if not plan.concurrent_arms:
        queue = context.Queue()
        process = context.Process(
            target=_shared_arm_worker,
            args=(
                str(root),
                asdict(plan),
                run_contract_cid,
                resume,
                wall_seconds,
                queue,
            ),
            name="learned-associative-fit-shared",
        )
        process.start()
        outcome = _collect_worker(
            process,
            queue,
            timeout=HARD_WALL_CEILING_SECONDS + 60.0,
        )
        if not outcome.get("ok"):
            return {arm: dict(outcome) for arm in ARMS}
        return {
            arm: {"ok": True, "result": outcome["result"][arm]}
            for arm in ARMS
        }
    outcomes: dict[str, Any] = {}
    active: dict[str, tuple[Any, Any]] = {}
    completed_elapsed = 0.0
    for arm in ARMS:
        result_path = _arm_result_path(root, arm)
        if result_path.exists():
            outcomes[arm] = {"ok": True, "result": _load_arm_result(
                root,
                arm,
                run_contract_cid=run_contract_cid,
                plan_cid=plan.identity()["plan_cid"],
            )}
            if not plan.concurrent_arms:
                completed_elapsed += float(outcomes[arm]["result"]["elapsed_seconds"])
            continue
        queue = context.Queue()
        process = context.Process(
            target=_arm_worker,
            args=(
                str(root),
                arm,
                asdict(plan),
                run_contract_cid,
                resume,
                wall_seconds if plan.concurrent_arms else max(0.0, wall_seconds - completed_elapsed),
                queue,
            ),
            name=f"learned-associative-fit-{arm}",
        )
        process.start()
        if plan.concurrent_arms:
            active[arm] = (process, queue)
        else:
            outcome = _collect_worker(process, queue, timeout=HARD_WALL_CEILING_SECONDS + 60.0)
            outcomes[arm] = outcome
            if outcome.get("ok"):
                completed_elapsed += float(outcome["result"].get("elapsed_seconds", 0.0))
    for arm, (process, queue) in active.items():
        outcomes[arm] = _collect_worker(process, queue, timeout=HARD_WALL_CEILING_SECONDS + 60.0)
    return outcomes


class _ArmView:
    """Expose one learned arm through the scorer's ordinary model interface."""

    def __init__(self, model: R4LearnedCandidateLeafAssociativeReadoutV1, arm: str) -> None:
        self.model = model
        self.arm = arm

    def eval(self) -> _ArmView:
        self.model.eval()
        return self

    def __call__(self, token_ids: Tensor, *, attention_off: bool = False) -> Any:
        return self.model.forward_arm(self.arm, token_ids, attention_off=attention_off)


def _load_scoring_model(
    preparation: CampaignPreparation,
    device: torch.device,
    *,
    geometric_artifact: bytes | None = None,
    pooled_artifact: bytes | None = None,
) -> R4LearnedCandidateLeafAssociativeReadoutV1:
    model = _new_model(preparation, device)
    if geometric_artifact is not None:
        model.load_head_artifact("geometric", geometric_artifact)
    if pooled_artifact is not None:
        model.load_head_artifact("pooled", pooled_artifact)
    model.eval()
    return model


@torch.no_grad()
def _evaluate_language(
    model: Any,
    windows: LanguagePathWindowStore,
    device: torch.device,
    *,
    attention_off: bool = False,
) -> dict[str, Any]:
    if hasattr(model, "eval"):
        model.eval()
    loss_sum = 0.0
    top1 = 0
    rows = 0
    forbidden_reads = 0
    digest = blake3()
    for start in range(0, FRESH_HELDOUT_WINDOWS, BATCH_SIZE):
        count = min(BATCH_SIZE, FRESH_HELDOUT_WINDOWS - start)
        batch = _window_batch(windows, start, count, device)
        output = model(batch[:, :-1], attention_off=attention_off)
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
        audited = getattr(output.audit, "attention_off", getattr(output.audit, "state_off", None))
        if audited is not attention_off:
            raise RuntimeError("fresh-heldout evaluation reported the wrong state mode")
        digest.update(logits.cpu().contiguous().numpy().tobytes())
    if rows != FRESH_HELDOUT_DECISIONS or forbidden_reads != 0:
        raise RuntimeError("fresh-heldout evaluation failed its causal/coverage audit")
    return {
        "rows": rows,
        "ce_nats": loss_sum / rows,
        "top1_correct": top1,
        "top1_rate": top1 / rows,
        "logits_cid": f"blake3:{digest.hexdigest()}",
        "forbidden_reads": forbidden_reads,
        "attention_off": attention_off,
    }


def fresh_generalization_gates(
    *,
    candidate: Mapping[str, Any],
    predecessor: Mapping[str, Any],
    state_off: Mapping[str, Any],
) -> dict[str, Any]:
    """Apply the per-arm fresh-language nonregression/load-bearing gates."""

    if any(int(value.get("rows", -1)) != FRESH_HELDOUT_DECISIONS for value in (candidate, predecessor, state_off)):
        raise ValueError("fresh-heldout gate rows differ")
    if candidate.get("attention_off") is not False or predecessor.get("attention_off") is not False or state_off.get("attention_off") is not True:
        raise ValueError("fresh-heldout gate modes differ")
    predecessor_nll_delta = float(candidate["ce_nats"]) - float(predecessor["ce_nats"])
    predecessor_top1_point_delta = 100.0 * (
        float(candidate["top1_rate"]) - float(predecessor["top1_rate"])
    )
    state_off_nll_cost = float(state_off["ce_nats"]) - float(candidate["ce_nats"])
    state_off_top1_decision_loss = int(candidate["top1_correct"]) - int(state_off["top1_correct"])
    gates = {
        "predecessor_nll_nonregression": predecessor_nll_delta <= PREDECESSOR_NLL_TOLERANCE,
        "predecessor_top1_nonregression": predecessor_top1_point_delta >= -PREDECESSOR_TOP1_POINT_TOLERANCE,
        "state_off_nll_load_bearing": state_off_nll_cost >= REQUIRED_STATE_OFF_NLL_COST,
        "state_off_top1_load_bearing": state_off_top1_decision_loss >= REQUIRED_STATE_OFF_TOP1_DECISION_LOSS,
        "forbidden_reads_zero": all(int(value.get("forbidden_reads", -1)) == 0 for value in (candidate, predecessor, state_off)),
    }
    return {
        "passed": all(gates.values()),
        "gates": gates,
        "predecessor_nll_delta": predecessor_nll_delta,
        "predecessor_top1_point_delta": predecessor_top1_point_delta,
        "state_off_nll_cost": state_off_nll_cost,
        "state_off_top1_decision_loss": state_off_top1_decision_loss,
        "thresholds": {
            "predecessor_nll_tolerance": PREDECESSOR_NLL_TOLERANCE,
            "predecessor_top1_point_tolerance": PREDECESSOR_TOP1_POINT_TOLERANCE,
            "state_off_nll_cost": REQUIRED_STATE_OFF_NLL_COST,
            "state_off_top1_decision_loss": REQUIRED_STATE_OFF_TOP1_DECISION_LOSS,
        },
    }


def terminal_decision(
    *,
    geometric_capacity_verdict: str,
    pooled_capacity_verdict: str,
    geometry_verdict: str,
    geometric_fresh_passed: bool,
    pooled_fresh_passed: bool,
    geometric_fresh_nll: float,
    pooled_fresh_nll: float,
    mechanics_passed: bool,
) -> dict[str, Any]:
    """Map the frozen independent science decisions to divergent next actions."""

    if (
        not mechanics_passed
        or geometric_capacity_verdict == VERDICT_INVALID
        or pooled_capacity_verdict == VERDICT_INVALID
        or geometry_verdict == GEOMETRY_ATTRIBUTION_INVALID
    ):
        return {
            "verdict": TERMINAL_INVALID,
            "action": "repair only the failed causal, replay, seal, or control mechanic; do not interpret model metrics",
            "selected_arm": None,
        }
    geometric_capacity = geometric_capacity_verdict == VERDICT_PASS
    pooled_capacity = pooled_capacity_verdict == VERDICT_PASS
    if (
        geometric_capacity
        and geometric_fresh_passed
        and geometry_verdict == GEOMETRY_ATTRIBUTION_PASS
    ):
        return {
            "verdict": TERMINAL_GEOMETRIC_PASS,
            "action": "preserve the geometric head and authorize a separately frozen disjoint autonomous smoke",
            "selected_arm": "geometric",
        }
    passing = [
        (geometric_fresh_nll, "geometric")
        for _ in (0,)
        if geometric_capacity and geometric_fresh_passed
    ] + [
        (pooled_fresh_nll, "pooled")
        for _ in (0,)
        if pooled_capacity and pooled_fresh_passed
    ]
    if passing:
        selected = min(passing)[1]
        return {
            "verdict": TERMINAL_CONTROL_ONLY,
            "action": "report geometry unestablished; authorize a smoke only for the lowest-fresh-NLL passing arm",
            "selected_arm": selected,
        }
    if geometric_capacity or pooled_capacity:
        return {
            "verdict": TERMINAL_FRESH_REGRESSION,
            "action": "do not run generation; freeze a joint prompt-capacity and fresh-language objective",
            "selected_arm": None,
        }
    return {
        "verdict": TERMINAL_NO_CAPACITY,
        "action": "reject this exact associative law without tuning or retry; revisit retained value representation and binding",
        "selected_arm": None,
    }


def _score_campaign(
    preparation: CampaignPreparation,
    *,
    geometric_artifact: bytes,
    pooled_artifact: bytes,
) -> dict[str, Any]:
    """Score every frozen arm/control on canonical CPU4 after one reveal."""

    if not (preparation.root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("final scoring is forbidden before outcome reveal")
    device, backend = _configure_device(ELIGIBLE_PLANS[0])
    if device.type != "cpu":
        raise RuntimeError("canonical final scoring requires CPU4")
    population = load_revealed_prompt_conditioning_population(preparation.root)
    heldout = LanguagePathWindowStore(
        preparation.root / HELDOUT_RELATIVE_PATH,
        window_count=FRESH_HELDOUT_WINDOWS,
    )
    base_artifact = preparation.predecessor_artifact_path.read_bytes()

    def baseline_factory() -> R4RetainedLanguagePathV1:
        model = R4RetainedLanguagePathV1(_exact_geometry(preparation.predecessor)).to(device)
        model.load_learned_artifact(base_artifact)
        model.eval()
        return model

    def view_factory(arm: str) -> _ArmView:
        model = _load_scoring_model(
            preparation,
            device,
            geometric_artifact=geometric_artifact if arm in ("geometric", "deranged") else None,
            pooled_artifact=pooled_artifact if arm == "pooled" else None,
        )
        return _ArmView(model, arm)

    prompt_scores = {
        "v1": score_prompt_conditioning(
            baseline_factory(), population, attention_off=False, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
        "geometric": score_prompt_conditioning(
            view_factory("geometric"), population, attention_off=False, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
        "pooled": score_prompt_conditioning(
            view_factory("pooled"), population, attention_off=False, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
        "deranged": score_prompt_conditioning(
            view_factory("deranged"), population, attention_off=False, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
        "geometric_state_off": score_prompt_conditioning(
            view_factory("geometric"), population, attention_off=True, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
        "pooled_state_off": score_prompt_conditioning(
            view_factory("pooled"), population, attention_off=True, direction_batch_size=DIRECTION_BATCH_SIZE, device=device
        ),
    }
    replay_exact: dict[str, bool] = {}
    for name in ("v1", "geometric", "pooled", "deranged"):
        replay_model: Any = baseline_factory() if name == "v1" else view_factory(name)
        replay = score_prompt_conditioning(
            replay_model,
            population,
            attention_off=False,
            direction_batch_size=DIRECTION_BATCH_SIZE,
            device=device,
        )
        replay_exact[name] = replay.record() == prompt_scores[name].record()

    capacity = {
        "geometric": associative_capacity_decision(
            prompt_scores["geometric"], prompt_scores["v1"], prompt_scores["geometric_state_off"]
        ),
        "pooled": associative_capacity_decision(
            prompt_scores["pooled"], prompt_scores["v1"], prompt_scores["pooled_state_off"]
        ),
    }
    geometry = geometry_attribution_decision(
        prompt_scores["geometric"], prompt_scores["pooled"], prompt_scores["deranged"]
    )
    language = {
        "v1": _evaluate_language(baseline_factory(), heldout, device),
        "geometric": _evaluate_language(view_factory("geometric"), heldout, device),
        "pooled": _evaluate_language(view_factory("pooled"), heldout, device),
        "geometric_state_off": _evaluate_language(
            view_factory("geometric"), heldout, device, attention_off=True
        ),
        "pooled_state_off": _evaluate_language(
            view_factory("pooled"), heldout, device, attention_off=True
        ),
    }
    fresh = {
        "geometric": fresh_generalization_gates(
            candidate=language["geometric"],
            predecessor=language["v1"],
            state_off=language["geometric_state_off"],
        ),
        "pooled": fresh_generalization_gates(
            candidate=language["pooled"],
            predecessor=language["v1"],
            state_off=language["pooled_state_off"],
        ),
    }
    return {
        "backend": backend,
        "population_cid": population.population_cid,
        "prompt_scores": {name: score.record() for name, score in prompt_scores.items()},
        "prompt_replay_exact": replay_exact,
        "capacity_decisions": {arm: capacity[arm].record() for arm in ARMS},
        "geometry_attribution": geometry.record(),
        "fresh_language": language,
        "fresh_gates": fresh,
    }


def _scoring_evidence_path(root: Path) -> Path:
    return root / SCORING_EVIDENCE_RELATIVE_PATH


def _scoring_worker(root: str, queue: Any) -> None:
    """Score in a killable fresh process and publish only a small queue message."""

    scoring_started = time.monotonic()
    try:
        campaign_root = Path(root)
        preparation = load_learned_associative_readout_preparation(campaign_root)
        reveal = _read_json(campaign_root / REVEAL_RELATIVE_PATH)
        _verify_self_cid(reveal, "reveal_cid")
        artifacts: dict[str, dict[str, Any]] = {}
        payloads: dict[str, bytes] = {}
        for arm in ARMS:
            path = _artifact_path(campaign_root, arm)
            if path.is_symlink() or not path.is_file():
                raise ValueError(f"{arm} scoring artifact is not a regular file")
            payloads[arm] = path.read_bytes()
            artifacts[arm] = {
                "path": str(path.relative_to(campaign_root)),
                "bytes": path.stat().st_size,
                "cid": cid_file(path),
            }
        if (
            reveal.get("geometric_artifact_cid") != artifacts["geometric"]["cid"]
            or reveal.get("pooled_artifact_cid") != artifacts["pooled"]["cid"]
        ):
            raise ValueError("scoring artifacts differ from the reveal")
        evidence = _score_campaign(
            preparation,
            geometric_artifact=payloads["geometric"],
            pooled_artifact=payloads["pooled"],
        )
        body = {
            "schema": "uor-r4.learned-associative-readout-scoring-evidence/1",
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation.manifest["preparation_cid"],
            "implementation": preparation.manifest["implementation"],
            "reveal_cid": reveal["reveal_cid"],
            "artifacts": artifacts,
            "scorer_process_id": os.getpid(),
            "scoring_seconds": time.monotonic() - scoring_started,
            "optimizer_created": False,
            "optimizer_steps": 0,
            "evidence": evidence,
        }
        record = _with_cid(body, "scoring_evidence_cid")
        path = _scoring_evidence_path(campaign_root)
        atomic_write_json(path, record)
        queue.put(
            {
                "ok": True,
                "record": {
                    "path": SCORING_EVIDENCE_RELATIVE_PATH,
                    "bytes": path.stat().st_size,
                    "cid": cid_file(path),
                },
            }
        )
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _load_scoring_evidence(
    preparation: CampaignPreparation,
    reveal: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    path = _scoring_evidence_path(preparation.root)
    record = _read_json(path)
    _verify_self_cid(record, "scoring_evidence_cid")
    artifacts = record.get("artifacts")
    scoring_seconds = record.get("scoring_seconds")
    scorer_process_id = record.get("scorer_process_id")
    if (
        record.get("schema")
        != "uor-r4.learned-associative-readout-scoring-evidence/1"
        or record.get("issue") != ISSUE
        or record.get("policy") != POLICY
        or record.get("preparation_cid")
        != preparation.manifest["preparation_cid"]
        or record.get("implementation") != preparation.manifest["implementation"]
        or record.get("reveal_cid") != reveal.get("reveal_cid")
        or not isinstance(artifacts, Mapping)
        or any(artifacts.get(arm) != arm_results[arm].get("artifact") for arm in ARMS)
        or isinstance(scorer_process_id, bool)
        or not isinstance(scorer_process_id, int)
        or isinstance(scoring_seconds, bool)
        or not isinstance(scoring_seconds, (int, float))
        or not math.isfinite(float(scoring_seconds))
        or float(scoring_seconds) < 0.0
        or record.get("optimizer_created") is not False
        or record.get("optimizer_steps") != 0
        or not isinstance(record.get("evidence"), Mapping)
    ):
        raise ValueError("canonical scoring evidence differs")
    for arm in ARMS:
        artifact_path = _artifact_path(preparation.root, arm)
        artifact = arm_results[arm]["artifact"]
        if (
            artifact_path.is_symlink()
            or not artifact_path.is_file()
            or artifact_path.stat().st_size != artifact["bytes"]
            or cid_file(artifact_path) != artifact["cid"]
        ):
            raise ValueError(f"{arm} artifact changed after canonical scoring")
    return record


def _spawned_scoring_executor(
    preparation: CampaignPreparation,
    reveal: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
    timeout_seconds: float,
) -> dict[str, Any]:
    """Run canonical scoring under the residual aggregate wall."""

    if not math.isfinite(timeout_seconds) or timeout_seconds <= 0.0:
        return {
            "ok": False,
            "error": {
                "type": "TimeoutError",
                "reason": "no residual aggregate wall remains for scoring",
            },
        }
    evidence_path = _scoring_evidence_path(preparation.root)
    if evidence_path.exists():
        record = _load_scoring_evidence(preparation, reveal, arm_results)
        return {
            "ok": True,
            "evidence": dict(record["evidence"]),
            "scoring_seconds": float(record["scoring_seconds"]),
            "cached": True,
        }
    context = mp.get_context("spawn")
    queue = context.Queue()
    process = context.Process(
        target=_scoring_worker,
        args=(str(preparation.root), queue),
        name="learned-associative-canonical-scoring",
    )
    process.start()
    outcome = _collect_worker(process, queue, timeout=timeout_seconds)
    if not outcome.get("ok"):
        return outcome
    pointer = outcome.get("record")
    if not isinstance(pointer, Mapping):
        raise ValueError("scoring worker omitted its evidence pointer")
    record = _load_scoring_evidence(preparation, reveal, arm_results)
    if pointer != {
        "path": SCORING_EVIDENCE_RELATIVE_PATH,
        "bytes": evidence_path.stat().st_size,
        "cid": cid_file(evidence_path),
    }:
        raise ValueError("scoring worker evidence pointer differs")
    return {
        "ok": True,
        "evidence": dict(record["evidence"]),
        "scoring_seconds": float(record["scoring_seconds"]),
        "cached": False,
    }


def _selected_plan(probe: Mapping[str, Any]) -> ExecutionPlan:
    selected = probe.get("selection", {}).get("selected_plan")
    if not isinstance(selected, Mapping):
        raise ValueError("execution probe did not bind a selected plan")
    matched = [plan for plan in ELIGIBLE_PLANS if plan.identity() == dict(selected)]
    if len(matched) != 1:
        raise ValueError("selected execution plan is outside the frozen candidates")
    return matched[0]


def _run_contract(preparation: CampaignPreparation, probe: Mapping[str, Any]) -> dict[str, Any]:
    plan = _selected_plan(probe)
    scoring_reserve_seconds = PROJECTION_SAFETY_FACTOR * float(
        probe["selection"]["selected_projection"][
            "common_canonical_cpu4_scoring_seconds"
        ]
    )
    if (
        not math.isfinite(scoring_reserve_seconds)
        or scoring_reserve_seconds <= 0.0
        or scoring_reserve_seconds >= HARD_WALL_CEILING_SECONDS
    ):
        raise ValueError("execution probe cannot reserve the aggregate scoring wall")
    return {
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation.manifest["preparation_cid"],
        "prompt_commitment_cid": preparation.commitment["commitment_cid"],
        "probe_cid": probe["probe_cid"],
        "implementation": preparation.manifest["implementation"],
        "plan": plan.identity(),
        "training": {
            "arms": list(ARMS),
            "shared_feature_pass_when_sequential": True,
            "independent_workers_when_concurrent": True,
            "windows": TRAIN_WINDOWS,
            "decisions": TRAIN_DECISIONS,
            "batch_size": BATCH_SIZE,
            "optimizer_steps": OPTIMIZER_STEPS,
            "train_order": _train_order_identity(preparation.predecessor),
            "separate_losses": True,
            "separate_optimizers": True,
            "separate_final_artifacts": True,
            "retry": "FORBIDDEN",
        },
        "artifacts_fixed_before_reveal": ["v1", "geometric", "pooled"],
        "post_reveal_optimization": "FORBIDDEN",
        "canonical_scoring": "CPU4",
        "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
        "canonical_scoring_reserve_seconds": scoring_reserve_seconds,
        "training_wall_seconds": HARD_WALL_CEILING_SECONDS
        - scoring_reserve_seconds,
        "cuda": "FORBIDDEN",
    }


def _load_or_create_started(
    preparation: CampaignPreparation,
    probe: Mapping[str, Any],
) -> dict[str, Any]:
    path = preparation.root / STARTED_RELATIVE_PATH
    if path.exists():
        return _load_started(preparation, probe)
    if (preparation.root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("a missing started envelope cannot be created after reveal")
    contract = _run_contract(preparation, probe)
    contract_cid = cid_bytes(canonical_json_bytes(contract))
    started = _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "run_contract": contract,
            "run_contract_cid": contract_cid,
            "writer_process_id": os.getpid(),
            "started_unix_seconds": time.time(),
            "sealed_prompt_reads": 0,
            "sealed_heldout_reads": 0,
        },
        "started_cid",
    )
    _write_exclusive_json(path, started)
    return started


def _load_started(
    preparation: CampaignPreparation,
    probe: Mapping[str, Any],
) -> dict[str, Any]:
    """Load an already-fixed start marker without any create/recovery path."""

    path = preparation.root / STARTED_RELATIVE_PATH
    if not path.exists() or path.is_symlink() or not path.is_file():
        raise FileNotFoundError("learned-associative started envelope is absent")
    contract = _run_contract(preparation, probe)
    contract_cid = cid_bytes(canonical_json_bytes(contract))
    started = _read_json(path)
    _verify_self_cid(started, "started_cid")
    if (
        started.get("schema") != STARTED_SCHEMA
        or started.get("issue") != ISSUE
        or started.get("policy") != POLICY
        or started.get("run_contract") != contract
        or started.get("run_contract_cid") != contract_cid
    ):
        raise ValueError("existing learned-associative started envelope differs")
    return started


def _write_unavailable(
    root: Path,
    *,
    preparation_cid: str,
    probe_cid: str,
    reason: Any,
) -> dict[str, Any]:
    body = {
        "schema": "uor-r4.learned-associative-readout-unavailable/1",
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation_cid,
        "probe_cid": probe_cid,
        "verdict": TERMINAL_UNAVAILABLE,
        "reason": reason,
        "reveal_created": False,
        "action": "preserve and resume only the identical pre-reveal trajectory; do not retune",
    }
    value = _with_cid(body, "unavailable_cid")
    atomic_write_json(root / UNAVAILABLE_RELATIVE_PATH, value)
    return value


def _load_prior_post_reveal_scoring_seconds(
    preparation: CampaignPreparation,
    probe: Mapping[str, Any],
    started: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
    reveal: Mapping[str, Any],
) -> float:
    """Charge prior failed scoring work while ignoring pre-reveal run notices."""

    path = preparation.root / UNAVAILABLE_RELATIVE_PATH
    if not path.exists():
        return 0.0
    value = _read_json(path)
    _verify_self_cid(value, "unavailable_cid")
    if value.get("reveal_created") is False:
        return 0.0
    timing = value.get("timing")
    scoring_seconds = timing.get("scoring_seconds") if isinstance(timing, Mapping) else None
    training_seconds = max(float(arm_results[arm]["elapsed_seconds"]) for arm in ARMS)
    if (
        value.get("schema") != "uor-r4.learned-associative-readout-unavailable/1"
        or value.get("issue") != ISSUE
        or value.get("policy") != POLICY
        or value.get("phase") != "POST_REVEAL_SCORING"
        or value.get("preparation_cid") != preparation.manifest["preparation_cid"]
        or value.get("probe_cid") != probe.get("probe_cid")
        or value.get("started_cid") != started.get("started_cid")
        or value.get("run_contract_cid") != started.get("run_contract_cid")
        or value.get("reveal_cid") != reveal.get("reveal_cid")
        or value.get("artifacts")
        != {arm: dict(arm_results[arm]["artifact"]) for arm in ARMS}
        or value.get("optimizer_created_after_reveal") is not False
        or value.get("optimizer_steps_after_reveal") != 0
        or not isinstance(timing, Mapping)
        or isinstance(scoring_seconds, bool)
        or not isinstance(scoring_seconds, (int, float))
        or not math.isfinite(float(scoring_seconds))
        or float(scoring_seconds) < 0.0
        or timing.get("training_seconds") != training_seconds
        or timing.get("total_seconds") != training_seconds + float(scoring_seconds)
        or timing.get("hard_wall_seconds") != HARD_WALL_CEILING_SECONDS
    ):
        raise ValueError("post-reveal scoring recovery record differs")
    return float(scoring_seconds)


def _write_post_reveal_unavailable(
    preparation: CampaignPreparation,
    probe: Mapping[str, Any],
    started: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
    reveal: Mapping[str, Any],
    *,
    reason: Any,
    training_seconds: float,
    scoring_seconds: float,
    scoring_timeout_seconds: float,
) -> dict[str, Any]:
    total_seconds = training_seconds + scoring_seconds
    residual = max(0.0, HARD_WALL_CEILING_SECONDS - total_seconds)
    resume_state = (
        "SCORING_ONLY_RESUME_AVAILABLE"
        if residual > RESULT_FINALIZATION_RESERVE_SECONDS
        else "NO_RESIDUAL_AGGREGATE_WALL"
    )
    body = {
        "schema": "uor-r4.learned-associative-readout-unavailable/1",
        "issue": ISSUE,
        "policy": POLICY,
        "phase": "POST_REVEAL_SCORING",
        "preparation_cid": preparation.manifest["preparation_cid"],
        "probe_cid": probe["probe_cid"],
        "started_cid": started["started_cid"],
        "run_contract_cid": started["run_contract_cid"],
        "verdict": TERMINAL_UNAVAILABLE,
        "reason": reason,
        "reveal_created": True,
        "reveal_cid": reveal["reveal_cid"],
        "artifacts": {arm: dict(arm_results[arm]["artifact"]) for arm in ARMS},
        "scoring_timeout_seconds": max(0.0, scoring_timeout_seconds),
        "timing": {
            "training_seconds": training_seconds,
            "scoring_seconds": scoring_seconds,
            "total_seconds": total_seconds,
            "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
        },
        "residual_wall_seconds": residual,
        "scoring_resume_state": resume_state,
        "optimizer_created_after_reveal": False,
        "optimizer_steps_after_reveal": 0,
        "action": (
            "preserve the fixed artifacts; resume scoring/finalization only when "
            "the durable residual wall permits it; never retrain"
        ),
    }
    value = _with_cid(body, "unavailable_cid")
    atomic_write_json(preparation.root / UNAVAILABLE_RELATIVE_PATH, value)
    return value


def _final_mechanics(
    *,
    probe: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
    evidence: Mapping[str, Any],
    reveal: Mapping[str, Any],
) -> dict[str, Any]:
    selected_name = str(probe["selection"]["selected_plan"]["name"])
    selected_records = [
        record
        for record in probe["selection"]["plans"]
        if record.get("plan", {}).get("name") == selected_name
    ]
    selected_mechanics = len(selected_records) == 1 and all(
        bool(selected_records[0].get("arms", {}).get(arm, {}).get("result", {}).get("mechanics", {}).get("passed"))
        for arm in ARMS
    )
    artifact_cids = {
        arm: arm_results[arm]["artifact"]["cid"] for arm in ARMS
    }
    prompt_scores = evidence.get("prompt_scores", {})
    fresh_language = evidence.get("fresh_language", {})
    gates = {
        "selected_probe_mechanics": selected_mechanics,
        "arm_artifact_replay_exact": all(
            float(arm_results[arm].get("artifact_replay_maximum_logits_delta", math.inf)) == 0.0
            for arm in ARMS
        ),
        "arm_forbidden_reads_zero": all(int(arm_results[arm].get("forbidden_reads", -1)) == 0 for arm in ARMS),
        "base_artifact_exact": all(arm_results[arm].get("base_artifact_cid") == PREDECESSOR_ARTIFACT_CID for arm in ARMS),
        "prompt_replay_exact": all(evidence.get("prompt_replay_exact", {}).get(name) is True for name in ("v1", "geometric", "pooled", "deranged")),
        "prompt_forbidden_reads_zero": all(int(record.get("forbidden_reads", -1)) == 0 for record in prompt_scores.values()),
        "fresh_forbidden_reads_zero": all(int(record.get("forbidden_reads", -1)) == 0 for record in fresh_language.values()),
        "reveal_artifact_binding": (
            reveal.get("baseline_artifact_cid") == PREDECESSOR_ARTIFACT_CID
            and reveal.get("geometric_artifact_cid") == artifact_cids["geometric"]
            and reveal.get("pooled_artifact_cid") == artifact_cids["pooled"]
            and reveal.get("fresh_heldout_cid") == FRESH_HELDOUT_CID
        ),
        "population_binding": reveal.get("population_cid") == evidence.get("population_cid"),
        "post_reveal_optimizer_steps_zero": True,
    }
    return {
        "passed": all(gates.values()),
        "gates": gates,
        "optimizer_created_after_reveal": False,
        "optimizer_steps_after_reveal": 0,
    }


def _load_result(root: Path) -> dict[str, Any]:
    result = _read_json(root / RESULT_RELATIVE_PATH)
    _verify_self_cid(result, "result_cid")
    if (
        result.get("schema") != RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
        or not isinstance(result.get("decision"), Mapping)
        or result.get("verdict") != result["decision"].get("verdict")
    ):
        raise ValueError("learned-associative terminal result differs")
    _validate_result_bindings(root, result)
    return result


def _validate_result_bindings(root: Path, result: Mapping[str, Any]) -> None:
    """Reproduce every immutable input before accepting a cached result."""

    preparation = load_learned_associative_readout_preparation(root)
    probe = probe_learned_associative_readout(root)
    started = _load_started(preparation, probe)
    contract = _run_contract(preparation, probe)
    contract_cid = cid_bytes(canonical_json_bytes(contract))
    plan = _selected_plan(probe)
    arm_results = {
        arm: _load_arm_result(
            root,
            arm,
            run_contract_cid=contract_cid,
            plan_cid=plan.identity()["plan_cid"],
        )
        for arm in ARMS
    }
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    artifacts = result.get("artifacts")
    writer_process_id = result.get("writer_process_id")
    expected_v1 = {
        "path": str(preparation.predecessor_artifact_path),
        "bytes": PREDECESSOR_ARTIFACT_BYTES,
        "cid": PREDECESSOR_ARTIFACT_CID,
    }
    if (
        isinstance(writer_process_id, bool)
        or not isinstance(writer_process_id, int)
        or result.get("model_policy") != MODEL_POLICY
        or result.get("preparation_cid") != preparation.manifest["preparation_cid"]
        or result.get("probe_cid") != probe["probe_cid"]
        or result.get("started_cid") != started["started_cid"]
        or result.get("run_contract_cid") != contract_cid
        or result.get("implementation") != preparation.manifest["implementation"]
        or result.get("reveal") != reveal
        or result.get("arm_results") != arm_results
        or not isinstance(artifacts, Mapping)
        or artifacts.get("v1") != expected_v1
        or any(artifacts.get(arm) != arm_results[arm]["artifact"] for arm in ARMS)
        or reveal.get("baseline_artifact_cid") != PREDECESSOR_ARTIFACT_CID
        or reveal.get("geometric_artifact_cid")
        != arm_results["geometric"]["artifact"]["cid"]
        or reveal.get("pooled_artifact_cid")
        != arm_results["pooled"]["artifact"]["cid"]
        or reveal.get("fresh_heldout_cid") != FRESH_HELDOUT_CID
    ):
        raise ValueError("cached terminal result immutable bindings differ")
    _validate_result_timing(result, arm_results)


def _validate_result_timing(
    result: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
) -> dict[str, float]:
    """Reproduce aggregate timing exactly from the two durable arm ledgers."""

    timing = result.get("timing")
    if not isinstance(timing, Mapping) or set(timing) != {
        "training_seconds",
        "scoring_seconds",
        "total_seconds",
        "hard_wall_seconds",
    }:
        raise ValueError("terminal timing fields differ")
    arm_elapsed: list[float] = []
    for arm in ARMS:
        value = arm_results[arm].get("elapsed_seconds")
        if (
            isinstance(value, bool)
            or not isinstance(value, (int, float))
            or not math.isfinite(float(value))
            or float(value) < 0.0
        ):
            raise ValueError(f"{arm} elapsed time differs")
        arm_elapsed.append(float(value))
    scoring = timing.get("scoring_seconds")
    if (
        isinstance(scoring, bool)
        or not isinstance(scoring, (int, float))
        or not math.isfinite(float(scoring))
        or float(scoring) < 0.0
    ):
        raise ValueError("terminal scoring time differs")
    training = max(arm_elapsed)
    total = training + float(scoring)
    if (
        timing.get("training_seconds") != training
        or timing.get("total_seconds") != total
        or timing.get("hard_wall_seconds") != HARD_WALL_CEILING_SECONDS
    ):
        raise ValueError("terminal aggregate timing does not reproduce")
    return {
        "training_seconds": training,
        "scoring_seconds": float(scoring),
        "total_seconds": total,
        "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
    }


def _decision_from_evidence(
    evidence: Mapping[str, Any],
    mechanics: Mapping[str, Any],
    *,
    total_seconds: float,
) -> dict[str, Any]:
    capacity = evidence["capacity_decisions"]
    geometry = evidence["geometry_attribution"]
    fresh = evidence["fresh_gates"]
    decision = terminal_decision(
        geometric_capacity_verdict=str(capacity["geometric"]["verdict"]),
        pooled_capacity_verdict=str(capacity["pooled"]["verdict"]),
        geometry_verdict=str(geometry["verdict"]),
        geometric_fresh_passed=bool(fresh["geometric"]["passed"]),
        pooled_fresh_passed=bool(fresh["pooled"]["passed"]),
        geometric_fresh_nll=float(evidence["fresh_language"]["geometric"]["ce_nats"]),
        pooled_fresh_nll=float(evidence["fresh_language"]["pooled"]["ce_nats"]),
        mechanics_passed=bool(mechanics["passed"]),
    )
    if total_seconds > HARD_WALL_CEILING_SECONDS:
        return {
            "verdict": TERMINAL_UNAVAILABLE,
            "action": "preserve fixed artifacts; permit scoring-only recovery without any optimizer",
            "selected_arm": None,
        }
    return decision


def _finalize_result(
    preparation: CampaignPreparation,
    probe: Mapping[str, Any],
    started: Mapping[str, Any],
    arm_results: Mapping[str, Mapping[str, Any]],
    *,
    scoring_executor: Callable[
        [
            CampaignPreparation,
            Mapping[str, Any],
            Mapping[str, Mapping[str, Any]],
            float,
        ],
        dict[str, Any],
    ]
    | None = None,
) -> dict[str, Any]:
    result_path = preparation.root / RESULT_RELATIVE_PATH
    if result_path.exists():
        return _load_result(preparation.root)
    reveal = _read_json(preparation.root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    arm_elapsed_seconds: list[float] = []
    for arm in ARMS:
        value = arm_results[arm].get("elapsed_seconds")
        if (
            isinstance(value, bool)
            or not isinstance(value, (int, float))
            or not math.isfinite(float(value))
            or float(value) < 0.0
        ):
            raise ValueError(f"{arm} elapsed time cannot bind the scoring deadline")
        arm_elapsed_seconds.append(float(value))
    training_seconds = max(arm_elapsed_seconds)
    prior_scoring_seconds = _load_prior_post_reveal_scoring_seconds(
        preparation,
        probe,
        started,
        arm_results,
        reveal,
    )
    residual_seconds = (
        HARD_WALL_CEILING_SECONDS - training_seconds - prior_scoring_seconds
    )
    scoring_timeout_seconds = max(
        0.0, residual_seconds - RESULT_FINALIZATION_RESERVE_SECONDS
    )
    if scoring_timeout_seconds <= 0.0:
        return _write_post_reveal_unavailable(
            preparation,
            probe,
            started,
            arm_results,
            reveal,
            reason="NO_RESIDUAL_AGGREGATE_WALL_FOR_CANONICAL_SCORING",
            training_seconds=training_seconds,
            scoring_seconds=prior_scoring_seconds,
            scoring_timeout_seconds=scoring_timeout_seconds,
        )
    executor = scoring_executor or _spawned_scoring_executor
    scoring_started = time.monotonic()
    outcome = executor(
        preparation,
        reveal,
        arm_results,
        scoring_timeout_seconds,
    )
    scoring_wall_seconds = time.monotonic() - scoring_started
    if outcome.get("ok") is not True:
        return _write_post_reveal_unavailable(
            preparation,
            probe,
            started,
            arm_results,
            reveal,
            reason=outcome.get("error", "CANONICAL_SCORING_UNAVAILABLE"),
            training_seconds=training_seconds,
            scoring_seconds=prior_scoring_seconds + scoring_wall_seconds,
            scoring_timeout_seconds=scoring_timeout_seconds,
        )
    evidence = outcome.get("evidence")
    reported_scoring_seconds = outcome.get("scoring_seconds", 0.0)
    if (
        not isinstance(evidence, Mapping)
        or isinstance(reported_scoring_seconds, bool)
        or not isinstance(reported_scoring_seconds, (int, float))
        or not math.isfinite(float(reported_scoring_seconds))
        or float(reported_scoring_seconds) < 0.0
    ):
        raise ValueError("canonical scoring executor returned invalid evidence")
    mechanics = _final_mechanics(
        probe=probe,
        arm_results=arm_results,
        evidence=evidence,
        reveal=reveal,
    )
    # A cached completed child score is already represented by a durable prior
    # timeout ledger, when present.  Otherwise charge at least the child's
    # measured work.  Always charge this process's recovery/finalization wall.
    reported_charge = (
        0.0
        if outcome.get("cached") is True and prior_scoring_seconds > 0.0
        else float(reported_scoring_seconds)
    )
    scoring_wall_seconds = time.monotonic() - scoring_started
    scoring_seconds = (
        prior_scoring_seconds
        + max(scoring_wall_seconds, reported_charge)
        + RESULT_FINALIZATION_RESERVE_SECONDS
    )
    total_seconds = training_seconds + scoring_seconds
    if total_seconds > HARD_WALL_CEILING_SECONDS:
        return _write_post_reveal_unavailable(
            preparation,
            probe,
            started,
            arm_results,
            reveal,
            reason="AGGREGATE_WALL_EXHAUSTED_DURING_CANONICAL_SCORING",
            training_seconds=training_seconds,
            scoring_seconds=scoring_seconds,
            scoring_timeout_seconds=scoring_timeout_seconds,
        )
    decision = _decision_from_evidence(
        evidence,
        mechanics,
        total_seconds=total_seconds,
    )
    body = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "model_policy": MODEL_POLICY,
        "writer_process_id": os.getpid(),
        "preparation_cid": preparation.manifest["preparation_cid"],
        "probe_cid": probe["probe_cid"],
        "started_cid": started["started_cid"],
        "run_contract_cid": started["run_contract_cid"],
        "implementation": preparation.manifest["implementation"],
        "artifacts": {
            "v1": {
                "path": str(preparation.predecessor_artifact_path),
                "bytes": PREDECESSOR_ARTIFACT_BYTES,
                "cid": PREDECESSOR_ARTIFACT_CID,
            },
            **{arm: dict(arm_results[arm]["artifact"]) for arm in ARMS},
        },
        "arm_results": {arm: dict(arm_results[arm]) for arm in ARMS},
        "reveal": reveal,
        "evidence": evidence,
        "mechanics": mechanics,
        "decision": decision,
        "verdict": decision["verdict"],
        "timing": {
            "training_seconds": training_seconds,
            "scoring_seconds": scoring_seconds,
            "total_seconds": total_seconds,
            "hard_wall_seconds": HARD_WALL_CEILING_SECONDS,
        },
        "nonclaims": {
            "generation": "NOT_RUN",
            "reasoning": "NOT_RUN",
            "lowering": "NOT_RUN",
            "geometry_native_lowering": "NOT_RUN",
            "transformerless_general_model": "NOT_ESTABLISHED",
        },
    }
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(result_path, result)
    return result


def run_learned_associative_readout(root: Path, resume: bool = False) -> dict[str, Any]:
    """Run the single frozen fit, reveal once, and score without post-reveal fit."""

    root = root.resolve()
    if (root / RESULT_RELATIVE_PATH).exists():
        return _load_result(root)
    preparation = load_learned_associative_readout_preparation(root)
    probe = probe_learned_associative_readout(root)
    if probe.get("eligible") is not True:
        return _write_unavailable(
            root,
            preparation_cid=preparation.manifest["preparation_cid"],
            probe_cid=probe["probe_cid"],
            reason="NO_ELIGIBLE_EXECUTION_PLAN",
        )
    plan = _selected_plan(probe)
    plan_cid = plan.identity()["plan_cid"]
    if (root / REVEAL_RELATIVE_PATH).exists():
        if not resume:
            raise RuntimeError("post-reveal finalization requires --resume and cannot train")
        started = _load_started(preparation, probe)
        run_contract_cid = str(started["run_contract_cid"])
        arm_results = {
            arm: _load_arm_result(root, arm, run_contract_cid=run_contract_cid, plan_cid=plan_cid)
            for arm in ARMS
        }
        return _finalize_result(preparation, probe, started, arm_results)

    started = _load_or_create_started(preparation, probe)
    run_contract_cid = str(started["run_contract_cid"])
    outcomes = _spawned_arm_runner(
        root,
        plan,
        run_contract_cid=run_contract_cid,
        resume=resume,
        wall_seconds=float(started["run_contract"]["training_wall_seconds"]),
    )
    if any(
        not outcomes.get(arm, {}).get("ok")
        or outcomes[arm].get("result", {}).get("status") != "COMPLETE"
        for arm in ARMS
    ):
        return _write_unavailable(
            root,
            preparation_cid=preparation.manifest["preparation_cid"],
            probe_cid=probe["probe_cid"],
            reason=outcomes,
        )
    arm_results = {arm: outcomes[arm]["result"] for arm in ARMS}
    for arm in ARMS:
        _load_arm_result(root, arm, run_contract_cid=run_contract_cid, plan_cid=plan_cid)
    actual_training_seconds = max(
        float(arm_results[arm]["elapsed_seconds"]) for arm in ARMS
    )
    if actual_training_seconds > float(
        started["run_contract"]["training_wall_seconds"]
    ):
        return _write_unavailable(
            root,
            preparation_cid=preparation.manifest["preparation_cid"],
            probe_cid=probe["probe_cid"],
            reason="AGGREGATE_SCORING_RESERVE_EXHAUSTED_BEFORE_REVEAL",
        )
    reveal_prompt_conditioning_population(
        root,
        baseline_artifact_cid=PREDECESSOR_ARTIFACT_CID,
        geometric_artifact_cid=str(arm_results["geometric"]["artifact"]["cid"]),
        pooled_artifact_cid=str(arm_results["pooled"]["artifact"]["cid"]),
    )
    return _finalize_result(preparation, probe, started, arm_results)


def verify_learned_associative_readout_result(root: Path) -> dict[str, Any]:
    """Independently replay all terminal scores without creating an optimizer."""

    root = root.resolve()
    result = _load_result(root)
    writer_process_id = result.get("writer_process_id")
    if (
        isinstance(writer_process_id, bool)
        or not isinstance(writer_process_id, int)
        or writer_process_id == os.getpid()
    ):
        raise ValueError("independent verification requires a fresh process")
    path = root / VERIFICATION_RELATIVE_PATH
    if path.exists():
        verification = _read_json(path)
        _verify_self_cid(verification, "verification_cid")
        comparisons = verification.get("comparisons")
        verifier_process_id = verification.get("verifier_process_id")
        if (
            verification.get("schema") != VERIFICATION_SCHEMA
            or verification.get("result_cid") != result.get("result_cid")
            or verification.get("preparation_cid") != result.get("preparation_cid")
            or verification.get("probe_cid") != result.get("probe_cid")
            or verification.get("reveal_cid")
            != result.get("reveal", {}).get("reveal_cid")
            or verification.get("passed") is not True
            or verification.get("optimizer_created") is not False
            or verification.get("optimizer_steps") != 0
            or verification.get("training_batches_scored") != 0
            or verification.get("writer_process_id") != writer_process_id
            or isinstance(verifier_process_id, bool)
            or not isinstance(verifier_process_id, int)
            or verifier_process_id == writer_process_id
            or not isinstance(comparisons, Mapping)
            or set(comparisons)
            != {"evidence_exact", "mechanics_exact", "terminal_decision_exact"}
            or any(value is not True for value in comparisons.values())
        ):
            raise ValueError("cached independent verification differs")
        return verification
    preparation = load_learned_associative_readout_preparation(root)
    probe = probe_learned_associative_readout(root)
    started = _load_started(preparation, probe)
    plan = _selected_plan(probe)
    arm_results = {
        arm: _load_arm_result(
            root,
            arm,
            run_contract_cid=str(started["run_contract_cid"]),
            plan_cid=plan.identity()["plan_cid"],
        )
        for arm in ARMS
    }
    evidence = _score_campaign(
        preparation,
        geometric_artifact=_artifact_path(root, "geometric").read_bytes(),
        pooled_artifact=_artifact_path(root, "pooled").read_bytes(),
    )
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    artifacts = result.get("artifacts")
    if (
        result.get("preparation_cid") != preparation.manifest["preparation_cid"]
        or result.get("probe_cid") != probe["probe_cid"]
        or result.get("started_cid") != started["started_cid"]
        or result.get("run_contract_cid") != started["run_contract_cid"]
        or result.get("implementation") != preparation.manifest["implementation"]
        or result.get("reveal") != reveal
        or not isinstance(artifacts, Mapping)
        or artifacts.get("v1", {}).get("cid") != PREDECESSOR_ARTIFACT_CID
        or any(
            artifacts.get(arm) != arm_results[arm].get("artifact")
            for arm in ARMS
        )
    ):
        raise ValueError("terminal result artifact/reveal/run binding differs")
    mechanics = _final_mechanics(
        probe=probe,
        arm_results=arm_results,
        evidence=evidence,
        reveal=reveal,
    )
    timing = _validate_result_timing(result, arm_results)
    decision = _decision_from_evidence(
        evidence,
        mechanics,
        total_seconds=float(timing["total_seconds"]),
    )
    science_exact = (
        evidence == result.get("evidence")
        and mechanics == result.get("mechanics")
        and decision == result.get("decision")
        and result.get("verdict") == decision["verdict"]
    )
    body = {
        "schema": VERIFICATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "result_cid": result["result_cid"],
        "preparation_cid": preparation.manifest["preparation_cid"],
        "probe_cid": probe["probe_cid"],
        "reveal_cid": reveal["reveal_cid"],
        "writer_process_id": writer_process_id,
        "verifier_process_id": os.getpid(),
        "fresh_model_instances": True,
        "optimizer_created": False,
        "optimizer_steps": 0,
        "training_batches_scored": 0,
        "comparisons": {
            "evidence_exact": evidence == result.get("evidence"),
            "mechanics_exact": mechanics == result.get("mechanics"),
            "terminal_decision_exact": decision == result.get("decision")
            and result.get("verdict") == decision["verdict"],
        },
        "passed": science_exact,
    }
    if not science_exact:
        raise ValueError("independent learned-associative evidence differs")
    verification = _with_cid(body, "verification_cid")
    _write_exclusive_json(path, verification)
    return verification
