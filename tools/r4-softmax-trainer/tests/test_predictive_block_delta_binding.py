"""Focused mechanics tests for ``R4PredictiveBlockDeltaBindingV1``."""

from __future__ import annotations

import math
import unittest
from types import SimpleNamespace

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    HIDDEN_SIZE,
    PARAMETER_COUNT,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)
from r4_softmax_trainer.predictive_block_delta_binding import (
    BINDING_STATE_VALUES,
    MATRIX_STATE_VALUES,
    POLICY,
    TRAINABLE_PARAMETER_COUNT,
    PredictiveBlockDeltaState,
    R4PredictiveBlockDeltaBindingV1,
)


def _geometry_and_frames() -> tuple[GroupAddressArtifact, SimpleNamespace]:
    order = 120
    table = torch.tensor(
        [[(left + right) % order for right in range(order)] for left in range(order)],
        dtype=torch.long,
    )
    leaves = torch.arange(VOCAB_SIZE, dtype=torch.long) % 12
    leaves[0] = 0
    geometry = GroupAddressArtifact(
        arm="exact_h4",
        identity_offset=0,
        token_leaves=leaves,
        left_actions=table,
        artifact_cid="synthetic-group",
    )
    angles = torch.arange(order, dtype=torch.float64) * (2.0 * math.pi / order)
    frames = torch.eye(4, dtype=torch.float64).repeat(order, 1, 1)
    frames[:, 0, 0] = torch.cos(angles)
    frames[:, 0, 1] = -torch.sin(angles)
    frames[:, 1, 0] = torch.sin(angles)
    frames[:, 1, 1] = torch.cos(angles)
    permutation = torch.arange(order, dtype=torch.long)
    permutation[1], permutation[2] = permutation[2].clone(), permutation[1].clone()
    artifact = SimpleNamespace(
        frame_matrices=frames,
        multiplication_indices=table,
        inverse_indices=torch.tensor(
            [(-index) % order for index in range(order)], dtype=torch.long
        ),
        transport_permutation=permutation,
        identity_index=0,
        artifact_cid="synthetic-frames",
    )
    return geometry, artifact


def _model(*, arm: str = "geometric") -> R4PredictiveBlockDeltaBindingV1:
    geometry, frames = _geometry_and_frames()
    return R4PredictiveBlockDeltaBindingV1(geometry, frames, arm=arm)  # type: ignore[arg-type]


def _assert_state_close(
    case: unittest.TestCase,
    first: PredictiveBlockDeltaState,
    second: PredictiveBlockDeltaState,
) -> None:
    for left, right in (
        (first.backbone.keys, second.backbone.keys),
        (first.backbone.values, second.backbone.values),
        (first.matrices, second.matrices),
        (first.previous_key, second.previous_key),
    ):
        case.assertTrue(torch.allclose(left, right, atol=3e-6, rtol=3e-5))
    case.assertTrue(torch.equal(first.backbone.occupied, second.backbone.occupied))
    case.assertTrue(torch.equal(first.frame_indices, second.frame_indices))
    case.assertTrue(torch.equal(first.key_valid, second.key_valid))


class PredictiveBlockDeltaBindingTests(unittest.TestCase):
    def setUp(self) -> None:
        torch.manual_seed(973)

    def test_frozen_ledgers_initialization_and_state_shape(self) -> None:
        geometric = _model()
        plain = _model(arm="plain")
        self.assertEqual(POLICY, "R4PredictiveBlockDeltaBindingV1")
        self.assertEqual(geometric.trainable_parameter_count(), TRAINABLE_PARAMETER_COUNT)
        self.assertEqual(geometric.parameter_count(), PARAMETER_COUNT + 9_228)
        self.assertEqual(geometric.export_binding_artifact(), plain.export_binding_artifact())
        self.assertTrue(all(not value.requires_grad for value in geometric.frozen_base_parameters()))
        self.assertTrue(all(value.requires_grad for value in geometric.trainable_parameters()))

        state = geometric.initial_state(2)
        self.assertEqual(state.matrices.numel() // 2, MATRIX_STATE_VALUES)
        self.assertEqual(
            (state.matrices.numel() + state.previous_key.numel()) // 2,
            BINDING_STATE_VALUES,
        )
        self.assertFalse(bool(state.key_valid.any()))
        self.assertTrue(torch.equal(state.frame_indices, torch.zeros(2, dtype=torch.long)))

    def test_state_off_is_exact_v1_and_controls_have_equal_work(self) -> None:
        geometry, _ = _geometry_and_frames()
        base = R4RetainedLanguagePathV1(geometry)
        model = _model()
        payload = base.export_learned_artifact()
        model.load_qualified_base_artifact(payload)
        tokens = torch.tensor([[0, 1, 4, 2, 7], [0, 2, 5, 3, 8]])
        expected = base(tokens, implementation="stationary")
        state_off = model(tokens, intervention="state_off")
        self.assertTrue(torch.equal(expected.logits, state_off.logits))
        self.assertTrue(torch.equal(expected.logits, state_off.base_logits))
        self.assertGreater(float(state_off.head_logits.abs().max()), 0.0)

        native = model(tokens)
        no_delta = model(tokens, intervention="no_delta")
        permuted = model(tokens, intervention="transport_permuted")
        plain = _model(arm="plain")(tokens)
        signatures = {
            output.audit.work_signature()
            for output in (native, no_delta, permuted, state_off, plain)
        }
        self.assertEqual(len(signatures), 1)
        self.assertFalse(torch.equal(native.logits, no_delta.logits))
        self.assertFalse(torch.equal(native.logits, permuted.logits))
        self.assertTrue(
            torch.equal(native.final_state.frame_indices, permuted.final_state.frame_indices)
        )

    def test_transport_control_permutes_only_each_new_leaf_action(self) -> None:
        model = _model()
        previous = torch.arange(120, dtype=torch.long).repeat_interleave(120)
        leaves = torch.arange(120, dtype=torch.long).repeat(120)
        current = model.frame_multiplication[previous, leaves]
        previous_frames = model.frame_matrices.index_select(0, previous)
        current_frames = model.frame_matrices.index_select(0, current)
        endpoint_transport = torch.matmul(
            current_frames.transpose(-1, -2), previous_frames
        )
        step_transport = model._step_transport(leaves, intervention="native")
        self.assertTrue(
            torch.allclose(endpoint_transport, step_transport, atol=3e-6, rtol=3e-5)
        )

        previous = torch.tensor([1], dtype=torch.long)
        leaf = torch.tensor([1], dtype=torch.long)
        current = model.frame_multiplication[previous, leaf]
        expected = model.frame_matrices.index_select(
            0, model.transport_permutation.index_select(0, leaf)
        ).transpose(-1, -2)
        observed = model._step_transport(leaf, intervention="transport_permuted")
        old_endpoint_permutation = torch.matmul(
            model.frame_matrices.index_select(
                0, model.transport_permutation.index_select(0, current)
            ).transpose(-1, -2),
            model.frame_matrices.index_select(
                0, model.transport_permutation.index_select(0, previous)
            ),
        )
        self.assertTrue(torch.equal(observed, expected))
        self.assertFalse(
            torch.allclose(observed, old_endpoint_permutation, atol=3e-6, rtol=3e-5)
        )

    def test_stationary_direct_incremental_causality_and_observability(self) -> None:
        model = _model()
        tokens = torch.tensor([[0, 1, 4, 2, 7, 3], [0, 2, 5, 3, 8, 4]])
        stationary = model(tokens, implementation="stationary")
        direct = model(tokens, implementation="direct")
        self.assertTrue(
            torch.allclose(stationary.logits, direct.logits, atol=3e-6, rtol=3e-5)
        )
        _assert_state_close(self, stationary.final_state, direct.final_state)

        state = model.initial_state(2)
        steps = []
        for position in range(tokens.shape[1]):
            output = model.step(tokens[:, position], state)
            steps.append(output.logits)
            state = output.final_state
        incremental = torch.stack(steps, dim=1)
        self.assertTrue(torch.allclose(direct.logits, incremental, atol=3e-6, rtol=3e-5))
        _assert_state_close(self, direct.final_state, state)

        changed = tokens.clone()
        changed[:, 4:] = torch.tensor([[11, 10], [9, 6]])
        changed_output = model(changed)
        self.assertTrue(torch.equal(stationary.logits[:, :4], changed_output.logits[:, :4]))
        target_a = tokens.roll(shifts=-1, dims=1)
        target_b = target_a.clone()
        target_b[:, -1] = 99
        self.assertTrue(
            torch.equal(model(tokens, target_a).logits, model(tokens, target_b).logits)
        )

        first = model(torch.tensor([[0, 1, 4]]))
        second = model(torch.tensor([[0, 1, 5]]))
        self.assertFalse(torch.equal(first.final_state.matrices, second.final_state.matrices))
        self.assertFalse(torch.equal(first.head_logits[:, -1], second.head_logits[:, -1]))

    def test_every_trainable_value_gets_a_finite_nonzero_gradient(self) -> None:
        model = _model()
        tokens = torch.tensor(
            [[0, 1, 4, 2, 7, 3, 9], [0, 2, 5, 3, 8, 4, 10]], dtype=torch.long
        )
        targets = torch.tensor(
            [[1, 4, 2, 7, 3, 9, 11], [2, 5, 3, 8, 4, 10, 12]], dtype=torch.long
        )
        output = model(tokens, targets)
        self.assertIsNotNone(output.loss)
        assert output.loss is not None
        output.loss.backward()
        seen = 0
        for parameter in model.trainable_parameters():
            self.assertIsNotNone(parameter.grad)
            assert parameter.grad is not None
            self.assertTrue(bool(torch.isfinite(parameter.grad).all()))
            self.assertTrue(bool((parameter.grad != 0).all()))
            seen += parameter.numel()
        self.assertEqual(seen, TRAINABLE_PARAMETER_COUNT)


if __name__ == "__main__":
    unittest.main()
