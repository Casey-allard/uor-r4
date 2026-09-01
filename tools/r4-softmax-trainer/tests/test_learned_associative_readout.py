"""Focused model-core checks for #973's learned associative readout."""

from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.group_retention_decoder import DecoderState
from r4_softmax_trainer.language_path_generalization import (
    GROUP_SIZE,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)
from r4_softmax_trainer.learned_associative_readout import (
    BUNDLE_PARAMETER_COUNT,
    EFFECTIVE_ARM_PARAMETER_COUNT,
    HEAD_PARAMETER_COUNT,
    QUERY_SHAPE,
    R4LearnedCandidateLeafAssociativeReadoutV1,
)


def _geometry(*, used_leaves: int = GROUP_SIZE) -> GroupAddressArtifact:
    elements = torch.arange(GROUP_SIZE, dtype=torch.long)
    actions = (elements[:, None] + elements[None, :]) % GROUP_SIZE
    leaves = torch.arange(VOCAB_SIZE, dtype=torch.long) % used_leaves
    leaves[0] = 0
    return GroupAddressArtifact(
        arm="exact_h4",
        identity_offset=0,
        token_leaves=leaves,
        left_actions=actions,
        artifact_cid=f"synthetic:learned-associative:{used_leaves}",
    )


def _tokens() -> torch.Tensor:
    return torch.tensor([[0, 1, 2, 3, 4, 5, 6, 7]], dtype=torch.long)


def _targets() -> torch.Tensor:
    return torch.tensor([[1, 2, 3, 4, 5, 6, 7, 8]], dtype=torch.long)


def _clone_state(state: DecoderState) -> DecoderState:
    return DecoderState(
        keys=state.keys.clone(),
        values=state.values.clone(),
        occupied=state.occupied.clone(),
    )


def _assert_state_equal(
    case: unittest.TestCase, first: DecoderState, second: DecoderState
) -> None:
    case.assertTrue(torch.equal(first.keys, second.keys))
    case.assertTrue(torch.equal(first.values, second.values))
    case.assertTrue(torch.equal(first.occupied, second.occupied))


def _assert_state_close(
    case: unittest.TestCase, first: DecoderState, second: DecoderState
) -> None:
    case.assertTrue(torch.allclose(first.keys, second.keys, atol=2e-6, rtol=2e-5))
    case.assertTrue(torch.allclose(first.values, second.values, atol=2e-6, rtol=2e-5))
    case.assertTrue(torch.equal(first.occupied, second.occupied))


class LearnedAssociativeReadoutTests(unittest.TestCase):
    def test_ledgers_zero_initialization_freeze_and_derangement(self) -> None:
        geometry = _geometry(used_leaves=5)
        baseline = R4RetainedLanguagePathV1(geometry)
        model = R4LearnedCandidateLeafAssociativeReadoutV1(geometry)

        self.assertEqual(tuple(model.geometric_queries.shape), QUERY_SHAPE)
        self.assertEqual(tuple(model.pooled_queries.shape), QUERY_SHAPE)
        self.assertEqual(model.head_parameter_count(), HEAD_PARAMETER_COUNT)
        self.assertEqual(
            model.effective_arm_parameter_count(), EFFECTIVE_ARM_PARAMETER_COUNT
        )
        self.assertEqual(EFFECTIVE_ARM_PARAMETER_COUNT, PARAMETER_COUNT + 393_216)
        self.assertEqual(model.parameter_count(), BUNDLE_PARAMETER_COUNT)
        self.assertEqual(model.state_value_count(), STATE_VALUES)
        self.assertEqual(model.validity_bit_count(), VALIDITY_BITS)
        self.assertTrue(torch.equal(model.geometric_queries, torch.zeros(QUERY_SHAPE)))
        self.assertTrue(torch.equal(model.pooled_queries, torch.zeros(QUERY_SHAPE)))
        self.assertNotEqual(
            model.geometric_queries.untyped_storage().data_ptr(),
            model.pooled_queries.untyped_storage().data_ptr(),
        )
        self.assertTrue(model.geometric_queries.requires_grad)
        self.assertTrue(model.pooled_queries.requires_grad)
        self.assertTrue(
            all(not parameter.requires_grad for parameter in model.frozen_base_parameters())
        )
        self.assertEqual(
            model.export_qualified_base_artifact(), baseline.export_learned_artifact()
        )

        self.assertTrue(
            torch.equal(model.used_candidate_leaves, torch.arange(5, dtype=torch.long))
        )
        self.assertEqual(
            model.used_candidate_leaf_count,
            int(model.deranged_candidate_leaves.numel()),
        )
        self.assertFalse(
            bool(
                (
                    model.used_candidate_leaves == model.deranged_candidate_leaves
                ).any()
            )
        )
        self.assertEqual(
            set(model.deranged_candidate_leaves.tolist()),
            set(model.used_candidate_leaves.tolist()),
        )
        self.assertTrue(
            torch.equal(
                model.deranged_candidate_leaves,
                torch.roll(model.used_candidate_leaves, shifts=-1),
            )
        )

    def test_zero_init_head_off_and_state_off_are_exact_v1(self) -> None:
        geometry = _geometry()
        baseline = R4RetainedLanguagePathV1(geometry)
        model = R4LearnedCandidateLeafAssociativeReadoutV1(geometry)
        tokens = _tokens()

        expected = baseline(tokens)
        initial = model(tokens)
        self.assertTrue(torch.equal(initial.geometric.logits, expected.logits))
        self.assertTrue(torch.equal(initial.pooled.logits, expected.logits))
        _assert_state_equal(self, initial.geometric.final_state, expected.final_state)

        with torch.no_grad():
            model.geometric_queries.normal_(mean=0.0, std=0.01)
            model.pooled_queries.normal_(mean=0.0, std=0.01)
        head_off = model(tokens, head_off=True)
        self.assertTrue(torch.equal(head_off.geometric.logits, expected.logits))
        self.assertTrue(torch.equal(head_off.pooled.logits, expected.logits))

        expected_state_off = baseline(tokens, attention_off=True)
        state_off = model(tokens, attention_off=True)
        self.assertTrue(
            torch.equal(state_off.geometric.logits, expected_state_off.logits)
        )
        self.assertTrue(torch.equal(state_off.pooled.logits, expected_state_off.logits))
        _assert_state_equal(
            self, state_off.geometric.final_state, expected_state_off.final_state
        )
        self.assertEqual(
            initial.geometric.audit.work_signature(),
            initial.pooled.audit.work_signature(),
        )
        self.assertEqual(
            initial.geometric.audit.work_signature(),
            head_off.geometric.audit.work_signature(),
        )
        self.assertEqual(
            initial.geometric.audit.work_signature(),
            state_off.geometric.audit.work_signature(),
        )

    def test_geometric_pooled_and_deranged_are_distinct_equal_work_views(self) -> None:
        model = R4LearnedCandidateLeafAssociativeReadoutV1(_geometry())
        generator = torch.Generator().manual_seed(9_739)
        with torch.no_grad():
            learned = torch.randn(QUERY_SHAPE, generator=generator) * 0.01
            model.geometric_queries.copy_(learned)
            model.pooled_queries.copy_(learned)

        state = model.initial_state(1)
        state.keys.copy_(torch.randn(state.keys.shape, generator=generator))
        state.values.copy_(torch.randn(state.values.shape, generator=generator))
        state.occupied.fill_(True)
        tokens = _tokens()
        geometric = model.forward_arm(
            "geometric", tokens, initial_state=_clone_state(state)
        )
        pooled = model.forward_arm("pooled", tokens, initial_state=_clone_state(state))
        deranged = model.forward_arm(
            "deranged", tokens, initial_state=_clone_state(state)
        )

        self.assertFalse(torch.equal(geometric.logits, pooled.logits))
        self.assertFalse(torch.equal(geometric.logits, deranged.logits))
        self.assertEqual(
            geometric.audit.work_signature(), pooled.audit.work_signature()
        )
        self.assertEqual(
            geometric.audit.work_signature(), deranged.audit.work_signature()
        )
        _assert_state_equal(self, geometric.final_state, pooled.final_state)
        _assert_state_equal(self, geometric.final_state, deranged.final_state)

    def test_stationary_direct_step_causality_and_targets(self) -> None:
        model = R4LearnedCandidateLeafAssociativeReadoutV1(_geometry())
        generator = torch.Generator().manual_seed(19_739)
        with torch.no_grad():
            model.geometric_queries.normal_(generator=generator, std=0.01)
            model.pooled_queries.normal_(generator=generator, std=0.01)
        tokens = _tokens()
        stationary = model(tokens)
        direct = model.forward_incremental(tokens)
        for arm in ("geometric", "pooled"):
            first = getattr(stationary, arm)
            second = getattr(direct, arm)
            self.assertTrue(
                torch.allclose(first.logits, second.logits, atol=2e-6, rtol=2e-5)
            )
            _assert_state_close(self, first.final_state, second.final_state)

        state = model.initial_state(1)
        step_logits: list[torch.Tensor] = []
        for column in range(int(tokens.shape[1])):
            step = model.step(tokens[:, column], state, arm="geometric")
            state = step.final_state
            step_logits.append(step.logits)
        self.assertTrue(
            torch.allclose(
                stationary.geometric.logits,
                torch.stack(step_logits, dim=1),
                atol=2e-6,
                rtol=2e-5,
            )
        )
        _assert_state_close(self, stationary.geometric.final_state, state)

        changed_future = tokens.clone()
        changed_future[:, 4:] = (changed_future[:, 4:] + 31) % VOCAB_SIZE
        future = model(changed_future)
        targets_only = model(tokens, torch.flip(tokens, dims=(1,)))
        self.assertTrue(
            torch.equal(
                stationary.geometric.logits[:, :4], future.geometric.logits[:, :4]
            )
        )
        self.assertTrue(
            torch.equal(stationary.geometric.logits, targets_only.geometric.logits)
        )

    def test_losses_train_only_their_independent_zero_initialized_head(self) -> None:
        model = R4LearnedCandidateLeafAssociativeReadoutV1(_geometry())
        output = model(_tokens(), _targets())
        self.assertIsNotNone(output.geometric.loss)
        assert output.geometric.loss is not None
        output.geometric.loss.backward()
        self.assertIsNotNone(model.geometric_queries.grad)
        assert model.geometric_queries.grad is not None
        self.assertTrue(torch.isfinite(model.geometric_queries.grad).all())
        self.assertGreater(int(torch.count_nonzero(model.geometric_queries.grad)), 0)
        self.assertIsNone(model.pooled_queries.grad)
        self.assertTrue(
            all(parameter.grad is None for parameter in model.frozen_base_parameters())
        )

        model.zero_grad(set_to_none=True)
        replay = model(_tokens(), _targets())
        self.assertIsNotNone(replay.pooled.loss)
        assert replay.pooled.loss is not None
        replay.pooled.loss.backward()
        self.assertIsNone(model.geometric_queries.grad)
        self.assertIsNotNone(model.pooled_queries.grad)
        assert model.pooled_queries.grad is not None
        self.assertTrue(torch.isfinite(model.pooled_queries.grad).all())
        self.assertGreater(int(torch.count_nonzero(model.pooled_queries.grad)), 0)
        self.assertTrue(
            all(parameter.grad is None for parameter in model.frozen_base_parameters())
        )

    def test_disjoint_head_artifacts_are_deterministic_and_replay(self) -> None:
        geometry = _geometry()
        source = R4LearnedCandidateLeafAssociativeReadoutV1(geometry)
        generator = torch.Generator().manual_seed(29_739)
        with torch.no_grad():
            source.geometric_queries.normal_(generator=generator, std=0.01)
            source.pooled_queries.normal_(generator=generator, std=0.02)
        geometric_artifact = source.export_head_artifact("geometric")
        pooled_artifact = source.export_head_artifact("pooled")
        self.assertEqual(
            geometric_artifact, source.export_head_artifact("geometric")
        )
        self.assertEqual(pooled_artifact, source.export_head_artifact("pooled"))
        self.assertNotEqual(geometric_artifact, pooled_artifact)

        replay = R4LearnedCandidateLeafAssociativeReadoutV1(geometry)
        pooled_before = replay.pooled_queries.detach().clone()
        replay.load_head_artifact("geometric", geometric_artifact)
        self.assertTrue(torch.equal(replay.pooled_queries, pooled_before))
        replay.load_head_artifact("pooled", pooled_artifact)
        self.assertEqual(
            replay.export_head_artifact("geometric"), geometric_artifact
        )
        self.assertEqual(replay.export_head_artifact("pooled"), pooled_artifact)
        tokens = _tokens()
        expected = source(tokens)
        observed = replay(tokens)
        self.assertTrue(
            torch.equal(expected.geometric.logits, observed.geometric.logits)
        )
        self.assertTrue(torch.equal(expected.pooled.logits, observed.pooled.logits))


if __name__ == "__main__":
    unittest.main()
