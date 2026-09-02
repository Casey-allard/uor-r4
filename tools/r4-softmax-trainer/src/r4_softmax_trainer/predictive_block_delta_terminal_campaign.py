"""Create-once V5 terminal campaign for predictive R4 block-delta memory.

The module enforces three phases.  Preparation deterministically selects and
seals the unopened V5 prompt contrast and fresh-language slice.  A
construction-only probe chooses among Apple Accelerate CPU4, CPU8, and two
CPU4 workers.  Exactly three byte-identically initialized arms are then fit on
the qualified construction order: geometric delta, plain delta, and geometric
additive/Hebbian.  Only after all three artifact CIDs are fixed is V5 revealed
and scored in a fresh process.  No optimizer is constructed after reveal.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import platform
import shutil
import stat
import statistics
import struct
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

from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization import CONTEXT, VOCAB_SIZE, R4RetainedLanguagePathV1
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
from .language_path_generalization_data import LanguagePathWindowStore
from .language_path_generalization_data import (
    EXPECTED_SOURCE_TRAIN_STORE_CID as FRESH_HELDOUT_SOURCE_TRAIN_CID,
    SOURCE_TRAIN_TOKENS as FRESH_HELDOUT_SOURCE_TRAIN_TOKENS,
)
from .layerwise_normalized_retained_readout_campaign import (
    PREDECESSOR_ARTIFACT_BYTES,
    PREDECESSOR_ARTIFACT_CID,
    PREDECESSOR_POLICY,
    PREDECESSOR_RESULT_CID,
    _verify_predecessor,
)
from .learned_associative_readout import (
    LearnedAssociativeReadoutAudit,
    R4LearnedCandidateLeafAssociativeReadoutV1,
)
from .predictive_block_delta_binding import (
    INITIALIZATION_SEED,
    TRAINABLE_PARAMETER_COUNT,
    R4PredictiveBlockDeltaBindingV1,
)
from .predictive_block_delta_campaign import (
    ABSOLUTE_GAIN_THRESHOLD,
    H4_FRAME_ARTIFACT_CID,
    H4_FRAME_FILE_CID,
    INTERVENTION_LOSS_THRESHOLD,
    V4_POOLED_ARTIFACT_CID,
    _output,
    transport_mechanics,
)
from .predictive_block_delta_campaign_v2 import (
    RESULT_RELATIVE_PATH as V2_RESULT_RELATIVE_PATH,
    _validate_cached_result as _validate_v2_result,
)
from .prompt_conditioning_v5 import (
    BOS_TOKEN_ID,
    CONTINUATION_TOKENS,
    DIRECTION_COUNT,
    PAIR_COUNT,
    POPULATION_SCHEMA,
    PROMPT_TOKENS,
    SCORED_TARGET_TOKENS,
    PromptConditioningPopulationV5,
    load_required_prior_story_cids,
    prompt_directions,
    select_prompt_conditioning_population_from_source,
)
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    tree_cid,
    trainer_implementation_contract,
)


ISSUE = 973
POLICY = "R4PredictiveBlockDeltaPromptCapacityV5"
MODEL_POLICY = "R4PredictiveBlockDeltaBindingV1"
ARMS = ("geometric", "plain", "additive")
Arm = Literal["geometric", "plain", "additive"]

V2_RESULT_CID = (
    "blake3:623bbd63321c18ad7e4172b325b2d22518b6b10a33f755d3bbbbdcf9b9c51637"
)
V4_RESULT_CID = (
    "blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1"
)
V4_RESULT_RELATIVE_PATH = "run/learned-associative-readout-result.json"
V4_POOLED_ARTIFACT_RELATIVE_PATH = "arms/pooled/head.safetensors"

FRESH_HELDOUT_SOURCE_OFFSET_TOKENS = 156_282_226
FRESH_HELDOUT_TOKENS = 249_986
FRESH_HELDOUT_WINDOWS = 2_066
FRESH_HELDOUT_DECISIONS = FRESH_HELDOUT_WINDOWS * CONTEXT
FRESH_HELDOUT_FIRST_CAPACITY_STORY = 765_248
FRESH_HELDOUT_FIRST_SOURCE_STORY = 849_803
FRESH_HELDOUT_LAST_CAPACITY_STORY = 766_489
FRESH_HELDOUT_LAST_SOURCE_STORY = 851_190
FRESH_HELDOUT_STORY_CIDS = 1_242
FRESH_HELDOUT_TRAIN_INDEX_CID = (
    "blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e"
)

PREPARATION_RELATIVE_PATH = "predictive-block-delta-v5-preparation.json"
COMMITMENT_RELATIVE_PATH = "evaluation/commitment.json"
REVEAL_RELATIVE_PATH = "evaluation/reveal.json"
REVEAL_TRANSITION_RELATIVE_PATH = "evaluation/reveal-transition.json"
POPULATION_RELATIVE_PATH = "evaluation/sealed/prompt-population.json"
HELDOUT_RELATIVE_PATH = "evaluation/sealed/fresh-heldout.u16"
PROBE_RELATIVE_PATH = "preflight/predictive-block-delta-v5-execution-probe.json"
STARTED_RELATIVE_PATH = "run/predictive-block-delta-v5-started.json"
FIT_BUDGET_RELATIVE_PATH = "run/predictive-block-delta-v5-fit-budget.json"
SCORING_RELATIVE_PATH = "run/predictive-block-delta-v5-scoring-evidence.json"
RESULT_RELATIVE_PATH = "run/predictive-block-delta-v5-result.json"
VERIFICATION_RELATIVE_PATH = "run/predictive-block-delta-v5-independent-verification.json"
UNAVAILABLE_RELATIVE_PATH = "run/predictive-block-delta-v5-unavailable.json"
SCORING_RECOVERY_RELATIVE_PATH = (
    "run/predictive-block-delta-v5-scoring-recovery.json"
)
SCORING_RECOVERY_UNAVAILABLE_RELATIVE_PATH = (
    "run/predictive-block-delta-v5-scoring-recovery-unavailable.json"
)

PREPARATION_SCHEMA = "uor-r4.predictive-block-delta-v5-preparation/1"
COMMITMENT_SCHEMA = "uor-r4.predictive-block-delta-v5-commitment/1"
REVEAL_SCHEMA = "uor-r4.predictive-block-delta-v5-reveal/1"
REVEAL_TRANSITION_SCHEMA = "uor-r4.predictive-block-delta-v5-reveal-transition/1"
PROBE_SCHEMA = "uor-r4.predictive-block-delta-v5-execution-probe/1"
STARTED_SCHEMA = "uor-r4.predictive-block-delta-v5-started/1"
FIT_BUDGET_SCHEMA = "uor-r4.predictive-block-delta-v5-fit-budget/1"
ARM_RESULT_SCHEMA = "uor-r4.predictive-block-delta-v5-arm-result/1"
SCORING_SCHEMA = "uor-r4.predictive-block-delta-v5-scoring-evidence/1"
RESULT_SCHEMA = "uor-r4.predictive-block-delta-v5-result/1"
VERIFICATION_SCHEMA = "uor-r4.predictive-block-delta-v5-verification/1"
SCORING_RECOVERY_SCHEMA = "uor-r4.predictive-block-delta-v5-scoring-recovery/1"

# The first V5 scoring attempt exposed a batch-tail accounting defect only
# after all arms and the reveal had been frozen.  These exact identities allow
# the corrected scorer to consume that immutable fit without relabelling any
# other historical implementation as current.
FROZEN_V5_FIT_IMPLEMENTATION_TREE_CID = (
    "blake3:000a3ae8a69ba9185ff66ee58ff891b3eb22ab857195d71d38441e277cceca24"
)
FROZEN_V5_PREPARATION_CID = (
    "blake3:1e65392c729ca349b2a9a61f4bfb503e5cb32392f42f69d7f4b836ea7692d10a"
)
FROZEN_V5_COMMITMENT_CID = (
    "blake3:8e9c02068bb1dfef956907b1b614ddb0c4fcf902262fc934f8b098f5fd7cf0c4"
)
FROZEN_V5_PROBE_CID = (
    "blake3:7adc13f30955b8843674d5a9b410500046fdd5376422979ff8f69f547c32aa08"
)
FROZEN_V5_STARTED_CID = (
    "blake3:c4c1dacb4e99a955c1d4777064cda0191aeecab7543e8e76a23a5b01d5c758a6"
)
FROZEN_V5_REVEAL_CID = (
    "blake3:6773e5ec1be496a5d1edae29f810d3b13a05b3953757b31ea22f909471ae5800"
)
FROZEN_V5_UNAVAILABLE_CID = (
    "blake3:a819ed7f2b558d80053362c6c229642835b1317ff367d576aeb6ab23a592536a"
)
FROZEN_V5_FIT_BUDGET_CID = (
    "blake3:cb5a1f1640ea08882542423721719c31b0044ee679ec06ba51a589f3c400ea3d"
)
FROZEN_V5_ARM_RESULT_CIDS = {
    "geometric": "blake3:c8b62dba59c23a93d04aa60cebfffe5c366bb4ce6b8ee48b64088bef4db77b60",
    "plain": "blake3:0c91f859e2d05e77dc81e8f17ae5c40e72d23d7bdb7c64fd1f03a7e727cbfc87",
    "additive": "blake3:e32101f0ff89e3b4099e9f23645873e2fb80d22478cce9f5bb52a8ec3debe155",
}

PROMPT_GAIN_THRESHOLD = ABSOLUTE_GAIN_THRESHOLD
INCREMENTAL_GAIN_THRESHOLD = INTERVENTION_LOSS_THRESHOLD
WIN_THRESHOLD = 308
FRESH_NLL_TOLERANCE = 0.05
FRESH_TOP1_POINT_TOLERANCE = 1.0
HARD_WALL_SECONDS = 3_600.0
SCORING_HARD_WALL_SECONDS = 1_800.0
DIRECTION_BATCH_SIZE = 8
PROMPT_SCORING_PASSES = 11
FRESH_SCORING_PASSES = 4

# The public compute contract excludes MPS and CUDA.  The third arm added by
# the matched-control correction is queued deterministically through the same
# two-worker plan; no plan silently creates a third worker.
ELIGIBLE_PLANS = (
    ExecutionPlan("cpu-accelerate-4t-sequential", "cpu", 4, 1, False),
    ExecutionPlan("cpu-accelerate-8t-sequential", "cpu", 8, 1, False),
    ExecutionPlan("cpu-accelerate-2x4t-concurrent", "cpu", 4, 2, True),
)


@dataclass(frozen=True, slots=True)
class TerminalPreparation:
    root: Path
    manifest: dict[str, Any]
    commitment: dict[str, Any]
    predecessor: Any
    predecessor_artifact_path: Path
    frames: H4SpinFrameArtifactV1
    pooled_artifact_path: Path


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _validate_bound_implementation(
    value: object,
    *,
    envelope_cid: object,
    frozen_envelope_cid: str,
) -> dict[str, Any]:
    """Accept current code or the one exact pre-reveal V5 fit implementation."""

    if not isinstance(value, Mapping) or set(value) != {"files", "tree_cid"}:
        raise ValueError("V5 implementation binding is malformed")
    files = value.get("files")
    observed = value.get("tree_cid")
    if (
        not isinstance(files, list)
        or not files
        or not isinstance(observed, str)
        or not observed.startswith("blake3:")
        or len(observed) != 71
    ):
        raise ValueError("V5 implementation binding is malformed")
    paths: list[str] = []
    for record in files:
        if not isinstance(record, Mapping) or set(record) != {"bytes", "cid", "path"}:
            raise ValueError("V5 implementation file ledger is malformed")
        size = record.get("bytes")
        cid = record.get("cid")
        path = record.get("path")
        if (
            isinstance(size, bool)
            or not isinstance(size, int)
            or size < 1
            or not isinstance(cid, str)
            or not cid.startswith("blake3:")
            or len(cid) != 71
            or not isinstance(path, str)
            or not path
        ):
            raise ValueError("V5 implementation file ledger is malformed")
        paths.append(path)
    implementation = {"files": [dict(record) for record in files], "tree_cid": observed}
    if paths != sorted(set(paths)) or tree_cid(implementation["files"]) != observed:
        raise ValueError("V5 implementation tree CID does not reproduce")
    if implementation == trainer_implementation_contract():
        return implementation
    if (
        envelope_cid == frozen_envelope_cid
        and observed == FROZEN_V5_FIT_IMPLEMENTATION_TREE_CID
    ):
        return implementation
    raise ValueError("V5 implementation binding is neither current nor frozen")


def _read_json(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"expected a regular non-symlink JSON file: {path}")
    payload = path.read_bytes()
    try:
        value = json.loads(payload.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot decode canonical JSON: {path}") from error
    if not isinstance(value, dict) or canonical_json_bytes(value) != payload:
        raise ValueError(f"JSON is not canonical: {path}")
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
        raise ValueError("V5 heldout source must be a regular non-symlink file")
    if (
        path.stat().st_size != FRESH_HELDOUT_SOURCE_TRAIN_TOKENS * 2
        or cid_file(path) != FRESH_HELDOUT_SOURCE_TRAIN_CID
    ):
        raise ValueError("V5 heldout source token store differs from #1019")
    byte_offset = offset_tokens * 2
    byte_count = token_count * 2
    if offset_tokens < 0 or token_count < 1 or byte_offset + byte_count > path.stat().st_size:
        raise ValueError("V5 heldout coordinates cross the source store")
    with path.open("rb") as source:
        source.seek(byte_offset)
        payload = source.read(byte_count)
    if len(payload) != byte_count:
        raise ValueError("V5 heldout source ended inside the frozen slice")
    return payload


def _verify_fresh_index(path: Path) -> tuple[dict[str, Any], frozenset[str]]:
    if path.is_symlink() or not path.is_file() or cid_file(path) != FRESH_HELDOUT_TRAIN_INDEX_CID:
        raise ValueError("V5 train-story index differs from the freeze")
    wanted = set(
        range(
            FRESH_HELDOUT_FIRST_CAPACITY_STORY,
            FRESH_HELDOUT_LAST_CAPACITY_STORY + 1,
        )
    )
    records: dict[int, dict[str, Any]] = {}
    with path.open("rb") as source:
        for ordinal, line in enumerate(source):
            if ordinal not in wanted:
                continue
            try:
                record = json.loads(line.decode("utf-8"))
            except (UnicodeError, json.JSONDecodeError) as error:
                raise ValueError("V5 train-story index is malformed") from error
            if not isinstance(record, dict) or canonical_json_bytes(record) != line:
                raise ValueError("V5 train-story record is not canonical")
            records[ordinal] = record
            if len(records) == len(wanted):
                break
    if set(records) != wanted:
        raise ValueError("V5 fresh-language story range is absent")
    first = records[FRESH_HELDOUT_FIRST_CAPACITY_STORY]
    last = records[FRESH_HELDOUT_LAST_CAPACITY_STORY]
    end_offset = FRESH_HELDOUT_SOURCE_OFFSET_TOKENS + FRESH_HELDOUT_TOKENS
    if (
        first.get("capacity_story_ordinal")
        != FRESH_HELDOUT_FIRST_CAPACITY_STORY
        or first.get("source_story_ordinal") != FRESH_HELDOUT_FIRST_SOURCE_STORY
        or first.get("story_token_offset") != FRESH_HELDOUT_SOURCE_OFFSET_TOKENS
        or last.get("capacity_story_ordinal") != FRESH_HELDOUT_LAST_CAPACITY_STORY
        or last.get("source_story_ordinal") != FRESH_HELDOUT_LAST_SOURCE_STORY
        or not isinstance(last.get("story_token_offset"), int)
        or not isinstance(last.get("story_token_count"), int)
        or not int(last["story_token_offset"]) < end_offset
        <= int(last["story_token_offset"]) + int(last["story_token_count"])
    ):
        raise ValueError("V5 fresh-language story boundaries differ")
    story_cids = frozenset(str(record.get("story_cid")) for record in records.values())
    if len(story_cids) != FRESH_HELDOUT_STORY_CIDS or any(
        not value.startswith("blake3:") for value in story_cids
    ):
        raise ValueError("V5 fresh-language story-CID coverage differs")
    story_cids_cid = cid_bytes(canonical_json_bytes(sorted(story_cids)))
    return {
        "path": str(path.resolve()),
        "cid": FRESH_HELDOUT_TRAIN_INDEX_CID,
        "first_capacity_story_ordinal": FRESH_HELDOUT_FIRST_CAPACITY_STORY,
        "first_source_story_ordinal": FRESH_HELDOUT_FIRST_SOURCE_STORY,
        "last_capacity_story_ordinal": FRESH_HELDOUT_LAST_CAPACITY_STORY,
        "last_source_story_ordinal": FRESH_HELDOUT_LAST_SOURCE_STORY,
        "story_cids": FRESH_HELDOUT_STORY_CIDS,
        "story_cids_cid": story_cids_cid,
    }, story_cids


def _verify_v2_authorization(path: Path) -> dict[str, Any]:
    result = _read_json(path)
    _validate_v2_result(result)
    production = result.get("production_v5")
    if (
        result.get("result_cid") != V2_RESULT_CID
        or result.get("admitted") is not True
        or not isinstance(production, Mapping)
        or production.get("authorized") is not True
        or production.get("created") is not False
        or production.get("inspected") is not False
    ):
        raise ValueError("V2 does not exactly authorize the V5 terminal campaign")
    return result


def _verify_pooled_comparator(root: Path) -> tuple[dict[str, Any], Path]:
    result_path = root / V4_RESULT_RELATIVE_PATH
    result = _read_json(result_path)
    _verify_self_cid(result, "result_cid")
    artifact_path = root / V4_POOLED_ARTIFACT_RELATIVE_PATH
    if (
        result.get("result_cid") != V4_RESULT_CID
        or artifact_path.is_symlink()
        or not artifact_path.is_file()
        or cid_file(artifact_path) != V4_POOLED_ARTIFACT_CID
    ):
        raise ValueError("frozen pooled V4 comparator differs")
    return result, artifact_path.resolve()


def _sealed_record(path: Path, *, relative_path: str) -> dict[str, Any]:
    return {
        "path": relative_path,
        "bytes": path.stat().st_size,
        "cid": cid_file(path),
    }


def _training_contract(predecessor: Any) -> dict[str, Any]:
    return {
        "objective": "mean causal cross-entropy over all 120 next-token positions",
        "windows": TRAIN_WINDOWS,
        "decisions": TRAIN_DECISIONS,
        "batch_size": BATCH_SIZE,
        "optimizer_steps_per_arm": OPTIMIZER_STEPS,
        "train_order": _train_order_identity(predecessor),
        "initialization_seed": INITIALIZATION_SEED,
        "trainable_parameters_per_arm": TRAINABLE_PARAMETER_COUNT,
        "arms": list(ARMS),
        "optimizer": {
            "name": "AdamW",
            "warmup_steps": WARMUP_STEPS,
            "maximum_learning_rate": MAXIMUM_LEARNING_RATE,
            "minimum_learning_rate": MINIMUM_LEARNING_RATE,
            "schedule": "linear warmup then cosine decay",
            "betas": [ADAM_BETA1, ADAM_BETA2],
            "epsilon": ADAM_EPSILON,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip": GRADIENT_CLIP,
        },
    }


def _scope_contract() -> dict[str, Any]:
    return {
        "cuda": "FORBIDDEN",
        "v5_inspected": False,
        "generation": "NOT_RUN",
        "reasoning": "NOT_RUN",
        "lowering": "NOT_RUN",
        "hard_wall_seconds_before_scoring": HARD_WALL_SECONDS,
    }


def prepare_predictive_block_delta_terminal(
    *,
    root: Path,
    predecessor_root: Path,
    source_train_path: Path,
    source_train_index_path: Path,
    raw_source_path: Path,
    prior_population_paths: Sequence[Path],
    frame_sidecar_path: Path,
    v2_result_path: Path,
    pooled_comparator_root: Path,
) -> TerminalPreparation:
    """Create and mode-000 seal V5 without returning either sealed payload."""

    root = root.resolve()
    if root.exists() or root.is_symlink():
        raise FileExistsError("V5 terminal root is create-once")
    if (
        FRESH_HELDOUT_TOKENS != FRESH_HELDOUT_WINDOWS * (CONTEXT + 1)
        or FRESH_HELDOUT_DECISIONS != 247_920
        or OPTIMIZER_STEPS != 2_730
        or TRAIN_WINDOWS != 43_680
        or TRAIN_DECISIONS != 5_241_600
        or INITIALIZATION_SEED != 9_739
    ):
        raise RuntimeError("V5 frozen arithmetic drifted")
    v2 = _verify_v2_authorization(v2_result_path)
    predecessor, predecessor_artifact = _verify_predecessor(predecessor_root.resolve())
    frames = H4SpinFrameArtifactV1.load(frame_sidecar_path.resolve())
    if (
        frames.artifact_cid != H4_FRAME_ARTIFACT_CID
        or frames.file_cid != H4_FRAME_FILE_CID
    ):
        raise ValueError("V5 H4 sidecar differs from the frozen input")
    pooled_result, pooled_artifact = _verify_pooled_comparator(
        pooled_comparator_root.resolve()
    )
    heldout = _read_u16_slice(
        source_train_path,
        offset_tokens=FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
        token_count=FRESH_HELDOUT_TOKENS,
    )
    heldout_cid = cid_bytes(heldout)
    index_record, heldout_story_cids = _verify_fresh_index(source_train_index_path)
    prior_story_cids = load_required_prior_story_cids(
        tuple(prior_population_paths)
    )
    population = select_prompt_conditioning_population_from_source(
        raw_source_path,
        predecessor.tokenizer_path,
        excluded_story_cids=prior_story_cids,
    )
    prompt_story_cids = frozenset(
        record.story_cid
        for pair in population.pairs
        for record in (pair.left, pair.right)
    )
    if prompt_story_cids.intersection(heldout_story_cids):
        raise ValueError("V5 prompt and fresh-language populations overlap")

    implementation = trainer_implementation_contract()
    root.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{root.name}.preparing-", dir=root.parent))
    try:
        population_path = staging / POPULATION_RELATIVE_PATH
        heldout_path = staging / HELDOUT_RELATIVE_PATH
        atomic_write(population_path, canonical_json_bytes(population.manifest()))
        atomic_write(heldout_path, heldout)
        population_record = _sealed_record(
            population_path, relative_path=POPULATION_RELATIVE_PATH
        )
        heldout_record = _sealed_record(heldout_path, relative_path=HELDOUT_RELATIVE_PATH)
        commitment = _with_cid(
            {
                "schema": COMMITMENT_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "population": {
                    **population_record,
                    "schema": POPULATION_SCHEMA,
                    "pairs": PAIR_COUNT,
                    "directions": DIRECTION_COUNT,
                    "scored_target_tokens": SCORED_TARGET_TOKENS,
                },
                "fresh_heldout": {
                    **heldout_record,
                    "source_store_cid": FRESH_HELDOUT_SOURCE_TRAIN_CID,
                    "source_offset_tokens": FRESH_HELDOUT_SOURCE_OFFSET_TOKENS,
                    "tokens": FRESH_HELDOUT_TOKENS,
                    "windows": FRESH_HELDOUT_WINDOWS,
                    "decisions": FRESH_HELDOUT_DECISIONS,
                    "train_index": index_record,
                },
                "story_disjoint": True,
                "sealed_mode": "000",
            },
            "commitment_cid",
        )
        _write_exclusive_json(staging / COMMITMENT_RELATIVE_PATH, commitment)
        sealed = (staging / POPULATION_RELATIVE_PATH).parent
        sealed.chmod(0o000)
        manifest = _with_cid(
            {
                "schema": PREPARATION_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "model_policy": MODEL_POLICY,
                "implementation": implementation,
                "v2_authorization": {
                    "path": str(v2_result_path.resolve()),
                    "result_cid": v2["result_cid"],
                    "verdict": v2["verdict"],
                    "authorized": True,
                },
                "predecessor": {
                    "root": str(predecessor_root.resolve()),
                    "policy": PREDECESSOR_POLICY,
                    "result_cid": PREDECESSOR_RESULT_CID,
                    "artifact": {
                        "path": str(predecessor_artifact),
                        "bytes": PREDECESSOR_ARTIFACT_BYTES,
                        "cid": PREDECESSOR_ARTIFACT_CID,
                    },
                },
                "h4_spin_frames": {
                    "path": str(frame_sidecar_path.resolve()),
                    "artifact_cid": frames.artifact_cid,
                    "file_cid": frames.file_cid,
                },
                "pooled_comparator": {
                    "root": str(pooled_comparator_root.resolve()),
                    "result_cid": pooled_result["result_cid"],
                    "artifact": {
                        "path": str(pooled_artifact),
                        "cid": V4_POOLED_ARTIFACT_CID,
                        "bytes": pooled_artifact.stat().st_size,
                    },
                },
                "training": _training_contract(predecessor),
                "commitment_cid": commitment["commitment_cid"],
                "population_cid": population.population_cid,
                "fresh_heldout_cid": heldout_cid,
                "prior_population_paths": [
                    str(path.resolve()) for path in prior_population_paths
                ],
                "scope": _scope_contract(),
            },
            "preparation_cid",
        )
        _write_exclusive_json(staging / PREPARATION_RELATIVE_PATH, manifest)
        if root.exists() or root.is_symlink():
            raise FileExistsError("V5 terminal root appeared during preparation")
        staging.rename(root)
    except BaseException:
        sealed = staging / POPULATION_RELATIVE_PATH
        if sealed.parent.exists() and not sealed.parent.is_symlink():
            sealed.parent.chmod(0o700)
        if staging.exists() and not staging.is_symlink():
            shutil.rmtree(staging)
        raise
    return load_predictive_block_delta_terminal_preparation(root)


def load_predictive_block_delta_terminal_preparation(root: Path) -> TerminalPreparation:
    """Verify preparation and its seal without opening either V5 payload."""

    if root.is_symlink() or not root.is_dir():
        raise ValueError("V5 root must be a regular directory")
    root = root.resolve()
    manifest = _read_json(root / PREPARATION_RELATIVE_PATH)
    commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
    _verify_self_cid(manifest, "preparation_cid")
    _verify_self_cid(commitment, "commitment_cid")
    _validate_bound_implementation(
        manifest.get("implementation"),
        envelope_cid=manifest.get("preparation_cid"),
        frozen_envelope_cid=FROZEN_V5_PREPARATION_CID,
    )
    population_record = commitment.get("population")
    heldout_record = commitment.get("fresh_heldout")
    if (
        manifest.get("schema") != PREPARATION_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("policy") != POLICY
        or manifest.get("model_policy") != MODEL_POLICY
        or manifest.get("commitment_cid") != commitment.get("commitment_cid")
        or commitment.get("schema") != COMMITMENT_SCHEMA
        or commitment.get("issue") != ISSUE
        or commitment.get("policy") != POLICY
        or commitment.get("sealed_mode") != "000"
        or commitment.get("story_disjoint") is not True
        or not isinstance(population_record, Mapping)
        or not isinstance(heldout_record, Mapping)
        or population_record.get("path") != POPULATION_RELATIVE_PATH
        or population_record.get("schema") != POPULATION_SCHEMA
        or population_record.get("pairs") != PAIR_COUNT
        or population_record.get("directions") != DIRECTION_COUNT
        or population_record.get("scored_target_tokens") != SCORED_TARGET_TOKENS
        or population_record.get("cid") != manifest.get("population_cid")
        or heldout_record.get("path") != HELDOUT_RELATIVE_PATH
        or heldout_record.get("cid") != manifest.get("fresh_heldout_cid")
        or heldout_record.get("source_store_cid")
        != FRESH_HELDOUT_SOURCE_TRAIN_CID
        or heldout_record.get("source_offset_tokens")
        != FRESH_HELDOUT_SOURCE_OFFSET_TOKENS
        or heldout_record.get("tokens") != FRESH_HELDOUT_TOKENS
        or heldout_record.get("windows") != FRESH_HELDOUT_WINDOWS
        or heldout_record.get("decisions") != FRESH_HELDOUT_DECISIONS
        or not isinstance(population_record.get("bytes"), int)
        or int(population_record["bytes"]) < 1
        or not isinstance(heldout_record.get("bytes"), int)
        or heldout_record.get("bytes") != FRESH_HELDOUT_TOKENS * 2
    ):
        raise ValueError("V5 preparation envelope differs")
    sealed = root / POPULATION_RELATIVE_PATH
    sealed_mode = stat.S_IMODE(sealed.parent.stat().st_mode)
    revealed = (root / REVEAL_RELATIVE_PATH).exists()
    transitioning = (root / REVEAL_TRANSITION_RELATIVE_PATH).exists()
    valid_mode = (
        sealed_mode == 0o700
        if revealed
        else sealed_mode in (0o000, 0o700)
        if transitioning
        else sealed_mode == 0o000
    )
    if sealed.parent.is_symlink() or not valid_mode or (revealed and not transitioning):
        raise ValueError("V5 sealed directory mode differs from its lifecycle phase")
    if transitioning:
        transition = _read_json(root / REVEAL_TRANSITION_RELATIVE_PATH)
        _verify_self_cid(transition, "transition_cid")
        if (
            transition.get("schema") != REVEAL_TRANSITION_SCHEMA
            or transition.get("issue") != ISSUE
            or transition.get("policy") != POLICY
            or transition.get("preparation_cid") != manifest.get("preparation_cid")
            or transition.get("commitment_cid") != commitment.get("commitment_cid")
            or not isinstance(transition.get("reveal_cid"), str)
        ):
            raise ValueError("V5 reveal transition envelope differs")
        if revealed:
            revealed_record = _read_json(root / REVEAL_RELATIVE_PATH)
            _verify_self_cid(revealed_record, "reveal_cid")
            if revealed_record.get("reveal_cid") != transition.get("reveal_cid"):
                raise ValueError("V5 reveal differs from its transition envelope")
    authorization = manifest.get("v2_authorization")
    predecessor_record = manifest.get("predecessor")
    frames_record = manifest.get("h4_spin_frames")
    pooled_record = manifest.get("pooled_comparator")
    if not all(
        isinstance(value, Mapping)
        for value in (authorization, predecessor_record, frames_record, pooled_record)
    ):
        raise ValueError("V5 preparation input bindings are malformed")
    v2 = _verify_v2_authorization(Path(str(authorization["path"])))
    predecessor, predecessor_artifact = _verify_predecessor(
        Path(str(predecessor_record["root"]))
    )
    frames = H4SpinFrameArtifactV1.load(Path(str(frames_record["path"])))
    pooled_result, pooled_artifact = _verify_pooled_comparator(
        Path(str(pooled_record["root"]))
    )
    expected_index = heldout_record.get("train_index")
    if (
        authorization
        != {
            "path": str(Path(str(authorization["path"])).resolve()),
            "result_cid": v2["result_cid"],
            "verdict": v2["verdict"],
            "authorized": True,
        }
        or predecessor_record
        != {
            "root": str(Path(str(predecessor_record["root"])).resolve()),
            "policy": PREDECESSOR_POLICY,
            "result_cid": PREDECESSOR_RESULT_CID,
            "artifact": {
                "path": str(predecessor_artifact),
                "bytes": PREDECESSOR_ARTIFACT_BYTES,
                "cid": PREDECESSOR_ARTIFACT_CID,
            },
        }
        or frames_record
        != {
            "path": str(Path(str(frames_record["path"])).resolve()),
            "artifact_cid": frames.artifact_cid,
            "file_cid": frames.file_cid,
        }
        or pooled_record
        != {
            "root": str(Path(str(pooled_record["root"])).resolve()),
            "result_cid": pooled_result["result_cid"],
            "artifact": {
                "path": str(pooled_artifact),
                "cid": V4_POOLED_ARTIFACT_CID,
                "bytes": pooled_artifact.stat().st_size,
            },
        }
        or manifest.get("training") != _training_contract(predecessor)
        or manifest.get("scope") != _scope_contract()
        or not isinstance(expected_index, Mapping)
        or expected_index.get("cid") != FRESH_HELDOUT_TRAIN_INDEX_CID
        or expected_index.get("first_capacity_story_ordinal")
        != FRESH_HELDOUT_FIRST_CAPACITY_STORY
        or expected_index.get("first_source_story_ordinal")
        != FRESH_HELDOUT_FIRST_SOURCE_STORY
        or expected_index.get("last_capacity_story_ordinal")
        != FRESH_HELDOUT_LAST_CAPACITY_STORY
        or expected_index.get("last_source_story_ordinal")
        != FRESH_HELDOUT_LAST_SOURCE_STORY
        or expected_index.get("story_cids") != FRESH_HELDOUT_STORY_CIDS
    ):
        raise ValueError("V5 preparation input identity differs")
    return TerminalPreparation(
        root=root,
        manifest=manifest,
        commitment=commitment,
        predecessor=predecessor,
        predecessor_artifact_path=predecessor_artifact,
        frames=frames,
        pooled_artifact_path=pooled_artifact,
    )


def _binding_arm(arm: Arm) -> Literal["geometric", "plain"]:
    return "plain" if arm == "plain" else "geometric"


def _fit_intervention(arm: Arm) -> Literal["native", "no_delta"]:
    return "no_delta" if arm == "additive" else "native"


def _new_predictive_model(
    preparation: TerminalPreparation,
    arm: Arm,
    device: torch.device,
) -> R4PredictiveBlockDeltaBindingV1:
    torch.manual_seed(INITIALIZATION_SEED)
    model = R4PredictiveBlockDeltaBindingV1(
        _exact_geometry(preparation.predecessor),
        preparation.frames,
        arm=_binding_arm(arm),
    ).to(device)
    model.load_qualified_base_artifact(
        preparation.predecessor_artifact_path.read_bytes()
    )
    return model


def _trainable_parameters(
    model: R4PredictiveBlockDeltaBindingV1,
) -> tuple[torch.nn.Parameter, ...]:
    parameters = tuple(model.trainable_parameters())
    frozen = tuple(model.frozen_base_parameters())
    if (
        sum(parameter.numel() for parameter in parameters)
        != TRAINABLE_PARAMETER_COUNT
        or not all(parameter.requires_grad for parameter in parameters)
        or any(parameter.requires_grad for parameter in frozen)
    ):
        raise RuntimeError("V5 trainable/frozen parameter partition differs")
    return parameters


def _optimizer(
    parameters: Sequence[torch.nn.Parameter],
) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        parameters,
        lr=learning_rate(0),
        betas=(ADAM_BETA1, ADAM_BETA2),
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
    )


def _train_step(
    model: R4PredictiveBlockDeltaBindingV1,
    arm: Arm,
    optimizer: torch.optim.Optimizer,
    parameters: Sequence[torch.nn.Parameter],
    batch: Tensor,
    *,
    step: int,
) -> tuple[float, float]:
    """One standard all-position causal cross-entropy construction update."""

    model.train()
    optimizer.zero_grad(set_to_none=True)
    output = model(
        batch[:, :-1],
        batch[:, 1:],
        intervention=_fit_intervention(arm),
    )
    if output.loss is None or not torch.isfinite(output.loss).item():
        raise RuntimeError(f"{arm} V5 construction loss is nonfinite")
    output.loss.backward()
    if any(parameter.grad is None for parameter in parameters):
        raise RuntimeError(f"{arm} V5 trainable value omitted its gradient tensor")
    gradient_norm = torch.nn.utils.clip_grad_norm_(parameters, GRADIENT_CLIP)
    if not torch.isfinite(gradient_norm).item():
        raise RuntimeError(f"{arm} V5 gradient norm is nonfinite")
    rate = learning_rate(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    optimizer.step()
    return float(output.loss.detach().cpu()), float(gradient_norm.detach().cpu())


def _probe_vector(
    model: R4PredictiveBlockDeltaBindingV1, logits: Tensor
) -> list[float]:
    values = logits.detach().float().reshape(-1)[:64].cpu().tolist()
    values.extend(
        model.export_binding_artifact()[:64]
    )
    return [float(value) for value in values]


def _probe_arm(root: Path, arm: Arm, plan: ExecutionPlan) -> dict[str, Any]:
    """Exercise one arm using construction data only; sealed read count is zero."""

    device, backend = _configure_device(plan)
    if device.type != "cpu":
        raise RuntimeError("V5 execution probing is CPU-only")
    preparation = load_predictive_block_delta_terminal_preparation(root)
    model = _new_predictive_model(preparation, arm, device)
    parameters = _trainable_parameters(model)
    base_before = model.export_qualified_base_artifact()
    binding_before = model.export_binding_artifact()
    initial_binding_cid = cid_bytes(binding_before)
    expected_initial = _new_predictive_model(preparation, "geometric", device)
    if expected_initial.export_binding_artifact() != binding_before:
        raise RuntimeError("V5 arms do not start from byte-identical binding values")
    del expected_initial

    batch = _ordered_train_batch(preparation.predecessor, 1, device)
    model.eval()
    with torch.no_grad():
        original_targets = batch[:1, 1:]
        mutated_targets = (original_targets + 1) % VOCAB_SIZE
        initial = model(
            batch[:1, :-1], intervention=_fit_intervention(arm)
        )
        supervised = model(
            batch[:1, :-1],
            original_targets,
            intervention=_fit_intervention(arm),
        )
        target_mutated = model(
            batch[:1, :-1],
            mutated_targets,
            intervention=_fit_intervention(arm),
        )
        state_off = model(batch[:1, :-1], intervention="state_off")
        prefix = model(
            batch[:1, :63], intervention=_fit_intervention(arm)
        )
        auxiliary = model(batch[:1, :-1], intervention="no_delta")
        deranged = (
            model(batch[:1, :-1], intervention="transport_permuted")
            if arm != "plain"
            else None
        )
    state_off_delta = float(
        (state_off.logits - state_off.base_logits).abs().max().cpu()
    )
    causal_delta = float(
        (initial.logits[:, :63] - prefix.logits).abs().max().cpu()
    )
    unobserved_target_delta = float(
        (supervised.logits - target_mutated.logits).abs().max().cpu()
    )
    work = tuple(int(value) for value in initial.audit.work_signature())
    equal_intervention_work = work == tuple(
        int(value) for value in state_off.audit.work_signature()
    ) == tuple(int(value) for value in auxiliary.audit.work_signature())
    deranged_effect = (
        float((initial.head_logits - deranged.head_logits).abs().max().cpu())
        if deranged is not None
        else None
    )
    transport = (
        transport_mechanics(model, device=device) if arm != "plain" else None
    )

    optimizer = _optimizer(parameters)
    measured: list[float] = []
    final_loss = math.nan
    final_gradient_norm = math.nan
    for offset in range(PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS):
        _sync(device)
        started = time.perf_counter()
        train_batch = _ordered_train_batch(
            preparation.predecessor, offset + 1, device
        )
        final_loss, final_gradient_norm = _train_step(
            model,
            arm,
            optimizer,
            parameters,
            train_batch,
            step=offset + 1,
        )
        _sync(device)
        if offset >= PROBE_WARMUP_STEPS:
            measured.append(time.perf_counter() - started)

    evaluation_batch = _ordered_train_batch(
        preparation.predecessor,
        PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS + 1,
        device,
    )
    model.eval()
    _sync(device)
    evaluation_started = time.perf_counter()
    with torch.no_grad():
        evaluation = model(
            evaluation_batch[:, :-1],
            evaluation_batch[:, 1:],
            intervention=_fit_intervention(arm),
        )
    _sync(device)
    evaluation_seconds = time.perf_counter() - evaluation_started
    artifact_started = time.perf_counter()
    artifact = model.export_binding_artifact()
    artifact_seconds = time.perf_counter() - artifact_started
    replay = _new_predictive_model(preparation, arm, device)
    replay.load_binding_artifact(artifact)
    replay.eval()
    with torch.no_grad():
        replay_output = replay(
            evaluation_batch[:, :-1], intervention=_fit_intervention(arm)
        )
    replay_delta = float(
        (evaluation.logits - replay_output.logits).abs().max().cpu()
    )
    mechanics = {
        "strict_causal_prefix_maximum_logits_delta": causal_delta,
        "unobserved_target_mutation_maximum_logits_delta": unobserved_target_delta,
        "state_off_v1_maximum_logits_delta": state_off_delta,
        "artifact_replay_maximum_logits_delta": replay_delta,
        "equal_runtime_intervention_work": equal_intervention_work,
        "transport_permutation_head_effect": deranged_effect,
        "transport": transport,
        "qualified_base_unchanged": (
            model.export_qualified_base_artifact() == base_before
        ),
        "binding_values_changed": artifact != binding_before,
        "forbidden_reads": int(evaluation.audit.forbidden_reads),
    }
    transport_passed = bool(
        transport is None
        or (
            transport["all_frame_identity_maximum_delta"] <= 2e-5
            and transport["all_frame_step_connection_maximum_delta"] <= 2e-5
            and transport["transported_matrix_read_covariance_maximum_delta"]
            <= 2e-5
            and deranged_effect is not None
            and deranged_effect > 0.0
        )
    )
    mechanics["passed"] = bool(
        causal_delta <= 2e-5
        and unobserved_target_delta == 0.0
        and state_off_delta == 0.0
        and replay_delta == 0.0
        and equal_intervention_work
        and transport_passed
        and mechanics["qualified_base_unchanged"]
        and mechanics["binding_values_changed"]
        and mechanics["forbidden_reads"] == 0
    )
    return {
        "arm": arm,
        "backend": backend,
        "mean_train_step_seconds": statistics.fmean(measured),
        "measured_train_step_seconds": measured,
        "artifact_export_seconds": artifact_seconds,
        "evaluation_batch_seconds": evaluation_seconds,
        "peak_memory_bytes": _peak_rss_bytes(),
        "memory_budget_bytes": int(backend["memory_budget_bytes"]),
        "final_probe_train_loss": final_loss,
        "final_probe_gradient_norm": final_gradient_norm,
        "initial_binding_cid": initial_binding_cid,
        "probe_vector": _probe_vector(model, evaluation.logits),
        "mechanics": mechanics,
        "sealed_prompt_reads": 0,
        "sealed_heldout_reads": 0,
    }


def _probe_worker(root: str, arm: str, plan_value: Mapping[str, Any], queue: Any) -> None:
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
                "result": _probe_arm(Path(root), arm, plan),  # type: ignore[arg-type]
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


def _collect_worker(process: Any, queue: Any, *, timeout: float = 900.0) -> dict[str, Any]:
    process.join(timeout)
    if process.is_alive():
        process.terminate()
        process.join(10.0)
        return {
            "ok": False,
            "error": {"type": "TimeoutError", "reason": "V5 worker timed out"},
        }
    try:
        value = queue.get(timeout=5.0)
    except Empty:
        return {
            "ok": False,
            "error": {
                "type": "WorkerExitError",
                "reason": f"V5 worker exited {process.exitcode} without evidence",
            },
        }
    if not isinstance(value, dict):
        raise RuntimeError("V5 worker returned a non-object")
    return value


def _spawned_arm_batch(
    root: Path,
    plan: ExecutionPlan,
    arms: Sequence[Arm],
    *,
    timeout: float,
) -> dict[str, Any]:
    context = mp.get_context("spawn")
    outcomes: dict[str, Any] = {}
    if plan.concurrent_arms:
        for start in range(0, len(arms), plan.workers):
            active: dict[str, tuple[Any, Any]] = {}
            for arm in arms[start : start + plan.workers]:
                queue = context.Queue()
                process = context.Process(
                    target=_probe_worker,
                    args=(str(root), arm, asdict(plan), queue),
                    name=f"predictive-v5-probe-{plan.name}-{arm}",
                )
                process.start()
                active[arm] = (process, queue)
            for arm, (process, queue) in active.items():
                outcomes[arm] = _collect_worker(process, queue, timeout=timeout)
    else:
        for arm in arms:
            queue = context.Queue()
            process = context.Process(
                target=_probe_worker,
                args=(str(root), arm, asdict(plan), queue),
                name=f"predictive-v5-probe-{plan.name}-{arm}",
            )
            process.start()
            outcomes[arm] = _collect_worker(process, queue, timeout=timeout)
    return {"plan": plan.identity(), "arms": outcomes}


def _spawned_probe_executor(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    return _spawned_arm_batch(root, plan, ARMS, timeout=900.0)


ProbeExecutor = Callable[[Path, ExecutionPlan], Mapping[str, Any]]


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Require deterministic CPU4 parity, mechanics, wall, and memory."""

    if len(records) != len(ELIGIBLE_PLANS) or [
        dict(record.get("plan", {})) for record in records
    ] != [plan.identity() for plan in ELIGIBLE_PLANS]:
        raise ValueError("V5 probe plans differ from the frozen CPU plan set")
    reference_name = ELIGIBLE_PLANS[0].name
    by_name = {str(record.get("plan", {}).get("name")): record for record in records}
    reference = by_name.get(reference_name)
    if not isinstance(reference, Mapping) or not isinstance(reference.get("arms"), Mapping):
        raise ValueError("V5 CPU4 reference probe is absent")
    reference_arms = reference["arms"]
    scoring_batches = (
        PROMPT_SCORING_PASSES * math.ceil(DIRECTION_COUNT / DIRECTION_BATCH_SIZE)
        + FRESH_SCORING_PASSES * math.ceil(FRESH_HELDOUT_WINDOWS / BATCH_SIZE)
    )
    scoring_batch_seconds: float | None = None
    if all(reference_arms.get(arm, {}).get("ok") is True for arm in ARMS):
        candidates = [
            float(reference_arms[arm]["result"].get("evaluation_batch_seconds", math.nan))
            for arm in ARMS
        ]
        if all(math.isfinite(value) and value > 0.0 for value in candidates):
            scoring_batch_seconds = max(candidates)
    projected_scoring_seconds = (
        scoring_batch_seconds * scoring_batches * PROJECTION_SAFETY_FACTOR
        if scoring_batch_seconds is not None
        else None
    )
    projected: list[dict[str, Any]] = []
    for raw in records:
        record = dict(raw)
        plan = record.get("plan")
        arms = record.get("arms")
        if not isinstance(plan, Mapping) or not isinstance(arms, Mapping):
            raise ValueError("V5 probe record is malformed")
        available = all(
            arms.get(arm, {}).get("ok") is True
            and reference_arms.get(arm, {}).get("ok") is True
            for arm in ARMS
        )
        equivalent = available
        mechanics = available
        deltas: dict[str, float | None] = {}
        initial_cids: set[str] = set()
        per_arm_seconds: dict[str, float] = {}
        peak_values: list[int] = []
        memory_budget = 0
        if available:
            for arm in ARMS:
                observed = arms[arm]["result"]
                expected = reference_arms[arm]["result"]
                vector = observed.get("probe_vector", [])
                reference_vector = expected.get("probe_vector", [])
                if not vector or len(vector) != len(reference_vector):
                    deltas[arm] = None
                    equivalent = False
                else:
                    delta = max(
                        abs(float(left) - float(right))
                        for left, right in zip(vector, reference_vector, strict=True)
                    )
                    deltas[arm] = delta
                    equivalent = equivalent and delta <= EQUIVALENCE_ABS_TOLERANCE
                mechanics = mechanics and observed.get("mechanics", {}).get("passed") is True
                mechanics = mechanics and observed.get("sealed_prompt_reads") == 0
                mechanics = mechanics and observed.get("sealed_heldout_reads") == 0
                initial_cids.add(str(observed.get("initial_binding_cid")))
                per_arm_seconds[arm] = (
                    float(observed["mean_train_step_seconds"]) * OPTIMIZER_STEPS
                    + float(observed["artifact_export_seconds"])
                )
                peak_values.append(int(observed["peak_memory_bytes"]))
            memory_budget = min(
                int(arms[arm]["result"]["memory_budget_bytes"]) for arm in ARMS
            )
        if bool(plan.get("concurrent_arms")) and available:
            # The first two arms share two workers; additive is queued as the
            # deterministic third task after that pair completes.
            raw_seconds = max(per_arm_seconds["geometric"], per_arm_seconds["plain"])
            raw_seconds += per_arm_seconds["additive"]
            peak_bytes = sum(sorted(peak_values, reverse=True)[:2])
        else:
            raw_seconds = sum(per_arm_seconds.values()) if available else None
            peak_bytes = max(peak_values, default=0)
        projected_seconds = (
            raw_seconds * PROJECTION_SAFETY_FACTOR
            if raw_seconds is not None
            else None
        )
        memory_fraction = peak_bytes / memory_budget if memory_budget else None
        eligible = bool(
            available
            and equivalent
            and mechanics
            and len(initial_cids) == 1
            and projected_seconds is not None
            and projected_seconds <= HARD_WALL_SECONDS
            and memory_fraction is not None
            and memory_fraction <= MEMORY_FRACTION_CEILING
            and projected_scoring_seconds is not None
            and projected_scoring_seconds <= SCORING_HARD_WALL_SECONDS
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
            "raw_training_seconds": raw_seconds,
            "safety_factor": PROJECTION_SAFETY_FACTOR,
            "projected_training_seconds": projected_seconds,
            "hard_wall_seconds_before_scoring": HARD_WALL_SECONDS,
            "peak_memory_bytes": peak_bytes,
            "memory_budget_bytes": memory_budget,
            "memory_fraction": memory_fraction,
            "memory_fraction_ceiling": MEMORY_FRACTION_CEILING,
            "byte_identical_initialization": len(initial_cids) == 1,
            "mechanics_passed": mechanics,
            "scoring_projection": {
                "cpu4_evaluation_batch_seconds": scoring_batch_seconds,
                "scoring_batches": scoring_batches,
                "safety_factor": PROJECTION_SAFETY_FACTOR,
                "projected_seconds": projected_scoring_seconds,
                "hard_wall_seconds": SCORING_HARD_WALL_SECONDS,
                "eligible": (
                    projected_scoring_seconds is not None
                    and projected_scoring_seconds <= SCORING_HARD_WALL_SECONDS
                ),
            },
        }
        projected.append(record)
    eligible_records = [record for record in projected if record["projection"]["eligible"]]
    selected = min(
        eligible_records,
        key=lambda record: (
            float(record["projection"]["projected_training_seconds"]),
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


def _probe_predictive_block_delta_terminal(
    root: Path, *, executor: ProbeExecutor
) -> dict[str, Any]:
    root = root.resolve()
    preparation = load_predictive_block_delta_terminal_preparation(root)
    path = root / PROBE_RELATIVE_PATH
    if path.exists():
        result = _read_json(path)
        _verify_self_cid(result, "probe_cid")
        _validate_bound_implementation(
            result.get("implementation"),
            envelope_cid=result.get("probe_cid"),
            frozen_envelope_cid=FROZEN_V5_PROBE_CID,
        )
        selection = select_execution_plan(result.get("selection", {}).get("plans", []))
        expected_verdict = (
            "PREDICTIVE_V5_EXECUTION_ADMITTED"
            if selection["available"]
            else "UNAVAILABLE_PREDICTIVE_V5_COMPUTE"
        )
        if (
            result.get("schema") != PROBE_SCHEMA
            or result.get("issue") != ISSUE
            or result.get("policy") != POLICY
            or result.get("preparation_cid")
            != preparation.manifest["preparation_cid"]
            or result.get("commitment_cid")
            != preparation.commitment["commitment_cid"]
            or result.get("implementation") != preparation.manifest["implementation"]
            or selection != result.get("selection")
            or result.get("eligible") is not selection["available"]
            or result.get("verdict") != expected_verdict
            or result.get("contract")
            != {
                "plans": [plan.identity() for plan in ELIGIBLE_PLANS],
                "warmup_steps_per_arm": PROBE_WARMUP_STEPS,
                "measured_steps_per_arm": PROBE_MEASURED_STEPS,
                "construction_only": True,
                "prompt_reads": 0,
                "fresh_heldout_reads": 0,
                "cuda": "FORBIDDEN",
            }
        ):
            raise ValueError("cached V5 execution probe differs")
        return result
    implementation = trainer_implementation_contract()
    records = [dict(executor(root, plan)) for plan in ELIGIBLE_PLANS]
    if implementation != trainer_implementation_contract():
        raise ValueError("trainer implementation changed during V5 execution probe")
    selection = select_execution_plan(records)
    result = _with_cid(
        {
            "schema": PROBE_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation.manifest["preparation_cid"],
            "commitment_cid": preparation.commitment["commitment_cid"],
            "implementation": implementation,
            "contract": {
                "plans": [plan.identity() for plan in ELIGIBLE_PLANS],
                "warmup_steps_per_arm": PROBE_WARMUP_STEPS,
                "measured_steps_per_arm": PROBE_MEASURED_STEPS,
                "construction_only": True,
                "prompt_reads": 0,
                "fresh_heldout_reads": 0,
                "cuda": "FORBIDDEN",
            },
            "selection": selection,
            "eligible": selection["available"],
            "verdict": (
                "PREDICTIVE_V5_EXECUTION_ADMITTED"
                if selection["available"]
                else "UNAVAILABLE_PREDICTIVE_V5_COMPUTE"
            ),
        },
        "probe_cid",
    )
    _write_exclusive_json(path, result)
    return result


def probe_predictive_block_delta_terminal(root: Path) -> dict[str, Any]:
    return _probe_predictive_block_delta_terminal(root, executor=_spawned_probe_executor)


def _arm_directory(root: Path, arm: Arm) -> Path:
    return root / "arms" / arm


def _artifact_path(root: Path, arm: Arm) -> Path:
    return _arm_directory(root, arm) / "binding.safetensors"


def _arm_result_path(root: Path, arm: Arm) -> Path:
    return _arm_directory(root, arm) / "result.json"


def _checkpoint_path(root: Path, arm: Arm) -> Path:
    return _arm_directory(root, arm) / "checkpoint.pt"


def _progress_path(root: Path, arm: Arm) -> Path:
    return _arm_directory(root, arm) / "progress.json"


def _atomic_torch_save(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    os.close(descriptor)
    temporary = Path(temporary_name)
    try:
        torch.save(dict(value), temporary)
        with temporary.open("rb+") as target:
            os.fsync(target.fileno())
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def _load_arm_result(root: Path, arm: Arm) -> dict[str, Any]:
    result = _read_json(_arm_result_path(root, arm))
    _verify_self_cid(result, "arm_result_cid")
    artifact = result.get("artifact")
    backend = result.get("backend")
    path = _artifact_path(root, arm)
    preparation = load_predictive_block_delta_terminal_preparation(root)
    probe = _read_json(root / PROBE_RELATIVE_PATH)
    _verify_self_cid(probe, "probe_cid")
    plan = _selected_plan(probe)
    elapsed = result.get("elapsed_seconds")
    final_loss = result.get("final_loss")
    final_gradient_norm = result.get("final_gradient_norm")
    initial_binding_cid = result.get("initial_binding_cid")
    if (
        result.get("schema") != ARM_RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
        or result.get("arm") != arm
        or result.get("model_policy") != MODEL_POLICY
        or result.get("binding_arm") != _binding_arm(arm)
        or result.get("fit_intervention") != _fit_intervention(arm)
        or result.get("preparation_cid")
        != preparation.manifest["preparation_cid"]
        or result.get("probe_cid") != probe.get("probe_cid")
        or result.get("plan_cid") != plan.identity()["plan_cid"]
        or result.get("completed_steps") != OPTIMIZER_STEPS
        or result.get("presentations") != TRAIN_DECISIONS
        or result.get("objective")
        != "mean causal cross-entropy over all 120 next-token positions"
        or result.get("qualified_base_unchanged") is not True
        or result.get("artifact_replay_maximum_logits_delta") != 0.0
        or result.get("post_reveal_optimizer_steps") != 0
        or isinstance(elapsed, bool)
        or not isinstance(elapsed, (int, float))
        or not math.isfinite(float(elapsed))
        or float(elapsed) < 0.0
        or not isinstance(final_loss, (int, float))
        or not math.isfinite(float(final_loss))
        or not isinstance(final_gradient_norm, (int, float))
        or not math.isfinite(float(final_gradient_norm))
        or not isinstance(initial_binding_cid, str)
        or not initial_binding_cid.startswith("blake3:")
        or not isinstance(backend, Mapping)
        or backend.get("platform") != "Darwin"
        or backend.get("blas") != "Apple Accelerate"
        or backend.get("threads") != plan.threads_per_worker
        or not isinstance(backend.get("memory_budget_bytes"), int)
        or int(backend["memory_budget_bytes"]) < 1
        or not isinstance(artifact, Mapping)
        or path.is_symlink()
        or not path.is_file()
        or artifact.get("path") != str(path.relative_to(root))
        or artifact.get("bytes") != path.stat().st_size
        or artifact.get("cid") != cid_file(path)
    ):
        raise ValueError(f"cached V5 {arm} arm result differs")
    return result


def _fit_arm(
    root: Path,
    arm: Arm,
    plan: ExecutionPlan,
    *,
    resume: bool,
    wall_seconds: float,
) -> dict[str, Any]:
    """Fit one exact construction trajectory with a single recoverable checkpoint."""

    result_path = _arm_result_path(root, arm)
    if result_path.exists():
        return _load_arm_result(root, arm)
    if (root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("V5 fitting is forbidden after reveal")
    preparation = load_predictive_block_delta_terminal_preparation(root)
    probe = _read_json(root / PROBE_RELATIVE_PATH)
    _verify_self_cid(probe, "probe_cid")
    device, backend = _configure_device(plan)
    model = _new_predictive_model(preparation, arm, device)
    parameters = _trainable_parameters(model)
    optimizer = _optimizer(parameters)
    base_before = model.export_qualified_base_artifact()
    initial_binding = model.export_binding_artifact()
    start_step = 0
    elapsed_before = 0.0
    checkpoint = _checkpoint_path(root, arm)
    if checkpoint.exists():
        if not resume:
            raise RuntimeError(f"{arm} V5 checkpoint exists; explicit --resume is required")
        if checkpoint.is_symlink() or not checkpoint.is_file():
            raise ValueError(f"{arm} V5 checkpoint must be a regular file")
        saved = torch.load(checkpoint, map_location=device, weights_only=False)
        if (
            not isinstance(saved, Mapping)
            or saved.get("schema") != "uor-r4.predictive-block-delta-v5-checkpoint/1"
            or saved.get("arm") != arm
            or saved.get("preparation_cid") != preparation.manifest["preparation_cid"]
            or saved.get("probe_cid") != probe["probe_cid"]
            or saved.get("plan_cid") != plan.identity()["plan_cid"]
        ):
            raise ValueError(f"{arm} V5 checkpoint binding differs")
        start_step = int(saved["completed_steps"])
        elapsed_before = float(saved["elapsed_seconds"])
        if not 0 < start_step < OPTIMIZER_STEPS or elapsed_before < 0.0:
            raise ValueError(f"{arm} V5 checkpoint progress differs")
        model.load_state_dict(saved["model"])
        optimizer.load_state_dict(saved["optimizer"])
    elif resume:
        raise FileNotFoundError(f"{arm} V5 resume requested without a checkpoint")

    started = time.monotonic()
    final_loss = math.nan
    final_gradient_norm = math.nan
    for step in range(start_step + 1, OPTIMIZER_STEPS + 1):
        elapsed = elapsed_before + (time.monotonic() - started)
        if elapsed >= wall_seconds:
            if step > 1:
                _atomic_torch_save(
                    checkpoint,
                    {
                        "schema": "uor-r4.predictive-block-delta-v5-checkpoint/1",
                        "arm": arm,
                        "preparation_cid": preparation.manifest["preparation_cid"],
                        "probe_cid": probe["probe_cid"],
                        "plan_cid": plan.identity()["plan_cid"],
                        "completed_steps": step - 1,
                        "elapsed_seconds": elapsed,
                        "model": model.state_dict(),
                        "optimizer": optimizer.state_dict(),
                    },
                )
            atomic_write_json(
                _progress_path(root, arm),
                {
                    "arm": arm,
                    "completed_steps": step - 1,
                    "total_steps": OPTIMIZER_STEPS,
                    "final_loss": final_loss if step > 1 else None,
                    "elapsed_seconds": elapsed,
                },
            )
            raise TimeoutError(f"{arm} V5 fit exhausted its aggregate wall share")
        batch = _ordered_train_batch(preparation.predecessor, step, device)
        final_loss, final_gradient_norm = _train_step(
            model, arm, optimizer, parameters, batch, step=step
        )
        if step % CHECKPOINT_INTERVAL == 0 and step < OPTIMIZER_STEPS:
            elapsed = elapsed_before + (time.monotonic() - started)
            _atomic_torch_save(
                checkpoint,
                {
                    "schema": "uor-r4.predictive-block-delta-v5-checkpoint/1",
                    "arm": arm,
                    "preparation_cid": preparation.manifest["preparation_cid"],
                    "probe_cid": probe["probe_cid"],
                    "plan_cid": plan.identity()["plan_cid"],
                    "completed_steps": step,
                    "elapsed_seconds": elapsed,
                    "model": model.state_dict(),
                    "optimizer": optimizer.state_dict(),
                },
            )
        if step % PROGRESS_INTERVAL == 0:
            atomic_write_json(
                _progress_path(root, arm),
                {
                    "arm": arm,
                    "completed_steps": step,
                    "total_steps": OPTIMIZER_STEPS,
                    "final_loss": final_loss,
                    "elapsed_seconds": elapsed_before + (time.monotonic() - started),
                },
            )

    elapsed = elapsed_before + (time.monotonic() - started)
    if elapsed > wall_seconds:
        raise TimeoutError(f"{arm} V5 fit exceeded its aggregate wall share")
    artifact = model.export_binding_artifact()
    artifact_path = _artifact_path(root, arm)
    atomic_write(artifact_path, artifact)
    replay = _new_predictive_model(preparation, arm, device)
    replay.load_binding_artifact(artifact)
    check_batch = _ordered_train_batch(preparation.predecessor, OPTIMIZER_STEPS, device)
    model.eval()
    replay.eval()
    with torch.no_grad():
        fitted_output = model(
            check_batch[:1, :-1], intervention=_fit_intervention(arm)
        )
        replay_output = replay(
            check_batch[:1, :-1], intervention=_fit_intervention(arm)
        )
    replay_delta = float(
        (fitted_output.logits - replay_output.logits).abs().max().cpu()
    )
    result = _with_cid(
        {
            "schema": ARM_RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "arm": arm,
            "binding_arm": _binding_arm(arm),
            "fit_intervention": _fit_intervention(arm),
            "preparation_cid": preparation.manifest["preparation_cid"],
            "probe_cid": probe["probe_cid"],
            "plan_cid": plan.identity()["plan_cid"],
            "completed_steps": OPTIMIZER_STEPS,
            "presentations": TRAIN_DECISIONS,
            "objective": "mean causal cross-entropy over all 120 next-token positions",
            "final_loss": final_loss,
            "final_gradient_norm": final_gradient_norm,
            "elapsed_seconds": elapsed,
            "initial_binding_cid": cid_bytes(initial_binding),
            "qualified_base_unchanged": (
                model.export_qualified_base_artifact() == base_before
            ),
            "artifact_replay_maximum_logits_delta": replay_delta,
            "artifact": {
                "path": str(artifact_path.relative_to(root)),
                "bytes": len(artifact),
                "cid": cid_bytes(artifact),
            },
            "post_reveal_optimizer_steps": 0,
            "backend": backend,
        },
        "arm_result_cid",
    )
    _write_exclusive_json(result_path, result)
    if checkpoint.exists():
        checkpoint.unlink()
    return _load_arm_result(root, arm)


def _fit_worker(
    root: str,
    arm: str,
    plan_value: Mapping[str, Any],
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
        result = _fit_arm(
            Path(root),
            arm,  # type: ignore[arg-type]
            plan,
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


def _arm_persisted_elapsed(root: Path, arm: Arm) -> float:
    values = [0.0]
    result_path = _arm_result_path(root, arm)
    if result_path.exists():
        result = _load_arm_result(root, arm)
        values.append(float(result["elapsed_seconds"]))
    checkpoint = _checkpoint_path(root, arm)
    if checkpoint.exists():
        if checkpoint.is_symlink() or not checkpoint.is_file():
            raise ValueError(f"{arm} V5 checkpoint must be a regular file")
        saved = torch.load(checkpoint, map_location="cpu", weights_only=False)
        if (
            not isinstance(saved, Mapping)
            or saved.get("schema")
            != "uor-r4.predictive-block-delta-v5-checkpoint/1"
            or saved.get("arm") != arm
            or isinstance(saved.get("completed_steps"), bool)
            or not isinstance(saved.get("completed_steps"), int)
            or not 0 < int(saved["completed_steps"]) < OPTIMIZER_STEPS
        ):
            raise ValueError(f"{arm} V5 checkpoint progress differs")
        values.append(float(saved["elapsed_seconds"]))
    progress_path = _progress_path(root, arm)
    if progress_path.exists():
        progress = _read_json(progress_path)
        elapsed = progress.get("elapsed_seconds")
        completed = progress.get("completed_steps")
        if (
            progress.get("arm") != arm
            or progress.get("total_steps") != OPTIMIZER_STEPS
            or isinstance(completed, bool)
            or not isinstance(completed, int)
            or not 0 <= completed <= OPTIMIZER_STEPS
            or isinstance(elapsed, bool)
            or not isinstance(elapsed, (int, float))
            or not math.isfinite(float(elapsed))
            or float(elapsed) < 0.0
        ):
            raise ValueError(f"{arm} V5 progress ledger differs")
        values.append(float(elapsed))
    if not all(math.isfinite(value) and value >= 0.0 for value in values):
        raise ValueError(f"{arm} V5 elapsed ledger differs")
    return max(values)


def _derived_fit_wall(root: Path, plan: ExecutionPlan) -> float:
    elapsed = {arm: _arm_persisted_elapsed(root, arm) for arm in ARMS}
    if plan.concurrent_arms:
        return max(elapsed["geometric"], elapsed["plain"]) + elapsed["additive"]
    return math.fsum(elapsed.values())


def _fit_budget_path(root: Path) -> Path:
    return root / FIT_BUDGET_RELATIVE_PATH


def _load_fit_budget(root: Path, plan: ExecutionPlan) -> float:
    derived = _derived_fit_wall(root, plan)
    path = _fit_budget_path(root)
    if not path.exists():
        return derived
    value = _read_json(path)
    _verify_self_cid(value, "fit_budget_cid")
    consumed = value.get("consumed_seconds")
    if (
        value.get("schema") != FIT_BUDGET_SCHEMA
        or value.get("issue") != ISSUE
        or value.get("policy") != POLICY
        or value.get("plan_cid") != plan.identity()["plan_cid"]
        or isinstance(consumed, bool)
        or not isinstance(consumed, (int, float))
        or not math.isfinite(float(consumed))
        or float(consumed) < 0.0
    ):
        raise ValueError("V5 aggregate fit-budget ledger differs")
    return max(float(consumed), derived)


def _write_fit_budget(root: Path, plan: ExecutionPlan, consumed: float) -> None:
    if not math.isfinite(consumed) or consumed < 0.0:
        raise ValueError("V5 aggregate fit-budget value is invalid")
    atomic_write_json(
        _fit_budget_path(root),
        _with_cid(
            {
                "schema": FIT_BUDGET_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "plan_cid": plan.identity()["plan_cid"],
                "consumed_seconds": consumed,
            },
            "fit_budget_cid",
        ),
    )


def _run_fit_batch(
    root: Path,
    plan: ExecutionPlan,
    arms: Sequence[Arm],
    *,
    resume: bool,
    wall_seconds: float,
) -> dict[str, Any]:
    historical_elapsed = _load_fit_budget(root, plan)
    pending = [arm for arm in arms if not _arm_result_path(root, arm).exists()]
    if not pending:
        return {
            "ok": True,
            "arms": {
                arm: {"ok": True, "result": _load_arm_result(root, arm), "cached": True}
                for arm in arms
            },
            "aggregate_wall_seconds": historical_elapsed,
            "cached": True,
        }
    context = mp.get_context("spawn")
    outcomes: dict[str, Any] = {}
    _write_fit_budget(root, plan, historical_elapsed)
    if not resume and any(_checkpoint_path(root, arm).exists() for arm in pending):
        raise RuntimeError("V5 checkpoints require explicit --resume")
    if plan.concurrent_arms:
        phases: tuple[tuple[Arm, ...], ...] = (
            ("geometric", "plain"),
            ("additive",),
        )
        batches = [
            [arm for arm in phase if arm in pending]
            for phase in phases
            if any(arm in pending for arm in phase)
        ]
    else:
        batches = [[arm] for arm in pending]
    run_started = time.monotonic()
    for arm_batch in batches:
        current_elapsed = time.monotonic() - run_started
        remaining = wall_seconds - historical_elapsed - current_elapsed
        if remaining <= 0.0:
            _write_fit_budget(root, plan, historical_elapsed + current_elapsed)
            return {
                "ok": False,
                "error": {"type": "TimeoutError", "reason": "V5 aggregate fit wall exhausted"},
                "arms": outcomes,
            }
        active: dict[str, tuple[Any, Any]] = {}
        for arm in arm_batch:
            queue = context.Queue()
            arm_resume = _checkpoint_path(root, arm).exists()
            checkpoint_elapsed = 0.0
            if arm_resume:
                saved = torch.load(
                    _checkpoint_path(root, arm), map_location="cpu", weights_only=False
                )
                checkpoint_elapsed = float(saved["elapsed_seconds"])
            process = context.Process(
                target=_fit_worker,
                args=(
                    str(root),
                    arm,
                    asdict(plan),
                    arm_resume,
                    checkpoint_elapsed + remaining,
                    queue,
                ),
                name=f"predictive-v5-fit-{plan.name}-{arm}",
            )
            process.start()
            active[arm] = (process, queue)
        for arm, (process, queue) in active.items():
            outcomes[arm] = _collect_worker(process, queue, timeout=remaining + 60.0)
        _write_fit_budget(
            root,
            plan,
            historical_elapsed + (time.monotonic() - run_started),
        )
        if not all(outcomes[arm].get("ok") is True for arm in arm_batch):
            return {"ok": False, "arms": outcomes}
    total_elapsed = historical_elapsed + (time.monotonic() - run_started)
    _write_fit_budget(root, plan, total_elapsed)
    return {
        "ok": True,
        "arms": outcomes,
        "aggregate_wall_seconds": total_elapsed,
    }


def _selected_plan(probe: Mapping[str, Any]) -> ExecutionPlan:
    value = probe.get("selection", {}).get("selected_plan")
    if not isinstance(value, Mapping):
        raise ValueError("V5 execution probe has no selected plan")
    plan = ExecutionPlan(
        name=str(value["name"]),
        backend=str(value["backend"]),  # type: ignore[arg-type]
        threads_per_worker=int(value["threads_per_worker"]),
        workers=int(value["workers"]),
        concurrent_arms=bool(value["concurrent_arms"]),
    )
    if plan.identity() != dict(value):
        raise ValueError("V5 selected plan identity differs")
    return plan


def _load_revealed_population(root: Path) -> PromptConditioningPopulationV5:
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
    _verify_self_cid(commitment, "commitment_cid")
    population_record = commitment.get("population")
    heldout_record = commitment.get("fresh_heldout")
    if (
        reveal.get("schema") != REVEAL_SCHEMA
        or reveal.get("commitment_cid") != commitment.get("commitment_cid")
        or not isinstance(population_record, Mapping)
        or not isinstance(heldout_record, Mapping)
    ):
        raise ValueError("V5 reveal binding differs")
    population_path = root / POPULATION_RELATIVE_PATH
    heldout_path = root / HELDOUT_RELATIVE_PATH
    if (
        population_path.is_symlink()
        or not population_path.is_file()
        or population_path.stat().st_size != population_record.get("bytes")
        or cid_file(population_path) != population_record.get("cid")
        or heldout_path.is_symlink()
        or not heldout_path.is_file()
        or heldout_path.stat().st_size != heldout_record.get("bytes")
        or cid_file(heldout_path) != heldout_record.get("cid")
    ):
        raise ValueError("revealed V5 payload differs from its commitment")
    value = _read_json(population_path)
    population = PromptConditioningPopulationV5.from_manifest(value)
    if population.population_cid != population_record.get("cid"):
        raise ValueError("V5 population object CID differs")
    return population


def _expected_reveal(
    preparation: TerminalPreparation,
    arm_results: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    if any(arm not in arm_results for arm in ARMS):
        raise ValueError("all three V5 artifacts must be fixed before reveal")
    for arm in ARMS:
        _load_arm_result(preparation.root, arm)  # exact artifact check
    return _with_cid(
        {
            "schema": REVEAL_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation.manifest["preparation_cid"],
            "commitment_cid": preparation.commitment["commitment_cid"],
            "population_cid": preparation.manifest["population_cid"],
            "artifacts": {
                arm: dict(arm_results[arm]["artifact"]) for arm in ARMS
            },
            "v1_artifact_cid": PREDECESSOR_ARTIFACT_CID,
            "pooled_artifact_cid": V4_POOLED_ARTIFACT_CID,
            "post_reveal_optimizer_steps": 0,
            "revealed_once": True,
        },
        "reveal_cid",
    )


def _reveal_terminal_population(
    preparation: TerminalPreparation,
    arm_results: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    path = preparation.root / REVEAL_RELATIVE_PATH
    expected = _expected_reveal(preparation, arm_results)
    transition_path = preparation.root / REVEAL_TRANSITION_RELATIVE_PATH
    expected_transition = _with_cid(
        {
            "schema": REVEAL_TRANSITION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation.manifest["preparation_cid"],
            "commitment_cid": preparation.commitment["commitment_cid"],
            "reveal_cid": expected["reveal_cid"],
            "from_mode": "000",
            "to_mode": "700",
        },
        "transition_cid",
    )
    if transition_path.exists():
        if _read_json(transition_path) != expected_transition:
            raise ValueError("V5 reveal transition does not reproduce")
    else:
        _write_exclusive_json(transition_path, expected_transition)
    sealed = (preparation.root / POPULATION_RELATIVE_PATH).parent
    mode = stat.S_IMODE(sealed.stat().st_mode)
    if mode == 0o000:
        sealed.chmod(0o700)
    elif mode != 0o700:
        raise ValueError("V5 reveal transition found an invalid sealed mode")
    if path.exists():
        reveal = _read_json(path)
        if reveal != expected:
            raise ValueError("cached V5 reveal differs from fitted artifacts")
    else:
        _write_exclusive_json(path, expected)
        reveal = expected
    _load_revealed_population(preparation.root)
    return reveal


@dataclass(frozen=True, slots=True)
class TerminalPromptScore:
    mode: str
    directions: int
    targets: int
    mean_gain_nats_per_token: float
    wins: int
    own_nll_nats_per_token: float
    foreign_nll_nats_per_token: float
    forbidden_reads: int
    work_signature: tuple[int, ...]
    trace_cid: str
    suffix_logits_trace_cid: str
    head_logits_trace_cid: str | None
    direction_gains_nats_per_token: tuple[float, ...]

    def record(self) -> dict[str, Any]:
        value = asdict(self)
        value["work_signature"] = list(self.work_signature)
        value["direction_gains_nats_per_token"] = list(
            self.direction_gains_nats_per_token
        )
        return value


def _prompt_score_from_record(value: object, *, name: str) -> TerminalPromptScore:
    if not isinstance(value, Mapping) or set(value) != {
        "mode",
        "directions",
        "targets",
        "mean_gain_nats_per_token",
        "wins",
        "own_nll_nats_per_token",
        "foreign_nll_nats_per_token",
        "forbidden_reads",
        "work_signature",
        "trace_cid",
        "suffix_logits_trace_cid",
        "head_logits_trace_cid",
        "direction_gains_nats_per_token",
    }:
        raise ValueError(f"V5 {name} prompt score fields differ")
    numeric = (
        value["mean_gain_nats_per_token"],
        value["own_nll_nats_per_token"],
        value["foreign_nll_nats_per_token"],
    )
    gains = value["direction_gains_nats_per_token"]
    signature = value["work_signature"]
    if (
        value["mode"] != name
        or value["directions"] != DIRECTION_COUNT
        or value["targets"] != SCORED_TARGET_TOKENS
        or isinstance(value["wins"], bool)
        or not isinstance(value["wins"], int)
        or not 0 <= int(value["wins"]) <= DIRECTION_COUNT
        or isinstance(value["forbidden_reads"], bool)
        or not isinstance(value["forbidden_reads"], int)
        or int(value["forbidden_reads"]) < 0
        or not all(
            isinstance(item, (int, float))
            and not isinstance(item, bool)
            and math.isfinite(float(item))
            for item in numeric
        )
        or not isinstance(gains, list)
        or len(gains) != DIRECTION_COUNT
        or not all(
            isinstance(item, (int, float))
            and not isinstance(item, bool)
            and math.isfinite(float(item))
            for item in gains
        )
        or not isinstance(signature, list)
        or not signature
        or not all(isinstance(item, int) and not isinstance(item, bool) for item in signature)
        or not isinstance(value["trace_cid"], str)
        or not str(value["trace_cid"]).startswith("blake3:")
        or not isinstance(value["suffix_logits_trace_cid"], str)
        or not str(value["suffix_logits_trace_cid"]).startswith("blake3:")
        or (
            value["head_logits_trace_cid"] is not None
            and (
                not isinstance(value["head_logits_trace_cid"], str)
                or not str(value["head_logits_trace_cid"]).startswith("blake3:")
            )
        )
    ):
        raise ValueError(f"V5 {name} prompt score is malformed")
    normalized_gains = tuple(float(item) for item in gains)
    if (
        float(value["mean_gain_nats_per_token"])
        != math.fsum(normalized_gains) / DIRECTION_COUNT
        or int(value["wins"]) != sum(item > 0.0 for item in normalized_gains)
    ):
        raise ValueError(f"V5 {name} prompt score does not reproduce")
    return TerminalPromptScore(
        mode=name,
        directions=DIRECTION_COUNT,
        targets=SCORED_TARGET_TOKENS,
        mean_gain_nats_per_token=float(value["mean_gain_nats_per_token"]),
        wins=int(value["wins"]),
        own_nll_nats_per_token=float(value["own_nll_nats_per_token"]),
        foreign_nll_nats_per_token=float(value["foreign_nll_nats_per_token"]),
        forbidden_reads=int(value["forbidden_reads"]),
        work_signature=tuple(int(item) for item in signature),
        trace_cid=str(value["trace_cid"]),
        suffix_logits_trace_cid=str(value["suffix_logits_trace_cid"]),
        head_logits_trace_cid=(
            str(value["head_logits_trace_cid"])
            if value["head_logits_trace_cid"] is not None
            else None
        ),
        direction_gains_nats_per_token=normalized_gains,
    )


def _validate_fresh_record(value: object, *, name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping) or set(value) != {
        "mode",
        "rows",
        "ce_nats",
        "top1_correct",
        "top1_rate",
        "forbidden_reads",
        "work_signature",
        "logits_trace_cid",
        "prediction_trace_cid",
    }:
        raise ValueError(f"V5 {name} fresh-language fields differ")
    if (
        value["mode"] != name
        or value["rows"] != FRESH_HELDOUT_DECISIONS
        or isinstance(value["top1_correct"], bool)
        or not isinstance(value["top1_correct"], int)
        or not 0 <= int(value["top1_correct"]) <= FRESH_HELDOUT_DECISIONS
        or isinstance(value["forbidden_reads"], bool)
        or not isinstance(value["forbidden_reads"], int)
        or int(value["forbidden_reads"]) < 0
        or not all(
            isinstance(value[field], (int, float))
            and not isinstance(value[field], bool)
            and math.isfinite(float(value[field]))
            for field in ("ce_nats", "top1_rate")
        )
        or float(value["top1_rate"])
        != int(value["top1_correct"]) / FRESH_HELDOUT_DECISIONS
        or not isinstance(value["work_signature"], list)
        or not value["work_signature"]
        or not all(
            isinstance(item, int) and not isinstance(item, bool)
            for item in value["work_signature"]
        )
        or not all(
            isinstance(value[field], str) and str(value[field]).startswith("blake3:")
            for field in ("logits_trace_cid", "prediction_trace_cid")
        )
    ):
        raise ValueError(f"V5 {name} fresh-language score is malformed")
    return value


@dataclass(frozen=True, slots=True)
class _PooledView:
    model: R4LearnedCandidateLeafAssociativeReadoutV1

    def eval(self) -> _PooledView:
        self.model.eval()
        return self

    def __call__(self, token_ids: Tensor) -> Any:
        return self.model.forward_arm("pooled", token_ids)


ORDER_SHUFFLE_POSITIONS = tuple(
    sorted(
        range(PROMPT_TOKENS),
        key=lambda position: (
            blake3(struct.pack(">QI", INITIALIZATION_SEED, position)).digest(),
            position,
        ),
    )
)
ORDER_SHUFFLE_CID = cid_bytes(canonical_json_bytes(list(ORDER_SHUFFLE_POSITIONS)))


def _invoke(
    model: Any,
    token_ids: Tensor,
    *,
    mode: str,
) -> Any:
    if mode == "v1" or mode == "pooled":
        return model(token_ids)
    intervention = {
        "geometric": "native",
        "plain": "native",
        "additive": "no_delta",
        "transport_permuted": "transport_permuted",
        "same_fitted_weights_additive": "no_delta",
        "state_off": "state_off",
        "order_shuffled": "native",
        "geometric_replay": "native",
    }.get(mode)
    if intervention is None:
        raise ValueError(f"unknown V5 scoring mode: {mode}")
    return model(token_ids, intervention=intervention)


def _prompt_sequence(prompt: Sequence[int], continuation: Sequence[int]) -> list[int]:
    return [BOS_TOKEN_ID, *prompt, *continuation[:-1]]


def _order_shuffle(inputs: Tensor) -> Tensor:
    if sorted(ORDER_SHUFFLE_POSITIONS) != list(range(PROMPT_TOKENS)):
        raise RuntimeError("V5 order-shuffle permutation is not bijective")
    result = inputs.clone()
    positions = torch.tensor(
        [position + 1 for position in ORDER_SHUFFLE_POSITIONS],
        dtype=torch.long,
        device=inputs.device,
    )
    result[:, 1 : PROMPT_TOKENS + 1] = inputs.index_select(1, positions)
    if not torch.equal(
        torch.sort(result[:, 1 : PROMPT_TOKENS + 1], dim=1).values,
        torch.sort(inputs[:, 1 : PROMPT_TOKENS + 1], dim=1).values,
    ):
        raise RuntimeError("V5 order shuffle changed a prompt token multiset")
    return result


@torch.no_grad()
def _score_prompt(
    model: Any,
    population: PromptConditioningPopulationV5,
    *,
    mode: str,
    device: torch.device,
) -> TerminalPromptScore:
    if hasattr(model, "eval"):
        model.eval()
    directions = prompt_directions(population)
    gains: list[float] = []
    own_values: list[float] = []
    foreign_values: list[float] = []
    forbidden_reads = 0
    signature: tuple[int, ...] | None = None
    trace = blake3()
    suffix_logits_trace = blake3()
    head_logits_trace = blake3()
    has_head_logits: bool | None = None
    for start in range(0, len(directions), DIRECTION_BATCH_SIZE):
        selected = directions[start : start + DIRECTION_BATCH_SIZE]
        rows: list[list[int]] = []
        targets: list[tuple[int, ...]] = []
        for direction in selected:
            rows.append(_prompt_sequence(direction.own_prompt, direction.continuation))
            rows.append(
                _prompt_sequence(direction.crossed_prompt, direction.continuation)
            )
            targets.append(direction.continuation)
        inputs = torch.tensor(rows, dtype=torch.long, device=device)
        if mode == "order_shuffled":
            inputs = _order_shuffle(inputs)
        output = _invoke(model, inputs, mode=mode)
        logits = getattr(output, "logits", None)
        audit = getattr(output, "audit", None)
        if (
            not isinstance(logits, Tensor)
            or tuple(logits.shape[:2])
            != (len(rows), 1 + PROMPT_TOKENS + CONTINUATION_TOKENS - 1)
            or not torch.isfinite(logits).all().item()
            or audit is None
            or not callable(getattr(audit, "work_signature", None))
        ):
            raise ValueError(f"{mode} V5 prompt output contract differs")
        work = tuple(int(value) for value in audit.work_signature())
        if signature is None:
            signature = work
        elif signature != work:
            raise ValueError(f"{mode} V5 prompt work changed between batches")
        forbidden_reads += int(audit.forbidden_reads)
        suffix = logits[:, PROMPT_TOKENS : PROMPT_TOKENS + CONTINUATION_TOKENS]
        suffix_f32 = suffix.detach().float().contiguous().cpu()
        suffix_logits_trace.update(suffix_f32.numpy().tobytes())
        head_logits = getattr(output, "head_logits", None)
        current_has_head = isinstance(head_logits, Tensor)
        if has_head_logits is None:
            has_head_logits = current_has_head
        elif has_head_logits is not current_has_head:
            raise ValueError(f"{mode} V5 head-logit availability changed")
        if current_has_head:
            head_suffix = head_logits[
                :, PROMPT_TOKENS : PROMPT_TOKENS + CONTINUATION_TOKENS
            ]
            head_logits_trace.update(
                head_suffix.detach().float().contiguous().cpu().numpy().tobytes()
            )
        log_probabilities = F.log_softmax(suffix.float(), dim=-1)
        target = torch.tensor(targets, dtype=torch.long, device=device)
        expanded = target[:, None, :].expand(-1, 2, -1).reshape(
            -1, CONTINUATION_TOKENS
        )
        selected_log_probs = log_probabilities.gather(
            2, expanded[:, :, None]
        )[:, :, 0]
        own = selected_log_probs[0::2].double().cpu()
        foreign = selected_log_probs[1::2].double().cpu()
        for offset, direction in enumerate(selected):
            own_row = [float(value) for value in own[offset].tolist()]
            foreign_row = [float(value) for value in foreign[offset].tolist()]
            own_values.extend(own_row)
            foreign_values.extend(foreign_row)
            gain = math.fsum(
                left - right
                for left, right in zip(own_row, foreign_row, strict=True)
            ) / CONTINUATION_TOKENS
            gains.append(gain)
            trace.update(
                struct.pack(
                    ">IB",
                    int(direction.pair_index),
                    0 if direction.side == "left" else 1,
                )
            )
            for left, right in zip(own_row, foreign_row, strict=True):
                trace.update(struct.pack("<dd", left, right))
    if (
        len(gains) != DIRECTION_COUNT
        or len(own_values) != SCORED_TARGET_TOKENS
        or signature is None
    ):
        raise RuntimeError("V5 prompt scorer did not cover the population")
    return TerminalPromptScore(
        mode=mode,
        directions=DIRECTION_COUNT,
        targets=SCORED_TARGET_TOKENS,
        mean_gain_nats_per_token=math.fsum(gains) / DIRECTION_COUNT,
        wins=sum(value > 0.0 for value in gains),
        own_nll_nats_per_token=-math.fsum(own_values) / SCORED_TARGET_TOKENS,
        foreign_nll_nats_per_token=(
            -math.fsum(foreign_values) / SCORED_TARGET_TOKENS
        ),
        forbidden_reads=forbidden_reads,
        work_signature=signature,
        trace_cid=f"blake3:{trace.hexdigest()}",
        suffix_logits_trace_cid=f"blake3:{suffix_logits_trace.hexdigest()}",
        head_logits_trace_cid=(
            f"blake3:{head_logits_trace.hexdigest()}" if has_head_logits else None
        ),
        direction_gains_nats_per_token=tuple(gains),
    )


@torch.no_grad()
def _evaluate_language(
    model: Any,
    windows: LanguagePathWindowStore,
    *,
    mode: str,
    device: torch.device,
) -> dict[str, Any]:
    if hasattr(model, "eval"):
        model.eval()
    loss_sum = 0.0
    top1 = 0
    rows = 0
    forbidden_reads = 0
    reference_signature: tuple[int, ...] | None = None
    aggregate_signature: tuple[int, ...] | None = None
    structural_positions: frozenset[int] | None = None
    trace = blake3()
    logits_trace = blake3()
    for start in range(0, FRESH_HELDOUT_WINDOWS, BATCH_SIZE):
        count = min(BATCH_SIZE, FRESH_HELDOUT_WINDOWS - start)
        batch = _window_batch(windows, start, count, device)
        output = _invoke(model, batch[:, :-1], mode=mode)
        logits = getattr(output, "logits", None)
        audit = getattr(output, "audit", None)
        if (
            not isinstance(logits, Tensor)
            or tuple(logits.shape) != (count, CONTEXT, VOCAB_SIZE)
            or not torch.isfinite(logits).all().item()
            or audit is None
            or not callable(getattr(audit, "work_signature", None))
        ):
            raise ValueError(f"{mode} V5 language output contract differs")
        work = tuple(int(value) for value in audit.work_signature())
        current_structural_positions = {2, 3, 4}
        if isinstance(audit, LearnedAssociativeReadoutAudit):
            current_structural_positions.add(13)
        current_structural = frozenset(current_structural_positions)
        if len(work) <= max(current_structural) or work[0] != count:
            raise ValueError(f"{mode} V5 language output contract differs")
        if reference_signature is None:
            reference_signature = work
            aggregate_signature = work
            structural_positions = current_structural
        else:
            if (
                aggregate_signature is None
                or structural_positions != current_structural
                or len(reference_signature) != len(work)
            ):
                raise ValueError(f"{mode} V5 language work changed between batches")
            reference_batch = reference_signature[0]
            observed_batch = work[0]
            for index, (reference, observed) in enumerate(
                zip(reference_signature, work, strict=True)
            ):
                if index in structural_positions:
                    matches = reference == observed
                else:
                    matches = (
                        reference * observed_batch == observed * reference_batch
                    )
                if not matches:
                    raise ValueError(
                        f"{mode} V5 language work changed between batches"
                    )
            aggregate_signature = tuple(
                previous if index in structural_positions else previous + observed
                for index, (previous, observed) in enumerate(
                    zip(aggregate_signature, work, strict=True)
                )
            )
        targets = batch[:, 1:]
        batch_loss = float(
            F.cross_entropy(
                logits.float().reshape(-1, logits.shape[-1]),
                targets.reshape(-1),
                reduction="sum",
            ).cpu()
        )
        if not math.isfinite(batch_loss):
            raise ValueError(f"{mode} V5 language loss is nonfinite")
        loss_sum += batch_loss
        predictions = logits.argmax(dim=-1)
        top1 += int((predictions == targets).sum().cpu())
        rows += int(targets.numel())
        forbidden_reads += int(audit.forbidden_reads)
        logits_trace.update(
            logits.detach().float().contiguous().cpu().numpy().tobytes()
        )
        trace.update(predictions.detach().cpu().to(torch.int16).numpy().tobytes())
    if rows != FRESH_HELDOUT_DECISIONS or aggregate_signature is None:
        raise RuntimeError("V5 fresh-language scorer coverage differs")
    return {
        "mode": mode,
        "rows": rows,
        "ce_nats": loss_sum / rows,
        "top1_correct": top1,
        "top1_rate": top1 / rows,
        "forbidden_reads": forbidden_reads,
        "work_signature": list(aggregate_signature),
        "logits_trace_cid": f"blake3:{logits_trace.hexdigest()}",
        "prediction_trace_cid": f"blake3:{trace.hexdigest()}",
    }


def _paired_improvements(
    candidate: TerminalPromptScore, comparator: TerminalPromptScore
) -> int:
    return sum(
        left > right
        for left, right in zip(
            candidate.direction_gains_nats_per_token,
            comparator.direction_gains_nats_per_token,
            strict=True,
        )
    )


def terminal_decision(
    *,
    scores: Mapping[str, TerminalPromptScore],
    fresh: Mapping[str, Mapping[str, Any]],
    mechanics: Mapping[str, Any],
) -> dict[str, Any]:
    geometric = scores["geometric"]
    v1 = scores["v1"]
    pooled = scores["pooled"]
    plain = scores["plain"]
    transport = scores["transport_permuted"]
    additive = scores["additive"]
    state_off = scores["state_off"]
    capacity_gates = {
        "absolute_gain": geometric.mean_gain_nats_per_token >= PROMPT_GAIN_THRESHOLD,
        "gain_over_v1": (
            geometric.mean_gain_nats_per_token - v1.mean_gain_nats_per_token
            >= INCREMENTAL_GAIN_THRESHOLD
        ),
        "gain_over_pooled": (
            geometric.mean_gain_nats_per_token - pooled.mean_gain_nats_per_token
            >= INCREMENTAL_GAIN_THRESHOLD
        ),
        "directional_wins": geometric.wins >= WIN_THRESHOLD,
        "own_nll_no_worse_than_v1": (
            geometric.own_nll_nats_per_token <= v1.own_nll_nats_per_token
        ),
        "own_nll_no_worse_than_pooled": (
            geometric.own_nll_nats_per_token <= pooled.own_nll_nats_per_token
        ),
        "state_load_bearing": (
            geometric.mean_gain_nats_per_token - state_off.mean_gain_nats_per_token
            >= INCREMENTAL_GAIN_THRESHOLD
        ),
        "state_own_nll_nonregression": (
            geometric.own_nll_nats_per_token
            <= state_off.own_nll_nats_per_token
        ),
    }
    capacity_metric_positive = all(capacity_gates.values())

    geometry_comparators = {"plain": plain, "transport_permuted": transport}
    geometry_gates: dict[str, bool] = {}
    geometry_comparisons: dict[str, Any] = {}
    for name, comparator in geometry_comparators.items():
        gain = geometric.mean_gain_nats_per_token - comparator.mean_gain_nats_per_token
        paired = _paired_improvements(geometric, comparator)
        geometry_comparisons[name] = {
            "gain_nats_per_token": gain,
            "paired_directional_improvements": paired,
        }
        geometry_gates[f"gain_over_{name}"] = gain >= INCREMENTAL_GAIN_THRESHOLD
        geometry_gates[f"paired_over_{name}"] = paired >= WIN_THRESHOLD
        geometry_gates[f"own_nll_no_worse_than_{name}"] = (
            geometric.own_nll_nats_per_token
            <= comparator.own_nll_nats_per_token
        )
    geometry_metric_positive = all(geometry_gates.values())

    additive_language_valid = (
        additive.own_nll_nats_per_token
        <= state_off.own_nll_nats_per_token + FRESH_NLL_TOLERANCE
    )
    delta_gain = geometric.mean_gain_nats_per_token - additive.mean_gain_nats_per_token
    delta_paired = _paired_improvements(geometric, additive)
    delta_gates = {
        "additive_language_valid": additive_language_valid,
        "gain_over_independently_fitted_additive": (
            delta_gain >= INCREMENTAL_GAIN_THRESHOLD
        ),
        "paired_over_independently_fitted_additive": delta_paired >= WIN_THRESHOLD,
        "own_nll_no_worse_than_independently_fitted_additive": (
            geometric.own_nll_nats_per_token
            <= additive.own_nll_nats_per_token
        ),
    }
    delta_metric_positive = all(delta_gates.values())

    immutable_comparator = min(
        ("v1", "pooled"),
        key=lambda name: (
            float(fresh[name]["ce_nats"]),
            -float(fresh[name]["top1_rate"]),
            name,
        ),
    )
    fresh_gates = {
        "nll_within_tolerance": (
            float(fresh["geometric"]["ce_nats"])
            <= float(fresh[immutable_comparator]["ce_nats"])
            + FRESH_NLL_TOLERANCE
        ),
        "top1_within_tolerance": (
            float(fresh["geometric"]["top1_rate"])
            >= float(fresh[immutable_comparator]["top1_rate"])
            - FRESH_TOP1_POINT_TOLERANCE / 100.0
        ),
    }
    fresh_metric_positive = all(fresh_gates.values())
    metric_values = [
        score.mean_gain_nats_per_token
        for score in (geometric, v1, pooled, plain, transport, additive, state_off)
    ] + [
        score.own_nll_nats_per_token
        for score in (geometric, v1, pooled, plain, transport, additive, state_off)
    ] + [
        float(fresh[name][field])
        for name in ("geometric", "v1", "pooled")
        for field in ("ce_nats", "top1_rate")
    ]
    metrics_finite = all(math.isfinite(value) for value in metric_values)
    integrity = mechanics.get("passed") is True and metrics_finite
    capacity_positive = integrity and capacity_metric_positive
    geometry_positive = integrity and geometry_metric_positive
    delta_positive = integrity and delta_metric_positive
    fresh_positive = integrity and fresh_metric_positive
    delta_verdict = (
        "INVALID_DELTA_ATTRIBUTION"
        if not integrity
        else "PREDICTIVE_DELTA_PROMPT_SPECIFIC_SUPERIORITY"
        if delta_positive
        else "ADDITIVE_CONTROL_NO_STABLE_CAPACITY"
        if not additive_language_valid
        else "DELTA_PROMPT_SPECIFIC_SUPERIORITY_NOT_ESTABLISHED"
    )
    if not integrity:
        verdict = "INVALID_PREDICTIVE_V5_TERMINAL"
    elif not capacity_positive:
        verdict = "PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY"
    elif not fresh_positive:
        verdict = "PREDICTIVE_PROMPT_CAPACITY_FRESH_LANGUAGE_REGRESSION"
    elif geometry_positive:
        verdict = "PREDICTIVE_GEOMETRIC_CAPACITY_AND_ATTRIBUTION_PASS"
    else:
        verdict = "PREDICTIVE_CAPACITY_WITHOUT_GEOMETRY_ATTRIBUTION"
    return {
        "verdict": verdict,
        "capacity_positive": capacity_positive,
        "geometry_positive": geometry_positive,
        "delta_overwrite_positive": delta_positive,
        "fresh_language_positive": fresh_positive,
        "integrity_positive": integrity,
        "metrics_finite": metrics_finite,
        "capacity": {
            "raw_metric_positive": capacity_metric_positive,
            "gates": capacity_gates,
            "thresholds": {
                "absolute_gain_nats_per_token": PROMPT_GAIN_THRESHOLD,
                "incremental_gain_nats_per_token": INCREMENTAL_GAIN_THRESHOLD,
                "wins": WIN_THRESHOLD,
            },
        },
        "geometry_attribution": {
            "raw_metric_positive": geometry_metric_positive,
            "gates": geometry_gates,
            "comparisons": geometry_comparisons,
        },
        "delta_attribution": {
            "verdict": delta_verdict,
            "claimed": delta_positive,
            "gates": delta_gates,
            "raw_metric_positive": delta_metric_positive,
            "gain_nats_per_token": delta_gain,
            "paired_directional_improvements": delta_paired,
            "language_validity_tolerance_nats": FRESH_NLL_TOLERANCE,
        },
        "fresh_language": {
            "raw_metric_positive": fresh_metric_positive,
            "better_immutable_comparator": immutable_comparator,
            "gates": fresh_gates,
            "nll_tolerance": FRESH_NLL_TOLERANCE,
            "top1_point_tolerance": FRESH_TOP1_POINT_TOLERANCE,
        },
    }


def _load_scoring_models(
    preparation: TerminalPreparation,
    arm_results: Mapping[str, Mapping[str, Any]],
    device: torch.device,
) -> dict[str, Any]:
    models: dict[str, Any] = {}
    for arm in ARMS:
        model = _new_predictive_model(preparation, arm, device)  # type: ignore[arg-type]
        artifact_path = preparation.root / str(arm_results[arm]["artifact"]["path"])
        model.load_binding_artifact(artifact_path.read_bytes())
        model.eval()
        models[arm] = model
    v1 = R4RetainedLanguagePathV1(_exact_geometry(preparation.predecessor)).to(device)
    v1.load_learned_artifact(preparation.predecessor_artifact_path.read_bytes())
    v1.eval()
    pooled_model = R4LearnedCandidateLeafAssociativeReadoutV1(
        _exact_geometry(preparation.predecessor)
    ).to(device)
    pooled_model.load_qualified_base_artifact(
        preparation.predecessor_artifact_path.read_bytes()
    )
    pooled_model.load_head_artifact(
        "pooled", preparation.pooled_artifact_path.read_bytes()
    )
    pooled_model.eval()
    models["v1"] = v1
    models["pooled"] = _PooledView(pooled_model)
    replay = _new_predictive_model(preparation, "geometric", device)
    replay.load_binding_artifact(
        (preparation.root / str(arm_results["geometric"]["artifact"]["path"])).read_bytes()
    )
    replay.eval()
    models["geometric_replay"] = replay
    return models


def _score_campaign(
    preparation: TerminalPreparation,
    arm_results: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    if not (preparation.root / REVEAL_RELATIVE_PATH).exists():
        raise RuntimeError("V5 canonical scoring is forbidden before reveal")
    device, backend = _configure_device(ELIGIBLE_PLANS[0])
    population = _load_revealed_population(preparation.root)
    heldout = LanguagePathWindowStore(
        preparation.root / HELDOUT_RELATIVE_PATH,
        window_count=FRESH_HELDOUT_WINDOWS,
    )
    models = _load_scoring_models(preparation, arm_results, device)
    score_bindings = {
        "geometric": (models["geometric"], "geometric"),
        "plain": (models["plain"], "plain"),
        "additive": (models["additive"], "additive"),
        "transport_permuted": (models["geometric"], "transport_permuted"),
        "same_fitted_weights_additive": (
            models["geometric"],
            "same_fitted_weights_additive",
        ),
        "state_off": (models["geometric"], "state_off"),
        "order_shuffled": (models["geometric"], "order_shuffled"),
        "v1": (models["v1"], "v1"),
        "pooled": (models["pooled"], "pooled"),
    }
    scores = {
        name: _score_prompt(model, population, mode=mode, device=device)
        for name, (model, mode) in score_bindings.items()
    }
    replay = _score_prompt(
        models["geometric_replay"],
        population,
        mode="geometric_replay",
        device=device,
    )
    order_replay = _score_prompt(
        models["geometric"],
        population,
        mode="order_shuffled",
        device=device,
    )
    fresh_bindings = {
        "geometric": (models["geometric"], "geometric"),
        "state_off": (models["geometric"], "state_off"),
        "v1": (models["v1"], "v1"),
        "pooled": (models["pooled"], "pooled"),
    }
    fresh = {
        name: _evaluate_language(model, heldout, mode=mode, device=device)
        for name, (model, mode) in fresh_bindings.items()
    }
    predictive_signatures = {
        scores[name].work_signature
        for name in (
            "geometric",
            "plain",
            "additive",
            "transport_permuted",
            "same_fitted_weights_additive",
            "state_off",
            "order_shuffled",
        )
    }
    state_off_matches_v1 = bool(
        scores["state_off"].trace_cid == scores["v1"].trace_cid
        and scores["state_off"].suffix_logits_trace_cid
        == scores["v1"].suffix_logits_trace_cid
        and scores["state_off"].direction_gains_nats_per_token
        == scores["v1"].direction_gains_nats_per_token
        and scores["state_off"].own_nll_nats_per_token
        == scores["v1"].own_nll_nats_per_token
        and scores["state_off"].foreign_nll_nats_per_token
        == scores["v1"].foreign_nll_nats_per_token
    )
    fresh_state_off_matches_v1 = bool(
        fresh["state_off"]["logits_trace_cid"] == fresh["v1"]["logits_trace_cid"]
        and fresh["state_off"]["prediction_trace_cid"]
        == fresh["v1"]["prediction_trace_cid"]
        and fresh["state_off"]["ce_nats"] == fresh["v1"]["ce_nats"]
        and fresh["state_off"]["top1_correct"] == fresh["v1"]["top1_correct"]
        and fresh["state_off"]["top1_rate"] == fresh["v1"]["top1_rate"]
    )
    mechanics = {
        "forbidden_reads": sum(score.forbidden_reads for score in scores.values())
        + replay.forbidden_reads
        + order_replay.forbidden_reads
        + sum(int(value["forbidden_reads"]) for value in fresh.values()),
        "equal_predictive_prompt_work": len(predictive_signatures) == 1,
        "geometric_artifact_replay_exact": (
            replay.trace_cid == scores["geometric"].trace_cid
            and replay.suffix_logits_trace_cid
            == scores["geometric"].suffix_logits_trace_cid
            and replay.head_logits_trace_cid
            == scores["geometric"].head_logits_trace_cid
            and replay.direction_gains_nats_per_token
            == scores["geometric"].direction_gains_nats_per_token
        ),
        "state_off_exact_v1_reproduction": state_off_matches_v1,
        "fresh_state_off_exact_v1_reproduction": fresh_state_off_matches_v1,
        "equal_geometric_state_off_fresh_work": (
            fresh["geometric"]["work_signature"]
            == fresh["state_off"]["work_signature"]
        ),
        "order_shuffle": {
            "permutation_cid": ORDER_SHUFFLE_CID,
            "identity": ORDER_SHUFFLE_POSITIONS == tuple(range(PROMPT_TOKENS)),
            "bijection": sorted(ORDER_SHUFFLE_POSITIONS)
            == list(range(PROMPT_TOKENS)),
            "token_multiset_preserved": True,
            "deterministic_replay_exact": (
                order_replay.trace_cid == scores["order_shuffled"].trace_cid
                and order_replay.suffix_logits_trace_cid
                == scores["order_shuffled"].suffix_logits_trace_cid
                and order_replay.head_logits_trace_cid
                == scores["order_shuffled"].head_logits_trace_cid
                and order_replay.direction_gains_nats_per_token
                == scores["order_shuffled"].direction_gains_nats_per_token
                and order_replay.work_signature
                == scores["order_shuffled"].work_signature
            ),
            "head_trace_effect": (
                scores["geometric"].head_logits_trace_cid is not None
                and scores["order_shuffled"].head_logits_trace_cid is not None
                and scores["geometric"].head_logits_trace_cid
                != scores["order_shuffled"].head_logits_trace_cid
            ),
            "gain_effect_nats_per_token": (
                scores["geometric"].mean_gain_nats_per_token
                - scores["order_shuffled"].mean_gain_nats_per_token
            ),
            "audit_only": True,
        },
        "post_reveal_optimizer_steps": 0,
        "all_arm_replay_exact": all(
            float(arm_results[arm]["artifact_replay_maximum_logits_delta"]) == 0.0
            for arm in ARMS
        ),
        "all_bases_unchanged": all(
            arm_results[arm].get("qualified_base_unchanged") is True for arm in ARMS
        ),
        "initial_binding_values_byte_identical": len(
            {str(arm_results[arm]["initial_binding_cid"]) for arm in ARMS}
        )
        == 1,
        "equal_completed_work": all(
            arm_results[arm].get("completed_steps") == OPTIMIZER_STEPS for arm in ARMS
        ),
    }
    mechanics["passed"] = bool(
        mechanics["forbidden_reads"] == 0
        and mechanics["equal_predictive_prompt_work"]
        and mechanics["geometric_artifact_replay_exact"]
        and mechanics["state_off_exact_v1_reproduction"]
        and mechanics["fresh_state_off_exact_v1_reproduction"]
        and mechanics["equal_geometric_state_off_fresh_work"]
        and mechanics["order_shuffle"]["identity"] is False
        and mechanics["order_shuffle"]["bijection"]
        and mechanics["order_shuffle"]["token_multiset_preserved"]
        and mechanics["order_shuffle"]["deterministic_replay_exact"]
        and mechanics["order_shuffle"]["head_trace_effect"]
        and mechanics["post_reveal_optimizer_steps"] == 0
        and mechanics["all_arm_replay_exact"]
        and mechanics["all_bases_unchanged"]
        and mechanics["initial_binding_values_byte_identical"]
        and mechanics["equal_completed_work"]
    )
    decision = terminal_decision(scores=scores, fresh=fresh, mechanics=mechanics)
    fixed_weight = scores["same_fitted_weights_additive"]
    return {
        "backend": backend,
        "population_cid": population.population_cid,
        "prompt_scores": {name: score.record() for name, score in scores.items()},
        "fresh_language": fresh,
        "mechanics": mechanics,
        "decision": decision,
        "auxiliary_fixed_weight_additive": {
            "role": "AUXILIARY_ONLY_NOT_DELTA_ATTRIBUTION",
            "gain_delta_from_geometric": (
                scores["geometric"].mean_gain_nats_per_token
                - fixed_weight.mean_gain_nats_per_token
            ),
            "own_nll_delta_from_geometric": (
                fixed_weight.own_nll_nats_per_token
                - scores["geometric"].own_nll_nats_per_token
            ),
        },
    }


def _scoring_worker(root: str, queue: Any) -> None:
    started = time.monotonic()
    try:
        campaign_root = Path(root)
        preparation = load_predictive_block_delta_terminal_preparation(campaign_root)
        arm_results = {arm: _load_arm_result(campaign_root, arm) for arm in ARMS}
        recovery = (
            prepare_predictive_block_delta_scoring_recovery(campaign_root)
            if (campaign_root / SCORING_RECOVERY_RELATIVE_PATH).exists()
            else None
        )
        evidence = _score_campaign(preparation, arm_results)
        record = _with_cid(
            {
                "schema": SCORING_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "preparation_cid": preparation.manifest["preparation_cid"],
                "reveal_cid": _read_json(campaign_root / REVEAL_RELATIVE_PATH)[
                    "reveal_cid"
                ],
                "arm_result_cids": {
                    arm: arm_results[arm]["arm_result_cid"] for arm in ARMS
                },
                "recovery_cid": (
                    recovery["recovery_cid"] if recovery is not None else None
                ),
                "fit_implementation": preparation.manifest["implementation"],
                "scoring_implementation": trainer_implementation_contract(),
                "scorer_process_id": os.getpid(),
                "optimizer_created": False,
                "optimizer_steps": 0,
                "elapsed_seconds": time.monotonic() - started,
                "evidence": evidence,
            },
            "scoring_cid",
        )
        path = campaign_root / SCORING_RELATIVE_PATH
        _write_exclusive_json(path, record)
        queue.put(
            {
                "ok": True,
                "pointer": {
                    "path": SCORING_RELATIVE_PATH,
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


def _validate_scoring_evidence(
    evidence: object,
    *,
    population_cid: str,
    arm_results: Mapping[str, Mapping[str, Any]],
) -> None:
    if not isinstance(evidence, Mapping):
        raise ValueError("V5 scoring evidence is not an object")
    prompt_values = evidence.get("prompt_scores")
    fresh_values = evidence.get("fresh_language")
    mechanics = evidence.get("mechanics")
    decision = evidence.get("decision")
    auxiliary = evidence.get("auxiliary_fixed_weight_additive")
    prompt_names = {
        "geometric",
        "plain",
        "additive",
        "transport_permuted",
        "same_fitted_weights_additive",
        "state_off",
        "order_shuffled",
        "v1",
        "pooled",
    }
    fresh_names = {"geometric", "state_off", "v1", "pooled"}
    if (
        evidence.get("population_cid") != population_cid
        or not isinstance(evidence.get("backend"), Mapping)
        or evidence["backend"].get("platform") != "Darwin"
        or evidence["backend"].get("blas") != "Apple Accelerate"
        or evidence["backend"].get("threads") != ELIGIBLE_PLANS[0].threads_per_worker
        or not isinstance(prompt_values, Mapping)
        or set(prompt_values) != prompt_names
        or not isinstance(fresh_values, Mapping)
        or set(fresh_values) != fresh_names
        or not isinstance(mechanics, Mapping)
        or not isinstance(decision, Mapping)
        or not isinstance(auxiliary, Mapping)
    ):
        raise ValueError("V5 scoring evidence envelope differs")
    scores = {
        name: _prompt_score_from_record(prompt_values[name], name=name)
        for name in sorted(prompt_names)
    }
    fresh = {
        name: _validate_fresh_record(fresh_values[name], name=name)
        for name in sorted(fresh_names)
    }
    expected_state_off = bool(
        scores["state_off"].trace_cid == scores["v1"].trace_cid
        and scores["state_off"].suffix_logits_trace_cid
        == scores["v1"].suffix_logits_trace_cid
        and scores["state_off"].direction_gains_nats_per_token
        == scores["v1"].direction_gains_nats_per_token
        and scores["state_off"].own_nll_nats_per_token
        == scores["v1"].own_nll_nats_per_token
        and scores["state_off"].foreign_nll_nats_per_token
        == scores["v1"].foreign_nll_nats_per_token
    )
    expected_fresh_state_off = bool(
        fresh["state_off"]["logits_trace_cid"] == fresh["v1"]["logits_trace_cid"]
        and fresh["state_off"]["prediction_trace_cid"]
        == fresh["v1"]["prediction_trace_cid"]
        and fresh["state_off"]["ce_nats"] == fresh["v1"]["ce_nats"]
        and fresh["state_off"]["top1_correct"] == fresh["v1"]["top1_correct"]
        and fresh["state_off"]["top1_rate"] == fresh["v1"]["top1_rate"]
    )
    order = mechanics.get("order_shuffle")
    if (
        mechanics.get("state_off_exact_v1_reproduction") is not expected_state_off
        or mechanics.get("fresh_state_off_exact_v1_reproduction")
        is not expected_fresh_state_off
        or mechanics.get("equal_geometric_state_off_fresh_work")
        is not (
            fresh["geometric"]["work_signature"]
            == fresh["state_off"]["work_signature"]
        )
        or mechanics.get("equal_predictive_prompt_work")
        is not (
            len(
                {
                    scores[name].work_signature
                    for name in (
                        "geometric",
                        "plain",
                        "additive",
                        "transport_permuted",
                        "same_fitted_weights_additive",
                        "state_off",
                        "order_shuffled",
                    )
                }
            )
            == 1
        )
        or not isinstance(order, Mapping)
        or order.get("permutation_cid") != ORDER_SHUFFLE_CID
        or order.get("identity") is not False
        or order.get("bijection") is not True
        or order.get("token_multiset_preserved") is not True
        or order.get("audit_only") is not True
        or mechanics.get("post_reveal_optimizer_steps") != 0
        or mechanics.get("all_arm_replay_exact")
        is not all(
            float(arm_results[arm]["artifact_replay_maximum_logits_delta"]) == 0.0
            for arm in ARMS
        )
        or mechanics.get("all_bases_unchanged")
        is not all(
            arm_results[arm].get("qualified_base_unchanged") is True for arm in ARMS
        )
        or mechanics.get("initial_binding_values_byte_identical")
        is not (
            len({str(arm_results[arm]["initial_binding_cid"]) for arm in ARMS}) == 1
        )
        or mechanics.get("equal_completed_work")
        is not all(
            arm_results[arm].get("completed_steps") == OPTIMIZER_STEPS
            for arm in ARMS
        )
    ):
        raise ValueError("V5 scoring mechanics do not reproduce")
    expected_mechanics_pass = bool(
        mechanics.get("forbidden_reads") == 0
        and mechanics.get("equal_predictive_prompt_work") is True
        and mechanics.get("geometric_artifact_replay_exact") is True
        and mechanics.get("state_off_exact_v1_reproduction") is True
        and mechanics.get("fresh_state_off_exact_v1_reproduction") is True
        and mechanics.get("equal_geometric_state_off_fresh_work") is True
        and order.get("identity") is False
        and order.get("bijection") is True
        and order.get("token_multiset_preserved") is True
        and order.get("deterministic_replay_exact") is True
        and order.get("head_trace_effect") is True
        and mechanics.get("post_reveal_optimizer_steps") == 0
        and mechanics.get("all_arm_replay_exact") is True
        and mechanics.get("all_bases_unchanged") is True
        and mechanics.get("initial_binding_values_byte_identical") is True
        and mechanics.get("equal_completed_work") is True
    )
    if mechanics.get("passed") is not expected_mechanics_pass:
        raise ValueError("V5 mechanics verdict does not reproduce")
    expected_decision = terminal_decision(
        scores=scores,
        fresh=fresh,
        mechanics=mechanics,
    )
    fixed = scores["same_fitted_weights_additive"]
    expected_auxiliary = {
        "role": "AUXILIARY_ONLY_NOT_DELTA_ATTRIBUTION",
        "gain_delta_from_geometric": (
            scores["geometric"].mean_gain_nats_per_token
            - fixed.mean_gain_nats_per_token
        ),
        "own_nll_delta_from_geometric": (
            fixed.own_nll_nats_per_token
            - scores["geometric"].own_nll_nats_per_token
        ),
    }
    if decision != expected_decision or auxiliary != expected_auxiliary:
        raise ValueError("V5 terminal decision does not reproduce from scores")


def _load_scoring_evidence(root: Path) -> dict[str, Any]:
    record = _read_json(root / SCORING_RELATIVE_PATH)
    _verify_self_cid(record, "scoring_cid")
    preparation = load_predictive_block_delta_terminal_preparation(root)
    arm_results = {arm: _load_arm_result(root, arm) for arm in ARMS}
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    recovery = (
        prepare_predictive_block_delta_scoring_recovery(root)
        if (root / SCORING_RECOVERY_RELATIVE_PATH).exists()
        else None
    )
    expected_reveal = _expected_reveal(preparation, arm_results)
    scorer_process_id = record.get("scorer_process_id")
    elapsed = record.get("elapsed_seconds")
    if (
        record.get("schema") != SCORING_SCHEMA
        or record.get("issue") != ISSUE
        or record.get("policy") != POLICY
        or record.get("preparation_cid")
        != preparation.manifest["preparation_cid"]
        or record.get("reveal_cid") != reveal.get("reveal_cid")
        or reveal != expected_reveal
        or record.get("arm_result_cids")
        != {arm: arm_results[arm]["arm_result_cid"] for arm in ARMS}
        or record.get("recovery_cid")
        != (recovery["recovery_cid"] if recovery is not None else None)
        or record.get("fit_implementation")
        != preparation.manifest["implementation"]
        or record.get("scoring_implementation")
        != trainer_implementation_contract()
        or isinstance(scorer_process_id, bool)
        or not isinstance(scorer_process_id, int)
        or scorer_process_id < 1
        or record.get("optimizer_created") is not False
        or record.get("optimizer_steps") != 0
        or isinstance(elapsed, bool)
        or not isinstance(elapsed, (int, float))
        or not math.isfinite(float(elapsed))
        or float(elapsed) < 0.0
        or not isinstance(record.get("evidence"), Mapping)
    ):
        raise ValueError("V5 scoring evidence differs")
    _validate_scoring_evidence(
        record["evidence"],
        population_cid=str(preparation.manifest["population_cid"]),
        arm_results=arm_results,
    )
    return record


def _spawn_scoring(root: Path) -> dict[str, Any]:
    path = root / SCORING_RELATIVE_PATH
    if path.exists():
        return {"ok": True, "record": _load_scoring_evidence(root), "cached": True}
    context = mp.get_context("spawn")
    queue = context.Queue()
    process = context.Process(
        target=_scoring_worker,
        args=(str(root), queue),
        name="predictive-v5-canonical-scoring",
    )
    process.start()
    outcome = _collect_worker(process, queue, timeout=SCORING_HARD_WALL_SECONDS)
    if not outcome.get("ok"):
        return outcome
    pointer = outcome.get("pointer")
    record = _load_scoring_evidence(root)
    path = root / SCORING_RELATIVE_PATH
    if pointer != {
        "path": SCORING_RELATIVE_PATH,
        "bytes": path.stat().st_size,
        "cid": cid_file(path),
    }:
        raise ValueError("V5 scoring worker pointer differs")
    return {"ok": True, "record": record, "cached": False}


def _write_unavailable(root: Path, *, reason: Any, phase: str) -> dict[str, Any]:
    path = root / UNAVAILABLE_RELATIVE_PATH
    if path.exists():
        value = _read_json(path)
        _verify_self_cid(value, "unavailable_cid")
        return value
    result = _with_cid(
        {
            "schema": "uor-r4.predictive-block-delta-v5-unavailable/1",
            "issue": ISSUE,
            "policy": POLICY,
            "phase": phase,
            "reason": reason,
            "verdict": "UNAVAILABLE_PREDICTIVE_V5_COMPUTE",
            "scientific_result": "NOT_RUN",
        },
        "unavailable_cid",
    )
    _write_exclusive_json(path, result)
    return result


def _expected_scoring_recovery(
    root: Path, *, require_outputs_absent: bool = True
) -> dict[str, Any]:
    preparation = load_predictive_block_delta_terminal_preparation(root)
    probe = probe_predictive_block_delta_terminal(root)
    started = _read_json(root / STARTED_RELATIVE_PATH)
    _verify_self_cid(started, "started_cid")
    _validate_bound_implementation(
        started.get("implementation"),
        envelope_cid=started.get("started_cid"),
        frozen_envelope_cid=FROZEN_V5_STARTED_CID,
    )
    arm_results = {arm: _load_arm_result(root, arm) for arm in ARMS}
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    unavailable = _read_json(root / UNAVAILABLE_RELATIVE_PATH)
    _verify_self_cid(unavailable, "unavailable_cid")
    fit_budget = _read_json(root / FIT_BUDGET_RELATIVE_PATH)
    _verify_self_cid(fit_budget, "fit_budget_cid")
    plan = _selected_plan(probe)
    error = unavailable.get("reason", {}).get("error", {})
    expected_arm_cids = {
        arm: arm_results[arm]["arm_result_cid"] for arm in ARMS
    }
    if (
        preparation.manifest.get("preparation_cid") != FROZEN_V5_PREPARATION_CID
        or preparation.commitment.get("commitment_cid") != FROZEN_V5_COMMITMENT_CID
        or probe.get("probe_cid") != FROZEN_V5_PROBE_CID
        or started.get("started_cid") != FROZEN_V5_STARTED_CID
        or reveal.get("reveal_cid") != FROZEN_V5_REVEAL_CID
        or reveal != _expected_reveal(preparation, arm_results)
        or unavailable.get("unavailable_cid") != FROZEN_V5_UNAVAILABLE_CID
        or unavailable.get("phase") != "SCORING"
        or unavailable.get("verdict") != "UNAVAILABLE_PREDICTIVE_V5_COMPUTE"
        or unavailable.get("scientific_result") != "NOT_RUN"
        or error.get("type") != "ValueError"
        or error.get("reason")
        != "geometric V5 language work changed between batches"
        or expected_arm_cids != FROZEN_V5_ARM_RESULT_CIDS
        or fit_budget.get("fit_budget_cid") != FROZEN_V5_FIT_BUDGET_CID
        or fit_budget.get("plan_cid") != plan.identity()["plan_cid"]
        or float(fit_budget.get("consumed_seconds", -1.0))
        != _load_fit_budget(root, plan)
        or preparation.manifest["implementation"].get("tree_cid")
        != FROZEN_V5_FIT_IMPLEMENTATION_TREE_CID
        or (
            require_outputs_absent
            and (
                (root / SCORING_RELATIVE_PATH).exists()
                or (root / RESULT_RELATIVE_PATH).exists()
            )
        )
        or any(_checkpoint_path(root, arm).exists() for arm in ARMS)
    ):
        raise ValueError("V5 scoring recovery does not match the frozen failed attempt")
    scoring_implementation = trainer_implementation_contract()
    if scoring_implementation["tree_cid"] == FROZEN_V5_FIT_IMPLEMENTATION_TREE_CID:
        raise ValueError("V5 scoring recovery requires a distinct corrected implementation")
    return _with_cid(
        {
            "schema": SCORING_RECOVERY_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "phase": "SCORING_ONLY_RECOVERY",
            "preparation_cid": FROZEN_V5_PREPARATION_CID,
            "commitment_cid": FROZEN_V5_COMMITMENT_CID,
            "probe_cid": FROZEN_V5_PROBE_CID,
            "started_cid": FROZEN_V5_STARTED_CID,
            "reveal_cid": FROZEN_V5_REVEAL_CID,
            "unavailable_cid": FROZEN_V5_UNAVAILABLE_CID,
            "unavailable_phase": "SCORING",
            "unavailable_verdict": "UNAVAILABLE_PREDICTIVE_V5_COMPUTE",
            "unavailable_scientific_result": "NOT_RUN",
            "failure": {
                "type": "ValueError",
                "reason": "geometric V5 language work changed between batches",
            },
            "fit_implementation": preparation.manifest["implementation"],
            "scoring_implementation": scoring_implementation,
            "arm_result_cids": expected_arm_cids,
            "artifacts": {
                arm: dict(arm_results[arm]["artifact"]) for arm in ARMS
            },
            "fit_budget": {
                "cid": fit_budget["fit_budget_cid"],
                "consumed_seconds": fit_budget["consumed_seconds"],
            },
            "completed_steps_per_arm": OPTIMIZER_STEPS,
            "post_reveal_optimizer_steps": 0,
            "optimizer_created": False,
            "optimizer_steps": 0,
            "transition": "PRESERVE_FROZEN_ARMS_AND_REVEAL_RETRY_SCORING_ONLY",
            "repair": {
                "preserve_windows": FRESH_HELDOUT_WINDOWS,
                "full_batches": FRESH_HELDOUT_WINDOWS // BATCH_SIZE,
                "tail_rows": FRESH_HELDOUT_WINDOWS % BATCH_SIZE,
                "work_rule": "EXACT_PER_ROW_PROPORTIONALITY_AND_TOTAL_AGGREGATION",
            },
        },
        "recovery_cid",
    )


def prepare_predictive_block_delta_scoring_recovery(root: Path) -> dict[str, Any]:
    """Bind the one scoring-only recovery without changing fit or reveal state."""

    root = root.resolve()
    path = root / SCORING_RECOVERY_RELATIVE_PATH
    if path.exists():
        value = _read_json(path)
        _verify_self_cid(value, "recovery_cid")
        if value != _expected_scoring_recovery(
            root, require_outputs_absent=False
        ):
            raise ValueError("cached V5 scoring recovery differs")
        return value
    value = _expected_scoring_recovery(root)
    _write_exclusive_json(path, value)
    return value


def _write_scoring_recovery_unavailable(
    root: Path, *, recovery: Mapping[str, Any], reason: Any
) -> dict[str, Any]:
    path = root / SCORING_RECOVERY_UNAVAILABLE_RELATIVE_PATH
    if path.exists():
        value = _read_json(path)
        _verify_self_cid(value, "unavailable_cid")
        recorded_reason = value.get("reason")
        recorded_error = (
            recorded_reason.get("error")
            if isinstance(recorded_reason, Mapping)
            else None
        )
        if (
            value.get("schema")
            != "uor-r4.predictive-block-delta-v5-recovery-unavailable/1"
            or value.get("issue") != ISSUE
            or value.get("policy") != POLICY
            or value.get("phase") != "SCORING_RECOVERY"
            or value.get("verdict") != "UNAVAILABLE_PREDICTIVE_V5_COMPUTE"
            or value.get("scientific_result") != "NOT_RUN"
            or value.get("recovery_cid") != recovery.get("recovery_cid")
            or value.get("supersedes_unavailable_cid")
            != FROZEN_V5_UNAVAILABLE_CID
            or value.get("scoring_implementation")
            != recovery.get("scoring_implementation")
            or not isinstance(recorded_reason, Mapping)
            or recorded_reason.get("ok") is not False
            or not isinstance(recorded_error, Mapping)
            or not isinstance(recorded_error.get("type"), str)
            or not isinstance(recorded_error.get("reason"), str)
        ):
            raise ValueError("cached V5 scoring-recovery unavailable differs")
        return value
    value = _with_cid(
        {
            "schema": "uor-r4.predictive-block-delta-v5-recovery-unavailable/1",
            "issue": ISSUE,
            "policy": POLICY,
            "phase": "SCORING_RECOVERY",
            "reason": reason,
            "verdict": "UNAVAILABLE_PREDICTIVE_V5_COMPUTE",
            "scientific_result": "NOT_RUN",
            "recovery_cid": recovery["recovery_cid"],
            "supersedes_unavailable_cid": FROZEN_V5_UNAVAILABLE_CID,
            "scoring_implementation": recovery["scoring_implementation"],
        },
        "unavailable_cid",
    )
    _write_exclusive_json(path, value)
    return value


def _load_terminal_result(root: Path) -> dict[str, Any]:
    result = _read_json(root / RESULT_RELATIVE_PATH)
    _verify_self_cid(result, "result_cid")
    decision = result.get("decision")
    preparation = load_predictive_block_delta_terminal_preparation(root)
    probe = _read_json(root / PROBE_RELATIVE_PATH)
    _verify_self_cid(probe, "probe_cid")
    plan = _selected_plan(probe)
    started = _read_json(root / STARTED_RELATIVE_PATH)
    _verify_self_cid(started, "started_cid")
    arm_results = {arm: _load_arm_result(root, arm) for arm in ARMS}
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    recovery = (
        prepare_predictive_block_delta_scoring_recovery(root)
        if (root / SCORING_RECOVERY_RELATIVE_PATH).exists()
        else None
    )
    scoring = _load_scoring_evidence(root)
    writer_process_id = result.get("writer_process_id")
    aggregate_wall = result.get("aggregate_fit_wall_seconds")
    if (
        result.get("schema") != RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
        or result.get("model_policy") != MODEL_POLICY
        or not isinstance(decision, Mapping)
        or result.get("preparation_cid")
        != preparation.manifest["preparation_cid"]
        or result.get("probe_cid") != probe.get("probe_cid")
        or result.get("started_cid") != started.get("started_cid")
        or result.get("implementation") != preparation.manifest["implementation"]
        or result.get("scoring_implementation")
        != scoring.get("scoring_implementation")
        or result.get("recovery_cid")
        != (recovery["recovery_cid"] if recovery is not None else None)
        or result.get("supersedes_unavailable_cid")
        != (FROZEN_V5_UNAVAILABLE_CID if recovery is not None else None)
        or result.get("plan") != plan.identity()
        or isinstance(writer_process_id, bool)
        or not isinstance(writer_process_id, int)
        or writer_process_id < 1
        or int(scoring["scorer_process_id"]) == writer_process_id
        or isinstance(aggregate_wall, bool)
        or not isinstance(aggregate_wall, (int, float))
        or not math.isfinite(float(aggregate_wall))
        or not 0.0 <= float(aggregate_wall) <= HARD_WALL_SECONDS
        or float(aggregate_wall) != _load_fit_budget(root, plan)
        or result.get("hard_wall_seconds_before_scoring") != HARD_WALL_SECONDS
        or result.get("arm_results")
        != {arm: arm_results[arm]["arm_result_cid"] for arm in ARMS}
        or result.get("artifacts")
        != {arm: dict(arm_results[arm]["artifact"]) for arm in ARMS}
        or result.get("reveal") != reveal
        or reveal != _expected_reveal(preparation, arm_results)
        or result.get("scoring_cid") != scoring.get("scoring_cid")
        or result.get("evidence") != scoring.get("evidence")
        or decision != scoring["evidence"].get("decision")
        or result.get("verdict") != decision.get("verdict")
        or result.get("next_action")
        != _terminal_next_action(decision, scoring["evidence"]["mechanics"])
        or result.get("post_reveal_optimizer_steps") != 0
        or result.get("nonclaims")
        != {
            "coherent_generation": "NOT_RUN",
            "reasoning": "NOT_RUN",
            "integer_table_lowering": "NOT_RUN",
            "release_readiness": "NOT_ESTABLISHED",
        }
    ):
        raise ValueError("cached V5 terminal result differs")
    return result


def _terminal_next_action(
    decision: Mapping[str, Any], mechanics: Mapping[str, Any]
) -> str:
    if (
        decision.get("verdict")
        == "PREDICTIVE_GEOMETRIC_CAPACITY_AND_ATTRIBUTION_PASS"
        and decision.get("capacity_positive") is True
        and decision.get("fresh_language_positive") is True
        and decision.get("geometry_positive") is True
        and decision.get("integrity_positive") is True
        and mechanics.get("passed") is True
    ):
        return "FREEZE_ONE_BOUNDED_AUTONOMOUS_GENERATION_RUNG"
    if (
        decision.get("verdict") == "PREDICTIVE_CAPACITY_WITHOUT_GEOMETRY_ATTRIBUTION"
        and decision.get("capacity_positive") is True
        and decision.get("fresh_language_positive") is True
        and decision.get("geometry_positive") is False
        and decision.get("integrity_positive") is True
        and mechanics.get("passed") is True
    ):
        return "ISOLATE_LEAF_CONNECTION_TERM"
    return "STOP_WITHOUT_GENERATION"


def run_predictive_block_delta_terminal(
    root: Path, *, resume: bool = False
) -> dict[str, Any]:
    """Fit once, reveal once, and score once under the frozen V5 contract."""

    root = root.resolve()
    if (root / RESULT_RELATIVE_PATH).exists():
        return _load_terminal_result(root)
    preparation = load_predictive_block_delta_terminal_preparation(root)
    probe = probe_predictive_block_delta_terminal(root)
    if probe.get("eligible") is not True:
        return _write_unavailable(
            root, reason="NO_ELIGIBLE_CPU_EXECUTION_PLAN", phase="PROBE"
        )
    plan = _selected_plan(probe)
    started_path = root / STARTED_RELATIVE_PATH
    if started_path.exists():
        started = _read_json(started_path)
        _verify_self_cid(started, "started_cid")
        _validate_bound_implementation(
            started.get("implementation"),
            envelope_cid=started.get("started_cid"),
            frozen_envelope_cid=FROZEN_V5_STARTED_CID,
        )
        writer_process_id = started.get("writer_process_id")
        if (
            started.get("schema") != STARTED_SCHEMA
            or started.get("issue") != ISSUE
            or started.get("policy") != POLICY
            or started.get("preparation_cid")
            != preparation.manifest["preparation_cid"]
            or started.get("probe_cid") != probe["probe_cid"]
            or started.get("plan") != plan.identity()
            or started.get("implementation") != preparation.manifest["implementation"]
            or started.get("training") != preparation.manifest["training"]
            or started.get("v5_reads_before_artifact_freeze") != 0
            or started.get("hard_wall_seconds_before_scoring")
            != HARD_WALL_SECONDS
            or isinstance(writer_process_id, bool)
            or not isinstance(writer_process_id, int)
            or writer_process_id < 1
        ):
            raise ValueError("cached V5 started envelope differs")
        if not resume and not all(_arm_result_path(root, arm).exists() for arm in ARMS):
            raise RuntimeError("V5 terminal run already started; explicit --resume required")
    else:
        if resume:
            raise FileNotFoundError("V5 resume requested before a started envelope exists")
        started = _with_cid(
            {
                "schema": STARTED_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "preparation_cid": preparation.manifest["preparation_cid"],
                "probe_cid": probe["probe_cid"],
                "plan": plan.identity(),
                "implementation": trainer_implementation_contract(),
                "training": preparation.manifest["training"],
                "writer_process_id": os.getpid(),
                "v5_reads_before_artifact_freeze": 0,
                "hard_wall_seconds_before_scoring": HARD_WALL_SECONDS,
            },
            "started_cid",
        )
        _write_exclusive_json(started_path, started)

    recovery: dict[str, Any] | None = None
    if (root / UNAVAILABLE_RELATIVE_PATH).exists():
        if not resume:
            raise RuntimeError("V5 scoring-only recovery requires explicit --resume")
        recovery = prepare_predictive_block_delta_scoring_recovery(root)
        if (root / SCORING_RECOVERY_UNAVAILABLE_RELATIVE_PATH).exists():
            return _write_scoring_recovery_unavailable(
                root, recovery=recovery, reason={}
            )

    fit = _run_fit_batch(
        root,
        plan,
        ARMS,
        resume=resume,
        wall_seconds=HARD_WALL_SECONDS,
    )
    if fit.get("ok") is not True:
        return _write_unavailable(root, reason=fit, phase="FIT")
    arm_results = {arm: _load_arm_result(root, arm) for arm in ARMS}
    aggregate_wall = float(fit["aggregate_wall_seconds"])
    if aggregate_wall > HARD_WALL_SECONDS:
        return _write_unavailable(
            root, reason="AGGREGATE_FIT_WALL_EXCEEDED", phase="FIT"
        )
    if len({arm_results[arm]["initial_binding_cid"] for arm in ARMS}) != 1:
        raise ValueError("V5 fitted arms did not start byte-identically")
    reveal = _reveal_terminal_population(preparation, arm_results)
    scoring = _spawn_scoring(root)
    if scoring.get("ok") is not True:
        if recovery is not None:
            return _write_scoring_recovery_unavailable(
                root, recovery=recovery, reason=scoring
            )
        return _write_unavailable(root, reason=scoring, phase="SCORING")
    scoring_record = scoring["record"]
    evidence = scoring_record["evidence"]
    decision = evidence["decision"]
    result = _with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "preparation_cid": preparation.manifest["preparation_cid"],
            "probe_cid": probe["probe_cid"],
            "started_cid": started["started_cid"],
            "writer_process_id": os.getpid(),
            "implementation": preparation.manifest["implementation"],
            "scoring_implementation": scoring_record["scoring_implementation"],
            "recovery_cid": (
                recovery["recovery_cid"] if recovery is not None else None
            ),
            "supersedes_unavailable_cid": (
                FROZEN_V5_UNAVAILABLE_CID if recovery is not None else None
            ),
            "plan": plan.identity(),
            "aggregate_fit_wall_seconds": aggregate_wall,
            "hard_wall_seconds_before_scoring": HARD_WALL_SECONDS,
            "arm_results": {
                arm: arm_results[arm]["arm_result_cid"] for arm in ARMS
            },
            "artifacts": {
                arm: dict(arm_results[arm]["artifact"]) for arm in ARMS
            },
            "reveal": reveal,
            "scoring_cid": scoring_record["scoring_cid"],
            "evidence": evidence,
            "decision": decision,
            "verdict": decision["verdict"],
            "post_reveal_optimizer_steps": 0,
            "next_action": _terminal_next_action(decision, evidence["mechanics"]),
            "nonclaims": {
                "coherent_generation": "NOT_RUN",
                "reasoning": "NOT_RUN",
                "integer_table_lowering": "NOT_RUN",
                "release_readiness": "NOT_ESTABLISHED",
            },
        },
        "result_cid",
    )
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    return _load_terminal_result(root)


def verify_predictive_block_delta_terminal(root: Path) -> dict[str, Any]:
    """Fresh-process exact rescore; never constructs an optimizer."""

    root = root.resolve()
    path = root / VERIFICATION_RELATIVE_PATH
    if path.exists():
        value = _read_json(path)
        _verify_self_cid(value, "verification_cid")
        result = _load_terminal_result(root)
        scoring = _load_scoring_evidence(root)
        verifier_process_id = value.get("verifier_process_id")
        exact = value.get("exact_evidence_reproduction")
        if (
            value.get("schema") != VERIFICATION_SCHEMA
            or value.get("issue") != ISSUE
            or value.get("policy") != POLICY
            or value.get("result_cid") != result.get("result_cid")
            or value.get("scoring_cid") != scoring.get("scoring_cid")
            or value.get("recovery_cid") != result.get("recovery_cid")
            or value.get("scoring_implementation_tree_cid")
            != scoring["scoring_implementation"]["tree_cid"]
            or value.get("run_writer_process_id")
            != result.get("writer_process_id")
            or value.get("scoring_writer_process_id")
            != scoring.get("scorer_process_id")
            or isinstance(verifier_process_id, bool)
            or not isinstance(verifier_process_id, int)
            or verifier_process_id < 1
            or verifier_process_id
            in (result["writer_process_id"], scoring["scorer_process_id"])
            or value.get("optimizer_created") is not False
            or value.get("optimizer_steps") != 0
            or not isinstance(exact, bool)
            or value.get("verdict")
            != ("VERIFIED" if exact else "INVALID_REPLAY")
        ):
            raise ValueError("cached V5 independent verification differs")
        return value
    result = _load_terminal_result(root)
    scoring = _load_scoring_evidence(root)
    if os.getpid() in (
        int(result["writer_process_id"]),
        int(scoring["scorer_process_id"]),
    ):
        raise RuntimeError("V5 independent verifier must use a different process")
    preparation = load_predictive_block_delta_terminal_preparation(root)
    arm_results = {arm: _load_arm_result(root, arm) for arm in ARMS}
    observed = _score_campaign(preparation, arm_results)
    exact = observed == scoring["evidence"] == result["evidence"]
    verification = _with_cid(
        {
            "schema": VERIFICATION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "result_cid": result["result_cid"],
            "scoring_cid": scoring["scoring_cid"],
            "recovery_cid": result.get("recovery_cid"),
            "scoring_implementation_tree_cid": scoring[
                "scoring_implementation"
            ]["tree_cid"],
            "run_writer_process_id": result["writer_process_id"],
            "scoring_writer_process_id": scoring["scorer_process_id"],
            "verifier_process_id": os.getpid(),
            "optimizer_created": False,
            "optimizer_steps": 0,
            "exact_evidence_reproduction": exact,
            "verdict": "VERIFIED" if exact else "INVALID_REPLAY",
        },
        "verification_cid",
    )
    _write_exclusive_json(path, verification)
    return verification


__all__ = [
    "ARMS",
    "ELIGIBLE_PLANS",
    "TerminalPromptScore",
    "TerminalPreparation",
    "load_predictive_block_delta_terminal_preparation",
    "prepare_predictive_block_delta_terminal",
    "prepare_predictive_block_delta_scoring_recovery",
    "probe_predictive_block_delta_terminal",
    "run_predictive_block_delta_terminal",
    "select_execution_plan",
    "terminal_decision",
    "verify_predictive_block_delta_terminal",
]
