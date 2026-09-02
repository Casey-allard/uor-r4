"""Deterministic English counterfactual worlds and reversible lexical IDs (#1063)."""

from __future__ import annotations

import copy
import json
import random
import re
from collections import Counter
from collections.abc import Sequence
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_release.development import _canonical_safetensors

OWNERS = (
    "mara",
    "lena",
    "omar",
    "noah",
    "iris",
    "liam",
    "nora",
    "otto",
    "ada",
    "erin",
    "hugo",
    "iona",
    "jude",
    "kira",
    "leon",
    "mila",
)
OBJECTS = (
    "key",
    "book",
    "coin",
    "cup",
    "ring",
    "ball",
    "pen",
    "map",
    "hat",
    "toy",
    "jar",
    "box",
    "comb",
    "fork",
    "shoe",
    "doll",
)
LOCATIONS = (
    "drawer",
    "cabinet",
    "basket",
    "closet",
    "pouch",
    "locker",
    "crate",
    "trunk",
)
_GRAMMAR = (
    "<bos>",
    "put",
    "the",
    "in",
    ".",
    "where",
    "is",
    "'s",
    "?",
    "answer",
    ":",
    "unknown",
)
ACTIVE_VOCABULARY = _GRAMMAR + OWNERS + OBJECTS + LOCATIONS
VOCABULARY = ACTIVE_VOCABULARY + tuple(
    f"<unused-{index:04d}>" for index in range(len(ACTIVE_VOCABULARY), 4096)
)
TOKEN_IDS = {word: index for index, word in enumerate(VOCABULARY)}
BOS_ID = TOKEN_IDS["<bos>"]
UNKNOWN_ID = TOKEN_IDS["unknown"]
SEQUENCE_LENGTH = 41
ANSWER_POSITION = SEQUENCE_LENGTH - 1
PAIR_TYPES = ("same_owner", "same_object")
VARIANTS = ("base_q0", "base_q1", "swapped_q0", "swapped_q1", "absent_q0")
DATA_POLICY = {
    "name": "EnglishCounterfactualBindingV1",
    "issue": 1063,
    "construction_groups": 2048,
    "development_groups": 256,
    "construction_seed": 10631,
    "development_seed": 10632,
    "heldout_pair": "(owner_index + object_index) % 4 == 0",
    "required_pairs": ["A,x", "A,y", "B,x", "C,z", "A,z", "C,x"],
    "variants": list(VARIANTS),
    "pair_types": list(PAIR_TYPES),
    "sequence_length": SEQUENCE_LENGTH,
    "answer_position": ANSWER_POSITION,
    "vocabulary_size": len(VOCABULARY),
    "active_vocabulary_size": len(ACTIVE_VOCABULARY),
    "tokenizer": "lowercase lexical words, separate possessive 's and punctuation; one BOS; reversible declared IDs",
    "position_balance": "per pair type, every four worlds cover every q0/q1 slot; all twelve ordered slot pairs cycle",
    "location_balance": "per pair type, every eight worlds cover every q0/q1 location; all seven nonzero offsets cycle",
    "unknown_history": "swap the object words at logical facts zero and three; retain owners, locations, order and token bag",
}
MANIFEST = "manifest.json"
SCHEMA = "uor-r4.english-counterfactual-data/1"
_TOKENS = re.compile(r"<bos>|<unused-\d{4}>|'s|[a-z]+|[.?:]")
_ROW = re.compile(
    r"(?P<history>(?:[a-z]+ put the [a-z]+ in the [a-z]+\. ){4})where is (?P<owner>[a-z]+)'s (?P<object>[a-z]+)\? answer:"
)
_FACT = re.compile(r"([a-z]+) put the ([a-z]+) in the ([a-z]+)\.")


def encode(text: str, *, add_bos: bool = True) -> list[int]:
    """Tokenize declared lexical English; unknown words are errors, not labels."""
    text = text.lower()
    words: list[str] = []
    cursor = 0
    for match in _TOKENS.finditer(text):
        if text[cursor : match.start()].strip():
            raise ValueError("text contains unsupported lexical syntax")
        words.append(match.group())
        cursor = match.end()
    if text[cursor:].strip() or any(word not in TOKEN_IDS for word in words):
        raise ValueError("text contains an undeclared lexical token")
    if add_bos:
        if "<bos>" in words:
            raise ValueError("encode adds exactly one BOS")
        words.insert(0, "<bos>")
    return [TOKEN_IDS[word] for word in words]


def decode(ids: Sequence[int] | Tensor, *, skip_bos: bool = False) -> str:
    """Render declared IDs with reversible punctuation and possessive spacing."""
    values = ids.tolist() if isinstance(ids, Tensor) else list(ids)
    if any(
        isinstance(index, bool)
        or not isinstance(index, int)
        or not 0 <= index < len(VOCABULARY)
        for index in values
    ):
        raise ValueError("token IDs must be integers in the 4096-token vocabulary")
    if skip_bos and values and values[0] == BOS_ID:
        values = values[1:]
    text = ""
    for index in values:
        word = VOCABULARY[index]
        text += word if not text or word in (".", "?", ":", "'s") else " " + word
    return text


def parse_row(
    ids: Sequence[int] | Tensor,
) -> tuple[tuple[tuple[str, str, str], ...], tuple[str, str], str]:
    """Recover facts, question and answer from decoded input alone."""
    values = ids.tolist() if isinstance(ids, Tensor) else list(ids)
    if len(values) != SEQUENCE_LENGTH or values[0] != BOS_ID:
        raise ValueError("English input must have 41 tokens and one leading BOS")
    text = decode(values, skip_bos=True)
    if encode(text) != values:
        raise ValueError("lexical input is not reversible")
    match = _ROW.fullmatch(text)
    if match is None:
        raise ValueError(
            "decoded input does not have four facts and the declared query"
        )
    facts = tuple(_FACT.findall(match["history"]))
    query = (match["owner"], match["object"])
    if (
        query[0] not in OWNERS
        or query[1] not in OBJECTS
        or any(
            owner not in OWNERS or obj not in OBJECTS or location not in LOCATIONS
            for owner, obj, location in facts
        )
    ):
        raise ValueError("decoded fact or query is outside its lexical role")
    bindings = {(owner, obj): location for owner, obj, location in facts}
    if len(bindings) != 4:
        raise ValueError("decoded history has duplicate owner-object bindings")
    return facts, query, bindings.get(query, "unknown")


def oracle_target(ids: Sequence[int] | Tensor) -> int:
    """Answer by parsing the source text, without consulting row metadata."""
    return TOKEN_IDS[parse_row(ids)[2]]


def _heldout(pair: tuple[int, int]) -> bool:
    return (pair[0] + pair[1]) % 4 == 0


def _pairs(
    rng: random.Random, *, development: bool
) -> tuple[int, int, int, int, int, int]:
    if development:
        residue = rng.randrange(4)
        a, b, c = rng.sample([index for index in range(16) if index % 4 == residue], 3)
        x, y, z = rng.sample(
            [index for index in range(16) if (index + residue) % 4 == 0], 3
        )
    else:
        a, b, c = rng.sample(range(16), 3)
        x = rng.choice(
            [
                obj
                for obj in range(16)
                if all(not _heldout((owner, obj)) for owner in (a, b, c))
            ]
        )
        y = rng.choice(
            [obj for obj in range(16) if obj != x and not _heldout((a, obj))]
        )
        z = rng.choice(
            [
                obj
                for obj in range(16)
                if obj not in (x, y)
                and not _heldout((a, obj))
                and not _heldout((c, obj))
            ]
        )
    required = ((a, x), (a, y), (b, x), (c, z), (a, z), (c, x))
    if any(_heldout(pair) != development for pair in required):
        raise RuntimeError("world sampler violated the global pair split")
    return a, b, c, x, y, z


def _balanced_pairs(count: int, size: int, rng: random.Random) -> list[tuple[int, int]]:
    pairs: list[tuple[int, int]] = []
    while len(pairs) < count:
        for offset in rng.sample(range(1, size), size - 1):
            for first in rng.sample(range(size), size):
                pairs.append((first, (first + offset) % size))
                if len(pairs) == count:
                    return pairs
    return pairs


def _input(
    facts: Sequence[tuple[int, int, int]], order: Sequence[int], query: tuple[int, int]
) -> list[int]:
    history = " ".join(
        f"{OWNERS[facts[index][0]]} put the {OBJECTS[facts[index][1]]} in the {LOCATIONS[facts[index][2]]}."
        for index in order
    )
    return encode(
        f"{history} where is {OWNERS[query[0]]}'s {OBJECTS[query[1]]}? answer:"
    )


def _build_split(*, development: bool) -> dict[str, Tensor]:
    name = "development" if development else "construction"
    groups = DATA_POLICY[f"{name}_groups"]
    rng = random.Random(DATA_POLICY[f"{name}_seed"])
    slots = [_balanced_pairs(groups // 2, 4, rng) for _ in PAIR_TYPES]
    locations = [_balanced_pairs(groups // 2, 8, rng) for _ in PAIR_TYPES]
    inputs: list[list[int]] = []
    targets: list[list[int]] = []
    seen: set[tuple[tuple[int, int, int], ...]] = set()
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
        while True:
            a, b, c, x, y, z = _pairs(rng, development=development)
            facts = [
                (a, x, assigned[0]),
                (a, y, assigned[1]),
                (b, x, assigned[2]),
                (c, z, assigned[3]),
            ]
            signature = tuple(sorted(facts))
            if signature not in seen:
                seen.add(signature)
                break
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
        swapped = list(facts)
        swapped[0] = (*facts[0][:2], facts[second][2])
        swapped[second] = (*facts[second][:2], facts[0][2])
        absent = list(facts)
        absent[0] = (a, z, facts[0][2])
        absent[3] = (c, x, facts[3][2])
        rows = (
            _input(facts, order, q0),
            _input(facts, order, q1),
            _input(swapped, order, q0),
            _input(swapped, order, q1),
            _input(absent, order, q0),
        )
        inputs.extend(rows)
        targets.extend(
            [
                [TOKEN_IDS[LOCATIONS[first_location]]],
                [TOKEN_IDS[LOCATIONS[second_location]]],
                [TOKEN_IDS[LOCATIONS[second_location]]],
                [TOKEN_IDS[LOCATIONS[first_location]]],
                [UNKNOWN_ID],
            ]
        )
    count = groups * 5
    return {
        "inputs": torch.tensor(inputs, dtype=torch.long),
        "positions": torch.full((count, 1), ANSWER_POSITION, dtype=torch.long),
        "targets": torch.tensor(targets, dtype=torch.long),
        "group_ids": torch.arange(groups, dtype=torch.long).repeat_interleave(5),
        "variant_ids": torch.arange(5, dtype=torch.long).repeat(groups),
        "pair_types": (torch.arange(groups, dtype=torch.long) % 2).repeat_interleave(5),
    }


def _check_shapes(tensors: dict[str, Tensor], *, development: bool) -> None:
    groups = DATA_POLICY["development_groups" if development else "construction_groups"]
    rows = groups * 5
    shapes = {
        "inputs": (rows, SEQUENCE_LENGTH),
        "positions": (rows, 1),
        "targets": (rows, 1),
        "group_ids": (rows,),
        "variant_ids": (rows,),
        "pair_types": (rows,),
    }
    if set(tensors) != set(shapes) or any(
        tensors[key].dtype != torch.long or tuple(tensors[key].shape) != shape
        for key, shape in shapes.items()
    ):
        raise ValueError("English data tensor shape or dtype differs")
    if (
        not bool((tensors["positions"] == ANSWER_POSITION).all())
        or not torch.equal(
            tensors["group_ids"], torch.arange(groups).repeat_interleave(5)
        )
        or not torch.equal(tensors["variant_ids"], torch.arange(5).repeat(groups))
        or not torch.equal(
            tensors["pair_types"], (torch.arange(groups) % 2).repeat_interleave(5)
        )
    ):
        raise ValueError("English group order, variants or query positions differ")


def _audit_split(
    tensors: dict[str, Tensor], *, development: bool
) -> tuple[dict[str, Any], set, set, Counter]:
    _check_shapes(tensors, development=development)
    groups = DATA_POLICY["development_groups" if development else "construction_groups"]
    parsed = []
    pair_counts: Counter = Counter()
    token_counts: Counter = Counter()
    worlds: set = set()
    all_pairs: set = set()
    location_counts = {kind: Counter() for kind in PAIR_TYPES}
    slot_counts = {
        kind: {"q0": Counter(), "q1": Counter(), "joint": Counter()}
        for kind in PAIR_TYPES
    }
    input_rows = tensors["inputs"].tolist()
    target_rows = tensors["targets"].flatten().tolist()
    for row, target in zip(input_rows, target_rows, strict=True):
        facts, query, answer = parse_row(row)
        if TOKEN_IDS[answer] != target:
            raise ValueError(
                "stored answer differs from the independently parsed oracle"
            )
        for owner, obj in [*(fact[:2] for fact in facts), query]:
            pair = (OWNERS.index(owner), OBJECTS.index(obj))
            if _heldout(pair) != development:
                raise ValueError("owner-object pair leaks across the global split")
            all_pairs.add(pair)
            pair_counts[f"{owner}/{obj}"] += 1
        worlds.add(tuple(sorted(facts)))
        token_counts.update(row)
        token_counts.update([target])
        parsed.append((facts, query, answer))
    for group in range(groups):
        start = group * 5
        rows = parsed[start : start + 5]
        facts, q0, _ = rows[0]
        q1 = rows[1][1]
        kind = PAIR_TYPES[group % 2]
        if (
            rows[1][0] != facts
            or rows[2][0] != rows[3][0]
            or rows[2][1] != q0
            or rows[3][1] != q1
            or rows[4][1] != q0
        ):
            raise ValueError("same-history or same-question pairing differs")
        if (kind == "same_owner" and (q0[0] != q1[0] or q0[1] == q1[1])) or (
            kind == "same_object" and (q0[1] != q1[1] or q0[0] == q1[0])
        ):
            raise ValueError("question pairing does not exercise its declared relation")
        first = next(index for index, fact in enumerate(facts) if fact[:2] == q0)
        second = next(index for index, fact in enumerate(facts) if fact[:2] == q1)
        a, x = q0
        ay = [fact for fact in facts if fact[0] == a and fact[1] != x]
        bx = [fact for fact in facts if fact[0] != a and fact[1] == x]
        if len(ay) != 1 or len(bx) != 1:
            raise ValueError(
                "base world lacks its same-owner and same-object confounds"
            )
        other = [
            index
            for index, fact in enumerate(facts)
            if fact[:2] not in (q0, ay[0][:2], bx[0][:2])
        ]
        if len(other) != 1:
            raise ValueError("base world does not have four logical bindings")
        third = other[0]
        if (
            len({a, bx[0][0], facts[third][0]}) != 3
            or len({x, ay[0][1], facts[third][1]}) != 3
            or len({fact[2] for fact in facts}) != 4
        ):
            raise ValueError(
                "world owners, objects or locations are not distinct as declared"
            )
        swapped = list(facts)
        swapped[first] = (*facts[first][:2], facts[second][2])
        swapped[second] = (*facts[second][:2], facts[first][2])
        absent = list(facts)
        absent[first] = (facts[first][0], facts[third][1], facts[first][2])
        absent[third] = (facts[third][0], facts[first][1], facts[third][2])
        if (
            tuple(swapped) != rows[2][0]
            or tuple(absent) != rows[4][0]
            or rows[4][2] != "unknown"
        ):
            raise ValueError(
                "counterfactual location swap or absent-query object swap differs"
            )
        for left, right in ((0, 2), (1, 3), (0, 4)):
            if Counter(input_rows[start + left]) != Counter(input_rows[start + right]):
                raise ValueError("counterfactual inputs changed their token bag")
        for _, _, answer in rows[:4]:
            location_counts[kind][answer] += 1
        slot_counts[kind]["q0"][str(first)] += 1
        slot_counts[kind]["q1"][str(second)] += 1
        slot_counts[kind]["joint"][f"{first},{second}"] += 1
    if any(
        set(counter) != set(LOCATIONS) or len(set(counter.values())) != 1
        for counter in location_counts.values()
    ):
        raise ValueError("location targets are not balanced within each question type")
    if any(
        set(slot_counts[kind][query]) != {"0", "1", "2", "3"}
        or len(set(slot_counts[kind][query].values())) != 1
        for kind in PAIR_TYPES
        for query in ("q0", "q1")
    ):
        raise ValueError("relevant fact slots are not balanced")
    audit = {
        "groups": groups,
        "rows": groups * 5,
        "supported_rows": groups * 4,
        "unknown_rows": groups,
        "pair_type_groups": {kind: groups // 2 for kind in PAIR_TYPES},
        "unknown_id": UNKNOWN_ID,
        "target_counts": {
            name: target_rows.count(TOKEN_IDS[name]) for name in (*LOCATIONS, "unknown")
        },
        "supported_target_counts_by_pair_type": {
            key: dict(sorted(value.items())) for key, value in location_counts.items()
        },
        "relevant_fact_slot_counts": {
            kind: {key: dict(sorted(value.items())) for key, value in counters.items()}
            for kind, counters in slot_counts.items()
        },
        "owner_object_pair_count": len(all_pairs),
        "owner_object_pair_occurrences": dict(sorted(pair_counts.items())),
        "canonical_world_count": len(worlds),
        "bag_matched_supported_pairs": groups * 2,
        "bag_matched_unknown_pairs": groups,
        "oracle_rows_checked": groups * 5,
    }
    return audit, worlds, all_pairs, token_counts


def _audit(
    construction: dict[str, Tensor], development: dict[str, Tensor]
) -> dict[str, Any]:
    train, train_worlds, train_pairs, train_tokens = _audit_split(
        construction, development=False
    )
    dev, dev_worlds, dev_pairs, _ = _audit_split(development, development=True)
    if train_worlds & dev_worlds or train_pairs & dev_pairs:
        raise ValueError(
            "construction and development share a world or owner-object pair"
        )
    if any(TOKEN_IDS[word] not in train_tokens for word in ACTIVE_VOCABULARY):
        raise ValueError(
            "an active lexical token is absent from construction inputs and targets"
        )
    return {
        "construction": train,
        "development": dev,
        "canonical_world_overlap": 0,
        "owner_object_pair_overlap": 0,
        "construction_active_token_counts": {
            word: train_tokens[TOKEN_IDS[word]] for word in ACTIVE_VOCABULARY
        },
        "active_vocabulary_covered": True,
    }


def _vocabulary_bytes() -> bytes:
    return canonical_json_bytes(
        {
            "schema": "uor-r4.english-lexical-tokenizer/1",
            "vocabulary": list(VOCABULARY),
            "active_count": len(ACTIVE_VOCABULARY),
            "bos_id": BOS_ID,
            "unknown_id": UNKNOWN_ID,
            "tokenizer": DATA_POLICY["tokenizer"],
        }
    )


def build(root: Path) -> dict[str, Any]:
    """Create a new data directory; publish its manifest after all data bytes."""
    root = Path(root).resolve()
    construction, development = (
        _build_split(development=False),
        _build_split(development=True),
    )
    audit = _audit(construction, development)
    payloads = {
        "construction.safetensors": _canonical_safetensors(construction),
        "development.safetensors": _canonical_safetensors(development),
        "vocabulary.json": _vocabulary_bytes(),
    }
    records = [
        {"path": name, "bytes": len(payload), "cid": cid_bytes(payload)}
        for name, payload in sorted(payloads.items())
    ]
    metadata = {
        "schema": SCHEMA,
        "policy": copy.deepcopy(DATA_POLICY),
        "files": records,
        "tree_cid": tree_cid(records),
        "audit": audit,
    }
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
        raise ValueError("English data manifest identity or policy differs")
    records = metadata.get("files")
    if (
        not isinstance(records, list)
        or [row.get("path") for row in records]
        != ["construction.safetensors", "development.safetensors", "vocabulary.json"]
        or metadata.get("tree_cid") != tree_cid(records)
    ):
        raise ValueError("English data file inventory differs")
    return metadata


def _payload(root: Path, metadata: dict[str, Any], name: str) -> bytes:
    record = next(row for row in metadata["files"] if row["path"] == name)
    payload = (root / name).read_bytes()
    if len(payload) != record["bytes"] or cid_bytes(payload) != record["cid"]:
        raise ValueError(f"English data file changed: {name}")
    return payload


def _load_split(
    root: Path, metadata: dict[str, Any], *, development: bool
) -> dict[str, Tensor]:
    name = "development.safetensors" if development else "construction.safetensors"
    tensors = load_safetensors(_payload(root, metadata, name))
    _check_shapes(tensors, development=development)
    return tensors


def validate(root: Path) -> dict[str, Any]:
    """Validate data identities and independently parse both populations' answers."""
    root = Path(root).resolve()
    metadata = _manifest(root)
    if _payload(root, metadata, "vocabulary.json") != _vocabulary_bytes():
        raise ValueError("English lexical vocabulary differs")
    audit = _audit(
        _load_split(root, metadata, development=False),
        _load_split(root, metadata, development=True),
    )
    if audit != metadata["audit"]:
        raise ValueError("English data semantic audit differs")
    return metadata


def load_training(root: Path, *, mixed: bool) -> dict[str, Tensor]:
    """Load construction only; supported-only phase excludes all unknown rows."""
    root = Path(root).resolve()
    tensors = _load_split(root, _manifest(root), development=False)
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
    """Load canonical grouped development rows; metadata stays outside the model."""
    root = Path(root).resolve()
    return _load_split(root, _manifest(root), development=True)
