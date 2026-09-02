"""Focused mechanics tests for the frozen #1043 position-K/V policy."""

from __future__ import annotations

import math
import unittest
from types import SimpleNamespace

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    CONTEXT,
    HEAD_DIM,
    HEADS,
    LAYERS,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
)
from r4_softmax_trainer.position_kv_binding import (
    POLICY,
    PositionKVCacheState,
    R4PositionPreservingCausalKVBindingV1,
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
        artifact_cid="synthetic:position-kv-geometry",
    )

    angles = torch.arange(order, dtype=torch.float64) * (2.0 * math.pi / order)
    matrices = torch.eye(4, dtype=torch.float64).repeat(order, 1, 1)
    matrices[:, 0, 0] = torch.cos(angles)
    matrices[:, 0, 1] = -torch.sin(angles)
    matrices[:, 1, 0] = torch.sin(angles)
    matrices[:, 1, 1] = torch.cos(angles)
    permutation = torch.arange(order, dtype=torch.long)
    permutation[1], permutation[2] = permutation[2].clone(), permutation[1].clone()
    frames = SimpleNamespace(
        frame_matrices=matrices,
        multiplication_indices=table,
        transport_permutation=permutation,
        identity_index=0,
        artifact_cid="synthetic:position-kv-frames",
    )
    return geometry, frames


def _model() -> R4PositionPreservingCausalKVBindingV1:
    geometry, frames = _geometry_and_frames()
    return R4PositionPreservingCausalKVBindingV1(geometry, frames)  # type: ignore[arg-type]


def _assert_state_close(
    case: unittest.TestCase,
    first: PositionKVCacheState,
    second: PositionKVCacheState,
) -> None:
    case.assertEqual(first.length, second.length)
    case.assertTrue(torch.equal(first.valid, second.valid))
    case.assertTrue(
        torch.equal(first.source_frame_indices, second.source_frame_indices)
    )
    case.assertTrue(
        torch.equal(first.current_frame_indices, second.current_frame_indices)
    )
    case.assertTrue(torch.allclose(first.keys, second.keys, atol=2e-5, rtol=2e-5))
    case.assertTrue(
        torch.allclose(first.values, second.values, atol=2e-5, rtol=2e-5)
    )


class PositionKVBindingTests(unittest.TestCase):
    def setUp(self) -> None:
        torch.manual_seed(10_043)
        self.tokens = torch.tensor(
            [[0, 1, 7, 3, 9, 2, 12], [0, 2, 6, 4, 10, 3, 13]],
            dtype=torch.long,
        )
        self.targets = torch.tensor(
            [[1, 7, 3, 9, 2, 12, 4], [2, 6, 4, 10, 3, 13, 5]],
            dtype=torch.long,
        )

    def test_frozen_parameter_cache_and_artifact_ledgers(self) -> None:
        model = _model()
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        self.assertEqual(POLICY, "R4PositionPreservingCausalKVBindingV1")
        self.assertEqual(model.parameter_count(), PARAMETER_COUNT)
        self.assertEqual(model.state_value_count(), STATE_VALUES)
        self.assertEqual(model.state_byte_count_f32(), STATE_BYTES_F32)
        self.assertEqual(model.validity_bit_count(), VALIDITY_BITS)
        model_parameters = dict(model.named_parameters())
        ordinary_parameters = dict(ordinary.named_parameters())
        self.assertEqual(tuple(model_parameters), tuple(ordinary_parameters))
        for name in model_parameters:
            self.assertTrue(
                torch.equal(model_parameters[name], ordinary_parameters[name]), name
            )
        self.assertEqual(model.export_learned_artifact(), ordinary.export_learned_artifact())

        state = model.initial_state(2, execution="r4")
        self.assertEqual((state.keys.numel() + state.values.numel()) // 2, STATE_VALUES)
        self.assertEqual(state.valid.numel() // 2, VALIDITY_BITS)
        self.assertEqual(
            tuple(state.keys.shape), (LAYERS, 2, HEADS, CONTEXT, HEAD_DIM)
        )
        self.assertFalse(bool(state.valid.any()))
        self.assertEqual(state.length, 0)

        payload = ordinary.export_learned_artifact()
        geometry, frames = _geometry_and_frames()
        restored = R4PositionPreservingCausalKVBindingV1.from_learned_artifact(
            payload, geometry=geometry, frames=frames  # type: ignore[arg-type]
        )
        self.assertEqual(restored.export_learned_artifact(), payload)

    def test_plain_full_is_the_ordinary_reference_and_gradients_are_live(self) -> None:
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        model = _model()
        expected = ordinary(self.tokens, self.targets)
        observed = model(self.tokens, self.targets, execution="plain")
        self.assertTrue(torch.equal(observed.logits, expected.logits))
        self.assertEqual(
            observed.logits.argmax(dim=-1).tolist(),
            expected.logits.argmax(dim=-1).tolist(),
        )
        self.assertIsNotNone(observed.loss)
        assert observed.loss is not None
        observed.loss.backward()
        for name, parameter in model.named_parameters():
            self.assertIsNotNone(parameter.grad, name)
            assert parameter.grad is not None
            self.assertTrue(torch.isfinite(parameter.grad).all(), name)

    def test_full_incremental_and_step_paths_match(self) -> None:
        model = _model()
        for execution in ("plain", "r4"):
            full = model(self.tokens, execution=execution)
            incremental = model.forward_incremental(
                self.tokens, execution=execution
            )
            self.assertLessEqual(
                float((full.logits - incremental.logits).abs().max()), 2e-5
            )
            self.assertTrue(
                torch.equal(
                    full.logits.argmax(dim=-1), incremental.logits.argmax(dim=-1)
                )
            )
            _assert_state_close(self, full.final_state, incremental.final_state)

            state = model.initial_state(2, execution=execution)
            step_logits = []
            for position in range(self.tokens.shape[1]):
                step = model.step(
                    self.tokens[:, position], state, execution=execution
                )
                state = step.final_state
                step_logits.append(step.logits)
            stacked = torch.stack(step_logits, dim=1)
            self.assertLessEqual(float((full.logits - stacked).abs().max()), 2e-5)
            self.assertTrue(
                torch.equal(full.logits.argmax(dim=-1), stacked.argmax(dim=-1))
            )
            _assert_state_close(self, full.final_state, state)
            self.assertEqual(state.length, self.tokens.shape[1])
            self.assertEqual(state.audit.future_reads, 0)
            self.assertEqual(state.audit.forbidden_reads, 0)

    def test_coherent_r4_matches_plain_weights_logits_and_top1(self) -> None:
        model = _model()
        plain = model(self.tokens, execution="plain")
        r4 = model(self.tokens, execution="r4")
        self.assertLessEqual(
            float((plain.attention_weights - r4.attention_weights).abs().max()),
            2e-6,
        )
        self.assertLessEqual(float((plain.logits - r4.logits).abs().max()), 2e-5)
        self.assertTrue(
            torch.equal(plain.logits.argmax(dim=-1), r4.logits.argmax(dim=-1))
        )
        self.assertEqual(r4.audit.transported_r4_blocks, r4.audit.materialized_attention_scores * 6)
        self.assertEqual(plain.audit.transported_r4_blocks, 0)

    def test_controls_are_causal_destructive_and_audited(self) -> None:
        model = _model()
        native = model(self.tokens, execution="r4")
        current_only = model(
            self.tokens, execution="r4", intervention="current_only"
        )
        value_permuted = model(
            self.tokens, execution="r4", intervention="value_permuted"
        )
        mismatch = model(
            self.tokens, execution="r4", intervention="transport_mismatch"
        )
        self.assertFalse(torch.equal(native.logits[:, 1:], current_only.logits[:, 1:]))
        self.assertFalse(torch.equal(native.logits[:, 1:], value_permuted.logits[:, 1:]))
        self.assertFalse(torch.equal(native.logits[:, 1:], mismatch.logits[:, 1:]))
        for output in (native, current_only, value_permuted, mismatch):
            self.assertEqual(output.audit.source_reads, self.tokens.numel())
            self.assertEqual(output.audit.provider_calls, 0)
            self.assertEqual(output.audit.teacher_calls, 0)
            self.assertEqual(output.audit.future_reads, 0)
            self.assertEqual(output.audit.forbidden_reads, 0)
            self.assertTrue(
                torch.equal(
                    output.final_state.source_frame_indices,
                    native.final_state.source_frame_indices,
                )
            )
        self.assertLess(
            current_only.audit.admitted_attention_scores,
            native.audit.admitted_attention_scores,
        )
        with self.assertRaisesRegex(ValueError, "requires R4"):
            model(self.tokens, execution="plain", intervention="transport_mismatch")

        changed = self.tokens.clone()
        changed[:, 4:] = torch.tensor([[31, 32, 33], [34, 35, 36]])
        changed_output = model(changed, execution="r4")
        self.assertTrue(torch.equal(native.logits[:, :4], changed_output.logits[:, :4]))

        # Repeated leaf addresses remain distinct cache records at each position.
        repeated = torch.tensor([[0, 1, 25, 49]], dtype=torch.long)
        repeated_output = model(repeated, execution="plain")
        self.assertTrue(
            torch.equal(
                repeated_output.final_state.valid[0, 0, :4],
                torch.ones(4, dtype=torch.bool),
            )
        )
        self.assertEqual(repeated_output.final_state.length, 4)
        self.assertFalse(
            torch.equal(
                repeated_output.final_state.keys[0, 0, :, 1, :],
                repeated_output.final_state.keys[0, 0, :, 2, :],
            )
        )


if __name__ == "__main__":
    unittest.main()
