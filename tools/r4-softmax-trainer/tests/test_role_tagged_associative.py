"""Focused mechanics tests for the #1045 role-tagged associative model."""

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
    HIDDEN_SIZE,
    LAYERS,
    PARAMETER_COUNT as BASE_PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
)
from r4_softmax_trainer.position_kv_binding import (
    PositionKVCacheState,
    R4PositionPreservingCausalKVBindingV1,
)
from r4_softmax_trainer.role_tagged_associative import (
    KEY_ROLE,
    PARAMETER_COUNT,
    POLICY,
    QUERY_ROLE,
    ROLE_COUNT,
    ROLE_PARAMETER_COUNT,
    TEXT_ROLE,
    VALUE_ROLE,
    R4RoleTaggedAssociativeCurriculumV1,
    RoleTaggedAssociativeQueryOutput,
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
        artifact_cid="synthetic:role-tagged-geometry",
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
        artifact_cid="synthetic:role-tagged-frames",
    )
    return geometry, frames


def _model() -> R4RoleTaggedAssociativeCurriculumV1:
    geometry, frames = _geometry_and_frames()
    return R4RoleTaggedAssociativeCurriculumV1(  # type: ignore[arg-type]
        geometry, frames
    )


def _base_model() -> R4PositionPreservingCausalKVBindingV1:
    geometry, frames = _geometry_and_frames()
    return R4PositionPreservingCausalKVBindingV1(  # type: ignore[arg-type]
        geometry, frames
    )


def _assert_state_equal(
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
    case.assertTrue(torch.equal(first.keys, second.keys))
    case.assertTrue(torch.equal(first.values, second.values))


class RoleTaggedAssociativeTests(unittest.TestCase):
    def setUp(self) -> None:
        torch.manual_seed(10_045)
        self.tokens = torch.tensor(
            [[0, 256, 2048, 7, 256, 9], [0, 257, 2049, 8, 257, 10]],
            dtype=torch.long,
        )
        self.roles = torch.tensor(
            [
                [TEXT_ROLE, KEY_ROLE, VALUE_ROLE, TEXT_ROLE, QUERY_ROLE, TEXT_ROLE],
                [TEXT_ROLE, KEY_ROLE, VALUE_ROLE, TEXT_ROLE, QUERY_ROLE, TEXT_ROLE],
            ],
            dtype=torch.uint8,
        )
        self.targets = torch.full_like(self.tokens, -100)
        self.targets[:, 4] = torch.tensor([2048, 2049])
        self.selected = torch.tensor([[4], [4]], dtype=torch.long)

    def test_ledgers_and_ordinary_initialization_are_exact(self) -> None:
        ordinary = OrdinaryCausalSoftmaxLanguagePathV1()
        payload = ordinary.export_learned_artifact()
        geometry, frames = _geometry_and_frames()
        model = R4RoleTaggedAssociativeCurriculumV1.from_ordinary_artifact(
            payload,
            geometry=geometry,
            frames=frames,  # type: ignore[arg-type]
        )
        base = _base_model()
        base.load_learned_artifact(payload)

        self.assertEqual(POLICY, "R4RoleTaggedAssociativeCurriculumV1")
        self.assertEqual(
            (TEXT_ROLE, KEY_ROLE, VALUE_ROLE, QUERY_ROLE), (0, 1, 2, 3)
        )
        self.assertEqual(ROLE_COUNT, 4)
        self.assertEqual(ROLE_PARAMETER_COUNT, 4 * HIDDEN_SIZE)
        self.assertEqual(PARAMETER_COUNT, BASE_PARAMETER_COUNT + 4 * HIDDEN_SIZE)
        self.assertEqual(model.parameter_count(), PARAMETER_COUNT)
        self.assertEqual(model.state_value_count(), STATE_VALUES)
        self.assertEqual(model.state_byte_count_f32(), STATE_BYTES_F32)
        self.assertEqual(model.validity_bit_count(), VALIDITY_BITS)
        self.assertTrue(
            torch.equal(
                model.role_embedding.weight,
                torch.zeros_like(model.role_embedding.weight),
            )
        )

        observed = model(self.tokens, self.roles, execution="plain")
        expected = base(self.tokens, execution="plain")
        self.assertTrue(torch.equal(observed.logits, expected.logits))
        _assert_state_equal(self, observed.final_state, expected.final_state)

        model.role_embedding.weight.grad = None
        output = model(self.tokens, self.roles, self.targets, execution="plain")
        assert output.loss is not None
        output.loss.backward()
        gradient = model.role_embedding.weight.grad
        self.assertIsNotNone(gradient)
        assert gradient is not None
        self.assertTrue(
            torch.equal(gradient[TEXT_ROLE], torch.zeros(HIDDEN_SIZE))
        )
        self.assertTrue(bool((gradient[1:].abs().sum(dim=1) > 0).all()))

    def test_query_only_projection_matches_full_without_full_vocabulary_work(
        self,
    ) -> None:
        model = _model()
        with torch.no_grad():
            model.role_embedding.weight[KEY_ROLE].fill_(0.01)
            model.role_embedding.weight[VALUE_ROLE].fill_(-0.015)
            model.role_embedding.weight[QUERY_ROLE].fill_(0.02)
        full = model(self.tokens, self.roles, self.targets, execution="plain")
        selected = model(
            self.tokens,
            self.roles,
            self.targets,
            selected_positions=self.selected,
            execution="plain",
        )
        self.assertIsInstance(selected, RoleTaggedAssociativeQueryOutput)
        assert isinstance(selected, RoleTaggedAssociativeQueryOutput)
        expected_logits = torch.gather(
            full.logits,
            1,
            self.selected.unsqueeze(-1).expand(-1, -1, VOCAB_SIZE),
        )
        self.assertTrue(torch.equal(selected.logits, expected_logits))
        self.assertEqual(tuple(selected.logits.shape), (2, 1, VOCAB_SIZE))
        self.assertTrue(torch.equal(selected.selected_targets, self.targets[:, 4:5]))
        assert full.loss is not None and selected.loss is not None
        self.assertEqual(float(selected.loss), float(full.loss))
        self.assertEqual(selected.audit.target_reads, 2)
        self.assertEqual(selected.audit.vocabulary_scores, 2 * VOCAB_SIZE)
        self.assertEqual(full.audit.vocabulary_scores, self.tokens.numel() * VOCAB_SIZE)
        self.assertTrue(
            torch.equal(selected.attention_weights, full.attention_weights)
        )
        _assert_state_equal(self, selected.final_state, full.final_state)

        selected_labels = self.targets[:, 4:5]
        direct = model(
            self.tokens,
            self.roles,
            selected_labels,
            selected_positions=self.selected,
        )
        assert isinstance(direct, RoleTaggedAssociativeQueryOutput)
        self.assertTrue(torch.equal(direct.selected_targets, selected_labels))

    def test_roles_are_live_but_frames_remain_token_derived(self) -> None:
        model = _model()
        with torch.no_grad():
            model.role_embedding.weight[KEY_ROLE].normal_(0.0, 0.1)
            model.role_embedding.weight[VALUE_ROLE].normal_(0.0, 0.1)
            model.role_embedding.weight[QUERY_ROLE].normal_(0.0, 0.1)
        native = model(self.tokens, self.roles, execution="r4")
        role_off = model(
            self.tokens,
            torch.zeros_like(self.roles),
            execution="r4",
        )
        self.assertFalse(torch.equal(native.logits, role_off.logits))
        self.assertTrue(
            torch.equal(
                native.final_state.source_frame_indices,
                role_off.final_state.source_frame_indices,
            )
        )
        self.assertTrue(
            torch.equal(
                native.final_state.current_frame_indices,
                role_off.final_state.current_frame_indices,
            )
        )
        self.assertEqual(native.audit.future_reads, 0)
        self.assertEqual(native.audit.forbidden_reads, 0)
        self.assertEqual(native.audit.provider_calls, 0)
        self.assertEqual(native.audit.teacher_calls, 0)

    def test_role_aware_full_incremental_and_step_paths_match(self) -> None:
        model = _model()
        with torch.no_grad():
            model.role_embedding.weight[1:].normal_(0.0, 0.03)
        for execution in ("plain", "r4"):
            full = model(self.tokens, self.roles, execution=execution)
            incremental = model.forward_incremental(
                self.tokens,
                self.roles,
                execution=execution,
            )
            self.assertLessEqual(
                float((full.logits - incremental.logits).abs().max()), 2e-5
            )
            self.assertTrue(
                torch.equal(
                    full.logits.argmax(dim=-1), incremental.logits.argmax(dim=-1)
                )
            )
            self.assertEqual(
                tuple(full.attention_weights.shape),
                (LAYERS, 2, HEADS, self.tokens.shape[1], CONTEXT),
            )
            self.assertLessEqual(
                float(
                    (full.attention_weights - incremental.attention_weights)
                    .abs()
                    .max()
                ),
                2e-6,
            )

            state = model.initial_state(2, execution=execution)
            step_logits = []
            for position in range(self.tokens.shape[1]):
                output = model.step(
                    self.tokens[:, position],
                    self.roles[:, position],
                    state,
                    execution=execution,
                )
                state = output.final_state
                step_logits.append(output.logits)
            stacked = torch.stack(step_logits, dim=1)
            self.assertLessEqual(float((full.logits - stacked).abs().max()), 2e-5)
            self.assertTrue(
                torch.equal(full.logits.argmax(dim=-1), stacked.argmax(dim=-1))
            )
            self.assertEqual(state.length, self.tokens.shape[1])
            self.assertEqual(state.audit.future_reads, 0)
            self.assertEqual(state.audit.forbidden_reads, 0)

    def test_role_and_query_contracts_fail_closed(self) -> None:
        model = _model()
        with self.assertRaisesRegex(ValueError, "uint8"):
            model(self.tokens, self.roles.long())
        invalid = self.roles.clone()
        invalid[0, 0] = ROLE_COUNT
        with self.assertRaisesRegex(ValueError, "unsupported"):
            model(self.tokens, invalid)
        with self.assertRaisesRegex(ValueError, "unique"):
            model(
                self.tokens,
                self.roles,
                selected_positions=torch.tensor([[4, 4], [4, 5]]),
            )
        with self.assertRaisesRegex(ValueError, "out-of-prefix"):
            model(
                self.tokens,
                self.roles,
                selected_positions=torch.tensor([[6], [4]]),
            )


if __name__ == "__main__":
    unittest.main()
