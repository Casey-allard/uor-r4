"""Deterministic intact-fact rotation and a ledger derived from the source clock."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any

import torch
from torch import Tensor

_SUPPORTED_UPDATES = 2352
_TOTAL_UPDATES = 3920
_BATCH_SIZE = 512
_SUPPORTED_BATCHES = 16
_MIXED_BATCHES = 20
_UNKNOWN_PER_MIXED_TRAVERSAL = 2048

AUGMENTATION = {
    "policy": "CyclicFactOrderPerTraversalV1",
    "sequence_length": 41,
    "readout_position": 37,
    "fact_start": 1,
    "fact_count": 4,
    "fact_width": 8,
    "direction": "right cyclic shift of intact fact blocks",
    "offsets": [0, 1, 2, 3],
    "offset_rule": "floor(phase_completed_updates / batches_per_traversal) % 4",
    "phase_local_offset_reset": True,
    "supported_updates": _SUPPORTED_UPDATES,
    "total_updates": _TOTAL_UPDATES,
    "batch_size": _BATCH_SIZE,
    "supported_batches_per_traversal": _SUPPORTED_BATCHES,
    "mixed_batches_per_traversal": _MIXED_BATCHES,
    "additional_random_draws": 0,
}


def _validate_updates(completed_updates: int, *, allow_complete: bool) -> None:
    maximum = _TOTAL_UPDATES if allow_complete else _TOTAL_UPDATES - 1
    if (
        isinstance(completed_updates, bool)
        or not isinstance(completed_updates, int)
        or not 0 <= completed_updates <= maximum
    ):
        raise ValueError("completed updates are outside the frozen training clock")


def rotation_offset(completed_updates: int) -> int:
    """Choose the next batch's phase-local rotation from prior completed updates."""
    _validate_updates(completed_updates, allow_complete=False)
    if completed_updates < _SUPPORTED_UPDATES:
        return (completed_updates // _SUPPORTED_BATCHES) % 4
    return ((completed_updates - _SUPPORTED_UPDATES) // _MIXED_BATCHES) % 4


def rotate_inputs(inputs: Tensor, offset: int) -> Tensor:
    """Rotate only the four fact blocks; clone even at zero offset for matched work."""
    if (
        not isinstance(inputs, Tensor)
        or inputs.device.type != "cpu"
        or inputs.dtype != torch.long
        or inputs.ndim != 2
        or inputs.shape[0] == 0
        or inputs.shape[1] != 41
    ):
        raise ValueError("fact rotation requires nonempty CPU int64 inputs [rows,41]")
    if isinstance(offset, bool) or not isinstance(offset, int) or not 0 <= offset < 4:
        raise ValueError("fact rotation offset must be an integer from zero to three")
    result = inputs.clone()
    facts = inputs[:, 1:33].reshape(inputs.shape[0], 4, 8)
    result[:, 1:33] = torch.roll(facts, shifts=offset, dims=1).reshape(
        inputs.shape[0], 32
    )
    return result


def augment_training_batch(
    batch: Sequence[Tensor], *, completed_updates: int
) -> tuple[Tensor, Tensor, Tensor]:
    """Keep source sampler order, selected positions and targets unchanged."""
    if len(batch) != 3:
        raise ValueError(
            "fact rotation requires inputs, selected positions and targets"
        )
    inputs, positions, targets = batch
    return rotate_inputs(inputs, rotation_offset(completed_updates)), positions, targets


def _phase_ledger(
    updates: int, batches: int, unknown_per_traversal: int, measured_unknown: int
) -> dict[str, Any]:
    complete, partial = divmod(updates, batches)
    tail_unknown = measured_unknown - complete * unknown_per_traversal
    tail_presentations = partial * _BATCH_SIZE
    known_per_traversal = batches * _BATCH_SIZE - unknown_per_traversal
    if not (
        max(0, tail_presentations - known_per_traversal)
        <= tail_unknown
        <= min(unknown_per_traversal, tail_presentations)
    ):
        raise ValueError(
            "measured UNKNOWN count disagrees with complete/partial traversals"
        )
    by_offset = []
    for offset in range(4):
        traversals = (complete + 3 - offset) // 4
        steps = traversals * batches + (partial if offset == complete % 4 else 0)
        unknown = traversals * unknown_per_traversal + (
            tail_unknown if offset == complete % 4 else 0
        )
        by_offset.append(
            {
                "offset": offset,
                "optimizer_updates": steps,
                "presentations": steps * _BATCH_SIZE,
                "supported_presentations": steps * _BATCH_SIZE - unknown,
                "unknown_presentations": unknown,
            }
        )
    return {
        "completed_updates": updates,
        "complete_traversals": complete,
        "partial_traversal": {
            "offset": complete % 4 if partial else None,
            "updates": partial,
            "presentations": tail_presentations,
            "unknown_presentations": tail_unknown,
        },
        "by_offset": by_offset,
    }


def rotation_ledger(
    completed_updates: int, unknown_presentations: int
) -> dict[str, Any]:
    """Derive every offset count from existing checkpoint counters, including its tail."""
    _validate_updates(completed_updates, allow_complete=True)
    if (
        isinstance(unknown_presentations, bool)
        or not isinstance(unknown_presentations, int)
        or unknown_presentations < 0
    ):
        raise ValueError("measured UNKNOWN presentations must be a nonnegative integer")
    supported = _phase_ledger(
        min(completed_updates, _SUPPORTED_UPDATES), _SUPPORTED_BATCHES, 0, 0
    )
    mixed = _phase_ledger(
        max(0, completed_updates - _SUPPORTED_UPDATES),
        _MIXED_BATCHES,
        _UNKNOWN_PER_MIXED_TRAVERSAL,
        unknown_presentations,
    )
    totals = [
        {
            "offset": offset,
            **{
                key: supported["by_offset"][offset][key]
                + mixed["by_offset"][offset][key]
                for key in (
                    "optimizer_updates",
                    "presentations",
                    "supported_presentations",
                    "unknown_presentations",
                )
            },
        }
        for offset in range(4)
    ]
    return {
        "policy": dict(AUGMENTATION),
        "completed_updates": completed_updates,
        "supported_phase": supported,
        "mixed_phase": mixed,
        "totals_by_offset": totals,
        "train_query_presentations": completed_updates * _BATCH_SIZE,
        "supported_presentations": completed_updates * _BATCH_SIZE
        - unknown_presentations,
        "unknown_presentations": unknown_presentations,
    }
