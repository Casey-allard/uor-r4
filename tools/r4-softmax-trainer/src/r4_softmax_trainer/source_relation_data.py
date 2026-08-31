"""Frozen C1-SB2 source-relative relation population and sealed probes.

This module contains data and labels only.  It never loads the #1017 model,
extracts states, fits a head, or evaluates a reserved product record.
"""

from __future__ import annotations

from collections import Counter, defaultdict
from functools import lru_cache
from typing import Any, Iterable, Sequence

from .provenance import canonical_json_bytes, cid_bytes


ISSUE = 954
POLICY = "R4SourceRelativeRelationHeadV1"
RELATION_RECORD_SCHEMA = "uor-r4.source-relative-relation-record/1"
RELATION_DATASET_SCHEMA = "uor-r4.source-relative-relation-dataset/1"
RELATION_SPLIT_POLICY_SCHEMA = "uor-r4.source-relative-relation-split/1"
RELATION_PREFLIGHT_SCHEMA = "uor-r4.source-relative-relation-preflight/1"
RELATION_PRODUCT_PROBES_SCHEMA = "uor-r4.source-relative-relation-product-probes/1"
RELATION_SHORTCUT_CENSUS_SCHEMA = "uor-r4.source-relative-relation-shortcut-census/1"
RELATION_INPUT_POLICY = (
    "exact UTF-8 `Evidence:\\n<span>\\nQuestion:\\n<question>` with no terminal "
    "newline; evidence precedes the question and the retained final state is the "
    "question mark"
)
QUESTION_POLICY = "Where is the <subject>?"
SENTENCE_POLICY = "exact .!? terminated UTF-8 byte spans"
SOURCE_WIDTHS = tuple(range(2, 9))
OUTCOMES = ("answer", "abstain", "conflict")
ABSTAIN = "ABSTAIN"
CONTRADICTION = "CONTRADICTION"
CONSTRUCTION_PER_CELL = 160
DEVELOPMENT_PER_CELL = 20


CONSTRUCTION_BANK = {
    "name": "c1-sb2-construction",
    "subjects": (
        "azure compass",
        "bronze lantern",
        "ceramic falcon",
        "crimson ledger",
        "ivory kettle",
        "silver spindle",
        "violet banner",
        "marble flute",
        "canvas satchel",
        "walnut puzzle",
        "teal goblet",
        "linen drum",
        "pewter crown",
        "glass acorn",
        "ochre telescope",
        "indigo whistle",
        "scarlet thimble",
        "granite token",
        "willow map",
        "copper kite",
        "porcelain key",
        "umber book",
        "golden shell",
        "saffron bell",
    ),
    "locations": (
        "inside the willow cupboard",
        "beneath the granite arch",
        "beside the mossy fountain",
        "above the pantry window",
        "behind the cotton screen",
        "near the glass pavilion",
        "within the stone alcove",
        "under the copper awning",
        "inside the wicker hamper",
        "beneath the oak balcony",
        "beside the quiet canal",
        "above the cedar desk",
        "behind the garden trellis",
        "near the slate bridge",
        "within the paper chest",
        "under the narrow shelf",
        "inside the clay pantry",
        "beneath the round table",
        "beside the apple gate",
        "above the river bench",
        "behind the blue tapestry",
        "near the iron brazier",
        "within the birch closet",
        "under the vaulted passage",
    ),
    "nonlocatives": (
        "was inventoried at noon",
        "was cleaned after supper",
        "was photographed on Tuesday",
        "was repaired by the curator",
        "was weighed before breakfast",
        "was catalogued last winter",
        "was wrapped for transport",
        "was inspected during rehearsal",
        "was painted by an apprentice",
        "was displayed during the festival",
        "was counted twice yesterday",
        "was labeled for the archive",
    ),
}

DEVELOPMENT_BANK = {
    "name": "c1-sb2-development",
    "subjects": (
        "coral abacus",
        "jade pendulum",
        "navy brooch",
        "quartz horn",
        "russet helmet",
        "pearl tablet",
        "magenta shuttle",
        "obsidian wheel",
        "alabaster flute",
        "emerald stencil",
        "vermilion cup",
        "charcoal prism",
    ),
    "locations": (
        "inside the aspen locker",
        "beneath the tiled landing",
        "beside the fern courtyard",
        "above the brass railing",
        "behind the wool curtain",
        "near the marble kiosk",
        "within the elm cabinet",
        "under the painted canopy",
        "inside the reed basket",
        "beneath the limestone porch",
        "beside the silver cistern",
        "above the orchard wall",
    ),
    "nonlocatives": (
        "was audited at dusk",
        "was rinsed after the concert",
        "was sketched on Thursday",
        "was mended by the watchmaker",
        "was measured before lunch",
        "was indexed last spring",
        "was packed for exhibition",
        "was examined during intermission",
    ),
}

PREFLIGHT_FAMILIES = (
    {
        "name": "fit-quartz",
        "subject": "quartz chronometer",
        "distractor": "umber weather vane",
        "absent": "carmine sextant",
        "location_a": "inside the alder case",
        "location_b": "beside the basalt pillar",
        "nonlocative": "was serviced before the recital",
    },
    {
        "name": "fit-celadon",
        "subject": "celadon armillary",
        "distractor": "silver plumb bob",
        "absent": "indigo quadrant",
        "location_a": "beneath the ash gallery",
        "location_b": "near the brick colonnade",
        "nonlocative": "was engraved after the lecture",
    },
    {
        "name": "sealed-amber",
        "subject": "amber orrery",
        "distractor": "violet survey chain",
        "absent": "pearl inclinometer",
        "location_a": "within the beech hutch",
        "location_b": "under the chalk portico",
        "nonlocative": "was calibrated before the banquet",
    },
    {
        "name": "sealed-onyx",
        "subject": "onyx transit scope",
        "distractor": "scarlet level vial",
        "absent": "cerulean planimeter",
        "location_a": "behind the fir partition",
        "location_b": "above the sandstone landing",
        "nonlocative": "was documented after the procession",
    },
)

PRODUCT_OBJECTS = (
    "opal astrolabe",
    "brass sundial",
    "silk atlas",
    "jade sextant",
)
PRODUCT_LOCATIONS = (
    "beside the north alcove",
    "beneath the maple stair",
    "inside the cedar cabinet",
    "inside the linen drawer",
)
PRODUCT_NONLOCATIVES = (
    "was polished before sunrise",
    "was restored yesterday",
    "was calibrated yesterday",
)


def parse_subject(question: str) -> str:
    """Parse the sole admitted C1-SB2 question without loading trainer code."""
    prefix = "Where is the "
    if not question.startswith(prefix) or not question.endswith("?"):
        raise ValueError(f"question does not match {QUESTION_POLICY!r}")
    subject = question[len(prefix) : -1]
    if (
        not subject
        or subject != subject.strip()
        or "?" in subject
        or "\n" in subject
        or "\r" in subject
    ):
        raise ValueError(f"question does not match {QUESTION_POLICY!r}")
    return subject


def split_sentence_spans(source: str) -> list[dict[str, Any]]:
    """Split exact UTF-8 bytes on the admitted ASCII terminators."""
    if not source:
        raise ValueError("source is empty")
    encoded = source.encode("utf-8")
    spans: list[dict[str, Any]] = []
    byte_offset = 0
    byte_start: int | None = None
    for character in source:
        width = len(character.encode("utf-8"))
        if byte_start is None:
            if character.isspace():
                byte_offset += width
                continue
            byte_start = byte_offset
        byte_offset += width
        if character not in ".!?":
            continue
        raw = encoded[byte_start:byte_offset]
        text = raw.decode("utf-8")
        if not text[:-1].strip():
            raise ValueError("source contains an empty punctuation-only sentence")
        spans.append({"byte_start": byte_start, "byte_end": byte_offset, "text": text})
        if len(spans) > max(SOURCE_WIDTHS):
            raise ValueError("source exceeds the admitted sentence count")
        byte_start = None
    if byte_start is not None:
        raise ValueError("source has a non-whitespace suffix without .!? termination")
    return spans


def render_relation_input(span: str, question: str) -> str:
    """Render the exact evidence-first input ending at the question mark."""
    if not span or span != span.strip() or span[-1] not in ".!?":
        raise ValueError("relation evidence must be one trimmed terminated span")
    parse_subject(question)
    return f"Evidence:\n{span}\nQuestion:\n{question}"


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _question(subject: str) -> str:
    return f"Where is the {subject}?"


def _locative(subject: str, location: str) -> str:
    return f"The {subject} is {location}."


def _nonlocative(subject: str, phrase: str) -> str:
    return f"The {subject} {phrase}."


def _negated(subject: str, location: str) -> str:
    return f"The {subject} is not {location}."


def _entry(text: str, relation_label: int, role: str) -> dict[str, Any]:
    if relation_label not in (0, 1):
        raise ValueError("relation label must be binary")
    return {"text": text, "relation_label": relation_label, "role": role}


def _arrange(
    width: int,
    fixed: dict[int, dict[str, Any]],
    remaining: Sequence[dict[str, Any]],
    *,
    offset: int,
) -> list[dict[str, Any]]:
    if width not in SOURCE_WIDTHS or len(fixed) + len(remaining) != width:
        raise ValueError("candidate arrangement does not match the admitted width")
    ordered: list[dict[str, Any] | None] = [None] * width
    for index, value in fixed.items():
        if not 0 <= index < width or ordered[index] is not None:
            raise ValueError("candidate arrangement has a duplicate or invalid position")
        ordered[index] = value
    available = [index for index in range(width) if ordered[index] is None]
    if available:
        shift = offset % len(available)
        available = available[shift:] + available[:shift]
    for index, value in zip(available, remaining):
        ordered[index] = value
    if any(value is None for value in ordered):
        raise RuntimeError("candidate arrangement left an empty position")
    return [value for value in ordered if value is not None]


@lru_cache(maxsize=None)
def _balanced_pair_schedule(width: int) -> tuple[tuple[int, int], ...]:
    """Round-robin every unordered pair while balancing every prefix by position."""
    players: list[int | None] = list(range(width))
    if width % 2:
        players.append(None)
    rounds = len(players) - 1
    schedule: list[tuple[int, int]] = []
    for _ in range(rounds):
        round_pairs: list[tuple[int, int]] = []
        for index in range(len(players) // 2):
            left = players[index]
            right = players[-1 - index]
            if left is None or right is None:
                continue
            round_pairs.append(tuple(sorted((left, right))))
        schedule.extend(sorted(round_pairs))
        players = [players[0], players[-1], *players[1:-1]]
    expected = width * (width - 1) // 2
    if len(schedule) != expected or len(set(schedule)) != expected:
        raise RuntimeError("round-robin position schedule did not cover each pair once")
    return tuple(schedule)


def _pair(width: int, ordinal: int) -> tuple[int, int]:
    pairs = _balanced_pair_schedule(width)
    return pairs[ordinal % len(pairs)]


@lru_cache(maxsize=None)
def _answer_layout(
    width: int, records_per_cell: int
) -> tuple[tuple[int, ...], tuple[tuple[int, int], ...]]:
    """Balance singleton and duplicate positive incidence as one answer cell."""
    if records_per_cell % 2:
        raise ValueError("answer cell size must alternate evenly between its two motifs")
    motif_count = records_per_cell // 2
    duplicate_pairs = tuple(_pair(width, index) for index in range(motif_count))
    position_counts = Counter(
        position for pair in duplicate_pairs for position in pair
    )
    singleton_positions: list[int] = []
    for index in range(motif_count):
        position = min(
            range(width),
            key=lambda candidate: (
                position_counts[candidate],
                (candidate - index) % width,
            ),
        )
        singleton_positions.append(position)
        position_counts[position] += 1
    if max(position_counts.values()) - min(position_counts.values()) > 1:
        raise RuntimeError("answer position schedule is not balanced")
    return tuple(singleton_positions), duplicate_pairs


def _other_subjects(
    bank: dict[str, Any], subject: str, *, ordinal: int, count: int
) -> list[str]:
    subjects = tuple(str(value) for value in bank["subjects"])
    chosen: list[str] = []
    cursor = (ordinal * 5 + 3) % len(subjects)
    while len(chosen) < count:
        candidate = subjects[cursor % len(subjects)]
        cursor += 7
        if candidate == subject or candidate in chosen:
            continue
        chosen.append(candidate)
    return chosen


def _filler_entries(
    bank: dict[str, Any],
    subject: str,
    *,
    ordinal: int,
    width: int,
    count: int,
) -> list[dict[str, Any]]:
    others = _other_subjects(bank, subject, ordinal=ordinal + width, count=count)
    locations = tuple(str(value) for value in bank["locations"])
    return [
        _entry(
            _locative(other, locations[(ordinal * 7 + lane * 5 + width) % len(locations)]),
            0,
            "swap-locative" if lane == 0 else "locative-distractor",
        )
        for lane, other in enumerate(others)
    ]


def _make_record(
    *,
    population: str,
    motif: str,
    outcome: str,
    subject: str,
    entries: Sequence[dict[str, Any]],
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    if outcome not in OUTCOMES or len(entries) not in SOURCE_WIDTHS:
        raise ValueError("relation record outcome or source width is invalid")
    question = _question(subject)
    if parse_subject(question) != subject:
        raise RuntimeError("relation question did not reproduce its subject")
    source = " ".join(str(entry["text"]) for entry in entries)
    parsed = split_sentence_spans(source)
    if len(parsed) != len(entries):
        raise RuntimeError("relation source did not reproduce its candidate count")

    sentence_spans: list[dict[str, Any]] = []
    for index, (span, entry) in enumerate(zip(parsed, entries)):
        text = str(entry["text"])
        if span["text"] != text:
            raise RuntimeError("relation source span text changed during parsing")
        text_cid = cid_bytes(text.encode("utf-8"))
        relation_input = render_relation_input(text, question)
        sentence_spans.append(
            {
                "candidate_index": index,
                "byte_start": int(span["byte_start"]),
                "byte_end": int(span["byte_end"]),
                "text": text,
                "text_cid": text_cid,
                "role": str(entry["role"]),
                "relation_label": int(entry["relation_label"]),
                "relation_group_cid": text_cid,
                "relation_input": relation_input,
                "relation_input_cid": cid_bytes(relation_input.encode("utf-8")),
            }
        )

    positive = [
        int(span["candidate_index"])
        for span in sentence_spans
        if int(span["relation_label"]) == 1
    ]
    positive_groups = sorted(
        {str(sentence_spans[index]["relation_group_cid"]) for index in positive}
    )
    derived = (
        "abstain"
        if not positive_groups
        else "answer"
        if len(positive_groups) == 1
        else "conflict"
    )
    if derived != outcome:
        raise ValueError(
            f"relation labels derive {derived}, not the declared outcome {outcome}"
        )
    target_span_index: int | None = None
    answer = ABSTAIN if outcome == "abstain" else CONTRADICTION
    if outcome == "answer":
        target_span_index = min(positive)
        answer = str(sentence_spans[target_span_index]["text"])
    duplicate_agreement = outcome == "answer" and len(positive) > 1
    if duplicate_agreement and len(
        {sentence_spans[index]["text"] for index in positive}
    ) != 1:
        raise ValueError("answer agreement must consist of exact duplicate spans")
    if outcome == "conflict" and len(positive_groups) != 2:
        raise ValueError("conflict must contain exactly two distinct positive relations")

    value: dict[str, Any] = {
        "schema": RELATION_RECORD_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "population": population,
        "motif": motif,
        "target_outcome": outcome,
        "source_width": len(sentence_spans),
        "source": source,
        "source_cid": cid_bytes(source.encode("utf-8")),
        "question": question,
        "question_cid": cid_bytes(question.encode("utf-8")),
        "subject": subject,
        "sentence_spans": sentence_spans,
        "positive_span_indices": positive,
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


def _cell_record(
    bank: dict[str, Any],
    *,
    population: str,
    outcome: str,
    width: int,
    ordinal: int,
) -> dict[str, Any]:
    subjects = tuple(str(value) for value in bank["subjects"])
    locations = tuple(str(value) for value in bank["locations"])
    nonlocatives = tuple(str(value) for value in bank["nonlocatives"])
    subject = subjects[ordinal % len(subjects)]
    cycle = ordinal // len(subjects)
    location_a = locations[(cycle + width * 3 + OUTCOMES.index(outcome)) % len(locations)]
    location_b = locations[
        (cycle + width * 3 + OUTCOMES.index(outcome) + 1 + ordinal % 11)
        % len(locations)
    ]
    if location_b == location_a:
        location_b = locations[(locations.index(location_a) + 1) % len(locations)]
    nonlocative = nonlocatives[(ordinal * 7 + width) % len(nonlocatives)]
    negated_location = locations[(ordinal * 11 + width + 5) % len(locations)]

    fixed: dict[int, dict[str, Any]] = {}
    remaining: list[dict[str, Any]] = []
    motif: str
    if outcome == "answer":
        duplicate = ordinal % 2 == 1
        records_per_cell = (
            CONSTRUCTION_PER_CELL
            if population == "construction"
            else DEVELOPMENT_PER_CELL
        )
        singleton_positions, duplicate_pairs = _answer_layout(width, records_per_cell)
        if duplicate:
            motif = "duplicate-agreement"
            duplicate_fact = _entry(_locative(subject, location_a), 1, "agreement")
            first, second = duplicate_pairs[ordinal // 2]
            fixed[first] = duplicate_fact
            fixed[second] = dict(duplicate_fact)
            add_negative = width >= 3 and (ordinal // 2) % 2 == 1
            if add_negative:
                remaining.append(
                    _entry(_nonlocative(subject, nonlocative), 0, "same-subject-nonlocative")
                )
        else:
            motif = "singleton"
            target = singleton_positions[ordinal // 2]
            fixed[target] = _entry(_locative(subject, location_a), 1, "singleton-answer")
            remaining.append(
                _entry(_nonlocative(subject, nonlocative), 0, "same-subject-nonlocative")
            )
    elif outcome == "conflict":
        motif = "distinct-location-conflict"
        first, second = _pair(width, ordinal)
        fixed[first] = _entry(_locative(subject, location_a), 1, "conflict-location-a")
        fixed[second] = _entry(_locative(subject, location_b), 1, "conflict-location-b")
        if width >= 3 and ordinal % 2 == 1:
            remaining.append(
                _entry(_nonlocative(subject, nonlocative), 0, "same-subject-nonlocative")
            )
    else:
        motif = "same-subject-nonanswer"
        remaining.extend(
            (
                _entry(_nonlocative(subject, nonlocative), 0, "same-subject-nonlocative"),
                _entry(_negated(subject, negated_location), 0, "same-subject-negated"),
            )
        )
        if width >= 3 and ordinal % 2 == 1:
            second_phrase = nonlocatives[(ordinal * 7 + width + 1) % len(nonlocatives)]
            remaining.append(
                _entry(
                    _nonlocative(subject, second_phrase),
                    0,
                    "same-subject-nonlocative-second",
                )
            )

    filler_count = width - len(fixed) - len(remaining)
    if filler_count < 0:
        raise RuntimeError("relation cell overfilled its source width")
    remaining.extend(
        _filler_entries(
            bank,
            subject,
            ordinal=ordinal,
            width=width,
            count=filler_count,
        )
    )
    entries = _arrange(width, fixed, remaining, offset=ordinal + width)
    swap_indices = [
        index for index, entry in enumerate(entries) if entry["role"] == "swap-locative"
    ]
    extra: dict[str, Any] = {
        "cell_ordinal": ordinal,
        "lexical_bank": str(bank["name"]),
    }
    if swap_indices:
        swap_index = swap_indices[0]
        swap_subject = str(entries[swap_index]["text"])[4:].split(" is ", 1)[0]
        extra.update(
            {
                "query_swap_subject": swap_subject,
                "query_swap_target_span_index": swap_index,
            }
        )
    return _make_record(
        population=population,
        motif=motif,
        outcome=outcome,
        subject=subject,
        entries=entries,
        extra=extra,
    )


def _reversal_control(record: dict[str, Any]) -> dict[str, Any]:
    original = list(record["sentence_spans"])
    entries = [
        _entry(str(span["text"]), int(span["relation_label"]), str(span["role"]))
        for span in reversed(original)
    ]
    return _make_record(
        population="development-control",
        motif="reverse-candidate-order",
        outcome=str(record["target_outcome"]),
        subject=str(record["subject"]),
        entries=entries,
        extra={
            "control_kind": "order-reversal",
            "base_record_cid": str(record["record_cid"]),
            "candidate_original_indices": list(reversed(range(len(original)))),
        },
    )


def _query_swap_control(record: dict[str, Any]) -> dict[str, Any] | None:
    swap_subject = record.get("query_swap_subject")
    swap_index = record.get("query_swap_target_span_index")
    if not isinstance(swap_subject, str) or not isinstance(swap_index, int):
        return None
    entries = [
        _entry(
            str(span["text"]),
            int(int(span["candidate_index"]) == swap_index),
            str(span["role"]),
        )
        for span in record["sentence_spans"]
    ]
    return _make_record(
        population="development-control",
        motif="same-source-query-swap",
        outcome="answer",
        subject=swap_subject,
        entries=entries,
        extra={
            "control_kind": "query-swap",
            "base_record_cid": str(record["record_cid"]),
            "base_subject": str(record["subject"]),
            "expected_original_span_index": swap_index,
        },
    )


def _preflight_family_records(
    family: dict[str, str], *, population: str, family_index: int
) -> list[dict[str, Any]]:
    subject = family["subject"]
    distractor = family["distractor"]
    absent = family["absent"]
    location_a = family["location_a"]
    location_b = family["location_b"]
    nonlocative = family["nonlocative"]
    locative = _locative(subject, location_a)
    distractor_locative = _locative(distractor, location_b)
    subject_nonlocative = _nonlocative(subject, nonlocative)

    matched_source = [
        _entry(locative, 0, "matched-primary-locative"),
        _entry(distractor_locative, 0, "matched-distractor-locative"),
        _entry(subject_nonlocative, 0, "matched-primary-nonlocative"),
    ]
    shift = family_index % len(matched_source)
    matched_source = matched_source[shift:] + matched_source[:shift]
    absent_record = _make_record(
        population=population,
        motif="same-source-absent-query",
        outcome="abstain",
        subject=absent,
        entries=matched_source,
        extra={"lexical_family": family["name"]},
    )
    present_entries = [
        _entry(
            str(entry["text"]),
            int(str(entry["text"]) == locative),
            str(entry["role"]),
        )
        for entry in matched_source
    ]
    present_record = _make_record(
        population=population,
        motif="same-source-present-locative-query",
        outcome="answer",
        subject=subject,
        entries=present_entries,
        extra={"lexical_family": family["name"]},
    )

    nonlocative_source = [
        _entry(subject_nonlocative, 0, "matched-primary-nonlocative"),
        _entry(distractor_locative, 0, "matched-distractor-locative"),
    ]
    if family_index % 2:
        nonlocative_source.reverse()
    nonlocative_record = _make_record(
        population=population,
        motif="queried-subject-nonlocative-only",
        outcome="abstain",
        subject=subject,
        entries=nonlocative_source,
        extra={"lexical_family": family["name"]},
    )
    distractor_entries = [
        _entry(
            str(entry["text"]),
            int(str(entry["text"]) == distractor_locative),
            str(entry["role"]),
        )
        for entry in nonlocative_source
    ]
    distractor_record = _make_record(
        population=population,
        motif="query-locative-distractor-subject",
        outcome="answer",
        subject=distractor,
        entries=distractor_entries,
        extra={"lexical_family": family["name"]},
    )

    duplicate_entries = [
        _entry(locative, 1, "agreement"),
        _entry(locative, 1, "agreement"),
        _entry(subject_nonlocative, 0, "same-subject-nonlocative"),
    ]
    duplicate_shift = (family_index + 1) % len(duplicate_entries)
    duplicate_entries = duplicate_entries[duplicate_shift:] + duplicate_entries[:duplicate_shift]
    duplicate_record = _make_record(
        population=population,
        motif="exact-duplicate-agreement",
        outcome="answer",
        subject=subject,
        entries=duplicate_entries,
        extra={"lexical_family": family["name"]},
    )
    conflict_entries = [
        _entry(locative, 1, "conflict-location-a"),
        _entry(_locative(subject, location_b), 1, "conflict-location-b"),
        _entry(subject_nonlocative, 0, "same-subject-nonlocative"),
    ]
    conflict_shift = (family_index + 2) % len(conflict_entries)
    conflict_entries = conflict_entries[conflict_shift:] + conflict_entries[:conflict_shift]
    conflict_record = _make_record(
        population=population,
        motif="distinct-location-conflict",
        outcome="conflict",
        subject=subject,
        entries=conflict_entries,
        extra={"lexical_family": family["name"]},
    )
    return [
        absent_record,
        present_record,
        nonlocative_record,
        distractor_record,
        duplicate_record,
        conflict_record,
    ]


def build_relation_preflight() -> dict[str, Any]:
    """Build the frozen 12-fit/12-sealed matched-transfer preflight."""
    fit: list[dict[str, Any]] = []
    sealed: list[dict[str, Any]] = []
    matched_pairs: list[dict[str, Any]] = []
    for family_index, family in enumerate(PREFLIGHT_FAMILIES):
        target = fit if family_index < 2 else sealed
        population = "preflight-fit" if family_index < 2 else "preflight-sealed"
        records = _preflight_family_records(
            family, population=population, family_index=family_index
        )
        target.extend(records)
        matched_pairs.extend(
            (
                {
                    "lexical_family": family["name"],
                    "kind": "absent-to-present",
                    "left_record_cid": records[0]["record_cid"],
                    "right_record_cid": records[1]["record_cid"],
                    "same_source": records[0]["source_cid"] == records[1]["source_cid"],
                },
                {
                    "lexical_family": family["name"],
                    "kind": "nonlocative-to-locative-distractor",
                    "left_record_cid": records[2]["record_cid"],
                    "right_record_cid": records[3]["record_cid"],
                    "same_source": records[2]["source_cid"] == records[3]["source_cid"],
                },
            )
        )
    value: dict[str, Any] = {
        "schema": RELATION_PREFLIGHT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "selection": (
            "four disjoint lexical families; fit the first two families' six matched "
            "motifs and evaluate the final two families without fitting"
        ),
        "counts": {"fit": len(fit), "sealed": len(sealed), "matched_pairs": len(matched_pairs)},
        "fit_family_names": [family["name"] for family in PREFLIGHT_FAMILIES[:2]],
        "sealed_family_names": [family["name"] for family in PREFLIGHT_FAMILIES[2:]],
        "fit": fit,
        "sealed": sealed,
        "matched_pairs": matched_pairs,
    }
    return _canonical_with_cid(value, "preflight_cid")


def _product_record(
    *,
    probe: str,
    outcome: str,
    subject: str,
    entries: Sequence[dict[str, Any]],
) -> dict[str, Any]:
    return _make_record(
        population="product",
        motif=probe,
        outcome=outcome,
        subject=subject,
        entries=entries,
        extra={"probe": probe},
    )


def product_probes() -> dict[str, Any]:
    """Return four committed records; Python feature and fit code must not open them."""
    opal_nonlocative = "The opal astrolabe was polished before sunrise."
    brass_locative = "The brass sundial is beside the north alcove."
    opal_maple = "The opal astrolabe is beneath the maple stair."
    silk_nonlocative = "The silk atlas was restored yesterday."
    silk_negated = "The silk atlas is not beneath the maple stair."
    opal_cedar = "The opal astrolabe is inside the cedar cabinet."
    jade_linen = "The jade sextant is inside the linen drawer."
    jade_nonlocative = "The jade sextant was calibrated yesterday."
    records = [
        _product_record(
            probe="opal-astrolabe-supported",
            outcome="answer",
            subject="opal astrolabe",
            entries=(
                _entry(opal_nonlocative, 0, "same-subject-nonlocative"),
                _entry(brass_locative, 0, "locative-distractor"),
                _entry(opal_maple, 1, "singleton-answer"),
            ),
        ),
        _product_record(
            probe="silk-atlas-abstain",
            outcome="abstain",
            subject="silk atlas",
            entries=(
                _entry(silk_nonlocative, 0, "same-subject-nonlocative"),
                _entry(silk_negated, 0, "same-subject-negated"),
                _entry(brass_locative, 0, "locative-distractor"),
            ),
        ),
        _product_record(
            probe="opal-astrolabe-conflict",
            outcome="conflict",
            subject="opal astrolabe",
            entries=(
                _entry(opal_maple, 1, "conflict-location-a"),
                _entry(opal_cedar, 1, "conflict-location-b"),
                _entry(opal_nonlocative, 0, "same-subject-nonlocative"),
            ),
        ),
        _product_record(
            probe="jade-sextant-duplicate-agreement",
            outcome="answer",
            subject="jade sextant",
            entries=(
                _entry(jade_linen, 1, "agreement"),
                _entry(jade_linen, 1, "agreement"),
                _entry(jade_nonlocative, 0, "same-subject-nonlocative"),
            ),
        ),
    ]
    value: dict[str, Any] = {
        "schema": RELATION_PRODUCT_PROBES_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "access_policy": (
            "commit canonical record CIDs before feature extraction; exclude product text "
            "from Python feature extraction, preflight, fitting, and development evaluation"
        ),
        "records": records,
    }
    return _canonical_with_cid(value, "product_probes_cid")


def _sentence_inventory(records: Iterable[dict[str, Any]]) -> set[str]:
    return {
        str(span["text"])
        for record in records
        for span in record["sentence_spans"]
    }


def _subject_inventory(records: Iterable[dict[str, Any]]) -> set[str]:
    return {str(record["subject"]) for record in records}


def _count_outcomes(records: Iterable[dict[str, Any]]) -> dict[int, Counter[str]]:
    cells: dict[int, Counter[str]] = defaultdict(Counter)
    for record in records:
        cells[int(record["raw_subject_occurrence_count"])][
            str(record["target_outcome"])
        ] += 1
    return dict(cells)


def shortcut_census(
    construction: Sequence[dict[str, Any]],
    development: Sequence[dict[str, Any]],
    products: dict[str, Any],
) -> dict[str, Any]:
    """Prove before training that raw mention count and lexical overlap are shortcuts."""
    construction_cells = _count_outcomes(construction)
    development_cells = _count_outcomes(development)

    def summarize(cells: dict[int, Counter[str]]) -> list[dict[str, Any]]:
        return [
            {
                "raw_subject_occurrences": count,
                "outcome_counts": {name: cells[count][name] for name in OUTCOMES},
                "outcomes_present": [name for name in OUTCOMES if cells[count][name]],
            }
            for count in sorted(cells)
        ]

    construction_subjects = _subject_inventory(construction)
    development_subjects = _subject_inventory(development)
    product_records = list(products["records"])
    product_subjects = _subject_inventory(product_records)
    construction_sentences = _sentence_inventory(construction)
    development_sentences = _sentence_inventory(development)
    product_sentences = _sentence_inventory(product_records)
    count_is_ambiguous = all(
        len([name for name in OUTCOMES if counter[name]]) >= 2
        for cells in (construction_cells, development_cells)
        for counter in cells.values()
    )
    subject_disjoint = not (
        construction_subjects & development_subjects
        or construction_subjects & product_subjects
        or development_subjects & product_subjects
    )
    sentence_disjoint = not (
        construction_sentences & development_sentences
        or construction_sentences & product_sentences
        or development_sentences & product_sentences
    )
    value: dict[str, Any] = {
        "schema": RELATION_SHORTCUT_CENSUS_SCHEMA,
        "policy": POLICY,
        "raw_subject_occurrence_definition": (
            "sum exact queried-subject substring occurrences across admitted sentence spans; "
            "relation labels and locative parsing are not consulted"
        ),
        "construction_count_cells": summarize(construction_cells),
        "development_count_cells": summarize(development_cells),
        "count_only_label_lookup_is_perfect": not count_is_ambiguous,
        "construction_development_subjects_disjoint": not (
            construction_subjects & development_subjects
        ),
        "construction_development_sentences_disjoint": not (
            construction_sentences & development_sentences
        ),
        "construction_development_product_subjects_disjoint": subject_disjoint,
        "construction_development_product_sentences_disjoint": sentence_disjoint,
        "passed": count_is_ambiguous and subject_disjoint and sentence_disjoint,
    }
    if not value["passed"]:
        raise RuntimeError("C1-SB2 zero-training shortcut census failed")
    return _canonical_with_cid(value, "census_cid")


def _bank_contract(bank: dict[str, Any]) -> dict[str, Any]:
    value = {
        "name": str(bank["name"]),
        "subjects": list(bank["subjects"]),
        "locations": list(bank["locations"]),
        "nonlocatives": list(bank["nonlocatives"]),
    }
    return _canonical_with_cid(value, "bank_cid")


def _assert_lexical_banks_disjoint() -> None:
    construction = {
        *CONSTRUCTION_BANK["subjects"],
        *CONSTRUCTION_BANK["locations"],
        *CONSTRUCTION_BANK["nonlocatives"],
    }
    development = {
        *DEVELOPMENT_BANK["subjects"],
        *DEVELOPMENT_BANK["locations"],
        *DEVELOPMENT_BANK["nonlocatives"],
    }
    product = {*PRODUCT_OBJECTS, *PRODUCT_LOCATIONS, *PRODUCT_NONLOCATIVES}
    preflight = {
        value
        for family in PREFLIGHT_FAMILIES
        for key, value in family.items()
        if key != "name"
    }
    if (
        construction & development
        or construction & product
        or development & product
        or preflight & construction
        or preflight & development
        or preflight & product
    ):
        raise RuntimeError("C1-SB2 lexical banks overlap")


def build_source_relation_population() -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Build the full frozen dataset, matched preflight, and sealed product file."""
    _assert_lexical_banks_disjoint()
    construction = [
        _cell_record(
            CONSTRUCTION_BANK,
            population="construction",
            outcome=outcome,
            width=width,
            ordinal=ordinal,
        )
        for outcome in OUTCOMES
        for width in SOURCE_WIDTHS
        for ordinal in range(CONSTRUCTION_PER_CELL)
    ]
    development = [
        _cell_record(
            DEVELOPMENT_BANK,
            population="development",
            outcome=outcome,
            width=width,
            ordinal=ordinal,
        )
        for outcome in OUTCOMES
        for width in SOURCE_WIDTHS
        for ordinal in range(DEVELOPMENT_PER_CELL)
    ]
    if len(construction) != 3_360 or len(development) != 420:
        raise RuntimeError("C1-SB2 population count drifted")
    for label, records in (("construction", construction), ("development", development)):
        if len({record["record_cid"] for record in records}) != len(records):
            raise RuntimeError(f"C1-SB2 {label} records collide")

    preflight = build_relation_preflight()
    products = product_probes()
    reversal_controls = [_reversal_control(record) for record in development]
    query_swap_controls = [
        control
        for record in development
        if (control := _query_swap_control(record)) is not None
    ]
    census = shortcut_census(construction, development, products)
    construction_bank = _bank_contract(CONSTRUCTION_BANK)
    development_bank = _bank_contract(DEVELOPMENT_BANK)
    generator_contract = _canonical_with_cid(
        {
            "schema": "uor-r4.source-relative-relation-generator/1",
            "policy": POLICY,
            "seed": 9_542,
            "source_widths": list(SOURCE_WIDTHS),
            "outcomes": list(OUTCOMES),
            "construction_per_cell": CONSTRUCTION_PER_CELL,
            "development_per_cell": DEVELOPMENT_PER_CELL,
            "answer_policy": "alternate singleton and exact-duplicate agreement",
            "abstain_policy": "zero positive locative relations with same-subject nonanswers",
            "conflict_policy": "two positive assertions at distinct locations",
            "position_policy": (
                "cycle singleton positions and unordered duplicate/conflict position pairs"
            ),
            "relation_input_policy": RELATION_INPUT_POLICY,
        },
        "generator_contract_cid",
    )
    split_policy = _canonical_with_cid(
        {
            "schema": RELATION_SPLIT_POLICY_SCHEMA,
            "selection": (
                "independent construction/development lexical banks; enumerate outcomes, "
                "widths 2..8, and fixed per-cell ordinals without shuffling"
            ),
            "construction_bank_cid": construction_bank["bank_cid"],
            "development_bank_cid": development_bank["bank_cid"],
            "construction_records": len(construction),
            "development_records": len(development),
            "product_records": len(products["records"]),
            "preflight_policy": "12 fit records and 12 sealed-transfer records",
            "product_policy": (
                "four canonical product commitments excluded from every Python data access"
            ),
        },
        "split_policy_cid",
    )
    dataset_value: dict[str, Any] = {
        "schema": RELATION_DATASET_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "relation_input_policy": RELATION_INPUT_POLICY,
        "counts": {
            "construction": len(construction),
            "development": len(development),
            "development_reversal_controls": len(reversal_controls),
            "development_query_swap_controls": len(query_swap_controls),
            "preflight_fit": len(preflight["fit"]),
            "preflight_sealed": len(preflight["sealed"]),
            "product_probe_commitments": len(products["records"]),
        },
        "construction_bank": construction_bank,
        "development_bank": development_bank,
        "generator_contract": generator_contract,
        "generator_contract_cid": generator_contract["generator_contract_cid"],
        "split_policy": split_policy,
        "split_policy_cid": split_policy["split_policy_cid"],
        "shortcut_census": census,
        "preflight_cid": preflight["preflight_cid"],
        "preflight_fit_commitments": [record["record_cid"] for record in preflight["fit"]],
        "preflight_sealed_commitments": [
            record["record_cid"] for record in preflight["sealed"]
        ],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": [
            record["record_cid"] for record in products["records"]
        ],
        "construction": construction,
        "development": development,
        "development_controls": {
            "reversal": reversal_controls,
            "query_swap": query_swap_controls,
        },
    }
    dataset = _canonical_with_cid(dataset_value, "dataset_cid")
    return dataset, preflight, products
