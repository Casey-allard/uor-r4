"""Focused checks for the C1-SB3 all-layer attention adapter."""

from __future__ import annotations

import math
import unittest
from types import SimpleNamespace

import torch
from torch import nn

from r4_softmax_trainer.model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.source_relation_adapter import (
    ARTIFACT_SCHEMA,
    LORA_ALPHA,
    LORA_DROPOUT,
    LORA_RANK,
    INITIALIZATION_SEED,
    NO_TOKEN_ID,
    POLICY,
    RELATION_INPUT_TEMPLATE,
    TRAINABLE_PARAMETER_COUNT,
    YES_TOKEN_ID,
    AttendedRelationAdapterConfig,
    EncodedRelationDataset,
    LoRALinear,
    R4AttendedRelationAdapter,
    RelationExample,
    relation_binary_loss,
    relation_examples_from_records,
    tied_relation_scores,
)


class _StubTokenizer:
    def encode(self, text: str, *, add_special_tokens: bool) -> SimpleNamespace:
        if add_special_tokens:
            raise AssertionError("adapter tokenizer must disable implicit specials")
        if text == " yes":
            return SimpleNamespace(ids=[YES_TOKEN_ID])
        if text == " no":
            return SimpleNamespace(ids=[NO_TOKEN_ID])
        if text.endswith("\nSupported:"):
            return SimpleNamespace(ids=[11, 12, 13])
        return SimpleNamespace(ids=[99])

    def decode(self, token_ids: list[int], *, skip_special_tokens: bool) -> str:
        if skip_special_tokens:
            raise AssertionError("adapter tokenizer must inspect exact tokens")
        return {
            (YES_TOKEN_ID,): " yes",
            (NO_TOKEN_ID,): " no",
            (13,): ":",
        }.get(tuple(token_ids), "fixture")


class SourceRelationAdapterTests(unittest.TestCase):
    def test_contract_is_all_six_layers_without_a_trainable_head(self) -> None:
        contract = AttendedRelationAdapterConfig()
        contract.validate()
        self.assertEqual(POLICY, "R4AttendedRelationAdapterV1")
        self.assertEqual(ARTIFACT_SCHEMA, "uor-r4.attended-relation-adapter/1")
        self.assertTrue(RELATION_INPUT_TEMPLATE.endswith("Supported:"))
        self.assertEqual((LORA_RANK, LORA_ALPHA, LORA_DROPOUT), (8, 8, 0.0))
        self.assertEqual(INITIALIZATION_SEED, 9_543)
        self.assertEqual(TRAINABLE_PARAMETER_COUNT, 110_592)
        self.assertEqual(contract.trainable_parameter_count, 110_592)

        torch.manual_seed(9_543)
        model = R4SoftmaxForCausalLM()
        adapter = R4AttendedRelationAdapter(model)
        names = adapter.trainable_parameter_names()
        self.assertEqual(len(names), 48)
        self.assertTrue(all(name.endswith(("lora_a", "lora_b")) for name in names))
        self.assertEqual(
            sum(parameter.numel() for parameter in adapter.adapter_parameters()),
            110_592,
        )
        for layer in model.model.layers:
            for name in ("q_proj", "k_proj", "v_proj", "o_proj"):
                self.assertIsInstance(getattr(layer.self_attn, name), LoRALinear)
        self.assertFalse(model.model.embed_tokens.weight.requires_grad)

    def test_lora_starts_as_exact_base_and_merge_matches_forward(self) -> None:
        torch.manual_seed(7)
        base = nn.Linear(5, 4, bias=False)
        original = base.weight.detach().clone()
        rng_before = torch.random.get_rng_state()
        lora = LoRALinear(base)
        self.assertTrue(torch.equal(torch.random.get_rng_state(), rng_before))
        values = torch.randn(3, 5)
        expected_base = torch.nn.functional.linear(values, original)
        self.assertTrue(torch.equal(lora(values), expected_base))

        with torch.no_grad():
            lora.lora_b[0, 0] = 0.25
            lora.lora_b[2, 3] = -0.5
        merged = lora.merged_weight()
        expected = torch.nn.functional.linear(values, merged)
        self.assertTrue(torch.allclose(lora(values), expected, atol=1e-6, rtol=1e-6))
        self.assertTrue(torch.equal(base.weight, original))

    def test_clean_merge_has_only_standard_hf_tensors(self) -> None:
        torch.manual_seed(9_543)
        model = R4SoftmaxForCausalLM()
        original_query = model.model.layers[0].self_attn.q_proj.weight.detach().clone()
        adapter = R4AttendedRelationAdapter(model)
        first = model.model.layers[0].self_attn.q_proj
        self.assertIsInstance(first, LoRALinear)
        with torch.no_grad():
            first.lora_b[0, 0] = 0.125

        merged_state = adapter.merged_state_dict()
        self.assertEqual(set(merged_state), expected_hf_tensor_names())
        self.assertNotEqual(
            torch.count_nonzero(
                merged_state["model.layers.0.self_attn.q_proj.weight"]
                - original_query
            ).item(),
            0,
        )
        audit = adapter.delta_audit()
        self.assertEqual(audit["target_tensor_count"], 24)
        self.assertEqual(audit["trainable_parameter_count"], 110_592)
        self.assertEqual(
            {tensor["initialization_seed"] for tensor in audit["tensors"]},
            set(range(9_543, 9_567)),
        )
        clean = adapter.merged_model()
        self.assertEqual(set(clean.state_dict()), expected_hf_tensor_names())
        self.assertFalse(
            any("lora" in name or ".base." in name for name in clean.state_dict())
        )
        input_ids = torch.tensor([[0, 11, 12, 13]], dtype=torch.long)
        adapter.eval()
        with torch.no_grad():
            wrapped_score = adapter(input_ids)
            clean_hidden = clean.model(input_ids)[:, -1]
            clean_score = tied_relation_scores(
                clean_hidden, clean.model.embed_tokens.weight
            )
        self.assertTrue(
            torch.allclose(wrapped_score, clean_score, atol=1e-5, rtol=1e-5)
        )

    def test_tied_verbalizer_is_yes_minus_no_and_has_no_parameters(self) -> None:
        embedding = torch.zeros(4_096, 288)
        embedding[YES_TOKEN_ID] = 2.0
        embedding[NO_TOKEN_ID] = -1.0
        hidden = torch.stack((torch.ones(288), -torch.ones(288)))
        scores = tied_relation_scores(hidden, embedding)
        self.assertTrue(torch.equal(scores, torch.tensor([864.0, -864.0])))
        loss = relation_binary_loss(scores, torch.tensor([1.0, 0.0]))
        self.assertTrue(math.isfinite(float(loss)))
        self.assertLess(float(loss), 1e-6)

    def test_record_adapter_binds_exact_prompt_cid_and_terminal_colon(self) -> None:
        relation_input = (
            "Evidence:\nThe amber dial is inside the oak case.\n"
            "Question:\nWhere is the amber dial?\nSupported:"
        )
        record = {
            "record_cid": "blake3:" + "1" * 64,
            "question": "Where is the amber dial?",
            "sentence_spans": [
                {
                    "candidate_index": 0,
                    "text": "The amber dial is inside the oak case.",
                    "relation_input": relation_input,
                    "relation_input_cid": cid_bytes(relation_input.encode("utf-8")),
                    "relation_label": 1,
                }
            ],
        }
        examples = relation_examples_from_records([record])
        self.assertEqual(len(examples), 1)
        self.assertEqual(examples[0].relation_label, 1)
        self.assertFalse(examples[0].relation_input.endswith("\n"))
        dataset = EncodedRelationDataset(examples, _StubTokenizer())  # type: ignore[arg-type]
        batch = dataset.batch([0], device=torch.device("cpu"))
        self.assertEqual(batch.terminal_indices.tolist(), [3])
        self.assertEqual(batch.input_ids[0, 3].item(), 13)

        malformed_span = dict(record["sentence_spans"][0])
        malformed_span["relation_input"] = relation_input + "\n"
        malformed = {**record, "sentence_spans": [malformed_span]}
        with self.assertRaisesRegex(ValueError, "Supported"):
            relation_examples_from_records([malformed])

    def test_seeded_schedule_covers_every_sorted_row_before_repeat(self) -> None:
        relation_input = (
            "Evidence:\nThe amber dial is inside the oak case.\n"
            "Question:\nWhere is the amber dial?\nSupported:"
        )
        examples = [
            RelationExample(
                record_id=f"record-{index:03}",
                candidate_index=0,
                relation_input=relation_input,
                relation_input_cid=cid_bytes(relation_input.encode("utf-8")),
                relation_label=index % 2,
            )
            for index in reversed(range(70))
        ]
        dataset = EncodedRelationDataset(examples, _StubTokenizer())  # type: ignore[arg-type]
        first = dataset.deterministic_indices(seed=9_543, step=1, batch_size=64)
        second = dataset.deterministic_indices(seed=9_543, step=2, batch_size=64)
        self.assertEqual(len(set((*first, *second[:6]))), 70)
        self.assertEqual(
            first,
            dataset.deterministic_indices(seed=9_543, step=1, batch_size=64),
        )


if __name__ == "__main__":
    unittest.main()
