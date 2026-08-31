"""Contract tests for the frozen C1-SB1 population."""

from __future__ import annotations

import unittest

from r4_softmax_trainer.source_span_data import (
    build_source_span_population,
    parse_subject,
    split_sentence_spans,
)


class SourceSpanParsingTests(unittest.TestCase):
    def test_question_policy_is_exact(self) -> None:
        self.assertEqual(parse_subject("Where is the blue book?"), "blue book")
        for rejected in [
            "where is the blue book?",
            "Where is blue book?",
            "Where is the blue book ?",
            "Where is the blue book? now",
            "Where is the ?",
        ]:
            with self.subTest(rejected=rejected), self.assertRaises(ValueError):
                parse_subject(rejected)

    def test_sentence_spans_preserve_exact_utf8_bytes(self) -> None:
        source = "  The café is here!\nThe key is there?  "
        spans = split_sentence_spans(source)
        encoded = source.encode("utf-8")
        self.assertEqual([span["text"] for span in spans], ["The café is here!", "The key is there?"])
        for span in spans:
            self.assertEqual(
                encoded[span["byte_start"] : span["byte_end"]].decode("utf-8"),
                span["text"],
            )

    def test_unterminated_or_overwide_sources_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            split_sentence_spans("The source has no terminator")
        with self.assertRaises(ValueError):
            split_sentence_spans(" ".join(f"Sentence {index}." for index in range(9)))


class SourceSpanPopulationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.probes = build_source_span_population()

    def test_population_preserves_frozen_balanced_counts(self) -> None:
        self.assertEqual(self.dataset["counts"]["construction"], 3_072)
        self.assertEqual(self.dataset["counts"]["development"], 384)
        for records, per_class in [
            (self.dataset["construction"], 1_024),
            (self.dataset["development"], 128),
        ]:
            counts = {name: 0 for name in ["answer", "abstain", "conflict"]}
            for record in records:
                counts[record["target_outcome"]] += 1
            self.assertEqual(counts, {name: per_class for name in counts})

    def test_product_family_is_committed_but_absent_from_training(self) -> None:
        commitments = [record["record_cid"] for record in self.probes["records"]]
        self.assertEqual(self.dataset["product_probe_commitments"], commitments)
        training = "\n".join(
            record["source"]
            for record in [
                *self.dataset["construction"],
                *self.dataset["development"],
            ]
        )
        self.assertNotIn("copper compass", training)
        self.assertNotIn("linen map", training)
        self.assertEqual(
            [record["probe"] for record in self.probes["records"]],
            [
                "copper-compass-supported",
                "linen-map-unsupported",
                "copper-compass-conflict",
            ],
        )

    def test_dataset_and_probe_cids_reproduce(self) -> None:
        other_dataset, other_probes = build_source_span_population()
        self.assertEqual(other_dataset, self.dataset)
        self.assertEqual(other_probes, self.probes)


if __name__ == "__main__":
    unittest.main()
