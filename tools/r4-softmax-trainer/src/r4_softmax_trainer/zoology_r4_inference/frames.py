"""Read the native #1059 frame export without redefining its token mapping."""

from __future__ import annotations

import copy
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import torch
from blake3 import blake3
from torch import Tensor

from ..h4_spin_frame_sidecar import H4SpinFrameArtifactV1

SCHEMA = "uor-r4.zoology-r4-token-frames/1"
VOCABULARY_SIZE = 8192
NATIVE_POLICY = (
    "schema=helm-d-r4-gauge-softmax/1\n"
    "scope=offline-full-prefix-causal-softmax-oracle\n"
    "head-layout=complete-consecutive-R4-blocks\n"
    "frame=exact-cumulative-UOR-Spin-H4-left-quaternion\n"
    "encode=F_position_transpose-times-model-vector\n"
    "transport=P_source_to_query=F_query_transpose-times-F_source\n"
    "transported-state=every-causal-key-and-value\n"
    "score=unchanged-scaled-dot-product-in-query-gauge\n"
    "selector=unchanged-stable-causal-softmax\n"
    "aggregate=unchanged-weighted-value-sum-in-query-gauge\n"
    "decode=F_query-times-query-gauge-output-before-Wo\n"
    "control=source-frame-permuted-with-identical-shape-and-work\n"
    "expected=ordinary-attention-numerical-and-behavioral-parity\n"
    "not-claimed=geometry-advantage,intrinsic-distance,transformerless-serving,"
    "softmax-removal,source-free-language-model"
)
_CID = re.compile(r"blake3:[0-9a-f]{64}\Z")
_FIELDS = (
    "schema",
    "policy_identity",
    "maximum_token_id",
    "identity_index",
    "frame_artifact_cid",
    "frame_file_cid",
    "token_leaf_indices",
    "prefix_witnesses",
    "direct_leaf_count",
    "witness_frame_count",
    "artifact_cid",
)


def _cid(payload: bytes) -> str:
    return f"blake3:{blake3(payload).hexdigest()}"


def _canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, allow_nan=False, separators=(",", ":")
    ).encode()


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate token-frame field {key!r}")
        value[key] = item
    return value


def _integer(value: Any, lower: int, upper: int) -> int:
    if (
        isinstance(value, bool)
        or not isinstance(value, int)
        or not lower <= value <= upper
    ):
        raise ValueError("token-frame integer is outside its declared range")
    return value


def _read_regular(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"frame input must be a regular non-symlink file: {path}")
    return path.read_bytes()


@dataclass(frozen=True, slots=True)
class PrefixWitness:
    tokens: tuple[int, ...]
    frame_indices: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class R4InferenceFrames:
    """Canonical f64 matrix actions and native token leaves, all on CPU.

    The matrices act only on the root/table-index coordinate. This object does
    not export fiber/torsion fields or assert that all 120 frames are reachable.
    """

    frame_matrices: Tensor
    multiplication_indices: Tensor
    token_leaf_indices: Tensor
    identity_index: int
    artifact_cid: str
    file_cid: str
    frame_artifact_cid: str
    frame_file_cid: str
    prefix_witnesses: tuple[PrefixWitness, ...]
    direct_leaf_count: int
    witness_frame_count: int
    policy_identity: str = NATIVE_POLICY

    def cumulative_frame_indices(self, inputs: Tensor) -> Tensor:
        if (
            inputs.device.type != "cpu"
            or inputs.dtype != torch.long
            or inputs.ndim != 2
            or inputs.shape[0] < 1
            or inputs.shape[1] < 1
            or bool((inputs < 0).any())
            or bool((inputs >= self.token_leaf_indices.numel()).any())
        ):
            raise ValueError(
                "frame input must be nonempty CPU int64 tokens within the exported map"
            )
        leaves = self.token_leaf_indices[inputs]
        current = torch.full((inputs.shape[0],), self.identity_index, dtype=torch.long)
        steps: list[Tensor] = []
        for position in range(inputs.shape[1]):
            current = self.multiplication_indices[current, leaves[:, position]]
            steps.append(current)
        return torch.stack(steps, dim=1)


def load_frames(directory: Path | str) -> R4InferenceFrames:
    """Validate both native files and reproduce their nonlabelled prefix witnesses."""

    directory = Path(directory)
    raw_frames = _read_regular(directory / "h4-frames.json")
    native = H4SpinFrameArtifactV1.from_bytes(raw_frames)
    raw_map = _read_regular(directory / "token-frames.json")
    try:
        value = json.loads(raw_map, object_pairs_hook=_unique_object)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid token-frame JSON") from error
    if (
        not isinstance(value, dict)
        or tuple(value) != _FIELDS
        or _canonical(value) != raw_map
    ):
        raise ValueError("token-frame schema or canonical byte order differs")
    if value["schema"] != SCHEMA or value["policy_identity"] != NATIVE_POLICY:
        raise ValueError("token frames do not name the native HELM gauge policy")
    if (
        _integer(value["maximum_token_id"], 0, VOCABULARY_SIZE - 1)
        != VOCABULARY_SIZE - 1
    ):
        raise ValueError("token-frame export must cover all 8192 source tokens")
    identity = _integer(value["identity_index"], 0, 119)
    if identity != native.identity_index:
        raise ValueError("token map and matrix sidecar have different identities")
    for field in ("artifact_cid", "frame_artifact_cid", "frame_file_cid"):
        if not isinstance(value[field], str) or not _CID.fullmatch(value[field]):
            raise ValueError(f"malformed {field}")
    if (
        value["frame_artifact_cid"] != native.artifact_cid
        or value["frame_file_cid"] != native.file_cid
    ):
        raise ValueError("token-frame map is bound to another H4 matrix sidecar")
    seed = copy.deepcopy(value)
    seed["artifact_cid"] = ""
    if _cid(_canonical(seed)) != value["artifact_cid"]:
        raise ValueError("token-frame artifact CID does not reproduce")
    leaf_values = value["token_leaf_indices"]
    if not isinstance(leaf_values, list) or len(leaf_values) != VOCABULARY_SIZE:
        raise ValueError("token-frame map must contain exactly 8192 leaves")
    leaves = tuple(_integer(item, 0, 119) for item in leaf_values)
    if leaves[0] != identity:
        raise ValueError("native token zero must map to identity")
    witnesses = value["prefix_witnesses"]
    if not isinstance(witnesses, list) or len(witnesses) != 3:
        raise ValueError("native export must contain three fixed prefix witnesses")
    prefixes: list[PrefixWitness] = []
    for witness in witnesses:
        if not isinstance(witness, dict) or tuple(witness) != (
            "tokens",
            "frame_indices",
        ):
            raise ValueError("native prefix witness schema differs")
        tokens, indices = witness["tokens"], witness["frame_indices"]
        if (
            not isinstance(tokens, list)
            or not isinstance(indices, list)
            or not 1 <= len(tokens) <= 8
            or len(tokens) != len(indices)
        ):
            raise ValueError("native prefix witness shape differs")
        prefixes.append(
            PrefixWitness(
                tuple(_integer(item, 0, VOCABULARY_SIZE - 1) for item in tokens),
                tuple(_integer(item, 0, 119) for item in indices),
            )
        )
    if not any(VOCABULARY_SIZE - 1 in prefix.tokens for prefix in prefixes):
        raise ValueError("native witnesses do not cover the maximum token ID")
    direct_count = _integer(value["direct_leaf_count"], 1, 120)
    witness_count = _integer(value["witness_frame_count"], 1, 120)
    if direct_count != len(set(leaves)) or witness_count != len(
        {index for prefix in prefixes for index in prefix.frame_indices}
    ):
        raise ValueError("native frame coverage accounting differs")
    # Reuse the strict loader's validation; retain its exact native f64 bits
    # for gauge arithmetic instead of upcasting already-rounded f32 matrices.
    matrix_bits = np.asarray(
        json.loads(raw_frames)["frame_matrix_f64_bits"], dtype=np.uint64
    )
    frames = R4InferenceFrames(
        frame_matrices=torch.from_numpy(matrix_bits.view(np.float64).copy()),
        multiplication_indices=native.multiplication_indices,
        token_leaf_indices=torch.tensor(leaves, dtype=torch.long),
        identity_index=identity,
        artifact_cid=value["artifact_cid"],
        file_cid=_cid(raw_map),
        frame_artifact_cid=native.artifact_cid,
        frame_file_cid=native.file_cid,
        prefix_witnesses=tuple(prefixes),
        direct_leaf_count=direct_count,
        witness_frame_count=witness_count,
    )
    for prefix in frames.prefix_witnesses:
        actual = frames.cumulative_frame_indices(
            torch.tensor([prefix.tokens], dtype=torch.long)
        )
        if actual[0].tolist() != list(prefix.frame_indices):
            raise ValueError(
                "Python cumulative frame fold differs from native prefix witness"
            )
    return frames
