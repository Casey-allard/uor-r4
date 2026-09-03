"""Clause variations over the already observed #1073 binding worlds."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_compound_binding import data as retained
from ..zoology_english_binding import data as english
from ..zoology_release.development import _canonical_safetensors

MAXLEN = 13
PAD_ID = 57
UNKNOWN_ID = english.UNKNOWN_ID
VOCABULARY = (
    english.VOCABULARY[:52]
    + ("not", "but", ",", "owned", "by", "<pad>")
    + english.VOCABULARY[58:]
)
TOKEN_IDS = {word: index for index, word in enumerate(VOCABULARY)}
ROLE_NAMES = ("owner", "object", "location")
VIEW_COMBINATIONS = ((0, 0), (1, 1), (0, 1), (1, 0))
FACT_TEMPLATES = (
    "O , not D , put the X in the L .",
    "in the L , not D but O put the X .",
    "not D but O put the X in the L .",
    "in the L , O , not D , put the X .",
)
QUERY_TEMPLATE = "where is the X owned by O , not D ? answer :"
SCHEMA = "uor-r4.language-interface-data/1"
SOURCE_MANIFEST_CID = (
    "blake3:574d667e61b70e32c39b26d43547d5aeb29e92f16260fa840ddd4eda30c4e694"
)
SOURCE_TREE_CID = (
    "blake3:c150f19e02caaa7537e2fd244ebd3dae27f6e561c8869b69c91b93351044a6da"
)
_FILES = ("construction.safetensors", "development.safetensors", "vocabulary.json")
DATA_POLICY = {
    "name": "ObservedWorldCompositionalClauseViewsV1",
    "issue": 1077,
    "source_issue": 1073,
    "semantic_population": "unchanged, already observed #1073 construction and development worlds",
    "construction_source_rows": 10240,
    "development_source_rows": 1280,
    "construction_rows": 20480,
    "development_rows": 5120,
    "construction_views": [0, 1],
    "development_seen_views": [0, 1],
    "development_heldout_views": [2, 3],
    "view_combinations": [list(pair) for pair in VIEW_COMBINATIONS],
    "fact_templates": list(FACT_TEMPLATES),
    "query_template": QUERY_TEMPLATE,
    "clause_placement": ["owner phrase first", "location phrase first"],
    "owner_phrase": ["O, not D,", "not D but O"],
    "syntax_split": "construction diagonal C0R0/C1R1; development matched seen diagonal and held-out off-diagonal C0R1/C1R0",
    "query_scope": "one seen query template in every view",
    "fact_distractor": "owner index plus four modulo sixteen; same owner-object partition",
    "query_distractor": "same-object family uses reciprocal queried owners; same-owner family uses lowest-index other base owner preserving both pair partitions, else owner plus four; absence reuses q0 distractor",
    "partition": "every actual or negated owner-object mention preserves (owner_index + object_index) % 4 == 0 iff development",
    "row_order": "view-major, then unchanged source group-major variants 0,1,2,3,4",
    "fact_order": "unchanged displayed source order; no added rotations",
    "segments": 5,
    "max_length": MAXLEN,
    "padding_id": PAD_ID,
    "role_order": list(ROLE_NAMES),
    "role_labels_per_row": 14,
    "query_location_label": -100,
    "reader_vocabulary_size": 4096,
    "reader_aliases": {str(index): VOCABULARY[index] for index in range(52, 58)},
    "core_vocabulary": "all 4096 source IDs and frozen embeddings unchanged; new aliases belong only to the separate reader",
    "next_word_target": "source answer is the next word after answer colon, outside every input",
    "model_inputs": ["inputs", "lengths"],
    "supervision_only": ["role_positions", "targets"],
    "default_validation": "construction semantics and all payload hashes; never deserialize or regenerate development",
    "fresh_semantic_generalization": False,
}


def _record(path: str, payload: bytes) -> dict:
    return {"path": path, "bytes": len(payload), "cid": cid_bytes(payload)}


def _held(owner: str, obj: str) -> bool:
    return (english.OWNERS.index(owner) + english.OBJECTS.index(obj)) % 4 == 0


def _owner_plus_four(owner: str) -> str:
    return english.OWNERS[(english.OWNERS.index(owner) + 4) % 16]


def _query_distractors(
    base_facts: list, q0: tuple, q1: tuple, pair_type: int
) -> tuple[tuple[str, str], bool]:
    if pair_type == 1:
        if q0[0] == q1[0] or q0[1] != q1[1]:
            raise ValueError("same-object source questions differ")
        return (q1[0], q0[0]), False
    if pair_type != 0 or q0[0] != q1[0] or q0[1] == q1[1]:
        raise ValueError("same-owner source questions differ")
    eligible = sorted(
        {
            owner
            for owner, _, _ in base_facts
            if owner != q0[0]
            and all(_held(owner, obj) == _held(q0[0], obj) for obj in (q0[1], q1[1]))
        },
        key=english.OWNERS.index,
    )
    distractor = eligible[0] if eligible else _owner_plus_four(q0[0])
    return (distractor, distractor), not bool(eligible)


def _clause(template: str, values: dict[str, str]) -> tuple[list[int], list[int]]:
    words = template.split()
    roles = [
        words.index(symbol) if symbol in words else -100 for symbol in ("O", "X", "L")
    ]
    ids = [TOKEN_IDS[values.get(word, word)] for word in words]
    return ids, roles


def _parse_clause(ids: list[int], *, query: bool) -> tuple[dict, list[int], int]:
    """Derive labels from the rendered words, independently of saved metadata."""
    if any(not 0 <= token < 4096 for token in ids):
        raise ValueError("rendered token outside vocabulary")
    words = [VOCABULARY[token] for token in ids]
    candidates = (QUERY_TEMPLATE,) if query else FACT_TEMPLATES
    for view, template in enumerate(candidates):
        expected = template.split()
        if len(words) != len(expected):
            continue
        values = {}
        for symbol, word in zip(expected, words, strict=True):
            if symbol in ("O", "D", "X", "L"):
                values[symbol] = word
            elif symbol != word:
                break
        else:
            if (
                values["O"] not in english.OWNERS
                or values["D"] not in english.OWNERS
                or values["O"] == values["D"]
                or values["X"] not in english.OBJECTS
                or (not query and values["L"] not in english.LOCATIONS)
            ):
                raise ValueError("invalid rendered role lexicon or equal distractor")
            roles = [
                expected.index(symbol) if symbol in expected else -100
                for symbol in ("O", "X", "L")
            ]
            return values, roles, view
    raise ValueError("rendered clause does not match declared lexical grammar")


def decode(ids: list[int]) -> str:
    """Readable lexical tokens; padding is omitted, with all IDs reversible."""
    return " ".join(VOCABULARY[int(token)] for token in ids if int(token) != PAD_ID)


def _source_rows(source: dict[str, Tensor]) -> tuple[list, list, list, list]:
    rows = source["inputs"].tolist()
    groups = source["group_ids"].tolist()
    variants = source["variant_ids"].tolist()
    pair_types = source["pair_types"].tolist()
    if (
        not rows
        or len(rows) % 5
        or groups != [row // 5 for row in range(len(rows))]
        or variants != [row % 5 for row in range(len(rows))]
        or pair_types != [(row // 5) % 2 for row in range(len(rows))]
    ):
        raise ValueError("source canonical group/variant/type order differs")
    return rows, groups, variants, pair_types


def _render_population(
    source: dict[str, Tensor], views: tuple[int, ...]
) -> dict[str, Tensor]:
    """Pure rendering helper; no random numbers, source changes or model access."""
    rows, _, _, pair_types = _source_rows(source)
    parsed = [english.parse_row(row) for row in rows]
    choices = {}
    for start in range(0, len(rows), 5):
        choices[start // 5] = _query_distractors(
            parsed[start][0], parsed[start][1], parsed[start + 1][1], pair_types[start]
        )[0]
    all_inputs, all_lengths, all_roles = [], [], []
    for view in views:
        if view not in range(4):
            raise ValueError("unknown clause view")
        for index, (facts, query, _) in enumerate(parsed):
            segments, lengths, positions = [], [], []
            for owner, obj, location in facts:
                ids, roles = _clause(
                    FACT_TEMPLATES[view],
                    {
                        "O": owner,
                        "D": _owner_plus_four(owner),
                        "X": obj,
                        "L": location,
                    },
                )
                lengths.append(len(ids))
                segments.append(ids + [PAD_ID] * (MAXLEN - len(ids)))
                positions.append(roles)
            distractors = choices[index // 5]
            distractor = distractors[1 if index % 5 in (1, 3) else 0]
            ids, roles = _clause(
                QUERY_TEMPLATE, {"O": query[0], "X": query[1], "D": distractor}
            )
            segments.append(ids)
            lengths.append(len(ids))
            positions.append(roles)
            all_inputs.append(segments)
            all_lengths.append(lengths)
            all_roles.append(positions)
    repeats = len(views)
    result = {
        "inputs": torch.tensor(all_inputs, dtype=torch.long),
        "lengths": torch.tensor(all_lengths, dtype=torch.long),
        "role_positions": torch.tensor(all_roles, dtype=torch.long),
        "canonical_inputs": source["inputs"].repeat(repeats, 1),
        "targets": source["targets"].repeat(repeats, 1),
        "view_ids": torch.tensor(
            [view for view in views for _ in rows], dtype=torch.long
        ),
    }
    for key in ("group_ids", "variant_ids", "pair_types"):
        result[key] = source[key].repeat(repeats)
    return result


def _histogram(values: list) -> dict[str, int]:
    return {str(key): value for key, value in sorted(Counter(values).items())}


def _check_shapes(tensors: dict[str, Tensor]) -> int:
    count = tensors["inputs"].shape[0]
    shapes = {
        "inputs": (count, 5, MAXLEN),
        "lengths": (count, 5),
        "role_positions": (count, 5, 3),
        "canonical_inputs": (count, 41),
        "targets": (count, 1),
        "view_ids": (count,),
        "group_ids": (count,),
        "variant_ids": (count,),
        "pair_types": (count,),
    }
    if set(tensors) != set(shapes) or count == 0:
        raise ValueError("language-interface tensor inventory differs")
    for key, shape in shapes.items():
        value = tensors[key]
        if (
            value.shape != shape
            or value.dtype != torch.long
            or value.device.type != "cpu"
        ):
            raise ValueError(f"language-interface tensor shape/type differs: {key}")
    return count


def _audit_population(tensors: dict[str, Tensor], *, development: bool) -> dict:
    count = _check_shapes(tensors)
    views = (0, 1, 2, 3) if development else (0, 1)
    if count % (5 * len(views)):
        raise ValueError("language-interface population is not complete groups/views")
    per_view = count // len(views)
    values = {key: tensor.tolist() for key, tensor in tensors.items()}
    canonical_first = values["canonical_inputs"][:per_view]
    expected_groups = [row // 5 for row in range(per_view)] * len(views)
    if (
        values["view_ids"] != [view for view in views for _ in range(per_view)]
        or values["group_ids"] != expected_groups
        or values["variant_ids"] != [row % 5 for row in range(per_view)] * len(views)
        or values["pair_types"] != [group % 2 for group in expected_groups]
        or values["canonical_inputs"] != canonical_first * len(views)
    ):
        raise ValueError("language-interface canonical ordering differs")
    parsed = [english.parse_row(row) for row in canonical_first]
    distractors, fallbacks = {}, 0
    for start in range(0, per_view, 5):
        choice, fallback = _query_distractors(
            parsed[start][0], parsed[start][1], parsed[start + 1][1], (start // 5) % 2
        )
        distractors[start // 5] = choice
        fallbacks += fallback
    audit_views = {}
    for view_number, view in enumerate(views):
        offset = view_number * per_view
        role_hist = {name: [] for name in ROLE_NAMES}
        query_hist = {name: [] for name in ROLE_NAMES[:2]}
        lengths, target_slots, target_ids, actual_pairs, negative_pairs = (
            [],
            [],
            [],
            set(),
            set(),
        )
        owner_counts, distractor_counts = [], []
        same_bag_question_pairs = 0
        for local in range(per_view):
            index = offset + local
            canonical_facts, canonical_query, canonical_answer = parsed[local]
            derived_facts = []
            for segment in range(5):
                length = values["lengths"][index][segment]
                ids = values["inputs"][index][segment]
                if not 1 <= length <= MAXLEN or ids[length:] != [PAD_ID] * (
                    MAXLEN - length
                ):
                    raise ValueError("rendered clause length/padding differs")
                if PAD_ID in ids[:length]:
                    raise ValueError("padding occurs within a valid clause")
                roles, positions, parsed_view = _parse_clause(
                    ids[:length], query=segment == 4
                )
                if values["role_positions"][index][segment] != positions:
                    raise ValueError(
                        "rendered role supervision differs from parsed words"
                    )
                owner, obj, negative = roles["O"], roles["X"], roles["D"]
                if (
                    _held(owner, obj) != development
                    or _held(negative, obj) != development
                ):
                    raise ValueError(
                        "rendered positive/negated pair crosses the source partition"
                    )
                actual_pairs.add((owner, obj))
                negative_pairs.add((negative, obj))
                owner_counts.append(owner)
                distractor_counts.append(negative)
                if segment < 4:
                    if parsed_view != view or negative != _owner_plus_four(owner):
                        raise ValueError("rendered fact view or distractor differs")
                    derived_facts.append((owner, obj, roles["L"]))
                    lengths.append(length)
                    for name, position in zip(ROLE_NAMES, positions, strict=True):
                        role_hist[name].append(position)
                else:
                    query = (owner, obj)
                    expected = distractors[local // 5][1 if local % 5 in (1, 3) else 0]
                    if negative != expected or length != MAXLEN:
                        raise ValueError("rendered query distractor or length differs")
                    for name, position in zip(
                        ROLE_NAMES[:2], positions[:2], strict=True
                    ):
                        query_hist[name].append(position)
            bindings = {
                (owner, obj): location for owner, obj, location in derived_facts
            }
            if len(bindings) != 4:
                raise ValueError("rendered facts contain duplicate bindings")
            answer = bindings.get(query, "unknown")
            if (
                tuple(derived_facts) != canonical_facts
                or query != canonical_query
                or answer != canonical_answer
                or values["targets"][index] != [TOKEN_IDS[answer]]
                or (answer == "unknown") != (local % 5 == 4)
            ):
                raise ValueError(
                    "rendered semantic oracle differs from source or target"
                )
            target_ids.append(TOKEN_IDS[answer])
            if answer != "unknown":
                target_slots.append(
                    next(
                        slot
                        for slot, fact in enumerate(derived_facts)
                        if fact[:2] == query
                    )
                )
        for start in range(0, per_view, 5):
            rows = values["inputs"][offset + start : offset + start + 5]
            labels = values["targets"][offset + start : offset + start + 5]
            for left, right in ((0, 1), (2, 3)):
                if rows[left][:4] != rows[right][:4] or labels[left] == labels[right]:
                    raise ValueError(
                        "question counterfactual loses fixed facts or distinct answers"
                    )
                if (start // 5) % 2 == 1:
                    if Counter(rows[left][4]) != Counter(rows[right][4]):
                        raise ValueError(
                            "same-object owner questions lose identical token bags"
                        )
                    same_bag_question_pairs += 1
            for left, right in ((0, 2), (1, 3), (0, 4)):
                if rows[left][4] != rows[right][4] or Counter(
                    token for clause in rows[left] for token in clause
                ) != Counter(token for clause in rows[right] for token in clause):
                    raise ValueError(
                        "history counterfactual loses fixed query or token bag"
                    )
        audit_views[str(view)] = {
            "combination": list(VIEW_COMBINATIONS[view]),
            "rows": per_view,
            "groups": per_view // 5,
            "supported": per_view * 4 // 5,
            "unknown": per_view // 5,
            "pair_type_rows": _histogram(
                values["pair_types"][offset : offset + per_view]
            ),
            "target_histogram": _histogram(target_ids),
            "target_displayed_slots": _histogram(target_slots),
            "fact_length_histogram": _histogram(lengths),
            "query_length_histogram": {str(MAXLEN): per_view},
            "fact_role_positions": {
                name: _histogram(items) for name, items in role_hist.items()
            },
            "query_role_positions": {
                name: _histogram(items) for name, items in query_hist.items()
            },
            "same_object_identical_query_bag_pairs": same_bag_question_pairs,
            "fixed_fact_different_question_pairs": per_view * 2 // 5,
            "fixed_query_same_bag_history_pairs": per_view * 3 // 5,
            "positive_owner_object_pairs": len(actual_pairs),
            "negated_owner_object_pairs": len(negative_pairs),
            "partition_violations": 0,
            "positive_owner_tokens": _histogram(owner_counts),
            "negated_owner_tokens": _histogram(distractor_counts),
        }
    return {
        "rows": count,
        "source_rows": per_view,
        "groups_per_view": per_view // 5,
        "role_labels": 14 * count,
        "same_owner_distractor_fallback_groups": fallbacks,
        "canonical_source_tensor_cid": cid_bytes(
            _canonical_safetensors({"inputs": tensors["canonical_inputs"][:per_view]})
        ),
        "views": audit_views,
    }


def _input_set(tensors: dict[str, Tensor]) -> set[bytes]:
    return {row.numpy().tobytes() for row in tensors["inputs"]}


def _audit_joint(
    construction: dict[str, Tensor], development: dict[str, Tensor]
) -> dict:
    overlap = len(_input_set(construction) & _input_set(development))
    if overlap:
        raise ValueError("rendered construction/development exact inputs overlap")
    return {
        "construction_development_exact_input_overlap": overlap,
        "construction_combinations": [[0, 0], [1, 1]],
        "development_seen_combinations": [[0, 0], [1, 1]],
        "development_heldout_combinations": [[0, 1], [1, 0]],
        "construction_heldout_combination_overlap": 0,
        "development_same_semantic_rows_across_all_views": True,
        "fresh_semantic_worlds": 0,
    }


def _vocabulary_bytes() -> bytes:
    return canonical_json_bytes(
        {
            "vocabulary": list(VOCABULARY),
            "padding_id": PAD_ID,
            "core_vocabulary": list(english.VOCABULARY),
            "reader_aliases": DATA_POLICY["reader_aliases"],
            "source_core_vocabulary_changed": False,
        }
    )


def _source_contract(source_root: Path) -> dict:
    data_root = source_root.resolve() / "data"
    payload = (data_root / "manifest.json").read_bytes()
    manifest = json.loads(payload)
    body = dict(manifest)
    expected = body.pop("manifest_cid", None)
    records = artifact_records(data_root, _FILES)
    if (
        payload != canonical_json_bytes(manifest)
        or expected != SOURCE_MANIFEST_CID
        or cid_bytes(canonical_json_bytes(body)) != SOURCE_MANIFEST_CID
        or manifest["tree_cid"] != SOURCE_TREE_CID
        or tree_cid(records) != SOURCE_TREE_CID
        or records != manifest["files"]
    ):
        raise ValueError("source #1073 observed dataset identity differs")
    return {
        "issue": 1073,
        "root": str(source_root.resolve()),
        "manifest_cid": SOURCE_MANIFEST_CID,
        "tree_cid": SOURCE_TREE_CID,
        "manifest": _record("manifest.json", payload),
        "files": records,
    }


def _write_package(
    root: Path, source: dict, construction: dict, development: dict
) -> dict:
    """Serialize already rendered tensors; useful for tiny synthetic boundary tests."""
    audit = {
        "construction": _audit_population(construction, development=False),
        "development": _audit_population(development, development=True),
        "joint": _audit_joint(construction, development),
    }
    root.mkdir(parents=True, exist_ok=False)
    payloads = {
        "construction.safetensors": _canonical_safetensors(construction),
        "development.safetensors": _canonical_safetensors(development),
        "vocabulary.json": _vocabulary_bytes(),
    }
    files = []
    for name in sorted(payloads):
        with (root / name).open("xb") as handle:
            handle.write(payloads[name])
        files.append(_record(name, payloads[name]))
    body = {
        "schema": SCHEMA,
        "policy": DATA_POLICY,
        "source": source,
        "files": files,
        "tree_cid": tree_cid(files),
        "audit": audit,
    }
    body["manifest_cid"] = cid_bytes(canonical_json_bytes(body))
    with (root / "manifest.json").open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def prepare(root: Path, source_root: Path) -> dict[str, Any]:
    """Create exactly once, before fitting; read only already observed #1073 data."""
    if root.exists():
        raise FileExistsError("language-interface data already exists")
    source = _source_contract(source_root)
    original_construction = retained.load_construction(source_root / "data")
    original_development = retained.load_development(source_root / "data")
    if original_construction["inputs"].shape != (10240, 41) or original_development[
        "inputs"
    ].shape != (1280, 41):
        raise ValueError("source #1073 population size differs")
    construction = _render_population(original_construction, (0, 1))
    development = _render_population(original_development, (0, 1, 2, 3))
    return _write_package(root, source, construction, development)


def _manifest(root: Path) -> dict:
    payload = (root / "manifest.json").read_bytes()
    value = json.loads(payload)
    body = dict(value)
    cid = body.pop("manifest_cid", None)
    if (
        payload != canonical_json_bytes(value)
        or cid != cid_bytes(canonical_json_bytes(body))
        or value["schema"] != SCHEMA
        or value["policy"] != DATA_POLICY
        or [row["path"] for row in value["files"]] != list(_FILES)
        or value["tree_cid"] != tree_cid(value["files"])
        or value["source"]["manifest_cid"] != SOURCE_MANIFEST_CID
        or value["source"]["tree_cid"] != SOURCE_TREE_CID
        or tree_cid(value["source"]["files"]) != SOURCE_TREE_CID
    ):
        raise ValueError("language-interface data manifest differs")
    return value


def _payload(root: Path, manifest: dict, name: str) -> bytes:
    payload = (root / name).read_bytes()
    if _record(name, payload) != next(
        row for row in manifest["files"] if row["path"] == name
    ):
        raise ValueError(f"language-interface payload differs: {name}")
    return payload


def _load(root: Path, manifest: dict, name: str) -> dict[str, Tensor]:
    tensors = load_safetensors(_payload(root, manifest, name))
    _check_shapes(tensors)
    return tensors


def validate(root: Path, *, inspect_development: bool = False) -> dict[str, Any]:
    """Default checks hash the closed development payload without tensor access."""
    value = _manifest(root)
    if artifact_records(root, _FILES) != value["files"]:
        raise ValueError("language-interface dataset payload identity differs")
    if _payload(root, value, "vocabulary.json") != _vocabulary_bytes():
        raise ValueError("language-interface lexical aliases differ")
    construction = _load(root, value, "construction.safetensors")
    if (
        _audit_population(construction, development=False)
        != value["audit"]["construction"]
    ):
        raise ValueError("language-interface construction audit differs")
    if inspect_development:
        development = _load(root, value, "development.safetensors")
        if (
            _audit_population(development, development=True)
            != value["audit"]["development"]
            or _audit_joint(construction, development) != value["audit"]["joint"]
        ):
            raise ValueError("language-interface development audit differs")
    return value


def load_construction(root: Path) -> dict[str, Tensor]:
    return _load(root, _manifest(root), "construction.safetensors")


def load_development(root: Path) -> dict[str, Tensor]:
    return _load(root, _manifest(root), "development.safetensors")
