"""Focused model/config/export checks for the frozen #1019 capacity rung."""

from __future__ import annotations

import json
import tempfile
import unittest
from dataclasses import FrozenInstanceError, replace
from pathlib import Path
from unittest import mock

import torch

from r4_softmax_trainer.constants import CAPACITY_MODEL_CONFIG, FROZEN_MODEL_CONFIG
from r4_softmax_trainer.export import export_hugging_face_snapshot
from r4_softmax_trainer.model import (
    EXPECTED_PARAMETER_COUNT,
    R4SoftmaxForCausalLM,
    expected_hf_tensor_names,
    expected_parameter_count,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes, verify_bound_manifest


LEGACY_CONTRACT_CID = "blake3:d2b75bf1a1303a9511e476c7478a18ab356b4be5b819832e706bc01b350ac1f8"
LEGACY_HF_CONFIG_CID = "blake3:1f1ddb6de22f5c81c04d3093eeff8e0991d63b79ee33bc8ff3cf7c68ef0a9497"
LEGACY_PARAMETER_COUNT = 7_155_360
CAPACITY_PARAMETER_COUNT = 13_130_784


def _export_with_stubbed_weights(
    model: R4SoftmaxForCausalLM, root: Path
) -> tuple[dict[str, object], set[str]]:
    tokenizer_path = root / "source-tokenizer.json"
    tokenizer_path.write_bytes(b'{"version":"test"}\n')
    exported_names: set[str] = set()

    def save_stub(
        tensors: dict[str, torch.Tensor],
        filename: str,
        *,
        metadata: dict[str, str],
    ) -> None:
        exported_names.update(tensors)
        if metadata != {"format": "pt"}:
            raise AssertionError(f"unexpected metadata: {metadata}")
        Path(filename).write_bytes(b"stubbed-safetensors\n")

    with mock.patch("r4_softmax_trainer.export.save_file", side_effect=save_stub):
        manifest = export_hugging_face_snapshot(
            model,
            output_dir=root / "export",
            tokenizer_path=tokenizer_path,
            training_result={"result_cid": "blake3:" + "a" * 64},
            dataset_manifest_cid="blake3:" + "b" * 64,
            training_view_manifest_cid="blake3:" + "c" * 64,
            split_policy_cid="blake3:" + "d" * 64,
            run_contract_cid="blake3:" + "e" * 64,
            selected_checkpoint_cid=None,
        )
    verified = verify_bound_manifest(
        root / "export/export-manifest.json", artifact_root=root / "export"
    )
    if verified != manifest:
        raise AssertionError("export manifest did not reproduce")
    return manifest, exported_names


class CapacityModelTests(unittest.TestCase):
    def test_legacy_config_bytes_and_default_model_are_unchanged(self) -> None:
        self.assertEqual(
            cid_bytes(canonical_json_bytes(FROZEN_MODEL_CONFIG.as_contract())),
            LEGACY_CONTRACT_CID,
        )
        self.assertEqual(
            cid_bytes(canonical_json_bytes(FROZEN_MODEL_CONFIG.as_hugging_face_config())),
            LEGACY_HF_CONFIG_CID,
        )
        self.assertEqual(EXPECTED_PARAMETER_COUNT, LEGACY_PARAMETER_COUNT)
        self.assertEqual(expected_parameter_count(FROZEN_MODEL_CONFIG), LEGACY_PARAMETER_COUNT)

        model = R4SoftmaxForCausalLM()
        self.assertIs(model.config, FROZEN_MODEL_CONFIG)
        self.assertEqual(model.parameter_count(), LEGACY_PARAMETER_COUNT)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, names = _export_with_stubbed_weights(model, root)
            config_bytes = (root / "export/config.json").read_bytes()
            self.assertEqual(cid_bytes(config_bytes), LEGACY_HF_CONFIG_CID)
            self.assertEqual(manifest["model_contract"], FROZEN_MODEL_CONFIG.as_contract())
            self.assertEqual(names, expected_hf_tensor_names(FROZEN_MODEL_CONFIG))

    def test_capacity_contract_is_exact_immutable_and_validated(self) -> None:
        config = CAPACITY_MODEL_CONFIG
        config.validate()
        self.assertEqual(config.num_hidden_layers, 12)
        self.assertEqual(config.hidden_size, 288)
        self.assertEqual(config.intermediate_size, 768)
        self.assertEqual(config.num_attention_heads, 6)
        self.assertEqual(config.num_key_value_heads, 6)
        self.assertEqual(config.head_dim, 48)
        self.assertEqual(config.r4_blocks_per_head, 12)
        self.assertEqual(expected_parameter_count(config), CAPACITY_PARAMETER_COUNT)
        with self.assertRaises(FrozenInstanceError):
            config.num_hidden_layers = 6  # type: ignore[misc]
        with self.assertRaisesRegex(ValueError, "num_hidden_layers must be positive"):
            replace(config, num_hidden_layers=0).validate()
        with self.assertRaisesRegex(ValueError, "hidden_size must equal"):
            replace(config, hidden_size=289).validate()

    def test_capacity_forward_and_export_use_explicit_model_config(self) -> None:
        model = R4SoftmaxForCausalLM(CAPACITY_MODEL_CONFIG)
        self.assertIs(model.config, CAPACITY_MODEL_CONFIG)
        self.assertEqual(model.parameter_count(), CAPACITY_PARAMETER_COUNT)

        model.eval()
        with torch.no_grad():
            output = model(torch.tensor([[0, 1, 2]], dtype=torch.long))
        self.assertEqual(tuple(output.logits.shape), (1, 3, CAPACITY_MODEL_CONFIG.vocab_size))
        self.assertIsNone(output.loss)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, names = _export_with_stubbed_weights(model, root)
            exported_config = json.loads((root / "export/config.json").read_text(encoding="utf-8"))
            self.assertEqual(
                exported_config,
                CAPACITY_MODEL_CONFIG.as_hugging_face_config(),
            )
            self.assertEqual(manifest["model_contract"], CAPACITY_MODEL_CONFIG.as_contract())
            self.assertEqual(names, expected_hf_tensor_names(CAPACITY_MODEL_CONFIG))
            self.assertIn("model.layers.11.self_attn.q_proj.weight", names)
            self.assertNotIn("model.layers.12.self_attn.q_proj.weight", names)


if __name__ == "__main__":
    unittest.main()
