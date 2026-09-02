from __future__ import annotations

import copy
import unittest

import torch

from r4_softmax_trainer.zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from r4_softmax_trainer.zoology_release import development as release


def _tiny_loader_tensors() -> dict[str, torch.Tensor]:
    train_rows = 32
    test_rows = 16
    train_inputs = torch.arange(train_rows).unsqueeze(1).repeat(1, 64) % 8192
    test_inputs = (torch.arange(test_rows) + 100).unsqueeze(1).repeat(1, 64) % 8192
    return {
        "train_inputs": train_inputs.long(),
        "train_positions": torch.tensor([[8, 10, 12, 14]]).repeat(train_rows, 1),
        "train_targets": torch.tensor([[4096, 4097, 4098, 4099]]).repeat(train_rows, 1),
        "test_inputs": test_inputs.long(),
        "test_positions": torch.tensor([[8, 10, 12, 14]]).repeat(test_rows, 1),
        "test_targets": torch.tensor([[4096, 4097, 4098, 4099]]).repeat(test_rows, 1),
    }


def _loader_trajectory(*, consume_test: bool) -> tuple[list[int], list[int]]:
    set_zoology_seed(release.SOURCE_MODEL_SEED)
    release._new_model()  # Consume the exact released initialization RNG first.
    train, test = release._loaders(_tiny_loader_tensors())
    first = next(iter(train))[0][:, 0].tolist()
    if consume_test:
        next(iter(test))
    second = next(iter(train))[0][:, 0].tolist()
    return first, second


class ZoologyReleaseTests(unittest.TestCase):
    def test_tensor_container_bytes_are_deterministic(self) -> None:
        left = {
            "z": torch.tensor([3, 4], dtype=torch.long),
            "a": torch.tensor([1, 2], dtype=torch.long),
        }
        right = {"a": left["a"], "z": left["z"]}
        self.assertEqual(
            release._canonical_safetensors(left),
            release._canonical_safetensors(right),
        )

    def test_locked_source_learning_rates_bind_decimal_and_float_hex(self) -> None:
        contract = release._learning_rate_contract()
        self.assertEqual(
            release.LEARNING_RATES,
            (
                0.00046415888336127773,
                0.0001,
                0.002154434690031882,
                0.01,
            ),
        )
        self.assertEqual(contract["execution_source_indices"], [1, 0, 2, 3])
        self.assertEqual(
            [record["float_hex"] for record in contract["source_order"]],
            [
                "0x1.a36e2eb1c432dp-14",
                "0x1.e6b4b396428e5p-12",
                "0x1.1a62d511f2b4fp-9",
                "0x1.47ae147ae147bp-7",
            ],
        )

    def test_source_threshold_is_strictly_greater_than_99_percent(self) -> None:
        self.assertFalse(release._source_pass(11_880 / 12_000))
        self.assertTrue(release._source_pass(11_881 / 12_000))

    def test_train_and_test_dataloaders_share_source_rng_trajectory(self) -> None:
        with_test = _loader_trajectory(consume_test=True)
        self.assertEqual(with_test, _loader_trajectory(consume_test=True))
        self.assertEqual(with_test[0], _loader_trajectory(consume_test=False)[0])
        self.assertNotEqual(with_test[1], _loader_trajectory(consume_test=False)[1])

    def test_query_only_projection_matches_full_loss_and_gradients(self) -> None:
        config = ZoologyFigure2Config(
            vocab_size=32,
            d_model=8,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=16,
            attention_dropout=0.0,
            embed_dropout=0.0,
            resid_dropout=0.0,
        )
        set_zoology_seed(123)
        full = ZoologyFigure2Model(config)
        selected = copy.deepcopy(full)
        inputs = torch.tensor([[2, 20, 4, 21, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0]])
        positions = torch.tensor([[4, 6]])
        targets = torch.tensor([[20, 21]])
        labels = torch.full_like(inputs, -100)
        labels.scatter_(1, positions, targets)

        full_output = full.forward_full(inputs, labels)
        selected_output = selected.forward_selected(inputs, positions, targets)
        assert full_output.loss is not None
        assert selected_output.loss is not None
        full_output.loss.backward()
        selected_output.loss.backward()

        torch.testing.assert_close(full_output.loss, selected_output.loss)
        full_gradients = dict(full.named_parameters())
        selected_gradients = dict(selected.named_parameters())
        self.assertEqual(set(full_gradients), set(selected_gradients))
        for name in full_gradients:
            torch.testing.assert_close(
                full_gradients[name].grad,
                selected_gradients[name].grad,
                rtol=2e-5,
                atol=2e-6,
            )

    def test_preflight_selects_fastest_stable_eligible_cpu_plan(self) -> None:
        records = []
        for threads, projected, stable in (
            (1, 200.0, True),
            (4, 90.0, True),
            (8, 70.0, True),
        ):
            records.append(
                {
                    "plan": {"threads": threads},
                    "stable": stable,
                    "projected_arm_seconds": projected,
                    "peak_rss_bytes": 1_000_000,
                }
            )
        selected = release._select_plan(records)
        self.assertTrue(selected["available"])
        self.assertEqual(selected["selected_plan"], {"threads": 8})


if __name__ == "__main__":
    unittest.main()
