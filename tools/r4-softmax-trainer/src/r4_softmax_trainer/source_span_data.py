"""Frozen construction, development, and sealed product data for C1-SB1."""

from __future__ import annotations

from typing import Any

from .grounding_data import (
    ABSTAIN,
    CONTRADICTION,
    build_grounding_corpus,
)
from .provenance import canonical_json_bytes, cid_bytes


SOURCE_SPAN_DATASET_SCHEMA = "uor-r4.source-span-pointer-dataset/1"
SOURCE_SPAN_SPLIT_POLICY_SCHEMA = "uor-r4.source-span-pointer-split/1"
PRODUCT_PROBES_SCHEMA = "uor-r4.source-span-pointer-product-probes/1"
QUESTION_POLICY = "Where is the <subject>?"
SENTENCE_POLICY = "exact .!? terminated UTF-8 byte spans"
MAXIMUM_SOURCE_SPANS = 8
OUTCOMES = ("answer", "abstain", "conflict")


def parse_subject(question: str) -> str:
    """Parse the sole admitted question shape and reject every other form."""
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
    """Split exact UTF-8 bytes on ASCII ``.``, ``!``, or ``?`` terminators."""
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
        assert byte_start is not None
        raw = encoded[byte_start:byte_offset]
        text = raw.decode("utf-8")
        if not text[:-1].strip():
            raise ValueError("source contains an empty punctuation-only sentence")
        spans.append(
            {
                "byte_start": byte_start,
                "byte_end": byte_offset,
                "text": text,
            }
        )
        if len(spans) > MAXIMUM_SOURCE_SPANS:
            raise ValueError(f"source exceeds {MAXIMUM_SOURCE_SPANS} sentence spans")
        byte_start = None
    if byte_start is not None:
        raise ValueError("source has a non-whitespace suffix without .!? termination")
    if not spans:
        raise ValueError("source has no punctuation-terminated sentence")
    for span in spans:
        start = int(span["byte_start"])
        end = int(span["byte_end"])
        if encoded[start:end].decode("utf-8") != span["text"]:
            raise RuntimeError("sentence byte span does not reproduce exact UTF-8")
    return spans


def _target_outcome(category: str) -> str:
    mapping = {
        "supported": "answer",
        "unsupported": "abstain",
        "contradiction": "conflict",
    }
    try:
        return mapping[category]
    except KeyError as error:
        raise ValueError(f"unknown grounding category: {category}") from error


def pointer_record(record: dict[str, str]) -> dict[str, Any]:
    """Convert one existing #954 grounding record into the frozen pointer form."""
    subject = parse_subject(record["question"])
    spans = split_sentence_spans(record["source"])
    outcome = _target_outcome(record["category"])
    target_span_index: int | None = None
    if outcome == "answer":
        matches = [
            index for index, span in enumerate(spans) if span["text"] == record["answer"]
        ]
        if len(matches) != 1:
            raise ValueError("supported answer must equal exactly one admitted sentence span")
        target_span_index = matches[0]
    elif outcome == "abstain":
        if record["answer"] != ABSTAIN:
            raise ValueError("unsupported pointer record does not target ABSTAIN")
        if any(subject in str(span["text"]) for span in spans):
            raise ValueError("unsupported pointer record contains the queried subject")
    else:
        if record["answer"] != CONTRADICTION:
            raise ValueError("contradiction pointer record does not target CONTRADICTION")
        if sum(subject in str(span["text"]) for span in spans) < 2:
            raise ValueError("conflict pointer record needs two subject-bearing spans")

    value: dict[str, Any] = {
        "source_record_cid": record["record_cid"],
        "category": record["category"],
        "target_outcome": outcome,
        "source": record["source"],
        "question": record["question"],
        "subject": subject,
        "sentence_spans": spans,
        "target_span_index": target_span_index,
    }
    value["record_cid"] = cid_bytes(canonical_json_bytes(value))
    return value


def _probe_record(
    *,
    name: str,
    category: str,
    source: str,
    question: str,
    answer: str,
) -> dict[str, Any]:
    source_record = {
        "record_cid": cid_bytes(
            canonical_json_bytes(
                {
                    "category": category,
                    "source": source,
                    "question": question,
                    "answer": answer,
                    "population": "C1-SB1-reserved-product",
                }
            )
        ),
        "category": category,
        "source": source,
        "question": question,
        "answer": answer,
    }
    return {"probe": name, **pointer_record(source_record)}


def product_probes() -> dict[str, Any]:
    """Return the three newly reserved records; callers must never fit on them."""
    supported_source = (
        "The ivory lantern is beside the river stone. "
        "The copper compass is beneath the cedar bench."
    )
    unsupported_source = (
        "The porcelain star is within the canvas satchel. "
        "The willow flute is beyond the marble arch."
    )
    conflict_source = (
        "The copper compass is beneath the cedar bench. "
        "The copper compass is within the canvas satchel."
    )
    records = [
        _probe_record(
            name="copper-compass-supported",
            category="supported",
            source=supported_source,
            question="Where is the copper compass?",
            answer="The copper compass is beneath the cedar bench.",
        ),
        _probe_record(
            name="linen-map-unsupported",
            category="unsupported",
            source=unsupported_source,
            question="Where is the linen map?",
            answer=ABSTAIN,
        ),
        _probe_record(
            name="copper-compass-conflict",
            category="contradiction",
            source=conflict_source,
            question="Where is the copper compass?",
            answer=CONTRADICTION,
        ),
    ]
    value: dict[str, Any] = {
        "schema": PRODUCT_PROBES_SCHEMA,
        "policy": "three exact records reserved before feature extraction and never evaluated by Python",
        "records": records,
    }
    value["product_probes_cid"] = cid_bytes(canonical_json_bytes(value))
    return value


def source_span_split_policy(source_dataset_cid: str) -> dict[str, Any]:
    return {
        "schema": SOURCE_SPAN_SPLIT_POLICY_SCHEMA,
        "source_grounding_dataset_cid": source_dataset_cid,
        "selection": (
            "reuse all 3072 construction and 384 development records from the "
            "deterministic balanced #954 generator; transform without reordering"
        ),
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "maximum_source_spans": MAXIMUM_SOURCE_SPANS,
        "product_probe_policy": (
            "commit three disjoint records in product-probes.json; exclude them from "
            "feature extraction, preflight, fitting, and development evaluation"
        ),
    }


def build_source_span_population() -> tuple[dict[str, Any], dict[str, Any]]:
    """Build the exact training view and separate sealed product-probe file."""
    grounding = build_grounding_corpus()
    construction = [pointer_record(record) for record in grounding["train"]]
    development = [pointer_record(record) for record in grounding["development"]]
    probes = product_probes()
    commitments = [record["record_cid"] for record in probes["records"]]

    if len(construction) != 3_072 or len(development) != 384:
        raise RuntimeError("source-span population does not preserve the frozen counts")
    all_records = [*construction, *development]
    if len({record["record_cid"] for record in all_records}) != len(all_records):
        raise RuntimeError("source-span construction/development records collide")
    reserved_terms = (
        "copper compass",
        "linen map",
        "beneath the cedar bench",
        "within the canvas satchel",
        "beyond the marble arch",
        "beside the river stone",
    )
    training_text = "\n".join(
        str(record[field])
        for record in all_records
        for field in ("source", "question", "subject")
    )
    if any(term in training_text for term in reserved_terms):
        raise RuntimeError("reserved C1-SB1 product family leaked into training data")

    split_policy = source_span_split_policy(str(grounding["dataset_cid"]))
    split_policy_cid = cid_bytes(canonical_json_bytes(split_policy))
    dataset: dict[str, Any] = {
        "schema": SOURCE_SPAN_DATASET_SCHEMA,
        "source_grounding_dataset_cid": grounding["dataset_cid"],
        "split_policy": split_policy,
        "split_policy_cid": split_policy_cid,
        "counts": {
            "construction": len(construction),
            "development": len(development),
            "product_probe_commitments": len(commitments),
        },
        "product_probe_commitments": commitments,
        "construction": construction,
        "development": development,
    }
    dataset["dataset_cid"] = cid_bytes(canonical_json_bytes(dataset))
    return dataset, probes
