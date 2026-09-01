"""Focused checks for ``R4GroupAddressedRetentionLMV1``."""

from __future__ import annotations

import math
import unittest

import torch

from r4_softmax_trainer.group_retention import (
    LEARNED_PARAMETER_NAMES,
    POLICY,
    PRODUCTION_BANKS,
    PRODUCTION_GROUP_SIZE,
    PRODUCTION_HIDDEN_SIZE,
    PRODUCTION_PARAMETER_COUNT,
    PRODUCTION_STATE_BYTES_F32,
    PRODUCTION_STATE_VALUES,
    PRODUCTION_VOCAB_SIZE,
    GroupAddressArtifact,
    GroupRetentionConfig,
    R4GroupAddressedRetentionLMV1,
    expected_parameter_count,
    expected_state_value_count,
)


def _cyclic_actions(group_size: int) -> torch.Tensor:
    elements = torch.arange(group_size, dtype=torch.long)
    return (elements[:, None] + elements[None, :]) % group_size


def _artifact(
    *,
    vocab_size: int,
    group_size: int,
    arm: str = "exact_h4",
    row_permutation: torch.Tensor | None = None,
) -> GroupAddressArtifact:
    leaves = torch.arange(vocab_size, dtype=torch.long) % group_size
    leaves[0] = 0
    actions = _cyclic_actions(group_size)
    if row_permutation is not None:
        actions = actions.index_select(0, row_permutation)
    return GroupAddressArtifact(
        arm=arm,
        identity_offset=0,
        token_leaves=leaves,
        left_actions=actions,
        artifact_cid=f"synthetic:{arm}:{group_size}:{vocab_size}",
    )


def _config(
    *,
    vocab_size: int = 7,
    hidden_size: int = 8,
    group_size: int = 4,
    banks: int = 2,
    context: int = 32,
) -> GroupRetentionConfig:
    return GroupRetentionConfig(
        vocab_size=vocab_size,
        hidden_size=hidden_size,
        group_size=group_size,
        banks=banks,
        max_sequence_length=context,
        initialization_seed=9736,
    )


def _logit(probability: float) -> float:
    return math.log(probability / (1.0 - probability))


class GroupRetentionTests(unittest.TestCase):
    def test_supplied_geometry_is_validated_without_constructing_h4(self) -> None:
        artifact = _artifact(vocab_size=7, group_size=4)
        artifact.validate(group_size=4, vocab_size=7, max_candidate_leaves=4)
        self.assertEqual(artifact.direct_leaf_count, 4)

        invalid = artifact.left_actions.clone()
        invalid[1, 0] = invalid[1, 1]
        with self.assertRaisesRegex(ValueError, "complete permutation"):
            GroupAddressArtifact(
                arm="exact_h4",
                identity_offset=0,
                token_leaves=artifact.token_leaves,
                left_actions=invalid,
            ).validate(group_size=4, vocab_size=7)

        with self.assertRaisesRegex(ValueError, "frozen bound"):
            artifact.validate(group_size=4, vocab_size=7, max_candidate_leaves=3)

    def test_one_step_matches_the_frozen_equations_exactly(self) -> None:
        config = _config(vocab_size=5, hidden_size=4, group_size=3, banks=2)
        artifact = _artifact(vocab_size=5, group_size=3)
        model = R4GroupAddressedRetentionLMV1(config, artifact)
        with torch.no_grad():
            model.query_table.copy_(
                torch.tensor(
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                        [1.0, -1.0, 0.5, 0.25],
                    ]
                )
            )
            model.value_table.copy_(
                torch.tensor(
                    [
                        [0.0, 0.0, 0.0, 0.0],
                        [2.0, -1.0, 0.5, 1.0],
                        [1.0, 1.0, 1.0, 1.0],
                        [-1.0, 2.0, 0.0, 0.5],
                        [0.25, 0.5, 0.75, 1.0],
                    ]
                )
            )
            model.decay_logits.copy_(torch.tensor([_logit(0.5), _logit(0.75)]))
            model.write_logits.copy_(torch.tensor([_logit(0.25), _logit(0.6)]))
            model.bank_logits.copy_(torch.log(torch.tensor([0.3, 0.7])))

        initial = torch.arange(24, dtype=torch.float32).view(1, 2, 3, 4) / 10.0
        output = model(torch.tensor([[1]]), initial_state=initial)

        action = artifact.left_actions[artifact.token_leaves[1]]
        recentered = initial.index_select(2, action)
        rho = torch.tensor([0.5, 0.75]).view(1, 2, 1, 1)
        eta = torch.tensor([0.25, 0.6]).view(1, 2, 1)
        expected_state = recentered * rho
        prior_identity = expected_state[:, :, 0, :]
        replacement = prior_identity + eta * (model.value_table[1].view(1, 1, 4) - prior_identity)
        expected_state = expected_state.clone()
        expected_state[:, :, 0, :] = replacement

        alpha = torch.tensor([0.3, 0.7])
        aggregate = torch.einsum("k,nkgd->ngd", alpha, expected_state)
        expected_logits = []
        for candidate in range(config.vocab_size):
            query = model.query_table[candidate]
            leaf = int(artifact.token_leaves[candidate])
            expected_logits.append(
                (
                    torch.dot(query, model.value_table[1])
                    + torch.dot(query, aggregate[0, leaf])
                )
                / math.sqrt(config.hidden_size)
            )
        expected_logits_tensor = torch.stack(expected_logits).view(1, 1, -1)

        self.assertTrue(torch.allclose(output.final_state, expected_state, atol=1e-6, rtol=1e-6))
        self.assertTrue(torch.allclose(output.logits, expected_logits_tensor, atol=1e-6, rtol=1e-6))
        self.assertEqual(output.audit.forbidden_reads, 0)

    def test_prefix_order_is_live_and_state_off_preserves_work(self) -> None:
        config = _config(vocab_size=7, hidden_size=4, group_size=4, banks=1)
        artifact = _artifact(vocab_size=7, group_size=4)
        model = R4GroupAddressedRetentionLMV1(config, artifact)
        with torch.no_grad():
            model.query_table.zero_()
            model.query_table[:4].copy_(torch.eye(4))
            model.query_table[4:].copy_(torch.eye(4)[:3])
            model.value_table.zero_()
            model.value_table[1, 0] = 1.0
            model.value_table[2, 1] = 2.0
            model.value_table[3, 2] = 3.0
            model.decay_logits.fill_(12.0)
            model.write_logits.fill_(12.0)
            model.bank_logits.zero_()

        first = model(torch.tensor([[1, 2, 3]]))
        second = model(torch.tensor([[2, 1, 3]]))
        self.assertFalse(torch.allclose(first.logits[:, -1], second.logits[:, -1]))

        disabled = model(torch.tensor([[1, 2, 3]]), state_off=True)
        expected_current = torch.nn.functional.linear(model.value_table[3], model.query_table)
        expected_current = expected_current / math.sqrt(config.hidden_size)
        self.assertTrue(torch.allclose(disabled.logits[0, -1], expected_current, atol=1e-6))
        self.assertTrue(torch.allclose(first.final_state, disabled.final_state))
        self.assertEqual(first.audit.work_signature(), disabled.audit.work_signature())
        self.assertNotEqual(first.audit.state_off, disabled.audit.state_off)

    def test_last_position_path_matches_full_causal_scoring_with_less_work(self) -> None:
        config = _config(vocab_size=11, hidden_size=8, group_size=5, banks=2, context=8)
        model = R4GroupAddressedRetentionLMV1(
            config,
            _artifact(vocab_size=config.vocab_size, group_size=config.group_size),
        )
        tokens = torch.tensor([[1, 2, 3, 4, 1, 2], [4, 3, 2, 1, 3, 4]])
        full = model(tokens)
        last = model.score_last(tokens)
        last_off = model.score_last(tokens, state_off=True)

        self.assertEqual(tuple(last.logits.shape), (2, config.vocab_size))
        self.assertTrue(torch.allclose(last.logits, full.logits[:, -1], atol=1e-7, rtol=1e-6))
        self.assertEqual(last.audit.token_steps, 12)
        self.assertEqual(last.audit.current_candidate_dot_products, 2 * config.vocab_size)
        self.assertEqual(last.audit.retained_candidate_dot_products, 2 * config.vocab_size)
        self.assertEqual(last.audit.work_signature(), last_off.audit.work_signature())
        self.assertFalse(torch.allclose(last.logits, last_off.logits))

    def test_logits_are_prefix_causal_and_target_invariant(self) -> None:
        config = _config(vocab_size=11, hidden_size=8, group_size=5, banks=2, context=8)
        model = R4GroupAddressedRetentionLMV1(
            config,
            _artifact(vocab_size=config.vocab_size, group_size=config.group_size),
        )
        first_tokens = torch.tensor([[1, 2, 3, 4, 5]])
        changed_future = torch.tensor([[1, 2, 3, 9, 8]])
        first_targets = torch.tensor([[2, 3, 4, 5, 6]])
        changed_targets = torch.tensor([[9, 8, 7, 6, 5]])

        first = model(first_tokens, first_targets)
        future = model(changed_future, first_targets)
        targets = model(first_tokens, changed_targets)
        self.assertTrue(torch.equal(first.logits[:, :3], future.logits[:, :3]))
        self.assertTrue(torch.equal(first.logits, targets.logits))
        self.assertEqual(first.audit.forbidden_reads, 0)
        self.assertEqual(targets.audit.forbidden_reads, 0)
        self.assertGreaterEqual(
            first.audit.retained_executed_dot_products,
            first.audit.retained_candidate_dot_products,
        )

    def test_supplied_arm_permutation_changes_the_retained_path(self) -> None:
        config = _config(vocab_size=8, hidden_size=8, group_size=4, banks=2)
        exact_artifact = _artifact(vocab_size=8, group_size=4, arm="exact_h4")
        permutation = torch.tensor([0, 2, 1, 3], dtype=torch.long)
        scrambled_artifact = _artifact(
            vocab_size=8,
            group_size=4,
            arm="scrambled_h4",
            row_permutation=permutation,
        )
        exact = R4GroupAddressedRetentionLMV1(config, exact_artifact)
        scrambled = R4GroupAddressedRetentionLMV1(config, scrambled_artifact)
        scrambled.load_learned_artifact_(exact.export_learned_artifact())

        token_ids = torch.tensor([[1, 2, 3, 1]])
        exact_output = exact(token_ids)
        scrambled_output = scrambled(token_ids)
        self.assertTrue(torch.allclose(exact_output.logits[:, 0], scrambled_output.logits[:, 0]))
        self.assertFalse(torch.allclose(exact_output.logits[:, -1], scrambled_output.logits[:, -1]))
        self.assertFalse(torch.allclose(exact_output.final_state, scrambled_output.final_state))

    def test_parameter_state_counts_deterministic_export_and_full_vocab_output(self) -> None:
        self.assertEqual(POLICY, "R4GroupAddressedRetentionLMV1")
        production_config = GroupRetentionConfig.production()
        self.assertEqual(expected_parameter_count(production_config), PRODUCTION_PARAMETER_COUNT)
        self.assertEqual(expected_state_value_count(production_config), PRODUCTION_STATE_VALUES)
        self.assertEqual(PRODUCTION_STATE_VALUES * 4, PRODUCTION_STATE_BYTES_F32)
        self.assertEqual(
            (PRODUCTION_GROUP_SIZE, PRODUCTION_HIDDEN_SIZE, PRODUCTION_BANKS, PRODUCTION_VOCAB_SIZE),
            (120, 288, 4, 4096),
        )

        group = torch.arange(PRODUCTION_GROUP_SIZE, dtype=torch.long)
        production_leaves = torch.zeros(PRODUCTION_VOCAB_SIZE, dtype=torch.long)
        production_leaves[1:] = (torch.arange(1, PRODUCTION_VOCAB_SIZE) - 1) % 34 + 1
        production_artifact = GroupAddressArtifact(
            arm="cyclic_120",
            identity_offset=0,
            token_leaves=production_leaves,
            left_actions=(group[:, None] + group[None, :]) % PRODUCTION_GROUP_SIZE,
            artifact_cid="synthetic:production-shape",
        )
        production = R4GroupAddressedRetentionLMV1.production(production_artifact)
        self.assertEqual(production.parameter_count(), PRODUCTION_PARAMETER_COUNT)
        self.assertEqual(production.state_value_count(), PRODUCTION_STATE_VALUES)
        self.assertEqual(production.candidate_leaf_group_count, 35)
        production_output = production(torch.tensor([[1]]))
        self.assertEqual(tuple(production_output.logits.shape), (1, 1, PRODUCTION_VOCAB_SIZE))

        config = _config()
        artifact = _artifact(vocab_size=config.vocab_size, group_size=config.group_size)
        first = R4GroupAddressedRetentionLMV1(config, artifact)
        second = R4GroupAddressedRetentionLMV1(config, artifact)
        first_export = first.export_learned_artifact()
        second_export = second.export_learned_artifact()
        self.assertEqual(set(first_export), set(LEARNED_PARAMETER_NAMES))
        for name in LEARNED_PARAMETER_NAMES:
            self.assertTrue(first_export[name].is_contiguous())
            self.assertEqual(first_export[name].device.type, "cpu")
            self.assertTrue(torch.equal(first_export[name], second_export[name]))

        with torch.no_grad():
            second.query_table.add_(1.0)
        second.load_learned_artifact_(first_export)
        for name in LEARNED_PARAMETER_NAMES:
            self.assertTrue(torch.equal(first_export[name], second.export_learned_artifact()[name]))

    def test_gradients_cross_recenter_write_read_and_full_vocab_logits(self) -> None:
        config = _config(vocab_size=9, hidden_size=8, group_size=4, banks=3)
        model = R4GroupAddressedRetentionLMV1(
            config,
            _artifact(vocab_size=config.vocab_size, group_size=config.group_size),
        )
        token_ids = torch.tensor([[1, 2, 3, 4], [2, 1, 4, 3]])
        targets = torch.tensor([[2, 3, 4, 5], [1, 4, 3, 6]])
        initial_state = torch.randn(2, 3, 4, 8, requires_grad=True)
        output = model(token_ids, targets, initial_state=initial_state)
        self.assertEqual(tuple(output.logits.shape), (2, 4, config.vocab_size))
        self.assertIsNotNone(output.loss)
        assert output.loss is not None
        output.loss.backward()

        self.assertIsNotNone(initial_state.grad)
        assert initial_state.grad is not None
        self.assertGreater(float(initial_state.grad.abs().sum()), 0.0)
        for name in LEARNED_PARAMETER_NAMES:
            gradient = getattr(model, name).grad
            self.assertIsNotNone(gradient, name)
            assert gradient is not None
            self.assertTrue(bool(torch.isfinite(gradient).all()), name)
            self.assertGreater(float(gradient.abs().sum()), 0.0, name)

    def test_stationary_closed_form_matches_direct_recenter_recurrence(self) -> None:
        config = _config(vocab_size=13, hidden_size=8, group_size=5, banks=3, context=12)
        # Reorder non-identity action rows so this check covers supplied
        # permutation actions without assuming their row labels multiply.
        row_permutation = torch.tensor([0, 3, 1, 4, 2], dtype=torch.long)
        artifact = _artifact(
            vocab_size=config.vocab_size,
            group_size=config.group_size,
            arm="scrambled_h4",
            row_permutation=row_permutation,
        )
        closed = R4GroupAddressedRetentionLMV1(config, artifact)
        direct = R4GroupAddressedRetentionLMV1(config, artifact)
        direct.load_learned_artifact_(closed.export_learned_artifact())
        tokens = torch.tensor(
            [[1, 2, 1, 4, 3, 2, 6], [4, 1, 3, 1, 2, 4, 1]], dtype=torch.long
        )
        targets = torch.tensor(
            [[2, 1, 4, 3, 2, 6, 5], [1, 3, 1, 2, 4, 1, 7]], dtype=torch.long
        )
        closed_initial = torch.randn(2, 3, 5, 8, requires_grad=True)
        direct_initial = closed_initial.detach().clone().requires_grad_(True)

        closed_output = closed(tokens, targets, initial_state=closed_initial)
        coefficients = direct.resolved_coefficients()
        direct_state = direct_initial
        addressed_states = []
        current_values = []
        for offset in range(tokens.shape[1]):
            direct_state, current_value = direct._advance_state(
                direct_state,
                tokens[:, offset],
                rho=coefficients["rho"],
                eta=coefficients["eta"],
            )
            addressed_states.append(
                direct_state.index_select(2, direct.candidate_group_leaves)
            )
            current_values.append(current_value)
        direct_reads = torch.einsum(
            "k,ntkud->ntud", coefficients["alpha"], torch.stack(addressed_states, dim=1)
        )
        direct_logits = direct._score_sequence(
            direct_reads,
            torch.stack(current_values, dim=1),
            state_off=False,
        )
        direct_loss = torch.nn.functional.cross_entropy(
            direct_logits.reshape(-1, config.vocab_size), targets.reshape(-1)
        )

        self.assertTrue(
            torch.allclose(closed_output.logits, direct_logits, atol=2e-6, rtol=2e-5)
        )
        self.assertTrue(
            torch.allclose(closed_output.final_state, direct_state, atol=2e-6, rtol=2e-5)
        )
        assert closed_output.loss is not None
        closed_output.loss.backward()
        direct_loss.backward()
        assert closed_initial.grad is not None and direct_initial.grad is not None
        self.assertTrue(
            torch.allclose(closed_initial.grad, direct_initial.grad, atol=2e-6, rtol=2e-5)
        )
        for name in LEARNED_PARAMETER_NAMES:
            closed_gradient = getattr(closed, name).grad
            direct_gradient = getattr(direct, name).grad
            assert closed_gradient is not None and direct_gradient is not None
            self.assertTrue(
                torch.allclose(closed_gradient, direct_gradient, atol=2e-6, rtol=2e-5),
                name,
            )

    def test_sixteen_token_checkpoint_chunks_match_full_closed_form_and_gradients(self) -> None:
        config = _config(vocab_size=11, hidden_size=8, group_size=5, banks=2, context=20)
        artifact = _artifact(vocab_size=config.vocab_size, group_size=config.group_size)
        direct = R4GroupAddressedRetentionLMV1(config, artifact)
        checked = R4GroupAddressedRetentionLMV1(config, artifact)
        checked.load_learned_artifact_(direct.export_learned_artifact())
        token_ids = (torch.arange(34, dtype=torch.long).view(2, 17) % 10) + 1
        targets = torch.roll(token_ids, shifts=-1, dims=1)

        direct_output = direct(token_ids, targets)
        checked_output = checked(token_ids, targets, use_checkpoint=True)
        self.assertEqual(checked_output.audit.checkpoint_chunks, 2)
        self.assertTrue(torch.allclose(direct_output.logits, checked_output.logits, atol=1e-7, rtol=1e-6))
        self.assertTrue(
            torch.allclose(direct_output.final_state, checked_output.final_state, atol=1e-7, rtol=1e-6)
        )
        assert direct_output.loss is not None and checked_output.loss is not None
        direct_output.loss.backward()
        checked_output.loss.backward()
        for name in LEARNED_PARAMETER_NAMES:
            direct_gradient = getattr(direct, name).grad
            checked_gradient = getattr(checked, name).grad
            assert direct_gradient is not None and checked_gradient is not None
            self.assertTrue(
                torch.allclose(direct_gradient, checked_gradient, atol=1e-7, rtol=1e-5),
                name,
            )

    @unittest.skipUnless(
        torch.backends.mps.is_built() and torch.backends.mps.is_available(),
        "Apple MPS is unavailable",
    )
    def test_required_mps_path_completes_forward_and_backward(self) -> None:
        config = _config(vocab_size=11, hidden_size=8, group_size=5, banks=2, context=8)
        model = R4GroupAddressedRetentionLMV1(
            config,
            _artifact(vocab_size=config.vocab_size, group_size=config.group_size),
        ).to("mps")
        token_ids = torch.tensor([[1, 2, 3, 4]], dtype=torch.long, device="mps")
        targets = torch.tensor([[2, 3, 4, 5]], dtype=torch.long, device="mps")
        output = model(token_ids, targets, use_checkpoint=True)
        assert output.loss is not None
        output.loss.backward()
        torch.mps.synchronize()
        self.assertTrue(bool(torch.isfinite(output.loss.detach()).cpu()))
        for name in LEARNED_PARAMETER_NAMES:
            gradient = getattr(model, name).grad
            self.assertIsNotNone(gradient, name)
            assert gradient is not None
            self.assertTrue(bool(torch.isfinite(gradient).all().cpu()), name)


if __name__ == "__main__":
    unittest.main()
