"""Byte-preserved construction and fresh development for query readout (#1067)."""

from __future__ import annotations

import copy
import json
import random
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_english_binding import data as source
from ..zoology_release.development import _canonical_safetensors

VOCABULARY = source.VOCABULARY
LOCATIONS = source.LOCATIONS
UNKNOWN_ID = source.UNKNOWN_ID
SEQUENCE_LENGTH = 41
ANSWER_POSITION = 37
OWNER_POSITION = 35
MANIFEST = "manifest.json"
SCHEMA = "uor-r4.query-object-readout-data/1"
SOURCE_MANIFEST_CID = (
    "blake3:fbf3c3d6b5694dea16b2d5c1f5e4fb5d198b339b36a80b0dab91d4714ce04d7d"
)
_FILES = ("construction.safetensors", "development.safetensors", "vocabulary.json")
DATA_POLICY = {
    "name": "EnglishQueryObjectReadoutDataV1",
    "issue": 1067,
    "source_issue": 1063,
    "construction_groups": 2048,
    "development_groups": 256,
    "development_seed": 10672,
    "sequence_length": SEQUENCE_LENGTH,
    "answer_position": ANSWER_POSITION,
    "owner_position": OWNER_POSITION,
    "source_answer_position": 40,
    "construction": "copy source bytes; change only returned positions from 40 to 37",
    "historical_development": "regenerate unchanged source generator and verify its canonical bytes against the frozen source manifest; never read its payload",
    "exclusion": "all base, location-swapped and absent-binding canonical worlds; also exact input rows; no new world reused across groups",
    "heldout_pair": source.DATA_POLICY["heldout_pair"],
    "variants": list(source.VARIANTS),
    "pair_types": list(source.PAIR_TYPES),
    "position_balance": source.DATA_POLICY["position_balance"],
    "location_balance": source.DATA_POLICY["location_balance"],
    "tokenizer": source.DATA_POLICY["tokenizer"],
    "vocabulary_size": len(VOCABULARY),
}


def _record(name: str, payload: bytes) -> dict[str, Any]:
    return {"path": name, "bytes": len(payload), "cid": cid_bytes(payload)}


def _source_record(metadata: dict[str, Any], name: str) -> dict[str, Any]:
    return next(
        record for record in metadata["source"]["files"] if record["path"] == name
    )


def _historical_development(metadata: dict[str, Any]) -> dict[str, Tensor]:
    """Reproduce the public old population without opening its retained payload."""
    tensors = source._build_split(development=True)
    actual = _record("development.safetensors", _canonical_safetensors(tensors))
    if actual != _source_record(metadata, "development.safetensors"):
        raise ValueError(
            "regenerated historical development differs from source identity"
        )
    return tensors


def _old_position_view(tensors: dict[str, Tensor]) -> dict[str, Tensor]:
    if "positions" not in tensors or not bool(
        (tensors["positions"] == ANSWER_POSITION).all()
    ):
        raise ValueError("query-readout positions must be 37")
    return {**tensors, "positions": tensors["positions"] + (40 - ANSWER_POSITION)}


def _check_shapes(tensors: dict[str, Tensor], *, development: bool) -> None:
    source._check_shapes(_old_position_view(tensors), development=development)
    owners = torch.tensor([source.TOKEN_IDS[word] for word in source.OWNERS])
    objects = torch.tensor([source.TOKEN_IDS[word] for word in source.OBJECTS])
    if not bool(
        torch.isin(tensors["inputs"][:, OWNER_POSITION], owners).all()
        and torch.isin(tensors["inputs"][:, ANSWER_POSITION], objects).all()
    ):
        raise ValueError("owner/object tokens are not at causal positions 35/37")


def _audit_population(
    tensors: dict[str, Tensor], *, development: bool
) -> tuple[dict[str, Any], set, set, Any]:
    _check_shapes(tensors, development=development)
    return source._audit_split(_old_position_view(tensors), development=development)


def _worlds(tensors: dict[str, Tensor]) -> set:
    return {
        tuple(sorted(source.parse_row(row)[0])) for row in tensors["inputs"].tolist()
    }


def _inputs(tensors: dict[str, Tensor]) -> set:
    return {tuple(row) for row in tensors["inputs"].tolist()}


def _named_world(facts: list[tuple[int, int, int]]) -> tuple:
    return tuple(
        sorted(
            (source.OWNERS[owner], source.OBJECTS[obj], LOCATIONS[location])
            for owner, obj, location in facts
        )
    )


def _build_development(excluded_worlds: set) -> dict[str, Tensor]:
    """Retain preassigned slot/location balance while rejecting every collision."""
    groups = DATA_POLICY["development_groups"]
    rng = random.Random(DATA_POLICY["development_seed"])
    slots = [source._balanced_pairs(groups // 2, 4, rng) for _ in source.PAIR_TYPES]
    locations = [source._balanced_pairs(groups // 2, 8, rng) for _ in source.PAIR_TYPES]
    inputs: list[list[int]] = []
    targets: list[list[int]] = []
    seen = set(excluded_worlds)
    for group in range(groups):
        pair_type, within_type = group % 2, group // 2
        second = 1 if pair_type == 0 else 2
        first_location, second_location = locations[pair_type][within_type]
        assigned = {0: first_location, second: second_location}
        remainder = rng.sample(
            [index for index in range(8) if index not in assigned.values()], 2
        )
        for logical, location in zip(
            [index for index in range(4) if index not in assigned],
            remainder,
            strict=True,
        ):
            assigned[logical] = location
        for _ in range(100_000):
            a, b, c, x, y, z = source._pairs(rng, development=True)
            facts = [
                (a, x, assigned[0]),
                (a, y, assigned[1]),
                (b, x, assigned[2]),
                (c, z, assigned[3]),
            ]
            swapped = list(facts)
            swapped[0] = (*facts[0][:2], facts[second][2])
            swapped[second] = (*facts[second][:2], facts[0][2])
            absent = list(facts)
            absent[0] = (a, z, facts[0][2])
            absent[3] = (c, x, facts[3][2])
            signatures = {_named_world(world) for world in (facts, swapped, absent)}
            if len(signatures) == 3 and not signatures & seen:
                seen.update(signatures)
                break
        else:
            raise ValueError(
                "fresh development exclusion exhausted its fixed candidate budget"
            )
        first_slot, second_slot = slots[pair_type][within_type]
        order = [-1] * 4
        order[first_slot], order[second_slot] = 0, second
        for slot, logical in zip(
            [index for index in range(4) if order[index] == -1],
            rng.sample([index for index in range(4) if index not in (0, second)], 2),
            strict=True,
        ):
            order[slot] = logical
        q0, q1 = (a, x), (a, y) if pair_type == 0 else (b, x)
        inputs.extend(
            (
                source._input(facts, order, q0),
                source._input(facts, order, q1),
                source._input(swapped, order, q0),
                source._input(swapped, order, q1),
                source._input(absent, order, q0),
            )
        )
        targets.extend(
            [
                [source.TOKEN_IDS[LOCATIONS[first_location]]],
                [source.TOKEN_IDS[LOCATIONS[second_location]]],
                [source.TOKEN_IDS[LOCATIONS[second_location]]],
                [source.TOKEN_IDS[LOCATIONS[first_location]]],
                [UNKNOWN_ID],
            ]
        )
    rows = groups * 5
    return {
        "inputs": torch.tensor(inputs, dtype=torch.long),
        "positions": torch.full((rows, 1), ANSWER_POSITION, dtype=torch.long),
        "targets": torch.tensor(targets, dtype=torch.long),
        "group_ids": torch.arange(groups, dtype=torch.long).repeat_interleave(5),
        "variant_ids": torch.arange(5, dtype=torch.long).repeat(groups),
        "pair_types": (torch.arange(groups, dtype=torch.long) % 2).repeat_interleave(5),
    }


def _audit_all(
    construction: dict[str, Tensor],
    development: dict[str, Tensor],
    historical: dict[str, Tensor],
) -> dict[str, Any]:
    train, train_worlds, train_pairs, train_tokens = _audit_population(
        construction, development=False
    )
    dev, dev_worlds, dev_pairs, _ = _audit_population(development, development=True)
    old, old_worlds, _, _ = source._audit_split(historical, development=True)
    old_overlap = dev_worlds & old_worlds
    train_overlap = dev_worlds & train_worlds
    pair_overlap = train_pairs & dev_pairs
    old_input_overlap = _inputs(development) & _inputs(historical)
    train_input_overlap = _inputs(development) & _inputs(construction)
    if (
        old_overlap
        or train_overlap
        or pair_overlap
        or old_input_overlap
        or train_input_overlap
    ):
        raise ValueError(
            "fresh development overlaps an excluded world, input or construction pair"
        )
    if len(dev_worlds) != DATA_POLICY["development_groups"] * 3:
        raise ValueError(
            "fresh development reuses a canonical world across variants or groups"
        )
    if any(
        source.TOKEN_IDS[word] not in train_tokens for word in source.ACTIVE_VOCABULARY
    ):
        raise ValueError("construction does not cover the active lexical vocabulary")
    return {
        "construction": train,
        "development": dev,
        "historical_development_canonical_worlds": len(old_worlds),
        "historical_development_rows": old["rows"],
        "fresh_development_canonical_worlds": len(dev_worlds),
        "canonical_world_overlap_with_historical": 0,
        "canonical_world_overlap_with_construction": 0,
        "exact_input_overlap_with_historical": 0,
        "exact_input_overlap_with_construction": 0,
        "owner_object_pair_overlap_with_construction": 0,
        "active_vocabulary_covered": True,
        "retained_input_length": SEQUENCE_LENGTH,
        "owner_position": OWNER_POSITION,
        "answer_position": ANSWER_POSITION,
    }


def build(root: Path, source_data_root: Path) -> dict[str, Any]:
    """Copy construction exactly and freeze fresh development before any fitting."""
    root, source_data_root = Path(root).resolve(), Path(source_data_root).resolve()
    if source.ANSWER_POSITION != 40 or source.SEQUENCE_LENGTH != SEQUENCE_LENGTH:
        raise ValueError("source serialization contract changed")
    original = source._manifest(source_data_root)
    if original["manifest_cid"] != SOURCE_MANIFEST_CID:
        raise ValueError("source data manifest is not the published #1063 identity")
    source_manifest_payload = (source_data_root / source.MANIFEST).read_bytes()
    construction_payload = source._payload(
        source_data_root, original, "construction.safetensors"
    )
    vocabulary_payload = source._payload(source_data_root, original, "vocabulary.json")
    if vocabulary_payload != source._vocabulary_bytes():
        raise ValueError("source vocabulary differs from the unchanged lexical map")
    metadata = {
        "schema": SCHEMA,
        "policy": copy.deepcopy(DATA_POLICY),
        "source": {
            "root": str(source_data_root),
            "manifest": _record(source.MANIFEST, source_manifest_payload),
            "manifest_cid": original["manifest_cid"],
            "tree_cid": original["tree_cid"],
            "files": copy.deepcopy(original["files"]),
        },
    }
    original_construction = load_safetensors(construction_payload)
    source._check_shapes(original_construction, development=False)
    construction = {
        **original_construction,
        "positions": torch.full_like(
            original_construction["positions"], ANSWER_POSITION
        ),
    }
    historical = _historical_development(metadata)
    development = _build_development(_worlds(construction) | _worlds(historical))
    audit = _audit_all(construction, development, historical)
    if audit["construction"] != original["audit"]["construction"]:
        raise ValueError("copied construction semantic audit differs from the source")
    payloads = {
        "construction.safetensors": construction_payload,
        "development.safetensors": _canonical_safetensors(development),
        "vocabulary.json": vocabulary_payload,
    }
    records = [_record(name, payload) for name, payload in sorted(payloads.items())]
    metadata.update({"files": records, "tree_cid": tree_cid(records), "audit": audit})
    metadata["manifest_cid"] = cid_bytes(canonical_json_bytes(metadata))
    root.parent.mkdir(parents=True, exist_ok=True)
    root.mkdir(exist_ok=False)
    for name, payload in (
        *payloads.items(),
        (MANIFEST, canonical_json_bytes(metadata)),
    ):
        with (root / name).open("xb") as output:
            output.write(payload)
    return metadata


def _manifest(root: Path) -> dict[str, Any]:
    metadata = json.loads((root / MANIFEST).read_text())
    body = dict(metadata)
    expected = body.pop("manifest_cid", None)
    if (
        metadata.get("schema") != SCHEMA
        or metadata.get("policy") != DATA_POLICY
        or expected != cid_bytes(canonical_json_bytes(body))
    ):
        raise ValueError("query-readout manifest identity or policy differs")
    records = metadata.get("files")
    source_records = metadata.get("source", {}).get("files")
    if (
        not isinstance(records, list)
        or not isinstance(source_records, list)
        or [row.get("path") for row in records] != list(_FILES)
        or [row.get("path") for row in source_records] != list(_FILES)
        or metadata["source"].get("manifest_cid") != SOURCE_MANIFEST_CID
        or metadata.get("tree_cid") != tree_cid(records)
        or metadata["source"].get("tree_cid") != tree_cid(source_records)
    ):
        raise ValueError("query-readout file inventory differs")
    for name in ("construction.safetensors", "vocabulary.json"):
        if next(row for row in records if row["path"] == name) != _source_record(
            metadata, name
        ):
            raise ValueError(
                "construction or vocabulary bytes do not match source identity"
            )
    return metadata


def _payload(root: Path, metadata: dict[str, Any], name: str) -> bytes:
    record = next(row for row in metadata["files"] if row["path"] == name)
    payload = (root / name).read_bytes()
    if _record(name, payload) != record:
        raise ValueError(f"query-readout data file changed: {name}")
    return payload


def _construction(root: Path, metadata: dict[str, Any]) -> dict[str, Tensor]:
    tensors = load_safetensors(_payload(root, metadata, "construction.safetensors"))
    source._check_shapes(tensors, development=False)
    # Preserve every source input token, label and metadata tensor. Only this
    # returned tensor changes; the source payload and predecessor stay intact.
    tensors["positions"] = torch.full_like(tensors["positions"], ANSWER_POSITION)
    _check_shapes(tensors, development=False)
    return tensors


def validate(root: Path, inspect_development: bool = False) -> dict[str, Any]:
    """Default validation leaves both historical and fresh development unopened."""
    root = Path(root).resolve()
    metadata = _manifest(root)
    if _payload(root, metadata, "vocabulary.json") != source._vocabulary_bytes():
        raise ValueError("query-readout vocabulary differs")
    construction = _construction(root, metadata)
    construction_audit, _, _, _ = _audit_population(construction, development=False)
    if construction_audit != metadata["audit"]["construction"]:
        raise ValueError("query-readout construction semantic audit differs")
    if inspect_development:
        development = load_safetensors(
            _payload(root, metadata, "development.safetensors")
        )
        historical = _historical_development(metadata)
        if _audit_all(construction, development, historical) != metadata["audit"]:
            raise ValueError(
                "query-readout development semantic/exclusion audit differs"
            )
    return metadata


def load_construction(root: Path) -> dict[str, Tensor]:
    """Load all 10,240 construction rows, returning only positions changed to 37."""
    root = Path(root).resolve()
    return _construction(root, _manifest(root))


def load_training(root: Path, mixed: bool) -> dict[str, Tensor]:
    """Load construction only; preserve the two original curriculum populations."""
    tensors = load_construction(root)
    admitted = (
        torch.ones(tensors["variant_ids"].shape, dtype=torch.bool)
        if mixed
        else tensors["variant_ids"] < 4
    )
    return {
        f"train_{name}": tensors[name][admitted].contiguous()
        for name in ("inputs", "positions", "targets")
    }


def load_development(root: Path) -> dict[str, Tensor]:
    """Open only the fresh development payload; the campaign controls admission."""
    root = Path(root).resolve()
    tensors = load_safetensors(
        _payload(root, _manifest(root), "development.safetensors")
    )
    _check_shapes(tensors, development=True)
    return tensors
