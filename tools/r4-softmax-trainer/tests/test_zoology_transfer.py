from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from r4_softmax_trainer.zoology_release import development as release
from r4_softmax_trainer.zoology_transfer import development as transfer


def _model() -> ZoologyFigure2Model:
    return ZoologyFigure2Model(
        ZoologyFigure2Config(
            vocab_size=32,
            d_model=8,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=16,
            attention_dropout=0.0,
            embed_dropout=0.0,
        )
    )


def _tensors() -> dict[str, torch.Tensor]:
    inputs = torch.tensor([[2, 20, 4, 21, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0]]).repeat(
        8, 1
    )
    positions = torch.tensor([[4, 6]]).repeat(8, 1)
    targets = torch.tensor([[20, 21]]).repeat(8, 1)
    return {
        f"{split}_{kind}": value.clone()
        for split in ("train", "test")
        for kind, value in (
            ("inputs", inputs),
            ("positions", positions),
            ("targets", targets),
        )
    }


def _score(rate: float) -> dict[str, object]:
    return {
        "decisions": 16,
        "top1_correct": int(rate * 16),
        "top1_rate": rate,
        "nll_nats": 0.1,
        "selected_logits_cid": "blake3:" + "1" * 64,
    }


def _preflight() -> dict[str, object]:
    plans = [
        {
            "plan": {"threads": threads},
            "stable": True,
            "repeat_deterministic": True,
            "peak_rss_bytes": 1000,
            "projected_primary_seconds": wall,
        }
        for threads, wall in ((1, 901), (4, 100), (8, 120))
    ]
    return release._with_cid(
        {
            "preparation_cid": "prep",
            "implementation": {},
            "c0": {"passed": True},
            "passed": True,
            "plans": plans,
            "selected": transfer._select_plan(plans),
        },
        "preflight_cid",
    )


class ZoologyTransferTests(unittest.TestCase):
    def test_strict_first_pass_and_no_post_pass_epochs(self) -> None:
        self.assertFalse(
            transfer._history_pass(
                [{"epoch": 1, "strict_source_pass": False, "development": _score(0.99)}]
            )
        )
        self.assertTrue(
            transfer._history_pass(
                [{"epoch": 1, "strict_source_pass": True, "development": _score(1.0)}]
            )
        )
        with self.assertRaisesRegex(ValueError, "beyond"):
            transfer._history_pass(
                [
                    {
                        "epoch": 1,
                        "strict_source_pass": True,
                        "development": _score(1.0),
                    },
                    {
                        "epoch": 2,
                        "strict_source_pass": True,
                        "development": _score(1.0),
                    },
                ]
            )

    def test_passing_checkpoint_finalizes_without_another_step_or_evaluation(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(transfer, "_new_model", side_effect=_model),
        ):
            root = Path(directory)
            with (
                patch.object(transfer, "_score", return_value=_score(1.0)),
                patch.object(
                    transfer,
                    "_write_or_match",
                    side_effect=RuntimeError("simulated finalization crash"),
                ),
            ):
                with self.assertRaisesRegex(RuntimeError, "simulated"):
                    transfer._run_primary(
                        root, _tensors(), threads=4, binding_cid="binding"
                    )
            checkpoint = torch.load(root / "primary/checkpoint.pt", weights_only=True)
            self.assertEqual(checkpoint["completed_epochs"], 1)
            with (
                patch.object(
                    transfer,
                    "_train_epoch",
                    side_effect=AssertionError("extra optimizer step"),
                ),
                patch.object(
                    transfer, "_score", side_effect=AssertionError("extra evaluation")
                ),
            ):
                result = transfer._run_primary(
                    root, _tensors(), threads=4, binding_cid="binding"
                )
            self.assertTrue(result["passed"])
            self.assertEqual(result["epochs"], 1)
            state = {
                name: tensor
                for name, tensor in checkpoint["model"].items()
                if name != "lm_head.weight"
            }
            self.assertEqual(
                result["artifact"]["state_cid"], release._tensor_mapping_cid(state)
            )
            with self.assertRaisesRegex(ValueError, "binding changed"):
                transfer._run_primary(
                    root, _tensors(), threads=4, binding_cid="different"
                )

    def test_scheduler_steps_on_miss_only(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(transfer, "_new_model", side_effect=_model),
            patch.object(transfer, "MAXIMUM_EPOCHS", 2),
            patch.object(transfer, "_score", side_effect=[_score(0.5), _score(1.0)]),
        ):
            root = Path(directory)
            result = transfer._run_primary(
                root, _tensors(), threads=4, binding_cid="binding"
            )
            checkpoint = torch.load(root / "primary/checkpoint.pt", weights_only=True)
            self.assertEqual(result["epochs"], 2)
            self.assertEqual(checkpoint["scheduler"]["last_epoch"], 1)
            self.assertEqual(
                result["history"][1]["learning_rate"], transfer.LEARNING_RATE / 2
            )

    def test_train_and_development_share_loader_rng(self) -> None:
        tensors = _tensors()
        tensors["train_inputs"][:, 0] = torch.arange(8)

        def trajectory(consume_development: bool) -> tuple[list[int], list[int]]:
            set_zoology_seed(123)
            train, development = transfer._loaders(tensors)
            first = next(iter(train))[0][:, 0].tolist()
            if consume_development:
                list(development)
            return first, next(iter(train))[0][:, 0].tolist()

        self.assertEqual(trajectory(True), trajectory(True))
        self.assertNotEqual(trajectory(True)[1], trajectory(False)[1])

    def test_cpu_selection_uses_fastest_eligible_plan(self) -> None:
        records = [
            {
                "plan": {"threads": threads},
                "stable": True,
                "repeat_deterministic": True,
                "peak_rss_bytes": 1000,
                "projected_primary_seconds": wall,
            }
            for threads, wall in ((1, 901), (4, 100), (8, 120))
        ]
        self.assertEqual(transfer._select_plan(records)["plan"]["threads"], 4)
        records[1]["repeat_deterministic"] = False
        self.assertEqual(transfer._select_plan(records)["plan"]["threads"], 8)

    def test_primary_miss_never_opens_binding_control(self) -> None:
        preparation = {
            "preparation_cid": "prep",
            "implementation": {},
            "dataset": {},
            "read_ledger": {},
        }
        preflight = _preflight()
        primary = {"passed": False, "status": "STOCK_CELL_TRANSFER_MISS"}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release._write_exclusive_json(root / transfer.PREFLIGHT_PATH, preflight)
            with (
                patch.object(
                    transfer.contract, "validate_preparation", return_value=preparation
                ),
                patch.object(
                    transfer.contract, "load_dataset", return_value=_tensors()
                ),
                patch.object(transfer, "_run_primary", return_value=primary),
                patch.object(
                    transfer.contract,
                    "load_control",
                    side_effect=AssertionError("control opened"),
                ) as control_load,
            ):
                result = transfer.run_transfer(root)
            control_load.assert_not_called()
            self.assertEqual(result["control"]["status"], "NOT_RUN_PRIMARY_MISS")
            self.assertEqual(result["read_ledger"]["control_query_decisions"], 0)

    def test_preflight_and_primary_causal_bindings_are_checked(self) -> None:
        preparation = {"preparation_cid": "prep", "implementation": {}, "dataset": {}}
        preflight = _preflight()
        transfer._validate_preflight(preparation, preflight)
        with self.assertRaisesRegex(ValueError, "preflight no longer"):
            transfer._validate_preflight(
                {**preparation, "preparation_cid": "stale"}, preflight
            )
        binding, binding_cid = transfer._primary_binding(preparation, preflight)
        changed = {**preflight, "selected": {"plan": {"threads": 8}}}
        self.assertNotEqual(
            transfer._primary_binding(preparation, changed)[1], binding_cid
        )
        with self.assertRaisesRegex(ValueError, "admission differs"):
            transfer._validate_preflight(preparation, changed)
        self.assertEqual(binding["plan"], {"threads": 4})
        result = release._with_cid(
            {
                "preparation_cid": "prep",
                "preflight_cid": preflight["preflight_cid"],
                "implementation": {},
                "dataset": {},
                "primary": {"binding_cid": "stale"},
            },
            "result_cid",
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release._write_exclusive_json(root / transfer.PREFLIGHT_PATH, preflight)
            release._write_exclusive_json(root / transfer.RESULT_PATH, result)
            with (
                patch.object(
                    transfer.contract, "validate_preparation", return_value=preparation
                ),
                self.assertRaisesRegex(ValueError, "causal binding"),
            ):
                transfer.verify_transfer(root)


if __name__ == "__main__":
    unittest.main()
