"""Descriptive construction-only error analysis of frozen English binding logits."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

import torch
from torch import Tensor

from ..zoology_english_binding.data import (
    LOCATIONS,
    UNKNOWN_ID,
    VOCABULARY,
    parse_row,
)

CATEGORIES = (
    "target",
    "same_owner_confound",
    "same_object_confound",
    "unrelated_fact_location",
    "out_of_history_location",
    "unknown",
    "other_vocabulary",
)
_FACT_CATEGORIES = CATEGORIES[:4]
_PAIR_TYPES = ("same_owner", "same_object")
_TOKEN_IDS = {word: index for index, word in enumerate(VOCABULARY)}
_BLOCK_ROWS = 256


@dataclass(frozen=True)
class ClassifiedRow:
    """Input-derived factual opportunities and the model's selected category."""

    facts: tuple[tuple[str, str, str], ...]
    query: tuple[str, str]
    target: int
    prediction: int
    category: str
    target_slot: int
    selected_slot: int | None
    eligible_facts: tuple[int, ...]


def _fact_category(owner: str, obj: str, query: tuple[str, str]) -> str:
    if (owner, obj) == query:
        return "target"
    if owner == query[0]:
        return "same_owner_confound"
    if obj == query[1]:
        return "same_object_confound"
    return "unrelated_fact_location"


def classify_row(
    input_ids: Sequence[int] | Tensor, target: int, prediction: int
) -> ClassifiedRow:
    """Classify one supported answer using decoded facts, never group labels."""
    if any(
        isinstance(value, bool)
        or not isinstance(value, int)
        or not 0 <= value < len(VOCABULARY)
        for value in (target, prediction)
    ):
        raise ValueError("target and prediction must be vocabulary IDs")
    facts, query, answer = parse_row(input_ids)
    if answer == "unknown" or _TOKEN_IDS[answer] != target:
        raise ValueError("supported target disagrees with the parsed input")
    if len({fact[2] for fact in facts}) != 4:
        raise ValueError("displayed locations must be distinct")
    factual_categories = tuple(
        _fact_category(owner, obj, query) for owner, obj, _ in facts
    )
    target_slot = factual_categories.index("target")
    word = VOCABULARY[prediction]
    selected_slot = next(
        (slot for slot, fact in enumerate(facts) if fact[2] == word), None
    )
    if selected_slot is not None:
        category = factual_categories[selected_slot]
    elif word in LOCATIONS:
        category = "out_of_history_location"
    elif prediction == UNKNOWN_ID:
        category = "unknown"
    else:
        category = "other_vocabulary"
    return ClassifiedRow(
        facts,
        query,
        target,
        prediction,
        category,
        target_slot,
        selected_slot,
        tuple(factual_categories.count(name) for name in _FACT_CATEGORIES),
    )


def _rate(count: int, denominator: int) -> float | None:
    return count / denominator if denominator else None


def _distribution(values: Tensor) -> dict[str, Any]:
    values = values.reshape(-1).to(dtype=torch.float64)
    count = values.numel()
    if not count:
        return {
            "count": 0,
            "mean": None,
            "median": None,
            "min": None,
            "max": None,
            "positive": 0,
            "zero": 0,
            "negative": 0,
        }
    if not bool(torch.isfinite(values).all()):
        raise ValueError("diagnostic distribution contains non-finite values")
    ordered = values.sort().values
    middle = count // 2
    median = (
        float(ordered[middle])
        if count % 2
        else (float(ordered[middle - 1]) + float(ordered[middle])) / 2
    )
    return {
        "count": count,
        "mean": float(values.sum()) / count,
        "median": median,
        "min": float(ordered[0]),
        "max": float(ordered[-1]),
        "positive": int(torch.count_nonzero(values > 0)),
        "zero": int(torch.count_nonzero(values == 0)),
        "negative": int(torch.count_nonzero(values < 0)),
    }


def _target_margins(logits: Tensor, targets: Tensor) -> Tensor:
    margins = torch.empty(logits.shape[0], dtype=torch.float64)
    for start in range(0, logits.shape[0], _BLOCK_ROWS):
        end = min(start + _BLOCK_ROWS, logits.shape[0])
        block = logits[start:end].to(dtype=torch.float64)
        chosen = targets[start:end].reshape(-1)
        best, indices = block.topk(2, dim=1)
        other = torch.where(indices[:, 0] == chosen, best[:, 1], best[:, 0])
        margins[start:end] = block.gather(1, chosen[:, None]).squeeze(1) - other
    return margins


def summarize_rows(
    rows: Sequence[ClassifiedRow], margins: Tensor | None = None
) -> dict[str, Any]:
    """Summarize actual eligibility; absent q1 confounds have null rates."""
    count = len(rows)
    in_history = sum(row.selected_slot is not None for row in rows)
    categories: dict[str, Any] = {}
    for index, name in enumerate(CATEGORIES):
        selected = sum(row.category == name for row in rows)
        eligible_rows = (
            sum(row.eligible_facts[index] > 0 for row in rows) if index < 4 else count
        )
        eligible_facts = (
            sum(row.eligible_facts[index] for row in rows) if index < 4 else None
        )
        categories[name] = {
            "count": selected,
            "eligible_rows": eligible_rows,
            "eligible_facts": eligible_facts,
            "rate_per_eligible_row": _rate(selected, eligible_rows),
            "rate_per_eligible_fact": (
                _rate(selected, eligible_facts) if eligible_facts is not None else None
            ),
        }
    slots: dict[str, Any] = {}
    for slot in range(4):
        exposure = sum(row.target_slot == slot for row in rows)
        selections = sum(row.selected_slot == slot for row in rows)
        correct = sum(
            row.target_slot == slot and row.category == "target" for row in rows
        )
        selected_rate = _rate(selections, in_history)
        exposure_rate = _rate(exposure, count)
        slots[str(slot)] = {
            "target_exposure": exposure,
            "target_exposure_rate": exposure_rate,
            "selections": selections,
            "selection_rate_among_in_history": selected_rate,
            "correct": correct,
            "accuracy_when_target_at_slot": _rate(correct, exposure),
            "selection_minus_target_exposure_rate": (
                selected_rate - exposure_rate
                if selected_rate is not None and exposure_rate is not None
                else None
            ),
        }
    correct = categories["target"]["count"]
    return {
        "rows": count,
        "correct": correct,
        "accuracy": _rate(correct, count),
        "in_history_predictions": in_history,
        "categories": categories,
        "displayed_slots": slots,
        "target_versus_best_other_margin": (
            _distribution(margins) if margins is not None else None
        ),
    }


def pair_summary(
    logits: Tensor,
    targets: Tensor,
    predictions: Tensor,
    pairs: Tensor,
    *,
    margins: Tensor | None = None,
) -> dict[str, Any]:
    """Compare fixed pairs without choosing the contrast from predictions."""
    if logits.ndim != 2 or logits.shape[1] < 2:
        raise ValueError("pair logits must be a rows by vocabulary matrix")
    targets = targets.reshape(-1)
    predictions = predictions.reshape(-1)
    if (
        targets.shape != predictions.shape
        or targets.shape[0] != logits.shape[0]
        or pairs.ndim != 2
        or pairs.shape[1] != 2
        or pairs.dtype != torch.long
    ):
        raise ValueError("pair population dimensions disagree")
    if pairs.numel() and (int(pairs.min()) < 0 or int(pairs.max()) >= logits.shape[0]):
        raise ValueError("pair index is outside the population")
    if margins is None:
        margins = _target_margins(logits, targets)
    left, right = pairs[:, 0], pairs[:, 1]
    a, b = targets[left], targets[right]
    if bool((a == b).any()):
        raise ValueError("paired target contrast requires distinct answers")
    changed = predictions[left] != predictions[right]
    both = (predictions[left] == a) & (predictions[right] == b)
    contrasts = torch.empty(pairs.shape[0], dtype=torch.float64)
    absolute_sum = 0.0
    absolute_max = 0.0
    for start in range(0, pairs.shape[0], _BLOCK_ROWS):
        end = min(start + _BLOCK_ROWS, pairs.shape[0])
        zl = logits[left[start:end]].to(dtype=torch.float64)
        zr = logits[right[start:end]].to(dtype=torch.float64)
        if not bool(torch.isfinite(zl).all() and torch.isfinite(zr).all()):
            raise ValueError("paired logits contain non-finite values")
        ar, br = a[start:end, None], b[start:end, None]
        contrasts[start:end] = (
            (zl.gather(1, ar) - zl.gather(1, br))
            - (zr.gather(1, ar) - zr.gather(1, br))
        ).squeeze(1)
        absolute = (zl - zr).abs()
        absolute_sum += float(absolute.sum())
        absolute_max = max(absolute_max, float(absolute.max()))
    count = pairs.shape[0]
    scalar_count = count * logits.shape[1]
    changed_count = int(torch.count_nonzero(changed))
    both_count = int(torch.count_nonzero(both))
    return {
        "pairs": count,
        "changed": changed_count,
        "invariant": count - changed_count,
        "both_correct": both_count,
        "changed_but_not_both_correct": int(torch.count_nonzero(changed & ~both)),
        "changed_rate": _rate(changed_count, count),
        "invariant_rate": _rate(count - changed_count, count),
        "both_correct_rate": _rate(both_count, count),
        "target_contrast_delta": _distribution(contrasts),
        "full_vocabulary_absolute_logit_difference": {
            "scalars": scalar_count,
            "max": absolute_max if scalar_count else None,
            "mean": absolute_sum / scalar_count if scalar_count else None,
        },
        "left_target_versus_best_other_margin": _distribution(margins[left]),
        "right_target_versus_best_other_margin": _distribution(margins[right]),
    }


def _majority(count: int, denominator: int) -> dict[str, Any]:
    if denominator < 0 or count < 0 or count > denominator:
        raise ValueError("invalid majority numerator or denominator")
    return {
        "count": count,
        "denominator": denominator,
        "rate": _rate(count, denominator),
        "flag": count * 2 > denominator if denominator else None,
    }


def choose_focus(
    slot_selections: Sequence[int],
    q0_in_history_errors: dict[str, int],
    question_pairs: int,
    question_invariant: int,
) -> dict[str, Any]:
    """Apply frozen strict-majority ranking; these are not causal claims."""
    if len(slot_selections) != 4 or any(count < 0 for count in slot_selections):
        raise ValueError("four nonnegative displayed-slot counts are required")
    error_counts = [q0_in_history_errors[name] for name in _FACT_CATEGORIES[1:]]
    if any(count < 0 for count in error_counts):
        raise ValueError("q0 error counts must be nonnegative")
    slot = max(range(4), key=lambda index: slot_selections[index])
    flags = {
        "displayed_slot": _majority(slot_selections[slot], sum(slot_selections)),
        "same_owner_confound": _majority(error_counts[0], sum(error_counts)),
        "same_object_confound": _majority(error_counts[1], sum(error_counts)),
        "question_top1_invariance": _majority(question_invariant, question_pairs),
    }
    flags["displayed_slot"]["slot"] = (
        slot if flags["displayed_slot"]["flag"] is True else None
    )
    has_slot = flags["displayed_slot"]["flag"] is True
    has_owner = flags["same_owner_confound"]["flag"] is True
    has_object = flags["same_object_confound"]["flag"] is True
    if has_slot and (has_owner or has_object):
        label = "JOINT_POSITION_ATTRIBUTE"
    elif has_slot:
        label = "POSITION_READOUT"
    elif has_owner:
        label = "OBJECT_DISAMBIGUATION"
    elif has_object:
        label = "OWNER_DISAMBIGUATION"
    elif flags["question_top1_invariance"]["flag"] is True:
        label = "QUESTION_READOUT"
    else:
        label = "DISTRIBUTED_BINDING"
    return {
        "label": label,
        "majority_flags": flags,
        "scope": "Descriptive ranking of a future diagnostic focus; no capability or causal attribution.",
    }


def _validate_groups(rows: Sequence[ClassifiedRow], pair_types: list[int]) -> None:
    for start in range(0, len(rows), 4):
        r0, r1, r2, r3 = rows[start : start + 4]
        if (
            r0.facts != r1.facts
            or r2.facts != r3.facts
            or r0.query != r2.query
            or r1.query != r3.query
            or r0.query == r1.query
        ):
            raise ValueError(
                "group does not match the declared paired histories/questions"
            )
        same_owner = r0.query[0] == r1.query[0]
        same_object = r0.query[1] == r1.query[1]
        if same_owner == same_object or pair_types[start] != int(same_object):
            raise ValueError("pair type disagrees with decoded questions")
        expected = list(r0.facts)
        left, right = r0.target_slot, r1.target_slot
        expected[left] = (*expected[left][:2], r0.facts[right][2])
        expected[right] = (*expected[right][:2], r0.facts[left][2])
        if tuple(expected) != r2.facts:
            raise ValueError(
                "swapped history changes more than the two answer locations"
            )
        if r0.eligible_facts != (1, 1, 1, 1) or r2.eligible_facts != (1, 1, 1, 1):
            raise ValueError("q0 does not have one of each factual confound")


def analyze(
    inputs: Tensor,
    targets: Tensor,
    predictions: Tensor,
    logits: Tensor,
    group_ids: Tensor,
    variant_ids: Tensor,
    pair_types: Tensor,
) -> dict[str, Any]:
    """Analyze exactly 8,192 canonical supported construction rows, read-only."""
    integer_tensors = {
        "inputs": (inputs, (8192, 41)),
        "targets": (targets, (8192, 1)),
        "predictions": (predictions, (8192, 1)),
        "group_ids": (group_ids, (8192,)),
        "variant_ids": (variant_ids, (8192,)),
        "pair_types": (pair_types, (8192,)),
    }
    for name, (value, shape) in integer_tensors.items():
        if (
            value.shape != shape
            or value.dtype != torch.long
            or value.device.type != "cpu"
        ):
            raise ValueError(f"{name} must have shape {shape} and CPU int64 dtype")
    if (
        logits.shape != (8192, 1, 4096)
        or logits.device.type != "cpu"
        or not logits.is_floating_point()
        or logits.requires_grad
    ):
        raise ValueError("logits must be detached CPU floats with shape (8192,1,4096)")
    if not torch.equal(
        group_ids, torch.arange(2048).repeat_interleave(4)
    ) or not torch.equal(variant_ids, torch.arange(4).repeat(2048)):
        raise ValueError("rows must contain canonical group IDs and variants 0,1,2,3")
    if bool(((pair_types < 0) | (pair_types > 1)).any()) or not bool(
        (pair_types.reshape(2048, 4) == pair_types.reshape(2048, 4)[:, :1]).all()
    ):
        raise ValueError("pair types must be constant within each group and in {0,1}")
    flat_logits = logits[:, 0, :]
    if not bool(torch.isfinite(flat_logits).all()) or not torch.equal(
        flat_logits.argmax(dim=1), predictions[:, 0]
    ):
        raise ValueError("predictions disagree with finite full-vocabulary logits")
    rows = [
        classify_row(input_ids, target, prediction)
        for input_ids, target, prediction in zip(
            inputs.tolist(),
            targets[:, 0].tolist(),
            predictions[:, 0].tolist(),
            strict=True,
        )
    ]
    types = pair_types.tolist()
    _validate_groups(rows, types)
    margins = _target_margins(flat_logits, targets)

    def subset(indices: list[int]) -> dict[str, Any]:
        return summarize_rows([rows[index] for index in indices], margins[indices])

    strata = {
        "all": summarize_rows(rows, margins),
        "pair_type": {
            name: subset([index for index, kind in enumerate(types) if kind == kind_id])
            for kind_id, name in enumerate(_PAIR_TYPES)
        },
        "question": {
            f"q{question}": subset(list(range(question, 8192, 2)))
            for question in range(2)
        },
        "question_by_pair_type": {
            name: {
                f"q{question}": subset(
                    [
                        index
                        for index in range(question, 8192, 2)
                        if types[index] == kind_id
                    ]
                )
                for question in range(2)
            }
            for kind_id, name in enumerate(_PAIR_TYPES)
        },
        "history": {
            name: subset(
                [index for index in range(8192) if (index % 4 >= 2) == swapped]
            )
            for name, swapped in (("base", False), ("swapped", True))
        },
        "target_displayed_slot": {
            str(slot): subset(
                [index for index, row in enumerate(rows) if row.target_slot == slot]
            )
            for slot in range(4)
        },
    }
    q0 = rows[0::2]
    wrong = sum(row.category != "target" for row in q0)
    error_counts = {
        name: sum(row.category == name for row in q0) for name in CATEGORIES[1:]
    }
    in_history_errors = sum(error_counts[name] for name in _FACT_CATEGORIES[1:])
    q0_errors = {
        "rows": len(q0),
        "all_errors": wrong,
        "in_history_errors": in_history_errors,
        "equal_confound_eligible_rows": len(q0),
        "equal_confound_eligible_facts_per_category": len(q0),
        "categories": {
            name: {
                "count": count,
                "rate_among_all_q0_errors": _rate(count, wrong),
                "rate_among_q0_in_history_errors": (
                    _rate(count, in_history_errors)
                    if name in _FACT_CATEGORIES
                    else None
                ),
            }
            for name, count in error_counts.items()
        },
    }
    paired: dict[str, Any] = {}
    for name, offsets in (
        ("question", ((0, 1), (2, 3))),
        ("location_swap", ((0, 2), (1, 3))),
    ):
        pairs = torch.tensor(
            [
                (start + left, start + right)
                for start in range(0, 8192, 4)
                for left, right in offsets
            ],
            dtype=torch.long,
        )

        def summarize_pairs(selected: Tensor) -> dict[str, Any]:
            return pair_summary(
                flat_logits, targets, predictions, selected, margins=margins
            )

        paired[name] = {
            "variant_pairs": [list(pair) for pair in offsets],
            "all": summarize_pairs(pairs),
            "pair_type": {
                label: summarize_pairs(pairs[pair_types[pairs[:, 0]] == index])
                for index, label in enumerate(_PAIR_TYPES)
            },
        }
    slots = [sum(row.selected_slot == slot for row in rows) for slot in range(4)]
    return {
        "schema": "uor-r4.english-binding-construction-diagnostic-analysis/1",
        "rows": 8192,
        "groups": 2048,
        "category_definitions": {
            "target": "fact matches both queried owner and object",
            "same_owner_confound": "fact matches queried owner but not object",
            "same_object_confound": "fact matches queried object but not owner",
            "unrelated_fact_location": "fact matches neither queried owner nor object",
            "out_of_history_location": "one of eight location IDs absent from the four facts",
            "unknown": "declared unknown answer ID",
            "other_vocabulary": "any remaining full-head vocabulary ID",
        },
        "denominator_policy": "Factual category rates use actual eligible rows/facts; absent eligibility has null rates. Non-factual categories have no eligible-fact denominator. Displayed slots are zero-based.",
        "contrast_definition": "a=left target, b=right target; delta=(z_left[a]-z_left[b])-(z_right[a]-z_right[b]); float64 arithmetic, no epsilon",
        "strata": strata,
        "q0_error_breakdown": q0_errors,
        "paired": paired,
        "focus": choose_focus(
            slots,
            error_counts,
            paired["question"]["all"]["pairs"],
            paired["question"]["all"]["invariant"],
        ),
    }
