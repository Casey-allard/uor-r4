from __future__ import annotations

import copy
import math
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.zoology_continuation import contract


def _checkpoint_fixture():
    model = {f"parameter_{index}": torch.tensor([float(index)]) for index in range(19)}
    model["backbone.embeddings.word_embeddings.weight"] = torch.tensor([23.0])
    model["lm_head.weight"] = model[
        "backbone.embeddings.word_embeddings.weight"
    ].clone()
    history = [
        {
            "block": index,
            "completed_updates": index * 196,
            "strict_source_pass": False,
            "train": {"updates": 196, "decisions": 196 * 512 * 8},
            "development": {"decisions": 8192, "top1_rate": 0.5},
        }
        for index in range(1, 21)
    ]
    state_cid = contract.parent_contract.previous.release._tensor_mapping_cid(
        {name: value for name, value in model.items() if name != "lm_head.weight"}
    )
    primary = {"history": history, "artifact": {"config": {}, "state_cid": state_cid}}
    preparation = {
        "parent_primary": primary,
        "parent_binding_cid": cid_bytes(b"parent binding"),
        "parent_history_cid": cid_bytes(canonical_json_bytes(history)),
    }
    saved = {
        "binding_cid": preparation["parent_binding_cid"],
        "model_config": {},
        "history": history,
        "completed_updates": 3920,
        "model": model,
        "accumulator": {"updates": 0, "decisions": 0, "correct": 0, "loss_sum": 0.0},
        "scheduler": {"last_epoch": 20, "T_max": 64, "eta_min": 0.0},
        "optimizer": {
            "param_groups": [
                {
                    "lr": contract.parent_development.LEARNING_RATE
                    * (1 + math.cos(math.pi * 20 / 64))
                    / 2
                }
            ],
            "state": {0: {"step": torch.tensor(3920)}},
        },
        "sampler": {"cycles": 245, "cursor": 8192, "permutation": torch.arange(8192)},
        "torch_rng_state": torch.get_rng_state().clone(),
        "evaluation_rng": torch.get_rng_state().clone(),
    }
    return saved, preparation


class ZoologyContinuationContractTests(unittest.TestCase):
    def test_continuation_keeps_mechanism_and_adds_only_one_bounded_window(self):
        parent = contract.parent_contract.training_contract()
        continuation = contract.training_contract()
        for name in (
            "learning_rate",
            "learning_rate_float_hex",
            "batch_size",
            "optimizer",
            "betas",
            "weight_decay",
            "scheduler",
            "scheduler_step",
            "train_rows",
            "development_rows",
            "d_model",
            "n_layers",
            "num_heads",
            "attention_dropout",
            "embed_dropout",
            "resid_dropout",
            "sampler_cursor",
        ):
            self.assertEqual(continuation[name], parent[name], name)
        self.assertEqual(continuation["parent_optimizer_updates"], 3920)
        self.assertEqual(continuation["maximum_additional_optimizer_updates"], 3920)
        self.assertEqual(continuation["maximum_optimizer_updates"], 7840)
        self.assertEqual(continuation["maximum_source_blocks"], 40)
        self.assertEqual(continuation["additional_budget_seconds"], 1800)
        self.assertEqual(
            continuation["maximum_additional_train_query_presentations"], 16_056_320
        )
        self.assertEqual(
            continuation["maximum_additional_development_query_presentations"], 163_840
        )
        self.assertEqual(continuation["automatic_further_extension"], "FORBIDDEN")

    def test_parent_state_validation_rejects_reset_or_tampered_components(self):
        saved, preparation = _checkpoint_fixture()
        contract._validate_checkpoint_state(saved, preparation)
        mutations = (
            ("completed_updates", 0),
            (
                "accumulator",
                {"updates": 1, "decisions": 4096, "correct": 0, "loss_sum": 0.0},
            ),
            ("scheduler", {"last_epoch": 0, "T_max": 64, "eta_min": 0.0}),
            (
                "sampler",
                {"cycles": 245, "cursor": 0, "permutation": torch.arange(8192)},
            ),
            ("torch_rng_state", torch.tensor([1], dtype=torch.long)),
        )
        for name, value in mutations:
            with self.subTest(name=name):
                changed = copy.deepcopy(saved)
                changed[name] = value
                with self.assertRaises(ValueError):
                    contract._validate_checkpoint_state(changed, preparation)
        changed = copy.deepcopy(saved)
        changed["optimizer"]["state"][0]["step"] = torch.tensor(0)
        with self.assertRaisesRegex(ValueError, "optimizer counters"):
            contract._validate_checkpoint_state(changed, preparation)
        changed = copy.deepcopy(saved)
        changed["model"]["lm_head.weight"] += 1
        with self.assertRaisesRegex(ValueError, "tied vocabulary head"):
            contract._validate_checkpoint_state(changed, preparation)

    def test_tampered_parent_checkpoint_is_rejected_before_deserialization(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / contract.PARENT_CHECKPOINT["path"]
            path.parent.mkdir()
            path.write_bytes(b"not the frozen parent checkpoint")
            preparation = {
                "parent_root": str(root),
                "parent_checkpoint": dict(contract.PARENT_CHECKPOINT),
            }
            with patch.object(
                torch, "load", side_effect=AssertionError("unverified state loaded")
            ) as load:
                with self.assertRaisesRegex(ValueError, "parent payload changed"):
                    contract.load_checkpoint(preparation)
                load.assert_not_called()


if __name__ == "__main__":
    unittest.main()
