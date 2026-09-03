"""Unchanged construction and fresh development excluding four prior populations."""

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
from ..zoology_cyclic_facts import data as previous
from ..zoology_english_binding import data as source
from ..zoology_query_readout.data import (
    _audit_population,
    _check_shapes,
    _inputs,
    _named_world,
    _record,
    _source_record,
    _worlds,
)
from ..zoology_release.development import _canonical_safetensors

VOCABULARY = source.VOCABULARY
LOCATIONS = source.LOCATIONS
UNKNOWN_ID = source.UNKNOWN_ID
SEQUENCE_LENGTH = 41
ANSWER_POSITION = 37
OWNER_POSITION = 35
MANIFEST = "manifest.json"
SCHEMA = "uor-r4.compound-binding-data/1"
SOURCE_MANIFEST_CID = previous.SOURCE_MANIFEST_CID
PREVIOUS_MANIFEST_CID = (
    "blake3:6eeb4831d9558dbbb0ac1235b314fb47e1412a561aad429d2af56acd81dc0a54"
)
PREVIOUS_PREPARATION_CID = (
    "blake3:92ffef681fe6bd4cfc6532bd811496b6e9c4d052b978c172d3a5ca9bcef8ef05"
)
PREVIOUS_DEVELOPMENT = {
    "path": "development.safetensors",
    "bytes": 471496,
    "cid": "blake3:31dca3ff6327c75bd5208758dcd748661002c8bb109346d8c2d4a3b414c02ed9",
}
_PREVIOUS_DOCUMENT = "docs/r4_zoology_cyclic_facts_1071_preparation.json"
_FILES = ("construction.safetensors", "development.safetensors", "vocabulary.json")
DATA_POLICY = copy.deepcopy(previous.DATA_POLICY)
DATA_POLICY.update(
    {
        "name": "EnglishCompoundBindingDataV1",
        "issue": 1073,
        "development_seed": 10732,
        "historical_development": "regenerate unchanged #1063, #1067, #1069 and #1071 generators; verify canonical bytes against their published manifest records; never open retained development payloads",
        "exclusion": "all #1063/#1067/#1069/#1071 development and construction base, location-swapped and absent-binding canonical worlds; also exact input rows; no new world reused across groups",
    }
)


def _verify_envelope(value: dict[str, Any], key: str, expected: str) -> None:
    body = dict(value)
    if (
        body.pop(key, None) != expected
        or cid_bytes(canonical_json_bytes(body)) != expected
    ):
        raise ValueError("published historical envelope identity differs")


def _published_previous() -> dict[str, Any]:
    payload = (Path(__file__).resolve().parents[5] / _PREVIOUS_DOCUMENT).read_bytes()
    preparation = json.loads(payload)
    _verify_envelope(preparation, "preparation_cid", PREVIOUS_PREPARATION_CID)
    dataset = preparation["dataset"]
    _verify_envelope(dataset, "manifest_cid", PREVIOUS_MANIFEST_CID)
    if (
        dataset["policy"] != previous.DATA_POLICY
        or dataset["schema"] != previous.SCHEMA
        or dataset["source"]["manifest_cid"] != SOURCE_MANIFEST_CID
        or next(
            row for row in dataset["files"] if row["path"] == "development.safetensors"
        )
        != PREVIOUS_DEVELOPMENT
    ):
        raise ValueError("published #1071 data policy or file identity differs")
    return {
        "manifest_cid": PREVIOUS_MANIFEST_CID,
        "development": copy.deepcopy(PREVIOUS_DEVELOPMENT),
        "published_preparation": _record(_PREVIOUS_DOCUMENT, payload),
        "preparation_cid": PREVIOUS_PREPARATION_CID,
        "dataset": copy.deepcopy(dataset),
    }


def _historical_development(
    metadata: dict[str, Any], construction: dict[str, Tensor]
) -> dict[str, dict[str, Tensor]]:
    """Reproduce every predecessor using its own unchanged generator constants."""
    historical = previous._historical_development(metadata, construction)
    excluded = _worlds(construction)
    for tensors in historical.values():
        excluded.update(_worlds(tensors))
    old_1071 = previous._build_development(excluded)
    actual = _record("development.safetensors", _canonical_safetensors(old_1071))
    if actual != metadata["historical_development"]["1071"]["development"]:
        raise ValueError(
            "regenerated #1071 development differs from its published identity"
        )
    historical["1071"] = old_1071
    return historical


def _build_development(excluded_worlds: set) -> dict[str, Tensor]:
    """Keep fixed slot/location schedules while rejecting all three world variants."""
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
                "compound-binding development exhausted its fixed candidate budget"
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
    historical: dict[str, dict[str, Tensor]],
) -> dict[str, Any]:
    train, train_worlds, train_pairs, train_tokens = _audit_population(
        construction, development=False
    )
    dev, fresh_worlds, dev_pairs, _ = _audit_population(development, development=True)
    train_inputs, fresh_inputs = _inputs(construction), _inputs(development)
    if (
        train_worlds & fresh_worlds
        or train_inputs & fresh_inputs
        or train_pairs & dev_pairs
    ):
        raise ValueError(
            "fresh development overlaps construction worlds, inputs or pairs"
        )
    if len(fresh_worlds) != DATA_POLICY["development_groups"] * 3:
        raise ValueError("fresh development reuses a world across variants or groups")
    if any(
        source.TOKEN_IDS[word] not in train_tokens for word in source.ACTIVE_VOCABULARY
    ):
        raise ValueError("construction does not cover the active lexical vocabulary")
    history_audit: dict[str, Any] = {}
    union_worlds: set = set()
    union_inputs: set = set()
    total_rows = 0
    if set(historical) != {"1063", "1067", "1069", "1071"}:
        raise ValueError("historical population inventory differs")
    for issue, tensors in historical.items():
        old_worlds, old_inputs = _worlds(tensors), _inputs(tensors)
        if fresh_worlds & old_worlds or fresh_inputs & old_inputs:
            raise ValueError(f"fresh development overlaps historical #{issue}")
        rows = tensors["inputs"].shape[0]
        history_audit[issue] = {
            "rows": rows,
            "unique_input_rows": len(old_inputs),
            "canonical_worlds": len(old_worlds),
            "canonical_world_overlap": 0,
            "exact_input_overlap": 0,
        }
        total_rows += rows
        union_worlds.update(old_worlds)
        union_inputs.update(old_inputs)
    return {
        "construction": train,
        "development": dev,
        "fresh_development_canonical_worlds": len(fresh_worlds),
        "fresh_development_unique_input_rows": len(fresh_inputs),
        "canonical_world_overlap_with_construction": 0,
        "exact_input_overlap_with_construction": 0,
        "owner_object_pair_overlap_with_construction": 0,
        "historical_development": history_audit,
        "historical_development_union": {
            "populations": len(historical),
            "rows": total_rows,
            "unique_input_rows": len(union_inputs),
            "canonical_worlds": len(union_worlds),
            "canonical_world_overlap": 0,
            "exact_input_overlap": 0,
        },
        "all_exclusions_union": {
            "populations": len(historical) + 1,
            "rows": total_rows + construction["inputs"].shape[0],
            "unique_input_rows": len(union_inputs | train_inputs),
            "canonical_worlds": len(union_worlds | train_worlds),
            "canonical_world_overlap": 0,
            "exact_input_overlap": 0,
        },
        "active_vocabulary_covered": True,
        "retained_input_length": SEQUENCE_LENGTH,
        "owner_position": OWNER_POSITION,
        "answer_position": ANSWER_POSITION,
    }


def build(root: Path, source_data_root: Path) -> dict[str, Any]:
    """Freeze one fresh population while copying original construction bytes."""
    root, source_data_root = Path(root).resolve(), Path(source_data_root).resolve()
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
    published = _published_previous()
    if published["dataset"]["source"]["files"] != original["files"]:
        raise ValueError("published #1071 and supplied #1063 data identities disagree")
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
        "historical_development": {
            **copy.deepcopy(published["dataset"]["historical_development"]),
            "1071": published,
        },
    }
    construction = load_safetensors(construction_payload)
    source._check_shapes(construction, development=False)
    construction["positions"] = torch.full_like(
        construction["positions"], ANSWER_POSITION
    )
    historical = _historical_development(metadata, construction)
    excluded = _worlds(construction)
    for tensors in historical.values():
        excluded.update(_worlds(tensors))
    development = _build_development(excluded)
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
        raise ValueError("compound-binding manifest identity or policy differs")
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
        raise ValueError("compound-binding file inventory differs")
    for name in ("construction.safetensors", "vocabulary.json"):
        if next(row for row in records if row["path"] == name) != _source_record(
            metadata, name
        ):
            raise ValueError(
                "copied construction or vocabulary source identity differs"
            )
    history = metadata.get("historical_development", {})
    if set(history) != {"1063", "1067", "1069", "1071"}:
        raise ValueError("historical development inventory differs")
    latest = history["1071"]
    _verify_envelope(latest["dataset"], "manifest_cid", PREVIOUS_MANIFEST_CID)
    older = {issue: record for issue, record in history.items() if issue != "1071"}
    if (
        history["1063"]["manifest_cid"] != SOURCE_MANIFEST_CID
        or history["1063"]["development"]
        != _source_record(metadata, "development.safetensors")
        or latest["manifest_cid"] != PREVIOUS_MANIFEST_CID
        or latest["preparation_cid"] != PREVIOUS_PREPARATION_CID
        or latest["development"] != PREVIOUS_DEVELOPMENT
        or latest["dataset"]["source"]["files"] != source_records
        or older != latest["dataset"]["historical_development"]
    ):
        raise ValueError(
            "historical development identities differ from the published chain"
        )
    return metadata


def _payload(root: Path, metadata: dict[str, Any], name: str) -> bytes:
    record = next(row for row in metadata["files"] if row["path"] == name)
    payload = (root / name).read_bytes()
    if _record(name, payload) != record:
        raise ValueError(f"compound-binding data file changed: {name}")
    return payload


def _construction(root: Path, metadata: dict[str, Any]) -> dict[str, Tensor]:
    tensors = load_safetensors(_payload(root, metadata, "construction.safetensors"))
    source._check_shapes(tensors, development=False)
    tensors["positions"] = torch.full_like(tensors["positions"], ANSWER_POSITION)
    _check_shapes(tensors, development=False)
    return tensors


def validate(root: Path, inspect_development: bool = False) -> dict[str, Any]:
    """Default validation reads construction, vocabulary and manifest only."""
    root = Path(root).resolve()
    metadata = _manifest(root)
    if _payload(root, metadata, "vocabulary.json") != source._vocabulary_bytes():
        raise ValueError("compound-binding vocabulary differs")
    construction = _construction(root, metadata)
    construction_audit, _, _, _ = _audit_population(construction, development=False)
    if construction_audit != metadata["audit"]["construction"]:
        raise ValueError("compound-binding construction semantic audit differs")
    if inspect_development:
        development = load_safetensors(
            _payload(root, metadata, "development.safetensors")
        )
        historical = _historical_development(metadata, construction)
        if _audit_all(construction, development, historical) != metadata["audit"]:
            raise ValueError(
                "compound-binding development semantic/exclusion audit differs"
            )
    return metadata


def load_construction(root: Path) -> dict[str, Tensor]:
    """Return all 10,240 original construction rows with position 37."""
    root = Path(root).resolve()
    return _construction(root, _manifest(root))


def load_training(root: Path, mixed: bool) -> dict[str, Tensor]:
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
    """Read only this issue's fresh development; the campaign controls admission."""
    root = Path(root).resolve()
    tensors = load_safetensors(
        _payload(root, _manifest(root), "development.safetensors")
    )
    _check_shapes(tensors, development=True)
    return tensors
