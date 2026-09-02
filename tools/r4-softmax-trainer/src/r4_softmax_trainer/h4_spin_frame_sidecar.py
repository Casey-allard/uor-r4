"""Strict loader for Rust's canonical 120-frame H4 compiler sidecar."""

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


SCHEMA = 1
DOMAIN = "uor-r4.h4-spin-frame-sidecar/1"
GROUP_SIZE = 120
FRAME_WIDTH = 4
PRODUCT_COUNT = GROUP_SIZE * GROUP_SIZE
ROOT_TABLE_KAPPA = (
    "blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76"
)
PRODUCT_TABLE_KAPPA = (
    "blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759"
)
ROOT_COORDINATE_CONVENTION = (
    "fixed canonical H4 order; quaternion basis (1,i,j,k); each coordinate is "
    "(a,b) for (a+b*phi)/2"
)
MATRIX_CONVENTION = (
    "row-major left-quaternion decode matrix F: local R4 -> model R4; transport "
    "T(a->b)=transpose(F_b)*F_a"
)
TRANSPORT_CONTROL_POLICY = (
    "identity-fixing deterministic rotation within each exact H4 element-order class; "
    "candidate leaves remain true and only transport actions use pi(leaf)"
)

_CID_PATTERN = re.compile(r"blake3:[0-9a-f]{64}\Z")
_TOP_KEYS = (
    "schema",
    "domain",
    "frame_count",
    "frame_width",
    "product_count",
    "root_coordinate_convention",
    "matrix_convention",
    "h4_root_table_kappa",
    "h4_multiplication_table_kappa",
    "identity_index",
    "root_coordinates",
    "frame_matrix_f64_bits",
    "inverse_indices",
    "multiplication_indices",
    "connection_control_policy",
    "connection_control_source_cid",
    "connection_control_permutation",
    "artifact_cid",
)


def _object_without_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"H4 spin-frame JSON repeats field {key!r}")
        result[key] = value
    return result


def _reject_json_constant(value: str) -> None:
    raise ValueError(f"H4 spin-frame JSON contains non-finite constant {value}")


def _integer(value: object, *, path: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ValueError(f"{path} must be an integer")
    return value


def _integer_vector(
    value: object, *, length: int, upper_bound: int, path: str
) -> tuple[int, ...]:
    if not isinstance(value, list) or len(value) != length:
        raise ValueError(f"{path} must contain exactly {length} entries")
    result = tuple(
        _integer(item, path=f"{path}[{offset}]")
        for offset, item in enumerate(value)
    )
    if any(item < 0 or item >= upper_bound for item in result):
        raise ValueError(f"{path} contains an out-of-range index")
    return result


def _rust_json_bytes(value: object) -> bytes:
    """Mirror serde_json::to_vec for this ASCII, integer-only schema."""
    return json.dumps(
        value, ensure_ascii=False, allow_nan=False, separators=(",", ":")
    ).encode("utf-8")


def _cid_bytes(value: bytes) -> str:
    return f"blake3:{blake3(value).hexdigest()}"


def _root_coordinates(value: object) -> tuple[tuple[tuple[int, int], ...], ...]:
    if not isinstance(value, list) or len(value) != GROUP_SIZE:
        raise ValueError("root_coordinates must contain exactly 120 roots")
    roots: list[tuple[tuple[int, int], ...]] = []
    for root_offset, root in enumerate(value):
        if not isinstance(root, list) or len(root) != FRAME_WIDTH:
            raise ValueError(f"root_coordinates[{root_offset}] must have four coordinates")
        coordinates: list[tuple[int, int]] = []
        for coordinate_offset, coordinate in enumerate(root):
            if not isinstance(coordinate, list) or len(coordinate) != 2:
                raise ValueError(
                    f"root_coordinates[{root_offset}][{coordinate_offset}] must be an (a,b) pair"
                )
            coordinates.append(
                (
                    _integer(
                        coordinate[0],
                        path=f"root_coordinates[{root_offset}][{coordinate_offset}][0]",
                    ),
                    _integer(
                        coordinate[1],
                        path=f"root_coordinates[{root_offset}][{coordinate_offset}][1]",
                    ),
                )
            )
        roots.append(tuple(coordinates))
    return tuple(roots)


def _matrix_bits(value: object) -> np.ndarray:
    if not isinstance(value, list) or len(value) != GROUP_SIZE:
        raise ValueError("frame_matrix_f64_bits must contain exactly 120 matrices")
    flat: list[int] = []
    for frame_offset, matrix in enumerate(value):
        if not isinstance(matrix, list) or len(matrix) != FRAME_WIDTH:
            raise ValueError(f"frame_matrix_f64_bits[{frame_offset}] must have four rows")
        for row_offset, row in enumerate(matrix):
            if not isinstance(row, list) or len(row) != FRAME_WIDTH:
                raise ValueError(
                    f"frame_matrix_f64_bits[{frame_offset}][{row_offset}] must have four entries"
                )
            for column_offset, item in enumerate(row):
                bits = _integer(
                    item,
                    path=(
                        f"frame_matrix_f64_bits[{frame_offset}]"
                        f"[{row_offset}][{column_offset}]"
                    ),
                )
                if not 0 <= bits < 1 << 64:
                    raise ValueError("frame matrix contains an out-of-range f64 bit pattern")
                flat.append(bits)
    bits = np.asarray(flat, dtype=np.uint64).reshape(
        GROUP_SIZE, FRAME_WIDTH, FRAME_WIDTH
    )
    matrices = bits.view(np.float64)
    if not np.isfinite(matrices).all():
        raise ValueError("registered H4 frame contains a non-finite value")
    return matrices


def _validate_matrices(
    matrices: np.ndarray,
    roots: tuple[tuple[tuple[int, int], ...], ...],
    products: tuple[int, ...],
    identity: int,
) -> None:
    eye = np.eye(FRAME_WIDTH, dtype=np.float64)
    orthogonality = np.matmul(np.swapaxes(matrices, 1, 2), matrices)
    if not np.allclose(orthogonality, eye[None, :, :], rtol=0.0, atol=1.0e-12):
        raise ValueError("registered H4 frames are not orthogonal")
    if not np.array_equal(matrices[identity], eye):
        raise ValueError("declared H4 identity frame is not the exact identity matrix")

    root_array = np.asarray(roots, dtype=np.float64)
    phi = (1.0 + np.sqrt(5.0)) * 0.5
    quaternions = (root_array[:, :, 0] + root_array[:, :, 1] * phi) * 0.5
    quaternions /= np.linalg.norm(quaternions, axis=1, keepdims=True)
    w, x, y, z = (quaternions[:, offset] for offset in range(FRAME_WIDTH))
    expected = np.stack(
        (
            w,
            -x,
            -y,
            -z,
            x,
            w,
            -z,
            y,
            y,
            z,
            w,
            -x,
            z,
            -y,
            x,
            w,
        ),
        axis=1,
    ).reshape(GROUP_SIZE, FRAME_WIDTH, FRAME_WIDTH)
    if not np.allclose(matrices, expected, rtol=0.0, atol=2.0e-15):
        raise ValueError("registered matrices do not reproduce the bound H4 coordinates")

    table = np.asarray(products, dtype=np.int64).reshape(GROUP_SIZE, GROUP_SIZE)
    composed = np.matmul(matrices[:, None, :, :], matrices[None, :, :, :])
    if not np.allclose(composed, matrices[table], rtol=0.0, atol=2.0e-12):
        raise ValueError("registered matrices do not realize all 14,400 exact products")


@dataclass(frozen=True, slots=True)
class H4SpinFrameArtifactV1:
    """Validated tensors needed by the predictive R4 delta cell."""

    frame_matrices: Tensor
    multiplication_indices: Tensor
    inverse_indices: Tensor
    transport_permutation: Tensor
    identity_index: int
    root_coordinates: tuple[tuple[tuple[int, int], ...], ...]
    h4_root_table_kappa: str
    h4_multiplication_table_kappa: str
    matrix_convention: str
    transport_control_source_cid: str
    artifact_cid: str
    file_cid: str

    @classmethod
    def load(cls, path: Path) -> H4SpinFrameArtifactV1:
        path = path.resolve()
        if path.is_symlink() or not path.is_file():
            raise ValueError("H4 spin-frame sidecar must be a regular, non-symlink file")
        return cls.from_bytes(path.read_bytes())

    @classmethod
    def from_bytes(cls, raw: bytes) -> H4SpinFrameArtifactV1:
        try:
            value = json.loads(
                raw.decode("utf-8", errors="strict"),
                object_pairs_hook=_object_without_duplicate_keys,
                parse_constant=_reject_json_constant,
            )
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ValueError("H4 spin-frame sidecar is not strict UTF-8 JSON") from error
        if not isinstance(value, dict) or tuple(value) != _TOP_KEYS:
            raise ValueError("H4 spin-frame sidecar fields differ from the frozen schema")
        if _rust_json_bytes(value) != raw:
            raise ValueError("H4 spin-frame sidecar bytes are not canonical Rust JSON")
        if (
            _integer(value["schema"], path="schema") != SCHEMA
            or value["domain"] != DOMAIN
            or _integer(value["frame_count"], path="frame_count") != GROUP_SIZE
            or _integer(value["frame_width"], path="frame_width") != FRAME_WIDTH
            or _integer(value["product_count"], path="product_count") != PRODUCT_COUNT
            or value["root_coordinate_convention"] != ROOT_COORDINATE_CONVENTION
            or value["matrix_convention"] != MATRIX_CONVENTION
            or value["h4_root_table_kappa"] != ROOT_TABLE_KAPPA
            or value["h4_multiplication_table_kappa"] != PRODUCT_TABLE_KAPPA
            or value["connection_control_policy"] != TRANSPORT_CONTROL_POLICY
        ):
            raise ValueError("H4 spin-frame schema or canonical provenance differs")

        identity = _integer(value["identity_index"], path="identity_index")
        if not 0 <= identity < GROUP_SIZE:
            raise ValueError("H4 identity index is outside the group")
        inverses = _integer_vector(
            value["inverse_indices"],
            length=GROUP_SIZE,
            upper_bound=GROUP_SIZE,
            path="inverse_indices",
        )
        products = _integer_vector(
            value["multiplication_indices"],
            length=PRODUCT_COUNT,
            upper_bound=GROUP_SIZE,
            path="multiplication_indices",
        )
        permutation = _integer_vector(
            value["connection_control_permutation"],
            length=GROUP_SIZE,
            upper_bound=GROUP_SIZE,
            path="connection_control_permutation",
        )
        if tuple(sorted(permutation)) != tuple(range(GROUP_SIZE)) or permutation[identity] != identity:
            raise ValueError("transport control is not an identity-fixing permutation")

        table = np.asarray(products, dtype=np.int64).reshape(GROUP_SIZE, GROUP_SIZE)
        expected = np.arange(GROUP_SIZE, dtype=np.int64)
        if not np.array_equal(table[identity], expected) or not np.array_equal(
            table[:, identity], expected
        ):
            raise ValueError("declared H4 identity does not act exactly")
        if any(tuple(sorted(row.tolist())) != tuple(range(GROUP_SIZE)) for row in table):
            raise ValueError("H4 multiplication table contains a non-permutation row")
        for element, inverse in enumerate(inverses):
            if table[element, inverse] != identity or table[inverse, element] != identity:
                raise ValueError(f"H4 inverse table fails at element {element}")

        mapped_product = np.asarray(permutation, dtype=np.int64)[table]
        product_of_mapped = table[
            np.asarray(permutation, dtype=np.int64)[:, None],
            np.asarray(permutation, dtype=np.int64)[None, :],
        ]
        if np.array_equal(mapped_product, product_of_mapped):
            raise ValueError("transport permutation does not destroy the canonical connection")

        roots = _root_coordinates(value["root_coordinates"])
        matrices_f64 = _matrix_bits(value["frame_matrix_f64_bits"])
        _validate_matrices(matrices_f64, roots, products, identity)

        artifact_cid = value["artifact_cid"]
        control_source_cid = value["connection_control_source_cid"]
        if not isinstance(artifact_cid, str) or not _CID_PATTERN.fullmatch(artifact_cid):
            raise ValueError("H4 spin-frame artifact CID is invalid")
        if not isinstance(control_source_cid, str) or not _CID_PATTERN.fullmatch(control_source_cid):
            raise ValueError("H4 transport-control source CID is invalid")
        seed = copy.deepcopy(value)
        seed["artifact_cid"] = ""
        if artifact_cid != _cid_bytes(_rust_json_bytes(seed)):
            raise ValueError("H4 spin-frame artifact CID does not reproduce")

        artifact = cls(
            frame_matrices=torch.from_numpy(matrices_f64.copy()).to(torch.float32).contiguous(),
            multiplication_indices=torch.from_numpy(table.copy()).to(torch.long).contiguous(),
            inverse_indices=torch.tensor(inverses, dtype=torch.long),
            transport_permutation=torch.tensor(permutation, dtype=torch.long),
            identity_index=identity,
            root_coordinates=roots,
            h4_root_table_kappa=ROOT_TABLE_KAPPA,
            h4_multiplication_table_kappa=PRODUCT_TABLE_KAPPA,
            matrix_convention=MATRIX_CONVENTION,
            transport_control_source_cid=control_source_cid,
            artifact_cid=artifact_cid,
            file_cid=_cid_bytes(raw),
        )
        artifact.validate()
        return artifact

    def validate(self, *, group_size: int = GROUP_SIZE) -> None:
        if group_size != GROUP_SIZE:
            raise ValueError("H4 spin-frame sidecar has one frozen 120-frame contract")
        for name, tensor, shape, dtype in (
            ("frame_matrices", self.frame_matrices, (120, 4, 4), torch.float32),
            ("multiplication_indices", self.multiplication_indices, (120, 120), torch.long),
            ("inverse_indices", self.inverse_indices, (120,), torch.long),
            ("transport_permutation", self.transport_permutation, (120,), torch.long),
        ):
            if tensor.device.type != "cpu" or tensor.dtype != dtype or tuple(tensor.shape) != shape:
                raise ValueError(f"{name} differs from the frozen CPU tensor contract")
        if not torch.isfinite(self.frame_matrices).all():
            raise ValueError("frame_matrices contains a non-finite value")
        eye = torch.eye(4, dtype=torch.float32)
        if not torch.equal(self.frame_matrices[self.identity_index], eye):
            raise ValueError("identity frame changed after loading")
        observed = torch.matmul(
            self.frame_matrices[:, None], self.frame_matrices[None, :]
        )
        expected = self.frame_matrices[self.multiplication_indices]
        if not torch.allclose(observed, expected, rtol=0.0, atol=2.0e-6):
            raise ValueError("f32 frame matrices no longer realize the bound product table")


def load_h4_spin_frame_sidecar(path: Path) -> H4SpinFrameArtifactV1:
    return H4SpinFrameArtifactV1.load(path)
