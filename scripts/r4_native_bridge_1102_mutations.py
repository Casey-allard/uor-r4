"""Independent, zero-forward loader fixtures for the frozen #1102 gate.

Only ``build_mutations`` consumes bytes, and it performs no I/O. It must not be
called until the coordinator's reviewed export/integrity release admits the
actual exported artifact. The coordinator freezes each returned artifact,
fixture-only ExpectedBinding and expected error before invoking the loader.

There are exactly eleven rejected fixtures, in #1086 loader-stage order. The
valid export load and missing-qualification probe are separate coordinator
work, using one successful gate engine. No inference or model loading occurs
here. Numeric payload changes use integer bit patterns, never float decoding.

Every materialized fixture is a complete artifact copy and counts against the
export/integrity byte ledger, including the 16-MiB-plus-one limit fixture. The
unchanged artifact in the wrong-expected-hash case is still charged if written
to its own fixture path. These synthetic expected hashes are never host trust
anchors or evidence qualifying an export.
"""

from __future__ import annotations

import copy
import hashlib
import json
import struct

__all__ = ["build_mutations"]

_CONTRACT_SHA256 = "e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115"
_PROFILE = "cpu-scalar-f32-f64-1086/1"
_MAGIC = b"R4LR0001"
_MAX_BYTES = 16 * 1024 * 1024
_MAX_MANIFEST_BYTES = 256 * 1024
_PAYLOAD_BYTES = 2_160_742

# Metadata copied from the frozen #1086 contract, not imported from the loader
# or exporter. Component payloads are supplied only by the admitted caller.
_COMPONENTS = (
    ("reader.context.bias", "parameter", "f32le", (64,), 0, 256),
    ("reader.context.weight", "parameter", "f32le", (64, 32, 5), 256, 40960),
    ("reader.embedding.weight", "parameter", "f32le", (4096, 32), 41216, 524288),
    ("reader.role_projection.bias", "parameter", "f32le", (3,), 565504, 12),
    ("reader.role_projection.weight", "parameter", "f32le", (3, 64), 565516, 768),
    ("core.embedding.weight", "parameter", "f32le", (4096, 64), 566284, 1048576),
    ("core.key_projection.weight", "parameter", "f32le", (64, 128), 1614860, 32768),
    ("core.null_key", "parameter", "f32le", (64,), 1647628, 256),
    ("core.null_value", "parameter", "f32le", (64,), 1647884, 256),
    ("core.output_norm.bias", "parameter", "f32le", (64,), 1648140, 256),
    ("core.output_norm.weight", "parameter", "f32le", (64,), 1648396, 256),
    ("core.output_projection.weight", "parameter", "f32le", (64, 64), 1648652, 16384),
    ("core.query_projection.weight", "parameter", "f32le", (64, 128), 1665036, 32768),
    ("core.value_projection.weight", "parameter", "f32le", (64, 64), 1697804, 16384),
    ("vocabulary.json", "original_json", "u8", (130245,), 1714188, 130245),
    ("policy.json", "original_json", "u8", (6769,), 1844433, 6769),
    ("h4-frames.json", "original_json", "u8", (81255,), 1851202, 81255),
    ("token-frames.json", "original_json", "u8", (32189,), 1932457, 32189),
    ("frames", "frame_table", "f64le", (120, 4, 4), 1964646, 15360),
    ("multiplication", "frame_table", "i64le", (120, 120), 1980006, 115200),
    ("token_leaves", "frame_table", "i64le", (8192,), 2095206, 65536),
)
_ACCEPTED_BINDING = {
    "assets": {
        "core": {
            "bytes": 1148672,
            "sha256": "43ffff3c24f8030701e340cab802b985f7c0b7e4e12e270ec1107d141d65b079",
            "cid": "blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4",
        },
        "h4_frames": {
            "bytes": 81255,
            "sha256": "ea9ea1de2f666aff24761991e16cb3d7ab21f3b36e38992e04b2376927c18b65",
            "cid": "blake3:9df624162d14ba133fed34c560e4828961a4dc8d6a9438c731e8f8c209c16ad4",
        },
        "reader": {
            "bytes": 566692,
            "sha256": "912ac1d8a3dfb80a04755557576fbb87d518e78f04163085111fccfd329e5250",
            "cid": "blake3:c11d21817bff818fa242f653279e9e0c12d21641ff63df3a5f7a6680bcc732a7",
        },
        "token_frames": {
            "bytes": 32189,
            "sha256": "427f20c223886131910ebc3a16dcc4d7898c732b1654a37e483b5494b0b83fc0",
            "cid": "blake3:303a734c069af0c8910c8b473c87b549a687cf1f04570b979238d5d187576a13",
        },
        "vocabulary": {
            "bytes": 130245,
            "sha256": "01d70796333a5c94c87a45d012a04038a9c79da2127792f5acd0132fd0255a82",
            "cid": "blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b",
        },
    },
    "reader_state_cid": "blake3:7c659422df2e65a0ce24c08738dc9f08dca99775de1702251097a0fc6483404e",
    "core_state_cid": "blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59",
    "frame_tree_cid": "blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c",
    "policy_sha256": "91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0",
}
_EXPECTED_FIELDS = {
    "artifact_sha256", "contract_sha256", "accepted_binding", "operator_profile",
    "export_release_sha256",
}
_MANIFEST_FIELDS = {
    "schema", "name", "canonicalization", "contract_sha256", "operation",
    "operator_profile", "source_binding", "export_provenance", "components",
    "identity_index", "native_state_sha256", "tied_aliases",
}
_COMPONENT_FIELDS = {"name", "kind", "dtype", "shape", "offset", "bytes", "sha256"}


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _hex64(value: object) -> bool:
    return (
        type(value) is str
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


def _different_hex(value: str) -> str:
    if not _hex64(value):
        raise ValueError("fixture digest must be lowercase hexadecimal SHA256")
    return ("1" if value[0] == "0" else "0") + value[1:]


def _canonical(value: object) -> bytes:
    """The contract's ASCII JSON spelling, including long control escapes."""

    if value is None:
        return b"null"
    if type(value) is bool:
        return b"true" if value else b"false"
    if type(value) is int:
        if not 0 <= value <= 0xFFFFFFFFFFFFFFFF:
            raise ValueError("manifest integer outside unsigned 64-bit range")
        return str(value).encode("ascii")
    if type(value) is str:
        if not value.isascii():
            raise ValueError("manifest string must be ASCII")
        pieces = ['"']
        for character in value:
            if character in ('"', "\\"):
                pieces.append("\\" + character)
            elif ord(character) < 32:
                pieces.append(f"\\u{ord(character):04x}")
            else:
                pieces.append(character)
        pieces.append('"')
        return "".join(pieces).encode("ascii")
    if type(value) is list:
        return b"[" + b",".join(_canonical(item) for item in value) + b"]"
    if type(value) is dict:
        if any(type(key) is not str for key in value):
            raise ValueError("manifest object key must be a string")
        return b"{" + b",".join(
            _canonical(key) + b":" + _canonical(value[key]) for key in sorted(value)
        ) + b"}"
    raise ValueError("unsupported manifest value type")


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate manifest key")
        result[key] = value
    return result


def _reject_number(_: str) -> None:
    raise ValueError("manifest must not contain floating-point numbers")


def _read_baseline(artifact: bytes, expected_binding: dict) -> tuple[dict, bytes]:
    """Check metadata framing and hashes without interpreting numeric state."""

    if type(artifact) is not bytes or not 20 <= len(artifact) <= _MAX_BYTES:
        raise ValueError("baseline must be complete in-limit immutable artifact bytes")
    if type(expected_binding) is not dict or set(expected_binding) != _EXPECTED_FIELDS:
        raise ValueError("baseline ExpectedBinding has a different field set")
    if (
        expected_binding["artifact_sha256"] != _sha256(artifact)
        or expected_binding["contract_sha256"] != _CONTRACT_SHA256
        or expected_binding["accepted_binding"] != _ACCEPTED_BINDING
        or expected_binding["operator_profile"] != _PROFILE
        or not _hex64(expected_binding["export_release_sha256"])
    ):
        raise ValueError("baseline ExpectedBinding does not match the frozen contract")
    if artifact[:8] != _MAGIC:
        raise ValueError("baseline has different container magic")
    manifest_length = struct.unpack_from("<I", artifact, 8)[0]
    manifest_end = 12 + manifest_length
    payload_start = manifest_end + 8
    if manifest_length > _MAX_MANIFEST_BYTES or payload_start > len(artifact):
        raise ValueError("baseline has invalid manifest bounds")
    payload_length = struct.unpack_from("<Q", artifact, manifest_end)[0]
    if payload_length != _PAYLOAD_BYTES or payload_start + payload_length != len(artifact):
        raise ValueError("baseline has different fixed payload framing")
    manifest_bytes = artifact[12:manifest_end]
    manifest = json.loads(
        manifest_bytes,
        object_pairs_hook=_unique_object,
        parse_float=_reject_number,
        parse_constant=_reject_number,
    )
    if type(manifest) is not dict or set(manifest) != _MANIFEST_FIELDS:
        raise ValueError("baseline has a different manifest field set")
    if _canonical(manifest) != manifest_bytes:
        raise ValueError("baseline manifest is not ascii-json-1086/1 canonical")
    if (
        manifest["schema"] != "uor-r4.native-reference-manifest/1"
        or manifest["name"] != "R4LearnedReferenceV1"
        or manifest["canonicalization"] != "ascii-json-1086/1"
        or manifest["operation"] != "answer_four_fact_raw_text/v1"
        or manifest["operator_profile"] != _PROFILE
        or manifest["contract_sha256"] != _CONTRACT_SHA256
        or manifest["source_binding"] != _ACCEPTED_BINDING
        or manifest["tied_aliases"] != {"core.lm_head.weight": "core.embedding.weight"}
        or type(manifest["identity_index"]) is not int
        or not 0 <= manifest["identity_index"] < 120
        or not _hex64(manifest["native_state_sha256"])
    ):
        raise ValueError("baseline manifest does not carry frozen identities")
    provenance = manifest["export_provenance"]
    if (
        type(provenance) is not dict
        or provenance.get("release_sha256") != expected_binding["export_release_sha256"]
    ):
        raise ValueError("baseline export-release provenance differs")
    components = manifest["components"]
    if type(components) is not list or len(components) != len(_COMPONENTS):
        raise ValueError("baseline has a different component count")
    payload = artifact[payload_start:]
    for component, frozen in zip(components, _COMPONENTS, strict=True):
        if type(component) is not dict or set(component) != _COMPONENT_FIELDS:
            raise ValueError("baseline component has different fields")
        name, kind, dtype, shape, offset, length = frozen
        fixed = {
            "name": name, "kind": kind, "dtype": dtype, "shape": list(shape),
            "offset": offset, "bytes": length,
        }
        if any(component[key] != value for key, value in fixed.items()):
            raise ValueError("baseline component differs from frozen layout")
        if component["sha256"] != _sha256(payload[offset:offset + length]):
            raise ValueError("baseline component digest does not match its bytes")
    return manifest, payload


def _pack(manifest: dict, payload: bytes) -> bytes:
    encoded = _canonical(manifest)
    if len(encoded) > _MAX_MANIFEST_BYTES or len(payload) != _PAYLOAD_BYTES:
        raise ValueError("fixture mutation changed fixed framing limits")
    return _MAGIC + struct.pack("<I", len(encoded)) + encoded + struct.pack("<Q", len(payload)) + payload


def _replace_component_prefix(
    manifest: dict, payload: bytes, component_index: int, replacement: bytes
) -> tuple[dict, bytes]:
    """Change only synthetic bytes and refresh their prior-stage digest."""

    mutated_manifest = copy.deepcopy(manifest)
    component = mutated_manifest["components"][component_index]
    offset, length = component["offset"], component["bytes"]
    if not 0 < len(replacement) <= length:
        raise ValueError("fixture replacement exceeds its fixed component")
    if payload[offset:offset + len(replacement)] == replacement:
        raise ValueError("fixture replacement would not change the baseline")
    mutated_payload = payload[:offset] + replacement + payload[offset + len(replacement):]
    component["sha256"] = _sha256(mutated_payload[offset:offset + length])
    return mutated_manifest, mutated_payload


def build_mutations(artifact: bytes, expected_binding: dict) -> list[dict]:
    """Return eleven fixtures without calling the loader or mutating inputs.

    ``expected_error`` is the complete native error object, with exact tag and
    optional component/offset. Returned ExpectedBindings differ from the input
    only in their fixture-only whole-artifact SHA256. Accepted source, state,
    codec, frame, profile and release trust anchors stay unchanged.
    """

    manifest, payload = _read_baseline(artifact, expected_binding)
    records = []

    def record(
        name: str, tag: str, body: bytes, component: str | None = None,
        offset: int | None = None, *, wrong_expected_hash: bool = False,
    ) -> None:
        binding = copy.deepcopy(expected_binding)
        actual = _sha256(body)
        binding["artifact_sha256"] = _different_hex(actual) if wrong_expected_hash else actual
        records.append({
            "name": name,
            "expected_error": {"tag": tag, "component": component, "offset": offset},
            "artifact": body,
            "expected_binding": binding,
        })

    # The length cap precedes both header parsing and digest verification.
    record("container_limit", "CONTAINER_LIMIT", artifact + bytes(_MAX_BYTES + 1 - len(artifact)))
    record("invalid_magic", "INVALID_CONTAINER", bytes([artifact[0] ^ 1]) + artifact[1:])
    record("wrong_expected_artifact_hash", "ARTIFACT_IDENTITY_MISMATCH", artifact, wrong_expected_hash=True)

    changed = copy.deepcopy(manifest)
    changed["unexpected_fixture_field"] = 0
    record("unknown_manifest_field", "UNSUPPORTED_MANIFEST", _pack(changed, payload))

    changed = copy.deepcopy(manifest)
    changed["operator_profile"] = "unsupported-fixture-profile/1102"
    record("unsupported_operator_profile", "UNSUPPORTED_PROFILE", _pack(changed, payload))

    changed = copy.deepcopy(manifest)
    changed["export_provenance"]["release_sha256"] = _different_hex(expected_binding["export_release_sha256"])
    record("export_release_binding_mismatch", "SOURCE_BINDING_MISMATCH", _pack(changed, payload))

    changed = copy.deepcopy(manifest)
    changed["components"][0]["sha256"] = _different_hex(changed["components"][0]["sha256"])
    record("component_digest_mismatch", "INVALID_COMPONENT", _pack(changed, payload), "reader.context.bias", 0)

    changed, changed_payload = _replace_component_prefix(manifest, payload, 0, struct.pack("<I", 0x7FC00000))
    record("nonfinite_parameter_bits", "INVALID_TENSOR", _pack(changed, changed_payload), "reader.context.bias", 0)

    policy_offset = _COMPONENTS[15][4]
    changed, changed_payload = _replace_component_prefix(manifest, payload, 15, bytes([payload[policy_offset] ^ 1]))
    record("policy_bytes_mismatch", "INVALID_CODEC_POLICY", _pack(changed, changed_payload))

    changed, changed_payload = _replace_component_prefix(manifest, payload, 19, struct.pack("<q", 120))
    record("out_of_range_multiplication_index", "INVALID_FRAME_TABLE", _pack(changed, changed_payload), "multiplication", 0)

    changed = copy.deepcopy(manifest)
    changed["native_state_sha256"] = _different_hex(changed["native_state_sha256"])
    record("native_state_digest_mismatch", "STATE_IDENTITY_MISMATCH", _pack(changed, payload))
    return records
