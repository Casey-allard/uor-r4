"""Frozen nonsealed language-path population for issue #973.

Preparation verifies #1019 through its training-view loader, copies only the
named nonsealed token slices and tokenizer, and copies one explicitly supplied
canonical geometry artifact.  Loading the resulting root is self-contained:
it verifies only those copied artifacts and never revisits #1019.
"""

from __future__ import annotations

import shutil
import struct
import tempfile
from collections.abc import Callable, Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import torch
from blake3 import blake3
from torch import Tensor

from .capacity_data import load_capacity_training_view_manifest
from .group_retention_campaign import GroupGeometryBundle, load_group_geometry_artifacts
from .paths import model_store_root
from .provenance import (
    atomic_write,
    cid_bytes,
    verify_bound_manifest,
    write_bound_manifest,
)


ISSUE = 973
POLICY = "R4RetainedLanguagePathV1"

DATA_MANIFEST_NAME = "retained-language-path-data-manifest.json"
DATA_MANIFEST_SCHEMA = "uor-r4.retained-language-path-data/1"

TRAIN_RELATIVE_PATH = "data/train.u16"
VALIDATION_RELATIVE_PATH = "data/validation.u16"
TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"
GEOMETRY_RELATIVE_PATH = "geometry/r4-group-address-geometry.json"
COPIED_ARTIFACT_PATHS = frozenset(
    {
        TRAIN_RELATIVE_PATH,
        VALIDATION_RELATIVE_PATH,
        TOKENIZER_RELATIVE_PATH,
        GEOMETRY_RELATIVE_PATH,
    }
)

SOURCE_TRAIN_RELATIVE_PATH = "tokens/train.u16"
SOURCE_VALIDATION_RELATIVE_PATH = "tokens/dev.u16"
SOURCE_TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"

EXPECTED_SOURCE_TRAINING_VIEW_CID = (
    "blake3:bb090c4b87fb62e71ce073c2e4df525745109e71e0db3e9846852a696af5501e"
)
EXPECTED_SOURCE_DATASET_MANIFEST_CID = (
    "blake3:6efbffeb1b6cb20ae9bbcda03428a4b820824224c578a168cd9a65f616f3dd5c"
)
EXPECTED_SOURCE_SPLIT_POLICY_CID = (
    "blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
EXPECTED_SOURCE_TRAIN_STORE_CID = (
    "blake3:c2752553b0b855a75685bb8ed16e113221e9a93575771e20046e95224b347e79"
)
EXPECTED_SOURCE_VALIDATION_STORE_CID = (
    "blake3:16e81a98cee6075fe740b7c612a2ef101a0bd790c1e664cb69af3a43f5aad2ca"
)

EXPECTED_GEOMETRY_ARTIFACT_CID = (
    "blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b"
)
EXPECTED_GEOMETRY_FILE_CID = (
    "blake3:a812cf6749e637f4c486a6ad206b96c90d695b5c4bb2ed029df3c6bef147d702"
)

WINDOW_TOKENS = 121
CONTEXT = WINDOW_TOKENS - 1
TRAIN_SOURCE_OFFSET_TOKENS = 149_996_595
TRAIN_TOKENS = 5_285_280
TRAIN_WINDOWS = 43_680
TRAIN_DECISIONS = 5_241_600
VALIDATION_SOURCE_OFFSET_TOKENS = 0
VALIDATION_TOKENS = 249_986
VALIDATION_WINDOWS = 2_066
VALIDATION_DECISIONS = 247_920
WINDOW_ORDER_SEED = 9_738
WINDOW_ORDER_ENCODING = "BLAKE3(struct.pack('>QQ', seed, window_ordinal))"

EXPECTED_TRAIN_SLICE_CID = (
    "blake3:8efeef090f1d729ad7782cd9f14a52561438a6cf256b58c62240f3fab83ae118"
)
EXPECTED_VALIDATION_SLICE_CID = (
    "blake3:75b8d841a580211d55a81df04eee54807fec80549504cabc4238e5bd883bdfb8"
)

SOURCE_TRAIN_TOKENS = 275_251_200
SOURCE_VALIDATION_TOKENS = 250_000


def default_language_path_source_root() -> Path:
    """Return the shared #1019 root while honoring ``UOR_MODEL_STORE``."""

    return model_store_root() / "research" / "issue-1019"


def default_language_path_root() -> Path:
    """Return a new-root location while honoring ``UOR_MODEL_STORE``."""

    return model_store_root() / "research" / "issue-973-retained-language-path-v1"


def deterministic_window_order(
    window_count: int, *, seed: int = WINDOW_ORDER_SEED
) -> tuple[int, ...]:
    """Return the frozen one-pass, without-replacement window permutation."""

    if isinstance(window_count, bool) or not isinstance(window_count, int):
        raise TypeError("window count must be an integer")
    if isinstance(seed, bool) or not isinstance(seed, int):
        raise TypeError("window-order seed must be an integer")
    if not 0 < window_count <= 2**64:
        raise ValueError("window count must fit the nonempty unsigned 64-bit domain")
    if not 0 <= seed < 2**64:
        raise ValueError("window-order seed must fit an unsigned 64-bit integer")
    return tuple(
        sorted(
            range(window_count),
            key=lambda ordinal: (
                blake3(struct.pack(">QQ", seed, ordinal)).digest(),
                ordinal,
            ),
        )
    )


class LanguagePathWindowStore:
    """Read-only little-endian u16 store of nonoverlapping 121-token windows."""

    __slots__ = ("path", "window_count", "_windows")

    def __init__(self, path: Path, *, window_count: int) -> None:
        if (
            isinstance(window_count, bool)
            or not isinstance(window_count, int)
            or window_count < 1
        ):
            raise ValueError("language-path window count must be a positive integer")
        if path.is_symlink():
            raise ValueError("language-path token store must not be a symlink")
        path = path.resolve()
        if not path.is_file():
            raise ValueError("language-path token store must be a regular non-symlink file")
        expected_bytes = window_count * WINDOW_TOKENS * 2
        if path.stat().st_size != expected_bytes:
            raise ValueError(
                f"language-path token store has {path.stat().st_size} bytes; "
                f"expected {expected_bytes}"
            )
        self.path = path
        self.window_count = window_count
        self._windows = np.memmap(
            path,
            mode="r",
            dtype="<u2",
            shape=(window_count, WINDOW_TOKENS),
        )

    def __len__(self) -> int:
        return self.window_count

    @property
    def windows(self) -> np.ndarray:
        """Expose the read-only ``[window, 121]`` memory-mapped token view."""

        return self._windows

    def window(self, ordinal: int) -> np.ndarray:
        """Return one read-only 121-token view."""

        if isinstance(ordinal, bool) or not isinstance(ordinal, int):
            raise TypeError("window ordinal must be an integer")
        if not 0 <= ordinal < self.window_count:
            raise IndexError("window ordinal is outside the store")
        return self._windows[ordinal]

    def batch(self, ordinals: Sequence[int]) -> tuple[Tensor, Tensor]:
        """Copy selected windows into causal ``[batch, 120]`` long tensors."""

        if len(ordinals) == 0:
            raise ValueError("language-path batch cannot be empty")
        normalized: list[int] = []
        for ordinal in ordinals:
            if isinstance(ordinal, bool) or not isinstance(ordinal, int):
                raise TypeError("window ordinal must be an integer")
            if not 0 <= ordinal < self.window_count:
                raise IndexError("window ordinal is outside the store")
            normalized.append(ordinal)
        selected = np.array(self._windows[normalized], dtype=np.int64, copy=True)
        tokens = torch.from_numpy(selected)
        return tokens[:, :-1], tokens[:, 1:]

    def batches(
        self, ordinals: Sequence[int], *, batch_size: int
    ) -> Iterator[tuple[Tensor, Tensor]]:
        """Yield deterministic batches over an explicitly supplied order."""

        if isinstance(batch_size, bool) or not isinstance(batch_size, int):
            raise TypeError("batch size must be an integer")
        if batch_size < 1:
            raise ValueError("batch size must be positive")
        for start in range(0, len(ordinals), batch_size):
            yield self.batch(ordinals[start : start + batch_size])


@dataclass(frozen=True, slots=True)
class LanguagePathData:
    """Fully verified, self-contained language-path construction inputs."""

    root: Path
    manifest: dict[str, Any]
    train_windows: LanguagePathWindowStore
    validation_windows: LanguagePathWindowStore
    geometry: GroupGeometryBundle
    tokenizer_path: Path
    train_order: tuple[int, ...]


def _artifact_by_path(manifest: dict[str, Any], relative_path: str) -> dict[str, Any]:
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError("source training view has no artifact records")
    matched = [
        record
        for record in records
        if isinstance(record, dict) and record.get("path") == relative_path
    ]
    if len(matched) != 1:
        raise ValueError(f"source training view does not bind exactly one {relative_path}")
    return matched[0]


def _validate_source_training_view(manifest: dict[str, Any]) -> None:
    """Bind the safe #1019 loader result to the independently frozen source."""

    if (
        manifest.get("manifest_cid") != EXPECTED_SOURCE_TRAINING_VIEW_CID
        or manifest.get("dataset_manifest_cid")
        != EXPECTED_SOURCE_DATASET_MANIFEST_CID
        or manifest.get("split_policy_cid") != EXPECTED_SOURCE_SPLIT_POLICY_CID
        or manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("#1019 training-view identity differs from the frozen source")
    expected = {
        SOURCE_TRAIN_RELATIVE_PATH: (
            EXPECTED_SOURCE_TRAIN_STORE_CID,
            SOURCE_TRAIN_TOKENS * 2,
        ),
        SOURCE_VALIDATION_RELATIVE_PATH: (
            EXPECTED_SOURCE_VALIDATION_STORE_CID,
            SOURCE_VALIDATION_TOKENS * 2,
        ),
        SOURCE_TOKENIZER_RELATIVE_PATH: (EXPECTED_TOKENIZER_CID, None),
    }
    for relative_path, (expected_cid, expected_bytes) in expected.items():
        record = _artifact_by_path(manifest, relative_path)
        if record.get("cid") != expected_cid or (
            expected_bytes is not None and record.get("bytes") != expected_bytes
        ):
            raise ValueError(f"#1019 {relative_path} identity differs")


def _validate_frozen_arithmetic() -> None:
    if (
        CONTEXT != WINDOW_TOKENS - 1
        or TRAIN_TOKENS != TRAIN_WINDOWS * WINDOW_TOKENS
        or VALIDATION_TOKENS != VALIDATION_WINDOWS * WINDOW_TOKENS
        or TRAIN_DECISIONS != TRAIN_WINDOWS * CONTEXT
        or VALIDATION_DECISIONS != VALIDATION_WINDOWS * CONTEXT
        or TRAIN_SOURCE_OFFSET_TOKENS + TRAIN_TOKENS > SOURCE_TRAIN_TOKENS
        or VALIDATION_SOURCE_OFFSET_TOKENS + VALIDATION_TOKENS
        > SOURCE_VALIDATION_TOKENS
    ):
        raise ValueError("language-path population arithmetic differs from the freeze")


def _read_u16_slice(path: Path, *, offset_tokens: int, token_count: int) -> bytes:
    """Read one exact little-endian u16 slice without materializing its prefix."""

    if any(
        isinstance(value, bool) or not isinstance(value, int)
        for value in (offset_tokens, token_count)
    ):
        raise TypeError("u16 slice coordinates must be integers")
    if offset_tokens < 0 or token_count < 1:
        raise ValueError("u16 slice coordinates are invalid")
    if path.is_symlink():
        raise ValueError("u16 source must not be a symlink")
    path = path.resolve()
    if not path.is_file():
        raise ValueError("u16 source must be a regular non-symlink file")
    source_bytes = path.stat().st_size
    if source_bytes % 2:
        raise ValueError("u16 source has an odd byte length")
    byte_offset = offset_tokens * 2
    byte_count = token_count * 2
    if byte_offset + byte_count > source_bytes:
        raise ValueError("u16 slice crosses the source boundary")
    with path.open("rb") as source:
        source.seek(byte_offset)
        value = source.read(byte_count)
    if len(value) != byte_count:
        raise ValueError("u16 source ended before the frozen slice")
    return value


def _read_verified_file(path: Path, *, expected_cid: str, label: str) -> bytes:
    if path.is_symlink():
        raise ValueError(f"{label} must not be a symlink")
    path = path.resolve()
    if not path.is_file():
        raise ValueError(f"{label} must be a regular non-symlink file")
    value = path.read_bytes()
    if cid_bytes(value) != expected_cid:
        raise ValueError(f"{label} CID differs from the frozen identity")
    return value


def _payload() -> dict[str, Any]:
    return {
        "schema": DATA_MANIFEST_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "source": {
            "issue": 1019,
            "training_view_manifest_cid": EXPECTED_SOURCE_TRAINING_VIEW_CID,
            "dataset_manifest_cid": EXPECTED_SOURCE_DATASET_MANIFEST_CID,
            "split_policy_cid": EXPECTED_SOURCE_SPLIT_POLICY_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "train_store_cid": EXPECTED_SOURCE_TRAIN_STORE_CID,
            "validation_store_cid": EXPECTED_SOURCE_VALIDATION_STORE_CID,
            "eligible_view": "nonsealed training view only",
        },
        "population": {
            "window_tokens": WINDOW_TOKENS,
            "context": CONTEXT,
            "train": {
                "source_relative_path": SOURCE_TRAIN_RELATIVE_PATH,
                "source_offset_tokens": TRAIN_SOURCE_OFFSET_TOKENS,
                "tokens": TRAIN_TOKENS,
                "windows": TRAIN_WINDOWS,
                "decisions": TRAIN_DECISIONS,
                "slice_cid": EXPECTED_TRAIN_SLICE_CID,
            },
            "validation": {
                "source_relative_path": SOURCE_VALIDATION_RELATIVE_PATH,
                "source_offset_tokens": VALIDATION_SOURCE_OFFSET_TOKENS,
                "tokens": VALIDATION_TOKENS,
                "windows": VALIDATION_WINDOWS,
                "decisions": VALIDATION_DECISIONS,
                "slice_cid": EXPECTED_VALIDATION_SLICE_CID,
            },
        },
        "window_order": {
            "policy": "one deterministic epoch without replacement",
            "digest": "BLAKE3",
            "encoding": WINDOW_ORDER_ENCODING,
            "seed": WINDOW_ORDER_SEED,
            "sort": "ascending lexicographic digest, then ordinal",
        },
        "geometry": {
            "artifact_cid": EXPECTED_GEOMETRY_ARTIFACT_CID,
            "file_cid": EXPECTED_GEOMETRY_FILE_CID,
            "copied_relative_path": GEOMETRY_RELATIVE_PATH,
        },
        "access": {
            "source_training_view_loader_calls": 1,
            "source_dataset_loader_calls": 0,
            "source_sealed_artifact_reads": 0,
            "source_checkpoint_reads": 0,
            "source_weight_reads": 0,
            "teacher_logit_reads": 0,
            "heldout_reveal_reads": 0,
        },
    }


def _validate_manifest_contract(manifest: dict[str, Any]) -> None:
    expected_payload = _payload()
    observed_payload = {
        key: manifest.get(key)
        for key in (
            "schema",
            "issue",
            "policy",
            "source",
            "population",
            "window_order",
            "geometry",
            "access",
        )
    }
    if observed_payload != expected_payload:
        raise ValueError("language-path data manifest differs from the frozen contract")
    allowed_keys = set(expected_payload) | {"artifacts", "tree_cid", "manifest_cid"}
    if set(manifest) != allowed_keys:
        raise ValueError("language-path data manifest has unexpected fields")
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError("language-path data manifest has no artifact records")
    paths = [record.get("path") for record in records if isinstance(record, dict)]
    if len(paths) != len(records) or set(paths) != COPIED_ARTIFACT_PATHS:
        raise ValueError("language-path data manifest binds unexpected artifacts")
    expected_cids = {
        TRAIN_RELATIVE_PATH: EXPECTED_TRAIN_SLICE_CID,
        VALIDATION_RELATIVE_PATH: EXPECTED_VALIDATION_SLICE_CID,
        TOKENIZER_RELATIVE_PATH: EXPECTED_TOKENIZER_CID,
        GEOMETRY_RELATIVE_PATH: EXPECTED_GEOMETRY_FILE_CID,
    }
    expected_bytes = {
        TRAIN_RELATIVE_PATH: TRAIN_TOKENS * 2,
        VALIDATION_RELATIVE_PATH: VALIDATION_TOKENS * 2,
    }
    for record in records:
        path = str(record["path"])
        if record.get("cid") != expected_cids[path] or (
            path in expected_bytes and record.get("bytes") != expected_bytes[path]
        ):
            raise ValueError(f"language-path copied artifact identity differs: {path}")


def load_language_path_preparation(
    root: Path,
    *,
    _geometry_loader: Callable[[Path], GroupGeometryBundle] = load_group_geometry_artifacts,
) -> LanguagePathData:
    """Verify only the copied language-path root and return read-only inputs."""

    _validate_frozen_arithmetic()
    if root.is_symlink():
        raise ValueError("language-path data root must not be a symlink")
    root = root.resolve()
    managed_paths = (*COPIED_ARTIFACT_PATHS, DATA_MANIFEST_NAME)
    for relative_path in managed_paths:
        current = root
        for part in Path(relative_path).parts:
            current /= part
            if current.is_symlink():
                raise ValueError("language-path copied root must not contain symlinks")
    manifest = verify_bound_manifest(root / DATA_MANIFEST_NAME, artifact_root=root)
    _validate_manifest_contract(manifest)
    geometry = _geometry_loader(root / GEOMETRY_RELATIVE_PATH)
    if (
        geometry.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or geometry.geometry_file_cid != EXPECTED_GEOMETRY_FILE_CID
    ):
        raise ValueError("copied geometry identity differs from the frozen exact artifact")
    train = LanguagePathWindowStore(
        root / TRAIN_RELATIVE_PATH,
        window_count=TRAIN_WINDOWS,
    )
    validation = LanguagePathWindowStore(
        root / VALIDATION_RELATIVE_PATH,
        window_count=VALIDATION_WINDOWS,
    )
    return LanguagePathData(
        root=root,
        manifest=manifest,
        train_windows=train,
        validation_windows=validation,
        geometry=geometry,
        tokenizer_path=(root / TOKENIZER_RELATIVE_PATH).resolve(),
        train_order=deterministic_window_order(TRAIN_WINDOWS),
    )


def prepare_language_path_data(
    *,
    source_root: Path,
    output_root: Path,
    geometry_path: Path,
    _source_loader: Callable[[Path], dict[str, Any]] = load_capacity_training_view_manifest,
    _geometry_loader: Callable[[Path], GroupGeometryBundle] = load_group_geometry_artifacts,
) -> LanguagePathData:
    """Create the frozen four-artifact root exactly once.

    ``geometry_path`` is intentionally mandatory: preparation may copy only an
    explicitly selected, strictly validated canonical exact-geometry export.
    """

    _validate_frozen_arithmetic()
    source_root = source_root.resolve()
    if output_root.is_symlink():
        raise FileExistsError("language-path data root must not be a symlink")
    output_root = output_root.resolve()
    if geometry_path.is_symlink():
        raise ValueError("explicit geometry path must not be a symlink")
    geometry_path = geometry_path.resolve()
    if output_root == source_root or output_root.is_relative_to(source_root):
        raise ValueError("language-path output root must not be inside the #1019 source root")
    if output_root.exists() or output_root.is_symlink():
        raise FileExistsError("language-path data root is create-once and must be new")

    source_manifest = _source_loader(source_root)
    _validate_source_training_view(source_manifest)

    geometry = _geometry_loader(geometry_path)
    if (
        geometry.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or geometry.geometry_file_cid != EXPECTED_GEOMETRY_FILE_CID
    ):
        raise ValueError("explicit geometry differs from the frozen canonical artifact")

    train_bytes = _read_u16_slice(
        source_root / SOURCE_TRAIN_RELATIVE_PATH,
        offset_tokens=TRAIN_SOURCE_OFFSET_TOKENS,
        token_count=TRAIN_TOKENS,
    )
    validation_bytes = _read_u16_slice(
        source_root / SOURCE_VALIDATION_RELATIVE_PATH,
        offset_tokens=VALIDATION_SOURCE_OFFSET_TOKENS,
        token_count=VALIDATION_TOKENS,
    )
    if cid_bytes(train_bytes) != EXPECTED_TRAIN_SLICE_CID:
        raise ValueError("training slice CID differs from the independent freeze")
    if cid_bytes(validation_bytes) != EXPECTED_VALIDATION_SLICE_CID:
        raise ValueError("validation slice CID differs from the independent freeze")
    tokenizer_bytes = _read_verified_file(
        source_root / SOURCE_TOKENIZER_RELATIVE_PATH,
        expected_cid=EXPECTED_TOKENIZER_CID,
        label="#1019 tokenizer",
    )
    geometry_bytes = _read_verified_file(
        geometry_path,
        expected_cid=EXPECTED_GEOMETRY_FILE_CID,
        label="canonical exact geometry",
    )

    output_root.parent.mkdir(parents=True, exist_ok=True)
    staging_root = Path(
        tempfile.mkdtemp(
            prefix=f".{output_root.name}.preparing-",
            dir=output_root.parent,
        )
    )
    try:
        atomic_write(staging_root / TRAIN_RELATIVE_PATH, train_bytes)
        atomic_write(staging_root / VALIDATION_RELATIVE_PATH, validation_bytes)
        atomic_write(staging_root / TOKENIZER_RELATIVE_PATH, tokenizer_bytes)
        atomic_write(staging_root / GEOMETRY_RELATIVE_PATH, geometry_bytes)
        write_bound_manifest(
            staging_root / DATA_MANIFEST_NAME,
            _payload(),
            artifact_root=staging_root,
            relative_paths=COPIED_ARTIFACT_PATHS,
        )
        load_language_path_preparation(
            staging_root,
            _geometry_loader=_geometry_loader,
        )
        if output_root.exists() or output_root.is_symlink():
            raise FileExistsError("language-path data root appeared during preparation")
        staging_root.rename(output_root)
    except BaseException:
        if staging_root.exists() and not staging_root.is_symlink():
            shutil.rmtree(staging_root)
        raise
    return load_language_path_preparation(
        output_root,
        _geometry_loader=_geometry_loader,
    )
