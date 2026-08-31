"""Focused contract tests for the frozen C1-SB2 relation population."""

from __future__ import annotations

from collections import Counter
import unittest

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.source_relation_data import (
    CONSTRUCTION_BANK,
    DEVELOPMENT_BANK,
    OUTCOMES,
    POLICY,
    PREFLIGHT_FAMILIES,
    PRODUCT_LOCATIONS,
    PRODUCT_NONLOCATIVES,
    PRODUCT_OBJECTS,
    RELATION_INPUT_POLICY,
    RELATION_RECORD_SCHEMA,
    SOURCE_WIDTHS,
    build_source_relation_population,
    render_relation_input,
)


def _reproduce_cid(value: dict[str, object], field: str) -> str:
    unsigned = dict(value)
    expected = str(unsigned.pop(field))
    actual = cid_bytes(canonical_json_bytes(unsigned))
    if actual != expected:
        raise AssertionError(f"{field} does not reproduce: {expected} != {actual}")
    return actual


class RelationInputTests(unittest.TestCase):
    def test_evidence_precedes_question_and_question_mark_is_final(self) -> None:
        rendered = render_relation_input(
            "The coral abacus is inside the aspen locker.",
            "Where is the coral abacus?",
        )
        self.assertEqual(
            rendered,
            "Evidence:\nThe coral abacus is inside the aspen locker.\n"
            "Question:\nWhere is the coral abacus?",
        )
        self.assertTrue(rendered.endswith("?"))
        self.assertFalse(rendered.endswith("?\n"))
        self.assertIn("no terminal newline", RELATION_INPUT_POLICY)

    def test_relation_input_rejects_noncanonical_evidence_or_question(self) -> None:
        rejected = [
            ("unterminated", "Where is the coral abacus?"),
            (" trailing. ", "Where is the coral abacus?"),
            ("The coral abacus is listed.", "where is the coral abacus?"),
        ]
        for span, question in rejected:
            with self.subTest(span=span, question=question), self.assertRaises(ValueError):
                render_relation_input(span, question)


class SourceRelationPopulationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.preflight, cls.products = build_source_relation_population()

    def test_exact_balanced_population_counts_and_widths(self) -> None:
        self.assertEqual(self.dataset["policy"], POLICY)
        self.assertEqual(self.dataset["counts"]["construction"], 3_360)
        self.assertEqual(self.dataset["counts"]["development"], 420)
        self.assertEqual(self.dataset["counts"]["product_probe_commitments"], 4)
        for population, per_cell in (("construction", 160), ("development", 20)):
            cells: Counter[tuple[str, int]] = Counter(
                (record["target_outcome"], record["source_width"])
                for record in self.dataset[population]
            )
            self.assertEqual(
                cells,
                Counter(
                    {
                        (outcome, width): per_cell
                        for outcome in OUTCOMES
                        for width in SOURCE_WIDTHS
                    }
                ),
            )

    def test_relations_require_locative_value_not_raw_subject_count(self) -> None:
        for population, singleton_count, agreement_count in (
            ("construction", 80, 80),
            ("development", 10, 10),
        ):
            records = self.dataset[population]
            for width in SOURCE_WIDTHS:
                answers = [
                    record
                    for record in records
                    if record["target_outcome"] == "answer"
                    and record["source_width"] == width
                ]
                motifs = Counter(record["motif"] for record in answers)
                self.assertEqual(
                    motifs,
                    {"singleton": singleton_count, "duplicate-agreement": agreement_count},
                )
                conflicts = [
                    record
                    for record in records
                    if record["target_outcome"] == "conflict"
                    and record["source_width"] == width
                ]
                self.assertTrue(conflicts)
                for record in conflicts:
                    positives = [
                        record["sentence_spans"][index]["text"]
                        for index in record["positive_span_indices"]
                    ]
                    self.assertEqual(len(positives), 2)
                    self.assertEqual(len(set(positives)), 2)
                abstentions = [
                    record
                    for record in records
                    if record["target_outcome"] == "abstain"
                    and record["source_width"] == width
                ]
                self.assertTrue(
                    all(not record["positive_span_indices"] for record in abstentions)
                )
                self.assertTrue(
                    all(record["raw_subject_occurrence_count"] >= 2 for record in abstentions)
                )
                if width >= 3:
                    for cell in (answers, abstentions, conflicts):
                        self.assertTrue(
                            any(
                                span["role"].startswith("same-subject-")
                                for candidate in cell
                                for span in candidate["sentence_spans"]
                            )
                        )

            for record in records:
                if record["motif"] == "singleton":
                    self.assertEqual(len(record["positive_span_indices"]), 1)
                    self.assertFalse(record["duplicate_agreement"])
                elif record["motif"] == "duplicate-agreement":
                    positives = [
                        record["sentence_spans"][index]
                        for index in record["positive_span_indices"]
                    ]
                    self.assertEqual(len(positives), 2)
                    self.assertEqual(positives[0]["text_cid"], positives[1]["text_cid"])
                    self.assertEqual(
                        record["target_span_index"], min(record["positive_span_indices"])
                    )

        census = self.dataset["shortcut_census"]
        self.assertTrue(census["passed"])
        self.assertFalse(census["count_only_label_lookup_is_perfect"])
        for partition in ("construction", "development"):
            for cell in census[f"{partition}_count_cells"]:
                self.assertEqual(set(cell["outcomes_present"]), set(OUTCOMES))

    def test_candidate_and_conflict_pair_positions_are_balanced(self) -> None:
        for population in ("construction", "development"):
            records = self.dataset[population]
            for outcome in ("answer", "conflict"):
                for width in SOURCE_WIDTHS:
                    cell = [
                        record
                        for record in records
                        if record["target_outcome"] == outcome
                        and record["source_width"] == width
                    ]
                    incidence = Counter(
                        index for record in cell for index in record["positive_span_indices"]
                    )
                    self.assertLessEqual(
                        max(incidence.values()) - min(incidence.values()), 1
                    )
                    if outcome == "conflict":
                        pair_counts = Counter(
                            tuple(record["positive_span_indices"]) for record in cell
                        )
                        self.assertLessEqual(
                            max(pair_counts.values()) - min(pair_counts.values()), 1
                        )

    def test_preflight_is_matched_twelve_fit_twelve_sealed_transfer(self) -> None:
        self.assertEqual(self.preflight["counts"]["fit"], 12)
        self.assertEqual(self.preflight["counts"]["sealed"], 12)
        self.assertEqual(self.preflight["counts"]["matched_pairs"], 8)
        expected_motifs = {
            "same-source-absent-query",
            "same-source-present-locative-query",
            "queried-subject-nonlocative-only",
            "query-locative-distractor-subject",
            "exact-duplicate-agreement",
            "distinct-location-conflict",
        }
        for partition, family_names in (
            ("fit", self.preflight["fit_family_names"]),
            ("sealed", self.preflight["sealed_family_names"]),
        ):
            records = self.preflight[partition]
            self.assertEqual(len(family_names), 2)
            for family_name in family_names:
                family_records = [
                    record
                    for record in records
                    if record["lexical_family"] == family_name
                ]
                self.assertEqual(
                    {record["motif"] for record in family_records}, expected_motifs
                )
                duplicate = next(
                    record
                    for record in family_records
                    if record["motif"] == "exact-duplicate-agreement"
                )
                self.assertTrue(duplicate["duplicate_agreement"])
                conflict = next(
                    record
                    for record in family_records
                    if record["motif"] == "distinct-location-conflict"
                )
                self.assertEqual(len(conflict["positive_relation_group_cids"]), 2)
        self.assertTrue(all(pair["same_source"] for pair in self.preflight["matched_pairs"]))
        self.assertTrue(
            set(self.preflight["fit_family_names"]).isdisjoint(
                self.preflight["sealed_family_names"]
            )
        )

    def test_development_controls_preserve_source_or_remap_order(self) -> None:
        development = {
            record["record_cid"]: record for record in self.dataset["development"]
        }
        reversals = self.dataset["development_controls"]["reversal"]
        swaps = self.dataset["development_controls"]["query_swap"]
        self.assertEqual(len(reversals), 420)
        self.assertGreaterEqual(len(swaps), 300)
        for control in reversals:
            base = development[control["base_record_cid"]]
            self.assertEqual(
                control["candidate_original_indices"],
                list(reversed(range(base["source_width"]))),
            )
            self.assertEqual(control["target_outcome"], base["target_outcome"])
            self.assertEqual(
                [span["text"] for span in control["sentence_spans"]],
                list(reversed([span["text"] for span in base["sentence_spans"]])),
            )
        for control in swaps:
            base = development[control["base_record_cid"]]
            self.assertEqual(control["source_cid"], base["source_cid"])
            self.assertNotEqual(control["subject"], base["subject"])
            self.assertEqual(control["target_outcome"], "answer")
            self.assertEqual(
                control["target_span_index"], control["expected_original_span_index"]
            )
            if base["target_outcome"] == "answer":
                self.assertNotEqual(control["target_span_index"], base["target_span_index"])

    def test_four_product_commitments_are_exact_and_python_disjoint(self) -> None:
        records = self.products["records"]
        self.assertEqual(
            [record["probe"] for record in records],
            [
                "opal-astrolabe-supported",
                "silk-atlas-abstain",
                "opal-astrolabe-conflict",
                "jade-sextant-duplicate-agreement",
            ],
        )
        self.assertEqual(
            [record["target_outcome"] for record in records],
            ["answer", "abstain", "conflict", "answer"],
        )
        self.assertEqual(
            records[0]["source"],
            "The opal astrolabe was polished before sunrise. "
            "The brass sundial is beside the north alcove. "
            "The opal astrolabe is beneath the maple stair.",
        )
        self.assertEqual(records[3]["target_span_index"], 0)
        self.assertTrue(records[3]["duplicate_agreement"])
        self.assertEqual(
            self.dataset["product_probe_commitments"],
            [record["record_cid"] for record in records],
        )
        training_sentences = {
            span["text"]
            for record in [
                *self.dataset["construction"],
                *self.dataset["development"],
            ]
            for span in record["sentence_spans"]
        }
        product_sentences = {
            span["text"] for record in records for span in record["sentence_spans"]
        }
        self.assertTrue(training_sentences.isdisjoint(product_sentences))

    def test_lexical_banks_records_and_content_ids_are_disjoint_and_canonical(self) -> None:
        construction_bank = {
            *CONSTRUCTION_BANK["subjects"],
            *CONSTRUCTION_BANK["locations"],
            *CONSTRUCTION_BANK["nonlocatives"],
        }
        development_bank = {
            *DEVELOPMENT_BANK["subjects"],
            *DEVELOPMENT_BANK["locations"],
            *DEVELOPMENT_BANK["nonlocatives"],
        }
        product_bank = {*PRODUCT_OBJECTS, *PRODUCT_LOCATIONS, *PRODUCT_NONLOCATIVES}
        preflight_bank = {
            value
            for family in PREFLIGHT_FAMILIES
            for key, value in family.items()
            if key != "name"
        }
        self.assertTrue(construction_bank.isdisjoint(development_bank))
        self.assertTrue(construction_bank.isdisjoint(product_bank))
        self.assertTrue(development_bank.isdisjoint(product_bank))
        self.assertTrue(
            preflight_bank.isdisjoint(
                construction_bank | development_bank | product_bank
            )
        )
        census = self.dataset["shortcut_census"]
        self.assertTrue(census["construction_development_subjects_disjoint"])
        self.assertTrue(census["construction_development_sentences_disjoint"])
        self.assertTrue(census["construction_development_product_subjects_disjoint"])
        self.assertTrue(census["construction_development_product_sentences_disjoint"])

        all_records = [
            *self.dataset["construction"],
            *self.dataset["development"],
            *self.dataset["development_controls"]["reversal"],
            *self.dataset["development_controls"]["query_swap"],
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.products["records"],
        ]
        for record in all_records:
            self.assertEqual(record["schema"], RELATION_RECORD_SCHEMA)
            _reproduce_cid(record, "record_cid")
            for span in record["sentence_spans"]:
                self.assertEqual(
                    span["relation_input"],
                    render_relation_input(span["text"], record["question"]),
                )
                self.assertTrue(span["relation_input"].endswith("?"))
                self.assertEqual(span["relation_group_cid"], span["text_cid"])
        _reproduce_cid(self.dataset, "dataset_cid")
        _reproduce_cid(self.preflight, "preflight_cid")
        _reproduce_cid(self.products, "product_probes_cid")

    def test_rebuild_is_byte_identical(self) -> None:
        other_dataset, other_preflight, other_products = build_source_relation_population()
        self.assertEqual(canonical_json_bytes(other_dataset), canonical_json_bytes(self.dataset))
        self.assertEqual(
            canonical_json_bytes(other_preflight), canonical_json_bytes(self.preflight)
        )
        self.assertEqual(canonical_json_bytes(other_products), canonical_json_bytes(self.products))


if __name__ == "__main__":
    unittest.main()
