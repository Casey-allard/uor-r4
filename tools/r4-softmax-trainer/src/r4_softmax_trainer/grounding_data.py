"""Deterministic context-grounding examples for the bounded #954 SFT run."""

from __future__ import annotations

import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterator, Sequence

import torch
from blake3 import blake3
from tokenizers import Tokenizer
from torch import Tensor

from .constants import BOS_TOKEN_ID, EOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .provenance import canonical_json_bytes, cid_bytes


GROUNDING_DATASET_SCHEMA = "uor-r4-softmax-grounding-dataset/1"
GROUNDING_SPLIT_POLICY_SCHEMA = "uor-r4-softmax-grounding-split/1"
GROUNDING_PROMPT = (
    "Use only the context. Copy one exact contiguous answer span from the context. "
    "If the context does not answer the question, write ABSTAIN. If the context gives "
    "conflicting answers, write CONTRADICTION."
)
ABSTAIN = "ABSTAIN"
CONTRADICTION = "CONTRADICTION"
TRAIN_EXAMPLES_PER_CLASS = 1_024
DEV_EXAMPLES_PER_CLASS = 128
GROUNDING_CLASSES = ("supported", "unsupported", "contradiction")


_OBJECTS = (
    "blue book",
    "red scarf",
    "brass bell",
    "white shell",
    "purple kite",
    "small drum",
    "green cup",
    "black stone",
    "yellow ribbon",
    "paper crown",
    "wooden horse",
    "orange ball",
    "glass bead",
    "tin whistle",
    "brown basket",
    "pink flower",
    "gray feather",
    "golden button",
)

_LOCATIONS = (
    "inside the wicker chest",
    "under the kitchen chair",
    "beside the oak door",
    "on the library shelf",
    "behind the blue curtain",
    "near the stone fountain",
    "inside the paper bag",
    "under the narrow bridge",
    "beside the apple tree",
    "on the bedroom desk",
    "behind the garden shed",
    "near the quiet pond",
    "inside the wooden cupboard",
    "under the red blanket",
    "beside the front window",
    "on the round stool",
    "behind the tall clock",
    "near the little boat",
)


def render_grounding_prompt(source: str, question: str) -> str:
    """Return the one exact prompt contract shared by training and Rust inference."""
    return (
        f"{GROUNDING_PROMPT}\n"
        f"Context:\n{source}\n"
        f"Question:\n{question}\n"
        "Answer:\n"
    )


def _fact(object_name: str, location: str) -> str:
    return f"The {object_name} is {location}."


def _question(object_name: str) -> str:
    return f"Where is the {object_name}?"


def _record(category: str, source: str, question: str, answer: str) -> dict[str, str]:
    if category not in GROUNDING_CLASSES:
        raise ValueError(f"unknown grounding category: {category}")
    if category == "supported" and answer not in source:
        raise ValueError("supported answer is not an exact contiguous source span")
    if category == "unsupported" and answer != ABSTAIN:
        raise ValueError("unsupported records must target ABSTAIN")
    if category == "contradiction" and answer != CONTRADICTION:
        raise ValueError("contradiction records must target CONTRADICTION")
    value = {
        "category": category,
        "source": source,
        "question": question,
        "prompt": render_grounding_prompt(source, question),
        "answer": answer,
    }
    return {"record_cid": cid_bytes(canonical_json_bytes(value)), **value}


def _rotated_facts(facts: list[str], variant: int) -> str:
    offset = variant % len(facts)
    ordered = facts[offset:] + facts[:offset]
    return " ".join(ordered)


def _other_object(query_index: int, offset: int, *, exclude: set[int]) -> int:
    candidate = (query_index + offset) % len(_OBJECTS)
    while candidate in exclude:
        candidate = (candidate + 1) % len(_OBJECTS)
    return candidate


def _candidate_records() -> dict[str, list[dict[str, str]]]:
    candidates: dict[str, dict[str, dict[str, str]]] = {
        category: {} for category in GROUNDING_CLASSES
    }
    for query_index, object_name in enumerate(_OBJECTS):
        for location_index, location in enumerate(_LOCATIONS):
            for variant in range(8):
                first_index = _other_object(
                    query_index, variant * 5 + 1, exclude={query_index}
                )
                second_index = _other_object(
                    query_index,
                    variant * 7 + 2,
                    exclude={query_index, first_index},
                )
                first_location = _LOCATIONS[(location_index + variant * 3 + 1) % len(_LOCATIONS)]
                second_location = _LOCATIONS[(location_index + variant * 5 + 2) % len(_LOCATIONS)]

                answer = _fact(object_name, location)
                supported_source = _rotated_facts(
                    [
                        answer,
                        _fact(_OBJECTS[first_index], first_location),
                        _fact(_OBJECTS[second_index], second_location),
                    ],
                    variant,
                )
                supported = _record(
                    "supported", supported_source, _question(object_name), answer
                )
                candidates["supported"][supported["record_cid"]] = supported

                unsupported_source = _rotated_facts(
                    [
                        _fact(_OBJECTS[first_index], first_location),
                        _fact(_OBJECTS[second_index], second_location),
                    ],
                    variant,
                )
                unsupported = _record(
                    "unsupported",
                    unsupported_source,
                    _question(object_name),
                    ABSTAIN,
                )
                candidates["unsupported"][unsupported["record_cid"]] = unsupported

                conflicting_location = _LOCATIONS[
                    (location_index + variant + 1) % len(_LOCATIONS)
                ]
                if conflicting_location == location:
                    raise AssertionError("contradiction generator selected one location twice")
                contradiction_source = _rotated_facts(
                    [
                        _fact(object_name, location),
                        _fact(object_name, conflicting_location),
                        _fact(_OBJECTS[first_index], first_location),
                    ],
                    variant,
                )
                contradiction = _record(
                    "contradiction",
                    contradiction_source,
                    _question(object_name),
                    CONTRADICTION,
                )
                candidates["contradiction"][contradiction["record_cid"]] = contradiction
    return {
        category: sorted(records.values(), key=lambda record: record["record_cid"])
        for category, records in candidates.items()
    }


def product_probes() -> list[dict[str, str]]:
    """Return three fixed prompts whose object/location combinations never train."""
    shared_source = (
        "The amber coin is inside the green basket. "
        "The silver key is beside the garden gate."
    )
    probes = [
        _record(
            "supported",
            shared_source,
            "Where is the amber coin?",
            "The amber coin is inside the green basket.",
        ),
        _record(
            "unsupported",
            shared_source,
            "Where is the velvet ribbon?",
            ABSTAIN,
        ),
        _record(
            "contradiction",
            (
                "The amber coin is inside the green basket. "
                "The amber coin is under the wooden table."
            ),
            "Where is the amber coin?",
            CONTRADICTION,
        ),
    ]
    return [
        {"probe": f"grounding-{index + 1}", **probe}
        for index, probe in enumerate(probes)
    ]


def grounding_split_policy() -> dict[str, Any]:
    return {
        "schema": GROUNDING_SPLIT_POLICY_SCHEMA,
        "selection": (
            "enumerate deterministic templates, deduplicate by record CID, sort by CID, "
            "take 128 development records then 1024 training records independently per class"
        ),
        "classes": list(GROUNDING_CLASSES),
        "train_examples_per_class": TRAIN_EXAMPLES_PER_CLASS,
        "development_examples_per_class": DEV_EXAMPLES_PER_CLASS,
        "product_probe_policy": (
            "three fixed reserved object/location combinations excluded from all generated "
            "training and development templates"
        ),
    }


def build_grounding_corpus() -> dict[str, Any]:
    candidates = _candidate_records()
    train: list[dict[str, str]] = []
    development: list[dict[str, str]] = []
    required = TRAIN_EXAMPLES_PER_CLASS + DEV_EXAMPLES_PER_CLASS
    for category in GROUNDING_CLASSES:
        records = candidates[category]
        if len(records) < required:
            raise RuntimeError(f"grounding generator produced too few {category} records")
        development.extend(records[:DEV_EXAMPLES_PER_CLASS])
        train.extend(records[DEV_EXAMPLES_PER_CLASS:required])
    train.sort(key=lambda record: record["record_cid"])
    development.sort(key=lambda record: record["record_cid"])
    probes = product_probes()
    seen_prompts = {record["prompt"] for record in [*train, *development]}
    if any(probe["prompt"] in seen_prompts for probe in probes):
        raise RuntimeError("a frozen product probe leaked into the grounding corpus")
    value: dict[str, Any] = {
        "schema": GROUNDING_DATASET_SCHEMA,
        "prompt_contract": GROUNDING_PROMPT,
        "split_policy": grounding_split_policy(),
        "counts": {
            "train": len(train),
            "development": len(development),
            "product_probes": len(probes),
        },
        "train": train,
        "development": development,
        "product_probes": probes,
    }
    value["dataset_cid"] = cid_bytes(canonical_json_bytes(value))
    return value


@dataclass(frozen=True, slots=True)
class EncodedGroundingExample:
    inputs: Tensor
    targets: Tensor
    supervised_tokens: int


class GroundingStore:
    """In-memory, fixed-context SFT examples with prompt and padding loss masked."""

    def __init__(self, records: Sequence[dict[str, str]], tokenizer_path: Path) -> None:
        self.tokenizer = Tokenizer.from_file(str(tokenizer_path))
        self.records = list(records)
        self.examples = [self._encode(record) for record in self.records]
        if not self.examples:
            raise ValueError("grounding store is empty")

    def _encode(self, record: dict[str, str]) -> EncodedGroundingExample:
        prompt_ids = self.tokenizer.encode(
            record["prompt"], add_special_tokens=False
        ).ids
        answer_ids = self.tokenizer.encode(
            record["answer"], add_special_tokens=False
        ).ids
        full = [BOS_TOKEN_ID, *prompt_ids, *answer_ids, EOS_TOKEN_ID]
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        if len(full) - 1 > context:
            raise ValueError(
                f"grounding record {record['record_cid']} exceeds {context} input tokens"
            )
        answer_start = 1 + len(prompt_ids)
        inputs = full[:-1]
        targets = full[1:]
        first_answer_target = answer_start - 1
        targets[:first_answer_target] = [-100] * first_answer_target
        supervised_tokens = len(answer_ids) + 1
        padding = context - len(inputs)
        inputs.extend([EOS_TOKEN_ID] * padding)
        targets.extend([-100] * padding)
        if sum(target != -100 for target in targets) != supervised_tokens:
            raise RuntimeError("grounding answer-mask arithmetic differs")
        return EncodedGroundingExample(
            inputs=torch.tensor(inputs, dtype=torch.long),
            targets=torch.tensor(targets, dtype=torch.long),
            supervised_tokens=supervised_tokens,
        )

    def deterministic_batch(
        self, *, seed: int, batch_index: int, batch_size: int
    ) -> tuple[Tensor, Tensor]:
        selected: list[EncodedGroundingExample] = []
        for lane in range(batch_size):
            material = struct.pack(">QQQ", seed, batch_index, lane)
            index = int.from_bytes(blake3(material).digest(), "big") % len(self.examples)
            selected.append(self.examples[index])
        return (
            torch.stack([example.inputs for example in selected]),
            torch.stack([example.targets for example in selected]),
        )

    def sequential_batches(self, batch_size: int) -> Iterator[tuple[Tensor, Tensor]]:
        for base in range(0, len(self.examples), batch_size):
            selected = self.examples[base : base + batch_size]
            yield (
                torch.stack([example.inputs for example in selected]),
                torch.stack([example.targets for example in selected]),
            )

    def parity_prefix(self, record_index: int = 0, count: int = 32) -> list[int]:
        prompt_ids = self.tokenizer.encode(
            self.records[record_index]["prompt"], add_special_tokens=False
        ).ids
        prefix = [BOS_TOKEN_ID, *prompt_ids]
        if len(prefix) < count:
            raise ValueError("grounding prompt cannot supply the parity prefix")
        return prefix[:count]
