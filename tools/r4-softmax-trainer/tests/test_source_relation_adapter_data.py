"""Focused contract tests for the frozen C1-SB3 relation population."""

from __future__ import annotations

from collections import Counter, defaultdict
import unittest

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.source_relation_adapter_data import (
    DEVELOPMENT_WORLDS_PER_WIDTH,
    NO_TOKEN_ID,
    NO_TOKEN_TEXT,
    OUTCOMES,
    POLICY,
    RELATION_INPUT_POLICY,
    RELATION_RECORD_SCHEMA,
    SOURCE_WIDTHS,
    YES_TOKEN_ID,
    YES_TOKEN_TEXT,
    build_source_relation_adapter_population,
    render_adapter_relation_input,
)


def _reproduce_cid(value: dict[str, object], field: str) -> str:
    unsigned = dict(value)
    expected = str(unsigned.pop(field))
    actual = cid_bytes(canonical_json_bytes(unsigned))
    if actual != expected:
        raise AssertionError(f"{field} does not reproduce: {expected} != {actual}")
    return actual


class RelationAdapterInputTests(unittest.TestCase):
    def test_exact_prefix_ends_at_supported_colon(self) -> None:
        value = render_adapter_relation_input(
            "The coral alder compass is inside the brass cabinet.",
            "Where is the coral alder compass?",
        )
        self.assertEqual(
            value,
            "Evidence:\nThe coral alder compass is inside the brass cabinet.\n"
            "Question:\nWhere is the coral alder compass?\nSupported:",
        )
        self.assertTrue(value.endswith("Supported:"))
        self.assertFalse(value.endswith("Supported:\n"))
        self.assertIn("no terminal newline", RELATION_INPUT_POLICY)
        self.assertEqual((YES_TOKEN_ID, YES_TOKEN_TEXT), (1_771, " yes"))
        self.assertEqual((NO_TOKEN_ID, NO_TOKEN_TEXT), (542, " no"))


class RelationAdapterPopulationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.preflight, cls.products = (
            build_source_relation_adapter_population()
        )

    def test_exact_partition_counts_and_balanced_cells(self) -> None:
        self.assertEqual(self.dataset["policy"], POLICY)
        self.assertEqual(
            self.dataset["counts"],
            {
                "preflight_fit": 126,
                "preflight_sealed": 63,
                "construction": 756,
                "development": 252,
                "development_reversal_controls": 252,
                "development_query_swap_controls": 168,
                "product_probe_commitments": 4,
            },
        )
        for records, worlds_per_width in (
            (self.preflight["fit"], 2),
            (self.preflight["sealed"], 1),
            (self.dataset["construction"], 12),
            (self.dataset["development"], DEVELOPMENT_WORLDS_PER_WIDTH),
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

    def test_every_world_has_nine_counterfactual_motifs(self) -> None:
        expected = {
            "matched-primary-answer",
            "matched-secondary-answer",
            "exact-duplicate-agreement",
            "negated-nonlocative-abstain",
            "primary-source-secondary-abstain",
            "secondary-source-primary-abstain",
            "primary-distinct-location-conflict",
            "secondary-distinct-location-conflict",
            "duplicate-distinct-location-conflict",
        }
        for records in (
            self.preflight["fit"],
            self.preflight["sealed"],
            self.dataset["construction"],
            self.dataset["development"],
        ):
            worlds: dict[str, list[dict[str, object]]] = defaultdict(list)
            for record in records:
                worlds[str(record["lexical_world"])].append(record)
            for world_records in worlds.values():
                self.assertEqual(len(world_records), 9)
                self.assertEqual(
                    {record["motif"] for record in world_records}, expected
                )
                self.assertEqual(
                    Counter(record["target_outcome"] for record in world_records),
                    {"answer": 3, "abstain": 3, "conflict": 3},
                )

    def test_zero_training_census_rejects_lexical_and_position_shortcuts(self) -> None:
        census = self.dataset["census"]
        self.assertTrue(census["passed"])
        self.assertTrue(census["sentence_partitions_pairwise_disjoint"])
        self.assertTrue(census["lexical_banks_pairwise_disjoint"])
        for checks in census["partition_checks"].values():
            self.assertTrue(checks["balanced_outcomes_per_width"])
            self.assertTrue(checks["every_locative_text_has_both_labels"])
            self.assertTrue(checks["every_query_subject_has_answer_and_nonanswer"])
            self.assertTrue(checks["every_candidate_position_has_both_labels"])

    def test_duplicate_agreement_and_distinct_conflict_are_exact(self) -> None:
        records = [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.dataset["construction"],
            *self.dataset["development"],
        ]
        for record in records:
            if record["motif"] == "exact-duplicate-agreement":
                positives = [
                    record["sentence_spans"][index]
                    for index in record["positive_span_indices"]
                ]
                self.assertEqual(len(positives), 2)
                self.assertEqual(len({span["text"] for span in positives}), 1)
                self.assertTrue(record["duplicate_agreement"])
                self.assertEqual(
                    record["target_span_index"], min(record["positive_span_indices"])
                )
            if record["target_outcome"] == "conflict":
                self.assertEqual(len(record["positive_relation_group_cids"]), 2)
                self.assertIsNone(record["target_span_index"])
                self.assertFalse(record["duplicate_agreement"])

    def test_development_reversals_and_query_swaps_are_bound(self) -> None:
        development = {
            record["record_cid"]: record for record in self.dataset["development"]
        }
        reversals = self.dataset["development_controls"]["reversal"]
        swaps = self.dataset["development_controls"]["query_swap"]
        self.assertEqual(len(reversals), 252)
        self.assertEqual(len(swaps), 168)
        for control in reversals:
            base = development[control["base_record_cid"]]
            self.assertEqual(control["target_outcome"], base["target_outcome"])
            self.assertEqual(
                [span["text"] for span in control["sentence_spans"]],
                list(reversed([span["text"] for span in base["sentence_spans"]])),
            )
            self.assertEqual(
                control["candidate_original_indices"],
                list(reversed(range(base["source_width"]))),
            )
        by_cid = {record["record_cid"]: record for record in self.dataset["development"]}
        for control in swaps:
            base = by_cid[control["base_record_cid"]]
            paired = by_cid[control["paired_record_cid"]]
            self.assertEqual(base["source_cid"], paired["source_cid"])
            self.assertEqual(control["source_cid"], base["source_cid"])
            self.assertNotEqual(control["subject"], base["subject"])
            self.assertEqual(control["target_outcome"], paired["target_outcome"])

    def test_four_product_records_are_committed_and_disjoint(self) -> None:
        records = self.products["records"]
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
        self.assertTrue(records[3]["duplicate_agreement"])
        self.assertEqual(
            self.dataset["product_probe_commitments"],
            [record["record_cid"] for record in records],
        )
        training_sentences = {
            span["text"]
            for record in [
                *self.preflight["fit"],
                *self.preflight["sealed"],
                *self.dataset["construction"],
                *self.dataset["development"],
            ]
            for span in record["sentence_spans"]
        }
        product_sentences = {
            span["text"] for record in records for span in record["sentence_spans"]
        }
        self.assertTrue(training_sentences.isdisjoint(product_sentences))
        self.assertIn("must not open record text", self.products["access_policy"])

    def test_records_and_envelopes_are_canonical_and_reproducible(self) -> None:
        all_records = [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.dataset["construction"],
            *self.dataset["development"],
            *self.dataset["development_controls"]["reversal"],
            *self.dataset["development_controls"]["query_swap"],
            *self.products["records"],
        ]
        for record in all_records:
            self.assertEqual(record["schema"], RELATION_RECORD_SCHEMA)
            _reproduce_cid(record, "record_cid")
            for span in record["sentence_spans"]:
                self.assertEqual(
                    span["relation_input"],
                    render_adapter_relation_input(span["text"], record["question"]),
                )
                self.assertTrue(span["relation_input"].endswith("Supported:"))
        _reproduce_cid(self.dataset["census"], "census_cid")
        _reproduce_cid(self.dataset, "dataset_cid")
        _reproduce_cid(self.preflight, "preflight_cid")
        _reproduce_cid(self.products, "product_probes_cid")

        other = build_source_relation_adapter_population()
        self.assertEqual(
            [canonical_json_bytes(value) for value in other],
            [
                canonical_json_bytes(self.dataset),
                canonical_json_bytes(self.preflight),
                canonical_json_bytes(self.products),
            ],
        )


if __name__ == "__main__":
    unittest.main()
