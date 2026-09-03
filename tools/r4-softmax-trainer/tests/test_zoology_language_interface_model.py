"""Four synthetic reader/core checks; no retained model or fitted inference."""

from __future__ import annotations

import itertools
import unittest
from unittest.mock import patch

import torch
from torch.nn import functional as F

from r4_softmax_trainer.zoology_compound_binding.model import CompoundBindingModel
from r4_softmax_trainer.zoology_language_interface.model import (
    MODEL_POLICY,
    LanguageInterfaceModel,
    LearnedRoleReader,
)
from r4_softmax_trainer.zoology_release.development import _tensor_mapping_cid


def _inputs() -> tuple[torch.Tensor, torch.Tensor]:
    inputs = torch.arange(20, 150).reshape(2, 5, 13)
    lengths = torch.tensor([[11, 12, 13, 10, 13], [13, 12, 11, 13, 12]])
    valid = torch.arange(13) < lengths.unsqueeze(-1)
    return inputs.masked_fill(~valid, 57), lengths


def _model() -> LanguageInterfaceModel:
    torch.manual_seed(123)
    core = CompoundBindingModel()
    return LanguageInterfaceModel(core, LearnedRoleReader()).eval()


class LanguageInterfaceModelTests(unittest.TestCase):
    def test_all_token_mixtures_padding_isolation_and_label_absence(self) -> None:
        model = _model()
        inputs, lengths = _inputs()
        self.assertEqual(model.reader.parameter_count(), 141571)
        self.assertEqual(
            sum(p.numel() for p in model.parameters() if p.requires_grad), 141571
        )
        self.assertIs(model.core.embedding.weight, model.core.lm_head.weight)
        self.assertFalse(MODEL_POLICY["position_embeddings"])
        valid = torch.arange(13) < lengths.unsqueeze(-1)
        with torch.inference_mode():
            output = model(inputs, lengths)
            self.assertEqual(output["logits"].shape, (2, 4096))
            self.assertEqual(output["binding_attention"].shape, (2, 5))
            self.assertEqual(output["role_attention"].shape, (2, 5, 3, 13))
            self.assertEqual(output["role_vectors"].shape, (2, 5, 3, 64))
            expanded_valid = valid.unsqueeze(2).expand_as(output["role_attention"])
            self.assertTrue(bool((output["role_attention"][expanded_valid] > 0).all()))
            self.assertTrue(
                bool((output["role_attention"][~expanded_valid] == 0).all())
            )
            torch.testing.assert_close(
                output["role_attention"].sum(-1), torch.ones(2, 5, 3)
            )
            changed_padding = inputs.masked_fill(~valid, -999)
            ignored = model(changed_padding, lengths)
            for key in output:
                torch.testing.assert_close(ignored[key], output[key], rtol=0, atol=0)
        with self.assertRaises(TypeError):
            model(inputs, lengths, targets=torch.zeros(2, dtype=torch.long))
        with self.assertRaises(TypeError):
            model.reader(inputs, lengths, targets=torch.zeros(2, 5, 3))
        with self.assertRaisesRegex(ValueError, "lengths"):
            model(inputs, lengths * 0)
        invalid = inputs.clone()
        invalid[:, 0, 0] = 4096
        with self.assertRaisesRegex(ValueError, "vocabulary"):
            model(invalid, lengths)

    def test_shared_clause_reader_fact_permutation_and_local_context(self) -> None:
        model = _model()
        inputs, lengths = _inputs()
        permutations = torch.tensor(list(itertools.permutations(range(4))))
        orders = torch.cat((permutations, torch.full((24, 1), 4)), dim=1)
        permuted_inputs = inputs[0, orders]
        permuted_lengths = lengths[0, orders]
        with torch.inference_mode():
            base = model(inputs[:1], lengths[:1])
            observed = model(permuted_inputs, permuted_lengths)
            torch.testing.assert_close(
                observed["logits"], base["logits"].expand(24, -1), rtol=1e-5, atol=1e-6
            )
            torch.testing.assert_close(
                observed["binding_attention"],
                base["binding_attention"][0, orders],
                rtol=1e-6,
                atol=1e-7,
            )
            torch.testing.assert_close(
                observed["role_attention"], base["role_attention"][0, orders]
            )
            scores = model.reader.role_scores(inputs, lengths)
            changed = inputs.clone()
            changed[:, :, 0] = 500
            local_scores = model.reader.role_scores(changed, lengths)
            # Token0 lies outside token3's radius-two context; no position table
            # or cross-clause operation can carry the change to that score.
            torch.testing.assert_close(local_scores[:, :, :, 3], scores[:, :, :, 3])
            self.assertFalse(torch.equal(local_scores[:, :, :, 0], scores[:, :, :, 0]))

    def test_external_role_loss_gradients_and_frozen_core_state(self) -> None:
        model = _model().train()
        inputs, lengths = _inputs()
        before_state = _tensor_mapping_cid(model.core.state_dict())
        parameters = {name: id(value) for name, value in model.core.named_parameters()}
        self.assertFalse(model.core.training)
        targets = torch.tensor([[[0, 4, 7]] * 4 + [[2, 8, -100]]] * 2)
        scores = model.reader.role_scores(inputs, lengths)
        # Gold role positions are loss targets only, never a model argument.
        loss = F.cross_entropy(
            scores.flatten(0, 2), targets.flatten(), ignore_index=-100
        )
        loss.backward()
        for name, parameter in model.reader.named_parameters():
            with self.subTest(parameter=name):
                self.assertIsNotNone(parameter.grad)
                self.assertTrue(bool(torch.isfinite(parameter.grad).all()))
                self.assertGreater(float(parameter.grad.abs().sum()), 0.0)
        self.assertTrue(
            all(parameter.grad is None for parameter in model.core.parameters())
        )
        self.assertTrue(
            all(not parameter.requires_grad for parameter in model.core.parameters())
        )
        self.assertEqual(_tensor_mapping_cid(model.core.state_dict()), before_state)
        self.assertEqual(
            {name: id(value) for name, value in model.core.named_parameters()},
            parameters,
        )

    def test_original_full_mixture_and_value_cycle_leave_attention_unchanged(
        self,
    ) -> None:
        model = _model()
        inputs, lengths = _inputs()
        before_state = _tensor_mapping_cid(model.state_dict())
        before_rng = torch.get_rng_state().clone()
        with torch.inference_mode():
            plain = model(inputs, lengths)
            control = model(inputs, lengths, control="value_cycle")
            for key in ("role_attention", "role_vectors", "binding_attention"):
                torch.testing.assert_close(control[key], plain[key], rtol=0, atol=0)
            role_vectors = plain["role_vectors"]
            core = model.core
            query = core.query_projection(
                core.compound_norm(role_vectors[:, 4, :2].reshape(2, 1, 128))
            )
            keys = core.key_projection(
                core.compound_norm(role_vectors[:, :4, :2].reshape(2, 4, 128))
            )
            keys = torch.cat((keys, core.null_key.expand(2, 1, -1)), dim=1)
            weights = torch.softmax(query @ keys.transpose(-2, -1) / 8.0, dim=-1)
            torch.testing.assert_close(
                weights[:, 0], plain["binding_attention"], rtol=0, atol=0
            )
            values = core.value_projection(core.location_norm(role_vectors[:, :4, 2]))
            for name, expected in (("none", plain), ("value_cycle", control)):
                ordered = values if name == "none" else values[:, [3, 0, 1, 2]]
                complete = torch.cat((ordered, core.null_value.expand(2, 1, -1)), dim=1)
                hidden = core.output_norm(core.output_projection(weights @ complete))
                torch.testing.assert_close(
                    core.lm_head(hidden)[:, 0], expected["logits"], rtol=0, atol=0
                )
            self.assertFalse(torch.equal(plain["logits"], control["logits"]))
            # The unused question-location mixture cannot affect a binding answer.
            changed_roles = plain["role_attention"].clone()
            changed_roles[:, 4, 2] = 0
            changed_roles[:, 4, 2, 0] = 1
            with patch.object(model.reader, "forward", return_value=changed_roles):
                unused = model(inputs, lengths)
            torch.testing.assert_close(
                unused["logits"], plain["logits"], rtol=0, atol=0
            )
        self.assertEqual(_tensor_mapping_cid(model.state_dict()), before_state)
        self.assertTrue(torch.equal(torch.get_rng_state(), before_rng))
        with self.assertRaisesRegex(ValueError, "unknown"):
            model(inputs, lengths, control="entity_filter")
        model.train()
        with self.assertRaisesRegex(ValueError, "forbidden during training"):
            model(inputs, lengths, control="value_cycle")


if __name__ == "__main__":
    unittest.main()
