"""Bounded H4 candidate admission over the fixed recurrent #973 state.

The persistent memory, learned tensors, transported Q/K score, softmax, and
value aggregation are inherited unchanged from the accepted fixed recurrent
path.  This V1 changes only source admission.  It examines at most the eight
live and four summary metadata records, selects at most eight persistent
records, and only then gathers their K/V tensors.  The transient current record
is always appended.

Primary order is the exact signed S3 angular shell of ``source^-1 * current``.
If one shell overfills the remaining budget, greedy maximin separation between
the full exact H4 relative roots chooses a diverse subset.  Causal age and the
physical recurrent slot are deterministic final tie breakers.  The budget and
ranking are an unfitted engineering hypothesis, not a semantic or quality
claim.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import torch
from torch import Tensor

from .fixed_recurrent_kv_binding import (
    LIVE_WINDOW,
    SUMMARY_BANKS,
    FixedRecurrentCandidateSelection,
    FixedRecurrentKVState,
    R4FixedRecurrentCausalKVBindingV1,
)
from .group_retention import GroupAddressArtifact


POLICY = "R4SparseGeometricCandidateSoftmaxKVBindingV1"
PERSISTENT_CANDIDATE_BUDGET = LIVE_WINDOW
MAXIMUM_READ_SOURCES = PERSISTENT_CANDIDATE_BUDGET + 1
SIGNED_S3_SHELL_DEGREES = (0, 36, 60, 72, 90, 108, 120, 144, 180)

_SHELL_BY_SCALAR_Z_PHI = {
    (2, 0): 0,
    (0, 1): 1,
    (1, 0): 2,
    (-1, 1): 3,
    (0, 0): 4,
    (1, -1): 5,
    (-1, 0): 6,
    (0, -1): 7,
    (-2, 0): 8,
}


@dataclass(frozen=True, slots=True)
class _Candidate:
    physical_slot: int
    kind_code: int
    position: int
    represented_count: int
    frame_index: int
    relative_frame_index: int
    query_shell_index: int
    age: int
    selection_rank: int = -1
    minimum_pairwise_shell_index: int = -1


def _z_phi_sign(a: int, b: int) -> int:
    """Return the exact sign of ``a + b*phi`` using integer comparisons."""

    p = 2 * a + b
    q = b
    if q == 0:
        return (p > 0) - (p < 0)
    if p == 0:
        return (q > 0) - (q < 0)
    if (p > 0) == (q > 0):
        return 1 if p > 0 else -1
    p_squared = p * p
    five_q_squared = 5 * q * q
    if p > 0:
        return (p_squared > five_q_squared) - (p_squared < five_q_squared)
    return (five_q_squared > p_squared) - (five_q_squared < p_squared)


def _heatmap_trace(
    root: tuple[tuple[int, int], ...],
) -> dict[str, Any]:
    """Retain the exact #970 (1,i) chart as trace, never as admission."""

    q0 = root[0]
    q1 = root[1]
    activation = (
        q0[0] * q0[0] + q0[1] * q0[1],
        2 * q0[0] * q0[1] + q0[1] * q0[1],
    )
    return {
        "sin_z_phi": {"numerator": list(q0), "denominator": 2},
        "cos_z_phi": {"numerator": list(q1), "denominator": 2},
        "activation_z_phi": {
            "numerator": list(activation),
            "denominator": 4,
        },
        "chirality": _z_phi_sign(*q0),
        "cosine_polarity": _z_phi_sign(*q1),
        "typed_null_projection": q0 == (0, 0) and q1 == (0, 0),
        "used_for_admission": False,
    }


class R4SparseGeometricCandidateSoftmaxKVBindingV1(
    R4FixedRecurrentCausalKVBindingV1
):
    """Fixed recurrent memory with exact-H4 sparse source admission."""

    POLICY_NAME = POLICY
    MAXIMUM_ATTENTION_SOURCES = MAXIMUM_READ_SOURCES

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        frames: object,
    ) -> None:
        inverse_indices = getattr(frames, "inverse_indices", None)
        root_coordinates = getattr(frames, "root_coordinates", None)
        if inverse_indices is None or root_coordinates is None:
            raise ValueError(
                "sparse geometric admission requires exact H4 inverses and roots"
            )
        super().__init__(geometry, frames)

        inverses = torch.as_tensor(inverse_indices, dtype=torch.long).contiguous()
        if tuple(inverses.shape) != (120,):
            raise ValueError("H4 inverse indices must have shape [120]")
        elements = torch.arange(120, dtype=torch.long)
        identity = torch.full((120,), self.identity_index, dtype=torch.long)
        multiplication = self.frame_multiplication.detach().cpu()
        left_inverses = torch.equal(multiplication[inverses, elements], identity)
        right_inverses = torch.equal(multiplication[elements, inverses], identity)
        if not left_inverses or not right_inverses:
            raise ValueError(
                "H4 inverse indices do not invert the multiplication table"
            )

        roots = tuple(
            tuple((int(coordinate[0]), int(coordinate[1])) for coordinate in root)
            for root in root_coordinates
        )
        if len(roots) != 120 or any(len(root) != 4 for root in roots):
            raise ValueError("H4 exact roots must contain 120 four-coordinate rows")
        try:
            shell_indices = torch.tensor(
                [_SHELL_BY_SCALAR_Z_PHI[root[0]] for root in roots],
                dtype=torch.long,
            )
        except KeyError as error:
            raise ValueError("H4 root has an unknown signed S3 scalar shell") from error
        if int(shell_indices[self.identity_index]) != 0:
            raise ValueError("H4 identity does not occupy the coincident shell")

        self.register_buffer("frame_inverse_indices", inverses)
        self.register_buffer("signed_s3_shell_indices", shell_indices)
        self._exact_root_coordinates = roots

    def _relative_index(self, source_frame: int, current_frame: int) -> int:
        inverse = int(self.frame_inverse_indices[source_frame])
        return int(self.frame_multiplication[inverse, current_frame])

    def _pairwise_shell_index(self, left: int, right: int) -> int:
        inverse = int(self.frame_inverse_indices[left])
        relative = int(self.frame_multiplication[inverse, right])
        return int(self.signed_s3_shell_indices[relative])

    @staticmethod
    def _attention_order(candidate: _Candidate) -> tuple[int, int]:
        if candidate.kind_code == 0:
            return (0, -candidate.physical_slot)
        return (1, candidate.physical_slot)

    def _rank_candidates(
        self, candidates: list[_Candidate]
    ) -> tuple[list[_Candidate], int, int]:
        pairwise_shells: dict[tuple[int, int], int] = {}
        for left_offset, left in enumerate(candidates):
            for right in candidates[left_offset + 1 :]:
                shell = self._pairwise_shell_index(
                    left.relative_frame_index, right.relative_frame_index
                )
                pairwise_shells[
                    (left.physical_slot, right.physical_slot)
                ] = shell
                pairwise_shells[
                    (right.physical_slot, left.physical_slot)
                ] = shell
        selected: list[_Candidate] = []
        cost_comparisons = 0
        for query_shell in range(len(SIGNED_S3_SHELL_DEGREES)):
            remaining = [
                candidate
                for candidate in candidates
                if candidate.query_shell_index == query_shell
            ]
            while remaining and len(selected) < PERSISTENT_CANDIDATE_BUDGET:
                choices: list[tuple[tuple[int, int, int], _Candidate, int]] = []
                for candidate in remaining:
                    cost_comparisons += 1
                    minimum_pairwise = (
                        min(
                            pairwise_shells[
                                (candidate.physical_slot, prior.physical_slot)
                            ]
                            for prior in selected
                        )
                        if selected
                        else -1
                    )
                    choices.append(
                        (
                            (-minimum_pairwise, candidate.age, candidate.physical_slot),
                            candidate,
                            minimum_pairwise,
                        )
                    )
                _, chosen, minimum_pairwise = min(choices, key=lambda item: item[0])
                selected.append(
                    _Candidate(
                        physical_slot=chosen.physical_slot,
                        kind_code=chosen.kind_code,
                        position=chosen.position,
                        represented_count=chosen.represented_count,
                        frame_index=chosen.frame_index,
                        relative_frame_index=chosen.relative_frame_index,
                        query_shell_index=chosen.query_shell_index,
                        age=chosen.age,
                        selection_rank=len(selected),
                        minimum_pairwise_shell_index=minimum_pairwise,
                    )
                )
                remaining.remove(chosen)
            if len(selected) == PERSISTENT_CANDIDATE_BUDGET:
                break
        return (
            sorted(selected, key=self._attention_order),
            len(pairwise_shells) // 2,
            cost_comparisons,
        )

    def _select_persistent_candidates(
        self,
        state: FixedRecurrentKVState,
        current_frames: Tensor,
    ) -> FixedRecurrentCandidateSelection:
        batch_size = int(current_frames.shape[0])
        occupied_summary_banks = [
            bank
            for bank in range(SUMMARY_BANKS - 1, -1, -1)
            if int(state.summary_counts[0, bank]) > 0
        ]
        selected_by_batch: list[list[_Candidate]] = []
        age_only_by_batch: list[list[int]] = []
        pairwise_shell_evaluations = 0
        candidate_cost_comparisons = 0
        pairwise_by_batch: list[int] = []
        comparisons_by_batch: list[int] = []
        eligible_per_batch = len(occupied_summary_banks) + state.live_length

        for batch_offset in range(batch_size):
            current_frame = int(current_frames[batch_offset])
            candidates: list[_Candidate] = []
            for bank in occupied_summary_banks:
                frame = int(state.summary_frame_indices[batch_offset, bank])
                position = int(state.summary_last_positions[batch_offset, bank])
                relative = self._relative_index(frame, current_frame)
                candidates.append(
                    _Candidate(
                        physical_slot=bank,
                        kind_code=0,
                        position=position,
                        represented_count=int(
                            state.summary_counts[batch_offset, bank]
                        ),
                        frame_index=frame,
                        relative_frame_index=relative,
                        query_shell_index=int(
                            self.signed_s3_shell_indices[relative]
                        ),
                        age=state.tokens_seen - position,
                    )
                )
            for live_offset in range(state.live_length):
                frame = int(state.live_frame_indices[batch_offset, live_offset])
                position = int(state.live_positions[batch_offset, live_offset])
                relative = self._relative_index(frame, current_frame)
                candidates.append(
                    _Candidate(
                        physical_slot=SUMMARY_BANKS + live_offset,
                        kind_code=1,
                        position=position,
                        represented_count=1,
                        frame_index=frame,
                        relative_frame_index=relative,
                        query_shell_index=int(
                            self.signed_s3_shell_indices[relative]
                        ),
                        age=state.tokens_seen - position,
                    )
                )
            selected, pairwise_count, comparison_count = self._rank_candidates(
                candidates
            )
            selected_by_batch.append(selected)
            pairwise_shell_evaluations += pairwise_count
            candidate_cost_comparisons += comparison_count
            pairwise_by_batch.append(pairwise_count)
            comparisons_by_batch.append(comparison_count)
            age_only_by_batch.append(
                sorted(
                    (
                        candidate.physical_slot
                        for candidate in sorted(
                            candidates,
                            key=lambda item: (item.age, item.physical_slot),
                        )[:PERSISTENT_CANDIDATE_BUDGET]
                    )
                )
            )

        selected_count = min(eligible_per_batch, PERSISTENT_CANDIDATE_BUDGET)
        if any(len(selected) != selected_count for selected in selected_by_batch):
            raise RuntimeError("sparse candidate count differs across batch lanes")

        def field_tensor(name: str) -> Tensor:
            return torch.tensor(
                [
                    [int(getattr(candidate, name)) for candidate in selected]
                    for selected in selected_by_batch
                ],
                device=current_frames.device,
                dtype=torch.long,
            )

        return FixedRecurrentCandidateSelection(
            source_slots=field_tensor("physical_slot"),
            source_kind_codes=field_tensor("kind_code"),
            source_positions=field_tensor("position"),
            source_represented_counts=field_tensor("represented_count"),
            source_frame_indices=field_tensor("frame_index"),
            relative_frame_indices=field_tensor("relative_frame_index"),
            query_shell_indices=field_tensor("query_shell_index"),
            selection_ranks=field_tensor("selection_rank"),
            minimum_pairwise_shell_indices=field_tensor(
                "minimum_pairwise_shell_index"
            ),
            current_frame_indices=current_frames.detach().clone(),
            current_position=state.tokens_seen,
            eligible_persistent_sources=batch_size * eligible_per_batch,
            age_only_source_slots=torch.tensor(
                age_only_by_batch,
                device=current_frames.device,
                dtype=torch.long,
            ),
            pairwise_shell_evaluations=pairwise_shell_evaluations,
            candidate_cost_comparisons=candidate_cost_comparisons,
            pairwise_shell_evaluations_by_batch=torch.tensor(
                pairwise_by_batch,
                device=current_frames.device,
                dtype=torch.long,
            ),
            candidate_cost_comparisons_by_batch=torch.tensor(
                comparisons_by_batch,
                device=current_frames.device,
                dtype=torch.long,
            ),
        )

    def describe_candidate_selection(
        self,
        selection: FixedRecurrentCandidateSelection,
        *,
        batch_offset: int = 0,
    ) -> dict[str, Any]:
        if not 0 <= batch_offset < int(selection.source_slots.shape[0]):
            raise IndexError("candidate trace batch offset is out of range")
        admitted: list[dict[str, Any]] = []
        for selected_offset in range(int(selection.source_slots.shape[1])):
            relative_index = int(
                selection.relative_frame_indices[batch_offset, selected_offset]
            )
            root = self._exact_root_coordinates[relative_index]
            pairwise_shell = int(
                selection.minimum_pairwise_shell_indices[
                    batch_offset, selected_offset
                ]
            )
            admitted.append(
                {
                    "source_kind": (
                        "summary"
                        if int(
                            selection.source_kind_codes[
                                batch_offset, selected_offset
                            ]
                        )
                        == 0
                        else "live"
                    ),
                    "physical_slot": int(
                        selection.source_slots[batch_offset, selected_offset]
                    ),
                    "causal_position": int(
                        selection.source_positions[batch_offset, selected_offset]
                    ),
                    "represented_token_count": int(
                        selection.source_represented_counts[
                            batch_offset, selected_offset
                        ]
                    ),
                    "causal_age": selection.current_position
                    - int(
                        selection.source_positions[batch_offset, selected_offset]
                    ),
                    "source_frame_index": int(
                        selection.source_frame_indices[
                            batch_offset, selected_offset
                        ]
                    ),
                    "relative_frame_index": relative_index,
                    "relative_root_z_phi_over_2": [list(pair) for pair in root],
                    "query_shell_degrees": SIGNED_S3_SHELL_DEGREES[
                        int(
                            selection.query_shell_indices[
                                batch_offset, selected_offset
                            ]
                        )
                    ],
                    "selection_rank": int(
                        selection.selection_ranks[batch_offset, selected_offset]
                    ),
                    "minimum_pairwise_shell_degrees": (
                        None
                        if pairwise_shell < 0
                        else SIGNED_S3_SHELL_DEGREES[pairwise_shell]
                    ),
                    "heatmap_trace": _heatmap_trace(root),
                }
            )

        current_frame = int(selection.current_frame_indices[batch_offset])
        identity_root = self._exact_root_coordinates[self.identity_index]
        admitted.append(
            {
                "source_kind": "current",
                "physical_slot": SUMMARY_BANKS + LIVE_WINDOW,
                "causal_position": selection.current_position,
                "represented_token_count": 1,
                "causal_age": 0,
                "source_frame_index": current_frame,
                "relative_frame_index": self.identity_index,
                "relative_root_z_phi_over_2": [
                    list(pair) for pair in identity_root
                ],
                "query_shell_degrees": 0,
                "selection_rank": None,
                "minimum_pairwise_shell_degrees": None,
                "heatmap_trace": _heatmap_trace(identity_root),
            }
        )
        age_only = (
            []
            if selection.age_only_source_slots is None
            else selection.age_only_source_slots[batch_offset].tolist()
        )
        selected_slots = selection.source_slots[batch_offset].tolist()
        return {
            "position": selection.current_position,
            "eligible_persistent_sources": (
                selection.eligible_persistent_sources
                // int(selection.source_slots.shape[0])
            ),
            "selected_persistent_sources": len(selected_slots),
            "admitted_sources_including_current": len(admitted),
            "selected_physical_slots": selected_slots,
            "age_only_physical_slots": age_only,
            "differs_from_age_only": set(selected_slots) != set(age_only),
            "pairwise_shell_evaluations": (
                selection.pairwise_shell_evaluations
                if selection.pairwise_shell_evaluations_by_batch is None
                else int(
                    selection.pairwise_shell_evaluations_by_batch[batch_offset]
                )
            ),
            "candidate_cost_comparisons": (
                selection.candidate_cost_comparisons
                if selection.candidate_cost_comparisons_by_batch is None
                else int(
                    selection.candidate_cost_comparisons_by_batch[batch_offset]
                )
            ),
            "admitted": admitted,
        }


__all__ = [
    "MAXIMUM_READ_SOURCES",
    "PERSISTENT_CANDIDATE_BUDGET",
    "POLICY",
    "SIGNED_S3_SHELL_DEGREES",
    "R4SparseGeometricCandidateSoftmaxKVBindingV1",
]
