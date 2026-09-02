"""Focused source-parity tests for the #1047 Zoology attention control."""

from __future__ import annotations

import unittest

import torch
from torch.nn import functional as F

from r4_softmax_trainer.zoology_control.model import (
    SOURCE_COMMIT,
    SOURCE_URL,
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)


def _tiny_config() -> ZoologyFigure2Config:
    return ZoologyFigure2Config(
        vocab_size=32,
        d_model=8,
        n_layers=2,
        num_heads=1,
        max_position_embeddings=8,
        attention_dropout=0.1,
        embed_dropout=0.1,
        resid_dropout=0.0,
    )


class ZoologyControlModelTests(unittest.TestCase):
    def test_default_source_shape_and_parameter_contract(self) -> None:
        set_zoology_seed()
        model = ZoologyFigure2Model()
        tokens = torch.tensor(
            [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10]],
            dtype=torch.long,
        )
        result = model.forward_full(tokens, return_attention=True)

        self.assertEqual(
            model.config,
            ZoologyFigure2Config(
                vocab_size=4096,
                d_model=64,
                n_layers=2,
                num_heads=1,
                max_position_embeddings=120,
                attention_dropout=0.1,
                embed_dropout=0.1,
                resid_dropout=0.0,
                layer_norm_epsilon=1.0e-5,
                pad_vocab_size_multiple=1,
            ),
        )
        self.assertEqual(model.parameter_count(), 303_744)
        self.assertEqual(result.logits.shape, (2, 5, 4096))
        self.assertEqual(result.hidden_states.shape, (2, 5, 64))
        self.assertIsNone(result.loss)
        self.assertIsNotNone(result.attention_weights)
        assert result.attention_weights is not None
        self.assertEqual(len(result.attention_weights), 2)
        for weights in result.attention_weights:
            self.assertEqual(weights.shape, (2, 1, 5, 5))
        self.assertEqual(result.logits.device.type, "cpu")

        first = model.backbone.layers[0]
        self.assertIsNotNone(first.sequence_mixer.Wqkv.bias)
        self.assertIsNotNone(first.sequence_mixer.out_proj.bias)
        self.assertIsInstance(first.state_mixer, torch.nn.Identity)
        self.assertEqual(
            model.backbone.embeddings.position_embeddings.num_embeddings,
            120,
        )

    def test_attention_mask_is_causal_and_suffix_cannot_change_prefix(self) -> None:
        set_zoology_seed()
        model = ZoologyFigure2Model(_tiny_config()).eval()
        tokens = torch.tensor([[1, 2, 3, 4, 5, 6]], dtype=torch.long)
        changed_suffix = tokens.clone()
        changed_suffix[:, 4:] = torch.tensor([[29, 30]])

        result = model.forward_full(tokens, return_attention=True)
        changed = model.forward_full(changed_suffix)
        assert result.attention_weights is not None
        future = torch.triu(torch.ones(6, 6, dtype=torch.bool), diagonal=1)
        for weights in result.attention_weights:
            self.assertTrue(
                torch.equal(
                    weights[..., future],
                    torch.zeros_like(weights[..., future]),
                )
            )
            torch.testing.assert_close(
                weights.sum(dim=-1),
                torch.ones_like(weights.sum(dim=-1)),
                rtol=0.0,
                atol=2.0e-7,
            )
        torch.testing.assert_close(
            result.logits[:, :4],
            changed.logits[:, :4],
            rtol=0.0,
            atol=0.0,
        )

    def test_selected_projection_and_loss_equal_full_masked_cross_entropy(self) -> None:
        set_zoology_seed()
        model = ZoologyFigure2Model(_tiny_config()).eval()
        tokens = torch.tensor(
            [[1, 2, 3, 4, 5, 6], [6, 5, 4, 3, 2, 1]],
            dtype=torch.long,
        )
        selected_positions = torch.tensor([[2, 5], [1, 4]], dtype=torch.long)
        selected_targets = torch.tensor([[11, 12], [13, 14]], dtype=torch.long)
        full_targets = torch.full_like(tokens, -100)
        full_targets.scatter_(1, selected_positions, selected_targets)

        projected_shapes: list[tuple[int, ...]] = []

        def record_projection_input(
            _module: torch.nn.Module,
            inputs: tuple[torch.Tensor, ...],
        ) -> None:
            projected_shapes.append(tuple(inputs[0].shape))

        handle = model.lm_head.register_forward_pre_hook(record_projection_input)
        try:
            selected = model.forward_selected(
                tokens,
                selected_positions,
                full_targets,
            )
        finally:
            handle.remove()
        full = model.forward_full(tokens, full_targets)
        direct = model.forward_selected(
            tokens,
            selected_positions,
            selected_targets,
        )

        expected_logits = torch.gather(
            full.logits,
            1,
            selected_positions.unsqueeze(-1).expand(-1, -1, 32),
        )
        self.assertEqual(projected_shapes, [(2, 2, 8)])
        self.assertEqual(selected.logits.shape, (2, 2, 32))
        self.assertEqual(selected.hidden_states.shape, (2, 2, 8))
        self.assertTrue(torch.equal(selected.logits, expected_logits))
        self.assertTrue(torch.equal(selected.selected_targets, selected_targets))
        self.assertTrue(torch.equal(direct.selected_targets, selected_targets))
        assert selected.loss is not None
        assert full.loss is not None
        assert direct.loss is not None
        manual = F.cross_entropy(
            expected_logits.reshape(-1, 32),
            selected_targets.reshape(-1),
        )
        # The full and compact CE kernels reduce tensors with different
        # physical extents; their four labelled terms agree within one f32
        # reduction ulp even though the ignored full positions are absent.
        torch.testing.assert_close(
            selected.loss,
            full.loss,
            rtol=0.0,
            atol=3.0e-7,
        )
        torch.testing.assert_close(selected.loss, direct.loss, rtol=0.0, atol=0.0)
        torch.testing.assert_close(selected.loss, manual, rtol=0.0, atol=0.0)

    def test_tied_head_and_released_seed_are_deterministic(self) -> None:
        set_zoology_seed(123)
        first = ZoologyFigure2Model(_tiny_config())
        set_zoology_seed(123)
        second = ZoologyFigure2Model(_tiny_config())

        self.assertIs(
            first.lm_head.weight,
            first.backbone.embeddings.word_embeddings.weight,
        )
        self.assertEqual(
            first.lm_head.weight.data_ptr(),
            first.backbone.embeddings.word_embeddings.weight.data_ptr(),
        )
        self.assertEqual(list(first.state_dict()), list(second.state_dict()))
        for name, tensor in first.state_dict().items():
            self.assertTrue(torch.equal(tensor, second.state_dict()[name]), name)

        set_zoology_seed(124)
        different = ZoologyFigure2Model(_tiny_config())
        self.assertFalse(
            torch.equal(
                first.backbone.embeddings.word_embeddings.weight,
                different.backbone.embeddings.word_embeddings.weight,
            )
        )

    def test_iclr24_source_parity_golden(self) -> None:
        """Bind init/forward to literal pinned source execution.

        These float32 values were generated by executing the unmodified
        ``zoology/model.py`` and ``zoology/mixers/attention.py`` blobs at the
        exact Apache-2.0 ICLR24 tag, with only Identity dependency stubs for
        zero-probability stochastic depth and the two released reshape calls.
        The port and pinned source produced byte-equal state dictionaries and
        byte-equal logits before these values were recorded.
        """

        self.assertEqual(
            SOURCE_COMMIT,
            "de4e258784224e09909c257ff3ea040f089ed660",
        )
        self.assertEqual(
            SOURCE_URL,
            "https://github.com/HazyResearch/zoology/tree/"
            "de4e258784224e09909c257ff3ea040f089ed660",
        )
        set_zoology_seed(123)
        model = ZoologyFigure2Model(_tiny_config()).eval()

        torch.testing.assert_close(
            model.backbone.embeddings.word_embeddings.weight[0, :4],
            torch.tensor(
                [
                    0.01682600937783718,
                    0.007630460895597935,
                    0.03871878609061241,
                    0.004360933322459459,
                ]
            ),
            rtol=0.0,
            atol=0.0,
        )
        torch.testing.assert_close(
            model.backbone.embeddings.position_embeddings.weight[0, :4],
            torch.tensor(
                [
                    0.002722495701164007,
                    -0.0014288985403254628,
                    -0.004648478236049414,
                    -0.01643170788884163,
                ]
            ),
            rtol=0.0,
            atol=0.0,
        )
        torch.testing.assert_close(
            model.backbone.layers[0].sequence_mixer.Wqkv.weight[0, :4],
            torch.tensor(
                [
                    0.03611164167523384,
                    -0.02012898586690426,
                    0.003164451103657484,
                    0.00013401817705016583,
                ]
            ),
            rtol=0.0,
            atol=0.0,
        )
        logits = model(
            torch.tensor(
                [[1, 2, 3, 4], [4, 3, 2, 1]],
                dtype=torch.long,
            )
        )
        torch.testing.assert_close(
            logits[0, -1, :6],
            torch.tensor(
                [
                    -0.03831605240702629,
                    -0.05481904745101929,
                    -0.031969670206308365,
                    0.005645290017127991,
                    0.10721376538276672,
                    -0.07025852799415588,
                ]
            ),
            rtol=0.0,
            atol=0.0,
        )


if __name__ == "__main__":
    unittest.main()
