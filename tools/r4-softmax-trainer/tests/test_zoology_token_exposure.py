"""Focused synthetic diagnostic checks; no retained-model outcome access."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch
from test_zoology_language_r4_attention import _inputs, _model
from test_zoology_r4_attention import _synthetic_frames

from r4_softmax_trainer.zoology_token_exposure import campaign, contract
from r4_softmax_trainer.zoology_token_exposure.diagnostic import (
    METRICS,
    displacement_metrics,
    measure_batch,
)


class TokenExposureTests(unittest.TestCase):
    def test_mass_triangle_and_cancellation_have_distinct_zero_cases(self):
        weights = torch.tensor([[[0.5, 0.5]], [[0.25, 0.75]], [[0.0, 1.0]]])
        changed = torch.tensor([[True, True], [True, False], [True, False]])
        delta = torch.zeros(3, 2, 64, dtype=torch.float64)
        delta[0, :, 0] = torch.tensor([2.0, -2.0])
        delta[1, 0, 0], delta[2, 0, 0] = 4, 4
        coherent = torch.zeros(3, 1, 64)
        controlled = coherent.clone()
        controlled[1, 0, 0] = 1
        metrics, _ = displacement_metrics(weights, changed, delta, coherent, controlled)
        torch.testing.assert_close(
            metrics[:, 0, 0], torch.tensor([1.0, 0.25, 0.0], dtype=torch.float64)
        )
        torch.testing.assert_close(
            metrics[:, 0, 2], torch.tensor([2.0, 1.0, 0.0], dtype=torch.float64)
        )
        torch.testing.assert_close(
            metrics[:, 0, 3], torch.tensor([0.0, 1.0, 0.0], dtype=torch.float64)
        )
        torch.testing.assert_close(
            metrics[:, 0, 6], torch.tensor([0.0, 1.0, 0.0], dtype=torch.float64)
        )
        stationary, _ = displacement_metrics(
            weights[:1], changed[:1], delta[:1] * 0, coherent[:1], coherent[:1]
        )
        self.assertEqual(stationary[0, 0, 0], 1)
        self.assertEqual(stationary[0, 0, 2], 0)
        with self.assertRaisesRegex(ValueError, "nonnegative"):
            displacement_metrics(weights - 2, changed, delta, coherent, controlled)

    def test_actual_matrix_equality_padding_used_role_mask_and_no_downstream(self):
        model, frames = _model(), _synthetic_frames()
        inputs, lengths = _inputs()
        before = {key: value.clone() for key, value in model.state_dict().items()}
        handles = [
            child.register_forward_pre_hook(campaign._reject_downstream)
            for name, child in model.core.named_children()
            if name != "embedding"
        ]
        handles.append(model.register_forward_pre_hook(campaign._reject_downstream))
        try:
            result = measure_batch(model, inputs, lengths, frames)
            valid = torch.arange(inputs.shape[-1]) < lengths.unsqueeze(-1)
            changed = measure_batch(
                model, inputs.masked_fill(~valid, 999999), lengths, frames
            )
            self.assertTrue(torch.equal(result["metrics"], changed["metrics"]))
            self.assertEqual(result["metrics"].shape, (2, 14, len(METRICS)))
            self.assertEqual(result["coherent"].shape, (2, 5, 3, 64))
            with self.assertRaisesRegex(ValueError, "forbidden"):
                model(inputs, lengths)
            with self.assertRaises(TypeError):
                measure_batch(model, inputs, lengths, frames, targets=torch.zeros(2))
            # Different frame indices with identical matrices are zero exposure.
            frames.frame_matrices[:] = torch.eye(4, dtype=torch.float64)
            stationary = measure_batch(model, inputs, lengths, frames)
            self.assertEqual(stationary["changed_source_matrices"], 0)
            self.assertTrue(bool((stationary["metrics"][:, :, :5] == 0).all()))
            self.assertTrue(
                all(
                    torch.equal(before[key], value)
                    for key, value in model.state_dict().items()
                )
            )
        finally:
            for handle in handles:
                handle.remove()

    def test_summary_uses_recorded_answer_mask_and_excludes_only_undefined_ratios(self):
        metrics = torch.zeros(4, 14, len(METRICS), dtype=torch.float64)
        metrics[0, :, 0], metrics[1, :, 0] = 0.25, 0.75
        metrics[1, :, 2], metrics[1, :, 6] = 2.0, 0.5
        supported = torch.tensor([True, True, False, False])
        recorded_changed = torch.tensor([True, False, True, False])
        summary = campaign._summaries(metrics, supported, recorded_changed)
        changed = summary["supported"]["changed"]["roles"]["fact_0.owner"]
        retained = summary["supported"]["retained"]["roles"]["fact_0.owner"]
        self.assertEqual(changed["changed_attention_mass"]["mean"], 0.25)
        self.assertEqual(changed["weighted_individual_displacement"]["count"], 1)
        self.assertEqual(changed["cancellation_retained_fraction"]["count"], 0)
        self.assertIsNone(changed["cancellation_retained_fraction"]["mean"])
        self.assertEqual(retained["cancellation_retained_fraction"]["mean"], 0.5)
        swapped = campaign._summaries(metrics, supported, ~recorded_changed)
        self.assertEqual(
            swapped["supported"]["changed"]["roles"]["fact_0.owner"][
                "changed_attention_mass"
            ]["mean"],
            0.75,
        )

    def test_preparation_is_exclusive_binds_policy_and_does_not_run_measurement(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, source = (
                (Path(temporary) / "output").resolve(),
                (Path(temporary) / "source").resolve(),
            )
            binding = {
                "historical": {"root": str(source)},
                "construction": [],
                "implementation": {"tree_cid": "synthetic"},
            }
            with (
                patch.object(contract, "_bindings", return_value=binding),
                patch.object(
                    campaign,
                    "_evaluate",
                    side_effect=AssertionError("outcomes forbidden"),
                ),
            ):
                prepared = contract.prepare(root, source)
                self.assertEqual(prepared, contract.validate_preparation(root))
                with self.assertRaises(FileExistsError):
                    contract.prepare(root, source)
                with patch.dict(contract.POLICY, {"threads": 8}):
                    with self.assertRaisesRegex(ValueError, "policy"):
                        contract.validate_preparation(root)


if __name__ == "__main__":
    unittest.main()
