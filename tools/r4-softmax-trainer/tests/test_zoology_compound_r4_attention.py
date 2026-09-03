"""Synthetic rectangular transport checks; no fitted artifacts or new native export."""

from __future__ import annotations

import unittest
from unittest.mock import patch

import torch
from test_zoology_r4_attention import _synthetic_frames

from r4_softmax_trainer.zoology_compound_binding.model import CompoundBindingModel
from r4_softmax_trainer.zoology_compound_r4.attention import (
    AUDIT_COUNTS,
    R4CompoundInference,
    _gauge_attention,
    frame_assignment,
)


def _inputs() -> tuple[torch.Tensor, torch.Tensor]:
    inputs = torch.arange(100, 141).unsqueeze(0).repeat(2, 1)
    inputs[:, [1, 9, 17, 25]] = torch.tensor([12, 13, 14, 15])
    inputs[:, [4, 12, 20, 28]] = torch.tensor([20, 21, 22, 23])
    inputs[:, [7, 15, 23, 31]] = torch.tensor([32, 33, 34, 35])
    inputs[:, 35] = torch.tensor([12, 13])
    inputs[:, 37] = torch.tensor([20, 21])
    return inputs, torch.full((2, 1), 37, dtype=torch.long)


class CompoundR4AttentionTests(unittest.TestCase):
    def test_frame_assignment_is_causal_and_null_is_identity(self) -> None:
        inputs, _ = _inputs()
        frames = _synthetic_frames()
        # This retained synthetic cyclic atlas has an independently simple fold.
        expected = inputs[:, :38].cumsum(dim=1) % 120
        query, sources = frame_assignment(inputs, frames)
        self.assertTrue(torch.equal(query, expected[:, 37:38]))
        self.assertTrue(torch.equal(sources[:, :4], expected[:, [7, 15, 23, 31]]))
        self.assertTrue(torch.equal(sources[:, 4], torch.zeros(2, dtype=torch.long)))
        changed = inputs.clone()
        changed[:, 38:] = -999
        changed_query, changed_sources = frame_assignment(changed, frames)
        self.assertTrue(torch.equal(changed_query, query))
        self.assertTrue(torch.equal(changed_sources, sources))
        with self.assertRaisesRegex(ValueError, "batch,41"):
            frame_assignment(inputs[:, :40], frames)
        changed[:, 37] = 8192
        with self.assertRaisesRegex(ValueError, "exported map"):
            frame_assignment(changed, frames)

    def test_plain_source_parity_state_rng_causal_outputs_and_batch_audit(self) -> None:
        torch.manual_seed(1075)
        model = CompoundBindingModel().eval()
        frames = _synthetic_frames()
        inputs, positions = _inputs()
        state = {name: value.clone() for name, value in model.state_dict().items()}
        parameters = [id(value) for value in model.parameters()]
        rng = torch.get_rng_state().clone()
        with torch.inference_mode():
            expected = model.forward_selected(inputs, positions, return_attention=True)
        wrappers = {
            execution: R4CompoundInference(model, frames, execution)
            for execution in ("plain", "r4", "source_frame_permuted")
        }
        with patch.object(
            model, "forward_selected", wraps=model.forward_selected
        ) as source:
            plain = wrappers["plain"].forward_selected(inputs, positions)
            source.assert_called_once_with(inputs, positions, return_attention=True)
        coherent = wrappers["r4"].forward_selected(inputs, positions)
        broken = wrappers["source_frame_permuted"].forward_selected(inputs, positions)
        self.assertTrue(torch.equal(plain.logits, expected.logits))
        self.assertTrue(
            torch.equal(plain.attention_weights[0], expected.attention_weights[0])
        )
        self.assertEqual(tuple(coherent.logits.shape), (2, 1, 4096))
        self.assertEqual(tuple(coherent.attention_weights[0].shape), (2, 1, 1, 5))
        torch.testing.assert_close(coherent.logits, plain.logits, rtol=1e-5, atol=1e-6)
        torch.testing.assert_close(
            coherent.attention_weights[0],
            plain.attention_weights[0],
            rtol=1e-5,
            atol=1e-6,
        )
        self.assertFalse(torch.equal(broken.logits, coherent.logits))
        for wrapper in wrappers.values():
            self.assertIs(wrapper.model, model)
            audit = wrapper.audit
            self.assertEqual(audit["rows"], 2)
            self.assertEqual(audit["admitted_attention_pairs"], 10)
            self.assertEqual(audit["materialized_score_slots"], 10)
            self.assertEqual(audit["null_attention_pairs"], 2)
            self.assertEqual(audit["future_position_reads"], 0)
            self.assertEqual(audit["future_score_slots_materialized"], 0)
            self.assertIn(frames.identity_index, audit["reached_frame_indices"])
            changed = inputs.clone()
            changed[:, 38:] = -999
            left = wrapper.forward_selected(inputs, positions)
            right = wrapper.forward_selected(changed, positions)
            self.assertTrue(torch.equal(left.logits, right.logits))
            self.assertTrue(
                torch.equal(left.attention_weights[0], right.attention_weights[0])
            )
            self.assertFalse(right.logits.requires_grad)
            self.assertIsNone(right.loss)
            self.assertIsNone(right.selected_targets)
        for key in AUDIT_COUNTS:
            if key not in (
                "source_frame_positions_changed",
                "source_frame_matrices_changed",
            ):
                self.assertEqual(
                    wrappers["r4"].audit[key],
                    wrappers["source_frame_permuted"].audit[key],
                )
        self.assertEqual(wrappers["plain"].audit["key_blocks_transported"], 0)
        self.assertEqual(wrappers["r4"].audit["key_blocks_transported"], 480)
        self.assertEqual(wrappers["r4"].audit["value_blocks_transported"], 480)
        self.assertEqual(wrappers["r4"].audit["query_blocks_encoded"], 96)
        self.assertEqual(wrappers["r4"].audit["output_blocks_decoded"], 96)
        self.assertEqual(
            wrappers["source_frame_permuted"].audit["source_frame_positions_changed"],
            24,
        )
        _, sources = frame_assignment(inputs, frames)
        changed_matrices = int((sources[:, :4] != sources[:, [1, 2, 3, 0]]).sum())
        self.assertEqual(
            wrappers["source_frame_permuted"].audit["source_frame_matrices_changed"],
            3 * changed_matrices,
        )
        wrapper = wrappers["r4"]
        wrapper.reset_audit()
        self.assertTrue(all(wrapper.audit[key] == 0 for key in AUDIT_COUNTS))
        self.assertEqual(wrapper.audit["reached_frame_indices"], [])
        first = wrapper.forward_selected(
            inputs[:1], positions[:1], return_attention=False
        )
        second = wrapper.forward_selected(
            inputs[1:], positions[1:], return_attention=False
        )
        self.assertIsNone(first.attention_weights)
        self.assertIsNone(second.attention_weights)
        self.assertEqual(wrapper.audit["rows"], 2)
        self.assertEqual(wrapper.audit["value_blocks_encoded"], 160)
        self.assertEqual([id(value) for value in model.parameters()], parameters)
        self.assertEqual(list(model.state_dict()), list(state))
        self.assertTrue(
            all(
                torch.equal(model.state_dict()[name], value)
                for name, value in state.items()
            )
        )
        self.assertIs(model.embedding.weight, model.lm_head.weight)
        self.assertTrue(torch.equal(torch.get_rng_state(), rng))

    def test_full_head_scale_true_frame_corruption_and_complete_null_mixture(
        self,
    ) -> None:
        torch.manual_seed(1076)
        frames = _synthetic_frames()
        query_frames = frames.frame_matrices[torch.tensor([19, 43])]
        source_frames = frames.frame_matrices[
            torch.tensor([[2, 11, 29, 55, 0], [3, 14, 37, 61, 0]])
        ]
        query = torch.randn(2, 1, 64)
        keys = torch.randn(2, 5, 64)
        values = torch.randn(2, 5, 64)
        with torch.inference_mode():
            coherent, weights = _gauge_attention(
                query,
                keys,
                values,
                query_frames,
                source_frames,
                permute_source_frames=False,
            )
            expected_weights = torch.softmax(
                query @ keys.transpose(-2, -1) / 8.0, dim=-1
            )
            expected = expected_weights @ values
            torch.testing.assert_close(
                weights[:, 0], expected_weights, rtol=1e-5, atol=1e-6
            )
            torch.testing.assert_close(coherent, expected, rtol=1e-5, atol=1e-6)
            broken, broken_weights = _gauge_attention(
                query,
                keys,
                values,
                query_frames,
                source_frames,
                permute_source_frames=True,
            )
            # In model coordinates, inconsistent transport acts as
            # F_permuted F_true^T on the true source K/V, including identity null.
            corruption = source_frames[:, [1, 2, 3, 0, 4]] @ source_frames.transpose(
                -2, -1
            )
            corrupted_keys = torch.einsum(
                "bsij,bsdj->bsdi", corruption, keys.double().reshape(2, 5, 16, 4)
            ).reshape(2, 5, 64)
            corrupted_values = torch.einsum(
                "bsij,bsdj->bsdi", corruption, values.double().reshape(2, 5, 16, 4)
            ).reshape(2, 5, 64)
            expected_broken_weights = torch.softmax(
                (query.double() @ corrupted_keys.transpose(-2, -1)).float() / 8.0,
                dim=-1,
            )
            expected_broken = (
                expected_broken_weights.double() @ corrupted_values
            ).float()
            torch.testing.assert_close(
                broken_weights[:, 0], expected_broken_weights, rtol=1e-5, atol=1e-6
            )
            torch.testing.assert_close(broken, expected_broken, rtol=1e-5, atol=1e-6)
            self.assertGreater(float((broken - coherent).abs().max()), 0.1)
            self.assertTrue(torch.equal(corrupted_keys[:, 4], keys[:, 4].double()))
            self.assertTrue(torch.equal(corrupted_values[:, 4], values[:, 4].double()))
            # Equal scores force a five-way mixture. The null contributes 1/5,
            # and no fact or null argmax shortcut can produce this exact mean.
            keys.zero_()
            values = (
                torch.arange(1, 6, dtype=torch.float32)
                .reshape(1, 5, 1)
                .expand(2, 5, 64)
                .clone()
            )
            mixed, uniform = _gauge_attention(
                query,
                keys,
                values,
                query_frames,
                source_frames,
                permute_source_frames=False,
            )
            torch.testing.assert_close(
                uniform, torch.full((2, 1, 1, 5), 0.2), rtol=0, atol=0
            )
            torch.testing.assert_close(
                mixed, torch.full((2, 1, 64), 3.0), rtol=0, atol=1e-6
            )
            # A null-only value still travels from identity to the query frame
            # and back. Omitting it or treating it as already query-local fails.
            values[:, :4].zero_()
            null_only, _ = _gauge_attention(
                query,
                keys,
                values,
                query_frames,
                source_frames,
                permute_source_frames=False,
            )
            torch.testing.assert_close(
                null_only, torch.ones(2, 1, 64), rtol=0, atol=1e-6
            )

    def test_inference_interface_rejects_labels_controls_modes_and_failed_batches(
        self,
    ) -> None:
        model = CompoundBindingModel().eval()
        inputs, positions = _inputs()
        wrapper = R4CompoundInference(model, _synthetic_frames(), "r4")
        with self.assertRaises(TypeError):
            wrapper.forward_selected(
                inputs, positions, targets=torch.tensor([[1], [2]])
            )
        with self.assertRaisesRegex(TypeError, "labels are not accepted"):
            wrapper.forward_selected(inputs, positions, torch.tensor([[1], [2]]))
        with self.assertRaisesRegex(ValueError, "control='none'"):
            wrapper.forward_selected(inputs, positions, control="value_cycle")
        with self.assertRaisesRegex(ValueError, "position37"):
            wrapper.forward_selected(inputs, positions + 1)
        with self.assertRaisesRegex(ValueError, "unsupported"):
            R4CompoundInference(model, _synthetic_frames(), "train")
        model.train()
        with self.assertRaisesRegex(RuntimeError, "eval"):
            wrapper.forward_selected(inputs, positions)
        model.eval()
        model.double()
        with self.assertRaisesRegex(ValueError, "CPU f32"):
            wrapper.forward_selected(inputs, positions)
        model.float()
        with (
            patch.object(
                model.query_projection,
                "forward",
                side_effect=RuntimeError("synthetic failure"),
            ),
            self.assertRaisesRegex(RuntimeError, "synthetic failure"),
        ):
            wrapper.forward_selected(inputs, positions)
        self.assertFalse(wrapper._active)
        self.assertTrue(all(wrapper.audit[key] == 0 for key in AUDIT_COUNTS))


if __name__ == "__main__":
    unittest.main()
