"""Fresh C1-SB5 paired-query candidate-matrix data.

The unit of supervision in this module is one exact source with two questions
and one shared, ordered set of exact-text candidate groups.  Every affirmative
candidate is positive for exactly one question and negative for the other, so
question-independent candidate syntax cannot satisfy a complete pair.

Population construction is pure and deterministic.  Tokenizer binding is a
separate, explicit step because the source-token anchors are part of the model
contract and must be derived with the pinned #1017 tokenizer before training.
Product text is returned in a separate envelope and is never named by the
training-view whitelist.
"""

from __future__ import annotations

from collections import Counter, defaultdict
from collections.abc import Mapping, Sequence
from copy import deepcopy
import json
from pathlib import Path
import struct
from typing import Any

from tokenizers import Tokenizer

from .constants import BOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .provenance import canonical_json_bytes, cid_bytes
from .source_relation_adapter_data import (
    ABSTAIN,
    CONTRADICTION,
    LexicalWorld,
    SOURCE_WIDTHS,
    _world,
    _world_inventory,
    _world_records,
)
from .source_relation_data import split_sentence_spans


ISSUE = 954
POLICY = "R4PairedQueryCandidateMatrixV1"

PAIR_SCHEMA = "uor-r4.paired-query-binding-pair/1"
QUERY_SCHEMA = "uor-r4.paired-query-binding-query/1"
DATASET_SCHEMA = "uor-r4.paired-query-binding-dataset/1"
PREFLIGHT_SCHEMA = "uor-r4.paired-query-binding-preflight/1"
PRODUCT_SCHEMA = "uor-r4.paired-query-binding-products/1"
CENSUS_SCHEMA = "uor-r4.paired-query-binding-census/1"
SPLIT_SCHEMA = "uor-r4.paired-query-binding-split/1"
TOKENIZER_CENSUS_SCHEMA = "uor-r4.paired-query-binding-tokenizer-census/1"

DATASET_FILENAME = "paired-query-binding-dataset.json"
PREFLIGHT_FILENAME = "paired-query-binding-preflight.json"
CENSUS_FILENAME = "paired-query-binding-census.json"
TOKENIZER_CENSUS_FILENAME = "paired-query-binding-tokenizer-census.json"
PRODUCT_FILENAME = "paired-query-binding-product-probes.json"
TRAINING_VIEW_MANIFEST_FILENAME = "training-view-manifest.json"
PRODUCT_MANIFEST_FILENAME = "product-commitments-manifest.json"
TRAINING_VIEW_FILENAMES = (
    CENSUS_FILENAME,
    DATASET_FILENAME,
    PREFLIGHT_FILENAME,
    TOKENIZER_CENSUS_FILENAME,
)
PRODUCT_DENIED_FILENAMES = (PRODUCT_FILENAME, PRODUCT_MANIFEST_FILENAME)

INPUT_POLICY = (
    "exact UTF-8 `E:<exact full source>\\nQ:<question>\\nBind:` with no "
    "terminal newline; prepend one pinned BOS token; candidate states are the "
    "terminal punctuation states of the earliest occurrence of each distinct "
    "exact-text group and precede Q; the query state is the final Bind colon"
)
QUESTION_POLICY = "Where is the <subject>?"
SENTENCE_POLICY = "exact .!? terminated UTF-8 byte spans"

FRESH_WORLD_ORDINAL_START = 162
PREFLIGHT_FIT_WORLDS_PER_WIDTH = 2
PREFLIGHT_SEALED_WORLDS_PER_WIDTH = 1
PAIRS_PER_WORLD = 4
QUERIES_PER_PAIR = 2
MAX_POSITIONS_INCLUDING_BOS = FROZEN_MODEL_CONFIG.max_position_embeddings

PAIR_KINDS = (
    "matched-primary-secondary-answers",
    "primary-conflict-secondary-abstain",
    "secondary-conflict-primary-abstain",
    "duplicate-primary-agreement-secondary-abstain",
)

EXPECTED_COUNTS = {
    "fit": {
        "pairs": 56,
        "query_rows": 112,
        "candidate_groups": 266,
        "query_candidate_cells": 532,
        "positive_cells": 98,
        "negative_cells": 434,
        "required_flips": 98,
        "outcomes": {"answer": 42, "abstain": 42, "conflict": 28},
        "duplicate_pairs": 14,
    },
    "sealed": {
        "pairs": 28,
        "query_rows": 56,
        "candidate_groups": 133,
        "query_candidate_cells": 266,
        "positive_cells": 49,
        "negative_cells": 217,
        "required_flips": 49,
        "outcomes": {"answer": 21, "abstain": 21, "conflict": 14},
        "duplicate_pairs": 7,
    },
    "product": {
        "pairs": 4,
        "query_rows": 8,
        "candidate_groups": 11,
        "query_candidate_cells": 22,
        "positive_cells": 7,
        "negative_cells": 15,
        "required_flips": 7,
        "outcomes": {"answer": 3, "abstain": 3, "conflict": 2},
        "duplicate_pairs": 1,
    },
}


def _canonical_with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def artifact_bytes(value: Mapping[str, Any]) -> bytes:
    """Return the one canonical JSON representation used by campaign manifests."""
    return canonical_json_bytes(value)


def verify_artifact_cid(value: Mapping[str, Any], field: str) -> str:
    """Reproduce one self-CID and fail closed on schema/artifact drift."""
    unsigned = dict(value)
    expected = unsigned.pop(field, None)
    if not isinstance(expected, str):
        raise ValueError(f"artifact omits self-CID field {field}")
    actual = cid_bytes(canonical_json_bytes(unsigned))
    if actual != expected:
        raise ValueError(f"artifact {field} does not reproduce")
    return actual


def load_artifact(path: Path, *, schema: str, cid_field: str) -> dict[str, Any]:
    """Load a canonical JSON artifact with an exact schema and reproduced CID."""
    raw = path.read_bytes()
    value = json.loads(raw)
    if not isinstance(value, dict) or value.get("schema") != schema:
        raise ValueError(f"artifact schema differs at {path}")
    if raw != canonical_json_bytes(value):
        raise ValueError(f"artifact is not canonical JSON at {path}")
    verify_artifact_cid(value, cid_field)
    return value


def render_paired_query_input(source: str, question: str) -> str:
    """Render one lane; no candidate text is repeated after the source."""
    if not source or source != source.strip():
        raise ValueError("paired-query source must be nonempty and trimmed")
    spans = split_sentence_spans(source)
    if not spans or " ".join(str(span["text"]) for span in spans) != source:
        raise ValueError("paired-query source must be exact terminated spans")
    if (
        not question.startswith("Where is the ")
        or not question.endswith("?")
        or question != question.strip()
    ):
        raise ValueError("paired-query question differs from the frozen policy")
    value = f"E:{source}\nQ:{question}\nBind:"
    if value.endswith("\n"):
        raise AssertionError("paired-query renderer added a terminal newline")
    return value


def _candidate_subject(text: str, world: LexicalWorld) -> str:
    matches = [subject for subject in world.subjects if text.startswith(f"The {subject} ")]
    if len(matches) != 1:
        raise ValueError("candidate sentence does not bind exactly one world subject")
    return matches[0]


def _candidate_groups(
    record: Mapping[str, Any], world: LexicalWorld
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    source = str(record["source"])
    source_bytes = source.encode("utf-8")
    parsed = split_sentence_spans(source)
    spans = list(record["sentence_spans"])
    if len(parsed) != len(spans) or len(spans) != int(record["source_width"]):
        raise ValueError("paired-query source width/span count differs")

    occurrences: list[dict[str, Any]] = []
    grouped: dict[str, dict[str, Any]] = {}
    for expected_index, (parsed_span, inherited) in enumerate(zip(parsed, spans)):
        text = str(inherited["text"])
        byte_start = int(inherited["byte_start"])
        byte_end = int(inherited["byte_end"])
        text_cid = cid_bytes(text.encode("utf-8"))
        if (
            parsed_span["text"] != text
            or int(parsed_span["byte_start"]) != byte_start
            or int(parsed_span["byte_end"]) != byte_end
            or source_bytes[byte_start:byte_end] != text.encode("utf-8")
            or inherited["candidate_index"] != expected_index
            or inherited["relation_group_cid"] != text_cid
        ):
            raise ValueError("paired-query inherited exact span differs")
        subject = _candidate_subject(text, world)
        semantic_kind = str(inherited["semantic_kind"])
        occurrence = {
            "candidate_index": expected_index,
            "byte_start": byte_start,
            "byte_end": byte_end,
            "terminal_byte_end": byte_end,
            "text": text,
            "text_cid": text_cid,
            "relation_group_cid": text_cid,
            "role": str(inherited["role"]),
            "semantic_kind": semantic_kind,
            "candidate_subject": subject,
            "candidate_subject_cid": cid_bytes(subject.encode("utf-8")),
        }
        occurrences.append(occurrence)
        group = grouped.get(text_cid)
        if group is None:
            grouped[text_cid] = {
                "relation_group_cid": text_cid,
                "text": text,
                "text_cid": text_cid,
                "semantic_kind": semantic_kind,
                "candidate_subject": subject,
                "candidate_subject_cid": occurrence["candidate_subject_cid"],
                "occurrence_indices": [expected_index],
                "earliest_occurrence_index": expected_index,
                "earliest_terminal_byte_end": byte_end,
            }
        else:
            if (
                group["text"] != text
                or group["semantic_kind"] != semantic_kind
                or group["candidate_subject"] != subject
            ):
                raise ValueError("duplicate exact-text candidate metadata disagrees")
            group["occurrence_indices"].append(expected_index)

    ordered = sorted(grouped.values(), key=lambda group: group["earliest_occurrence_index"])
    for group_index, group in enumerate(ordered):
        group["group_index"] = group_index
        group["source_scoped_group_cid"] = cid_bytes(
            canonical_json_bytes(
                {
                    "source_cid": record["source_cid"],
                    "relation_group_cid": group["relation_group_cid"],
                }
            )
        )
    return occurrences, ordered


def _query_row(
    *,
    source: str,
    query_lane: int,
    question: str,
    subject: str,
    groups: Sequence[Mapping[str, Any]],
    inherited: Mapping[str, Any] | None,
) -> dict[str, Any]:
    if question != f"Where is the {subject}?":
        raise ValueError("paired-query question/subject binding differs")
    labels = [
        int(
            group["semantic_kind"] == "locative"
            and group["candidate_subject"] == subject
        )
        for group in groups
    ]
    positive_indices = [index for index, label in enumerate(labels) if label]
    positive_group_cids = [
        str(groups[index]["relation_group_cid"]) for index in positive_indices
    ]
    outcome = (
        "abstain"
        if not positive_indices
        else "answer"
        if len(positive_indices) == 1
        else "conflict"
    )
    target_group_index = positive_indices[0] if outcome == "answer" else None
    target_span_index = (
        int(groups[target_group_index]["earliest_occurrence_index"])
        if target_group_index is not None
        else None
    )
    answer = (
        str(groups[target_group_index]["text"])
        if target_group_index is not None
        else ABSTAIN
        if outcome == "abstain"
        else CONTRADICTION
    )
    positive_span_indices = [
        int(occurrence)
        for index in positive_indices
        for occurrence in groups[index]["occurrence_indices"]
    ]
    relation_input = render_paired_query_input(source, question)
    value: dict[str, Any] = {
        "schema": QUERY_SCHEMA,
        "query_lane": query_lane,
        "question": question,
        "question_cid": cid_bytes(question.encode("utf-8")),
        "subject": subject,
        "subject_cid": cid_bytes(subject.encode("utf-8")),
        "relation_input": relation_input,
        "relation_input_cid": cid_bytes(relation_input.encode("utf-8")),
        "labels": labels,
        "positive_relation_group_cids": positive_group_cids,
        "positive_span_indices": positive_span_indices,
        "target_outcome": outcome,
        "target_group_index": target_group_index,
        "target_group_cid": (
            groups[target_group_index]["relation_group_cid"]
            if target_group_index is not None
            else None
        ),
        "target_span_index": target_span_index,
        "answer": answer,
        "duplicate_agreement": bool(
            outcome == "answer"
            and target_group_index is not None
            and len(groups[target_group_index]["occurrence_indices"]) > 1
        ),
    }
    if inherited is not None:
        inherited_labels: dict[str, int] = {}
        for span in inherited["sentence_spans"]:
            group_cid = str(span["relation_group_cid"])
            prior = inherited_labels.setdefault(group_cid, int(span["relation_label"]))
            if prior != int(span["relation_label"]):
                raise ValueError("inherited duplicate labels disagree")
        expected_labels = [
            inherited_labels[str(group["relation_group_cid"])] for group in groups
        ]
        if (
            inherited["source"] != source
            or inherited["question"] != question
            or inherited["subject"] != subject
            or inherited["target_outcome"] != outcome
            or expected_labels != labels
            or inherited["target_span_index"] != target_span_index
            or inherited["answer"] != answer
        ):
            raise ValueError("fresh paired-query oracle differs from inherited row")
        value["inherited_record_cid"] = inherited["record_cid"]
    else:
        value["inherited_record_cid"] = None
        value["oracle"] = (
            "affirmative exact locative candidate whose generated subject equals "
            "the query subject"
        )
    return _canonical_with_cid(value, "query_row_cid")


def _pair_from_world(
    world: LexicalWorld, *, population: str, pair_slot: int
) -> dict[str, Any]:
    if pair_slot not in range(PAIRS_PER_WORLD):
        raise ValueError("paired-query pair slot is outside 0..3")
    rows = _world_records(world, population=population)
    if len(rows) != 9:
        raise RuntimeError("paired-query source world no longer has nine base rows")
    inherited_rows: tuple[Mapping[str, Any] | None, Mapping[str, Any] | None]
    if pair_slot == 0:
        inherited_rows = (rows[0], rows[1])
    elif pair_slot == 1:
        inherited_rows = (rows[6], rows[4])
    elif pair_slot == 2:
        inherited_rows = (rows[5], rows[7])
    else:
        inherited_rows = (rows[2], None)

    base = inherited_rows[0]
    if base is None:
        raise AssertionError("paired-query pair has no base source")
    source = str(base["source"])
    source_cid = cid_bytes(source.encode("utf-8"))
    for inherited in inherited_rows:
        if inherited is not None and (
            inherited["source"] != source or inherited["source_cid"] != source_cid
        ):
            raise ValueError("paired-query inherited rows do not share exact source bytes")

    occurrences, groups = _candidate_groups(base, world)
    first = inherited_rows[0]
    second = inherited_rows[1]
    if first is None:
        raise AssertionError("paired-query first query cannot be synthetic")
    query_specs: tuple[tuple[str, str, Mapping[str, Any] | None], ...] = (
        (str(first["question"]), str(first["subject"]), first),
        (
            str(second["question"])
            if second is not None
            else f"Where is the {world.subjects[1]}?",
            str(second["subject"]) if second is not None else world.subjects[1],
            second,
        ),
    )
    queries = [
        _query_row(
            source=source,
            query_lane=lane,
            question=question,
            subject=subject,
            groups=groups,
            inherited=inherited,
        )
        for lane, (question, subject, inherited) in enumerate(query_specs)
    ]
    if queries[0]["subject"] == queries[1]["subject"]:
        raise ValueError("paired-query questions do not relocate the subject")
    label_matrix = [list(query["labels"]) for query in queries]
    flip_group_cids = [
        str(group["relation_group_cid"])
        for column, group in enumerate(groups)
        if {label_matrix[0][column], label_matrix[1][column]} == {0, 1}
    ]
    positive_cells = sum(sum(row) for row in label_matrix)
    if len(flip_group_cids) != positive_cells:
        raise ValueError("every positive paired-query cell must own one required flip")

    value = {
        "schema": PAIR_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "population": population,
        "lexical_world": world.name,
        "world_ordinal": world.ordinal,
        "world_lane": world.lane,
        "source_width": world.width,
        "pair_slot": pair_slot,
        "pair_kind": PAIR_KINDS[pair_slot],
        "source": source,
        "source_cid": source_cid,
        "sentence_spans": occurrences,
        "candidate_groups": groups,
        "queries": queries,
        "label_matrix": label_matrix,
        "flip_group_cids": flip_group_cids,
        "required_flip_count": len(flip_group_cids),
        "duplicate_pair": pair_slot == 3,
        "candidate_group_count": len(groups),
        "query_candidate_cell_count": QUERIES_PER_PAIR * len(groups),
        "positive_cell_count": positive_cells,
        "negative_cell_count": QUERIES_PER_PAIR * len(groups) - positive_cells,
    }
    return _canonical_with_cid(value, "record_cid")


def _world_partition(
    *, partition: str, worlds_per_width: int, ordinal_start: int
) -> tuple[list[LexicalWorld], list[dict[str, Any]], int]:
    worlds: list[LexicalWorld] = []
    pairs: list[dict[str, Any]] = []
    ordinal = ordinal_start
    for width in SOURCE_WIDTHS:
        for lane in range(worlds_per_width):
            world = _world(
                partition=partition, width=width, lane=lane, ordinal=ordinal
            )
            worlds.append(world)
            pairs.extend(
                _pair_from_world(world, population=partition, pair_slot=pair_slot)
                for pair_slot in range(PAIRS_PER_WORLD)
            )
            ordinal += 1
    return worlds, pairs, ordinal


def _product_partition(
    *, ordinal_start: int
) -> tuple[list[LexicalWorld], dict[str, Any], int]:
    worlds: list[LexicalWorld] = []
    pairs: list[dict[str, Any]] = []
    ordinal = ordinal_start
    for lane, pair_slot in enumerate(range(PAIRS_PER_WORLD)):
        world = _world(
            partition="c1-sb5-product",
            width=3,
            lane=lane,
            ordinal=ordinal,
        )
        worlds.append(world)
        pairs.append(
            _pair_from_world(world, population="product", pair_slot=pair_slot)
        )
        ordinal += 1
    value = {
        "schema": PRODUCT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "access_policy": (
            "commit this separate envelope before optimization; product text and "
            "token-bound lanes are denied to every training/evaluation view until "
            "all pre-product gates pass"
        ),
        "training_view_access": "DENIED",
        "records": pairs,
    }
    return worlds, _canonical_with_cid(value, "product_probes_cid"), ordinal


def _partition_counts(pairs: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    outcomes = Counter(
        str(query["target_outcome"])
        for pair in pairs
        for query in pair["queries"]
    )
    groups = sum(len(pair["candidate_groups"]) for pair in pairs)
    cells = sum(int(pair["query_candidate_cell_count"]) for pair in pairs)
    positives = sum(int(pair["positive_cell_count"]) for pair in pairs)
    flips = sum(int(pair["required_flip_count"]) for pair in pairs)
    return {
        "pairs": len(pairs),
        "query_rows": sum(len(pair["queries"]) for pair in pairs),
        "candidate_groups": groups,
        "query_candidate_cells": cells,
        "positive_cells": positives,
        "negative_cells": cells - positives,
        "required_flips": flips,
        "outcomes": {
            outcome: outcomes[outcome] for outcome in ("answer", "abstain", "conflict")
        },
        "duplicate_pairs": sum(bool(pair["duplicate_pair"]) for pair in pairs),
    }


def _sentence_inventory(pairs: Sequence[Mapping[str, Any]]) -> set[str]:
    return {
        str(span["text"]) for pair in pairs for span in pair["sentence_spans"]
    }


def _old_sentence_inventory() -> set[str]:
    """Generate a width-eight superset for all frozen SB3/SB4 ordinals 0..161."""
    sentences: set[str] = set()
    for ordinal in range(FRESH_WORLD_ORDINAL_START):
        world = _world(partition="historical", width=8, lane=0, ordinal=ordinal)
        sentences.update(
            str(span["text"])
            for record in _world_records(world, population="historical")
            for span in record["sentence_spans"]
        )
    return sentences


def _partition_contract(pairs: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    exact = True
    question_blind_exact_pairs = 0
    query_order_counter = Counter()
    slot_width = Counter()
    for pair in pairs:
        groups = list(pair["candidate_groups"])
        matrix = list(pair["label_matrix"])
        if (
            len(pair["queries"]) != QUERIES_PER_PAIR
            or len(matrix) != QUERIES_PER_PAIR
            or any(len(row) != len(groups) for row in matrix)
            or pair["candidate_group_count"] != len(groups)
        ):
            exact = False
            continue
        slot_width[(int(pair["source_width"]), int(pair["pair_slot"]))] += 1
        query_order_counter[
            tuple(str(query["target_outcome"]) for query in pair["queries"])
        ] += 1
        expected_flips = {
            str(groups[column]["relation_group_cid"])
            for column in range(len(groups))
            if {int(matrix[0][column]), int(matrix[1][column])} == {0, 1}
        }
        declared_flips = {str(value) for value in pair["flip_group_cids"]}
        expected_labels = [
            [
                int(
                    group["semantic_kind"] == "locative"
                    and group["candidate_subject"] == query["subject"]
                )
                for group in groups
            ]
            for query in pair["queries"]
        ]
        if (
            matrix != expected_labels
            or expected_flips != declared_flips
            or len(expected_flips) != sum(sum(row) for row in matrix)
        ):
            exact = False
        shortcut = [
            [int(group["semantic_kind"] == "locative") for group in groups]
            for _ in pair["queries"]
        ]
        question_blind_exact_pairs += int(shortcut == matrix)

    widths_per_slot = {
        str(width): {
            str(pair_slot): slot_width[(width, pair_slot)]
            for pair_slot in range(PAIRS_PER_WORLD)
        }
        for width in SOURCE_WIDTHS
    }
    return {
        **_partition_counts(pairs),
        "pair_oracle_and_flip_matrix_exact": exact,
        "question_blind_affirmative_locative_exact_pairs": question_blind_exact_pairs,
        "question_blind_shortcut_rejected": question_blind_exact_pairs == 0,
        "width_pair_slot_counts": widths_per_slot,
        "query_outcome_order_counts": {
            "|".join(key): value for key, value in sorted(query_order_counter.items())
        },
    }


def _population_census(
    *,
    fit_worlds: Sequence[LexicalWorld],
    fit: Sequence[Mapping[str, Any]],
    sealed_worlds: Sequence[LexicalWorld],
    sealed: Sequence[Mapping[str, Any]],
    product_worlds: Sequence[LexicalWorld],
    products: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    partitions = {"fit": fit, "sealed": sealed, "product": products}
    worlds = {
        "fit": fit_worlds,
        "sealed": sealed_worlds,
        "product": product_worlds,
    }
    sentence_sets = {
        name: _sentence_inventory(rows) for name, rows in partitions.items()
    }
    composite_sets = {
        name: _world_inventory(rows) for name, rows in worlds.items()
    }
    names = tuple(partitions)
    sentence_disjoint = all(
        sentence_sets[left].isdisjoint(sentence_sets[right])
        for left_index, left in enumerate(names)
        for right in names[left_index + 1 :]
    )
    composite_disjoint = all(
        composite_sets[left].isdisjoint(composite_sets[right])
        for left_index, left in enumerate(names)
        for right in names[left_index + 1 :]
    )
    historical_worlds = [
        _world(partition="historical", width=8, lane=0, ordinal=ordinal)
        for ordinal in range(FRESH_WORLD_ORDINAL_START)
    ]
    historical_composites = _world_inventory(historical_worlds)
    historical_sentences = _old_sentence_inventory()
    vs_historical = {
        name: {
            "subjects_locations_nonlocatives_disjoint": composite_sets[name].isdisjoint(
                historical_composites
            ),
            "exact_sentences_disjoint": sentence_sets[name].isdisjoint(
                historical_sentences
            ),
        }
        for name in names
    }
    checks = {
        name: _partition_contract(rows) for name, rows in partitions.items()
    }
    counts_exact = all(
        {key: checks[name][key] for key in EXPECTED_COUNTS[name]}
        == EXPECTED_COUNTS[name]
        for name in names
    )
    ordinal_set = {
        world.ordinal for rows in worlds.values() for world in rows
    }
    ordinal_contract = {
        "historical_sb4_max_world_ordinal": FRESH_WORLD_ORDINAL_START - 1,
        "sb5_min_world_ordinal": min(ordinal_set),
        "sb5_max_world_ordinal": max(ordinal_set),
        "starts_exactly_after_sb4": min(ordinal_set) == FRESH_WORLD_ORDINAL_START,
        "ordinals_contiguous": ordinal_set
        == set(range(FRESH_WORLD_ORDINAL_START, FRESH_WORLD_ORDINAL_START + 25)),
    }
    passed = (
        counts_exact
        and sentence_disjoint
        and composite_disjoint
        and all(
            bool(value)
            for comparison in vs_historical.values()
            for value in comparison.values()
        )
        and all(
            bool(check["pair_oracle_and_flip_matrix_exact"])
            and bool(check["question_blind_shortcut_rejected"])
            for check in checks.values()
        )
        and bool(ordinal_contract["starts_exactly_after_sb4"])
        and bool(ordinal_contract["ordinals_contiguous"])
    )
    value = {
        "schema": CENSUS_SCHEMA,
        "policy": POLICY,
        "fresh_world_ordinal_start": FRESH_WORLD_ORDINAL_START,
        "partition_checks": checks,
        "expected_counts": EXPECTED_COUNTS,
        "counts_exact": counts_exact,
        "new_sentence_partitions_pairwise_disjoint": sentence_disjoint,
        "new_composite_world_partitions_pairwise_disjoint": composite_disjoint,
        "new_vs_sb3_sb4": vs_historical,
        "composite_world_item_definition": (
            "complete generated subject phrases, complete generated location phrases, "
            "and complete generated nonlocative phrases"
        ),
        "primitive_component_vocabulary": (
            "DELIBERATELY_SHARED; only complete composite items and exact sentences "
            "are disjoint"
        ),
        "ordinal_contract": ordinal_contract,
        "passed": passed,
    }
    if not passed:
        raise RuntimeError(f"C1-SB5 population census failed: {value}")
    return _canonical_with_cid(value, "census_cid")


def build_paired_query_binding_semantic_population(
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Build fresh semantic pairs, split commitments, and opaque products."""
    ordinal = FRESH_WORLD_ORDINAL_START
    fit_worlds, fit, ordinal = _world_partition(
        partition="c1-sb5-preflight-fit",
        worlds_per_width=PREFLIGHT_FIT_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    sealed_worlds, sealed, ordinal = _world_partition(
        partition="c1-sb5-preflight-sealed",
        worlds_per_width=PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    product_worlds, products, ordinal = _product_partition(ordinal_start=ordinal)
    if ordinal != FRESH_WORLD_ORDINAL_START + 25:
        raise RuntimeError("C1-SB5 lexical-world ordinal allocation drifted")

    census = _population_census(
        fit_worlds=fit_worlds,
        fit=fit,
        sealed_worlds=sealed_worlds,
        sealed=sealed,
        product_worlds=product_worlds,
        products=products["records"],
    )
    preflight_value = {
        "schema": PREFLIGHT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "binding": "SEMANTIC_UNTOKENIZED",
        "selection": (
            "fresh ordinals 162..182; two fit and one sealed world per width "
            "2..8; four exact same-source two-query pairs per world"
        ),
        "counts": {
            "fit": EXPECTED_COUNTS["fit"],
            "sealed": EXPECTED_COUNTS["sealed"],
        },
        "fit_world_names": [world.name for world in fit_worlds],
        "sealed_world_names": [world.name for world in sealed_worlds],
        "fit": fit,
        "sealed": sealed,
        "census_cid": census["census_cid"],
    }
    preflight = _canonical_with_cid(preflight_value, "preflight_cid")
    split_policy = _canonical_with_cid(
        {
            "schema": SPLIT_SCHEMA,
            "policy": POLICY,
            "source_widths": list(SOURCE_WIDTHS),
            "pair_kinds": list(PAIR_KINDS),
            "worlds_per_width": {
                "fit": PREFLIGHT_FIT_WORLDS_PER_WIDTH,
                "sealed": PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
            },
            "fresh_world_ordinal_start": FRESH_WORLD_ORDINAL_START,
            "product_world_ordinals": [world.ordinal for world in product_worlds],
            "product_policy": "four separately committed opaque two-query pairs",
        },
        "split_policy_cid",
    )
    dataset_value = {
        "schema": DATASET_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "input_policy": INPUT_POLICY,
        "counts": EXPECTED_COUNTS,
        "split_policy": split_policy,
        "split_policy_cid": split_policy["split_policy_cid"],
        "census": census,
        "census_cid": census["census_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": [
            record["record_cid"] for record in products["records"]
        ],
        "training_view": {
            "allowed_filenames": list(TRAINING_VIEW_FILENAMES),
            "denied_filenames": list(PRODUCT_DENIED_FILENAMES),
            "product_text_access": "DENIED",
        },
    }
    dataset = _canonical_with_cid(dataset_value, "dataset_cid")
    return dataset, preflight, products


def _token_ids_cid(token_ids: Sequence[int]) -> str:
    material = bytearray(b"uor-r4-token-ids-u32be/1\x00")
    for token_id in token_ids:
        if not isinstance(token_id, int) or not 0 <= token_id <= 0xFFFF_FFFF:
            raise ValueError("token ID is outside canonical u32")
        material.extend(struct.pack(">I", token_id))
    return cid_bytes(bytes(material))


def _tokenizer_identity(tokenizer: Tokenizer, tokenizer_cid: str | None) -> str:
    if tokenizer_cid is not None:
        if not tokenizer_cid.startswith("blake3:"):
            raise ValueError("tokenizer CID is not a BLAKE3 identifier")
        return tokenizer_cid
    return cid_bytes(tokenizer.to_str().encode("utf-8"))


def _terminal_content_index(
    *, text: str, offsets: Sequence[tuple[int, int]], byte_end: int, marker: str
) -> int:
    if not text.isascii():
        raise ValueError("bounded paired-query token anchors require ASCII generated text")
    matches = [
        index
        for index, (start, end) in enumerate(offsets)
        if start < end and end == byte_end and text[start:end] == marker
    ]
    if len(matches) != 1:
        raise ValueError(f"paired-query terminal marker {marker!r} is not one exact token")
    return matches[0]


def encode_paired_query_pair(
    pair: Mapping[str, Any],
    tokenizer: Tokenizer,
    *,
    tokenizer_cid: str | None = None,
) -> dict[str, Any]:
    """Bind one semantic pair to exact token IDs and source/query state anchors."""
    if pair.get("schema") != PAIR_SCHEMA or pair.get("policy") != POLICY:
        raise ValueError("paired-query pair schema/policy differs")
    verify_artifact_cid(pair, "record_cid")
    tokenizer_identity = _tokenizer_identity(tokenizer, tokenizer_cid)
    source = str(pair["source"])
    if not source.isascii():
        raise ValueError("bounded paired-query source must be ASCII for byte anchors")
    source_prefix_end = 2 + len(source)
    encoded_queries: list[dict[str, Any]] = []
    shared_prefix_ids: list[int] | None = None
    shared_candidate_indices: list[int] | None = None

    spans_by_index = {
        int(span["candidate_index"]): span for span in pair["sentence_spans"]
    }
    for semantic_query in pair["queries"]:
        relation_input = render_paired_query_input(
            source, str(semantic_query["question"])
        )
        if relation_input != semantic_query["relation_input"]:
            raise ValueError("paired-query semantic lane renderer differs")
        encoding = tokenizer.encode(relation_input, add_special_tokens=False)
        token_ids = [int(token_id) for token_id in encoding.ids]
        offsets = [(int(start), int(end)) for start, end in encoding.offsets]
        if not token_ids or len(token_ids) != len(offsets):
            raise ValueError("paired-query tokenizer returned an empty/malformed lane")
        positions = len(token_ids) + 1
        if positions > MAX_POSITIONS_INCLUDING_BOS:
            raise ValueError(
                "paired-query lane exceeds 256 positions including BOS; truncation forbidden"
            )
        terminal_content_index = _terminal_content_index(
            text=relation_input,
            offsets=offsets,
            byte_end=len(relation_input),
            marker=":",
        )
        if terminal_content_index != len(token_ids) - 1:
            raise ValueError("paired-query Bind colon is not the final content token")

        prefix_crossings = [
            (start, end)
            for start, end in offsets
            if start < source_prefix_end < end
        ]
        if prefix_crossings:
            raise ValueError("paired-query tokenizer merged across the source/Q boundary")
        prefix_content_count = sum(
            start < end and end <= source_prefix_end for start, end in offsets
        )
        prefix_ids = [BOS_TOKEN_ID, *token_ids[:prefix_content_count]]
        source_prefix_token_count = len(prefix_ids)

        candidate_terminal_indices: list[int] = []
        candidate_terminal_anchors: list[dict[str, Any]] = []
        for group in pair["candidate_groups"]:
            occurrence_index = int(group["earliest_occurrence_index"])
            occurrence = spans_by_index[occurrence_index]
            terminal_byte_end = 2 + int(occurrence["terminal_byte_end"])
            content_index = _terminal_content_index(
                text=relation_input,
                offsets=offsets,
                byte_end=terminal_byte_end,
                marker=str(occurrence["text"])[-1],
            )
            model_index = content_index + 1
            if model_index >= source_prefix_token_count:
                raise ValueError("candidate state is not strictly before Q")
            candidate_terminal_indices.append(model_index)
            candidate_terminal_anchors.append(
                {
                    "relation_group_cid": group["relation_group_cid"],
                    "occurrence_index": occurrence_index,
                    "content_token_index": content_index,
                    "model_token_index_including_bos": model_index,
                    "token_id": token_ids[content_index],
                    "token_bit_offset_u32": model_index * 32,
                    "terminal_byte_end_in_lane": terminal_byte_end,
                }
            )
        query_terminal_index = terminal_content_index + 1
        if query_terminal_index < source_prefix_token_count:
            raise ValueError("query terminal state falls inside the source prefix")

        if shared_prefix_ids is None:
            shared_prefix_ids = prefix_ids
            shared_candidate_indices = candidate_terminal_indices
        elif (
            shared_prefix_ids != prefix_ids
            or shared_candidate_indices != candidate_terminal_indices
        ):
            raise ValueError("same-source paired lanes do not have identical source anchors")

        unsigned_query = dict(semantic_query)
        semantic_query_cid = unsigned_query.pop("query_row_cid")
        unsigned_query.update(
            {
                "semantic_query_row_cid": semantic_query_cid,
                "tokenizer_cid": tokenizer_identity,
                "token_ids": token_ids,
                "token_ids_cid": _token_ids_cid(token_ids),
                "input_ids_including_bos_cid": _token_ids_cid(
                    [BOS_TOKEN_ID, *token_ids]
                ),
                "positions_including_bos": positions,
                "source_prefix_token_count": source_prefix_token_count,
                "source_prefix_token_ids_cid": _token_ids_cid(prefix_ids),
                "source_prefix_bit_range_u32": [0, source_prefix_token_count * 32],
                "candidate_terminal_indices": candidate_terminal_indices,
                "candidate_terminal_bit_offsets_u32": [
                    index * 32 for index in candidate_terminal_indices
                ],
                "candidate_terminal_anchors": candidate_terminal_anchors,
                "query_terminal_index": query_terminal_index,
                "query_terminal_bit_offset_u32": query_terminal_index * 32,
                "all_candidate_states_before_query": all(
                    index < source_prefix_token_count
                    for index in candidate_terminal_indices
                ),
                "truncation": "FORBIDDEN_NOT_USED",
            }
        )
        encoded_queries.append(_canonical_with_cid(unsigned_query, "query_row_cid"))

    if shared_prefix_ids is None or shared_candidate_indices is None:
        raise RuntimeError("paired-query pair encoded no lanes")
    unsigned_pair = deepcopy(dict(pair))
    semantic_record_cid = unsigned_pair.pop("record_cid")
    unsigned_pair["semantic_record_cid"] = semantic_record_cid
    unsigned_pair["binding"] = "TOKENIZER_BOUND"
    unsigned_pair["tokenizer_cid"] = tokenizer_identity
    unsigned_pair["queries"] = encoded_queries
    unsigned_pair["source_prefix_token_count"] = len(shared_prefix_ids)
    unsigned_pair["source_prefix_token_ids_cid"] = _token_ids_cid(shared_prefix_ids)
    unsigned_pair["source_prefix_identity_exact"] = True
    unsigned_pair["candidate_terminal_indices"] = shared_candidate_indices
    unsigned_pair["candidate_anchor_identity_exact"] = True
    unsigned_pair["all_candidate_states_before_query"] = all(
        query["all_candidate_states_before_query"] for query in encoded_queries
    )
    return _canonical_with_cid(unsigned_pair, "record_cid")


def _tokenizer_partition(
    pairs: Sequence[Mapping[str, Any]],
    tokenizer: Tokenizer,
    *,
    tokenizer_cid: str,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    encoded = [
        encode_paired_query_pair(pair, tokenizer, tokenizer_cid=tokenizer_cid)
        for pair in pairs
    ]
    positions = [
        int(query["positions_including_bos"])
        for pair in encoded
        for query in pair["queries"]
    ]
    value = {
        "pairs": len(encoded),
        "query_rows": len(positions),
        "minimum_positions_including_bos": min(positions),
        "maximum_positions_including_bos": max(positions),
        "context_ceiling_including_bos": MAX_POSITIONS_INCLUDING_BOS,
        "no_prompt_truncated": all(
            query["truncation"] == "FORBIDDEN_NOT_USED"
            for pair in encoded
            for query in pair["queries"]
        ),
        "source_prefix_identity_exact": all(
            pair["source_prefix_identity_exact"] for pair in encoded
        ),
        "candidate_anchor_identity_exact": all(
            pair["candidate_anchor_identity_exact"] for pair in encoded
        ),
        "all_candidate_states_before_query": all(
            pair["all_candidate_states_before_query"] for pair in encoded
        ),
        "passed": all(position <= MAX_POSITIONS_INCLUDING_BOS for position in positions),
    }
    return encoded, value


def build_paired_query_tokenizer_census(
    preflight: Mapping[str, Any],
    products: Mapping[str, Any],
    tokenizer: Tokenizer,
    *,
    tokenizer_cid: str | None = None,
) -> dict[str, Any]:
    """Bind every fit/sealed/product lane before optimization; never truncate."""
    if preflight.get("schema") != PREFLIGHT_SCHEMA:
        raise ValueError("paired-query preflight schema differs")
    if products.get("schema") != PRODUCT_SCHEMA:
        raise ValueError("paired-query product schema differs")
    identity = _tokenizer_identity(tokenizer, tokenizer_cid)
    partitions: dict[str, Any] = {}
    for name, pairs in (
        ("fit", preflight["fit"]),
        ("sealed", preflight["sealed"]),
        ("product", products["records"]),
    ):
        _, partitions[name] = _tokenizer_partition(
            pairs, tokenizer, tokenizer_cid=identity
        )
    passed = all(
        bool(partition["passed"])
        and bool(partition["no_prompt_truncated"])
        and bool(partition["source_prefix_identity_exact"])
        and bool(partition["candidate_anchor_identity_exact"])
        and bool(partition["all_candidate_states_before_query"])
        for partition in partitions.values()
    )
    value = {
        "schema": TOKENIZER_CENSUS_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "tokenizer_cid": identity,
        "input_policy": INPUT_POLICY,
        "partitions": partitions,
        "product_text_access": "PREPARATION_ONLY_DENIED_TO_TRAINING_VIEW",
        "passed": passed,
    }
    if not passed:
        raise RuntimeError(f"C1-SB5 tokenizer census failed: {value}")
    return _canonical_with_cid(value, "tokenizer_census_cid")


def build_paired_query_binding_population(
    tokenizer: Tokenizer | None = None,
    *,
    tokenizer_cid: str | None = None,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Build semantic data, optionally returning a tokenizer-bound preflight."""
    dataset, semantic_preflight, products = (
        build_paired_query_binding_semantic_population()
    )
    if tokenizer is None:
        if tokenizer_cid is not None:
            raise ValueError("tokenizer CID was provided without a tokenizer")
        return dataset, semantic_preflight, products

    identity = _tokenizer_identity(tokenizer, tokenizer_cid)
    encoded_fit, fit_census = _tokenizer_partition(
        semantic_preflight["fit"], tokenizer, tokenizer_cid=identity
    )
    encoded_sealed, sealed_census = _tokenizer_partition(
        semantic_preflight["sealed"], tokenizer, tokenizer_cid=identity
    )
    tokenizer_census = build_paired_query_tokenizer_census(
        semantic_preflight,
        products,
        tokenizer,
        tokenizer_cid=identity,
    )
    unsigned_preflight = {
        key: deepcopy(value)
        for key, value in semantic_preflight.items()
        if key not in {"preflight_cid", "fit", "sealed", "binding"}
    }
    unsigned_preflight.update(
        {
            "binding": "TOKENIZER_BOUND",
            "semantic_preflight_cid": semantic_preflight["preflight_cid"],
            "tokenizer_cid": identity,
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "fit_tokenizer_census": fit_census,
            "sealed_tokenizer_census": sealed_census,
            "fit": encoded_fit,
            "sealed": encoded_sealed,
        }
    )
    preflight = _canonical_with_cid(unsigned_preflight, "preflight_cid")

    unsigned_dataset = {
        key: deepcopy(value)
        for key, value in dataset.items()
        if key not in {"dataset_cid", "preflight_cid"}
    }
    unsigned_dataset.update(
        {
            "binding": "TOKENIZER_BOUND",
            "semantic_dataset_cid": dataset["dataset_cid"],
            "semantic_preflight_cid": semantic_preflight["preflight_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "tokenizer_census": tokenizer_census,
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "tokenizer_cid": identity,
        }
    )
    bound_dataset = _canonical_with_cid(unsigned_dataset, "dataset_cid")
    return bound_dataset, preflight, products


__all__ = [
    "CENSUS_FILENAME",
    "CENSUS_SCHEMA",
    "DATASET_FILENAME",
    "DATASET_SCHEMA",
    "EXPECTED_COUNTS",
    "FRESH_WORLD_ORDINAL_START",
    "INPUT_POLICY",
    "ISSUE",
    "MAX_POSITIONS_INCLUDING_BOS",
    "PAIR_KINDS",
    "PAIR_SCHEMA",
    "POLICY",
    "PREFLIGHT_FILENAME",
    "PREFLIGHT_SCHEMA",
    "PRODUCT_DENIED_FILENAMES",
    "PRODUCT_FILENAME",
    "PRODUCT_MANIFEST_FILENAME",
    "PRODUCT_SCHEMA",
    "QUERY_SCHEMA",
    "SENTENCE_POLICY",
    "SPLIT_SCHEMA",
    "TOKENIZER_CENSUS_FILENAME",
    "TOKENIZER_CENSUS_SCHEMA",
    "TRAINING_VIEW_FILENAMES",
    "TRAINING_VIEW_MANIFEST_FILENAME",
    "artifact_bytes",
    "build_paired_query_binding_population",
    "build_paired_query_binding_semantic_population",
    "build_paired_query_tokenizer_census",
    "encode_paired_query_pair",
    "load_artifact",
    "render_paired_query_input",
    "verify_artifact_cid",
]
