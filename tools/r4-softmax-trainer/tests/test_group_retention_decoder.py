"""Focused contract tests for ``R4GroupAddressedRetentionDecoderV1``."""

from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.group_retention_decoder import (
    POLICY,
    PRODUCTION_OCCUPANCY_BITS,
    PRODUCTION_PARAMETER_COUNT,
    PRODUCTION_STATE_BYTES_F32,
    PRODUCTION_STATE_VALUES,
    DecoderConfig,
    R4GroupAddressedRetentionDecoderV1,
    expected_occupancy_bit_count,
    expected_parameter_count,
    expected_state_value_count,
)
from r4_softmax_trainer.group_retention_decoder import _RetainedDecoderBlock


def _actions(group_size: int) -> torch.Tensor:
    elements = torch.arange(group_size, dtype=torch.long)
    return (elements[:, None] + elements[None, :]) % group_size


def _artifact(
    *, vocab_size: int, group_size: int, arm: str = "exact_h4"
) -> GroupAddressArtifact:
    actions = _actions(group_size)
    if arm == "scrambled_h4":
        if group_size != 5:
            raise ValueError("the focused noncommuting control is frozen at group size five")
        actions = torch.tensor(
            [
                [0, 1, 2, 3, 4],
                [1, 0, 2, 3, 4],
                [0, 2, 3, 1, 4],
                [4, 1, 2, 3, 0],
                [0, 1, 3, 4, 2],
            ],
            dtype=torch.long,
        )
    leaves = torch.arange(vocab_size, dtype=torch.long) % group_size
    leaves[0] = 0
    return GroupAddressArtifact(
        arm=arm,
        identity_offset=0,
        token_leaves=leaves,
        left_actions=actions,
        artifact_cid=f"synthetic:{arm}:{vocab_size}:{group_size}",
    )


def _config() -> DecoderConfig:
    return DecoderConfig(
        vocab_size=13,
        hidden_size=8,
        intermediate_size=16,
        layers=2,
        heads=2,
        head_dim=4,
        group_size=5,
        max_sequence_length=8,
        initialization_seed=9737,
        decay_half_lives=(2.0, 8.0),
    )


def _model(*, arm: str = "exact_h4") -> R4GroupAddressedRetentionDecoderV1:
    config = _config()
    return R4GroupAddressedRetentionDecoderV1(
        config,
        _artifact(vocab_size=config.vocab_size, group_size=config.group_size, arm=arm),
    )


def _assert_state_close(
    subject: unittest.TestCase, first, second, *, atol: float = 2e-6
) -> None:
    subject.assertTrue(torch.allclose(first.keys, second.keys, atol=atol, rtol=2e-5))
    subject.assertTrue(torch.allclose(first.values, second.values, atol=atol, rtol=2e-5))
    subject.assertTrue(torch.equal(first.occupied, second.occupied))


class GroupRetentionDecoderTests(unittest.TestCase):
    def test_mps_native_prefix_max_scan_matches_cpu_cummax(self) -> None:
        generator = torch.Generator().manual_seed(973)
        for time in (1, 2, 3, 7, 8, 13):
            values = torch.randint(
                -1, time + 2, (3, time, 5), generator=generator, dtype=torch.long
            )
            expected = torch.cummax(values, dim=1).values
            observed = _RetainedDecoderBlock._inclusive_prefix_max(values)
            self.assertTrue(torch.equal(observed, expected), time)

    def test_production_contract_has_exact_parameters_state_and_tied_storage(self) -> None:
        config = DecoderConfig.production()
        self.assertEqual(POLICY, "R4GroupAddressedRetentionDecoderV1")
        self.assertEqual(expected_parameter_count(config), PRODUCTION_PARAMETER_COUNT)
        self.assertEqual(expected_state_value_count(config), PRODUCTION_STATE_VALUES)
        self.assertEqual(PRODUCTION_STATE_VALUES * 4, PRODUCTION_STATE_BYTES_F32)
        self.assertEqual(expected_occupancy_bit_count(config), PRODUCTION_OCCUPANCY_BITS)

        leaves = torch.arange(config.vocab_size, dtype=torch.long) % 35
        leaves[0] = 0
        geometry = GroupAddressArtifact(
            arm="exact_h4",
            identity_offset=0,
            token_leaves=leaves,
            left_actions=_actions(config.group_size),
            artifact_cid="synthetic:production",
        )
        model = R4GroupAddressedRetentionDecoderV1.production(geometry)
        self.assertEqual(model.parameter_count(), PRODUCTION_PARAMETER_COUNT)
        self.assertEqual(model.state_value_count(), PRODUCTION_STATE_VALUES)
        self.assertEqual(model.occupancy_bit_count(), PRODUCTION_OCCUPANCY_BITS)
        self.assertEqual(
            model.token_embedding.weight.untyped_storage().data_ptr(),
            model.output_weight.untyped_storage().data_ptr(),
        )

    def test_stationary_matches_direct_logits_final_state_and_gradients(self) -> None:
        tokens = torch.tensor([[1, 2, 3, 4, 5], [4, 2, 1, 3, 6]])
        targets = torch.tensor([[2, 3, 4, 5, 6], [2, 1, 3, 6, 7]])
        stationary = _model()
        direct = _model()
        generator = torch.Generator().manual_seed(1973)
        initial = stationary.initial_state(2)
        initial.keys.copy_(torch.randn(initial.keys.shape, generator=generator))
        initial.values.copy_(torch.randn(initial.values.shape, generator=generator))
        initial.occupied[:, :, 0] = True
        initial.occupied[:, :, 3] = True
        direct_initial = type(initial)(
            keys=initial.keys.clone(),
            values=initial.values.clone(),
            occupied=initial.occupied.clone(),
        )
        stationary_output = stationary(
            tokens, targets, initial_state=initial, implementation="stationary"
        )
        direct_output = direct(
            tokens, targets, initial_state=direct_initial, implementation="direct"
        )

        self.assertTrue(
            torch.allclose(stationary_output.logits, direct_output.logits, atol=2e-6, rtol=2e-5)
        )
        self.assertIsNotNone(stationary_output.loss)
        self.assertIsNotNone(direct_output.loss)
        assert stationary_output.loss is not None and direct_output.loss is not None
        self.assertTrue(torch.allclose(stationary_output.loss, direct_output.loss, atol=2e-6))
        _assert_state_close(self, stationary_output.final_state, direct_output.final_state)
        self.assertEqual(
            stationary_output.audit.work_signature(), direct_output.audit.work_signature()
        )

        stationary_output.loss.backward()
        direct_output.loss.backward()
        direct_parameters = dict(direct.named_parameters())
        for name, parameter in stationary.named_parameters():
            other = direct_parameters[name]
            self.assertIsNotNone(parameter.grad, name)
            self.assertIsNotNone(other.grad, name)
            assert parameter.grad is not None and other.grad is not None
            self.assertTrue(torch.isfinite(parameter.grad).all(), name)
            self.assertGreater(int(torch.count_nonzero(parameter.grad)), 0, name)
            self.assertTrue(
                torch.allclose(parameter.grad, other.grad, atol=5e-6, rtol=2e-4), name
            )

    def test_incremental_steps_match_stationary_and_first_empty_read_is_exact_zero(self) -> None:
        model = _model()
        tokens = torch.tensor([[1, 2, 3, 4], [2, 3, 1, 5]])
        stationary = model(tokens)
        state = model.initial_state(2)
        step_logits = []
        for column in range(tokens.shape[1]):
            step = model.step(tokens[:, column], state)
            state = step.final_state
            step_logits.append(step.logits)
        incremental_logits = torch.stack(step_logits, dim=1)
        self.assertTrue(torch.allclose(stationary.logits, incremental_logits, atol=2e-6, rtol=2e-5))
        _assert_state_close(self, stationary.final_state, state)

        first_on = model(tokens[:, :1])
        first_off = model(tokens[:, :1], state_off=True)
        self.assertTrue(torch.equal(first_on.logits, first_off.logits))
        self.assertTrue(torch.isfinite(first_on.logits).all())
        self.assertEqual(int(first_on.final_state.occupied.sum()), 2 * model.config.layers)

    def test_logits_are_strictly_causal_and_targets_do_not_change_them(self) -> None:
        model = _model()
        original = torch.tensor([[1, 2, 3, 4, 5]])
        changed_future = torch.tensor([[1, 2, 3, 9, 8]])
        first_targets = torch.tensor([[2, 3, 4, 5, 6]])
        changed_targets = torch.tensor([[9, 8, 7, 6, 5]])
        first = model(original, first_targets)
        future = model(changed_future, first_targets)
        targets = model(original, changed_targets)
        self.assertTrue(torch.equal(first.logits[:, :3], future.logits[:, :3]))
        self.assertTrue(torch.equal(first.logits, targets.logits))

    def test_state_off_preserves_state_and_work_but_removes_retained_residual(self) -> None:
        model = _model()
        tokens = torch.tensor([[1, 2, 3, 4, 5], [2, 4, 1, 3, 6]])
        enabled = model(tokens)
        disabled = model(tokens, state_off=True)
        self.assertEqual(enabled.audit.work_signature(), disabled.audit.work_signature())
        self.assertFalse(torch.allclose(enabled.logits[:, 1:], disabled.logits[:, 1:]))
        # The first layer writes from the shared token embedding and is exactly
        # equal.  The second layer correctly receives the intervened residual,
        # so equal work does not imply an incorrectly frozen deeper state.
        self.assertTrue(
            torch.allclose(
                enabled.final_state.keys[0], disabled.final_state.keys[0], atol=2e-6
            )
        )
        self.assertFalse(
            torch.allclose(enabled.final_state.keys[1], disabled.final_state.keys[1])
        )
        self.assertTrue(torch.equal(enabled.final_state.occupied, disabled.final_state.occupied))
        self.assertFalse(enabled.audit.state_off)
        self.assertTrue(disabled.audit.state_off)

    def test_geometry_arms_have_byte_identical_initialization_and_equal_ledgers(self) -> None:
        exact = _model(arm="exact_h4")
        scrambled = _model(arm="scrambled_h4")
        self.assertEqual(exact.export_learned_artifact(), scrambled.export_learned_artifact())
        tokens = torch.tensor([[1, 2, 3, 4], [4, 3, 2, 1]])
        exact_output = exact(tokens)
        scrambled_output = scrambled(tokens)
        scrambled_direct = scrambled(tokens, implementation="direct")
        self.assertEqual(
            exact_output.audit.work_signature(), scrambled_output.audit.work_signature()
        )
        self.assertTrue(
            torch.allclose(
                scrambled_output.logits, scrambled_direct.logits, atol=2e-6, rtol=2e-5
            )
        )
        _assert_state_close(
            self, scrambled_output.final_state, scrambled_direct.final_state
        )
        self.assertEqual(exact.parameter_count(), scrambled.parameter_count())
        self.assertEqual(exact.state_value_count(), scrambled.state_value_count())

    def test_learned_artifact_export_and_load_are_deterministic(self) -> None:
        source = _model()
        payload = source.export_learned_artifact()
        self.assertEqual(payload, source.export_learned_artifact())
        destination = _model()
        with torch.no_grad():
            destination.token_embedding.weight.add_(1.0)
        geometry_before = destination.left_actions.clone()
        destination.load_learned_artifact(payload)
        self.assertEqual(payload, destination.export_learned_artifact())
        self.assertTrue(torch.equal(geometry_before, destination.left_actions))
        tokens = torch.tensor([[1, 2, 3, 4]])
        self.assertTrue(torch.equal(source(tokens).logits, destination(tokens).logits))


if __name__ == "__main__":
    unittest.main()
