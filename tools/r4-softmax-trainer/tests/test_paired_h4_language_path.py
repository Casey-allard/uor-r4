"""Focused tests for the layer-paired H4 retained-language successor."""

from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    GROUP_SIZE,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)
from r4_softmax_trainer.paired_h4_language_path import (
    CANONICAL_IDENTITY_INDEX,
    R4PairedH4LanguagePathV1,
    canonical_layer_token_leaves,
    joint_prefix_collision_census,
)


def _canonical_identity_geometry() -> GroupAddressArtifact:
    elements = torch.arange(GROUP_SIZE, dtype=torch.long)
    # Relabel the cyclic group so index 119 is its exact identity.  The tests
    # need only a complete associative permutation action; production supplies
    # the canonical noncommutative H4 action table under the same interface.
    actions = (elements[:, None] + elements[None, :] + 1) % GROUP_SIZE
    leaves = torch.empty(VOCAB_SIZE, dtype=torch.long)
    leaves[0] = CANONICAL_IDENTITY_INDEX
    leaves[1:] = (torch.arange(1, VOCAB_SIZE, dtype=torch.long) - 1).remainder(
        GROUP_SIZE
    )
    return GroupAddressArtifact(
        arm="exact_h4",
        identity_offset=CANONICAL_IDENTITY_INDEX,
        token_leaves=leaves,
        left_actions=actions,
        artifact_cid="synthetic:canonical-identity-h4-action",
    )


def _tokens_and_targets() -> tuple[torch.Tensor, torch.Tensor]:
    tokens = torch.tensor(
        [
            [0, 1, 121, 240, 4_095, 17],
            [0, 120, 241, 361, 481, 2],
        ],
        dtype=torch.long,
    )
    targets = torch.tensor(
        [
            [1, 121, 240, 4_095, 17, 1],
            [120, 241, 361, 481, 2, 1],
        ],
        dtype=torch.long,
    )
    return tokens, targets


class PairedH4LanguagePathTests(unittest.TestCase):
    def test_canonical_codebook_is_injective_with_exact_support(self) -> None:
        leaves = canonical_layer_token_leaves()
        self.assertEqual(tuple(leaves.shape), (2, VOCAB_SIZE))
        self.assertTrue(
            torch.equal(
                leaves[:, 0],
                torch.tensor(
                    [CANONICAL_IDENTITY_INDEX, CANONICAL_IDENTITY_INDEX]
                ),
            )
        )
        self.assertEqual(tuple(leaves[:, 1].tolist()), (0, 0))
        self.assertEqual(tuple(leaves[:, 120].tolist()), (119, 0))
        self.assertEqual(tuple(leaves[:, 121].tolist()), (0, 1))
        self.assertEqual(tuple(leaves[:, 4_095].tolist()), (14, 34))
        joint = leaves[0] * GROUP_SIZE + leaves[1]
        self.assertEqual(int(torch.unique(joint).numel()), VOCAB_SIZE)
        self.assertEqual(int(torch.unique(leaves[0]).numel()), 120)
        self.assertEqual(int(torch.unique(leaves[1]).numel()), 36)

    def test_joint_collision_census_separates_correlated_from_paired_routes(
        self,
    ) -> None:
        geometry = _canonical_identity_geometry()
        tokens = torch.tensor(
            [[1, 119, 1, 119], [2, 118, 2, 118]], dtype=torch.long
        )
        correlated = torch.stack((geometry.token_leaves, geometry.token_leaves))
        old = joint_prefix_collision_census(
            tokens,
            layer_token_leaves=correlated,
            left_actions=geometry.left_actions,
            identity_index=geometry.identity_offset,
        )
        paired = joint_prefix_collision_census(
            tokens,
            layer_token_leaves=canonical_layer_token_leaves(),
            left_actions=geometry.left_actions,
            identity_index=geometry.identity_offset,
        )
        self.assertEqual(old.repeats_per_sequence, (2, 2))
        self.assertEqual(old.repeated_joint_addresses, 4)
        self.assertEqual(old.collision_free_sequences, 0)
        self.assertEqual(old.mean_repeated_joint_addresses, 2.0)
        self.assertEqual(paired.repeats_per_sequence, (0, 0))
        self.assertEqual(paired.repeated_joint_addresses, 0)
        self.assertEqual(paired.collision_free_sequences, 2)

    def test_successor_preserves_ledgers_initialization_and_v1_behavior(self) -> None:
        geometry = _canonical_identity_geometry()
        before = R4RetainedLanguagePathV1(geometry)
        paired = R4PairedH4LanguagePathV1(geometry)
        after = R4RetainedLanguagePathV1(geometry)

        self.assertEqual(paired.parameter_count(), PARAMETER_COUNT)
        self.assertEqual(paired.state_value_count(), STATE_VALUES)
        self.assertEqual(paired.validity_bit_count(), VALIDITY_BITS)
        self.assertNotIn("layer_token_leaves", dict(paired.named_parameters()))
        self.assertTrue(torch.equal(before.token_leaves, geometry.token_leaves))
        self.assertTrue(torch.equal(after.token_leaves, geometry.token_leaves))

        before_parameters = dict(before.named_parameters())
        paired_parameters = dict(paired.named_parameters())
        after_parameters = dict(after.named_parameters())
        self.assertEqual(set(before_parameters), set(paired_parameters))
        for name in sorted(before_parameters):
            self.assertTrue(
                torch.equal(before_parameters[name], paired_parameters[name]), name
            )
            self.assertTrue(
                torch.equal(before_parameters[name], after_parameters[name]), name
            )

        tokens, _ = _tokens_and_targets()
        self.assertTrue(torch.equal(before(tokens).logits, after(tokens).logits))
        self.assertFalse(torch.equal(before(tokens).logits, paired(tokens).logits))

    def test_stationary_direct_step_state_off_and_gradients(self) -> None:
        model = R4PairedH4LanguagePathV1(_canonical_identity_geometry())
        tokens, targets = _tokens_and_targets()

        stationary = model(tokens, targets)
        direct = model.forward_incremental(tokens, targets)
        self.assertIsNotNone(stationary.loss)
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
        for position in range(tokens.shape[1]):
            step = model.step(tokens[:, position], state)
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
            torch.allclose(
                stationary.final_state.keys, state.keys, atol=2e-6, rtol=2e-5
            )
        )
        self.assertTrue(
            torch.allclose(
                stationary.final_state.values, state.values, atol=2e-6, rtol=2e-5
            )
        )
        self.assertTrue(torch.equal(stationary.final_state.occupied, state.occupied))

        disabled = model(tokens, targets, attention_off=True)
        self.assertTrue(torch.isfinite(disabled.logits).all())
        self.assertFalse(torch.equal(stationary.logits[:, 1:], disabled.logits[:, 1:]))
        self.assertEqual(
            stationary.audit.work_signature(), disabled.audit.work_signature()
        )

        assert stationary.loss is not None
        stationary.loss.backward()
        for name, parameter in model.named_parameters():
            self.assertIsNotNone(parameter.grad, name)
            assert parameter.grad is not None
            self.assertTrue(torch.isfinite(parameter.grad).all(), name)
            self.assertGreater(int(torch.count_nonzero(parameter.grad)), 0, name)

    def test_learned_artifact_reloads_without_serializing_the_codebook(self) -> None:
        geometry = _canonical_identity_geometry()
        source = R4PairedH4LanguagePathV1(geometry)
        payload = source.export_learned_artifact()
        expected = {
            name: parameter.detach().clone()
            for name, parameter in source.named_parameters()
        }
        with torch.no_grad():
            for parameter in source.parameters():
                parameter.add_(1.0)
        source.load_learned_artifact(payload)
        for name, parameter in source.named_parameters():
            self.assertTrue(torch.equal(parameter, expected[name]), name)
        self.assertTrue(
            torch.equal(source.layer_token_leaves, canonical_layer_token_leaves())
        )


if __name__ == "__main__":
    unittest.main()
