"""Synthetic mechanism, initialization, gradient and artifact checks; no fitting."""

from __future__ import annotations

import copy
import itertools
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.zoology_compound_binding import model as compound
from r4_softmax_trainer.zoology_release.development import (
    _artifact_payload,
    _tensor_mapping_cid,
)


def _inputs() -> tuple[torch.Tensor, torch.Tensor]:
    inputs = torch.arange(100, 141).unsqueeze(0).repeat(2, 1)
    inputs[:, [1, 9, 17, 25]] = torch.tensor([12, 13, 14, 15])
    inputs[:, [4, 12, 20, 28]] = torch.tensor([20, 21, 22, 23])
    inputs[:, [7, 15, 23, 31]] = torch.tensor([32, 33, 34, 35])
    inputs[:, 35] = torch.tensor([12, 13])
    inputs[:, 37] = torch.tensor([20, 21])
    return inputs, torch.full((2, 1), 37, dtype=torch.long)


def _state_cid(model: compound.CompoundBindingModel) -> str:
    return _tensor_mapping_cid(
        {
            key: value
            for key, value in model.state_dict().items()
            if key != "lm_head.weight"
        }
    )


class CompoundBindingModelTests(unittest.TestCase):
    def test_parameter_count_single_initialization_and_rng(self) -> None:
        shapes = {
            "embedding.weight": (4096, 64),
            "query_projection.weight": (64, 128),
            "key_projection.weight": (64, 128),
            "value_projection.weight": (64, 64),
            "output_projection.weight": (64, 64),
            "null_key": (64,),
            "null_value": (64,),
        }
        torch.manual_seed(123)
        expected = {
            name: torch.empty(shape).normal_(mean=0.0, std=0.02)
            for name, shape in shapes.items()
        }
        expected_rng = torch.get_rng_state().clone()
        torch.manual_seed(123)
        model = compound.CompoundBindingModel()
        self.assertTrue(torch.equal(torch.get_rng_state(), expected_rng))
        for name, tensor in expected.items():
            torch.testing.assert_close(model.state_dict()[name], tensor, rtol=0, atol=0)
        self.assertEqual(model.parameter_count(), 286976)
        self.assertIs(model.embedding.weight, model.lm_head.weight)
        self.assertIsNone(model.compound_norm.weight)
        self.assertIsNone(model.location_norm.weight)
        self.assertTrue(torch.equal(model.output_norm.weight, torch.ones(64)))
        self.assertTrue(torch.equal(model.output_norm.bias, torch.zeros(64)))
        self.assertFalse(
            any(isinstance(module, torch.nn.Dropout) for module in model.modules())
        )
        with self.assertRaisesRegex(ValueError, "frozen"):
            compound.CompoundBindingConfig(d_model=32)

    def test_causal_fields_permutation_equivariance_and_value_control(self) -> None:
        torch.manual_seed(123)
        model = compound.CompoundBindingModel().eval()
        inputs, positions = _inputs()
        before_state = _state_cid(model)
        before_rng = torch.get_rng_state().clone()
        with torch.inference_mode():
            original = model.forward_selected(inputs, positions, return_attention=True)
            self.assertEqual(original.logits.shape, (2, 1, 4096))
            self.assertEqual(original.attention_weights[0].shape, (2, 1, 1, 5))
            self.assertIsNone(original.loss)
            unread = inputs.clone()
            role_positions = {1, 9, 17, 25, 4, 12, 20, 28, 7, 15, 23, 31, 35, 37}
            unread[
                :,
                [position for position in range(41) if position not in role_positions],
            ] = -999
            labels_a = model.forward_selected(
                unread, positions, torch.tensor([[32], [33]])
            )
            labels_b = model.forward_selected(
                unread, positions, torch.tensor([[11], [11]])
            )
            torch.testing.assert_close(labels_a.logits, original.logits, rtol=0, atol=0)
            torch.testing.assert_close(labels_b.logits, original.logits, rtol=0, atol=0)
            self.assertNotEqual(float(labels_a.loss), float(labels_b.loss))

            # All 4! coherent fact permutations are one synthetic batched forward.
            permutations = list(itertools.permutations(range(4)))
            permuted = inputs[:1].repeat(len(permutations), 1)
            blocks = inputs[0, 1:33].reshape(4, 8)
            for row, permutation in enumerate(permutations):
                permuted[row, 1:33] = blocks[list(permutation)].reshape(-1)
            permuted_output = model.forward_selected(
                permuted, torch.full((len(permutations), 1), 37), return_attention=True
            )
            torch.testing.assert_close(
                permuted_output.logits,
                original.logits[:1].expand(len(permutations), -1, -1),
                rtol=1.0e-5,
                atol=1.0e-6,
            )
            for row, permutation in enumerate(permutations):
                torch.testing.assert_close(
                    permuted_output.attention_weights[0][row],
                    original.attention_weights[0][0, :, :, [*permutation, 4]],
                    rtol=1.0e-6,
                    atol=1.0e-7,
                )

            values = []
            contexts = []
            value_hook = model.value_projection.register_forward_hook(
                lambda _module, _args, output: values.append(output.clone())
            )
            context_hook = model.output_projection.register_forward_pre_hook(
                lambda _module, args: contexts.append(args[0].clone())
            )
            try:
                controlled = model.forward_selected(
                    inputs, positions, return_attention=True, control="value_cycle"
                )
            finally:
                value_hook.remove()
                context_hook.remove()
            torch.testing.assert_close(
                controlled.attention_weights[0],
                original.attention_weights[0],
                rtol=0,
                atol=0,
            )
            control_values = torch.cat(
                (values[0][:, [3, 0, 1, 2]], model.null_value.expand(2, 1, -1)), dim=1
            )
            torch.testing.assert_close(
                contexts[0],
                original.attention_weights[0][:, 0] @ control_values,
                rtol=0,
                atol=0,
            )
            self.assertFalse(torch.equal(controlled.logits, original.logits))
        self.assertEqual(_state_cid(model), before_state)
        self.assertTrue(torch.equal(torch.get_rng_state(), before_rng))
        with self.assertRaisesRegex(ValueError, "selected position 37"):
            model.forward_selected(inputs, positions + 1)
        with self.assertRaisesRegex(ValueError, "unknown"):
            model.forward_selected(inputs, positions, control="query_mask")
        model.train()
        with self.assertRaisesRegex(ValueError, "forbidden during training"):
            model.forward_selected(inputs, positions, control="value_cycle")

    def test_loss_gradients_reach_query_roles_fact_roles_and_every_parameter(
        self,
    ) -> None:
        torch.manual_seed(123)
        model = compound.CompoundBindingModel()
        inputs, positions = _inputs()
        embedded_roles = []

        def retain_roles(_module, _args, output):
            output.retain_grad()
            embedded_roles.append(output)

        handle = model.embedding.register_forward_hook(retain_roles)
        try:
            output = model.forward_selected(
                inputs, positions, torch.tensor([[32], [11]])
            )
            output.loss.backward()
        finally:
            handle.remove()
        self.assertEqual(len(embedded_roles), 5)
        for role in embedded_roles:
            self.assertTrue(bool(torch.isfinite(role.grad).all()))
            self.assertGreater(float(role.grad.abs().sum()), 0.0)
        for name, parameter in model.named_parameters():
            with self.subTest(parameter=name):
                self.assertIsNotNone(parameter.grad)
                self.assertTrue(bool(torch.isfinite(parameter.grad).all()))
                self.assertGreater(float(parameter.grad.abs().sum()), 0.0)

    def test_artifact_reload_requires_policy_and_exact_file_and_state(self) -> None:
        torch.manual_seed(123)
        model = compound.CompoundBindingModel().eval()
        inputs, positions = _inputs()
        with torch.inference_mode():
            expected = model.forward_selected(inputs, positions, return_attention=True)
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
                        "state_cid": _state_cid(model),
                        "config": dict(compound.MODEL_CONFIG),
                        "model_policy": copy.deepcopy(compound.MODEL_POLICY),
                    },
                }
            }
            loaded = compound.load_model(preparation)
            self.assertEqual(_state_cid(loaded), _state_cid(model))
            self.assertIs(loaded.embedding.weight, loaded.lm_head.weight)
            self.assertFalse(loaded.training)
            self.assertTrue(
                all(not parameter.requires_grad for parameter in loaded.parameters())
            )
            with torch.inference_mode():
                observed = loaded.forward_selected(
                    inputs, positions, return_attention=True
                )
            torch.testing.assert_close(observed.logits, expected.logits, rtol=0, atol=0)
            torch.testing.assert_close(
                observed.attention_weights[0],
                expected.attention_weights[0],
                rtol=0,
                atol=0,
            )
            with patch.object(
                Path, "read_bytes", side_effect=AssertionError("must stay unopened")
            ):
                missing = copy.deepcopy(preparation)
                del missing["source"]["model"]["model_policy"]
                with self.assertRaisesRegex(ValueError, "policy or config"):
                    compound.load_model(missing)
            wrong_state = copy.deepcopy(preparation)
            wrong_state["source"]["model"]["state_cid"] = "blake3:wrong"
            with self.assertRaisesRegex(ValueError, "tensor identity"):
                compound.load_model(wrong_state)
            (root / "model.safetensors").write_bytes(payload + b"changed")
            with self.assertRaisesRegex(ValueError, "file changed"):
                compound.load_model(preparation)


if __name__ == "__main__":
    unittest.main()
