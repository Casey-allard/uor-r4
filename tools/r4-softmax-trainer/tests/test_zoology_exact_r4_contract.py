"""New cross-root development wiring; inherited attention checks are reused."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_exact_r4_inference import contract


class DevelopmentWiringTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tensors = {
            "test_inputs": torch.arange(1024 * 120).reshape(1024, 120) % 4096,
            "test_positions": torch.arange(8).expand(1024, 8).clone() + 100,
            "test_targets": torch.arange(1024 * 8).reshape(1024, 8) % 4096,
        }
        self.preparation = {
            "source": {
                "root": "/retained-model",
                "dataset": {"root": "/retained-data", "path": "data/exact.safetensors"},
            }
        }

    def _read(self) -> tuple[dict[str, torch.Tensor], list[str]]:
        calls: list[str] = []
        tensors = self.tensors

        class Reader:
            def __enter__(self):
                return self

            def __exit__(self, *args):
                return False

            def get_tensor(self, name):
                calls.append(name)
                return tensors[name]

        with patch.object(contract, "safe_open", return_value=Reader()) as opened:
            result = contract.load_development(self.preparation)
        opened.assert_called_once_with(
            Path("/retained-data/data/exact.safetensors"), framework="pt", device="cpu"
        )
        return result, calls

    def test_separate_data_root_only_development_and_canonical_rows(self) -> None:
        result, calls = self._read()
        self.assertEqual(calls, ["test_inputs", "test_positions", "test_targets"])
        for key, tensor in self.tensors.items():
            self.assertTrue(torch.equal(result[key], tensor))
        self.assertEqual(result["test_inputs"].shape, (1024, 120))
        self.assertEqual(result["test_targets"].shape, (1024, 8))

    def test_rejects_old_shape_and_out_of_domain_tokens_or_positions(self) -> None:
        cases = (
            ("test_inputs", self.tensors["test_inputs"][:, :64]),
            ("test_inputs", torch.full((1024, 120), 4096, dtype=torch.long)),
            ("test_targets", torch.full((1024, 8), -1, dtype=torch.long)),
            ("test_positions", torch.full((1024, 8), 120, dtype=torch.long)),
        )
        for key, wrong in cases:
            with self.subTest(key=key):
                original = self.tensors[key]
                self.tensors[key] = wrong
                with self.assertRaises(ValueError):
                    self._read()
                self.tensors[key] = original

    def test_preparation_rejects_changed_policy_and_bound_inputs(self) -> None:
        body = {
            "issue": contract.ISSUE,
            "policy": contract.POLICY,
            "evaluation": dict(contract.EVALUATION),
            "source": {"root": "/model"},
            "frames": {"root": "/frames"},
            "implementation": {"tree_cid": "frozen"},
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(contract.prior, "_envelope", return_value=body):
                with (
                    patch.object(
                        contract,
                        "_bindings",
                        return_value={
                            "implementation": {"tree_cid": "changed"},
                        },
                    ),
                    self.assertRaisesRegex(ValueError, "binding changed"),
                ):
                    contract.validate_preparation(root)
                body["evaluation"] = {**contract.EVALUATION, "rows": 3000}
                with self.assertRaisesRegex(ValueError, "policy changed"):
                    contract.validate_preparation(root)


if __name__ == "__main__":
    unittest.main()
