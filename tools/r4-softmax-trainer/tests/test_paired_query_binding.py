"""Focused checks for C1-SB5 paired-query conditional binding."""

from __future__ import annotations

import unittest
from dataclasses import replace
from types import SimpleNamespace
from typing import Any
from unittest.mock import patch

import torch

from r4_softmax_trainer.model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from r4_softmax_trainer.paired_query_binding import (
    BINDING_BLOCKS,
    BINDING_HEAD_PARAMETER_COUNT,
    BINDING_RANK,
    FIT_SEED,
    FLIP_MARGIN,
    MARGIN,
    OPTIMIZER_STEPS,
    POLICY,
    RECORDS_PER_STEP,
    STEPS_PER_EPOCH,
    TRAINABLE_PARAMETER_COUNT,
    EncodedPairedQueryBindingDataset,
    PairedQueryBindingAdapterConfig,
    PairedQueryBindingFitConfig,
    PairedQueryBindingOutput,
    PairedQueryTokenBatch,
    R4PairedQueryCandidateMatrix,
    evaluate_paired_query_binding,
    fit_paired_query_binding,
    paired_query_binding_loss,
    paired_query_loss_terms,
)
from r4_softmax_trainer.source_relation_adapter import LoRALinear


def _outcome(labels: list[int]) -> str:
    positives = sum(labels)
    return "abstain" if positives == 0 else "answer" if positives == 1 else "conflict"


def _record(
    record_id: str,
    *,
    width: int = 2,
    pair_slot: int = 0,
    world_lane: int = 0,
    labels: tuple[list[int], list[int]] = ([1, 0], [0, 1]),
    groups: int = 2,
) -> dict[str, Any]:
    if groups not in (1, 2):
        raise ValueError("test fixture supports one or two candidate groups")
    group_cids = [f"group-{record_id}-{index}" for index in range(groups)]
    candidate_groups = [
        {
            "relation_group_cid": group_cid,
            "text": f"Candidate {index}.",
            "occurrence_indices": [index],
            "earliest_occurrence_index": index,
        }
        for index, group_cid in enumerate(group_cids)
    ]
    candidate_indices = [2, 4][:groups]
    token_lanes = ([10, 11, 12, 13, 20, 21], [10, 11, 12, 13, 30, 21])
    queries = [
        {
            "question": f"Where is the subject {query_index}?",
            "token_ids": list(token_lanes[query_index]),
            "query_terminal_index": 6,
            "candidate_terminal_indices": candidate_indices,
            "source_prefix_token_count": 5,
            "target_outcome": _outcome(labels[query_index][:groups]),
        }
        for query_index in range(2)
    ]
    flip_cids = [
        group_cids[index]
        for index in range(groups)
        if labels[0][index] != labels[1][index]
    ]
    return {
        "record_cid": record_id,
        "source_width": width,
        "pair_slot": pair_slot,
        "lexical_world": f"world-w{width}-lane{world_lane}",
        "source_prefix_identity_exact": True,
        "source_prefix_token_ids_cid": f"prefix-{record_id}",
        "candidate_groups": candidate_groups,
        "label_matrix": [labels[0][:groups], labels[1][:groups]],
        "flip_group_cids": flip_cids,
        "queries": queries,
    }


def _complete_fit_records() -> list[dict[str, Any]]:
    return [
        _record(
            f"record-w{width}-world{world_lane}-pair{pair_slot}",
            width=width,
            pair_slot=pair_slot,
            world_lane=world_lane,
        )
        for width in range(2, 9)
        for world_lane in range(2)
        for pair_slot in range(4)
    ]


class _ExactScoreAdapter:
    def eval(self) -> "_ExactScoreAdapter":
        return self

    def __call__(
        self,
        batch: PairedQueryTokenBatch,
        *,
        attention_off: bool,
        mean_query_ablation: bool,
    ) -> PairedQueryBindingOutput:
        if mean_query_ablation:
            scores = torch.zeros_like(batch.labels)
        else:
            scores = torch.where(
                batch.labels == 1.0,
                torch.full_like(batch.labels, 2.0),
                torch.full_like(batch.labels, -2.0),
            )
        hidden_width = 288
        query_states = torch.zeros(
            batch.input_ids.shape[0], 2, hidden_width, device=batch.input_ids.device
        )
        candidates = torch.zeros(
            batch.input_ids.shape[0],
            batch.candidate_indices.shape[1],
            hidden_width,
            device=batch.input_ids.device,
        )
        return PairedQueryBindingOutput(
            scores=scores,
            query_states=query_states,
            candidate_states=candidates,
            candidate_states_by_lane=candidates[:, None].expand(-1, 2, -1, -1),
            paired_candidate_states_exact=True,
            attention_off=attention_off,
            mean_query_ablation=mean_query_ablation,
        )


class PairedQueryBindingTests(unittest.TestCase):
    def test_frozen_contract_shapes_parameters_and_deterministic_initialization(self) -> None:
        self.assertEqual(POLICY, "R4PairedQueryCandidateMatrixV1")
        self.assertEqual(BINDING_RANK, 32)
        self.assertEqual(BINDING_BLOCKS, 8)
        self.assertEqual(BINDING_HEAD_PARAMETER_COUNT, 18_433)
        self.assertEqual(TRAINABLE_PARAMETER_COUNT, 129_025)
        self.assertEqual(FIT_SEED, 9_545)
        self.assertEqual(OPTIMIZER_STEPS, 120)
        self.assertEqual(RECORDS_PER_STEP, 7)
        self.assertEqual(STEPS_PER_EPOCH, 8)
        self.assertEqual(MARGIN, 1.0)
        self.assertEqual(FLIP_MARGIN, 2.0)

        config = PairedQueryBindingAdapterConfig()
        config.validate()
        self.assertEqual(config.trainable_parameter_count, 129_025)
        self.assertFalse(config.as_contract()["generic_lm_or_classification_head"])
        with self.assertRaisesRegex(ValueError, "frozen"):
            replace(config, binding_rank=16).validate()
        fit = PairedQueryBindingFitConfig()
        fit.validate()
        self.assertEqual(fit.as_contract()["epochs"], 15)
        with self.assertRaisesRegex(ValueError, "frozen"):
            replace(fit, learning_rate=0.002).validate()

        first = R4PairedQueryCandidateMatrix(R4SoftmaxForCausalLM())
        second = R4PairedQueryCandidateMatrix(R4SoftmaxForCausalLM())
        self.assertEqual(len(first.trainable_parameter_names()), 51)
        self.assertEqual(
            sum(parameter.numel() for parameter in first.adapter_parameters()),
            129_025,
        )
        for name in first.binding_head_state_dict():
            self.assertTrue(
                torch.equal(
                    first.binding_head_state_dict()[name],
                    second.binding_head_state_dict()[name],
                )
            )
        first_query = first.model.model.layers[0].self_attn.q_proj
        second_query = second.model.model.layers[0].self_attn.q_proj
        self.assertIsInstance(first_query, LoRALinear)
        self.assertIsInstance(second_query, LoRALinear)
        self.assertTrue(torch.equal(first_query.lora_a, second_query.lora_a))
        self.assertEqual(set(first.binding_head_state_dict()), {"query_weight", "candidate_weight", "bias"})
        self.assertTrue(
            all(
                tensor.device.type == "cpu" and tensor.is_contiguous()
                for tensor in first.binding_head_state_dict().values()
            )
        )

    def test_matrix_forward_enforces_causal_prefix_and_supports_controls(self) -> None:
        dataset = EncodedPairedQueryBindingDataset([_record("pair")])
        batch = dataset.batch([0], device=torch.device("cpu"))
        adapter = R4PairedQueryCandidateMatrix(R4SoftmaxForCausalLM()).eval()
        output = adapter(batch)
        self.assertEqual(tuple(output.scores.shape), (1, 2, 2))
        self.assertEqual(tuple(output.query_states.shape), (1, 2, 288))
        self.assertEqual(tuple(output.candidate_states.shape), (1, 2, 288))
        self.assertTrue(output.paired_candidate_states_exact)
        self.assertTrue(
            torch.equal(
                output.candidate_states_by_lane[:, 0],
                output.candidate_states_by_lane[:, 1],
            )
        )
        unchecked = adapter(batch, verify_candidate_state_identity=False)
        self.assertIsNone(unchecked.paired_candidate_states_exact)

        ablated = adapter(batch, mean_query_ablation=True)
        self.assertTrue(torch.equal(ablated.scores[:, 0], ablated.scores[:, 1]))
        self.assertTrue(torch.equal(ablated.query_states[:, 0], ablated.query_states[:, 1]))
        attention_off = adapter(batch, attention_off=True)
        self.assertTrue(attention_off.attention_off)
        self.assertEqual(tuple(attention_off.scores.shape), (1, 2, 2))

        tampered_ids = batch.input_ids.clone()
        tampered_ids[0, 1, 2] += 1
        with self.assertRaisesRegex(ValueError, "prefixes"):
            replace(batch, input_ids=tampered_ids).validate()
        late_candidate = batch.candidate_indices.clone()
        late_candidate[0, 0] = batch.source_prefix_lengths[0]
        with self.assertRaisesRegex(ValueError, "before the question"):
            replace(batch, candidate_indices=late_candidate).validate()

    def test_loss_arithmetic_includes_abstain_and_direct_flip_margin(self) -> None:
        labels = torch.tensor(
            [
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            ]
        )
        group_mask = torch.tensor([[True, True, True], [True, True, False]])
        flip_mask = torch.tensor([[True, True, False], [True, True, False]])
        exact_scores = torch.tensor(
            [
                [[1.0, -1.0, -1.0], [-1.0, 1.0, -1.0]],
                [[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]],
            ]
        )
        exact = paired_query_loss_terms(
            exact_scores, labels, group_mask, flip_mask
        )
        self.assertTrue(torch.equal(exact.row_losses, torch.zeros(4)))
        self.assertTrue(torch.equal(exact.flip_losses, torch.zeros(4)))
        self.assertEqual(float(exact.total), 0.0)

        zeros = torch.zeros_like(exact_scores)
        violated = paired_query_loss_terms(zeros, labels, group_mask, flip_mask)
        self.assertTrue(
            torch.equal(violated.row_losses, torch.tensor([2.0, 2.0, 1.0, 1.0]))
        )
        self.assertTrue(torch.equal(violated.flip_losses, torch.full((4,), 2.0)))
        self.assertEqual(float(violated.mean_row_loss), 1.5)
        self.assertEqual(float(violated.mean_flip_loss), 2.0)
        self.assertEqual(float(paired_query_binding_loss(zeros, labels, group_mask, flip_mask)), 3.5)

    def test_variable_groups_alignment_and_exact_eight_step_epoch(self) -> None:
        records = list(reversed(_complete_fit_records()))
        dataset = EncodedPairedQueryBindingDataset(records)
        dataset.validate_fit_schedule()
        selected = [
            dataset.record_indices_for_step(step)
            for step in range(1, STEPS_PER_EPOCH + 1)
        ]
        self.assertTrue(all(len(indices) == 7 for indices in selected))
        self.assertEqual(
            {
                index
                for indices in selected
                for index in indices
            },
            set(range(56)),
        )
        self.assertEqual(
            [dataset.records[index].source_width for index in selected[0]],
            list(range(2, 9)),
        )
        self.assertNotEqual(
            dataset.record_indices_for_step(1),
            dataset.record_indices_for_step(1 + STEPS_PER_EPOCH),
        )

        variable = EncodedPairedQueryBindingDataset(
            [
                _record("pair-two", groups=2),
                _record(
                    "pair-one",
                    groups=1,
                    labels=([1, 0], [0, 0]),
                ),
            ]
        )
        batch = variable.batch([0, 1], device=torch.device("cpu"))
        self.assertEqual(tuple(batch.input_ids.shape[:2]), (2, 2))
        self.assertEqual(tuple(batch.group_mask.shape), (2, 2))
        self.assertEqual(sorted(batch.group_mask.sum(dim=1).tolist()), [1, 2])
        self.assertTrue(torch.equal(batch.input_ids[:, 0, :5], batch.input_ids[:, 1, :5]))
        self.assertTrue(bool((batch.candidate_indices[batch.group_mask] < 5).all()))

    def test_evaluator_preserves_pair_identity_and_exposes_binding_controls(self) -> None:
        dataset = EncodedPairedQueryBindingDataset(
            [
                _record("pair-b", pair_slot=3),
                _record("pair-a", pair_slot=0),
            ]
        )
        exact = evaluate_paired_query_binding(
            _ExactScoreAdapter(),  # type: ignore[arg-type]
            dataset,
            device=torch.device("cpu"),
            pair_batch_size=2,
        )
        self.assertEqual(exact["pair_exact"], {"correct": 2, "total": 2})
        self.assertEqual(exact["row_exact"], {"correct": 4, "total": 4})
        self.assertEqual(exact["cell_exact"], {"correct": 8, "total": 8})
        self.assertEqual(exact["flip_exact"], {"correct": 4, "total": 4})
        self.assertEqual(exact["candidate_state_bit_identity"], {"correct": 2, "total": 2})
        self.assertEqual(exact["mean_loss"], 0.0)
        self.assertEqual(
            [row["record_id"] for row in exact["pair_evaluations"]],
            ["pair-a", "pair-b"],
        )

        swapped = evaluate_paired_query_binding(
            _ExactScoreAdapter(),  # type: ignore[arg-type]
            dataset,
            device=torch.device("cpu"),
            pair_batch_size=2,
            row_swap=True,
        )
        self.assertEqual(swapped["pair_exact"], {"correct": 2, "total": 2})
        ablated = evaluate_paired_query_binding(
            _ExactScoreAdapter(),  # type: ignore[arg-type]
            dataset,
            device=torch.device("cpu"),
            pair_batch_size=2,
            mean_query_ablation=True,
        )
        self.assertEqual(ablated["paired_rows_identical"], {"correct": 2, "total": 2})
        self.assertEqual(ablated["pair_exact"], {"correct": 0, "total": 2})
        self.assertGreater(ablated["mean_loss"], exact["mean_loss"])

    def test_changed_and_finite_tensor_census_covers_only_the_frozen_targets(self) -> None:
        adapter = R4PairedQueryCandidateMatrix(R4SoftmaxForCausalLM())
        # Wrapped projections have ``base.weight`` names, so bind the actual
        # immutable ordinary tensors through the merge surface before mutation.
        base_state = adapter.merged_state_dict()
        self.assertEqual(adapter.delta_audit()["changed_tensor_count"], 0)
        self.assertEqual(adapter.binding_head_audit()["changed_tensor_count"], 0)
        with torch.no_grad():
            for layer in adapter.model.model.layers:
                for projection_name in ("q_proj", "k_proj", "v_proj", "o_proj"):
                    projection = getattr(layer.self_attn, projection_name)
                    self.assertIsInstance(projection, LoRALinear)
                    projection.lora_b[0, 0] = 0.125
            adapter.binding_head.query_weight[0, 0] += 0.25
            adapter.binding_head.candidate_weight[0, 0] -= 0.25
            adapter.binding_head.bias += 0.125
        delta = adapter.delta_audit()
        self.assertEqual(delta["target_tensor_count"], 24)
        self.assertEqual(delta["changed_tensor_count"], 24)
        self.assertTrue(delta["all_finite"])
        head = adapter.binding_head_audit()
        self.assertEqual(head["tensor_count"], 3)
        self.assertEqual(head["changed_tensor_count"], 3)
        self.assertTrue(head["all_finite"])
        representation = adapter.representation_audit(base_state)
        self.assertEqual(representation["changed_target_tensor_count"], 24)
        self.assertEqual(representation["changed_nontarget_tensors"], [])
        self.assertTrue(representation["passed"])
        self.assertEqual(set(adapter.merged_state_dict()), expected_hf_tensor_names())
        self.assertIsInstance(adapter.merged_model(), R4SoftmaxForCausalLM)

    def test_fit_samples_host_loss_only_at_synchronized_boundaries(self) -> None:
        class FakeParameter:
            def numel(self) -> int:
                return TRAINABLE_PARAMETER_COUNT

        class FakeAdapter:
            config = SimpleNamespace(initialization_seed=FIT_SEED)

            def __init__(self) -> None:
                self.verify_identity: list[bool] = []

            def to(self, _device: torch.device) -> "FakeAdapter":
                return self

            def train(self) -> "FakeAdapter":
                return self

            def adapter_parameters(self) -> list[FakeParameter]:
                return [FakeParameter()]

            def __call__(
                self,
                _batch: object,
                *,
                verify_candidate_state_identity: bool,
            ) -> SimpleNamespace:
                self.verify_identity.append(verify_candidate_state_identity)
                return SimpleNamespace(scores=object())

            def delta_audit(self) -> dict[str, object]:
                return {}

            def binding_head_audit(self) -> dict[str, object]:
                return {}

        class FakeDataset:
            def validate_fit_schedule(self) -> None:
                return None

            def record_indices_for_step(self, _step: int) -> tuple[int, ...]:
                return tuple(range(RECORDS_PER_STEP))

            def batch(self, _indices: object, *, device: torch.device) -> object:
                self.device = device
                return SimpleNamespace(
                    labels=object(),
                    group_mask=object(),
                    flip_mask=object(),
                )

        class FakeOptimizer:
            def zero_grad(self, *, set_to_none: bool) -> None:
                self.set_to_none = set_to_none

            def step(self) -> None:
                return None

        class FakeLoss:
            def __init__(self, step: int) -> None:
                self.step = step

            def backward(self) -> None:
                return None

            def detach(self) -> "FakeLoss":
                return self

            def cpu(self) -> float:
                host_loss_reads.append(self.step)
                return 1.0

        adapter = FakeAdapter()
        dataset = FakeDataset()
        host_loss_reads: list[int] = []
        loss_step = 0

        def fake_loss(*_args: object) -> FakeLoss:
            nonlocal loss_step
            loss_step += 1
            return FakeLoss(loss_step)

        with (
            patch(
                "r4_softmax_trainer.paired_query_binding.require_mps",
                return_value=torch.device("mps"),
            ),
            patch(
                "r4_softmax_trainer.paired_query_binding.torch.optim.AdamW",
                return_value=FakeOptimizer(),
            ),
            patch(
                "r4_softmax_trainer.paired_query_binding.torch.nn.utils.clip_grad_norm_"
            ),
            patch(
                "r4_softmax_trainer.paired_query_binding.paired_query_binding_loss",
                side_effect=fake_loss,
            ),
            patch(
                "r4_softmax_trainer.paired_query_binding.torch.mps.synchronize"
            ) as synchronize,
        ):
            result = fit_paired_query_binding(  # type: ignore[arg-type]
                adapter,  # type: ignore[arg-type]
                dataset,  # type: ignore[arg-type]
            )

        boundaries = [1, *range(STEPS_PER_EPOCH, OPTIMIZER_STEPS + 1, STEPS_PER_EPOCH)]
        self.assertEqual(host_loss_reads, boundaries)
        self.assertEqual(synchronize.call_count, len(boundaries))
        self.assertEqual(adapter.verify_identity, [False] * OPTIMIZER_STEPS)
        self.assertEqual(result["initial_loss"], 1.0)
        self.assertEqual(result["final_loss"], 1.0)


if __name__ == "__main__":
    unittest.main()
