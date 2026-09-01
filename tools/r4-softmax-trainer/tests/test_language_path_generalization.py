"""Focused contract tests for the compact #973 language-path decoders."""

from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    CONTEXT,
    GROUP_SIZE,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
    R4RetainedLanguagePathV1,
    architecture_ledger,
    work_ledger,
)


def _geometry() -> GroupAddressArtifact:
    elements = torch.arange(GROUP_SIZE, dtype=torch.long)
    actions = (elements[:, None] + elements[None, :]) % GROUP_SIZE
    leaves = torch.arange(VOCAB_SIZE, dtype=torch.long) % GROUP_SIZE
    leaves[0] = 0
    return GroupAddressArtifact(
        arm="exact_h4",
        identity_offset=0,
        token_leaves=leaves,
        left_actions=actions,
        artifact_cid="synthetic:language-path-exact-h4",
    )


def _retained() -> R4RetainedLanguagePathV1:
    return R4RetainedLanguagePathV1(_geometry())


class LanguagePathGeneralizationTests(unittest.TestCase):
    def test_exact_parameter_state_and_work_ledgers_match(self) -> None:
        retained = _retained()
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        expected = architecture_ledger("retained")
        self.assertEqual(expected, architecture_ledger("ordinary"))
        self.assertEqual(expected.parameters, PARAMETER_COUNT)
        self.assertEqual(expected.state_values, STATE_VALUES)
        self.assertEqual(expected.state_bytes_f32, STATE_BYTES_F32)
        self.assertEqual(expected.validity_bits, VALIDITY_BITS)
        for model in (retained, ordinary):
            self.assertEqual(model.parameter_count(), PARAMETER_COUNT)
            self.assertEqual(model.state_value_count(), STATE_VALUES)
            self.assertEqual(model.validity_bit_count(), VALIDITY_BITS)
            self.assertEqual(
                model.token_embedding.weight.untyped_storage().data_ptr(),
                model.output_weight.untyped_storage().data_ptr(),
            )
        retained_work = work_ledger("retained", batch_size=16, time=CONTEXT)
        ordinary_work = work_ledger("ordinary", batch_size=16, time=CONTEXT)
        self.assertEqual(
            retained_work.materialized_attention_scores,
            ordinary_work.materialized_attention_scores,
        )
        self.assertEqual(
            retained_work.attention_value_reads, ordinary_work.attention_value_reads
        )
        self.assertEqual(retained_work.vocabulary_scores, ordinary_work.vocabulary_scores)

    def test_ordinary_per_head_score_and_output_gains_are_active(self) -> None:
        model = OrdinaryCausalSoftmaxLanguagePathV1()
        tokens = torch.tensor([[0, 4, 8, 12, 16, 20], [0, 3, 7, 11, 15, 19]])
        targets = torch.tensor([[4, 8, 12, 16, 20, 1], [3, 7, 11, 15, 19, 1]])
        output = model(tokens, targets)
        self.assertIsNotNone(output.loss)
        assert output.loss is not None
        output.loss.backward()
        for layer in model.layers:
            for parameter in (layer.log_score_gains, layer.log_output_gains):
                self.assertIsNotNone(parameter.grad)
                assert parameter.grad is not None
                self.assertTrue(torch.isfinite(parameter.grad).all())
                self.assertEqual(int(torch.count_nonzero(parameter.grad)), parameter.numel())

    def test_common_shape_initialization_is_byte_identical(self) -> None:
        retained = _retained()
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        retained_parameters = dict(retained.named_parameters())
        ordinary_parameters = dict(ordinary.named_parameters())
        retained_only = {
            f"layers.{layer}.{name}"
            for layer in range(2)
            for name in ("decay_logits", "write_logits")
        }
        ordinary_only = {
            f"layers.{layer}.{name}"
            for layer in range(2)
            for name in ("log_score_gains", "log_output_gains")
        }
        common = set(retained_parameters) - retained_only
        self.assertEqual(common, set(ordinary_parameters) - ordinary_only)
        for name in sorted(common):
            self.assertTrue(
                torch.equal(retained_parameters[name], ordinary_parameters[name]), name
            )

    def test_retained_full_direct_and_step_paths_match(self) -> None:
        model = _retained()
        tokens = torch.tensor([[0, 4, 8, 12], [0, 3, 7, 11]])
        stationary = model(tokens)
        direct = model.forward_incremental(tokens)
        self.assertTrue(
            torch.allclose(stationary.logits, direct.logits, atol=2e-6, rtol=2e-5)
        )
        self.assertTrue(
            torch.allclose(
                stationary.final_state.keys,
                direct.final_state.keys,
                atol=2e-6,
                rtol=2e-5,
            )
        )
        self.assertTrue(
            torch.allclose(
                stationary.final_state.values,
                direct.final_state.values,
                atol=2e-6,
                rtol=2e-5,
            )
        )
        self.assertTrue(
            torch.equal(stationary.final_state.occupied, direct.final_state.occupied)
        )

        state = model.initial_state(tokens.shape[0])
        step_logits = []
        for column in range(tokens.shape[1]):
            step = model.step(tokens[:, column], state)
            state = step.final_state
            step_logits.append(step.logits)
        self.assertTrue(
            torch.allclose(
                stationary.logits,
                torch.stack(step_logits, dim=1),
                atol=2e-6,
                rtol=2e-5,
            )
        )
        self.assertTrue(
            torch.allclose(stationary.final_state.keys, state.keys, atol=2e-6, rtol=2e-5)
        )
        self.assertTrue(
            torch.allclose(
                stationary.final_state.values, state.values, atol=2e-6, rtol=2e-5
            )
        )
        self.assertTrue(torch.equal(stationary.final_state.occupied, state.occupied))

        stationary_off = model(tokens, attention_off=True)
        direct_off = model.forward_incremental(tokens, attention_off=True)
        self.assertTrue(
            torch.allclose(
                stationary_off.logits, direct_off.logits, atol=2e-6, rtol=2e-5
            )
        )

    def test_ordinary_logits_are_strictly_causal(self) -> None:
        model = OrdinaryCausalSoftmaxLanguagePathV1()
        original = torch.tensor([[0, 4, 8, 12, 16, 20]])
        changed_future = torch.tensor([[0, 4, 8, 30, 31, 32]])
        first_targets = torch.tensor([[4, 8, 12, 16, 20, 1]])
        changed_targets = torch.tensor([[9, 8, 7, 6, 5, 4]])
        first = model(original, first_targets)
        future = model(changed_future, first_targets)
        targets = model(original, changed_targets)
        self.assertTrue(torch.equal(first.logits[:, :3], future.logits[:, :3]))
        self.assertTrue(torch.equal(first.logits, targets.logits))

    def test_both_arms_forward_and_attention_off_are_finite(self) -> None:
        tokens = torch.tensor([[0, 4, 8, 12], [0, 3, 7, 11]])
        targets = torch.tensor([[4, 8, 12, 1], [3, 7, 11, 1]])
        retained = _retained()
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        for model in (retained, ordinary):
            enabled = model(tokens, targets)
            disabled = model(tokens, targets, attention_off=True)
            self.assertEqual(tuple(enabled.logits.shape), (2, 4, VOCAB_SIZE))
            self.assertTrue(torch.isfinite(enabled.logits).all())
            self.assertIsNotNone(enabled.loss)
            self.assertFalse(torch.equal(enabled.logits[:, 1:], disabled.logits[:, 1:]))
            self.assertEqual(enabled.audit.work_signature(), disabled.audit.work_signature())


if __name__ == "__main__":
    unittest.main()
