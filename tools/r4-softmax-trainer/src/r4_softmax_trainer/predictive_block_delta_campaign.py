"""Disposable admission gate for #973 predictive block-delta binding.

This module is intentionally narrower than a production campaign.  It opens
only the already revealed V4 prompt population, fits no more than 256 updates,
destroys the fitted values in memory, and emits a binary admission record.
Nothing here selects, creates, or opens the independently frozen V5 population.
"""

from __future__ import annotations

import json
import math
import os
import re
import struct
import time
from collections.abc import Callable, Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Literal, Protocol

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .h4_spin_frame_sidecar import (
    PRODUCT_TABLE_KAPPA,
    ROOT_TABLE_KAPPA,
    H4SpinFrameArtifactV1,
)
from .language_path_generalization_campaign import _exact_geometry
from .layerwise_normalized_retained_readout_campaign import (
    PREDECESSOR_ARTIFACT_BYTES,
    PREDECESSOR_ARTIFACT_CID,
    PREDECESSOR_POLICY,
    PREDECESSOR_RESULT_CID,
    _verify_predecessor,
)
from .prompt_conditioning_v4 import (
    BOS_TOKEN_ID,
    CONTINUATION_TOKENS,
    PROMPT_TOKENS,
    PromptDirection,
    PromptConditioningPair,
    load_revealed_prompt_conditioning_population,
)
from .provenance import (
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    tree_cid,
)

ISSUE = 973
POLICY = "R4PredictiveBlockDeltaPromptCapacityV1"
MODEL_POLICY = "R4PredictiveBlockDeltaBindingV1"

PROBE_PAIRS = 32
PROBE_DIRECTIONS = 64
PROBE_TARGETS = 1_024
MAXIMUM_UPDATES = 256
BATCH_DIRECTIONS = 8
INITIALIZATION_SEED = 9_739
LEARNING_RATE = 3.0e-3
ADAM_BETAS = (0.9, 0.95)
ADAM_EPSILON = 1.0e-8
GRADIENT_CLIP = 1.0
HARD_WALL_SECONDS = 300.0
TRAINABLE_PARAMETERS = 9_228

ABSOLUTE_GAIN_THRESHOLD = math.log(2.0) / CONTINUATION_TOKENS
INTERVENTION_LOSS_THRESHOLD = math.log(1.5) / CONTINUATION_TOKENS
WIN_THRESHOLD = 52

V4_POPULATION_CID = (
    "blake3:cc9a1c40fe753e269ea31edd804c32b2a0c208ef20fceb1167636d6f28d7da11"
)
V4_COMMITMENT_CID = (
    "blake3:bc490bc0c4354ae08b00978dc6657200afb1638409f191c714693f0886981f58"
)
V4_REVEAL_CID = (
    "blake3:0fcbeffa06ed2ef7496a5ead77ff9a81320c44a4e4aec2d29082f86c0b8634a9"
)
V4_GEOMETRIC_ARTIFACT_CID = (
    "blake3:85a33965a7cd9ee952948ed6e6c5a925585edb9496377baa56a22ffaca40175f"
)
V4_POOLED_ARTIFACT_CID = (
    "blake3:4eeba8bb99d200e77558d89529a1e9f33d7c1ea6f4439ec3cae64c79d0b0f0d1"
)
H4_FRAME_ARTIFACT_CID = (
    "blake3:f1f556d3c93a2e21593c4f48de13efd64705fec11f7660e0b6fac7ba49263099"
)
H4_FRAME_FILE_CID = (
    "blake3:9df624162d14ba133fed34c560e4828961a4dc8d6a9438c731e8f8c209c16ad4"
)
V1_IMPLEMENTATION_TREE_CID = (
    "blake3:317aa564eb2041c37768186d81691666a19ce01c168e4ab74a0b966d761212d2"
)
V4_POPULATION_RELATIVE_PATH = "evaluation/sealed/prompt-population.json"
V4_REVEAL_RELATIVE_PATH = "evaluation/reveal.json"

RESULT_RELATIVE_PATH = "preflight/predictive-block-delta-admission.json"
RESULT_SCHEMA = "uor-r4.predictive-block-delta-admission/1"
VERDICT_ADMIT = "PREDICTIVE_BINDING_EXPRESSIVITY_ADMIT"
VERDICT_REJECT = "PREDICTIVE_BINDING_NOT_OBSERVABLE"
VERDICT_INVALID = "INVALID_PREDICTIVE_BINDING_PREFLIGHT"

_CID_PATTERN = re.compile(r"blake3:[0-9a-f]{64}\Z")


class _Audit(Protocol):
    forbidden_reads: int

    def work_signature(self) -> tuple[int, ...]: ...


class _Output(Protocol):
    logits: Tensor
    base_logits: Tensor
    head_logits: Tensor
    audit: _Audit


class _ProbeModel(Protocol):
    frame_matrices: Tensor
    frame_multiplication: Tensor

    def __call__(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        intervention: Literal[
            "native", "transport_permuted", "no_delta", "state_off"
        ] = "native",
    ) -> _Output: ...

    def train(self, mode: bool = True) -> Any: ...

    def eval(self) -> Any: ...

    def trainable_parameters(self) -> Sequence[torch.nn.Parameter]: ...

    def frozen_base_parameters(self) -> Sequence[torch.nn.Parameter]: ...

    def export_qualified_base_artifact(self) -> bytes: ...

    def export_binding_artifact(self) -> bytes: ...

    def load_binding_artifact(self, artifact: bytes) -> None: ...

    def _step_transport(
        self,
        leaves: Tensor,
        *,
        intervention: Literal[
            "native", "transport_permuted", "no_delta", "state_off"
        ],
    ) -> Tensor: ...


@dataclass(frozen=True, slots=True)
class ProbeScore:
    intervention: str
    directions: int
    targets: int
    mean_gain_nats_per_token: float
    wins: int
    own_nll_nats_per_token: float
    foreign_nll_nats_per_token: float
    maximum_head_logits: float
    forbidden_reads: int
    work_signature: tuple[int, ...]
    trace_cid: str

    def record(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True, slots=True)
class FrozenProbeInputs:
    predecessor: Any
    predecessor_artifact_path: Path
    frames: H4SpinFrameArtifactV1
    pairs: tuple[PromptConditioningPair, ...]
    records: Mapping[str, Any]


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


def _with_self_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
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


def load_frozen_probe_inputs(
    *,
    predecessor_root: Path,
    revealed_v4_root: Path,
    frame_sidecar_path: Path,
) -> FrozenProbeInputs:
    """Verify the qualified V1 and exact revealed V4, then take pairs 0..31."""

    predecessor, artifact_path = _verify_predecessor(predecessor_root.resolve())
    frames = H4SpinFrameArtifactV1.load(frame_sidecar_path)
    if (
        frames.artifact_cid != H4_FRAME_ARTIFACT_CID
        or frames.file_cid != H4_FRAME_FILE_CID
    ):
        raise ValueError("H4 spin-frame sidecar differs from the frozen campaign input")
    revealed_v4_root = revealed_v4_root.resolve()
    reveal_path = revealed_v4_root / V4_REVEAL_RELATIVE_PATH
    population_path = revealed_v4_root / V4_POPULATION_RELATIVE_PATH
    reveal = _read_canonical_json(reveal_path)
    _verify_self_cid(reveal, "reveal_cid")
    if (
        reveal.get("reveal_cid") != V4_REVEAL_CID
        or reveal.get("commitment_cid") != V4_COMMITMENT_CID
        or reveal.get("population_cid") != V4_POPULATION_CID
        or reveal.get("baseline_artifact_cid") != PREDECESSOR_ARTIFACT_CID
        or reveal.get("geometric_artifact_cid") != V4_GEOMETRIC_ARTIFACT_CID
        or reveal.get("pooled_artifact_cid") != V4_POOLED_ARTIFACT_CID
        or reveal.get("reveal_count") != 1
    ):
        raise ValueError("revealed V4 binding differs from the disposable freeze")
    if cid_file(population_path) != V4_POPULATION_CID:
        raise ValueError("revealed V4 population differs from its exact CID")
    population = load_revealed_prompt_conditioning_population(revealed_v4_root)
    pairs = tuple(population.pairs[:PROBE_PAIRS])
    if len(pairs) != PROBE_PAIRS or tuple(pair.pair_index for pair in pairs) != tuple(
        range(PROBE_PAIRS)
    ):
        raise ValueError("V4 disposable probe slice differs from pairs 0 through 31")
    return FrozenProbeInputs(
        predecessor=predecessor,
        predecessor_artifact_path=artifact_path,
        frames=frames,
        pairs=pairs,
        records={
            "predecessor": {
                "policy": PREDECESSOR_POLICY,
                "result_cid": PREDECESSOR_RESULT_CID,
                "artifact_cid": PREDECESSOR_ARTIFACT_CID,
                "artifact_bytes": PREDECESSOR_ARTIFACT_BYTES,
            },
            "revealed_v4": {
                "population_cid": V4_POPULATION_CID,
                "commitment_cid": V4_COMMITMENT_CID,
                "reveal_cid": V4_REVEAL_CID,
                "pairs": PROBE_PAIRS,
                "directions": PROBE_DIRECTIONS,
                "targets": PROBE_TARGETS,
            },
            "h4_spin_frames": {
                "artifact_cid": frames.artifact_cid,
                "file_cid": frames.file_cid,
                "root_table_kappa": frames.h4_root_table_kappa,
                "multiplication_table_kappa": frames.h4_multiplication_table_kappa,
            },
        },
    )


def _directions(
    pairs: Sequence[PromptConditioningPair],
) -> tuple[PromptDirection, ...]:
    directions = tuple(
        direction
        for pair in pairs
        for direction in (
            PromptDirection(
                pair_index=pair.pair_index,
                side="left",
                own_prompt=pair.left.prompt_token_ids,
                crossed_prompt=pair.right.prompt_token_ids,
                continuation=pair.left.continuation_token_ids,
            ),
            PromptDirection(
                pair_index=pair.pair_index,
                side="right",
                own_prompt=pair.right.prompt_token_ids,
                crossed_prompt=pair.left.prompt_token_ids,
                continuation=pair.right.continuation_token_ids,
            ),
        )
    )
    if len(directions) != PROBE_DIRECTIONS:
        raise RuntimeError("disposable direction count drifted")
    return directions


def _sequence(prompt: Sequence[int], continuation: Sequence[int]) -> list[int]:
    return [BOS_TOKEN_ID, *prompt, *continuation[:-1]]


def _batch(
    directions: Sequence[Any], *, device: torch.device
) -> tuple[Tensor, Tensor]:
    rows: list[list[int]] = []
    targets: list[Sequence[int]] = []
    for direction in directions:
        rows.append(_sequence(direction.own_prompt, direction.continuation))
        rows.append(_sequence(direction.crossed_prompt, direction.continuation))
        targets.append(direction.continuation)
    return (
        torch.tensor(rows, dtype=torch.long, device=device),
        torch.tensor(targets, dtype=torch.long, device=device),
    )


def _output(value: Any, *, rows: int) -> _Output:
    logits = getattr(value, "logits", None)
    base_logits = getattr(value, "base_logits", None)
    head_logits = getattr(value, "head_logits", None)
    audit = getattr(value, "audit", None)
    expected_time = 1 + PROMPT_TOKENS + CONTINUATION_TOKENS - 1
    if (
        not isinstance(logits, Tensor)
        or not isinstance(base_logits, Tensor)
        or not isinstance(head_logits, Tensor)
        or tuple(logits.shape[:2]) != (rows, expected_time)
        or logits.shape != base_logits.shape
        or logits.shape != head_logits.shape
        or not torch.isfinite(logits).all().item()
        or not torch.isfinite(base_logits).all().item()
        or not torch.isfinite(head_logits).all().item()
        or audit is None
        or isinstance(getattr(audit, "forbidden_reads", None), bool)
        or not isinstance(getattr(audit, "forbidden_reads", None), int)
        or not callable(getattr(audit, "work_signature", None))
    ):
        raise ValueError("predictive block-delta output contract differs")
    return value


def _suffix_log_probabilities(output: _Output, targets: Tensor) -> tuple[Tensor, Tensor]:
    suffix = output.logits[:, PROMPT_TOKENS : PROMPT_TOKENS + CONTINUATION_TOKENS]
    log_probabilities = F.log_softmax(suffix.float(), dim=-1)
    expanded = targets[:, None, :].expand(-1, 2, -1).reshape(-1, CONTINUATION_TOKENS)
    selected = log_probabilities.gather(2, expanded[:, :, None])[:, :, 0]
    return selected[0::2], selected[1::2]


def score_probe(
    model: _ProbeModel,
    pairs: Sequence[PromptConditioningPair],
    *,
    intervention: Literal["native", "no_delta", "state_off"],
    device: torch.device,
) -> ProbeScore:
    """Score the 64 revealed directions without retaining tensors or weights."""

    model.eval()
    directions = _directions(pairs)
    gains: list[float] = []
    own_values: list[float] = []
    foreign_values: list[float] = []
    maximum_head_logits = 0.0
    forbidden_reads = 0
    signature: tuple[int, ...] | None = None
    trace = blake3()
    with torch.inference_mode():
        for start in range(0, len(directions), BATCH_DIRECTIONS):
            batch_directions = directions[start : start + BATCH_DIRECTIONS]
            inputs, targets = _batch(batch_directions, device=device)
            output = _output(
                model(inputs, intervention=intervention), rows=len(inputs)
            )
            own, foreign = _suffix_log_probabilities(output, targets)
            own_cpu = own.double().cpu()
            foreign_cpu = foreign.double().cpu()
            work = tuple(int(value) for value in output.audit.work_signature())
            if signature is None:
                signature = work
            elif signature != work:
                raise ValueError("probe work signature changed between equal batches")
            forbidden_reads += int(output.audit.forbidden_reads)
            maximum_head_logits = max(
                maximum_head_logits,
                float(output.head_logits.detach().abs().max().cpu()),
            )
            for offset, direction in enumerate(batch_directions):
                own_row = [float(value) for value in own_cpu[offset].tolist()]
                foreign_row = [float(value) for value in foreign_cpu[offset].tolist()]
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
    if len(gains) != PROBE_DIRECTIONS or len(own_values) != PROBE_TARGETS:
        raise RuntimeError("disposable scorer did not cover the frozen population")
    assert signature is not None
    return ProbeScore(
        intervention=intervention,
        directions=PROBE_DIRECTIONS,
        targets=PROBE_TARGETS,
        mean_gain_nats_per_token=math.fsum(gains) / PROBE_DIRECTIONS,
        wins=sum(value > 0.0 for value in gains),
        own_nll_nats_per_token=-math.fsum(own_values) / PROBE_TARGETS,
        foreign_nll_nats_per_token=-math.fsum(foreign_values) / PROBE_TARGETS,
        maximum_head_logits=maximum_head_logits,
        forbidden_reads=forbidden_reads,
        work_signature=signature,
        trace_cid=f"blake3:{trace.hexdigest()}",
    )


def admission_decision(
    *,
    native: ProbeScore,
    additive: ProbeScore,
    state_off: ProbeScore,
    mechanics: Mapping[str, Any],
) -> dict[str, Any]:
    """Apply the frozen expressivity thresholds and intervention semantics."""

    work_equal = native.work_signature == additive.work_signature == state_off.work_signature
    gates = {
        "mechanics_passed": mechanics.get("passed") is True,
        "forbidden_reads_zero": (
            native.forbidden_reads == additive.forbidden_reads == state_off.forbidden_reads == 0
        ),
        "equal_work": work_equal,
        "absolute_gain": native.mean_gain_nats_per_token >= ABSOLUTE_GAIN_THRESHOLD,
        "directional_wins": native.wins >= WIN_THRESHOLD,
        "delta_over_additive": (
            native.mean_gain_nats_per_token - additive.mean_gain_nats_per_token
            >= INTERVENTION_LOSS_THRESHOLD
        ),
        "state_load_bearing": (
            native.mean_gain_nats_per_token - state_off.mean_gain_nats_per_token
            >= INTERVENTION_LOSS_THRESHOLD
        ),
    }
    integrity = all(
        gates[name]
        for name in ("mechanics_passed", "forbidden_reads_zero", "equal_work")
    )
    admitted = integrity and all(
        gates[name]
        for name in (
            "absolute_gain",
            "directional_wins",
            "delta_over_additive",
            "state_load_bearing",
        )
    )
    verdict = VERDICT_ADMIT if admitted else (VERDICT_REJECT if integrity else VERDICT_INVALID)
    return {
        "verdict": verdict,
        "admitted": admitted,
        "gates": gates,
        "thresholds": {
            "absolute_gain_nats_per_token": ABSOLUTE_GAIN_THRESHOLD,
            "wins": WIN_THRESHOLD,
            "delta_over_additive_nats_per_token": INTERVENTION_LOSS_THRESHOLD,
            "state_load_bearing_nats_per_token": INTERVENTION_LOSS_THRESHOLD,
        },
    }


def _exact_mapping(
    value: object, *, keys: tuple[str, ...], label: str
) -> Mapping[str, Any]:
    if not isinstance(value, Mapping) or set(value) != set(keys):
        raise ValueError(f"cached {label} fields differ from the result schema")
    return value


def _integer_field(value: object, *, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise ValueError(f"cached {label} must be an integer >= {minimum}")
    return value


def _float_field(value: object, *, label: str, minimum: float | None = None) -> float:
    if not isinstance(value, float) or not math.isfinite(value):
        raise ValueError(f"cached {label} must be a finite float")
    if minimum is not None and value < minimum:
        raise ValueError(f"cached {label} must be >= {minimum}")
    return value


def _boolean_field(value: object, *, label: str) -> bool:
    if not isinstance(value, bool):
        raise ValueError(f"cached {label} must be a boolean")
    return value


def _cid_field(value: object, *, label: str) -> str:
    if not isinstance(value, str) or not _CID_PATTERN.fullmatch(value):
        raise ValueError(f"cached {label} must be a BLAKE3 CID")
    return value


def _validate_historical_implementation(value: object) -> None:
    implementation = _exact_mapping(
        value, keys=("files", "tree_cid"), label="implementation"
    )
    files = implementation["files"]
    if not isinstance(files, list) or not files:
        raise ValueError("cached implementation file ledger is invalid")
    paths: list[str] = []
    for offset, item in enumerate(files):
        record = _exact_mapping(
            item,
            keys=("bytes", "cid", "path"),
            label=f"implementation.files[{offset}]",
        )
        if not isinstance(record["path"], str) or not record["path"]:
            raise ValueError("cached implementation path is invalid")
        _integer_field(
            record["bytes"],
            label=f"implementation.files[{offset}].bytes",
            minimum=1,
        )
        _cid_field(record["cid"], label=f"implementation.files[{offset}].cid")
        paths.append(record["path"])
    if paths != sorted(set(paths)):
        raise ValueError("cached implementation paths are not unique and sorted")
    observed_tree = _cid_field(
        implementation["tree_cid"], label="implementation.tree_cid"
    )
    if (
        observed_tree != V1_IMPLEMENTATION_TREE_CID
        or tree_cid(list(files)) != observed_tree
    ):
        raise ValueError("cached predictive implementation binding differs")


def _probe_score_from_record(
    value: object, *, expected_intervention: str, label: str
) -> ProbeScore:
    record = _exact_mapping(
        value,
        keys=(
            "intervention",
            "directions",
            "targets",
            "mean_gain_nats_per_token",
            "wins",
            "own_nll_nats_per_token",
            "foreign_nll_nats_per_token",
            "maximum_head_logits",
            "forbidden_reads",
            "work_signature",
            "trace_cid",
        ),
        label=label,
    )
    if record["intervention"] != expected_intervention:
        raise ValueError(f"cached {label} intervention differs")
    directions = _integer_field(record["directions"], label=f"{label}.directions")
    targets = _integer_field(record["targets"], label=f"{label}.targets")
    wins = _integer_field(record["wins"], label=f"{label}.wins")
    forbidden_reads = _integer_field(
        record["forbidden_reads"], label=f"{label}.forbidden_reads"
    )
    if directions != PROBE_DIRECTIONS or targets != PROBE_TARGETS or wins > directions:
        raise ValueError(f"cached {label} population counts differ")
    gain = _float_field(
        record["mean_gain_nats_per_token"], label=f"{label}.gain"
    )
    own_nll = _float_field(
        record["own_nll_nats_per_token"],
        label=f"{label}.own_nll",
        minimum=0.0,
    )
    foreign_nll = _float_field(
        record["foreign_nll_nats_per_token"],
        label=f"{label}.foreign_nll",
        minimum=0.0,
    )
    maximum_head = _float_field(
        record["maximum_head_logits"],
        label=f"{label}.maximum_head_logits",
        minimum=0.0,
    )
    if not math.isclose(
        gain, foreign_nll - own_nll, rel_tol=0.0, abs_tol=1.0e-12
    ):
        raise ValueError(f"cached {label} gain does not reproduce its NLLs")
    work = record["work_signature"]
    if (
        not isinstance(work, (list, tuple))
        or not work
        or any(
            isinstance(item, bool) or not isinstance(item, int) or item < 0
            for item in work
        )
    ):
        raise ValueError(f"cached {label} work signature is invalid")
    return ProbeScore(
        intervention=expected_intervention,
        directions=directions,
        targets=targets,
        mean_gain_nats_per_token=gain,
        wins=wins,
        own_nll_nats_per_token=own_nll,
        foreign_nll_nats_per_token=foreign_nll,
        maximum_head_logits=maximum_head,
        forbidden_reads=forbidden_reads,
        work_signature=tuple(work),
        trace_cid=_cid_field(record["trace_cid"], label=f"{label}.trace_cid"),
    )


def _validate_cached_result(value: Mapping[str, Any]) -> None:
    """Reproduce every decision-bearing field without reopening V4."""

    _verify_self_cid(value, "result_cid")
    result = _exact_mapping(
        value,
        keys=(
            "schema",
            "issue",
            "policy",
            "model_policy",
            "implementation",
            "execution",
            "inputs",
            "dose",
            "mechanics",
            "fit",
            "scores",
            "decision",
            "verdict",
            "admitted",
            "disposable_weights",
            "production_v5",
            "writer_process_id",
            "result_cid",
        ),
        label="result",
    )
    result_issue = _integer_field(result["issue"], label="issue", minimum=1)
    if (
        result["schema"] != RESULT_SCHEMA
        or result_issue != ISSUE
        or result["policy"] != POLICY
        or result["model_policy"] != MODEL_POLICY
    ):
        raise ValueError("cached predictive admission identity differs")
    _integer_field(result["writer_process_id"], label="writer_process_id", minimum=1)
    _validate_historical_implementation(result["implementation"])
    execution = _exact_mapping(
        result["execution"],
        keys=(
            "device",
            "torch_intraop_threads",
            "torch_interop_threads",
            "total_elapsed_seconds",
        ),
        label="execution",
    )
    intraop_threads = _integer_field(
        execution["torch_intraop_threads"],
        label="execution.torch_intraop_threads",
        minimum=1,
    )
    interop_threads = _integer_field(
        execution["torch_interop_threads"],
        label="execution.torch_interop_threads",
        minimum=1,
    )
    total_elapsed = _float_field(
        execution["total_elapsed_seconds"],
        label="execution.total_elapsed_seconds",
        minimum=0.0,
    )
    if (
        execution["device"] != "cpu"
        or intraop_threads > 1_024
        or interop_threads > 1_024
        or total_elapsed > HARD_WALL_SECONDS
    ):
        raise ValueError("cached predictive execution contract differs")

    inputs = _exact_mapping(
        result["inputs"],
        keys=("predecessor", "revealed_v4", "h4_spin_frames"),
        label="inputs",
    )
    predecessor = _exact_mapping(
        inputs["predecessor"],
        keys=("policy", "result_cid", "artifact_cid", "artifact_bytes"),
        label="inputs.predecessor",
    )
    _integer_field(
        predecessor["artifact_bytes"],
        label="inputs.predecessor.artifact_bytes",
        minimum=1,
    )
    if dict(predecessor) != {
        "policy": PREDECESSOR_POLICY,
        "result_cid": PREDECESSOR_RESULT_CID,
        "artifact_cid": PREDECESSOR_ARTIFACT_CID,
        "artifact_bytes": PREDECESSOR_ARTIFACT_BYTES,
    }:
        raise ValueError("cached predecessor binding differs")
    revealed = _exact_mapping(
        inputs["revealed_v4"],
        keys=(
            "population_cid",
            "commitment_cid",
            "reveal_cid",
            "pairs",
            "directions",
            "targets",
        ),
        label="inputs.revealed_v4",
    )
    for field in ("pairs", "directions", "targets"):
        _integer_field(
            revealed[field], label=f"inputs.revealed_v4.{field}", minimum=1
        )
    if dict(revealed) != {
        "population_cid": V4_POPULATION_CID,
        "commitment_cid": V4_COMMITMENT_CID,
        "reveal_cid": V4_REVEAL_CID,
        "pairs": PROBE_PAIRS,
        "directions": PROBE_DIRECTIONS,
        "targets": PROBE_TARGETS,
    }:
        raise ValueError("cached revealed-V4 binding differs")
    frames = _exact_mapping(
        inputs["h4_spin_frames"],
        keys=(
            "artifact_cid",
            "file_cid",
            "root_table_kappa",
            "multiplication_table_kappa",
        ),
        label="inputs.h4_spin_frames",
    )
    artifact_cid = _cid_field(frames["artifact_cid"], label="H4 artifact CID")
    file_cid = _cid_field(frames["file_cid"], label="H4 file CID")
    if (
        artifact_cid != H4_FRAME_ARTIFACT_CID
        or file_cid != H4_FRAME_FILE_CID
        or frames["root_table_kappa"] != ROOT_TABLE_KAPPA
        or frames["multiplication_table_kappa"] != PRODUCT_TABLE_KAPPA
    ):
        raise ValueError("cached H4 registry binding differs")

    dose = _exact_mapping(
        result["dose"],
        keys=(
            "pairs",
            "directions",
            "targets",
            "maximum_updates",
            "completed_updates",
            "cuda",
        ),
        label="dose",
    )
    pairs = _integer_field(dose["pairs"], label="dose.pairs", minimum=1)
    directions = _integer_field(
        dose["directions"], label="dose.directions", minimum=1
    )
    targets = _integer_field(dose["targets"], label="dose.targets", minimum=1)
    maximum_updates = _integer_field(
        dose["maximum_updates"], label="dose.maximum_updates", minimum=1
    )
    completed_updates = _integer_field(
        dose["completed_updates"], label="dose.completed_updates", minimum=1
    )
    if (
        pairs != PROBE_PAIRS
        or directions != PROBE_DIRECTIONS
        or targets != PROBE_TARGETS
        or maximum_updates != MAXIMUM_UPDATES
        or completed_updates > MAXIMUM_UPDATES
        or dose["cuda"] != "FORBIDDEN"
    ):
        raise ValueError("cached predictive dose differs")

    fit = _exact_mapping(
        result["fit"],
        keys=(
            "updates",
            "elapsed_seconds",
            "final_loss",
            "final_gradient_norm",
            "gradient_values_seen",
            "gradient_values_required",
            "all_trainable_values_received_finite_nonzero_gradient",
            "qualified_base_unchanged",
        ),
        label="fit",
    )
    updates = _integer_field(fit["updates"], label="fit.updates", minimum=1)
    gradient_seen = _integer_field(
        fit["gradient_values_seen"], label="fit.gradient_values_seen"
    )
    gradient_required = _integer_field(
        fit["gradient_values_required"], label="fit.gradient_values_required"
    )
    all_gradients = _boolean_field(
        fit["all_trainable_values_received_finite_nonzero_gradient"],
        label="fit.all_gradients",
    )
    base_unchanged = _boolean_field(
        fit["qualified_base_unchanged"], label="fit.qualified_base_unchanged"
    )
    _float_field(fit["elapsed_seconds"], label="fit.elapsed_seconds", minimum=0.0)
    _float_field(fit["final_loss"], label="fit.final_loss", minimum=0.0)
    _float_field(
        fit["final_gradient_norm"], label="fit.final_gradient_norm", minimum=0.0
    )
    if (
        updates != completed_updates
        or updates > MAXIMUM_UPDATES
        or gradient_required != TRAINABLE_PARAMETERS
        or gradient_seen > gradient_required
        or all_gradients != (gradient_seen == gradient_required)
    ):
        raise ValueError("cached predictive fit ledger differs")

    mechanics = _exact_mapping(
        result["mechanics"],
        keys=(
            "all_frame_identity_maximum_delta",
            "all_frame_step_connection_maximum_delta",
            "transported_matrix_read_covariance_maximum_delta",
            "strict_causal_prefix_maximum_logits_delta",
            "unobserved_target_mutation_maximum_prefix_delta",
            "state_off_v1_maximum_logits_delta",
            "artifact_replay_maximum_logits_delta",
            "transport_permutation_head_effect",
            "binding_observable_maximum_head_logits",
            "equal_geometric_plain_intervention_work",
            "forbidden_reads",
            "passed",
            "gradient_values_seen",
            "gradient_values_required",
            "all_trainable_values_received_finite_nonzero_gradient",
            "qualified_base_unchanged",
        ),
        label="mechanics",
    )
    identity_delta = _float_field(
        mechanics["all_frame_identity_maximum_delta"],
        label="mechanics.all_frame_identity_delta",
        minimum=0.0,
    )
    connection_delta = _float_field(
        mechanics["all_frame_step_connection_maximum_delta"],
        label="mechanics.all_frame_step_connection_delta",
        minimum=0.0,
    )
    covariance_delta = _float_field(
        mechanics["transported_matrix_read_covariance_maximum_delta"],
        label="mechanics.transported_matrix_read_covariance_delta",
        minimum=0.0,
    )
    causal_delta = _float_field(
        mechanics["strict_causal_prefix_maximum_logits_delta"],
        label="mechanics.causal_delta",
        minimum=0.0,
    )
    counterfactual_delta = _float_field(
        mechanics["unobserved_target_mutation_maximum_prefix_delta"],
        label="mechanics.counterfactual_delta",
        minimum=0.0,
    )
    state_off_delta = _float_field(
        mechanics["state_off_v1_maximum_logits_delta"],
        label="mechanics.state_off_delta",
        minimum=0.0,
    )
    replay_delta = _float_field(
        mechanics["artifact_replay_maximum_logits_delta"],
        label="mechanics.replay_delta",
        minimum=0.0,
    )
    transport_effect = _float_field(
        mechanics["transport_permutation_head_effect"],
        label="mechanics.transport_effect",
        minimum=0.0,
    )
    observable = _float_field(
        mechanics["binding_observable_maximum_head_logits"],
        label="mechanics.binding_observable",
        minimum=0.0,
    )
    equal_work = _boolean_field(
        mechanics["equal_geometric_plain_intervention_work"],
        label="mechanics.equal_work",
    )
    forbidden_reads = _integer_field(
        mechanics["forbidden_reads"], label="mechanics.forbidden_reads"
    )
    mechanics_passed = _boolean_field(
        mechanics["passed"], label="mechanics.passed"
    )
    mechanics_gradient_seen = _integer_field(
        mechanics["gradient_values_seen"], label="mechanics.gradient_values_seen"
    )
    mechanics_gradient_required = _integer_field(
        mechanics["gradient_values_required"],
        label="mechanics.gradient_values_required",
    )
    mechanics_all_gradients = _boolean_field(
        mechanics["all_trainable_values_received_finite_nonzero_gradient"],
        label="mechanics.all_gradients",
    )
    mechanics_base_unchanged = _boolean_field(
        mechanics["qualified_base_unchanged"],
        label="mechanics.qualified_base_unchanged",
    )
    if (
        mechanics_gradient_seen != gradient_seen
        or mechanics_gradient_required != gradient_required
        or mechanics_all_gradients is not all_gradients
        or mechanics_base_unchanged is not base_unchanged
    ):
        raise ValueError("cached mechanics/fit binding differs")
    reproduced_mechanics = bool(
        identity_delta <= 2e-5
        and connection_delta <= 2e-5
        and covariance_delta <= 2e-5
        and causal_delta <= 2e-5
        and counterfactual_delta <= 2e-5
        and state_off_delta == 0.0
        and replay_delta == 0.0
        and transport_effect > 0.0
        and observable > 0.0
        and equal_work
        and forbidden_reads == 0
        and all_gradients
        and base_unchanged
    )
    if mechanics_passed != reproduced_mechanics:
        raise ValueError("cached mechanics verdict does not reproduce")

    scores = _exact_mapping(
        result["scores"],
        keys=("full_delta", "additive_no_overwrite", "state_off"),
        label="scores",
    )
    native = _probe_score_from_record(
        scores["full_delta"], expected_intervention="native", label="scores.full_delta"
    )
    additive = _probe_score_from_record(
        scores["additive_no_overwrite"],
        expected_intervention="no_delta",
        label="scores.additive_no_overwrite",
    )
    state_off = _probe_score_from_record(
        scores["state_off"],
        expected_intervention="state_off",
        label="scores.state_off",
    )
    expected_decision = admission_decision(
        native=native,
        additive=additive,
        state_off=state_off,
        mechanics=mechanics,
    )
    if result["decision"] != expected_decision:
        raise ValueError("cached admission decision does not reproduce")
    admitted = _boolean_field(result["admitted"], label="admitted")
    if result["verdict"] != expected_decision["verdict"] or admitted != expected_decision[
        "admitted"
    ]:
        raise ValueError("cached top-level admission verdict differs")

    disposal = _exact_mapping(
        result["disposable_weights"],
        keys=("status", "values"),
        label="disposable_weights",
    )
    destroyed_values = _integer_field(
        disposal["values"], label="disposable_weights.values", minimum=1
    )
    if (
        disposal["status"] != "DESTROYED_IN_MEMORY_NO_ARTIFACT"
        or destroyed_values != TRAINABLE_PARAMETERS
    ):
        raise ValueError("cached disposable-weight destruction differs")
    production = _exact_mapping(
        result["production_v5"],
        keys=("authorized", "created", "inspected", "selector"),
        label="production_v5",
    )
    production_authorized = _boolean_field(
        production["authorized"], label="production_v5.authorized"
    )
    production_created = _boolean_field(
        production["created"], label="production_v5.created"
    )
    production_inspected = _boolean_field(
        production["inspected"], label="production_v5.inspected"
    )
    if (
        production_authorized is not admitted
        or production_created is not False
        or production_inspected is not False
        or production["selector"] != "NOT_IMPLEMENTED_IN_PREFLIGHT_MODULE"
    ):
        raise ValueError("cached V5 authorization boundary differs")


def destroy_disposable_weights(model: _ProbeModel) -> int:
    """Zero every fitted value before the disposable model leaves scope."""

    destroyed = 0
    with torch.no_grad():
        for parameter in model.trainable_parameters():
            destroyed += parameter.numel()
            parameter.zero_()
    if destroyed != TRAINABLE_PARAMETERS:
        raise RuntimeError("disposable trainable-value count differs from the freeze")
    return destroyed


def _paired_loss(model: _ProbeModel, directions: Sequence[Any], device: torch.device) -> Tensor:
    inputs, targets = _batch(directions, device=device)
    output = _output(model(inputs, intervention="native"), rows=len(inputs))
    own, foreign = _suffix_log_probabilities(output, targets)
    own_nll = -own.mean()
    contrast = F.softplus(-(own - foreign)).mean()
    return own_nll + contrast


def fit_disposable_probe(
    model: _ProbeModel,
    pairs: Sequence[PromptConditioningPair],
    *,
    device: torch.device,
    maximum_updates: int = MAXIMUM_UPDATES,
    hard_wall_seconds: float = HARD_WALL_SECONDS,
) -> dict[str, Any]:
    """Fit only the 9,228-value head on revealed data within the hard bound."""

    if not 1 <= maximum_updates <= MAXIMUM_UPDATES:
        raise ValueError("disposable updates exceed the frozen 256-update ceiling")
    parameters = tuple(model.trainable_parameters())
    frozen = tuple(model.frozen_base_parameters())
    if (
        sum(parameter.numel() for parameter in parameters) != TRAINABLE_PARAMETERS
        or any(parameter.requires_grad for parameter in frozen)
        or not all(parameter.requires_grad for parameter in parameters)
    ):
        raise ValueError("predictive trainable/frozen parameter boundary differs")
    base_before = model.export_qualified_base_artifact()
    directions = _directions(pairs)
    optimizer = torch.optim.AdamW(
        parameters,
        lr=LEARNING_RATE,
        betas=ADAM_BETAS,
        eps=ADAM_EPSILON,
        weight_decay=0.0,
    )
    coverage = [torch.zeros_like(parameter, dtype=torch.bool) for parameter in parameters]
    started = time.monotonic()
    final_loss = math.nan
    final_gradient_norm = math.nan
    model.train()
    for update in range(maximum_updates):
        if time.monotonic() - started > hard_wall_seconds:
            raise TimeoutError("disposable preflight exceeded its five-minute hard wall")
        offset = (update * BATCH_DIRECTIONS) % len(directions)
        batch = tuple(
            directions[(offset + index) % len(directions)]
            for index in range(BATCH_DIRECTIONS)
        )
        optimizer.zero_grad(set_to_none=True)
        loss = _paired_loss(model, batch, device)
        if not torch.isfinite(loss).item():
            raise RuntimeError("disposable paired loss is nonfinite")
        loss.backward()
        for observed, parameter in zip(coverage, parameters, strict=True):
            if parameter.grad is not None:
                observed.logical_or_(parameter.grad.detach().isfinite() & parameter.grad.detach().ne(0))
        norm = torch.nn.utils.clip_grad_norm_(parameters, GRADIENT_CLIP)
        if not torch.isfinite(norm).item():
            raise RuntimeError("disposable gradient norm is nonfinite")
        optimizer.step()
        final_loss = float(loss.detach().cpu())
        final_gradient_norm = float(norm.detach().cpu())
    elapsed = time.monotonic() - started
    gradient_values_seen = sum(int(values.sum().item()) for values in coverage)
    base_unchanged = model.export_qualified_base_artifact() == base_before
    return {
        "updates": maximum_updates,
        "elapsed_seconds": elapsed,
        "final_loss": final_loss,
        "final_gradient_norm": final_gradient_norm,
        "gradient_values_seen": gradient_values_seen,
        "gradient_values_required": TRAINABLE_PARAMETERS,
        "all_trainable_values_received_finite_nonzero_gradient": (
            gradient_values_seen == TRAINABLE_PARAMETERS
        ),
        "qualified_base_unchanged": base_unchanged,
    }


def transport_mechanics(model: _ProbeModel, *, device: torch.device) -> dict[str, float]:
    """Check the registered connection and R4 matrix/read covariance on all frames."""

    frames = getattr(model, "frame_matrices", None)
    products = getattr(model, "frame_multiplication", None)
    if (
        not isinstance(frames, Tensor)
        or not isinstance(products, Tensor)
        or tuple(frames.shape) != (120, 4, 4)
        or tuple(products.shape) != (120, 120)
        or frames.device != device
        or products.device != device
        or products.dtype != torch.long
        or not torch.isfinite(frames).all().item()
    ):
        raise ValueError("predictive model does not expose the registered H4 frame table")

    identity = torch.eye(4, device=device, dtype=frames.dtype)
    all_frame_identity = torch.matmul(frames.transpose(-1, -2), frames)
    identity_delta = float(
        (all_frame_identity - identity.unsqueeze(0)).abs().max().cpu()
    )

    source_indices = torch.arange(120, device=device).repeat_interleave(120)
    leaves = torch.arange(120, device=device).repeat(120)
    destination_indices = products[source_indices, leaves]
    relative = torch.matmul(
        frames.index_select(0, destination_indices).transpose(-1, -2),
        frames.index_select(0, source_indices),
    )
    step = model._step_transport(leaves, intervention="native")
    if tuple(step.shape) != (120 * 120, 4, 4) or not torch.isfinite(step).all().item():
        raise ValueError("predictive native step transport contract differs")
    connection_delta = float((relative - step).abs().max().cpu())

    transports = model._step_transport(
        torch.arange(120, device=device), intervention="native"
    )
    matrix = (
        torch.arange(1, 17, device=device, dtype=frames.dtype).view(4, 4) / 17.0
    )
    read = torch.tensor(
        [0.25, -0.5, 0.75, -1.0], device=device, dtype=frames.dtype
    )
    transported_matrix = torch.matmul(
        torch.matmul(transports, matrix), transports.transpose(-1, -2)
    )
    transported_read = torch.matmul(transports, read.view(4, 1))
    observed = torch.matmul(transported_matrix, transported_read)
    expected = torch.matmul(
        transports, torch.matmul(matrix, read.view(4, 1))
    )
    covariance_delta = float((observed - expected).abs().max().cpu())
    return {
        "all_frame_identity_maximum_delta": identity_delta,
        "all_frame_step_connection_maximum_delta": connection_delta,
        "transported_matrix_read_covariance_maximum_delta": covariance_delta,
    }


def basic_mechanics(
    native: _ProbeModel,
    plain: _ProbeModel,
    pairs: Sequence[PromptConditioningPair],
    *,
    replay_factory: Callable[[], _ProbeModel],
    device: torch.device,
) -> dict[str, Any]:
    """Exercise causality, replay, state-off, controls, and observability."""

    directions = _directions(pairs)
    inputs, _targets = _batch(directions[:2], device=device)
    native.eval()
    plain.eval()
    transport_checks = transport_mechanics(native, device=device)
    artifact = native.export_binding_artifact()
    plain.load_binding_artifact(artifact)
    replay = replay_factory()
    replay.load_binding_artifact(artifact)
    replay.eval()
    with torch.no_grad():
        output = _output(native(inputs, intervention="native"), rows=len(inputs))
        state_off = _output(native(inputs, intervention="state_off"), rows=len(inputs))
        deranged = _output(
            native(inputs, intervention="transport_permuted"), rows=len(inputs)
        )
        plain_output = _output(plain(inputs, intervention="native"), rows=len(inputs))
        replay_output = _output(replay(inputs, intervention="native"), rows=len(inputs))
        prefix = inputs[:, : PROMPT_TOKENS]
        prefix_output = native(prefix, intervention="native")
        mutated = inputs.clone()
        mutated[:, PROMPT_TOKENS + 2 :] = torch.flip(
            mutated[:, PROMPT_TOKENS + 2 :], dims=(1,)
        )
        mutated_output = _output(
            native(mutated, intervention="native"), rows=len(inputs)
        )
    causal_delta = float(
        (output.logits[:, : prefix.shape[1]] - prefix_output.logits).abs().max().cpu()
    )
    counterfactual_delta = float(
        (
            output.logits[:, : PROMPT_TOKENS + 2]
            - mutated_output.logits[:, : PROMPT_TOKENS + 2]
        )
        .abs()
        .max()
        .cpu()
    )
    state_off_delta = float((state_off.logits - state_off.base_logits).abs().max().cpu())
    replay_delta = float((output.logits - replay_output.logits).abs().max().cpu())
    deranged_effect = float((output.head_logits - deranged.head_logits).abs().max().cpu())
    observable = float(output.head_logits.abs().max().cpu())
    equal_work = (
        output.audit.work_signature()
        == state_off.audit.work_signature()
        == deranged.audit.work_signature()
        == plain_output.audit.work_signature()
    )
    mechanics = {
        **transport_checks,
        "strict_causal_prefix_maximum_logits_delta": causal_delta,
        "unobserved_target_mutation_maximum_prefix_delta": counterfactual_delta,
        "state_off_v1_maximum_logits_delta": state_off_delta,
        "artifact_replay_maximum_logits_delta": replay_delta,
        "transport_permutation_head_effect": deranged_effect,
        "binding_observable_maximum_head_logits": observable,
        "equal_geometric_plain_intervention_work": equal_work,
        "forbidden_reads": sum(
            int(value.audit.forbidden_reads)
            for value in (output, state_off, deranged, plain_output, replay_output)
        ),
    }
    mechanics["passed"] = bool(
        transport_checks["all_frame_identity_maximum_delta"] <= 2e-5
        and transport_checks["all_frame_step_connection_maximum_delta"] <= 2e-5
        and transport_checks["transported_matrix_read_covariance_maximum_delta"] <= 2e-5
        and causal_delta <= 2e-5
        and counterfactual_delta <= 2e-5
        and state_off_delta == 0.0
        and replay_delta == 0.0
        and deranged_effect > 0.0
        and observable > 0.0
        and equal_work
        and mechanics["forbidden_reads"] == 0
    )
    return mechanics


def _default_model_factory(inputs: FrozenProbeInputs, arm: str, device: torch.device) -> _ProbeModel:
    from .predictive_block_delta_binding import R4PredictiveBlockDeltaBindingV1

    geometry = _exact_geometry(inputs.predecessor)
    model = R4PredictiveBlockDeltaBindingV1(
        geometry, inputs.frames, arm=arm
    ).to(device)
    model.load_qualified_base_artifact(inputs.predecessor_artifact_path.read_bytes())
    return model


def run_predictive_block_delta_preflight(
    *,
    root: Path,
    predecessor_root: Path,
    revealed_v4_root: Path,
    frame_sidecar_path: Path,
    device: torch.device | str = "cpu",
    maximum_updates: int = MAXIMUM_UPDATES,
    model_factory: Callable[[FrozenProbeInputs, str, torch.device], _ProbeModel] = _default_model_factory,
) -> dict[str, Any]:
    """Run the sole disposable gate and write a create-once binary result."""

    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists() or result_path.is_symlink():
        result = _read_canonical_json(result_path)
        _validate_cached_result(result)
        return result
    frozen = load_frozen_probe_inputs(
        predecessor_root=predecessor_root,
        revealed_v4_root=revealed_v4_root,
        frame_sidecar_path=frame_sidecar_path,
    )
    selected_device = torch.device(device)
    if selected_device.type != "cpu":
        raise ValueError("predictive block-delta preflight is frozen to CPU execution")
    torch.manual_seed(INITIALIZATION_SEED)
    gate_started = time.monotonic()

    def remaining_gate_seconds() -> float:
        remaining = HARD_WALL_SECONDS - (time.monotonic() - gate_started)
        if remaining <= 0.0:
            raise TimeoutError(
                "disposable predictive gate exceeded its five-minute hard wall"
            )
        return remaining

    native = model_factory(frozen, "geometric", selected_device)
    plain = model_factory(frozen, "plain", selected_device)
    remaining_gate_seconds()
    mechanics = basic_mechanics(
        native,
        plain,
        frozen.pairs,
        replay_factory=lambda: model_factory(frozen, "geometric", selected_device),
        device=selected_device,
    )
    remaining = remaining_gate_seconds()
    fit = fit_disposable_probe(
        native,
        frozen.pairs,
        device=selected_device,
        maximum_updates=maximum_updates,
        hard_wall_seconds=remaining,
    )
    mechanics = {
        **mechanics,
        "gradient_values_seen": fit["gradient_values_seen"],
        "gradient_values_required": fit["gradient_values_required"],
        "all_trainable_values_received_finite_nonzero_gradient": fit[
            "all_trainable_values_received_finite_nonzero_gradient"
        ],
        "qualified_base_unchanged": fit["qualified_base_unchanged"],
    }
    mechanics["passed"] = bool(
        mechanics["passed"]
        and mechanics["all_trainable_values_received_finite_nonzero_gradient"]
        and mechanics["qualified_base_unchanged"]
    )
    native_score = score_probe(
        native, frozen.pairs, intervention="native", device=selected_device
    )
    remaining_gate_seconds()
    additive_score = score_probe(
        native, frozen.pairs, intervention="no_delta", device=selected_device
    )
    remaining_gate_seconds()
    state_off_score = score_probe(
        native, frozen.pairs, intervention="state_off", device=selected_device
    )
    remaining_gate_seconds()
    decision = admission_decision(
        native=native_score,
        additive=additive_score,
        state_off=state_off_score,
        mechanics=mechanics,
    )
    destroyed = destroy_disposable_weights(native)
    destroy_disposable_weights(plain)
    total_elapsed_seconds = time.monotonic() - gate_started
    if total_elapsed_seconds > HARD_WALL_SECONDS:
        raise TimeoutError("disposable predictive gate exceeded its five-minute hard wall")
    implementation = trainer_implementation_contract()
    result = _with_self_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "implementation": implementation,
            "execution": {
                "device": str(selected_device),
                "torch_intraop_threads": torch.get_num_threads(),
                "torch_interop_threads": torch.get_num_interop_threads(),
                "total_elapsed_seconds": total_elapsed_seconds,
            },
            "inputs": dict(frozen.records),
            "dose": {
                "pairs": PROBE_PAIRS,
                "directions": PROBE_DIRECTIONS,
                "targets": PROBE_TARGETS,
                "maximum_updates": MAXIMUM_UPDATES,
                "completed_updates": fit["updates"],
                "cuda": "FORBIDDEN",
            },
            "mechanics": mechanics,
            "fit": fit,
            "scores": {
                "full_delta": native_score.record(),
                "additive_no_overwrite": additive_score.record(),
                "state_off": state_off_score.record(),
            },
            "decision": decision,
            "verdict": decision["verdict"],
            "admitted": decision["admitted"],
            "disposable_weights": {
                "status": "DESTROYED_IN_MEMORY_NO_ARTIFACT",
                "values": destroyed,
            },
            "production_v5": {
                "authorized": decision["admitted"],
                "created": False,
                "inspected": False,
                "selector": "NOT_IMPLEMENTED_IN_PREFLIGHT_MODULE",
            },
            "writer_process_id": os.getpid(),
        },
        "result_cid",
    )
    _validate_cached_result(result)
    result_path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(result_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(canonical_json_bytes(result))
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return result
