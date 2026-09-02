"""Synthetic embedding, state, gradient and adapted-artifact checks; no fitting."""

from __future__ import annotations

import copy
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

import torch
from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from r4_softmax_trainer.zoology_joint_query import model as joint
from r4_softmax_trainer.zoology_r4_inference.campaign import _learned_state_cid
from r4_softmax_trainer.zoology_release.development import _artifact_payload


def _config() -> ZoologyFigure2Config:
    return ZoologyFigure2Config(
        vocab_size=64, d_model=8, n_layers=1, max_position_embeddings=41
    )


class JointQueryModelTests(unittest.TestCase):
    def test_installation_and_constructor_preserve_source_state_and_rng(self) -> None:
        set_zoology_seed(123)
        source = ZoologyFigure2Model(_config())
        source_rng = torch.get_rng_state().clone()
        source_state = {
            name: value.clone() for name, value in source.state_dict().items()
        }
        parameter_ids = [id(value) for value in source.parameters()]
        self.assertIs(joint.install_joint_query_embedding(source), source)
        self.assertTrue(torch.equal(torch.get_rng_state(), source_rng))
        self.assertEqual([id(value) for value in source.parameters()], parameter_ids)
        self.assertEqual(list(source.state_dict()), list(source_state))
        self.assertTrue(
            all(
                torch.equal(source.state_dict()[key], value)
                for key, value in source_state.items()
            )
        )
        self.assertIs(
            source.lm_head.weight, source.backbone.embeddings.word_embeddings.weight
        )
        with self.assertRaisesRegex(ValueError, "already installed"):
            joint.install_joint_query_embedding(source)
        set_zoology_seed(123)
        adapted = joint.ZoologyJointQueryModel(_config())
        self.assertTrue(torch.equal(torch.get_rng_state(), source_rng))
        self.assertEqual(list(adapted.state_dict()), list(source_state))
        self.assertTrue(
            all(
                torch.equal(adapted.state_dict()[key], value)
                for key, value in source_state.items()
            )
        )

    def test_embedding_locality_causality_gradients_and_shape(self) -> None:
        model = joint.ZoologyJointQueryModel(_config())
        embeddings = model.backbone.embeddings
        inputs = torch.arange(41).unsqueeze(0).repeat(2, 1)
        positions = torch.arange(41).unsqueeze(0)
        baseline = embeddings.word_embeddings(inputs) + embeddings.position_embeddings(
            positions
        )
        rng = torch.get_rng_state().clone()
        output = embeddings(inputs, positions)
        self.assertTrue(torch.equal(torch.get_rng_state(), rng))
        torch.testing.assert_close(output[:, :37], baseline[:, :37], rtol=0, atol=0)
        torch.testing.assert_close(output[:, 38:], baseline[:, 38:], rtol=0, atol=0)
        torch.testing.assert_close(
            output[:, 37],
            baseline[:, 37] + embeddings.word_embeddings(inputs[:, 35]),
            rtol=0,
            atol=0,
        )
        changed_future = inputs.clone()
        changed_future[:, 38:] += 1
        torch.testing.assert_close(
            embeddings(changed_future, positions)[:, 37], output[:, 37], rtol=0, atol=0
        )
        output[:, 37].sum().backward()
        expected_word_grad = torch.zeros_like(embeddings.word_embeddings.weight)
        expected_word_grad[35] = 2
        expected_word_grad[37] = 2
        expected_position_grad = torch.zeros_like(embeddings.position_embeddings.weight)
        expected_position_grad[37] = 2
        torch.testing.assert_close(
            embeddings.word_embeddings.weight.grad, expected_word_grad, rtol=0, atol=0
        )
        torch.testing.assert_close(
            embeddings.position_embeddings.weight.grad,
            expected_position_grad,
            rtol=0,
            atol=0,
        )
        with self.assertRaisesRegex(ValueError, "41 input tokens"):
            embeddings(inputs[:, :40], positions[:, :40])
        embeddings.project_in = torch.nn.Identity()
        with self.assertRaisesRegex(ValueError, "source embedding width"):
            embeddings(inputs, positions)
        source = ZoologyFigure2Model(_config())
        source.backbone.embeddings.project_in = torch.nn.Identity()
        with self.assertRaisesRegex(ValueError, "source embedding width"):
            joint.install_joint_query_embedding(source)

    def test_adapted_loader_requires_policy_and_reconstructs_synthetic_output(
        self,
    ) -> None:
        model = joint.ZoologyJointQueryModel(_config()).eval()
        inputs = torch.arange(41).unsqueeze(0)
        positions = torch.tensor([[37]])
        with torch.inference_mode():
            expected = model.forward_selected(inputs, positions).logits
        payload = _artifact_payload(model, learning_rate=0.00046415888336127773)
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "model.safetensors").write_bytes(payload)
            preparation = {
                "source": {
                    "root": str(root),
                    "model": {
                        "path": "model.safetensors",
                        "bytes": len(payload),
                        "cid": cid_bytes(payload),
                        "state_cid": _learned_state_cid(model),
                        "config": asdict(model.config),
                        "query_encoding": dict(joint.QUERY_ENCODING),
                    },
                }
            }
            loaded = joint.load_model(preparation)
            self.assertEqual(list(loaded.state_dict()), list(model.state_dict()))
            self.assertEqual(_learned_state_cid(loaded), _learned_state_cid(model))
            self.assertIs(
                loaded.lm_head.weight, loaded.backbone.embeddings.word_embeddings.weight
            )
            self.assertFalse(loaded.training)
            self.assertTrue(
                all(not parameter.requires_grad for parameter in loaded.parameters())
            )
            with torch.inference_mode():
                observed = loaded.forward_selected(inputs, positions).logits
            torch.testing.assert_close(observed, expected, rtol=0, atol=0)
            with patch.object(
                joint,
                "_load_source_model",
                side_effect=AssertionError("weights must stay unopened"),
            ):
                missing = copy.deepcopy(preparation)
                del missing["source"]["model"]["query_encoding"]
                with self.assertRaisesRegex(
                    ValueError, "encoding differs or is absent"
                ):
                    joint.load_model(missing)
                changed = copy.deepcopy(preparation)
                changed["source"]["model"]["query_encoding"]["owner_position"] = 36
                with self.assertRaisesRegex(
                    ValueError, "encoding differs or is absent"
                ):
                    joint.load_model(changed)


if __name__ == "__main__":
    unittest.main()
