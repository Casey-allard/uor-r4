"""Focused contract tests for the frozen C1-SB4 data and commitments."""

from __future__ import annotations

from collections import Counter, defaultdict
from copy import deepcopy
import unittest

from r4_softmax_trainer.joint_candidate_margin_data import (
    FRESH_WORLD_ORDINAL_START,
    JOINT_CENSUS_SCHEMA,
    JOINT_DATASET_SCHEMA,
    JOINT_INPUT_POLICY,
    JOINT_PREFLIGHT_SCHEMA,
    JOINT_PRODUCT_SCHEMA,
    JOINT_RECORD_SCHEMA,
    POLICY,
    build_joint_candidate_margin_population,
    render_joint_candidate_input,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.source_relation_adapter_data import (
    OUTCOMES,
    SOURCE_WIDTHS,
    build_source_relation_adapter_population,
)
from r4_softmax_trainer.source_relation_data import split_sentence_spans


SB3_FROZEN_CIDS = {
    "dataset": "blake3:ec211ee1397e1fae7a20597a2efec5c0fadb525a79c8babb859974b31e1adc24",
    "preflight": "blake3:639a7ad3c897c7bb360ec49204bf32f5ef48d3d42d5c396f1fa94a4a2c50ef24",
    "products": "blake3:05d4bc2f87761059d8c932906cd07b73f7a0d4dd42548ee2b4d0660d623669d3",
    "census": "blake3:b59391706078e9418829d28e20b1f73cfb808bec9b0e1da354592e3c7b192bb9",
}
SB4_FROZEN_CIDS = {
    "dataset": "blake3:46e95f83f05bd5a3bfd4ca0c39c4974f617ae9591ff083d3c6abb8f5593c0e51",
    "preflight": "blake3:b61a098a53fca1f30b69a0ef0d6e15c5b6fc5a310d0ac8f0f2df04d1fd208814",
    "products": "blake3:153f6075f165d6cf92aeb63c31f05c9a869cc94c4655f83b9fe036bf7c773e3e",
    "census": "blake3:2531f2c19cc60570c3807aa117b97923d4ce0b0c8111a8c239861cbae4303a92",
}


def _reproduce_cid(value: dict[str, object], field: str) -> str:
    unsigned = dict(value)
    expected = str(unsigned.pop(field))
    actual = cid_bytes(canonical_json_bytes(unsigned))
    if actual != expected:
        raise AssertionError(f"{field} does not reproduce: {expected} != {actual}")
    return actual


class JointCandidateInputTests(unittest.TestCase):
    def test_renderer_is_exact_full_source_candidate_conditioned_prefix(self) -> None:
        source = (
            "The amber alder abacus is above the arched alcove. "
            "The azure aspen astrolabe was audited after breakfast."
        )
        question = "Where is the amber alder abacus?"
        group = "The amber alder abacus is above the arched alcove."
        value = render_joint_candidate_input(source, question, group)
        self.assertEqual(
            value,
            f"E:{source}\nQ:{question}\nC:{group}\nSupported:",
        )
        self.assertTrue(value.endswith("Supported:"))
        self.assertFalse(value.endswith("\n"))
        self.assertIn("exact full source", JOINT_INPUT_POLICY)
        with self.assertRaisesRegex(ValueError, "not an exact source span"):
            render_joint_candidate_input(
                source,
                question,
                "The amber alder abacus is beneath the arched alcove.",
            )


class JointCandidatePopulationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.preflight, cls.products = (
            build_joint_candidate_margin_population()
        )

    def test_independent_schemas_counts_and_balanced_width_cells(self) -> None:
        self.assertEqual(self.dataset["schema"], JOINT_DATASET_SCHEMA)
        self.assertEqual(self.preflight["schema"], JOINT_PREFLIGHT_SCHEMA)
        self.assertEqual(self.products["schema"], JOINT_PRODUCT_SCHEMA)
        self.assertEqual(self.dataset["census"]["schema"], JOINT_CENSUS_SCHEMA)
        self.assertEqual(self.dataset["policy"], POLICY)
        self.assertEqual(
            self.dataset["counts"],
            {
                "preflight_fit": 126,
                "preflight_sealed": 63,
                "product_probe_commitments": 4,
                "preflight_fit_distinct_groups": 604,
                "preflight_sealed_distinct_groups": 302,
            },
        )
        for records, worlds_per_width in (
            (self.preflight["fit"], 2),
            (self.preflight["sealed"], 1),
        ):
            cells = Counter(
                (record["target_outcome"], record["source_width"])
                for record in records
            )
            self.assertEqual(
                cells,
                Counter(
                    {
                        (outcome, width): worlds_per_width * 3
                        for outcome in OUTCOMES
                        for width in SOURCE_WIDTHS
                    }
                ),
            )

    def test_fresh_worlds_and_every_sb3_partition_are_disjoint(self) -> None:
        census = self.dataset["census"]
        self.assertTrue(census["passed"])
        self.assertEqual(census["fresh_world_ordinal_start"], 137)
        self.assertEqual(FRESH_WORLD_ORDINAL_START, 137)
        self.assertEqual(
            census["ordinal_boundary"],
            {
                "sb3_max_world_ordinal": 136,
                "sb4_min_world_ordinal": 137,
                "sb4_starts_exactly_after_sb3": True,
                "sb4_ordinals_contiguous": True,
            },
        )
        self.assertTrue(census["new_sentence_partitions_pairwise_disjoint"])
        self.assertTrue(census["new_composite_world_item_banks_pairwise_disjoint"])
        self.assertEqual(
            census["primitive_component_vocabulary"],
            "DELIBERATELY_SHARED_ACROSS_SB3_AND_SB4; not a disjointness claim",
        )
        self.assertTrue(
            all(
                value
                for comparisons in census[
                    "new_vs_every_sb3_partition_sentences_disjoint"
                ].values()
                for value in comparisons.values()
            )
        )
        self.assertTrue(
            all(
                value
                for comparisons in census[
                    "new_vs_every_sb3_partition_composite_world_item_bank_disjoint"
                ].values()
                for value in comparisons.values()
            )
        )

    def test_exact_spans_labels_and_duplicate_collapse_are_ready(self) -> None:
        records = [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.products["records"],
        ]
        duplicate_records = 0
        for record in records:
            self.assertEqual(record["schema"], JOINT_RECORD_SCHEMA)
            self.assertEqual(record["policy"], POLICY)
            parsed = split_sentence_spans(record["source"])
            self.assertEqual(
                [span["text"] for span in parsed],
                [span["text"] for span in record["sentence_spans"]],
            )
            groups: dict[str, list[dict[str, object]]] = defaultdict(list)
            for span in record["sentence_spans"]:
                groups[str(span["relation_group_cid"])].append(span)
                expected_input = render_joint_candidate_input(
                    record["source"], record["question"], span["text"]
                )
                self.assertEqual(span["relation_input"], expected_input)
                self.assertEqual(
                    span["relation_input_cid"],
                    cid_bytes(expected_input.encode("utf-8")),
                )
            for group_cid, rows in groups.items():
                self.assertEqual({row["relation_group_cid"] for row in rows}, {group_cid})
                self.assertEqual(len({row["text"] for row in rows}), 1)
                self.assertEqual(len({row["relation_label"] for row in rows}), 1)
                self.assertEqual(len({row["relation_input"] for row in rows}), 1)
            positive_groups = {
                group_cid
                for group_cid, rows in groups.items()
                if int(rows[0]["relation_label"]) == 1
            }
            self.assertEqual(
                positive_groups, set(record["positive_relation_group_cids"])
            )
            if record["duplicate_agreement"]:
                duplicate_records += 1
                positive_rows = [
                    span
                    for span in record["sentence_spans"]
                    if span["relation_label"] == 1
                ]
                self.assertEqual(len(positive_rows), 2)
                self.assertEqual(
                    len({span["relation_group_cid"] for span in positive_rows}), 1
                )
                self.assertEqual(
                    record["target_span_index"],
                    min(span["candidate_index"] for span in positive_rows),
                )
            if record["target_outcome"] == "conflict":
                self.assertEqual(len(positive_groups), 2)
        self.assertEqual(duplicate_records, 22)

    def test_census_binds_grouping_and_defers_tokenizer_without_truncation(self) -> None:
        census = self.dataset["census"]
        for checks in census["partition_checks"].values():
            self.assertTrue(checks["complete_width_2_through_8_cells"])
            self.assertTrue(checks["balanced_three_outcomes_per_width"])
            self.assertTrue(checks["every_locative_text_has_both_labels"])
            self.assertTrue(checks["every_query_subject_has_answer_and_nonanswer"])
            self.assertTrue(checks["every_candidate_position_has_both_labels"])
            self.assertTrue(checks["group_prompts_and_labels_exact"])
            self.assertTrue(checks["distinct_group_prompt_cids_unique"])
        self.assertEqual(
            census["tokenizer_census"]["status"], "CAMPAIGN_BOUND_NOT_RUN"
        )
        self.assertIn("never truncate", census["tokenizer_census"]["required_next_action"])

    def test_products_are_separate_unopened_and_exactly_committed(self) -> None:
        records = self.products["records"]
        self.assertEqual(len(records), 4)
        self.assertEqual(
            [record["probe"] for record in records],
            [
                "answer-supported",
                "abstain-negated-nonlocative",
                "conflict-distinct-values",
                "answer-duplicate-agreement",
            ],
        )
        self.assertEqual(
            [record["target_outcome"] for record in records],
            ["answer", "abstain", "conflict", "answer"],
        )
        self.assertTrue(records[-1]["duplicate_agreement"])
        self.assertEqual(
            self.dataset["product_probe_commitments"],
            [record["record_cid"] for record in records],
        )
        self.assertEqual(
            self.dataset["product_probes_cid"], self.products["product_probes_cid"]
        )
        self.assertNotIn("records", self.dataset)
        self.assertNotIn("product", self.preflight)
        self.assertIn("must not open", self.products["access_policy"])

    def test_all_cids_reproduce_and_tampering_changes_commitments(self) -> None:
        all_records = [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.products["records"],
        ]
        for record in all_records:
            _reproduce_cid(record, "record_cid")
        _reproduce_cid(self.dataset["census"], "census_cid")
        _reproduce_cid(self.dataset["split_policy"], "split_policy_cid")
        _reproduce_cid(self.dataset, "dataset_cid")
        _reproduce_cid(self.preflight, "preflight_cid")
        _reproduce_cid(self.products, "product_probes_cid")

        tampered_record = deepcopy(all_records[0])
        original_record_cid = tampered_record.pop("record_cid")
        tampered_record["sentence_spans"][0]["relation_label"] ^= 1
        self.assertNotEqual(
            cid_bytes(canonical_json_bytes(tampered_record)), original_record_cid
        )
        tampered_dataset = deepcopy(self.dataset)
        original_dataset_cid = tampered_dataset.pop("dataset_cid")
        tampered_dataset["counts"]["preflight_fit"] += 1
        self.assertNotEqual(
            cid_bytes(canonical_json_bytes(tampered_dataset)), original_dataset_cid
        )

    def test_sb3_bytes_remain_frozen_and_sb4_bytes_are_deterministic(self) -> None:
        sb3_dataset, sb3_preflight, sb3_products = (
            build_source_relation_adapter_population()
        )
        self.assertEqual(sb3_dataset["dataset_cid"], SB3_FROZEN_CIDS["dataset"])
        self.assertEqual(sb3_preflight["preflight_cid"], SB3_FROZEN_CIDS["preflight"])
        self.assertEqual(
            sb3_products["product_probes_cid"], SB3_FROZEN_CIDS["products"]
        )
        self.assertEqual(
            sb3_dataset["census"]["census_cid"], SB3_FROZEN_CIDS["census"]
        )
        self.assertEqual(self.dataset["dataset_cid"], SB4_FROZEN_CIDS["dataset"])
        self.assertEqual(
            self.preflight["preflight_cid"], SB4_FROZEN_CIDS["preflight"]
        )
        self.assertEqual(
            self.products["product_probes_cid"], SB4_FROZEN_CIDS["products"]
        )
        self.assertEqual(
            self.dataset["census"]["census_cid"], SB4_FROZEN_CIDS["census"]
        )
        rebuilt = build_joint_candidate_margin_population()
        self.assertEqual(
            [canonical_json_bytes(value) for value in rebuilt],
            [
                canonical_json_bytes(self.dataset),
                canonical_json_bytes(self.preflight),
                canonical_json_bytes(self.products),
            ],
        )


if __name__ == "__main__":
    unittest.main()
