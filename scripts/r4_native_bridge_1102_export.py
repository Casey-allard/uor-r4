"""Offline, constructor-free export for the separately admitted #1102 bridge.

``export_artifact(asset_paths, release, release_sha256)`` returns one complete
container and its ExpectedBinding. It writes nothing and has no CLI. Calling it
reads source and asset bytes and therefore belongs inside the reviewed export /
integrity envelope; importing this module performs no explicit source or asset
access. The coordinator must
construct twice independently, compare complete bytes, and publish to separate
exclusive destinations. Merely writing one returned buffer twice is not two
exports.

Required asset_paths keys (absolute regular files): reader, core, vocabulary,
h4_frames, token_frames, policy. The outer release requires issue=1102 and the
``export`` object below; its other fields remain covered by release_sha256.
That digest is SHA256 of sorted compact UTF-8 JSON plus one trailing LF.

The export object has exactly:
  repo_root: absolute canonical repository directory
  contract_sha256: frozen #1086 contract digest
  source_revision, exporter_revision: reviewed 40-character Git revisions
  exporter_sources: sorted SourceFile records with repository, revision, path,
                    sha256 and bytes; repository-relative paths, no aliases
  exporter_runtime: the exact metadata-only runtime_identity() string
  exporter_lock_path: repository-relative Python dependency lockfile path
  exporter_lock_sha256: digest of that source-closure member

The source closure must include the exporter, native implementation and facade,
current #1102 scripts, Cargo manifests/lock, toolchain, contract/prose, and the
declared exporter lockfile. It is verified before asset access and again before
return. Source Git revisions are reviewed provenance: this function never runs
Git or substitutes a live branch name for a revision.

Safetensors decoding examines only its JSON header and raw F32 bytes. Finite
checks inspect exponent bits, never decode values to Python floats. Frame f64
bytes come directly from original u64 bit patterns. No Torch import, model
constructor, RNG, fit, numerical forward, vocabulary replacement or frame
regeneration is available here.
"""

from __future__ import annotations

import copy
import hashlib
import json
import os
import platform
import re
import stat
import struct
import sys
from pathlib import Path

from blake3 import blake3

__all__ = ["ExportError", "canonical_ascii", "export_artifact", "runtime_identity"]

_CONTRACT_SHA256 = "e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115"
_CONTRACT_PATH = "docs/r4_native_reference_1086_contract.json"
_EXPORTER_PATH = "scripts/r4_native_bridge_1102_export.py"
_REPOSITORY = "UOR-Foundation/uor-r4"
_PROFILE = "cpu-scalar-f32-f64-1086/1"
_ASSETS = {"reader", "core", "vocabulary", "h4_frames", "token_frames", "policy"}
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")
_GIT40 = re.compile(r"[0-9a-f]{40}\Z")
_CID = re.compile(r"blake3:[0-9a-f]{64}\Z")
_U64_MAX = (1 << 64) - 1
_EXPORT_FIELDS = {
    "repo_root", "contract_sha256", "source_revision", "exporter_revision",
    "exporter_sources", "exporter_runtime", "exporter_lock_path",
    "exporter_lock_sha256",
}
_SOURCE_FIELDS = {"repository", "revision", "path", "sha256", "bytes"}


class ExportError(ValueError):
    """Incomplete or mismatched export admission; no artifact is returned."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ExportError(message)


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _cid(raw: bytes) -> str:
    return "blake3:" + blake3(raw).hexdigest()


def _historical_json(value: object) -> bytes:
    """The existing state / frame-tree / release recipe, including its LF."""
    try:
        return (
            json.dumps(
                value, ensure_ascii=False, allow_nan=False, sort_keys=True,
                separators=(",", ":"),
            ) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError, UnicodeError) as error:
        raise ExportError("record cannot use the declared canonical JSON") from error


def canonical_ascii(value: object) -> bytes:
    """ascii-json-1086/1; no LF, floats, Unicode or JSON shortcut escapes."""
    def emit(item: object) -> str:
        if item is None:
            return "null"
        if type(item) is bool:
            return "true" if item else "false"
        if type(item) is int:
            _require(0 <= item <= _U64_MAX, "manifest integer is outside u64")
            return str(item)
        if type(item) is str:
            _require(item.isascii(), "manifest string is not ASCII")
            parts = ['"']
            for character in item:
                code = ord(character)
                if character == '"':
                    parts.append('\\"')
                elif character == "\\":
                    parts.append("\\\\")
                elif code < 32:
                    parts.append(f"\\u{code:04x}")
                else:
                    parts.append(character)
            parts.append('"')
            return "".join(parts)
        if type(item) is list:
            return "[" + ",".join(emit(child) for child in item) + "]"
        if type(item) is dict:
            _require(all(type(key) is str for key in item), "manifest key is not a string")
            return "{" + ",".join(
                emit(key) + ":" + emit(item[key]) for key in sorted(item)
            ) + "}"
        raise ExportError("manifest contains an unsupported value type")

    return emit(value).encode("ascii")


def runtime_identity() -> str:
    """Python/host metadata only; no model package or subprocess is loaded."""
    return canonical_ascii({
        "byteorder": sys.byteorder,
        "implementation": sys.implementation.name,
        "machine": platform.machine(),
        "platform": sys.platform,
        "python": platform.python_version(),
    }).decode("ascii")


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        _require(key not in result, "duplicate JSON field")
        result[key] = value
    return result


def _no_float(_value: str) -> None:
    raise ExportError("source JSON contains a floating or nonfinite number")


def _json_object(raw: bytes, label: str) -> dict:
    try:
        value = json.loads(
            raw.decode("utf-8"), object_pairs_hook=_unique_object,
            parse_float=_no_float, parse_constant=_no_float,
        )
    except (ValueError, UnicodeError) as error:
        raise ExportError(f"invalid {label} JSON") from error
    _require(type(value) is dict, f"{label} is not a JSON object")
    return value


def _uint(value: object, upper: int, label: str) -> int:
    _require(type(value) is int and 0 <= value <= upper, f"invalid {label} integer")
    return value


def _hex(value: object, pattern: re.Pattern, label: str) -> str:
    _require(type(value) is str and pattern.fullmatch(value) is not None, f"invalid {label}")
    return value


def _relative_path(value: object) -> str:
    _require(type(value) is str and value.isascii() and value != "", "invalid source path")
    _require(
        not value.startswith("/") and "\\" not in value
        and all(part not in ("", ".", "..") for part in value.split("/")),
        "source path is not a canonical repository-relative path",
    )
    return value


def _read_bound(path: Path, expected: dict, label: str) -> bytes:
    """Read one fixed-size regular file through one descriptor, never a link."""
    count = _uint(expected.get("bytes"), _U64_MAX, "file byte count")
    digest = _hex(expected.get("sha256"), _HEX64, "file SHA256")
    _require(path.is_absolute(), f"{label} path must be absolute")
    try:
        _require(path.resolve(strict=True) == path, f"{label} path contains an alias")
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        with os.fdopen(descriptor, "rb") as handle:
            before = os.fstat(handle.fileno())
            _require(stat.S_ISREG(before.st_mode), f"{label} is not a regular file")
            _require(before.st_size == count, f"{label} byte length differs")
            raw = handle.read(count + 1)
            after = os.fstat(handle.fileno())
    except OSError as error:
        raise ExportError(f"cannot read admitted {label}") from error
    _require(
        (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns)
        == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns),
        f"{label} changed while being read",
    )
    _require(len(raw) == count and _sha256(raw) == digest, f"{label} identity differs")
    if "cid" in expected:
        _hex(expected["cid"], _CID, "file CID")
        _require(_cid(raw) == expected["cid"], f"{label} BLAKE3 file CID differs")
    return raw


def _required_sources(root: Path, lock_path: str) -> set[str]:
    required = {
        _EXPORTER_PATH, _CONTRACT_PATH, "docs/r4_native_reference_1086.md",
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
        "crates/uor-r4-core/Cargo.toml", "crates/uor-r4-core/src/lib.rs",
        "crates/uor-r4-api/Cargo.toml", "crates/uor-r4-api/src/lib.rs",
        "crates/uor-r4-api/src/learned_reference.rs", lock_path,
    }
    for path in (root / "crates/uor-r4-core/src/learned_reference").rglob("*.rs"):
        required.add(path.relative_to(root).as_posix())
    for path in (root / "scripts").glob("r4_native_bridge_1102*.py"):
        required.add(path.relative_to(root).as_posix())
    # Include any native bridge bin/example present when its release is frozen.
    for crate in ("uor-r4-core", "uor-r4-api"):
        for path in (root / "crates" / crate).rglob("*.rs"):
            relative = path.relative_to(root).as_posix()
            normalized = path.name.replace("-", "_")
            if "native_reference" in normalized or "native_bridge" in normalized or "learned_reference" in normalized:
                required.add(relative)
    cargo_config = root / ".cargo/config.toml"
    if cargo_config.exists():
        required.add(".cargo/config.toml")
    return required


def _source_admission(release: dict, release_sha256: str) -> tuple[Path, dict, dict[str, bytes]]:
    _require(
        type(release) is dict and type(release.get("issue")) is int
        and release["issue"] == 1102,
        "release does not name #1102",
    )
    _hex(release_sha256, _HEX64, "release SHA256")
    _require(_sha256(_historical_json(release)) == release_sha256, "canonical release digest differs")
    section = release.get("export")
    _require(type(section) is dict and set(section) == _EXPORT_FIELDS, "export release fields differ")
    _require(section["contract_sha256"] == _CONTRACT_SHA256, "export release names another contract")
    for key in ("source_revision", "exporter_revision"):
        _hex(section[key], _GIT40, key)
    _require(section["exporter_runtime"] == runtime_identity(), "exporter Python metadata differs")
    _require(sys.byteorder == "little", "exporter requires the admitted little-endian host")
    root_text = section["repo_root"]
    _require(type(root_text) is str and root_text.isascii(), "invalid repository root")
    root = Path(root_text)
    _require(root.is_absolute() and root.resolve(strict=True) == root, "repository root is not canonical")
    _require(Path(__file__).resolve() == root / _EXPORTER_PATH, "executing exporter is outside the bound root")
    lock_path = _relative_path(section["exporter_lock_path"])
    _hex(section["exporter_lock_sha256"], _HEX64, "exporter lock SHA256")
    records = section["exporter_sources"]
    _require(type(records) is list and bool(records), "export source closure is absent")
    source_bytes = {}
    paths = []
    for record in records:
        _require(type(record) is dict and set(record) == _SOURCE_FIELDS, "source record fields differ")
        _require(record["repository"] == _REPOSITORY, "source repository differs")
        _hex(record["revision"], _GIT40, "source revision")
        _require(
            record["revision"] in (section["source_revision"], section["exporter_revision"]),
            "source closure revision is not bound by the export release",
        )
        relative = _relative_path(record["path"])
        paths.append(relative)
        _require(relative not in source_bytes, "duplicate source closure path")
        source_bytes[relative] = _read_bound(root / relative, record, "source closure member")
    _require(paths == sorted(paths), "source closure is not sorted by path")
    _require(_required_sources(root, lock_path).issubset(source_bytes), "source closure omits implementation/exporter files")
    _require(_sha256(source_bytes[lock_path]) == section["exporter_lock_sha256"], "exporter lock identity differs")
    _require(_sha256(source_bytes[_CONTRACT_PATH]) == _CONTRACT_SHA256, "frozen contract bytes differ")
    return root, section, source_bytes


def _f32_tensors(raw: bytes, templates: list[dict], owner: str) -> dict[str, bytes]:
    """Validate the Safetensors header and copy original F32 payload slices."""
    _require(len(raw) >= 8, f"{owner} Safetensors header is truncated")
    header_count = struct.unpack_from("<Q", raw, 0)[0]
    _require(0 < header_count <= len(raw) - 8 and header_count % 8 == 0, "invalid Safetensors header length")
    header_end = 8 + header_count
    header = _json_object(raw[8:header_end], f"{owner} Safetensors header")
    metadata = header.pop("__metadata__", None)
    if metadata is not None:
        _require(
            type(metadata) is dict
            and all(type(key) is str and type(value) is str for key, value in metadata.items()),
            "invalid Safetensors string metadata",
        )
    prefix = owner + "."
    expected = {item["name"][len(prefix):]: item for item in templates if item["name"].startswith(prefix)}
    _require(set(header) == set(expected), f"{owner} serialized tensor inventory differs")
    body = raw[header_end:]
    spans, result = [], {}
    for name in sorted(expected):
        item = header[name]
        template = expected[name]
        _require(type(item) is dict and set(item) == {"dtype", "shape", "data_offsets"}, "Safetensors tensor fields differ")
        _require(item["dtype"] == "F32" and item["shape"] == template["shape"], f"{owner} tensor dtype/shape differs")
        _require(
            type(item["shape"]) is list
            and all(type(dimension) is int and dimension > 0 for dimension in item["shape"]),
            "invalid Safetensors shape integers",
        )
        offsets = item["data_offsets"]
        _require(type(offsets) is list and len(offsets) == 2, "invalid Safetensors offset pair")
        start = _uint(offsets[0], len(body), "tensor offset")
        end = _uint(offsets[1], len(body), "tensor offset")
        _require(end >= start and end - start == template["bytes"], "Safetensors tensor byte span differs")
        payload = body[start:end]
        for (bits,) in struct.iter_unpack("<I", payload):
            _require(bits & 0x7F800000 != 0x7F800000, "Safetensors tensor contains a nonfinite value")
        spans.append((start, end))
        result[name] = payload
    end = 0
    for start, next_end in sorted(spans):
        _require(start == end, "Safetensors payload overlaps or has a gap")
        end = next_end
    _require(end == len(body), "Safetensors payload has an unclaimed trailer")
    return result


def _state_cid(tensors: dict[str, bytes], templates: list[dict], owner: str) -> str:
    shapes = {
        item["name"][len(owner) + 1:]: item["shape"]
        for item in templates if item["name"].startswith(owner + ".")
    }
    digest = blake3()
    for name in sorted(tensors):
        digest.update(_historical_json({"name": name, "shape": shapes[name], "dtype": "torch.float32"}))
        digest.update(tensors[name])
    return "blake3:" + digest.hexdigest()


def _shape_integers(value: object, shape: tuple[int, ...], upper: int, label: str) -> list[int]:
    if not shape:
        return [_uint(value, upper, label)]
    _require(type(value) is list and len(value) == shape[0], f"{label} shape differs")
    result = []
    for child in value:
        result.extend(_shape_integers(child, shape[1:], upper, label))
    return result


def _frame_json_identity(raw: bytes, value: dict) -> None:
    # Native frame files preserve their original field order and have no LF.
    canonical = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode("utf-8")
    _require(canonical == raw, "original frame JSON canonical bytes differ")
    artifact_cid = _hex(value.get("artifact_cid"), _CID, "frame artifact CID")
    seed = dict(value)
    seed["artifact_cid"] = ""
    seed_bytes = json.dumps(seed, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode("utf-8")
    _require(_cid(seed_bytes) == artifact_cid, "frame artifact CID does not reproduce")


def _frame_components(assets: dict[str, bytes], binding: dict) -> tuple[int, dict[str, bytes]]:
    h4_raw, token_raw = assets["h4_frames"], assets["token_frames"]
    records = [
        {"path": "h4-frames.json", "bytes": len(h4_raw), "cid": _cid(h4_raw)},
        {"path": "token-frames.json", "bytes": len(token_raw), "cid": _cid(token_raw)},
    ]
    _require(_cid(_historical_json(records)) == binding["frame_tree_cid"], "historical frame tree CID differs")
    h4, tokens = _json_object(h4_raw, "H4 frame"), _json_object(token_raw, "token frame")
    _frame_json_identity(h4_raw, h4)
    _frame_json_identity(token_raw, tokens)
    _require(type(h4.get("schema")) is int and h4["schema"] == 1, "H4 frame schema differs")
    _require(h4.get("domain") == "uor-r4.h4-spin-frame-sidecar/1", "H4 frame domain differs")
    _require(tokens.get("schema") == "uor-r4.zoology-r4-token-frames/1", "token frame schema differs")
    identity = _uint(h4.get("identity_index"), 119, "H4 identity")
    _require(_uint(tokens.get("identity_index"), 119, "token identity") == identity, "frame identities disagree")
    _require(tokens.get("frame_artifact_cid") == h4["artifact_cid"], "token map binds another frame artifact")
    _require(tokens.get("frame_file_cid") == _cid(h4_raw), "token map binds another frame file")
    _require(_uint(tokens.get("maximum_token_id"), 8191, "maximum token ID") == 8191, "token map coverage differs")
    bits = _shape_integers(h4.get("frame_matrix_f64_bits"), (120, 4, 4), _U64_MAX, "frame bit matrix")
    for bit_pattern in bits:
        _require(bit_pattern & 0x7FF0000000000000 != 0x7FF0000000000000, "frame matrix contains a nonfinite value")
    multiplication = _shape_integers(h4.get("multiplication_indices"), (14400,), 119, "multiplication table")
    leaves = _shape_integers(tokens.get("token_leaf_indices"), (8192,), 119, "token leaf map")
    _require(leaves[0] == identity, "token zero is not the native identity")
    for element in range(120):
        _require(
            multiplication[identity * 120 + element] == element
            and multiplication[element * 120 + identity] == element,
            "multiplication identity does not reproduce",
        )
    witnesses = tokens.get("prefix_witnesses")
    _require(type(witnesses) is list and len(witnesses) == 3, "prefix witness count differs")
    reached, maximum_present = set(), False
    for witness in witnesses:
        _require(type(witness) is dict and set(witness) == {"tokens", "frame_indices"}, "prefix witness fields differ")
        ids, indices = witness["tokens"], witness["frame_indices"]
        _require(type(ids) is list and type(indices) is list and 1 <= len(ids) <= 8 and len(ids) == len(indices), "prefix witness lengths differ")
        current = identity
        for token_id, expected_index in zip(ids, indices, strict=True):
            token_id = _uint(token_id, 8191, "witness token")
            expected_index = _uint(expected_index, 119, "witness frame")
            maximum_present |= token_id == 8191
            current = multiplication[current * 120 + leaves[token_id]]
            _require(current == expected_index, "native prefix witness does not reproduce")
            reached.add(expected_index)
    _require(maximum_present, "prefix witnesses omit the maximum token ID")
    _require(_uint(tokens.get("direct_leaf_count"), 120, "direct leaf count") == len(set(leaves)), "direct leaf count differs")
    _require(_uint(tokens.get("witness_frame_count"), 120, "witness frame count") == len(reached), "witness frame count differs")
    return identity, {
        "frames": b"".join(struct.pack("<Q", value) for value in bits),
        "multiplication": b"".join(struct.pack("<q", value) for value in multiplication),
        "token_leaves": b"".join(struct.pack("<q", value) for value in leaves),
    }


def _validate_codec(assets: dict[str, bytes], binding: dict) -> None:
    vocabulary = _json_object(assets["vocabulary"], "vocabulary")
    policy = _json_object(assets["policy"], "policy")
    _require(policy.get("schema") == "uor-r4.text-to-clauses-policy/1", "raw-text policy schema differs")
    lexical, limits = policy.get("lexical_artifact"), policy.get("limits")
    _require(type(lexical) is dict and type(limits) is dict, "raw-text policy metadata is absent")
    prefix = lexical.get("reader_prefix_by_id")
    _require(type(prefix) is list and len(prefix) == 58 and all(type(word) is str and word.isascii() for word in prefix), "reader lexical prefix differs")
    _require(lexical.get("cid") == binding["assets"]["vocabulary"]["cid"], "policy vocabulary CID differs")
    _require(type(vocabulary.get("padding_id")) is int and vocabulary["padding_id"] == 57 and limits.get("padding_id") == 57, "padding identity differs")
    # These are comparisons only; the payload always retains original bytes.
    expected_core = prefix[:52] + [f"<unused-{index:04d}>" for index in range(52, 4096)]
    expected_reader = prefix + expected_core[58:]
    _require(vocabulary.get("core_vocabulary") == expected_core, "core output vocabulary differs")
    _require(vocabulary.get("vocabulary") == expected_reader, "reader lexical aliases differ")


def _native_state(components: list[dict], payloads: list[bytes], identity: int) -> str:
    digest = hashlib.sha256()

    def string(value: str) -> None:
        raw = value.encode("ascii")
        digest.update(struct.pack("<I", len(raw)))
        digest.update(raw)

    string("uor-r4.native-reference-state/1")
    digest.update(struct.pack("<I", len(components)))
    for component, raw in zip(components, payloads, strict=True):
        for key in ("name", "kind", "dtype"):
            string(component[key])
        digest.update(struct.pack("<I", len(component["shape"])))
        for dimension in component["shape"]:
            digest.update(struct.pack("<Q", dimension))
        digest.update(struct.pack("<Q", component["bytes"]))
        digest.update(raw)
    digest.update(struct.pack("<I", identity))
    string(_PROFILE)
    return digest.hexdigest()


def export_artifact(asset_paths: dict, release: dict, release_sha256: str) -> tuple[bytes, dict]:
    """Read admitted originals and return one deterministic artifact; no writes."""
    _require(type(asset_paths) is dict and set(asset_paths) == _ASSETS, "asset path keys differ")
    root, section, source_bytes = _source_admission(release, release_sha256)
    contract = _json_object(source_bytes[_CONTRACT_PATH], "frozen contract")
    _require(contract.get("schema") == "uor-r4.native-reference-contract/1" and contract.get("issue") == 1086, "frozen contract identity differs")
    binding = contract["accepted_binding"]
    templates = contract["components"]
    assets = {}
    for name in sorted(_ASSETS):
        if name == "policy":
            policy_component = next(item for item in templates if item["name"] == "policy.json")
            expected = {"bytes": policy_component["bytes"], "sha256": binding["policy_sha256"]}
        else:
            expected = binding["assets"][name]
        assets[name] = _read_bound(Path(asset_paths[name]), expected, name)

    reader = _f32_tensors(assets["reader"], templates[:14], "reader")
    core = _f32_tensors(assets["core"], templates[:14], "core")
    _require(_state_cid(reader, templates[:14], "reader") == binding["reader_state_cid"], "decoded reader state CID differs")
    _require(_state_cid(core, templates[:14], "core") == binding["core_state_cid"], "decoded core state CID differs")
    _validate_codec(assets, binding)
    identity, frame_payloads = _frame_components(assets, binding)
    payload_by_name = {
        **{"reader." + name: raw for name, raw in reader.items()},
        **{"core." + name: raw for name, raw in core.items()},
        "vocabulary.json": assets["vocabulary"],
        "policy.json": assets["policy"],
        "h4-frames.json": assets["h4_frames"],
        "token-frames.json": assets["token_frames"],
        **frame_payloads,
    }
    _require(set(payload_by_name) == {item["name"] for item in templates}, "export component inventory differs")
    components, payloads, offset = [], [], 0
    for template in templates:
        raw = payload_by_name[template["name"]]
        _require(template["offset"] == offset and template["bytes"] == len(raw), "export component offset/length differs")
        components.append({**template, "sha256": _sha256(raw)})
        payloads.append(raw)
        offset += len(raw)
    _require(offset == contract["container"]["payload_bytes"], "complete export payload length differs")
    manifest = {
        "schema": "uor-r4.native-reference-manifest/1",
        "name": "R4LearnedReferenceV1",
        "canonicalization": "ascii-json-1086/1",
        "contract_sha256": _CONTRACT_SHA256,
        "operation": "answer_four_fact_raw_text/v1",
        "operator_profile": _PROFILE,
        "source_binding": copy.deepcopy(binding),
        "export_provenance": {
            "source_revision": section["source_revision"],
            "exporter_revision": section["exporter_revision"],
            "exporter_sources": copy.deepcopy(section["exporter_sources"]),
            "exporter_runtime": section["exporter_runtime"],
            "exporter_lock_sha256": section["exporter_lock_sha256"],
            "release_sha256": release_sha256,
        },
        "components": components,
        "identity_index": identity,
        "native_state_sha256": _native_state(components, payloads, identity),
        "tied_aliases": copy.deepcopy(contract["tied_aliases"]),
    }
    manifest_bytes = canonical_ascii(manifest)
    _require(len(manifest_bytes) <= contract["container"]["maximum_manifest_bytes"], "export manifest exceeds its limit")
    artifact = (
        b"R4LR0001" + struct.pack("<I", len(manifest_bytes)) + manifest_bytes
        + struct.pack("<Q", offset) + b"".join(payloads)
    )
    _require(len(artifact) <= contract["container"]["maximum_bytes"], "export container exceeds its limit")
    # Recheck the exact source closure, not mutable asset paths whose already
    # verified bytes are held in memory. The coordinator owns all publication.
    _, final_section, final_sources = _source_admission(release, release_sha256)
    _require(final_section == section and final_sources == source_bytes, "export sources changed during construction")
    expected_binding = {
        "artifact_sha256": _sha256(artifact),
        "contract_sha256": _CONTRACT_SHA256,
        "accepted_binding": copy.deepcopy(binding),
        "operator_profile": _PROFILE,
        "export_release_sha256": release_sha256,
    }
    return artifact, expected_binding
