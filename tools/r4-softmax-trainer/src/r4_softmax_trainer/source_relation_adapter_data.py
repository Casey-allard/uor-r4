"""Frozen C1-SB3 counterfactual relation population and sealed products.

This module contains data and labels only.  The training process may build the
construction, development, and matched-transfer populations, but it must treat
the separately committed product envelope as unopened until every pre-product
gate has passed.
"""

from __future__ import annotations

from collections import Counter, defaultdict
from dataclasses import dataclass
from typing import Any, Iterable, Sequence

from .provenance import canonical_json_bytes, cid_bytes
from .source_relation_data import parse_subject, split_sentence_spans


ISSUE = 954
POLICY = "R4AttendedRelationAdapterV1"
RELATION_RECORD_SCHEMA = "uor-r4.attended-relation-record/1"
RELATION_DATASET_SCHEMA = "uor-r4.attended-relation-dataset/1"
RELATION_PREFLIGHT_SCHEMA = "uor-r4.attended-relation-preflight/1"
RELATION_PRODUCT_SCHEMA = "uor-r4.attended-relation-products/1"
RELATION_CENSUS_SCHEMA = "uor-r4.attended-relation-census/1"
RELATION_SPLIT_SCHEMA = "uor-r4.attended-relation-split/1"
RELATION_INPUT_POLICY = (
    "exact UTF-8 `Evidence:\\n<span>\\nQuestion:\\n<question>\\nSupported:` "
    "with no terminal newline; score the fixed next-token yes/no verbalizer at "
    "the final colon"
)
QUESTION_POLICY = "Where is the <subject>?"
SENTENCE_POLICY = "exact .!? terminated UTF-8 byte spans"
SOURCE_WIDTHS = tuple(range(2, 9))
OUTCOMES = ("answer", "abstain", "conflict")
MOTIFS_PER_OUTCOME = 3
PREFLIGHT_FIT_WORLDS_PER_WIDTH = 2
PREFLIGHT_SEALED_WORLDS_PER_WIDTH = 1
CONSTRUCTION_WORLDS_PER_WIDTH = 12
DEVELOPMENT_WORLDS_PER_WIDTH = 4
YES_TOKEN_ID = 1_771
NO_TOKEN_ID = 542
YES_TOKEN_TEXT = " yes"
NO_TOKEN_TEXT = " no"
ABSTAIN = "ABSTAIN"
CONTRADICTION = "CONTRADICTION"


SUBJECT_FIRST = (
    "amber",
    "azure",
    "bronze",
    "cerulean",
    "citrine",
    "cobalt",
    "coral",
    "crimson",
    "emerald",
    "garnet",
    "indigo",
    "ivory",
    "jade",
    "magenta",
    "ochre",
    "onyx",
)
SUBJECT_SECOND = (
    "alder",
    "aspen",
    "beech",
    "birch",
    "cedar",
    "cypress",
    "elm",
    "fir",
    "hazel",
    "juniper",
    "larch",
    "maple",
    "oak",
    "pine",
    "rowan",
    "willow",
)
SUBJECT_NOUNS = (
    "abacus",
    "astrolabe",
    "banner",
    "bell",
    "compass",
    "falcon",
    "flute",
    "goblet",
    "kettle",
    "lantern",
    "ledger",
    "orrery",
    "pendulum",
    "prism",
    "sextant",
    "spindle",
)
LOCATION_PREPOSITIONS = (
    "above",
    "behind",
    "beneath",
    "beside",
    "inside",
    "near",
    "under",
    "within",
)
LOCATION_ADJECTIVES = (
    "arched",
    "brass",
    "cedar",
    "cinder",
    "clay",
    "fern",
    "glass",
    "granite",
    "linen",
    "marble",
    "mossy",
    "narrow",
    "painted",
    "paper",
    "reed",
    "slate",
)
LOCATION_NOUNS = (
    "alcove",
    "awning",
    "balcony",
    "cabinet",
    "canopy",
    "chest",
    "closet",
    "colonnade",
    "cupboard",
    "gallery",
    "hamper",
    "kiosk",
    "locker",
    "portico",
    "screen",
    "shelf",
)
NONLOCATIVE_VERBS = (
    "audited",
    "catalogued",
    "cleaned",
    "documented",
    "engraved",
    "examined",
    "indexed",
    "inspected",
    "inventoried",
    "labeled",
    "measured",
    "photographed",
    "repaired",
    "sketched",
    "weighed",
    "wrapped",
)
NONLOCATIVE_TIMES = (
    "after breakfast",
    "after rehearsal",
    "at dawn",
    "at dusk",
    "before lunch",
    "before sunrise",
    "during intermission",
    "during the concert",
    "last autumn",
    "last spring",
    "on Thursday",
    "on Tuesday",
    "this morning",
    "this winter",
    "yesterday afternoon",
    "yesterday evening",
)


@dataclass(frozen=True, slots=True)
class LexicalWorld:
    name: str
    partition: str
    width: int
    lane: int
    ordinal: int
    subjects: tuple[str, ...]
    locations: tuple[str, ...]
    nonlocative: str


def render_adapter_relation_input(span: str, question: str) -> str:
    """Render the exact evidence/question/verbalizer prefix without a newline."""
    if not span or span != span.strip() or span[-1] not in ".!?":
        raise ValueError("relation evidence must be one trimmed terminated span")
    parse_subject(question)
    return f"Evidence:\n{span}\nQuestion:\n{question}\nSupported:"


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _catalog_item(parts: Sequence[Sequence[str]], index: int) -> str:
    capacity = 1
    for part in parts:
        capacity *= len(part)
    if not 0 <= index < capacity:
        raise ValueError(f"lexical catalog index {index} exceeds {capacity}")
    chosen: list[str] = []
    cursor = index
    for part in reversed(parts):
        chosen.append(part[cursor % len(part)])
        cursor //= len(part)
    return " ".join(reversed(chosen))


def _subject(index: int) -> str:
    return _catalog_item((SUBJECT_FIRST, SUBJECT_SECOND, SUBJECT_NOUNS), index)


def _location(index: int) -> str:
    return _catalog_item(
        (LOCATION_PREPOSITIONS, LOCATION_ADJECTIVES, LOCATION_NOUNS), index
    ).replace(" ", " the ", 1)


def _nonlocative(index: int) -> str:
    phrase = _catalog_item((NONLOCATIVE_VERBS, NONLOCATIVE_TIMES), index)
    verb, remainder = phrase.split(" ", 1)
    return f"was {verb} {remainder}"


def _world(
    *, partition: str, width: int, lane: int, ordinal: int
) -> LexicalWorld:
    if width not in SOURCE_WIDTHS:
        raise ValueError("world width is outside 2..=8")
    subject_start = ordinal * 9
    location_start = ordinal * 8
    return LexicalWorld(
        name=f"{partition}-w{width}-lane{lane}",
        partition=partition,
        width=width,
        lane=lane,
        ordinal=ordinal,
        subjects=tuple(_subject(subject_start + index) for index in range(9)),
        locations=tuple(_location(location_start + index) for index in range(8)),
        nonlocative=_nonlocative(ordinal),
    )


def _question(subject: str) -> str:
    return f"Where is the {subject}?"


def _locative(subject: str, location: str) -> str:
    return f"The {subject} is {location}."


def _nonlocative_sentence(subject: str, phrase: str) -> str:
    return f"The {subject} {phrase}."


def _negated(subject: str, location: str) -> str:
    return f"The {subject} is not {location}."


def _entry(text: str, role: str, semantic_kind: str) -> dict[str, str]:
    if semantic_kind not in {"locative", "nonlocative", "negated-locative"}:
        raise ValueError("unknown relation semantic kind")
    return {"text": text, "role": role, "semantic_kind": semantic_kind}


def _fill_entries(
    world: LexicalWorld,
    entries: Sequence[dict[str, str]],
) -> list[dict[str, str]]:
    if len(entries) > world.width:
        raise ValueError("motif base exceeds its source width")
    result = [dict(entry) for entry in entries]
    for subject in world.subjects[3 : 3 + world.width - len(result)]:
        result.append(
            _entry(
                _nonlocative_sentence(subject, world.nonlocative),
                "nonlocative-filler",
                "nonlocative",
            )
        )
    if len(result) != world.width:
        raise RuntimeError("world did not supply enough nonlocative fillers")
    return result


def _arrange(
    entries: Sequence[dict[str, str]], *, offset: int
) -> list[dict[str, str]]:
    values = [dict(entry) for entry in entries]
    if not values:
        raise ValueError("cannot arrange an empty source")
    shift = offset % len(values)
    return values[shift:] + values[:shift]


def _positive_pair_offset(world: LexicalWorld, lane: int) -> int:
    """Place successive positive pairs two lanes apart, then rotate by world."""
    desired_start = (world.ordinal + 2 * lane) % world.width
    return (world.width - desired_start) % world.width


def _make_record(
    *,
    population: str,
    motif: str,
    outcome: str,
    world: LexicalWorld,
    subject: str,
    entries: Sequence[dict[str, str]],
    positive_texts: Iterable[str],
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    if outcome not in OUTCOMES or len(entries) != world.width:
        raise ValueError("relation record outcome or source width is invalid")
    question = _question(subject)
    if parse_subject(question) != subject:
        raise RuntimeError("relation question did not reproduce its subject")
    source = " ".join(str(entry["text"]) for entry in entries)
    parsed = split_sentence_spans(source)
    if len(parsed) != len(entries):
        raise RuntimeError("relation source did not reproduce its candidate count")
    positive_text_set = set(positive_texts)

    sentence_spans: list[dict[str, Any]] = []
    for index, (span, entry) in enumerate(zip(parsed, entries)):
        text = str(entry["text"])
        if span["text"] != text:
            raise RuntimeError("relation source span changed during parsing")
        text_cid = cid_bytes(text.encode("utf-8"))
        relation_input = render_adapter_relation_input(text, question)
        sentence_spans.append(
            {
                "candidate_index": index,
                "byte_start": int(span["byte_start"]),
                "byte_end": int(span["byte_end"]),
                "text": text,
                "text_cid": text_cid,
                "role": str(entry["role"]),
                "semantic_kind": str(entry["semantic_kind"]),
                "relation_label": int(text in positive_text_set),
                "relation_group_cid": text_cid,
                "relation_input": relation_input,
                "relation_input_cid": cid_bytes(relation_input.encode("utf-8")),
            }
        )

    positive_indices = [
        int(span["candidate_index"])
        for span in sentence_spans
        if int(span["relation_label"]) == 1
    ]
    positive_groups = sorted(
        {str(sentence_spans[index]["relation_group_cid"]) for index in positive_indices}
    )
    derived_outcome = (
        "abstain"
        if not positive_groups
        else "answer"
        if len(positive_groups) == 1
        else "conflict"
    )
    if derived_outcome != outcome:
        raise ValueError(
            f"relation labels derive {derived_outcome}, not declared {outcome}"
        )
    duplicate_agreement = outcome == "answer" and len(positive_indices) > 1
    if duplicate_agreement and len(
        {sentence_spans[index]["text"] for index in positive_indices}
    ) != 1:
        raise ValueError("answer agreement must contain one exact text group")
    if outcome == "conflict" and len(positive_groups) != 2:
        raise ValueError("conflict must contain exactly two distinct relation groups")
    target_span_index = min(positive_indices) if outcome == "answer" else None
    answer = (
        str(sentence_spans[target_span_index]["text"])
        if target_span_index is not None
        else ABSTAIN
        if outcome == "abstain"
        else CONTRADICTION
    )
    value: dict[str, Any] = {
        "schema": RELATION_RECORD_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "population": population,
        "lexical_world": world.name,
        "motif": motif,
        "target_outcome": outcome,
        "source_width": world.width,
        "source": source,
        "source_cid": cid_bytes(source.encode("utf-8")),
        "question": question,
        "question_cid": cid_bytes(question.encode("utf-8")),
        "subject": subject,
        "sentence_spans": sentence_spans,
        "positive_span_indices": positive_indices,
        "positive_relation_group_cids": positive_groups,
        "raw_subject_occurrence_count": sum(
            str(span["text"]).count(subject) for span in sentence_spans
        ),
        "target_span_index": target_span_index,
        "answer": answer,
        "duplicate_agreement": duplicate_agreement,
    }
    if extra:
        overlap = set(value).intersection(extra)
        if overlap:
            raise ValueError(f"extra relation fields collide: {sorted(overlap)}")
        value.update(extra)
    return _canonical_with_cid(value, "record_cid")


def _world_records(world: LexicalWorld, *, population: str) -> list[dict[str, Any]]:
    subject_a, subject_b = world.subjects[:2]
    location_a, location_b = world.locations[:2]
    a_location_a = _locative(subject_a, location_a)
    a_location_b = _locative(subject_a, location_b)
    b_location_a = _locative(subject_b, location_a)
    b_location_b = _locative(subject_b, location_b)
    a_nonlocative = _nonlocative_sentence(subject_a, world.nonlocative)
    b_nonlocative = _nonlocative_sentence(subject_b, world.nonlocative)

    matched = _arrange(
        _fill_entries(
            world,
            [
                _entry(a_location_a, "primary-location", "locative"),
                _entry(b_location_b, "secondary-location", "locative"),
                *(
                    [_entry(a_nonlocative, "primary-nonlocative", "nonlocative")]
                    if world.width >= 3
                    else []
                ),
            ],
        ),
        offset=_positive_pair_offset(world, 0),
    )
    duplicate = _arrange(
        _fill_entries(
            world,
            [
                _entry(a_location_a, "agreement", "locative"),
                _entry(a_location_a, "agreement", "locative"),
            ],
        ),
        offset=_positive_pair_offset(world, 1),
    )
    negated = _arrange(
        _fill_entries(
            world,
            [
                _entry(
                    _negated(subject_a, location_a),
                    "primary-negated-location",
                    "negated-locative",
                ),
                _entry(a_nonlocative, "primary-nonlocative", "nonlocative"),
            ],
        ),
        offset=world.ordinal + 1,
    )
    primary_conflict = _arrange(
        _fill_entries(
            world,
            [
                _entry(a_location_a, "primary-location-a", "locative"),
                _entry(a_location_b, "primary-location-b", "locative"),
                *(
                    [_entry(b_nonlocative, "secondary-nonlocative", "nonlocative")]
                    if world.width >= 3
                    else []
                ),
            ],
        ),
        offset=_positive_pair_offset(world, 2),
    )
    secondary_conflict = _arrange(
        _fill_entries(
            world,
            [
                _entry(b_location_a, "secondary-location-a", "locative"),
                _entry(b_location_b, "secondary-location-b", "locative"),
                *(
                    [_entry(a_nonlocative, "primary-nonlocative", "nonlocative")]
                    if world.width >= 3
                    else []
                ),
            ],
        ),
        offset=_positive_pair_offset(world, 3),
    )
    duplicate_conflict_base = (
        [
            _entry(a_location_a, "conflict-agreement", "locative"),
            _entry(a_location_a, "conflict-agreement", "locative"),
            _entry(a_location_b, "primary-location-b", "locative"),
        ]
        if world.width >= 3
        else [
            _entry(a_location_b, "primary-location-b", "locative"),
            _entry(a_location_a, "primary-location-a", "locative"),
        ]
    )
    duplicate_conflict = _arrange(
        _fill_entries(world, duplicate_conflict_base),
        offset=_positive_pair_offset(world, 4),
    )

    common = {"world_ordinal": world.ordinal, "world_lane": world.lane}
    return [
        _make_record(
            population=population,
            motif="matched-primary-answer",
            outcome="answer",
            world=world,
            subject=subject_a,
            entries=matched,
            positive_texts=(a_location_a,),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="matched-secondary-answer",
            outcome="answer",
            world=world,
            subject=subject_b,
            entries=matched,
            positive_texts=(b_location_b,),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="exact-duplicate-agreement",
            outcome="answer",
            world=world,
            subject=subject_a,
            entries=duplicate,
            positive_texts=(a_location_a,),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="negated-nonlocative-abstain",
            outcome="abstain",
            world=world,
            subject=subject_a,
            entries=negated,
            positive_texts=(),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="primary-source-secondary-abstain",
            outcome="abstain",
            world=world,
            subject=subject_b,
            entries=primary_conflict,
            positive_texts=(),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="secondary-source-primary-abstain",
            outcome="abstain",
            world=world,
            subject=subject_a,
            entries=secondary_conflict,
            positive_texts=(),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="primary-distinct-location-conflict",
            outcome="conflict",
            world=world,
            subject=subject_a,
            entries=primary_conflict,
            positive_texts=(a_location_a, a_location_b),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="secondary-distinct-location-conflict",
            outcome="conflict",
            world=world,
            subject=subject_b,
            entries=secondary_conflict,
            positive_texts=(b_location_a, b_location_b),
            extra=common,
        ),
        _make_record(
            population=population,
            motif="duplicate-distinct-location-conflict",
            outcome="conflict",
            world=world,
            subject=subject_a,
            entries=duplicate_conflict,
            positive_texts=(a_location_a, a_location_b),
            extra=common,
        ),
    ]


def _world_partition(
    *,
    partition: str,
    worlds_per_width: int,
    ordinal_start: int,
) -> tuple[list[LexicalWorld], list[dict[str, Any]], int]:
    worlds: list[LexicalWorld] = []
    records: list[dict[str, Any]] = []
    ordinal = ordinal_start
    for width in SOURCE_WIDTHS:
        for lane in range(worlds_per_width):
            world = _world(
                partition=partition,
                width=width,
                lane=lane,
                ordinal=ordinal,
            )
            worlds.append(world)
            records.extend(_world_records(world, population=partition))
            ordinal += 1
    return worlds, records, ordinal


def _record_from_template(
    template: dict[str, Any],
    *,
    world: LexicalWorld,
    population: str,
    motif: str,
    extra: dict[str, Any],
) -> dict[str, Any]:
    entries = [
        _entry(str(span["text"]), str(span["role"]), str(span["semantic_kind"]))
        for span in template["sentence_spans"]
    ]
    positive_texts = {
        str(span["text"])
        for span in template["sentence_spans"]
        if int(span["relation_label"]) == 1
    }
    return _make_record(
        population=population,
        motif=motif,
        outcome=str(template["target_outcome"]),
        world=world,
        subject=str(template["subject"]),
        entries=entries,
        positive_texts=positive_texts,
        extra=extra,
    )


def _development_controls(
    worlds: Sequence[LexicalWorld], records: Sequence[dict[str, Any]]
) -> dict[str, list[dict[str, Any]]]:
    by_world: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        by_world[str(record["lexical_world"])].append(record)
    world_by_name = {world.name: world for world in worlds}
    reversals: list[dict[str, Any]] = []
    swaps: list[dict[str, Any]] = []
    for world_name in sorted(by_world):
        world = world_by_name[world_name]
        world_records = by_world[world_name]
        if len(world_records) != 9:
            raise RuntimeError("development world does not contain nine motifs")
        for base in world_records:
            reversed_entries = [
                _entry(
                    str(span["text"]),
                    str(span["role"]),
                    str(span["semantic_kind"]),
                )
                for span in reversed(base["sentence_spans"])
            ]
            positive_texts = {
                str(span["text"])
                for span in base["sentence_spans"]
                if int(span["relation_label"]) == 1
            }
            reversals.append(
                _make_record(
                    population="development-control",
                    motif="reverse-candidate-order",
                    outcome=str(base["target_outcome"]),
                    world=world,
                    subject=str(base["subject"]),
                    entries=reversed_entries,
                    positive_texts=positive_texts,
                    extra={
                        "control_kind": "order-reversal",
                        "base_record_cid": str(base["record_cid"]),
                        "candidate_original_indices": list(
                            reversed(range(int(base["source_width"])))
                        ),
                    },
                )
            )

        for base_index, control_index in ((0, 1), (1, 0), (4, 6), (6, 4), (5, 7), (7, 5)):
            base = world_records[base_index]
            template = world_records[control_index]
            if base["source_cid"] != template["source_cid"]:
                raise RuntimeError("query-swap pair does not share an exact source")
            swaps.append(
                _record_from_template(
                    template,
                    world=world,
                    population="development-control",
                    motif="same-source-query-swap",
                    extra={
                        "control_kind": "query-swap",
                        "base_record_cid": str(base["record_cid"]),
                        "paired_record_cid": str(template["record_cid"]),
                        "base_subject": str(base["subject"]),
                        "base_outcome": str(base["target_outcome"]),
                        "expected_original_span_index": template["target_span_index"],
                    },
                )
            )
    return {"reversal": reversals, "query_swap": swaps}


def _sentence_inventory(records: Iterable[dict[str, Any]]) -> set[str]:
    return {
        str(span["text"])
        for record in records
        for span in record["sentence_spans"]
    }


def _world_inventory(worlds: Sequence[LexicalWorld]) -> set[str]:
    return {
        value
        for world in worlds
        for value in (*world.subjects, *world.locations, world.nonlocative)
    }


def _polarity_by_locative(records: Sequence[dict[str, Any]]) -> dict[str, set[int]]:
    values: dict[str, set[int]] = defaultdict(set)
    for record in records:
        for span in record["sentence_spans"]:
            if span["semantic_kind"] == "locative":
                values[str(span["text"])].add(int(span["relation_label"]))
    return values


def _query_outcomes(records: Sequence[dict[str, Any]]) -> dict[str, set[str]]:
    values: dict[str, set[str]] = defaultdict(set)
    for record in records:
        values[str(record["subject"])].add(
            "answer" if record["target_outcome"] == "answer" else "nonanswer"
        )
    return values


def _position_labels(records: Sequence[dict[str, Any]]) -> dict[tuple[int, int], set[int]]:
    values: dict[tuple[int, int], set[int]] = defaultdict(set)
    for record in records:
        width = int(record["source_width"])
        for span in record["sentence_spans"]:
            values[(width, int(span["candidate_index"]))].add(
                int(span["relation_label"])
            )
    return values


def _population_census(
    *,
    partitions: dict[str, Sequence[dict[str, Any]]],
    world_partitions: dict[str, Sequence[LexicalWorld]],
    products: Sequence[dict[str, Any]],
) -> dict[str, Any]:
    named_records = {**partitions, "product": products}
    sentence_sets = {
        name: _sentence_inventory(records) for name, records in named_records.items()
    }
    lexical_sets = {
        name: _world_inventory(worlds) for name, worlds in world_partitions.items()
    }
    pair_names = sorted(sentence_sets)
    sentences_disjoint = all(
        sentence_sets[left].isdisjoint(sentence_sets[right])
        for left_index, left in enumerate(pair_names)
        for right in pair_names[left_index + 1 :]
    )
    lexical_names = sorted(lexical_sets)
    lexical_banks_disjoint = all(
        lexical_sets[left].isdisjoint(lexical_sets[right])
        for left_index, left in enumerate(lexical_names)
        for right in lexical_names[left_index + 1 :]
    )

    partition_checks: dict[str, Any] = {}
    for name, records in partitions.items():
        cells = Counter(
            (str(record["target_outcome"]), int(record["source_width"]))
            for record in records
        )
        per_width = Counter(int(record["source_width"]) for record in records)
        expected_per_outcome = {
            width: per_width[width] // len(OUTCOMES) for width in SOURCE_WIDTHS
        }
        balanced = all(
            cells[(outcome, width)] == expected_per_outcome[width]
            for outcome in OUTCOMES
            for width in SOURCE_WIDTHS
        )
        locative_polarities = _polarity_by_locative(records)
        query_outcomes = _query_outcomes(records)
        position_labels = _position_labels(records)
        partition_checks[name] = {
            "records": len(records),
            "balanced_outcomes_per_width": balanced,
            "every_locative_text_has_both_labels": bool(locative_polarities)
            and all(labels == {0, 1} for labels in locative_polarities.values()),
            "every_query_subject_has_answer_and_nonanswer": bool(query_outcomes)
            and all(values == {"answer", "nonanswer"} for values in query_outcomes.values()),
            "every_candidate_position_has_both_labels": bool(position_labels)
            and all(labels == {0, 1} for labels in position_labels.values()),
        }
    passed = (
        sentences_disjoint
        and lexical_banks_disjoint
        and all(
            all(
                bool(checks[field])
                for field in (
                    "balanced_outcomes_per_width",
                    "every_locative_text_has_both_labels",
                    "every_query_subject_has_answer_and_nonanswer",
                    "every_candidate_position_has_both_labels",
                )
            )
            for checks in partition_checks.values()
        )
    )
    value = {
        "schema": RELATION_CENSUS_SCHEMA,
        "policy": POLICY,
        "sentence_partitions_pairwise_disjoint": sentences_disjoint,
        "lexical_banks_pairwise_disjoint": lexical_banks_disjoint,
        "partition_checks": partition_checks,
        "passed": passed,
    }
    if not passed:
        raise RuntimeError(f"C1-SB3 zero-training census failed: {value}")
    return _canonical_with_cid(value, "census_cid")


def _product_population(
    *, ordinal_start: int
) -> tuple[list[LexicalWorld], dict[str, Any]]:
    selections = (
        ("answer", 0, "answer-supported"),
        ("abstain", 3, "abstain-negated-nonlocative"),
        ("conflict", 6, "conflict-distinct-values"),
        ("answer", 2, "answer-duplicate-agreement"),
    )
    worlds: list[LexicalWorld] = []
    records: list[dict[str, Any]] = []
    for lane, (outcome, motif_index, probe) in enumerate(selections):
        world = _world(
            partition="product",
            width=3,
            lane=lane,
            ordinal=ordinal_start + lane,
        )
        worlds.append(world)
        candidates = _world_records(world, population="product")
        selected = candidates[motif_index]
        if selected["target_outcome"] != outcome:
            raise RuntimeError("product motif outcome drifted")
        unsigned = dict(selected)
        unsigned.pop("record_cid")
        unsigned["probe"] = probe
        records.append(_canonical_with_cid(unsigned, "record_cid"))
    value = {
        "schema": RELATION_PRODUCT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "access_policy": (
            "write and bind this envelope before optimization; the trainer receives only "
            "product_probes_cid and record count and must not open record text until every "
            "pre-product gate passes"
        ),
        "records": records,
    }
    return worlds, _canonical_with_cid(value, "product_probes_cid")


def build_source_relation_adapter_population(
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Build the frozen C1-SB3 data, transfer preflight, and sealed products."""
    ordinal = 0
    fit_worlds, fit, ordinal = _world_partition(
        partition="preflight-fit",
        worlds_per_width=PREFLIGHT_FIT_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    sealed_worlds, sealed, ordinal = _world_partition(
        partition="preflight-sealed",
        worlds_per_width=PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    construction_worlds, construction, ordinal = _world_partition(
        partition="construction",
        worlds_per_width=CONSTRUCTION_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    development_worlds, development, ordinal = _world_partition(
        partition="development",
        worlds_per_width=DEVELOPMENT_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    product_worlds, products = _product_population(ordinal_start=ordinal)

    controls = _development_controls(development_worlds, development)
    census = _population_census(
        partitions={
            "preflight-fit": fit,
            "preflight-sealed": sealed,
            "construction": construction,
            "development": development,
        },
        world_partitions={
            "preflight-fit": fit_worlds,
            "preflight-sealed": sealed_worlds,
            "construction": construction_worlds,
            "development": development_worlds,
            "product": product_worlds,
        },
        products=list(products["records"]),
    )
    expected_counts = {
        "preflight_fit": 126,
        "preflight_sealed": 63,
        "construction": 756,
        "development": 252,
        "development_reversal_controls": 252,
        "development_query_swap_controls": 168,
        "product_probe_commitments": 4,
    }
    observed_counts = {
        "preflight_fit": len(fit),
        "preflight_sealed": len(sealed),
        "construction": len(construction),
        "development": len(development),
        "development_reversal_controls": len(controls["reversal"]),
        "development_query_swap_controls": len(controls["query_swap"]),
        "product_probe_commitments": len(products["records"]),
    }
    if observed_counts != expected_counts:
        raise RuntimeError(f"C1-SB3 population count drifted: {observed_counts}")

    preflight_value = {
        "schema": RELATION_PREFLIGHT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "selection": (
            "two fit lexical worlds and one independently sealed lexical world for "
            "each source width 2..8; every world has nine matched motifs"
        ),
        "counts": {"fit": len(fit), "sealed": len(sealed)},
        "fit_world_names": [world.name for world in fit_worlds],
        "sealed_world_names": [world.name for world in sealed_worlds],
        "fit": fit,
        "sealed": sealed,
        "census_cid": census["census_cid"],
    }
    preflight = _canonical_with_cid(preflight_value, "preflight_cid")
    split_policy = _canonical_with_cid(
        {
            "schema": RELATION_SPLIT_SCHEMA,
            "selection": (
                "independent world-level lexical banks; enumerate widths, fixed lanes, "
                "and all nine balanced counterfactual motifs without shuffling"
            ),
            "source_widths": list(SOURCE_WIDTHS),
            "motifs_per_outcome": MOTIFS_PER_OUTCOME,
            "worlds_per_width": {
                "preflight_fit": PREFLIGHT_FIT_WORLDS_PER_WIDTH,
                "preflight_sealed": PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
                "construction": CONSTRUCTION_WORLDS_PER_WIDTH,
                "development": DEVELOPMENT_WORLDS_PER_WIDTH,
            },
            "product_policy": (
                "four product records are separately committed and unopened by training"
            ),
        },
        "split_policy_cid",
    )
    dataset_value = {
        "schema": RELATION_DATASET_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "relation_input_policy": RELATION_INPUT_POLICY,
        "fixed_verbalizer": {
            "positive_token_id": YES_TOKEN_ID,
            "positive_token_text": YES_TOKEN_TEXT,
            "negative_token_id": NO_TOKEN_ID,
            "negative_token_text": NO_TOKEN_TEXT,
            "decision": "positive iff yes_logit - no_logit > 0; zero is negative",
        },
        "counts": observed_counts,
        "split_policy": split_policy,
        "split_policy_cid": split_policy["split_policy_cid"],
        "census": census,
        "census_cid": census["census_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": [
            record["record_cid"] for record in products["records"]
        ],
        "construction": construction,
        "development": development,
        "development_controls": controls,
    }
    dataset = _canonical_with_cid(dataset_value, "dataset_cid")
    return dataset, preflight, products
