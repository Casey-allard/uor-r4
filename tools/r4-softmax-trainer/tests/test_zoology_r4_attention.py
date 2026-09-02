"""Small synthetic seam checks; native export witness check is explicitly bound."""

from __future__ import annotations

import math
import os
import unittest
from pathlib import Path

import torch

from r4_softmax_trainer.zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    _SelfAttention,
)
from r4_softmax_trainer.zoology_r4_inference.attention import (
    R4ZoologyInference,
    _R4InnerAttention,
)
from r4_softmax_trainer.zoology_r4_inference.frames import (
    R4InferenceFrames,
    load_frames,
)


def _synthetic_frames() -> R4InferenceFrames:
    # An explicitly synthetic cyclic orthogonal atlas exercises the matrix
    # equations. Only the separate native-export check establishes H4 identity.
    indices = torch.arange(120, dtype=torch.long)
    angles = indices.to(torch.float64) * (2.0 * math.pi / 120)
    frames = torch.eye(4, dtype=torch.float64).repeat(120, 1, 1)
    frames[:, 0, 0] = torch.cos(angles)
    frames[:, 0, 1] = -torch.sin(angles)
    frames[:, 1, 0] = torch.sin(angles)
    frames[:, 1, 1] = torch.cos(angles)
    leaves = torch.arange(8192, dtype=torch.long) % 120
    return R4InferenceFrames(
        frame_matrices=frames,
        multiplication_indices=(indices[:, None] + indices[None, :]) % 120,
        token_leaf_indices=leaves,
        identity_index=0,
        artifact_cid="synthetic:token-map",
        file_cid="synthetic:token-file",
        frame_artifact_cid="synthetic:frames",
        frame_file_cid="synthetic:frame-file",
        prefix_witnesses=(),
        direct_leaf_count=120,
        witness_frame_count=0,
    )


def _model() -> ZoologyFigure2Model:
    return ZoologyFigure2Model(
        ZoologyFigure2Config(
            vocab_size=32,
            d_model=8,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=8,
            attention_dropout=0.1,
            embed_dropout=0.1,
        )
    )


class ZoologyR4AttentionTests(unittest.TestCase):
    def test_preserves_source_state_eval_rng_and_plain_path(self) -> None:
        torch.manual_seed(1059)
        model = _model().eval()
        before = {name: value.clone() for name, value in model.state_dict().items()}
        inputs = torch.tensor([[1, 2, 3, 4, 5], [5, 4, 3, 2, 1]])
        positions = torch.tensor([[2, 4], [1, 3]])
        with torch.inference_mode():
            original = model.forward_selected(inputs, positions, return_attention=True)
        wrapper = R4ZoologyInference(model, _synthetic_frames())
        rng = torch.get_rng_state().clone()
        plain = wrapper.forward_selected(inputs, positions)
        r4 = wrapper.forward_selected(inputs, positions, execution="r4")
        replay = wrapper.forward_selected(inputs, positions, execution="r4")
        self.assertTrue(torch.equal(original.logits, plain.logits))
        self.assertTrue(torch.equal(r4.logits, replay.logits))
        torch.testing.assert_close(plain.logits, r4.logits, rtol=1e-5, atol=1e-6)
        for left, right in zip(
            plain.attention_weights, r4.attention_weights, strict=True
        ):
            self.assertEqual(tuple(right.shape), (2, 1, 5, 5))
            torch.testing.assert_close(left, right, rtol=1e-5, atol=1e-6)
        self.assertFalse(r4.logits.requires_grad)
        self.assertTrue(torch.equal(rng, torch.get_rng_state()))
        self.assertEqual(tuple(before), tuple(model.state_dict()))
        for name, value in model.state_dict().items():
            self.assertTrue(torch.equal(before[name], value), name)
        self.assertIs(
            model.lm_head.weight, model.backbone.embeddings.word_embeddings.weight
        )
        model.train()
        with self.assertRaisesRegex(RuntimeError, "eval"):
            wrapper.forward_selected(inputs, positions)
        model.eval()
        with self.assertRaises(ValueError):
            wrapper.forward_selected(inputs, positions, execution="train")
        with self.assertRaises(TypeError):
            wrapper.forward_selected(inputs, positions, targets=positions)
        self.assertTrue(
            all(adapter.frame_indices is None for adapter in wrapper.adapters)
        )

    def test_prefix_isolation_and_causal_work_for_every_policy(self) -> None:
        torch.manual_seed(1060)
        wrapper = R4ZoologyInference(_model(), _synthetic_frames())
        first = torch.tensor([[1, 2, 3, 4, 5, 6]])
        changed = torch.tensor([[1, 2, 3, 12, 15, 18]])
        positions = torch.tensor([[0, 2]])
        audits = {}
        for execution in ("plain", "r4", "source_frame_permuted"):
            left = wrapper.forward_selected(first, positions, execution=execution)
            audits[execution] = dict(wrapper.last_audit)
            right = wrapper.forward_selected(changed, positions, execution=execution)
            self.assertTrue(torch.equal(left.logits, right.logits), execution)
            for attention in left.attention_weights:
                self.assertEqual(
                    int(torch.count_nonzero(torch.triu(attention, diagonal=1))), 0
                )
        for execution in ("r4", "source_frame_permuted"):
            self.assertEqual(audits[execution]["future_position_reads"], 0)
            self.assertEqual(audits[execution]["future_score_slots_materialized"], 0)
            self.assertEqual(audits[execution]["admitted_attention_pairs"], 42)
            self.assertEqual(audits[execution]["materialized_score_slots"], 42)
            self.assertEqual(audits[execution]["key_blocks_transported"], 84)
        self.assertIsNone(audits["plain"]["future_position_reads"])
        self.assertEqual(audits["plain"]["future_score_slots_materialized"], 30)
        self.assertGreater(
            audits["source_frame_permuted"]["source_frame_matrices_changed"], 0
        )

    def test_full_width_qkv_scale_and_incoherent_control(self) -> None:
        torch.manual_seed(1061)
        frames = _synthetic_frames()
        adapter = _R4InnerAttention(_SelfAttention(dropout_p=0.1), frames).eval()
        adapter.frame_indices = frames.cumulative_frame_indices(
            torch.tensor([[2, 5, 9, 16, 23], [8, 3, 17, 11, 1]])
        )
        qkv = torch.randn(2, 5, 3, 1, 64)
        with torch.inference_mode():
            adapter.execution = "plain"
            plain, plain_weights = adapter(qkv)
            adapter.execution = "r4"
            coherent, coherent_weights = adapter(qkv)
            coherent_audit = dict(adapter.audit)
            adapter.execution = "source_frame_permuted"
            corrupted, _ = adapter(qkv)
            corrupt_audit = dict(adapter.audit)
        torch.testing.assert_close(coherent, plain, rtol=1e-5, atol=1e-6)
        torch.testing.assert_close(
            coherent_weights, plain_weights, rtol=1e-5, atol=1e-6
        )
        self.assertGreater(float((corrupted - coherent).abs().max()), 0.1)
        for key in (
            "materialized_score_slots",
            "admitted_attention_pairs",
            "key_blocks_transported",
            "value_blocks_transported",
        ):
            self.assertEqual(coherent_audit[key], corrupt_audit[key], key)

    @unittest.skipUnless(
        os.environ.get("R4_ZOOLOGY_FRAME_DIRECTORY"), "native frame export not bound"
    )
    def test_native_full_vocabulary_map_and_prefix_witnesses(self) -> None:
        frames = load_frames(Path(os.environ["R4_ZOOLOGY_FRAME_DIRECTORY"]))
        self.assertEqual(frames.token_leaf_indices.numel(), 8192)
        self.assertEqual(tuple(frames.frame_matrices.shape), (120, 4, 4))
        self.assertEqual(frames.frame_matrices.dtype, torch.float64)
        self.assertEqual(len(frames.prefix_witnesses), 3)
        self.assertLess(frames.direct_leaf_count, 120)
        for witness in frames.prefix_witnesses:
            actual = frames.cumulative_frame_indices(torch.tensor([witness.tokens]))
            self.assertEqual(actual[0].tolist(), list(witness.frame_indices))
        with self.assertRaises(ValueError):
            frames.cumulative_frame_indices(torch.tensor([[8192]]))


if __name__ == "__main__":
    unittest.main()
