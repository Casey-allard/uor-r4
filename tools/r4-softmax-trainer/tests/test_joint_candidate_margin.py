"""Focused checks for complete-record joint candidate margin learning."""

from __future__ import annotations

import unittest
from dataclasses import replace
from types import SimpleNamespace

import torch

from r4_softmax_trainer.model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.source_relation_adapter import (
    NO_TOKEN_ID,
    YES_TOKEN_ID,
    AttendedRelationAdapterConfig,
    LoRALinear,
)
from r4_softmax_trainer.joint_candidate_margin import (
    FIT_SEED,
    MARGIN,
    OPTIMIZER_STEPS,
    OUTCOMES,
    POLICY,
    RECORDS_PER_STEP,
    SOURCE_WIDTHS,
    STEPS_PER_EPOCH,
    EncodedJointCandidateMarginDataset,
    JointCandidateMarginAdapterConfig,
    JointCandidateMarginFitConfig,
    R4JointCandidateMarginAdapter,
    evaluate_joint_candidate_margin_adapter,
    joint_candidate_structured_margin_loss,
    structured_margin_per_record,
)
from r4_softmax_trainer.joint_candidate_margin_data import (
    render_joint_candidate_input,
)


class _StubTokenizer:
    def encode(self, text: str, *, add_special_tokens: bool) -> SimpleNamespace:
        if add_special_tokens:
            raise AssertionError("joint candidate tokenizer must disable implicit specials")
        if text == " yes":
            return SimpleNamespace(ids=[YES_TOKEN_ID])
        if text == " no":
            return SimpleNamespace(ids=[NO_TOKEN_ID])
        if "force-overflow" in text:
            return SimpleNamespace(ids=[11] * 255 + [13])
        if text.endswith("\nSupported:"):
            return SimpleNamespace(ids=[11, 12, 13])
        return SimpleNamespace(ids=[99])

    def decode(self, token_ids: list[int], *, skip_special_tokens: bool) -> str:
        if skip_special_tokens:
            raise AssertionError("joint candidate tokenizer must inspect exact tokens")
        return {
            (YES_TOKEN_ID,): " yes",
            (NO_TOKEN_ID,): " no",
            (13,): ":",
        }.get(tuple(token_ids), "fixture")


def _record(
    record_id: str,
    *,
    width: int,
    outcome: str,
    texts: list[str] | None = None,
    labels: list[int] | None = None,
    lexical_world: str | None = None,
    motif: str | None = None,
) -> dict[str, object]:
    if texts is None:
        texts = [f"The {record_id} item {index} was observed." for index in range(width)]
    if labels is None:
        labels = (
            [1, *([0] * (width - 1))]
            if outcome == "answer"
            else [0] * width
            if outcome == "abstain"
            else [1, 1, *([0] * (width - 2))]
        )
    source = " ".join(texts)
    source_bytes = source.encode("utf-8")
    spans = []
    cursor = 0
    for index, (text, label) in enumerate(zip(texts, labels)):
        encoded = text.encode("utf-8")
        start = source_bytes.index(encoded, cursor)
        end = start + len(encoded)
        group_cid = cid_bytes(encoded)
        spans.append(
            {
                "candidate_index": index,
                "byte_start": start,
                "byte_end": end,
                "text": text,
                "relation_group_cid": group_cid,
                "relation_label": label,
            }
        )
        cursor = end
    positive_groups = sorted(
        {span["relation_group_cid"] for span in spans if span["relation_label"] == 1}
    )
    question = f"Where is the {record_id} item?"
    for span in spans:
        relation_input = render_joint_candidate_input(
            source, question, str(span["text"])
        )
        span["relation_input"] = relation_input
        span["relation_input_cid"] = cid_bytes(relation_input.encode("utf-8"))
    return {
        "record_cid": record_id,
        "lexical_world": lexical_world or f"world-{record_id}",
        "motif": motif or f"motif-{outcome}",
        "target_outcome": outcome,
        "source_width": width,
        "source": source,
        "question": question,
        "sentence_spans": spans,
        "positive_relation_group_cids": positive_groups,
    }


def _complete_fit_records() -> list[dict[str, object]]:
    return [
        _record(
            f"record-w{width}-{outcome}-{lane}",
            width=width,
            outcome=outcome,
            lexical_world=f"world-w{width}-{lane // 3}",
            motif=f"motif-{outcome}-{lane}",
        )
        for width in SOURCE_WIDTHS
        for outcome in OUTCOMES
        for lane in range(6)
    ]


class _FixedScoreAdapter:
    def __init__(self, scores: list[float]) -> None:
        self._scores = torch.tensor(scores, dtype=torch.float32)
        self._cursor = 0

    def eval(self) -> "_FixedScoreAdapter":
        return self

    def __call__(self, input_ids: torch.Tensor, terminal_indices: torch.Tensor) -> torch.Tensor:
        self.last_terminal_indices = terminal_indices.detach().cpu().tolist()
        end = self._cursor + input_ids.shape[0]
        result = self._scores[self._cursor:end].to(input_ids.device)
        self._cursor = end
        return result


class JointCandidateMarginTests(unittest.TestCase):
    def test_contract_and_exact_renderer_are_frozen(self) -> None:
        self.assertEqual(POLICY, "R4JointCandidateMarginAdapterV1")
        self.assertEqual(MARGIN, 1.0)
        self.assertEqual(FIT_SEED, 9_544)
        self.assertEqual(OPTIMIZER_STEPS, 270)
        self.assertEqual(RECORDS_PER_STEP, 7)
        rendered = render_joint_candidate_input(
            "The opal dial is inside the oak case.",
            "Where is the opal dial?",
            "The opal dial is inside the oak case.",
        )
        self.assertEqual(
            rendered,
            "E:The opal dial is inside the oak case.\n"
            "Q:Where is the opal dial?\n"
            "C:The opal dial is inside the oak case.\nSupported:",
        )
        self.assertFalse(rendered.endswith("\n"))

        config = JointCandidateMarginFitConfig()
        config.validate()
        with self.assertRaisesRegex(ValueError, "frozen"):
            replace(config, learning_rate=0.002).validate()

    def test_adapter_seed_is_versioned_from_sb3_without_a_trainable_head(self) -> None:
        self.assertEqual(AttendedRelationAdapterConfig().initialization_seed, 9_543)
        config = JointCandidateMarginAdapterConfig()
        config.validate()
        self.assertEqual(config.initialization_seed, 9_544)
        self.assertEqual(config.trainable_parameter_count, 110_592)
        with self.assertRaisesRegex(ValueError, "frozen"):
            replace(config, initialization_seed=9_543).validate()

        model = R4SoftmaxForCausalLM()
        adapter = R4JointCandidateMarginAdapter(model)
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
        self.assertEqual(
            {tensor["initialization_seed"] for tensor in adapter.delta_audit()["tensors"]},
            set(range(9_544, 9_568)),
        )
        first_query = model.model.layers[0].self_attn.q_proj
        self.assertIsInstance(first_query, LoRALinear)
        with torch.no_grad():
            first_query.lora_b[0, 0] = 0.125
        merged = adapter.merged_state_dict()
        self.assertEqual(set(merged), expected_hf_tensor_names())
        self.assertNotEqual(
            torch.count_nonzero(
                merged["model.layers.0.self_attn.q_proj.weight"]
                - first_query.base.weight
            ).item(),
            0,
        )

    def test_duplicate_occurrences_collapse_and_disagreement_rejects(self) -> None:
        text = "The amber dial is inside the oak case."
        record = _record(
            "duplicate",
            width=2,
            outcome="answer",
            texts=[text, text],
            labels=[1, 1],
        )
        dataset = EncodedJointCandidateMarginDataset(
            [record], _StubTokenizer()  # type: ignore[arg-type]
        )
        self.assertEqual(len(dataset.records[0].groups), 1)
        group = dataset.records[0].groups[0]
        self.assertEqual(group.occurrence_indices, (0, 1))
        self.assertEqual(group.relation_label, 1)
        self.assertIn(f"C:{text}", group.relation_input)

        disagreement = _record(
            "disagreement",
            width=2,
            outcome="answer",
            texts=[text, text],
            labels=[1, 0],
        )
        # The helper's committed positive-group list cannot express the invalid
        # disagreement; the model layer must reject the row-level mismatch first.
        with self.assertRaisesRegex(ValueError, "labels disagree"):
            EncodedJointCandidateMarginDataset(
                [disagreement], _StubTokenizer()  # type: ignore[arg-type]
            )

    def test_terminal_colon_and_context_limit_include_bos(self) -> None:
        dataset = EncodedJointCandidateMarginDataset(
            [_record("normal", width=2, outcome="answer")],
            _StubTokenizer(),  # type: ignore[arg-type]
        )
        batch = dataset.batch([0], device=torch.device("cpu"))
        self.assertEqual(batch.input_ids[:, 0].tolist(), [0, 0])
        self.assertEqual(batch.terminal_indices.tolist(), [3, 3])
        self.assertEqual(batch.input_ids[:, 3].tolist(), [13, 13])
        self.assertEqual(batch.record_slices, ((0, 2),))

        tampered = _record("tampered", width=2, outcome="answer")
        tampered["sentence_spans"][0]["relation_input"] += "\n"  # type: ignore[index]
        with self.assertRaisesRegex(ValueError, "renderer differs"):
            EncodedJointCandidateMarginDataset(
                [tampered], _StubTokenizer()  # type: ignore[arg-type]
            )

        overflow = _record(
            "overflow",
            width=2,
            outcome="answer",
            texts=["The force-overflow candidate is here.", "A second candidate is here."],
        )
        with self.assertRaisesRegex(ValueError, "256-token context"):
            EncodedJointCandidateMarginDataset(
                [overflow], _StubTokenizer()  # type: ignore[arg-type]
            )

    def test_structured_margin_matches_answer_abstain_conflict_semantics(self) -> None:
        labels = torch.tensor(
            [1.0, 0.0, 0.0, 0.0, 1.0, 1.0], dtype=torch.float32
        )
        slices = ((0, 2), (2, 4), (4, 6))
        exact_scores = torch.tensor([1.0, -1.0, -2.0, -1.0, 1.0, 2.0])
        per_record = structured_margin_per_record(exact_scores, labels, slices)
        self.assertTrue(torch.equal(per_record, torch.zeros(3)))
        self.assertEqual(
            float(joint_candidate_structured_margin_loss(exact_scores, labels, slices)),
            0.0,
        )

        violating_scores = torch.zeros(6)
        violating = structured_margin_per_record(violating_scores, labels, slices)
        self.assertTrue(torch.equal(violating, torch.tensor([2.0, 1.0, 1.0])))
        self.assertAlmostEqual(
            float(
                joint_candidate_structured_margin_loss(
                    violating_scores, labels, slices
                )
            ),
            4.0 / 3.0,
        )
        with self.assertRaisesRegex(ValueError, "contiguous"):
            structured_margin_per_record(exact_scores, labels, ((0, 2), (3, 6)))

    def test_schedule_is_one_per_width_and_exactly_covers_each_epoch(self) -> None:
        dataset = EncodedJointCandidateMarginDataset(
            list(reversed(_complete_fit_records())),
            _StubTokenizer(),  # type: ignore[arg-type]
        )
        dataset.validate_fit_schedule()
        first_epoch = [
            dataset.record_indices_for_step(step)
            for step in range(1, STEPS_PER_EPOCH + 1)
        ]
        for selected in first_epoch:
            self.assertEqual(len(selected), 7)
            self.assertEqual(
                [dataset.records[index].source_width for index in selected],
                list(SOURCE_WIDTHS),
            )
            outcome_counts = {
                outcome: sum(
                    dataset.records[index].target_outcome == outcome
                    for index in selected
                )
                for outcome in OUTCOMES
            }
            self.assertEqual(sorted(outcome_counts.values()), [2, 2, 3])
        for width in SOURCE_WIDTHS:
            observed = [
                index
                for selected in first_epoch
                for index in selected
                if dataset.records[index].source_width == width
            ]
            expected = [
                index
                for index, record in enumerate(dataset.records)
                if record.source_width == width
            ]
            self.assertEqual(len(observed), 18)
            self.assertEqual(set(observed), set(expected))
        self.assertNotEqual(
            dataset.record_indices_for_step(1),
            dataset.record_indices_for_step(1 + STEPS_PER_EPOCH),
        )
        self.assertEqual(
            dataset.record_indices_for_step(1),
            dataset.record_indices_for_step(1),
        )

    def test_evaluation_preserves_stable_record_and_group_score_alignment(self) -> None:
        records = [
            _record("record-b", width=2, outcome="abstain"),
            _record("record-a", width=2, outcome="answer"),
        ]
        dataset = EncodedJointCandidateMarginDataset(
            records, _StubTokenizer()  # type: ignore[arg-type]
        )
        # Dataset order is record-a then record-b; each record has two groups.
        adapter = _FixedScoreAdapter([2.0, -2.0, -3.0, -2.0])
        result = evaluate_joint_candidate_margin_adapter(
            adapter,  # type: ignore[arg-type]
            dataset,
            device=torch.device("cpu"),
            record_batch_size=2,
        )
        self.assertEqual(result["records"], 2)
        self.assertEqual(result["groups"], 4)
        self.assertEqual(result["mean_structured_margin"], 0.0)
        self.assertEqual(
            [record["record_id"] for record in result["record_evaluations"]],
            ["record-a", "record-b"],
        )
        self.assertEqual(
            [
                group["score"]
                for record in result["record_evaluations"]
                for group in record["group_scores"]
            ],
            [2.0, -2.0, -3.0, -2.0],
        )
        with self.assertRaisesRegex(RuntimeError, "cardinality differs"):
            evaluate_joint_candidate_margin_adapter(
                _FixedScoreAdapter([1.0]),  # type: ignore[arg-type]
                dataset,
                device=torch.device("cpu"),
                record_batch_size=2,
            )


if __name__ == "__main__":
    unittest.main()
