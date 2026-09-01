"""Focused admission tests for the #973 direct retained readout seam."""

from __future__ import annotations

import unittest

import torch
from r4_softmax_trainer.direct_retained_readout import (
    R4DirectRetainedReadoutLanguagePathV1,
)
from r4_softmax_trainer.group_retention import GroupAddressArtifact
from r4_softmax_trainer.language_path_generalization import (
    GROUP_SIZE,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
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
        artifact_cid="synthetic:direct-retained-readout",
    )


def _assert_state_equal(case: unittest.TestCase, left: object, right: object) -> None:
    for name in ("keys", "values", "occupied"):
        case.assertTrue(
            torch.equal(getattr(left, name), getattr(right, name)),
            name,
        )


def _assert_state_close(case: unittest.TestCase, left: object, right: object) -> None:
    for name in ("keys", "values"):
        case.assertTrue(
            torch.allclose(
                getattr(left, name),
                getattr(right, name),
                atol=2e-6,
                rtol=2e-5,
            ),
            name,
        )
    case.assertTrue(torch.equal(left.occupied, right.occupied))


class DirectRetainedReadoutTests(unittest.TestCase):
    def test_initialization_artifact_and_ledgers_are_exactly_v1(self) -> None:
        geometry = _geometry()
        baseline = R4RetainedLanguagePathV1(geometry)
        candidate = R4DirectRetainedReadoutLanguagePathV1(geometry)
        control = R4DirectRetainedReadoutLanguagePathV1.matched_v1_control(geometry)

        self.assertTrue(candidate.direct_readout_enabled)
        self.assertEqual(candidate.direct_readout_gain, 1.0)
        self.assertFalse(control.direct_readout_enabled)
        self.assertEqual(control.direct_readout_gain, 0.0)
        for model in (candidate, control):
            self.assertEqual(model.parameter_count(), PARAMETER_COUNT)
            self.assertEqual(model.state_value_count(), STATE_VALUES)
            self.assertEqual(model.validity_bit_count(), VALIDITY_BITS)
            self.assertEqual(
                model.token_embedding.weight.untyped_storage().data_ptr(),
                model.output_weight.untyped_storage().data_ptr(),
            )
            self.assertEqual(
                set(dict(model.named_parameters())),
                set(dict(baseline.named_parameters())),
            )
            self.assertEqual(
                model.export_learned_artifact(), baseline.export_learned_artifact()
            )

    def test_g0_is_v1_exact_and_g1_changes_only_logits(self) -> None:
        geometry = _geometry()
        baseline = R4RetainedLanguagePathV1(geometry)
        candidate = R4DirectRetainedReadoutLanguagePathV1(geometry)
        control = R4DirectRetainedReadoutLanguagePathV1.matched_v1_control(geometry)
        tokens = torch.tensor(
            [[0, 4, 8, 12, 16, 20], [0, 3, 7, 11, 15, 19]],
            dtype=torch.long,
        )

        baseline_output = baseline(tokens)
        control_output = control(tokens)
        candidate_output = candidate(tokens)
        self.assertTrue(torch.equal(control_output.logits, baseline_output.logits))
        _assert_state_equal(
            self, control_output.final_state, baseline_output.final_state
        )
        _assert_state_equal(
            self, candidate_output.final_state, baseline_output.final_state
        )
        self.assertTrue(
            torch.equal(candidate_output.logits[:, :1], control_output.logits[:, :1])
        )
        self.assertFalse(
            torch.equal(candidate_output.logits[:, 1:], control_output.logits[:, 1:])
        )
        self.assertEqual(
            candidate_output.audit.work_signature(),
            control_output.audit.work_signature(),
        )
        self.assertEqual(
            candidate_output.audit.vocabulary_scores,
            tokens.numel() * VOCAB_SIZE,
        )

        baseline_off = baseline(tokens, attention_off=True)
        candidate_off = candidate(tokens, attention_off=True)
        self.assertTrue(torch.equal(candidate_off.logits, baseline_off.logits))
        self.assertEqual(
            candidate_output.audit.work_signature(),
            candidate_off.audit.work_signature(),
        )

    def test_stationary_direct_step_causality_and_artifact_replay(self) -> None:
        geometry = _geometry()
        model = R4DirectRetainedReadoutLanguagePathV1(geometry)
        tokens = torch.tensor(
            [[0, 4, 8, 12, 16, 20], [0, 3, 7, 11, 15, 19]],
            dtype=torch.long,
        )
        stationary = model(tokens)
        direct = model.forward_incremental(tokens)
        self.assertTrue(
            torch.allclose(stationary.logits, direct.logits, atol=2e-6, rtol=2e-5)
        )
        _assert_state_close(self, stationary.final_state, direct.final_state)

        state = model.initial_state(int(tokens.shape[0]))
        step_logits: list[torch.Tensor] = []
        for column in range(int(tokens.shape[1])):
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
        _assert_state_close(self, stationary.final_state, state)

        changed_future = tokens.clone()
        changed_future[:, 3:] = (changed_future[:, 3:] + 17) % VOCAB_SIZE
        changed_targets = torch.flip(tokens, dims=(1,))
        future = model(changed_future)
        target_only = model(tokens, changed_targets)
        self.assertTrue(torch.equal(stationary.logits[:, :3], future.logits[:, :3]))
        self.assertTrue(torch.equal(stationary.logits, target_only.logits))

        artifact = model.export_learned_artifact()
        replay = R4DirectRetainedReadoutLanguagePathV1(geometry)
        replay.load_learned_artifact(artifact)
        self.assertTrue(torch.equal(stationary.logits, replay(tokens).logits))

    def test_all_parameters_receive_finite_nonzero_gradient(self) -> None:
        model = R4DirectRetainedReadoutLanguagePathV1(_geometry())
        tokens = torch.tensor(
            [
                [0, 4, 8, 12, 16, 20, 24, 28, 32, 36],
                [0, 3, 7, 11, 15, 19, 23, 27, 31, 35],
            ],
            dtype=torch.long,
        )
        targets = torch.tensor(
            [
                [4, 8, 12, 16, 20, 24, 28, 32, 36, 1],
                [3, 7, 11, 15, 19, 23, 27, 31, 35, 1],
            ],
            dtype=torch.long,
        )
        output = model(tokens, targets)
        self.assertIsNotNone(output.loss)
        assert output.loss is not None
        output.loss.backward()
        inactive = [
            name
            for name, parameter in model.named_parameters()
            if parameter.grad is None
            or not bool(torch.isfinite(parameter.grad).all())
            or not bool((parameter.grad != 0).any())
        ]
        self.assertEqual(inactive, [])

    def test_state_off_exactly_collapses_matched_tail_prompt_swap(self) -> None:
        model = R4DirectRetainedReadoutLanguagePathV1(_geometry())
        shared_tail = [20, 21, 22, 23]
        continuation = [40, 41, 42, 43]
        left_prompt = [5, 6, 7, 8, *shared_tail]
        right_prompt = [30, 31, 32, 33, *shared_tail]
        rows = torch.tensor(
            [
                [0, *left_prompt, *continuation[:-1]],
                [0, *right_prompt, *continuation[:-1]],
            ],
            dtype=torch.long,
        )
        suffix = len(left_prompt)
        enabled = model(rows).logits[:, suffix:]
        disabled = model(rows, attention_off=True).logits[:, suffix:]
        self.assertFalse(torch.equal(enabled[0], enabled[1]))
        self.assertTrue(torch.equal(disabled[0], disabled[1]))


if __name__ == "__main__":
    unittest.main()
