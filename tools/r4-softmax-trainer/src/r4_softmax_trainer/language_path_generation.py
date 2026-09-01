"""Create-once autonomous generation smoke for the retained #973 language path.

The fitted retained artifact is immutable input.  This module performs no
training, checkpoint selection, held-out access, ordinary-arm comparison, or
state-off intervention.  It executes the direct retained recurrence on five
public prompts, then repeats every prompt from a fresh artifact load and zero
state.
"""

from __future__ import annotations

import importlib.metadata
import json
import math
import os
import platform
import struct
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol

import numpy as np
import torch
from blake3 import blake3
from torch import Tensor
from tokenizers import Tokenizer

from .group_retention import GroupAddressArtifact
from .group_retention_campaign import load_group_geometry_artifacts
from .language_path_generalization import (
    CONTEXT,
    HEADS,
    HEAD_DIM,
    LAYERS,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)
from .language_path_generalization_data import (
    DATA_MANIFEST_NAME,
    DATA_MANIFEST_SCHEMA,
    EXPECTED_GEOMETRY_ARTIFACT_CID,
    EXPECTED_GEOMETRY_FILE_CID,
    EXPECTED_TOKENIZER_CID,
    GEOMETRY_RELATIVE_PATH,
    TOKENIZER_RELATIVE_PATH,
)
from .provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    tree_cid,
    verify_artifact_subset,
    verify_manifest_envelope,
)


ISSUE = 973
POLICY = "R4RetainedLanguagePathGenerationSmokeV1"
RESULT_SCHEMA = "uor-r4.retained-language-path-generation-smoke/1"
ROLLOUT_AUDIT_SCHEMA = "uor-r4.retained-language-path-generation-audit/1"
PROMPT_SCHEMA = "uor-r4.retained-language-path-generation-prompts/1"

FREEZE_COMMENT_ID = 5_492_076_542
FREEZE_CID = "blake3:b0d9d515baf546514f6e1de126a78e4c6c485d53a4a61c5881df68e94df51e1f"
EXPECTED_PREPARATION_MANIFEST_CID = (
    "blake3:daef3fc9c7f6ccb3e6c4803140adba547c6a5dfa25abc1d974c4705306e2c207"
)
EXPECTED_PARENT_RESULT_CID = (
    "blake3:cf23d03a8809bb713774704630ef7e90129dd6224856ffd0cd4515554ed5eb95"
)
EXPECTED_PARENT_RUN_CONTRACT_CID = (
    "blake3:7a9c39514b74e6e105de64dd162143baffce055af8581fb8ae7f3fd03e7e8272"
)
EXPECTED_RETAINED_ARM_RESULT_CID = (
    "blake3:45fe48555ea5f18bfe2e9acc7ba53569101c599fc806513ea5ddc17934565d91"
)
EXPECTED_RETAINED_ARTIFACT_CID = (
    "blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d"
)
EXPECTED_RETAINED_ARTIFACT_BYTES = 1_010_792

PARENT_RESULT_RELATIVE_PATH = "run/language-path-result.json"
RETAINED_ARTIFACT_RELATIVE_PATH = "arms/retained/model.safetensors"
RESULT_RELATIVE_PATH = "generation/retained-language-path-generation-smoke.json"
GENERATOR_RELATIVE_PATH = "src/r4_softmax_trainer/language_path_generation.py"
EXECUTED_RUNNER_ARCHIVE_RELATIVE_PATH = (
    "docs/r4_retained_language_path_generation_runner_973_raw.py"
)

EXECUTED_RUNNER_RECORD = {
    "path": GENERATOR_RELATIVE_PATH,
    "bytes": 51_865,
    "cid": "blake3:522862195c01c83ee59dcd849db7824b3345c20bcbce170ce86e89a23b57cf64",
}
EXECUTED_IMPLEMENTATION_TREE_CID = (
    "blake3:36ec05919142e671b37ea5a5493bd4577a2096b4338d934673a3e5dc278f6353"
)

BOS_TOKEN_ID = 0
EOS_TOKEN_ID = 1
UNK_TOKEN_ID = 2
SEED = 9_738
MAX_NEW_TOKENS = 64
TOP_K = 40
TEMPERATURE = 0.8
THREADS = 4
SAMPLER_POLICY = (
    "r4-local-top-k-q32-splitmix64/1;temperature=0.8;top-k=40;"
    "rank=logit-desc-token-asc"
)
LOGITS_TRACE_ENCODING = "BLAKE3(u32-be row-width || contiguous f32-le row) per executed input"

EXPECTED_MODEL_DEPENDENCIES = {
    "pyproject.toml": (
        631,
        "blake3:31b00191b7832af2a4996f9ff054860cce7401226c986784351e22f1469fc52b",
    ),
    "src/r4_softmax_trainer/constants.py": (
        6_241,
        "blake3:2202fbb7da8640843680fe2836fa906606a712afe237f280b3eb672c534d1182",
    ),
    "src/r4_softmax_trainer/group_retention.py": (
        35_256,
        "blake3:db652c4ac0dec8b3583397285cb31bbd6c2af93a528cd6d1e4a04a2019a55852",
    ),
    "src/r4_softmax_trainer/group_retention_campaign.py": (
        62_177,
        "blake3:ded054865aea1fc310de1b820dd013ab7c7841db298ea9921fa9fa6e98b5f1bb",
    ),
    "src/r4_softmax_trainer/group_retention_decoder.py": (
        37_488,
        "blake3:8f2ed6d6c878f380c3e2461a791d6ca46cdf3592dd6e95362d449f2d057d92a7",
    ),
    "src/r4_softmax_trainer/language_path_generalization.py": (
        16_831,
        "blake3:7e9d39a04d6db1ebca04e448ad1bd15209d28d50ec7b2ff52f37dcd13aae3d9e",
    ),
    "src/r4_softmax_trainer/language_path_generalization_data.py": (
        22_109,
        "blake3:8a9493fcc28f8ec5020466d582e29b2eb16b4233215926d58f914114aaf6e3dd",
    ),
    "src/r4_softmax_trainer/model.py": (
        9_296,
        "blake3:c0d977f077dc50548849374f4ae371fb0c0593528a1695ebbf9c9baef73fdbb2",
    ),
    "src/r4_softmax_trainer/provenance.py": (
        5_496,
        "blake3:628d7035da6cb972f5c733b8c1474fcfa8c32170e7b7f5ce90484f26f04cb8ae",
    ),
    "uv.lock": (
        65_263,
        "blake3:f0c56308502b96980f050e392df5280b65034ac4bebdd5af271824fa21c196a3",
    ),
}


@dataclass(frozen=True, slots=True)
class GenerationPrompt:
    index: int
    text: str
    token_ids: tuple[int, ...]

    def record(self) -> dict[str, Any]:
        return {
            "index": self.index,
            "text": self.text,
            "token_ids": list(self.token_ids),
        }


PROMPTS = (
    GenerationPrompt(
        0,
        "A purple turtle found a clock in the garden",
        (35, 1765, 2104, 505, 261, 2692, 315, 265, 914),
    ),
    GenerationPrompt(
        1,
        "Mia promised the small robot she would return",
        (1428, 1763, 265, 563, 2214, 394, 527, 2510),
    ),
    GenerationPrompt(
        2,
        "The fox looked at the moon and heard a bell",
        (413, 1710, 506, 452, 265, 2143, 269, 827, 261, 2046),
    ),
    GenerationPrompt(
        3,
        "Ben opened the wooden box, but it was empty",
        (1101, 905, 265, 1334, 303, 616, 14, 412, 311, 285, 2309),
    ),
    GenerationPrompt(
        4,
        "When the rain stopped, the tiny dragon smiled",
        (1039, 265, 998, 1029, 14, 265, 1485, 1958, 565),
    ),
)


def prompt_contract() -> dict[str, Any]:
    return {
        "schema": PROMPT_SCHEMA,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "bos_token_id": BOS_TOKEN_ID,
        "eos_token_id": EOS_TOKEN_ID,
        "prompts": [prompt.record() for prompt in PROMPTS],
    }


EXPECTED_PROMPT_CONTRACT_CID = (
    "blake3:1262645de4102c040944ecde61f737be5e54bee40815b0ab9215ba2cf95b794b"
)
if cid_bytes(canonical_json_bytes(prompt_contract())) != EXPECTED_PROMPT_CONTRACT_CID:
    raise RuntimeError("public generation prompt contract CID drifted")


class _Tokenizer(Protocol):
    def encode(self, sequence: str, add_special_tokens: bool = True) -> Any: ...

    def decode(
        self, ids: list[int], skip_special_tokens: bool = True
    ) -> str: ...


class _RawDecoder(Protocol):
    def decode_bytes(self, ids: Sequence[int]) -> bytes: ...


@dataclass(frozen=True, slots=True)
class ByteLevelRawDecoder:
    """Exact pre-lossy byte decoder for the copied ByteLevel BPE vocabulary."""

    token_bytes: tuple[bytes, ...]

    @classmethod
    def from_tokenizer_json(cls, path: Path) -> ByteLevelRawDecoder:
        value = _strict_json_object(path)
        model = value.get("model")
        decoder = value.get("decoder")
        if (
            not isinstance(model, Mapping)
            or model.get("type") != "BPE"
            or not isinstance(model.get("vocab"), Mapping)
            or not isinstance(decoder, Mapping)
            or decoder.get("type") != "ByteLevel"
        ):
            raise ValueError("generation tokenizer is not the frozen ByteLevel BPE shape")
        vocab = model["vocab"]
        by_id: list[str | None] = [None] * len(vocab)
        for content, token_id in vocab.items():
            if (
                not isinstance(content, str)
                or isinstance(token_id, bool)
                or not isinstance(token_id, int)
                or not 0 <= token_id < len(by_id)
                or by_id[token_id] is not None
            ):
                raise ValueError("generation tokenizer vocabulary is not a dense ID prefix")
            by_id[token_id] = content
        if any(content is None for content in by_id):
            raise ValueError("generation tokenizer vocabulary contains an unassigned ID")

        direct_bytes = (
            *range(ord("!"), ord("~") + 1),
            *range(0xA1, 0xAC + 1),
            *range(0xAE, 0xFF + 1),
        )
        assigned = set(direct_bytes)
        byte_decoder = {chr(byte): byte for byte in direct_bytes}
        extra = 0
        for byte in range(256):
            if byte in assigned:
                continue
            byte_decoder[chr(256 + extra)] = byte
            extra += 1
        if extra != 68:
            raise RuntimeError("GPT-2 byte alphabet construction differs")

        decoded: list[bytes] = []
        for content in by_id:
            assert content is not None
            token = bytearray()
            for character in content:
                byte = byte_decoder.get(character)
                if byte is None:
                    token.extend(character.encode("utf-8"))
                else:
                    token.append(byte)
            decoded.append(bytes(token))

        added = value.get("added_tokens", [])
        if not isinstance(added, list):
            raise ValueError("generation tokenizer added_tokens must be a list")
        added_ids: set[int] = set()
        for entry in added:
            if not isinstance(entry, Mapping):
                raise ValueError("generation tokenizer contains an invalid added token")
            token_id = entry.get("id")
            content = entry.get("content")
            if (
                isinstance(token_id, bool)
                or not isinstance(token_id, int)
                or not 0 <= token_id < len(decoded)
                or not isinstance(content, str)
                or not content
            ):
                raise ValueError("generation tokenizer added token lies outside the vocabulary")
            if by_id[token_id] != content:
                raise ValueError("generation tokenizer added token conflicts with model vocab")
            added_ids.add(token_id)
        for token_id in added_ids:
            content = by_id[token_id]
            assert content is not None
            decoded[token_id] = content.encode("utf-8")
        return cls(tuple(decoded))

    def decode_bytes(self, ids: Sequence[int]) -> bytes:
        decoded = bytearray()
        for token_id in ids:
            if isinstance(token_id, bool) or not isinstance(token_id, int):
                raise TypeError("raw decoder token IDs must be integers")
            if 0 <= token_id < len(self.token_bytes):
                decoded.extend(self.token_bytes[token_id])
        return bytes(decoded)


class SplitMix64:
    """Exact wrapping SplitMix64 stream used by the repository Rust sampler."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        if isinstance(seed, bool) or not isinstance(seed, int) or not 0 <= seed <= self._MASK:
            raise ValueError("SplitMix64 seed must be an unsigned 64-bit integer")
        self.state = seed

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E37_79B9_7F4A_7C15) & self._MASK
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58_476D_1CE4_E5B9) & self._MASK
        value = ((value ^ (value >> 27)) * 0x94D0_49BB_1331_11EB) & self._MASK
        return (value ^ (value >> 31)) & self._MASK


def _as_f32(value: float) -> float:
    return struct.unpack("<f", struct.pack("<f", float(value)))[0]


def _f32_total_order_key(value: float) -> int:
    bits = struct.unpack("<I", struct.pack("<f", value))[0]
    if bits & 0x8000_0000:
        return (~bits) & 0xFFFF_FFFF
    return bits | 0x8000_0000


def _positive_rust_round(value: float) -> int:
    if not math.isfinite(value) or value < 0.0:
        raise ValueError("Q32 sampler rounding requires a finite nonnegative value")
    return math.floor(value + 0.5)


def _top_k_q32_weights(logits: Tensor | Sequence[float]) -> list[tuple[int, int]]:
    if isinstance(logits, Tensor):
        if logits.ndim != 1 or logits.dtype != torch.float32:
            raise ValueError("seeded sampler logits must be one-dimensional f32")
        values = [float(value) for value in logits.detach().cpu().tolist()]
    else:
        values = [_as_f32(value) for value in logits]
    if not values or any(not math.isfinite(value) for value in values):
        raise ValueError("seeded sampler requires nonempty finite logits")
    ranked = sorted(
        enumerate(values),
        key=lambda item: (-_f32_total_order_key(item[1]), item[0]),
    )[: min(TOP_K, len(values))]
    maximum = float(ranked[0][1])
    weighted: list[tuple[int, int]] = []
    for token, logit in ranked:
        probability_ratio = math.exp((float(logit) - maximum) / TEMPERATURE)
        weight = max(1, _positive_rust_round(probability_ratio * 4_294_967_296.0))
        weighted.append((token, weight))
    return weighted


def sample_top_k_q32(logits: Tensor | Sequence[float], sampler: SplitMix64) -> int:
    """Select one token using the exact existing top-k/Q32 policy."""

    weighted = _top_k_q32_weights(logits)
    total = sum(weight for _, weight in weighted)
    if total > (1 << 64) - 1:
        raise OverflowError("seeded sampler Q32 weight total overflowed")
    threshold = (sampler.next_u64() * total) >> 64
    cumulative = 0
    for token, weight in weighted:
        cumulative += weight
        if threshold < cumulative:
            return token
    raise RuntimeError("seeded sampler cumulative weights did not select a token")


def short_cycle_period(tokens: Sequence[int]) -> int | None:
    for period in range(1, 5):
        span = period * 3
        if len(tokens) < span:
            continue
        tail = tokens[-span:]
        if tail[:period] == tail[period : 2 * period] == tail[2 * period :]:
            return period
    return None


def _strict_json_object(path: Path) -> dict[str, Any]:
    def object_without_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        value: dict[str, Any] = {}
        for key, item in pairs:
            if key in value:
                raise ValueError(f"duplicate JSON key in {path}: {key}")
            value[key] = item
        return value

    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=object_without_duplicates,
            parse_constant=lambda constant: (_ for _ in ()).throw(
                ValueError(f"invalid JSON constant in {path}: {constant}")
            ),
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"invalid UTF-8 JSON object: {path}") from error
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    expected = value.get(field)
    unsigned = dict(value)
    unsigned.pop(field, None)
    if expected != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _with_self_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _canonical_equal(left: Any, right: Any) -> bool:
    return canonical_json_bytes(left) == canonical_json_bytes(right)


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


def _require_regular_path(root: Path, relative: str) -> Path:
    candidate = root
    for part in Path(relative).parts:
        candidate /= part
        if candidate.is_symlink():
            raise ValueError(
                f"generation input must not traverse a symlink: {relative}"
            )
    if candidate.is_symlink() or not candidate.is_file():
        raise ValueError(f"generation input must be a regular non-symlink file: {relative}")
    resolved = candidate.resolve()
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise ValueError(f"generation input escapes its root: {relative}") from error
    return resolved


def _verify_data_manifest(root: Path) -> dict[str, Any]:
    manifest_path = _require_regular_path(root, DATA_MANIFEST_NAME)
    manifest = verify_manifest_envelope(manifest_path)
    if (
        manifest.get("schema") != DATA_MANIFEST_SCHEMA
        or manifest.get("policy") != "R4RetainedLanguagePathV1"
        or manifest.get("manifest_cid") != EXPECTED_PREPARATION_MANIFEST_CID
        or manifest.get("geometry", {}).get("artifact_cid")
        != EXPECTED_GEOMETRY_ARTIFACT_CID
        or manifest.get("geometry", {}).get("file_cid") != EXPECTED_GEOMETRY_FILE_CID
        or manifest.get("source", {}).get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("language-path preparation manifest differs from generation freeze")
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=(TOKENIZER_RELATIVE_PATH, GEOMETRY_RELATIVE_PATH),
    )
    return manifest


def _verify_model_dependencies(parent: Mapping[str, Any]) -> dict[str, Any]:
    implementation = parent.get("implementation")
    if not isinstance(implementation, Mapping) or not isinstance(
        implementation.get("files"), list
    ):
        raise ValueError("parent result has no implementation dependency records")
    parent_records = {
        str(record.get("path")): record
        for record in implementation["files"]
        if isinstance(record, Mapping)
    }
    trainer_root = Path(__file__).resolve().parents[2]
    records: list[dict[str, Any]] = []
    for relative, (expected_bytes, expected_cid) in EXPECTED_MODEL_DEPENDENCIES.items():
        expected = {"path": relative, "bytes": expected_bytes, "cid": expected_cid}
        if parent_records.get(relative) != expected:
            raise ValueError(f"parent model dependency differs: {relative}")
        candidate = _require_regular_path(trainer_root, relative)
        observed = {
            "path": relative,
            "bytes": candidate.stat().st_size,
            "cid": cid_file(candidate),
        }
        if observed != expected:
            raise ValueError(f"current model dependency differs from fitted artifact: {relative}")
        records.append(observed)
    generator_record = artifact_records(
        trainer_root,
        [GENERATOR_RELATIVE_PATH],
    )[0]
    all_records = sorted([*records, generator_record], key=lambda record: str(record["path"]))
    return {"files": all_records, "tree_cid": tree_cid(all_records)}


def _verify_executed_runner_archive() -> dict[str, Any]:
    trainer_root = Path(__file__).resolve().parents[2]
    repository_root = trainer_root.parents[1]
    archive = _require_regular_path(
        repository_root, EXECUTED_RUNNER_ARCHIVE_RELATIVE_PATH
    )
    record = {
        "path": GENERATOR_RELATIVE_PATH,
        "bytes": archive.stat().st_size,
        "cid": cid_file(archive),
    }
    if not _canonical_equal(record, EXECUTED_RUNNER_RECORD):
        raise ValueError("archived executed generation runner does not reproduce")
    return record


def _verify_parent_result(root: Path) -> dict[str, Any]:
    result = _strict_json_object(_require_regular_path(root, PARENT_RESULT_RELATIVE_PATH))
    _verify_self_cid(result, "result_cid")
    retained = result.get("arms", {}).get("retained", {})
    artifact = retained.get("artifact", {}) if isinstance(retained, Mapping) else {}
    if (
        result.get("schema") != "uor-r4.retained-language-path-result/1"
        or result.get("policy") != "R4RetainedLanguagePathV1"
        or result.get("result_cid") != EXPECTED_PARENT_RESULT_CID
        or result.get("run_contract_cid") != EXPECTED_PARENT_RUN_CONTRACT_CID
        or result.get("preparation_manifest_cid") != EXPECTED_PREPARATION_MANIFEST_CID
        or result.get("verdict") != "RETAINED_LANGUAGE_PATH_PASS"
        or result.get("forbidden_reads") != 0
        or result.get("generation") != "NOT_RUN"
        or retained.get("arm_result_cid") != EXPECTED_RETAINED_ARM_RESULT_CID
        or retained.get("status") != "COMPLETE"
        or retained.get("forbidden_reads") != 0
        or retained.get("replay", {}).get("passed") is not True
        or artifact.get("path") != RETAINED_ARTIFACT_RELATIVE_PATH
        or artifact.get("cid") != EXPECTED_RETAINED_ARTIFACT_CID
        or artifact.get("bytes") != EXPECTED_RETAINED_ARTIFACT_BYTES
    ):
        raise ValueError("retained language-path parent result differs from positive freeze")
    return result


def _validate_prompts(tokenizer: _Tokenizer) -> None:
    for prompt in PROMPTS:
        encoded = tokenizer.encode(prompt.text, add_special_tokens=False).ids
        decoded = tokenizer.decode(list(prompt.token_ids), skip_special_tokens=True)
        if encoded != list(prompt.token_ids) or decoded != prompt.text:
            raise ValueError(f"public prompt {prompt.index} does not round-trip exactly")
        if any(token in (BOS_TOKEN_ID, EOS_TOKEN_ID, UNK_TOKEN_ID) for token in prompt.token_ids):
            raise ValueError(f"public prompt {prompt.index} contains a special content token")
        if 1 + len(prompt.token_ids) + MAX_NEW_TOKENS > CONTEXT:
            raise ValueError(f"public prompt {prompt.index} exceeds the frozen horizon")


def _validate_prompt_bytes(raw_decoder: _RawDecoder) -> None:
    for prompt in PROMPTS:
        if raw_decoder.decode_bytes(prompt.token_ids) != prompt.text.encode("utf-8"):
            raise ValueError(f"public prompt {prompt.index} does not round-trip byte-exact")


def _configure_cpu() -> dict[str, Any]:
    if platform.system() != "Darwin":
        raise RuntimeError("generation freeze requires the measured Apple CPU path")
    if torch.cuda.is_available():
        raise RuntimeError("CUDA is forbidden by the generation freeze")
    build = torch.__config__.show().lower()
    if "blas_info=accelerate" not in build:
        raise RuntimeError("generation freeze requires Apple Accelerate BLAS")
    os.environ["OMP_NUM_THREADS"] = str(THREADS)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(THREADS)
    os.environ["OPENBLAS_NUM_THREADS"] = str(THREADS)
    torch.use_deterministic_algorithms(True)
    torch.manual_seed(SEED)
    torch.set_num_threads(THREADS)
    try:
        torch.set_num_interop_threads(THREADS)
    except RuntimeError as error:
        if torch.get_num_interop_threads() != THREADS:
            raise RuntimeError("could not establish frozen interop thread count") from error
    if torch.get_num_threads() != THREADS or torch.get_num_interop_threads() != THREADS:
        raise RuntimeError("configured CPU thread counts differ from generation freeze")
    return {
        "platform": platform.system(),
        "backend": "cpu",
        "blas": "Apple Accelerate",
        "threads": THREADS,
        "workers": 1,
        "dtype": "float32",
        "deterministic_algorithms": True,
        "cuda": "FORBIDDEN",
        "torch": torch.__version__,
        "numpy": importlib.metadata.version("numpy"),
        "tokenizers": importlib.metadata.version("tokenizers"),
        "safetensors": importlib.metadata.version("safetensors"),
    }


def _fresh_model(
    geometry: GroupAddressArtifact, artifact_path: Path
) -> R4RetainedLanguagePathV1:
    payload = artifact_path.read_bytes()
    if len(payload) != EXPECTED_RETAINED_ARTIFACT_BYTES or cid_bytes(payload) != (
        EXPECTED_RETAINED_ARTIFACT_CID
    ):
        raise ValueError("retained learned artifact differs during fresh load")
    model = R4RetainedLanguagePathV1(geometry)
    model.load_learned_artifact(payload)
    model.to(device=torch.device("cpu"), dtype=torch.float32)
    model.eval()
    if model.parameter_count() != PARAMETER_COUNT:
        raise RuntimeError("fresh retained model parameter ledger differs")
    return model


_WORK_FIELDS = (
    "transported_state_values",
    "occupancy_slot_reads",
    "attention_slot_scores",
    "attention_value_reads",
    "key_delta_writes",
    "value_delta_writes",
    "vocabulary_scores",
)


def _consume_step_audit(totals: dict[str, int], audit: Any) -> None:
    if (
        audit.batch_size != 1
        or audit.token_steps != 1
        or audit.layers != LAYERS
        or audit.heads != HEADS
        or audit.group_size != 120
        or audit.state_off is not False
        or audit.implementation != "direct"
        or audit.forbidden_reads != 0
    ):
        raise RuntimeError("retained generation step audit differs from direct attention-on freeze")
    totals["token_steps"] += audit.token_steps
    totals["forbidden_reads"] += audit.forbidden_reads
    for field in _WORK_FIELDS:
        totals[field] += int(getattr(audit, field))


def _update_logits_trace(digest: Any, logits: Tensor) -> None:
    if logits.ndim != 1 or logits.shape[0] != VOCAB_SIZE or logits.dtype != torch.float32:
        raise RuntimeError("generation logits differ from the frozen f32 vocabulary shape")
    if not bool(torch.isfinite(logits).all()):
        raise RuntimeError("generation produced nonfinite logits")
    array = logits.detach().cpu().contiguous().numpy().astype("<f4", copy=False)
    digest.update(struct.pack(">I", int(array.shape[0])))
    digest.update(array.tobytes(order="C"))


def _stop_record(kind: str, period: int | None = None) -> str | dict[str, Any]:
    if kind == "short_cycle":
        if period not in (1, 2, 3, 4):
            raise ValueError("short-cycle stop requires a period from one through four")
        return {"short_cycle": {"period": period}}
    if kind not in ("eos", "maximum_new_tokens"):
        raise ValueError(f"unknown generation stop: {kind}")
    return kind


def _rollout(
    prompt: GenerationPrompt,
    *,
    raw_decoder: _RawDecoder,
    model_factory: Callable[[], Any],
    select_token: Callable[[Tensor | Sequence[float], SplitMix64], int] = sample_top_k_q32,
) -> dict[str, Any]:
    model = model_factory()
    state = model.initial_state(1, device=torch.device("cpu"), dtype=torch.float32)
    sampler = SplitMix64(SEED)
    input_token_ids = [BOS_TOKEN_ID, *prompt.token_ids]
    generated: list[int] = []
    trace = blake3()
    work = {"token_steps": 0, "forbidden_reads": 0, **{field: 0 for field in _WORK_FIELDS}}

    logits: Tensor | None = None
    with torch.inference_mode():
        for token in input_token_ids:
            step = model.step(
                torch.tensor([token], dtype=torch.long),
                state,
                attention_off=False,
            )
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(dtype=torch.float32).contiguous()
            _consume_step_audit(work, step.audit)
            _update_logits_trace(trace, logits)
        if logits is None:
            raise RuntimeError("generation prompt produced no next-token logits")

        stop_reason: str | dict[str, Any] = _stop_record("maximum_new_tokens")
        for decision in range(MAX_NEW_TOKENS):
            token = int(select_token(logits, sampler))
            if not 0 <= token < VOCAB_SIZE:
                raise RuntimeError("generation selected a token outside the vocabulary")
            generated.append(token)
            if token == EOS_TOKEN_ID:
                stop_reason = _stop_record("eos")
                break
            period = short_cycle_period(generated)
            if period is not None:
                stop_reason = _stop_record("short_cycle", period)
                break
            if decision + 1 == MAX_NEW_TOKENS:
                break
            step = model.step(
                torch.tensor([token], dtype=torch.long),
                state,
                attention_off=False,
            )
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(dtype=torch.float32).contiguous()
            _consume_step_audit(work, step.audit)
            _update_logits_trace(trace, logits)

    expected_positions = len(input_token_ids) + len(generated) - 1
    if work["token_steps"] != expected_positions:
        raise RuntimeError("generation did not self-feed exactly the emitted nonterminal prefix")
    expected_work = {
        "transported_state_values": expected_positions * STATE_VALUES,
        "occupancy_slot_reads": expected_positions * LAYERS * 120,
        "attention_slot_scores": expected_positions * LAYERS * HEADS * 120,
        "attention_value_reads": expected_positions * LAYERS * HEADS * 120 * HEAD_DIM,
        "key_delta_writes": expected_positions * LAYERS * HEADS * HEAD_DIM,
        "value_delta_writes": expected_positions * LAYERS * HEADS * HEAD_DIM,
        "vocabulary_scores": expected_positions * VOCAB_SIZE,
    }
    if any(work[field] != value for field, value in expected_work.items()):
        raise RuntimeError("generation work ledger differs from direct retained recurrence")

    first_eos_offset = generated.index(EOS_TOKEN_ID) if EOS_TOKEN_ID in generated else None
    response_ids = generated[:first_eos_offset] if first_eos_offset is not None else generated
    raw_bytes = raw_decoder.decode_bytes(generated)
    response_bytes = raw_decoder.decode_bytes(response_ids)
    raw_text, raw_utf8_decodable = _decode_utf8(raw_bytes)
    response_text, response_utf8_decodable = _decode_utf8(response_bytes)
    utf8_decodable = raw_utf8_decodable and response_utf8_decodable
    audit = {
        "schema": ROLLOUT_AUDIT_SCHEMA,
        "attention_off": False,
        "implementation": "direct",
        "positions_executed": expected_positions,
        "selected_tokens": len(generated),
        "work": work,
        "future_token_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "target_reads": 0,
        "source_data_reads": 0,
        "model_artifact_reads": 1,
    }
    record: dict[str, Any] = {
        "prompt": prompt.record(),
        "seed": SEED,
        "input_token_ids": input_token_ids,
        "generated_token_ids": generated,
        "fed_back_generated_token_ids": generated[:-1],
        "response_token_ids": response_ids,
        "stop_reason": stop_reason,
        "first_eos_offset": first_eos_offset,
        "short_cycle_period": short_cycle_period(generated),
        "raw_decoded_text": raw_text,
        "raw_decoded_utf8_hex": raw_bytes.hex(),
        "response_text": response_text,
        "response_utf8_hex": response_bytes.hex(),
        "utf8_decodable": utf8_decodable,
        "logits_trace": {
            "encoding": LOGITS_TRACE_ENCODING,
            "rows": expected_positions,
            "cid": f"blake3:{trace.hexdigest()}",
        },
        "audit": audit,
        "audit_cid": cid_bytes(canonical_json_bytes(audit)),
    }
    record["transcript_cid"] = cid_bytes(canonical_json_bytes(record))
    return record


def _model_contract() -> dict[str, Any]:
    return {
        "arm": "retained",
        "parameters": PARAMETER_COUNT,
        "state_values": STATE_VALUES,
        "state_bytes_f32": STATE_BYTES_F32,
        "validity_bits": VALIDITY_BITS,
        "context": CONTEXT,
        "attention_off": False,
        "implementation": "direct",
    }


def _decode_contract() -> dict[str, Any]:
    return {
        "sampler_policy": SAMPLER_POLICY,
        "seed": SEED,
        "seed_reset": "independently before every prompt and replay",
        "bos_token_id": BOS_TOKEN_ID,
        "bos_insertions_per_prompt": 1,
        "eos_token_id": EOS_TOKEN_ID,
        "max_new_tokens": MAX_NEW_TOKENS,
        "short_cycle_periods": [1, 2, 3, 4],
        "short_cycle_repetitions": 3,
        "stop_order": ["eos", "short_cycle", "maximum_new_tokens"],
        "self_feedback": "emitted past tokens only; terminal selected token is not fed",
    }


def _access_contract(*, isolated: bool) -> dict[str, Any]:
    return {
        "fresh_model_artifact_loads": 10,
        "fresh_zero_states": 10,
        "attention_off_executions": 0,
        "forbidden_reads": 0 if isolated else "NONZERO",
        "future_reads": 0 if isolated else "NONZERO",
        "provider_calls": 0,
        "teacher_calls": 0,
        "target_reads": 0,
        "source_data_reads": 0,
        "training_steps": 0,
        "optimizer_steps": 0,
    }


def _claims_contract(*, mechanical_passed: bool) -> dict[str, Any]:
    return {
        "autonomous_local_retained_decoding": "PASS" if mechanical_passed else "FAIL",
        "coherence": "NOT_EVALUATED_DESCRIPTIVE_OUTPUT_ONLY",
        "reasoning": "NOT_RUN",
        "h4_specific_superiority": "NOT_EVALUATED",
        "exact_table_runtime": "NOT_RUN",
        "browser_readiness": "NOT_RUN",
        "release_readiness": "NOT_RUN",
    }


def _behavior_contract(primary: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    return {
        "quality_gate": "NOT_DEFINED_DESCRIPTIVE_SMOKE",
        "records": [
            {
                "index": record["prompt"]["index"],
                "selected_tokens": len(record["generated_token_ids"]),
                "unique_generated_tokens": len(set(record["generated_token_ids"])),
                "stop_reason": record["stop_reason"],
                "response_text": record["response_text"],
            }
            for record in primary
        ],
    }


def _decode_utf8(value: bytes) -> tuple[str, bool]:
    try:
        return value.decode("utf-8", errors="strict"), True
    except UnicodeDecodeError:
        return value.decode("utf-8", errors="replace"), False


def _execute_generation(
    *,
    raw_decoder: _RawDecoder,
    model_factory: Callable[[], Any],
    input_evidence: Mapping[str, Any],
    implementation: Mapping[str, Any],
    environment: Mapping[str, Any],
    select_token: Callable[[Tensor | Sequence[float], SplitMix64], int] = sample_top_k_q32,
) -> dict[str, Any]:
    primary = [
        _rollout(
            prompt,
            raw_decoder=raw_decoder,
            model_factory=model_factory,
            select_token=select_token,
        )
        for prompt in PROMPTS
    ]
    replay = [
        _rollout(
            prompt,
            raw_decoder=raw_decoder,
            model_factory=model_factory,
            select_token=select_token,
        )
        for prompt in PROMPTS
    ]
    exact_replay = primary == replay
    all_bounded = all(
        1 <= len(record["generated_token_ids"]) <= MAX_NEW_TOKENS
        for record in primary
    )
    all_utf8 = all(record["utf8_decodable"] is True for record in primary)
    all_isolated = all(
        record["audit"][field] == 0
        for record in primary
        for field in (
            "future_token_reads",
            "provider_calls",
            "teacher_calls",
            "target_reads",
            "source_data_reads",
        )
    ) and all(record["audit"]["work"]["forbidden_reads"] == 0 for record in primary)
    mechanical_passed = (
        len(primary) == len(replay) == len(PROMPTS)
        and exact_replay
        and all_bounded
        and all_utf8
        and all_isolated
    )
    body: dict[str, Any] = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "freeze": {"comment_id": FREEZE_COMMENT_ID, "freeze_cid": FREEZE_CID},
        "inputs": dict(input_evidence),
        "implementation": dict(implementation),
        "environment": dict(environment),
        "model": _model_contract(),
        "prompt_contract": prompt_contract(),
        "prompt_contract_cid": EXPECTED_PROMPT_CONTRACT_CID,
        "decode": _decode_contract(),
        "primary": primary,
        "replay": replay,
        "replay_equality": {
            "exact": exact_replay,
            "primary_cid": cid_bytes(canonical_json_bytes(primary)),
            "replay_cid": cid_bytes(canonical_json_bytes(replay)),
            "fields": [
                "token IDs",
                "stop reason",
                "decoded bytes/text",
                "audit CID",
                "logits-trace CID",
            ],
        },
        "access": _access_contract(isolated=all_isolated),
        "behavior": _behavior_contract(primary),
        "mechanical_passed": mechanical_passed,
        "verdict": (
            "AUTONOMOUS_GENERATION_SMOKE_COMPLETE"
            if mechanical_passed
            else "INVALID_AUTONOMOUS_GENERATION_SMOKE"
        ),
        "claims": _claims_contract(mechanical_passed=mechanical_passed),
    }
    return _with_self_cid(body, "result_cid")


def _is_blake3_cid(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 71
        and value.startswith("blake3:")
        and all(character in "0123456789abcdef" for character in value[7:])
    )


def _expected_stop(
    generated: Sequence[int],
) -> tuple[str | dict[str, Any], int | None]:
    for offset, token in enumerate(generated):
        prefix = generated[: offset + 1]
        if token == EOS_TOKEN_ID:
            if offset + 1 != len(generated):
                raise ValueError("generation transcript continues after selected EOS")
            return _stop_record("eos"), offset
        period = short_cycle_period(prefix)
        if period is not None:
            if offset + 1 != len(generated):
                raise ValueError("generation transcript continues after a short cycle")
            return _stop_record("short_cycle", period), None
        if offset + 1 == MAX_NEW_TOKENS:
            return _stop_record("maximum_new_tokens"), None
    raise ValueError("generation transcript did not reach a frozen stop condition")


def _validate_rollout_record(
    record: Any,
    *,
    prompt: GenerationPrompt,
    raw_decoder: _RawDecoder,
) -> None:
    if not isinstance(record, Mapping):
        raise ValueError("generation transcript is not an object")
    expected_fields = {
        "prompt",
        "seed",
        "input_token_ids",
        "generated_token_ids",
        "fed_back_generated_token_ids",
        "response_token_ids",
        "stop_reason",
        "first_eos_offset",
        "short_cycle_period",
        "raw_decoded_text",
        "raw_decoded_utf8_hex",
        "response_text",
        "response_utf8_hex",
        "utf8_decodable",
        "logits_trace",
        "audit",
        "audit_cid",
        "transcript_cid",
    }
    if set(record) != expected_fields:
        raise ValueError("generation transcript fields differ from the freeze")
    if not _canonical_equal(record["prompt"], prompt.record()) or record["seed"] != SEED:
        raise ValueError("generation transcript prompt or seed differs from the freeze")
    expected_input = [BOS_TOKEN_ID, *prompt.token_ids]
    if not _canonical_equal(record["input_token_ids"], expected_input):
        raise ValueError("generation transcript does not prepend exactly one BOS")

    generated = record["generated_token_ids"]
    if (
        not isinstance(generated, list)
        or not 1 <= len(generated) <= MAX_NEW_TOKENS
        or any(
            isinstance(token, bool)
            or not isinstance(token, int)
            or not 0 <= token < VOCAB_SIZE
            for token in generated
        )
    ):
        raise ValueError("generation transcript has invalid selected token IDs")
    if not _canonical_equal(record["fed_back_generated_token_ids"], generated[:-1]):
        raise ValueError("generation transcript fed its terminal selected token")

    expected_stop, first_eos_offset = _expected_stop(generated)
    if (
        not _canonical_equal(record["stop_reason"], expected_stop)
        or not _canonical_equal(record["first_eos_offset"], first_eos_offset)
        or not _canonical_equal(record["short_cycle_period"], short_cycle_period(generated))
    ):
        raise ValueError("generation transcript stop evidence differs from the freeze")
    response_ids = generated[:first_eos_offset] if first_eos_offset is not None else generated
    if not _canonical_equal(record["response_token_ids"], response_ids):
        raise ValueError("generation transcript response does not slice only EOS")

    raw_bytes = raw_decoder.decode_bytes(generated)
    response_bytes = raw_decoder.decode_bytes(response_ids)
    raw_text, raw_valid = _decode_utf8(raw_bytes)
    response_text, response_valid = _decode_utf8(response_bytes)
    if (
        record["raw_decoded_utf8_hex"] != raw_bytes.hex()
        or record["response_utf8_hex"] != response_bytes.hex()
        or record["raw_decoded_text"] != raw_text
        or record["response_text"] != response_text
        or record["utf8_decodable"] is not (raw_valid and response_valid)
    ):
        raise ValueError("generation transcript byte decoding does not reproduce")

    positions = len(expected_input) + len(generated) - 1
    expected_work = {
        "token_steps": positions,
        "forbidden_reads": 0,
        "transported_state_values": positions * STATE_VALUES,
        "occupancy_slot_reads": positions * LAYERS * 120,
        "attention_slot_scores": positions * LAYERS * HEADS * 120,
        "attention_value_reads": positions * LAYERS * HEADS * 120 * HEAD_DIM,
        "key_delta_writes": positions * LAYERS * HEADS * HEAD_DIM,
        "value_delta_writes": positions * LAYERS * HEADS * HEAD_DIM,
        "vocabulary_scores": positions * VOCAB_SIZE,
    }
    expected_audit = {
        "schema": ROLLOUT_AUDIT_SCHEMA,
        "attention_off": False,
        "implementation": "direct",
        "positions_executed": positions,
        "selected_tokens": len(generated),
        "work": expected_work,
        "future_token_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "target_reads": 0,
        "source_data_reads": 0,
        "model_artifact_reads": 1,
    }
    if not _canonical_equal(record["audit"], expected_audit):
        raise ValueError("generation transcript audit differs from direct retained work")
    if record["audit_cid"] != cid_bytes(canonical_json_bytes(record["audit"])):
        raise ValueError("generation transcript audit CID does not reproduce")

    trace = record["logits_trace"]
    if (
        not isinstance(trace, Mapping)
        or set(trace) != {"encoding", "rows", "cid"}
        or trace["encoding"] != LOGITS_TRACE_ENCODING
        or trace["rows"] != positions
        or not _is_blake3_cid(trace["cid"])
    ):
        raise ValueError("generation transcript logits trace differs from the freeze")
    _verify_self_cid(record, "transcript_cid")


def _validate_environment(value: Any) -> dict[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError("generation environment is not an object")
    expected_fields = {
        "platform",
        "backend",
        "blas",
        "threads",
        "workers",
        "dtype",
        "deterministic_algorithms",
        "cuda",
        "torch",
        "numpy",
        "tokenizers",
        "safetensors",
    }
    constraints = {
        "platform": "Darwin",
        "backend": "cpu",
        "blas": "Apple Accelerate",
        "threads": THREADS,
        "workers": 1,
        "dtype": "float32",
        "deterministic_algorithms": True,
        "cuda": "FORBIDDEN",
    }
    if set(value) != expected_fields:
        raise ValueError("generation environment differs from the frozen CPU path")
    if any(
        not isinstance(value.get(package), str) or not value[package]
        for package in ("torch", "numpy", "tokenizers", "safetensors")
    ):
        raise ValueError("generation environment lacks dependency versions")
    expected = {
        **constraints,
        **{package: value[package] for package in ("torch", "numpy", "tokenizers", "safetensors")},
    }
    if not _canonical_equal(value, expected):
        raise ValueError("generation environment differs from the frozen CPU path")
    return dict(value)


def _validate_implementation(
    observed: Any,
    current: Mapping[str, Any],
    *,
    executed_runner_record: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    if not isinstance(observed, Mapping):
        raise ValueError("generation implementation is not an object")
    files = observed.get("files")
    if set(observed) != {"files", "tree_cid"} or not isinstance(files, list):
        raise ValueError("generation implementation dependency shape differs")
    if any(
        not isinstance(record, Mapping)
        or set(record) != {"path", "bytes", "cid"}
        or not isinstance(record["path"], str)
        or not record["path"]
        or isinstance(record["bytes"], bool)
        or not isinstance(record["bytes"], int)
        or record["bytes"] < 0
        or not _is_blake3_cid(record["cid"])
        for record in files
    ):
        raise ValueError("generation implementation has an invalid dependency record")
    if files != sorted(files, key=lambda record: str(record["path"])):
        raise ValueError("generation implementation dependencies are not canonical")
    if len({record["path"] for record in files}) != len(files):
        raise ValueError("generation implementation repeats a dependency path")
    if observed["tree_cid"] != tree_cid(files):
        raise ValueError("generation implementation tree CID does not reproduce")

    if not isinstance(current, Mapping):
        raise ValueError("current verified implementation contract is not an object")
    current_files = current.get("files")
    if (
        set(current) != {"files", "tree_cid"}
        or not isinstance(current_files, list)
        or current["tree_cid"] != tree_cid(current_files)
    ):
        raise ValueError("current verified implementation contract does not reproduce")
    current_runner = [
        record for record in current_files if record.get("path") == GENERATOR_RELATIVE_PATH
    ]
    if len(current_runner) != 1:
        raise ValueError("current implementation lacks exactly one generation runner")
    common = [
        record for record in current_files if record.get("path") != GENERATOR_RELATIVE_PATH
    ]
    if executed_runner_record is not None and not _canonical_equal(
        executed_runner_record, EXECUTED_RUNNER_RECORD
    ):
        raise ValueError("historical executed-runner record differs from official evidence")
    if executed_runner_record is not None:
        _verify_executed_runner_archive()
    runner = dict(executed_runner_record or current_runner[0])
    expected_files = sorted([*common, runner], key=lambda record: str(record["path"]))
    if not _canonical_equal(files, expected_files):
        raise ValueError("generation implementation differs from verified dependencies")
    expected_tree_cid = tree_cid(expected_files)
    if observed["tree_cid"] != expected_tree_cid:
        raise ValueError("generation implementation differs from its expected runner tree")
    if (
        executed_runner_record is not None
        and expected_tree_cid != EXECUTED_IMPLEMENTATION_TREE_CID
    ):
        raise RuntimeError("historical executed-runner tree constant does not reproduce")
    return dict(observed)


def _validate_frozen_result(
    result: Any,
    *,
    raw_decoder: _RawDecoder,
    input_evidence: Mapping[str, Any],
    implementation: Mapping[str, Any],
    executed_runner_record: Mapping[str, Any] | None = None,
) -> None:
    if not isinstance(result, Mapping):
        raise ValueError("generation result is not an object")
    _verify_self_cid(result, "result_cid")
    if not _canonical_equal(result.get("inputs"), input_evidence):
        raise ValueError("generation result inputs differ from verified frozen evidence")
    implementation_record = _validate_implementation(
        result.get("implementation"),
        implementation,
        executed_runner_record=executed_runner_record,
    )
    environment = _validate_environment(result.get("environment"))
    primary = result.get("primary")
    replay = result.get("replay")
    if (
        not isinstance(primary, list)
        or not isinstance(replay, list)
        or len(primary) != len(PROMPTS)
        or len(replay) != len(PROMPTS)
    ):
        raise ValueError("generation result must contain five primary and replay transcripts")
    for prompt, record in zip(PROMPTS, primary, strict=True):
        _validate_rollout_record(record, prompt=prompt, raw_decoder=raw_decoder)
    for prompt, record in zip(PROMPTS, replay, strict=True):
        _validate_rollout_record(record, prompt=prompt, raw_decoder=raw_decoder)

    exact_replay = _canonical_equal(primary, replay)
    all_bounded = all(
        1 <= len(record["generated_token_ids"]) <= MAX_NEW_TOKENS
        for record in primary
    )
    all_utf8 = all(record["utf8_decodable"] is True for record in primary)
    all_isolated = all(
        record["audit"][field] == 0
        for record in primary
        for field in (
            "future_token_reads",
            "provider_calls",
            "teacher_calls",
            "target_reads",
            "source_data_reads",
        )
    ) and all(record["audit"]["work"]["forbidden_reads"] == 0 for record in primary)
    mechanical_passed = exact_replay and all_bounded and all_utf8 and all_isolated
    expected_body: dict[str, Any] = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "freeze": {"comment_id": FREEZE_COMMENT_ID, "freeze_cid": FREEZE_CID},
        "inputs": dict(input_evidence),
        "implementation": implementation_record,
        "environment": environment,
        "model": _model_contract(),
        "prompt_contract": prompt_contract(),
        "prompt_contract_cid": EXPECTED_PROMPT_CONTRACT_CID,
        "decode": _decode_contract(),
        "primary": primary,
        "replay": replay,
        "replay_equality": {
            "exact": exact_replay,
            "primary_cid": cid_bytes(canonical_json_bytes(primary)),
            "replay_cid": cid_bytes(canonical_json_bytes(replay)),
            "fields": [
                "token IDs",
                "stop reason",
                "decoded bytes/text",
                "audit CID",
                "logits-trace CID",
            ],
        },
        "access": _access_contract(isolated=all_isolated),
        "behavior": _behavior_contract(primary),
        "mechanical_passed": mechanical_passed,
        "verdict": (
            "AUTONOMOUS_GENERATION_SMOKE_COMPLETE"
            if mechanical_passed
            else "INVALID_AUTONOMOUS_GENERATION_SMOKE"
        ),
        "claims": _claims_contract(mechanical_passed=mechanical_passed),
    }
    if not _canonical_equal(dict(result), _with_self_cid(expected_body, "result_cid")):
        raise ValueError("generation result differs from its frozen derived evidence")


def _historical_runner_for_result(result: Mapping[str, Any]) -> Mapping[str, Any] | None:
    implementation = result.get("implementation")
    files = implementation.get("files") if isinstance(implementation, Mapping) else None
    if not isinstance(files, list):
        return None
    runners = [
        record
        for record in files
        if isinstance(record, Mapping)
        and record.get("path") == GENERATOR_RELATIVE_PATH
    ]
    if len(runners) == 1 and _canonical_equal(runners[0], EXECUTED_RUNNER_RECORD):
        return EXECUTED_RUNNER_RECORD
    return None


def run_language_path_generation(root: Path) -> dict[str, Any]:
    """Execute or verify the one create-once retained generation smoke."""

    if root.is_symlink():
        raise ValueError("language-path generation root must not be a symlink")
    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists():
        if result_path.is_symlink():
            raise ValueError("generation result must not be a symlink")
        existing_result: dict[str, Any] | None = _strict_json_object(result_path)
    else:
        existing_result = None

    manifest = _verify_data_manifest(root)
    parent = _verify_parent_result(root)
    implementation = _verify_model_dependencies(parent)
    artifact_path = _require_regular_path(root, RETAINED_ARTIFACT_RELATIVE_PATH)
    if (
        artifact_path.stat().st_size != EXPECTED_RETAINED_ARTIFACT_BYTES
        or cid_file(artifact_path) != EXPECTED_RETAINED_ARTIFACT_CID
    ):
        raise ValueError("retained learned artifact differs from generation freeze")
    tokenizer_path = _require_regular_path(root, TOKENIZER_RELATIVE_PATH)
    geometry_path = _require_regular_path(root, GEOMETRY_RELATIVE_PATH)
    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    if tokenizer.get_vocab_size(with_added_tokens=True) != VOCAB_SIZE:
        raise ValueError("copied tokenizer vocabulary differs from retained model")
    _validate_prompts(tokenizer)
    raw_decoder = ByteLevelRawDecoder.from_tokenizer_json(tokenizer_path)
    if len(raw_decoder.token_bytes) != VOCAB_SIZE:
        raise ValueError("raw ByteLevel decoder vocabulary differs from retained model")
    _validate_prompt_bytes(raw_decoder)
    input_evidence = {
        "preparation_manifest_cid": manifest["manifest_cid"],
        "parent_result_cid": parent["result_cid"],
        "parent_run_contract_cid": parent["run_contract_cid"],
        "retained_arm_result_cid": parent["arms"]["retained"]["arm_result_cid"],
        "retained_artifact": {
            "path": RETAINED_ARTIFACT_RELATIVE_PATH,
            "bytes": EXPECTED_RETAINED_ARTIFACT_BYTES,
            "cid": EXPECTED_RETAINED_ARTIFACT_CID,
        },
        "tokenizer": {
            "path": TOKENIZER_RELATIVE_PATH,
            "cid": EXPECTED_TOKENIZER_CID,
        },
        "geometry": {
            "path": GEOMETRY_RELATIVE_PATH,
            "file_cid": EXPECTED_GEOMETRY_FILE_CID,
            "artifact_cid": EXPECTED_GEOMETRY_ARTIFACT_CID,
            "arm": "exact_h4",
        },
        "verified_data_artifact_subset": [TOKENIZER_RELATIVE_PATH, GEOMETRY_RELATIVE_PATH],
        "unopened_data_artifacts": ["data/train.u16", "data/validation.u16"],
    }
    if existing_result is not None:
        _validate_frozen_result(
            existing_result,
            raw_decoder=raw_decoder,
            input_evidence=input_evidence,
            implementation=implementation,
            executed_runner_record=_historical_runner_for_result(existing_result),
        )
        return existing_result

    environment = _configure_cpu()
    geometry_bundle = load_group_geometry_artifacts(geometry_path)
    if (
        geometry_bundle.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or geometry_bundle.geometry_file_cid != EXPECTED_GEOMETRY_FILE_CID
    ):
        raise ValueError("copied exact-H4 geometry differs from generation freeze")
    result = _execute_generation(
        raw_decoder=raw_decoder,
        model_factory=lambda: _fresh_model(geometry_bundle.exact_h4, artifact_path),
        input_evidence=input_evidence,
        implementation=implementation,
        environment=environment,
    )
    _validate_frozen_result(
        result,
        raw_decoder=raw_decoder,
        input_evidence=input_evidence,
        implementation=implementation,
    )
    _write_exclusive_json(result_path, result)
    return result
