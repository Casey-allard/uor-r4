"""Frozen C1-SB4 joint-candidate preflight data and sealed products.

The population deliberately reuses the nine C1-SB3 structural motifs while
starting from the first lexical-world ordinal after every C1-SB3 partition and
product.  Its prompt differs materially: every distinct exact-text candidate
group is scored while the model sees the complete source candidate set.

This module contains data, labels, commitments, and zero-training census
evidence only.  Tokenization is intentionally left to the later campaign code,
which must bind the pinned checkpoint tokenizer before any optimization.
"""

from __future__ import annotations

from collections import Counter, defaultdict
from typing import Any, Iterable, Sequence

from .provenance import canonical_json_bytes, cid_bytes
from .source_relation_adapter_data import (
    NO_TOKEN_ID,
    NO_TOKEN_TEXT,
    OUTCOMES,
    SOURCE_WIDTHS,
    YES_TOKEN_ID,
    YES_TOKEN_TEXT,
    LexicalWorld,
    _canonical_with_cid,
    _polarity_by_locative,
    _position_labels,
    _query_outcomes,
    _world,
    _world_inventory,
    _world_partition,
    _world_records,
    build_source_relation_adapter_population,
)
from .source_relation_data import parse_subject, split_sentence_spans


ISSUE = 954
POLICY = "R4JointCandidateMarginAdapterV1"
JOINT_RECORD_SCHEMA = "uor-r4.joint-candidate-margin-record/1"
JOINT_DATASET_SCHEMA = "uor-r4.joint-candidate-margin-dataset/1"
JOINT_PREFLIGHT_SCHEMA = "uor-r4.joint-candidate-margin-preflight/1"
JOINT_PRODUCT_SCHEMA = "uor-r4.joint-candidate-margin-products/1"
JOINT_CENSUS_SCHEMA = "uor-r4.joint-candidate-margin-census/1"
JOINT_SPLIT_SCHEMA = "uor-r4.joint-candidate-margin-split/1"
JOINT_INPUT_POLICY = (
    "exact UTF-8 `E:<exact full source>\\nQ:<question>\\nC:<exact distinct "
    "group text>\\nSupported:` with no terminal newline; duplicate exact-text "
    "spans share one group prompt; score the fixed next-token yes/no verbalizer "
    "at the final colon"
)
QUESTION_POLICY = "Where is the <subject>?"
SENTENCE_POLICY = "exact .!? terminated UTF-8 byte spans"
FRESH_WORLD_ORDINAL_START = 137
PREFLIGHT_FIT_WORLDS_PER_WIDTH = 2
PREFLIGHT_SEALED_WORLDS_PER_WIDTH = 1
MOTIFS_PER_OUTCOME = 3


def render_joint_candidate_input(
    source: str, question: str, group_text: str
) -> str:
    """Render one exact full-source, candidate-conditioned scoring prefix."""
    if not source or source != source.strip():
        raise ValueError("joint-candidate source must be nonempty and trimmed")
    spans = split_sentence_spans(source)
    if not spans or " ".join(str(span["text"]) for span in spans) != source:
        raise ValueError("joint-candidate source must be exact terminated spans")
    if (
        not group_text
        or group_text != group_text.strip()
        or group_text[-1] not in ".!?"
    ):
        raise ValueError("joint-candidate group text must be one trimmed span")
    if group_text not in {str(span["text"]) for span in spans}:
        raise ValueError("joint-candidate group text is not an exact source span")
    parse_subject(question)
    return f"E:{source}\nQ:{question}\nC:{group_text}\nSupported:"


def _restamp_record(
    record: dict[str, Any],
    *,
    population: str,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Bind an existing structural motif to the independently versioned policy."""
    value = dict(record)
    value.pop("record_cid", None)
    value["schema"] = JOINT_RECORD_SCHEMA
    value["policy"] = POLICY
    value["population"] = population
    source = str(value["source"])
    question = str(value["question"])
    sentence_spans: list[dict[str, Any]] = []
    for original_span in value["sentence_spans"]:
        span = dict(original_span)
        relation_input = render_joint_candidate_input(
            source, question, str(span["text"])
        )
        span["relation_input"] = relation_input
        span["relation_input_cid"] = cid_bytes(relation_input.encode("utf-8"))
        sentence_spans.append(span)
    value["sentence_spans"] = sentence_spans
    if extra:
        overlap = set(value).intersection(extra)
        if overlap:
            raise ValueError(f"joint record extra fields collide: {sorted(overlap)}")
        value.update(extra)
    return _canonical_with_cid(value, "record_cid")


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
            partition="c1-sb4-product",
            width=3,
            lane=lane,
            ordinal=ordinal_start + lane,
        )
        worlds.append(world)
        candidates = _world_records(world, population="product")
        selected = candidates[motif_index]
        if selected["target_outcome"] != outcome:
            raise RuntimeError("joint product motif outcome drifted")
        records.append(
            _restamp_record(selected, population="product", extra={"probe": probe})
        )
    value = {
        "schema": JOINT_PRODUCT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "access_policy": (
            "write and bind this envelope before optimization; the trainer receives "
            "only product_probes_cid and four record commitments and must not open "
            "record text until every pre-product gate passes"
        ),
        "records": records,
    }
    return worlds, _canonical_with_cid(value, "product_probes_cid")


def _sentence_inventory(records: Iterable[dict[str, Any]]) -> set[str]:
    return {
        str(span["text"])
        for record in records
        for span in record["sentence_spans"]
    }


def _worlds_from_records(records: Sequence[dict[str, Any]]) -> list[LexicalWorld]:
    worlds: dict[int, LexicalWorld] = {}
    for record in records:
        ordinal = int(record["world_ordinal"])
        reconstructed = _world(
            partition="c1-sb3-reference",
            width=int(record["source_width"]),
            lane=int(record["world_lane"]),
            ordinal=ordinal,
        )
        previous = worlds.setdefault(ordinal, reconstructed)
        if (
            previous.width != reconstructed.width
            or previous.lane != reconstructed.lane
        ):
            raise RuntimeError("one lexical ordinal described incompatible worlds")
    return list(worlds.values())


def _group_rows(record: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for span in record["sentence_spans"]:
        groups[str(span["relation_group_cid"])].append(span)
    return dict(groups)


def _group_contract(records: Sequence[dict[str, Any]]) -> dict[str, Any]:
    prompt_cids: list[str] = []
    distinct_groups = 0
    positive_groups = 0
    negative_groups = 0
    duplicate_records = 0
    exact = True
    for record in records:
        groups = _group_rows(record)
        distinct_groups += len(groups)
        if len(groups) < len(record["sentence_spans"]):
            duplicate_records += 1
        observed_positive: set[str] = set()
        for group_cid, rows in groups.items():
            texts = {str(row["text"]) for row in rows}
            labels = {int(row["relation_label"]) for row in rows}
            inputs = {str(row["relation_input"]) for row in rows}
            input_cids = {str(row["relation_input_cid"]) for row in rows}
            if len(texts) != 1 or len(labels) != 1 or len(inputs) != 1:
                exact = False
                continue
            text = next(iter(texts))
            relation_input = render_joint_candidate_input(
                str(record["source"]), str(record["question"]), text
            )
            expected_input_cid = cid_bytes(relation_input.encode("utf-8"))
            if (
                group_cid != cid_bytes(text.encode("utf-8"))
                or inputs != {relation_input}
                or input_cids != {expected_input_cid}
            ):
                exact = False
            prompt_cids.append(expected_input_cid)
            label = next(iter(labels))
            if label == 1:
                positive_groups += 1
                observed_positive.add(group_cid)
            elif label == 0:
                negative_groups += 1
            else:
                exact = False
        declared_positive = {
            str(value) for value in record["positive_relation_group_cids"]
        }
        derived_outcome = (
            "abstain"
            if not observed_positive
            else "answer"
            if len(observed_positive) == 1
            else "conflict"
        )
        if (
            observed_positive != declared_positive
            or derived_outcome != record["target_outcome"]
        ):
            exact = False
        if record["duplicate_agreement"]:
            positive_rows = [
                row
                for row in record["sentence_spans"]
                if int(row["relation_label"]) == 1
            ]
            if (
                len(positive_rows) != 2
                or len({str(row["relation_group_cid"]) for row in positive_rows}) != 1
                or record["target_span_index"]
                != min(int(row["candidate_index"]) for row in positive_rows)
            ):
                exact = False
    return {
        "distinct_groups": distinct_groups,
        "positive_groups": positive_groups,
        "negative_groups": negative_groups,
        "records_with_duplicate_spans": duplicate_records,
        "group_prompts_and_labels_exact": exact,
        "distinct_group_prompt_cids_unique": len(prompt_cids) == len(set(prompt_cids)),
    }


def _partition_contract(records: Sequence[dict[str, Any]]) -> dict[str, Any]:
    cells = Counter(
        (int(record["source_width"]), str(record["target_outcome"]))
        for record in records
    )
    complete_cells = set(cells) == {
        (width, outcome) for width in SOURCE_WIDTHS for outcome in OUTCOMES
    }
    balanced = complete_cells and all(
        cells[(width, "answer")]
        == cells[(width, "abstain")]
        == cells[(width, "conflict")]
        for width in SOURCE_WIDTHS
    )
    locative_polarities = _polarity_by_locative(list(records))
    query_outcomes = _query_outcomes(list(records))
    position_labels = _position_labels(list(records))
    group_contract = _group_contract(records)
    return {
        "records": len(records),
        "width_outcome_cell_counts": {
            str(width): {
                outcome: cells[(width, outcome)] for outcome in OUTCOMES
            }
            for width in SOURCE_WIDTHS
        },
        "complete_width_2_through_8_cells": complete_cells,
        "balanced_three_outcomes_per_width": balanced,
        "every_locative_text_has_both_labels": bool(locative_polarities)
        and all(labels == {0, 1} for labels in locative_polarities.values()),
        "every_query_subject_has_answer_and_nonanswer": bool(query_outcomes)
        and all(values == {"answer", "nonanswer"} for values in query_outcomes.values()),
        "every_candidate_position_has_both_labels": bool(position_labels)
        and all(labels == {0, 1} for labels in position_labels.values()),
        **group_contract,
    }


def _population_census(
    *,
    fit_worlds: Sequence[LexicalWorld],
    fit: Sequence[dict[str, Any]],
    sealed_worlds: Sequence[LexicalWorld],
    sealed: Sequence[dict[str, Any]],
    product_worlds: Sequence[LexicalWorld],
    products: Sequence[dict[str, Any]],
) -> dict[str, Any]:
    sb3_dataset, sb3_preflight, sb3_products = (
        build_source_relation_adapter_population()
    )
    new_records = {
        "preflight-fit": fit,
        "preflight-sealed": sealed,
        "product": products,
    }
    sb3_records = {
        "sb3-preflight-fit": sb3_preflight["fit"],
        "sb3-preflight-sealed": sb3_preflight["sealed"],
        "sb3-construction": sb3_dataset["construction"],
        "sb3-development": sb3_dataset["development"],
        "sb3-development-reversal-controls": sb3_dataset["development_controls"][
            "reversal"
        ],
        "sb3-development-query-swap-controls": sb3_dataset[
            "development_controls"
        ]["query_swap"],
        "sb3-product": sb3_products["records"],
    }
    new_sentence_sets = {
        name: _sentence_inventory(records) for name, records in new_records.items()
    }
    sb3_sentence_sets = {
        name: _sentence_inventory(records) for name, records in sb3_records.items()
    }
    new_names = sorted(new_sentence_sets)
    new_sentences_pairwise_disjoint = all(
        new_sentence_sets[left].isdisjoint(new_sentence_sets[right])
        for left_index, left in enumerate(new_names)
        for right in new_names[left_index + 1 :]
    )
    new_vs_sb3_sentences = {
        new_name: {
            old_name: new_sentence_sets[new_name].isdisjoint(old_sentences)
            for old_name, old_sentences in sorted(sb3_sentence_sets.items())
        }
        for new_name in new_names
    }

    new_worlds = {
        "preflight-fit": list(fit_worlds),
        "preflight-sealed": list(sealed_worlds),
        "product": list(product_worlds),
    }
    sb3_primary_records = {
        "sb3-preflight-fit": sb3_preflight["fit"],
        "sb3-preflight-sealed": sb3_preflight["sealed"],
        "sb3-construction": sb3_dataset["construction"],
        "sb3-development": sb3_dataset["development"],
        "sb3-product": sb3_products["records"],
    }
    new_lexical_sets = {
        name: _world_inventory(worlds) for name, worlds in new_worlds.items()
    }
    sb3_lexical_sets = {
        name: _world_inventory(_worlds_from_records(list(records)))
        for name, records in sb3_primary_records.items()
    }
    sb3_lexical_sets["sb3-development-reversal-controls"] = set(
        sb3_lexical_sets["sb3-development"]
    )
    sb3_lexical_sets["sb3-development-query-swap-controls"] = set(
        sb3_lexical_sets["sb3-development"]
    )
    new_composite_world_item_banks_pairwise_disjoint = all(
        new_lexical_sets[left].isdisjoint(new_lexical_sets[right])
        for left_index, left in enumerate(new_names)
        for right in new_names[left_index + 1 :]
    )
    new_vs_sb3_lexical_banks = {
        new_name: {
            old_name: new_lexical_sets[new_name].isdisjoint(old_lexemes)
            for old_name, old_lexemes in sorted(sb3_lexical_sets.items())
        }
        for new_name in new_names
    }

    checks = {
        "preflight-fit": _partition_contract(fit),
        "preflight-sealed": _partition_contract(sealed),
    }
    product_group_contract = _group_contract(products)
    old_ordinals = {
        int(record["world_ordinal"])
        for records in sb3_primary_records.values()
        for record in records
    }
    new_ordinals = {
        world.ordinal for worlds in new_worlds.values() for world in worlds
    }
    ordinal_boundary = {
        "sb3_max_world_ordinal": max(old_ordinals),
        "sb4_min_world_ordinal": min(new_ordinals),
        "sb4_starts_exactly_after_sb3": min(new_ordinals) == max(old_ordinals) + 1,
        "sb4_ordinals_contiguous": new_ordinals
        == set(range(FRESH_WORLD_ORDINAL_START, FRESH_WORLD_ORDINAL_START + 25)),
    }
    all_partition_checks_pass = all(
        all(
            bool(partition[field])
            for field in (
                "complete_width_2_through_8_cells",
                "balanced_three_outcomes_per_width",
                "every_locative_text_has_both_labels",
                "every_query_subject_has_answer_and_nonanswer",
                "every_candidate_position_has_both_labels",
                "group_prompts_and_labels_exact",
                "distinct_group_prompt_cids_unique",
            )
        )
        for partition in checks.values()
    )
    passed = (
        new_sentences_pairwise_disjoint
        and new_composite_world_item_banks_pairwise_disjoint
        and all(
            passed
            for comparisons in new_vs_sb3_sentences.values()
            for passed in comparisons.values()
        )
        and all(
            passed
            for comparisons in new_vs_sb3_lexical_banks.values()
            for passed in comparisons.values()
        )
        and all_partition_checks_pass
        and bool(product_group_contract["group_prompts_and_labels_exact"])
        and bool(product_group_contract["distinct_group_prompt_cids_unique"])
        and bool(ordinal_boundary["sb4_starts_exactly_after_sb3"])
        and bool(ordinal_boundary["sb4_ordinals_contiguous"])
    )
    value = {
        "schema": JOINT_CENSUS_SCHEMA,
        "policy": POLICY,
        "fresh_world_ordinal_start": FRESH_WORLD_ORDINAL_START,
        "ordinal_boundary": ordinal_boundary,
        "new_sentence_partitions_pairwise_disjoint": new_sentences_pairwise_disjoint,
        "new_vs_every_sb3_partition_sentences_disjoint": new_vs_sb3_sentences,
        "composite_world_item_definition": (
            "complete generated subject phrases, complete generated location phrases, "
            "and complete generated nonlocative phrases"
        ),
        "primitive_component_vocabulary": (
            "DELIBERATELY_SHARED_ACROSS_SB3_AND_SB4; not a disjointness claim"
        ),
        "new_composite_world_item_banks_pairwise_disjoint": (
            new_composite_world_item_banks_pairwise_disjoint
        ),
        "new_vs_every_sb3_partition_composite_world_item_bank_disjoint": (
            new_vs_sb3_lexical_banks
        ),
        "sb3_controls_composite_world_item_coverage": (
            "development reversal and query-swap controls reuse the bound SB3 "
            "development worlds; their sentence inventories are checked separately"
        ),
        "partition_checks": checks,
        "product_group_check": product_group_contract,
        "tokenizer_census": {
            "status": "CAMPAIGN_BOUND_NOT_RUN",
            "exact_inputs": (
                "every distinct relation_input string and CID is committed in the "
                "preflight or separately sealed product record"
            ),
            "required_next_action": (
                "before optimization, bind the pinned #1017 tokenizer, include BOS, "
                "reject any prompt over the frozen context budget, and never truncate"
            ),
        },
        "sb3_reference_cids": {
            "dataset_cid": sb3_dataset["dataset_cid"],
            "preflight_cid": sb3_preflight["preflight_cid"],
            "product_probes_cid": sb3_products["product_probes_cid"],
        },
        "passed": passed,
    }
    if not passed:
        raise RuntimeError(f"C1-SB4 zero-training census failed: {value}")
    return _canonical_with_cid(value, "census_cid")


def build_joint_candidate_margin_population(
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Build the frozen C1-SB4 dataset, preflight, and unopened products."""
    ordinal = FRESH_WORLD_ORDINAL_START
    fit_worlds, raw_fit, ordinal = _world_partition(
        partition="c1-sb4-preflight-fit",
        worlds_per_width=PREFLIGHT_FIT_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    sealed_worlds, raw_sealed, ordinal = _world_partition(
        partition="c1-sb4-preflight-sealed",
        worlds_per_width=PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
        ordinal_start=ordinal,
    )
    fit = [
        _restamp_record(record, population="preflight-fit") for record in raw_fit
    ]
    sealed = [
        _restamp_record(record, population="preflight-sealed")
        for record in raw_sealed
    ]
    product_worlds, products = _product_population(ordinal_start=ordinal)
    ordinal += len(products["records"])
    if ordinal != FRESH_WORLD_ORDINAL_START + 25:
        raise RuntimeError("C1-SB4 lexical-world ordinal allocation drifted")

    census = _population_census(
        fit_worlds=fit_worlds,
        fit=fit,
        sealed_worlds=sealed_worlds,
        sealed=sealed,
        product_worlds=product_worlds,
        products=list(products["records"]),
    )
    counts = {
        "preflight_fit": len(fit),
        "preflight_sealed": len(sealed),
        "product_probe_commitments": len(products["records"]),
        "preflight_fit_distinct_groups": int(
            census["partition_checks"]["preflight-fit"]["distinct_groups"]
        ),
        "preflight_sealed_distinct_groups": int(
            census["partition_checks"]["preflight-sealed"]["distinct_groups"]
        ),
    }
    expected_counts = {
        "preflight_fit": 126,
        "preflight_sealed": 63,
        "product_probe_commitments": 4,
        "preflight_fit_distinct_groups": 604,
        "preflight_sealed_distinct_groups": 302,
    }
    if counts != expected_counts:
        raise RuntimeError(f"C1-SB4 population count drifted: {counts}")

    preflight_value = {
        "schema": JOINT_PREFLIGHT_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "selection": (
            "two fresh fit lexical worlds and one independently sealed fresh lexical "
            "world for each source width 2..8; every world has the same nine matched "
            "structural motifs as C1-SB3, with no identical composite subject, "
            "location, or nonlocative item and no identical exact sentence; primitive "
            "component vocabulary is deliberately shared"
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
            "schema": JOINT_SPLIT_SCHEMA,
            "policy": POLICY,
            "selection": (
                "start at lexical-world ordinal 137, enumerate widths 2..8 and fixed "
                "lanes without shuffling, then reserve four later product worlds"
            ),
            "source_widths": list(SOURCE_WIDTHS),
            "motifs_per_outcome": MOTIFS_PER_OUTCOME,
            "worlds_per_width": {
                "preflight_fit": PREFLIGHT_FIT_WORLDS_PER_WIDTH,
                "preflight_sealed": PREFLIGHT_SEALED_WORLDS_PER_WIDTH,
            },
            "fresh_world_ordinal_start": FRESH_WORLD_ORDINAL_START,
            "product_policy": (
                "four product records are separately committed and unopened by training"
            ),
        },
        "split_policy_cid",
    )
    dataset_value = {
        "schema": JOINT_DATASET_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "relation_input_policy": JOINT_INPUT_POLICY,
        "fixed_verbalizer": {
            "positive_token_id": YES_TOKEN_ID,
            "positive_token_text": YES_TOKEN_TEXT,
            "negative_token_id": NO_TOKEN_ID,
            "negative_token_text": NO_TOKEN_TEXT,
            "decision": "positive iff yes_logit - no_logit > 0; zero is negative",
        },
        "counts": counts,
        "split_policy": split_policy,
        "split_policy_cid": split_policy["split_policy_cid"],
        "census": census,
        "census_cid": census["census_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": [
            record["record_cid"] for record in products["records"]
        ],
    }
    dataset = _canonical_with_cid(dataset_value, "dataset_cid")
    return dataset, preflight, products


__all__ = [
    "FRESH_WORLD_ORDINAL_START",
    "ISSUE",
    "JOINT_CENSUS_SCHEMA",
    "JOINT_DATASET_SCHEMA",
    "JOINT_INPUT_POLICY",
    "JOINT_PREFLIGHT_SCHEMA",
    "JOINT_PRODUCT_SCHEMA",
    "JOINT_RECORD_SCHEMA",
    "JOINT_SPLIT_SCHEMA",
    "POLICY",
    "build_joint_candidate_margin_population",
    "render_joint_candidate_input",
]
