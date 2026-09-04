"""Focused causal and memory checks for #973's fixed recurrent R4 cache."""

from __future__ import annotations

import math
import copy
import unittest
from types import SimpleNamespace

import torch

from r4_softmax_trainer.fixed_recurrent_kv_binding import (
    LIVE_WINDOW,
    MAXIMUM_READ_SOURCES,
    POLICY,
    RECURRENT_METADATA_I64_VALUES,
    RECURRENT_STATE_BYTES_F32,
    RECURRENT_STATE_VALUES,
    SUMMARY_BANKS,
    R4FixedRecurrentCausalKVBindingV1,
)
from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    HEAD_DIM,
    HEADS,
    LAYERS,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    VOCAB_SIZE,
)
from r4_softmax_trainer.position_kv_binding import (
    R4PositionPreservingCausalKVBindingV1,
)
from r4_softmax_trainer.sparse_geometric_kv_binding import (
    MAXIMUM_READ_SOURCES as SPARSE_MAXIMUM_READ_SOURCES,
    PERSISTENT_CANDIDATE_BUDGET,
    POLICY as SPARSE_POLICY,
    R4SparseGeometricCandidateSoftmaxKVBindingV1,
)


def _geometry_and_frames() -> tuple[GroupAddressArtifact, SimpleNamespace]:
    order = 120
    elements = torch.arange(order, dtype=torch.long)
    table = (elements[:, None] + elements[None, :]) % order
    leaves = torch.arange(VOCAB_SIZE, dtype=torch.long) % 24
    leaves[0] = 0
    geometry = GroupAddressArtifact(
        arm="exact_h4",
        identity_offset=0,
        token_leaves=leaves,
        left_actions=table,
        artifact_cid="synthetic:fixed-recurrent-geometry",
    )
    angles = torch.arange(order, dtype=torch.float64) * (2.0 * math.pi / order)
    matrices = torch.eye(4, dtype=torch.float64).repeat(order, 1, 1)
    matrices[:, 0, 0] = torch.cos(angles)
    matrices[:, 0, 1] = -torch.sin(angles)
    matrices[:, 1, 0] = torch.sin(angles)
    matrices[:, 1, 1] = torch.cos(angles)
    permutation = torch.arange(order, dtype=torch.long)
    inverses = (-elements) % order
    scalar_shells = (
        (2, 0),
        (0, 1),
        (1, 0),
        (-1, 1),
        (0, 0),
        (1, -1),
        (-1, 0),
        (0, -1),
        (-2, 0),
    )
    roots = tuple(
        (
            scalar_shells[offset % len(scalar_shells)],
            ((offset % 3) - 1, 0),
            (0, (offset % 2)),
            ((offset % 5) - 2, 1),
        )
        for offset in range(order)
    )
    roots = (((2, 0), (0, 0), (0, 0), (0, 0)), *roots[1:])
    frames = SimpleNamespace(
        frame_matrices=matrices,
        multiplication_indices=table,
        inverse_indices=inverses,
        transport_permutation=permutation,
        identity_index=0,
        root_coordinates=roots,
        artifact_cid="synthetic:fixed-recurrent-frames",
    )
    return geometry, frames


class FixedRecurrentKVBindingTests(unittest.TestCase):
    def test_fixed_state_reads_before_write_and_folds_every_eviction(self) -> None:
        torch.manual_seed(1_120)
        geometry, frames = _geometry_and_frames()
        source = R4PositionPreservingCausalKVBindingV1(  # type: ignore[arg-type]
            geometry, frames
        )
        artifact = source.export_learned_artifact()
        exact = R4PositionPreservingCausalKVBindingV1.from_learned_artifact(
            artifact, geometry=geometry, frames=frames  # type: ignore[arg-type]
        )
        recurrent = R4FixedRecurrentCausalKVBindingV1.from_learned_artifact(
            artifact, geometry=geometry, frames=frames
        )
        exact.eval()
        recurrent.eval()

        self.assertEqual(POLICY, "R4FixedRecurrentCausalKVBindingV1")
        self.assertEqual(recurrent.parameter_count(), PARAMETER_COUNT)
        self.assertEqual(recurrent.export_learned_artifact(), artifact)
        self.assertEqual(
            recurrent.recurrent_state_value_count(), RECURRENT_STATE_VALUES
        )
        self.assertEqual(
            recurrent.recurrent_state_byte_count_f32(),
            RECURRENT_STATE_BYTES_F32,
        )
        self.assertEqual(
            recurrent.recurrent_metadata_i64_value_count(),
            RECURRENT_METADATA_I64_VALUES,
        )
        self.assertEqual(RECURRENT_STATE_BYTES_F32, STATE_BYTES_F32 // 10)

        tokens = torch.tensor(
            [0, 1, 7, 3, 9, 2, 12, 4, 8, 5, 11, 6], dtype=torch.long
        )
        exact_state = exact.initial_state(1, execution="r4")
        recurrent_state = recurrent.initial_state(1)
        compression_was_read = False

        with torch.inference_mode():
            for position, token in enumerate(tokens):
                exact_output = exact.step(
                    token.view(1), exact_state, execution="r4"
                )
                prior_live = recurrent_state.live_keys.clone()
                prior_summaries = recurrent_state.summary_keys_local.clone()
                prior_counts = recurrent_state.summary_counts.clone()
                recurrent_output = recurrent.step(token.view(1), recurrent_state)

                # A step returns a new state; the causal input state is untouched.
                self.assertTrue(torch.equal(recurrent_state.live_keys, prior_live))
                self.assertTrue(
                    torch.equal(
                        recurrent_state.summary_keys_local, prior_summaries
                    )
                )
                self.assertTrue(
                    torch.equal(recurrent_state.summary_counts, prior_counts)
                )

                # The decision that causes the first eviction still reads all
                # nine sources exactly. Compression can affect only later reads.
                if position <= LIVE_WINDOW:
                    self.assertTrue(
                        torch.equal(
                            exact_output.logits, recurrent_output.logits
                        )
                    )
                    self.assertTrue(
                        torch.equal(
                            exact_output.attention_weights[
                                ..., : position + 1
                            ],
                            recurrent_output.attention_weights[
                                ..., : position + 1
                            ],
                        )
                    )
                if recurrent_output.audit.summary_slots_read:
                    compression_was_read = True

                self.assertEqual(
                    tuple(recurrent_output.attention_weights.shape),
                    (LAYERS, 1, HEADS, MAXIMUM_READ_SOURCES),
                )
                self.assertTrue(
                    torch.allclose(
                        recurrent_output.attention_weights.sum(dim=-1),
                        torch.ones(LAYERS, 1, HEADS),
                        atol=1.0e-6,
                        rtol=0.0,
                    )
                )
                exact_state = exact_output.final_state
                recurrent_state = recurrent_output.final_state

                state_values = (
                    recurrent_state.live_keys.numel()
                    + recurrent_state.live_values.numel()
                    + recurrent_state.summary_keys_local.numel()
                    + recurrent_state.summary_values_local.numel()
                )
                self.assertEqual(state_values, RECURRENT_STATE_VALUES)

        self.assertTrue(compression_was_read)
        self.assertEqual(recurrent_state.tokens_seen, len(tokens))
        self.assertEqual(recurrent_state.live_length, LIVE_WINDOW)
        self.assertEqual(
            int(recurrent_state.summary_counts.sum()),
            len(tokens) - LIVE_WINDOW,
        )
        self.assertEqual(
            recurrent_state.summary_counts.tolist(), [[0, 0, 4, 0]]
        )
        self.assertEqual(recurrent_state.audit.evictions, 4)
        self.assertEqual(recurrent_state.audit.summary_merges, 3)
        self.assertGreater(recurrent_state.audit.summary_slots_read, 0)
        self.assertLessEqual(
            recurrent_state.audit.peak_attention_source_slots,
            MAXIMUM_READ_SOURCES,
        )
        self.assertEqual(recurrent_state.audit.future_reads, 0)
        self.assertEqual(recurrent_state.audit.forbidden_reads, 0)

    def test_inherited_exact_and_control_paths_are_not_reachable(self) -> None:
        geometry, frames = _geometry_and_frames()
        model = R4FixedRecurrentCausalKVBindingV1(  # type: ignore[arg-type]
            geometry, frames
        )
        tokens = torch.tensor([[0, 1]], dtype=torch.long)
        with self.assertRaisesRegex(ValueError, "only execution='r4'"):
            model(tokens, execution="plain")
        with self.assertRaisesRegex(ValueError, "intervention='native'"):
            model(tokens, intervention="current_only")
        exact_state = R4PositionPreservingCausalKVBindingV1(  # type: ignore[arg-type]
            geometry, frames
        ).initial_state(1, execution="r4")
        with self.assertRaisesRegex(TypeError, "FixedRecurrentKVState"):
            model.step(tokens[:, 0], exact_state)

    def test_sparse_geometry_selects_before_payload_reads(self) -> None:
        torch.manual_seed(1_122)
        geometry, frames = _geometry_and_frames()
        source = R4PositionPreservingCausalKVBindingV1(  # type: ignore[arg-type]
            geometry, frames
        )
        artifact = source.export_learned_artifact()
        fixed = R4FixedRecurrentCausalKVBindingV1.from_learned_artifact(
            artifact, geometry=geometry, frames=frames
        )
        sparse = (
            R4SparseGeometricCandidateSoftmaxKVBindingV1.from_learned_artifact(
                artifact, geometry=geometry, frames=frames
            )
        )
        fixed.eval()
        sparse.eval()

        self.assertEqual(
            SPARSE_POLICY, "R4SparseGeometricCandidateSoftmaxKVBindingV1"
        )
        self.assertEqual(PERSISTENT_CANDIDATE_BUDGET, LIVE_WINDOW)
        self.assertEqual(SPARSE_MAXIMUM_READ_SOURCES, LIVE_WINDOW + 1)
        self.assertEqual(sparse.parameter_count(), PARAMETER_COUNT)
        self.assertEqual(sparse.export_learned_artifact(), artifact)

        fixed_state = fixed.initial_state(1)
        sparse_state = sparse.initial_state(1)
        tokens = torch.tensor([0, 1, 7, 3, 9, 2, 12, 4, 8], dtype=torch.long)
        with torch.inference_mode():
            for token in tokens:
                fixed_output = fixed.step(token.view(1), fixed_state)
                sparse_output = sparse.step(token.view(1), sparse_state)
                self.assertTrue(torch.equal(fixed_output.logits, sparse_output.logits))
                admitted = fixed_state.live_length + int(
                    torch.count_nonzero(fixed_state.summary_counts[0])
                ) + 1
                self.assertTrue(
                    torch.equal(
                        fixed_output.attention_weights[..., :admitted],
                        sparse_output.attention_weights[..., :admitted],
                    )
                )
                fixed_state = fixed_output.final_state
                sparse_state = sparse_output.final_state

        self.assertEqual(sparse_state.tokens_seen, LIVE_WINDOW + 1)
        self.assertEqual(int(sparse_state.summary_counts.sum()), 1)
        next_token = torch.tensor([5], dtype=torch.long)
        pristine = copy.deepcopy(sparse_state)
        with torch.inference_mode():
            baseline = sparse.step(next_token, pristine)
        selection = baseline.candidate_selection
        self.assertIsNotNone(selection)
        assert selection is not None
        self.assertEqual(selection.eligible_persistent_sources, 9)
        self.assertEqual(selection.source_slots.numel(), LIVE_WINDOW)
        self.assertEqual(selection.pairwise_shell_evaluations, 36)
        self.assertEqual(tuple(baseline.attention_weights.shape), (LAYERS, 1, HEADS, 9))
        self.assertEqual(baseline.audit.eligible_persistent_source_slots, 9)
        self.assertEqual(baseline.audit.selected_persistent_source_slots, 8)
        self.assertEqual(baseline.audit.materialized_attention_scores, 72)
        self.assertEqual(baseline.audit.unselected_key_value_reads, 0)
        self.assertEqual(baseline.audit.complete_prefix_scans, 0)
        self.assertEqual(baseline.audit.peak_attention_source_slots, 9)

        all_slots = set(range(SUMMARY_BANKS)) | set(
            range(SUMMARY_BANKS, SUMMARY_BANKS + sparse_state.live_length)
        )
        occupied_slots = {
            bank
            for bank in range(SUMMARY_BANKS)
            if int(sparse_state.summary_counts[0, bank]) > 0
        } | set(range(SUMMARY_BANKS, SUMMARY_BANKS + sparse_state.live_length))
        self.assertEqual(all_slots & occupied_slots, occupied_slots)
        selected_slots = set(selection.source_slots[0].tolist())
        unselected_slots = occupied_slots - selected_slots
        self.assertEqual(len(unselected_slots), 1)

        omitted_state = copy.deepcopy(sparse_state)
        omitted_slot = unselected_slots.pop()
        if omitted_slot < SUMMARY_BANKS:
            omitted_state.summary_keys_local[..., omitted_slot, :].add_(10_000.0)
            omitted_state.summary_values_local[..., omitted_slot, :].add_(10_000.0)
        else:
            live_offset = omitted_slot - SUMMARY_BANKS
            omitted_state.live_keys[..., live_offset, :].add_(10_000.0)
            omitted_state.live_values[..., live_offset, :].add_(10_000.0)
        with torch.inference_mode():
            omitted_output = sparse.step(next_token, omitted_state)
        self.assertTrue(torch.equal(baseline.logits, omitted_output.logits))

        selected_state = copy.deepcopy(sparse_state)
        selected_slot = next(iter(selected_slots))
        if selected_slot < SUMMARY_BANKS:
            selected_state.summary_values_local[..., selected_slot, :].add_(100.0)
        else:
            live_offset = selected_slot - SUMMARY_BANKS
            selected_state.live_values[..., live_offset, :].add_(100.0)
        with torch.inference_mode():
            selected_output = sparse.step(next_token, selected_state)
        self.assertFalse(torch.equal(baseline.logits, selected_output.logits))

        trace = sparse.describe_candidate_selection(selection)
        self.assertEqual(trace["eligible_persistent_sources"], 9)
        self.assertEqual(trace["selected_persistent_sources"], 8)
        self.assertEqual(trace["admitted_sources_including_current"], 9)
        self.assertEqual(trace["admitted"][-1]["source_kind"], "current")
        for source_trace in trace["admitted"]:
            self.assertEqual(len(source_trace["relative_root_z_phi_over_2"]), 4)
            self.assertFalse(source_trace["heatmap_trace"]["used_for_admission"])
        ranked = sorted(
            trace["admitted"][:-1], key=lambda item: item["selection_rank"]
        )
        self.assertEqual(
            [item["query_shell_degrees"] for item in ranked],
            sorted(item["query_shell_degrees"] for item in ranked),
        )
        current_frame = int(selection.current_frame_indices[0])
        for item in ranked:
            source_frame = item["source_frame_index"]
            expected_relative = int(
                frames.multiplication_indices[
                    frames.inverse_indices[source_frame], current_frame
                ]
            )
            self.assertEqual(item["relative_frame_index"], expected_relative)
            self.assertEqual(
                item["relative_root_z_phi_over_2"],
                [list(pair) for pair in frames.root_coordinates[expected_relative]],
            )


if __name__ == "__main__":
    unittest.main()
