"""Focused contracts for the fresh C1-SB5 paired-query population."""

from __future__ import annotations

from collections import Counter, defaultdict
from copy import deepcopy
from pathlib import Path
import tempfile
import unittest

from tokenizers import Tokenizer
from tokenizers.models import WordLevel
from tokenizers.pre_tokenizers import Whitespace

from r4_softmax_trainer.constants import BOS_TOKEN_ID
from r4_softmax_trainer.paired_query_binding_data import (
    DATASET_SCHEMA,
    EXPECTED_COUNTS,
    FRESH_WORLD_ORDINAL_START,
    MAX_POSITIONS_INCLUDING_BOS,
    PAIR_KINDS,
    POLICY,
    PREFLIGHT_SCHEMA,
    PRODUCT_DENIED_FILENAMES,
    PRODUCT_FILENAME,
    PRODUCT_MANIFEST_FILENAME,
    PRODUCT_SCHEMA,
    TOKENIZER_CENSUS_SCHEMA,
    TRAINING_VIEW_FILENAMES,
    artifact_bytes,
    build_paired_query_binding_population,
    build_paired_query_binding_semantic_population,
    load_artifact,
    render_paired_query_input,
    verify_artifact_cid,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


def _toy_tokenizer() -> Tokenizer:
    tokenizer = Tokenizer(
        WordLevel(
            {
                "[UNK]": 0,
                ".": 1,
                "?": 2,
                ":": 3,
            },
            unk_token="[UNK]",
        )
    )
    tokenizer.pre_tokenizer = Whitespace()
    return tokenizer


class PairedQueryRendererTests(unittest.TestCase):
    def test_exact_lane_contains_source_once_and_ends_at_bind_colon(self) -> None:
        source = "The coral alder compass is inside the brass cabinet."
        question = "Where is the coral alder compass?"
        rendered = render_paired_query_input(source, question)
        self.assertEqual(rendered, f"E:{source}\nQ:{question}\nBind:")
        self.assertEqual(rendered.count(source), 1)
        self.assertTrue(rendered.endswith("Bind:"))
        self.assertFalse(rendered.endswith("\n"))

    def test_renderer_rejects_noncanonical_source_or_question(self) -> None:
        rejected = (
            ("unterminated", "Where is the coral alder compass?"),
            (" trailing. ", "Where is the coral alder compass?"),
            ("The coral alder compass is listed.", "where is it?"),
        )
        for source, question in rejected:
            with self.subTest(source=source, question=question):
                with self.assertRaises(ValueError):
                    render_paired_query_input(source, question)


class PairedQuerySemanticPopulationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.preflight, cls.products = (
            build_paired_query_binding_semantic_population()
        )

    def test_exact_partition_counts_worlds_and_outcomes(self) -> None:
        self.assertEqual(self.dataset["schema"], DATASET_SCHEMA)
        self.assertEqual(self.preflight["schema"], PREFLIGHT_SCHEMA)
        self.assertEqual(self.products["schema"], PRODUCT_SCHEMA)
        self.assertEqual(self.dataset["policy"], POLICY)
        self.assertEqual(self.dataset["counts"], EXPECTED_COUNTS)
        self.assertEqual(self.preflight["counts"]["fit"], EXPECTED_COUNTS["fit"])
        self.assertEqual(
            self.preflight["counts"]["sealed"], EXPECTED_COUNTS["sealed"]
        )

        for partition, worlds_per_width in (("fit", 2), ("sealed", 1)):
            pairs = self.preflight[partition]
            self.assertEqual(
                Counter((pair["source_width"], pair["pair_slot"]) for pair in pairs),
                Counter(
                    {
                        (width, pair_slot): worlds_per_width
                        for width in range(2, 9)
                        for pair_slot in range(4)
                    }
                ),
            )
            by_world: dict[str, list[dict[str, object]]] = defaultdict(list)
            for pair in pairs:
                by_world[str(pair["lexical_world"])].append(pair)
            for world_pairs in by_world.values():
                self.assertEqual({pair["pair_slot"] for pair in world_pairs}, set(range(4)))
                self.assertEqual(
                    {pair["pair_kind"] for pair in world_pairs}, set(PAIR_KINDS)
                )

        fit_ordinals = {pair["world_ordinal"] for pair in self.preflight["fit"]}
        sealed_ordinals = {pair["world_ordinal"] for pair in self.preflight["sealed"]}
        product_ordinals = {pair["world_ordinal"] for pair in self.products["records"]}
        self.assertEqual(fit_ordinals, set(range(162, 176)))
        self.assertEqual(sealed_ordinals, set(range(176, 183)))
        self.assertEqual(product_ordinals, set(range(183, 187)))
        self.assertEqual(min(fit_ordinals), FRESH_WORLD_ORDINAL_START)

    def test_every_pair_is_an_exact_shared_source_counterfactual_matrix(self) -> None:
        pairs = [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.products["records"],
        ]
        for pair in pairs:
            self.assertEqual(len(pair["queries"]), 2)
            self.assertNotEqual(
                pair["queries"][0]["question"], pair["queries"][1]["question"]
            )
            self.assertEqual(
                {query["relation_input"].split("\nQ:", 1)[0] for query in pair["queries"]},
                {f'E:{pair["source"]}'},
            )
            groups = pair["candidate_groups"]
            matrix = pair["label_matrix"]
            self.assertEqual([query["labels"] for query in pair["queries"]], matrix)
            self.assertEqual(len(matrix), 2)
            self.assertTrue(all(len(row) == len(groups) for row in matrix))
            expected_flips = {
                group["relation_group_cid"]
                for index, group in enumerate(groups)
                if {matrix[0][index], matrix[1][index]} == {0, 1}
            }
            self.assertEqual(set(pair["flip_group_cids"]), expected_flips)
            self.assertEqual(len(expected_flips), sum(sum(row) for row in matrix))
            for query, row in zip(pair["queries"], matrix):
                positives = sum(row)
                expected_outcome = (
                    "abstain" if positives == 0 else "answer" if positives == 1 else "conflict"
                )
                self.assertEqual(query["target_outcome"], expected_outcome)
            for group in groups:
                self.assertEqual(
                    group["relation_group_cid"],
                    cid_bytes(group["text"].encode("utf-8")),
                )
                self.assertEqual(
                    group["earliest_occurrence_index"],
                    min(group["occurrence_indices"]),
                )

    def test_duplicate_pair_collapses_exact_text_and_uses_fresh_b_query(self) -> None:
        duplicate_pairs = [
            pair
            for pair in [*self.preflight["fit"], *self.preflight["sealed"]]
            if pair["pair_slot"] == 3
        ]
        self.assertEqual(len(duplicate_pairs), 21)
        for pair in duplicate_pairs:
            left, right = pair["queries"]
            self.assertEqual(left["target_outcome"], "answer")
            self.assertTrue(left["duplicate_agreement"])
            self.assertEqual(right["target_outcome"], "abstain")
            self.assertIsNone(right["inherited_record_cid"])
            supported_group = pair["candidate_groups"][left["target_group_index"]]
            self.assertEqual(len(supported_group["occurrence_indices"]), 2)
            self.assertEqual(
                left["target_span_index"], min(supported_group["occurrence_indices"])
            )

    def test_freshness_shortcut_and_product_isolation_census_is_positive(self) -> None:
        census = self.dataset["census"]
        self.assertTrue(census["passed"])
        self.assertTrue(census["counts_exact"])
        self.assertTrue(census["new_sentence_partitions_pairwise_disjoint"])
        self.assertTrue(census["new_composite_world_partitions_pairwise_disjoint"])
        for checks in census["new_vs_sb3_sb4"].values():
            self.assertTrue(checks["subjects_locations_nonlocatives_disjoint"])
            self.assertTrue(checks["exact_sentences_disjoint"])
        for checks in census["partition_checks"].values():
            self.assertTrue(checks["pair_oracle_and_flip_matrix_exact"])
            self.assertTrue(checks["question_blind_shortcut_rejected"])
            self.assertEqual(checks["question_blind_affirmative_locative_exact_pairs"], 0)

        training_view = self.dataset["training_view"]
        self.assertEqual(
            set(training_view["allowed_filenames"]), set(TRAINING_VIEW_FILENAMES)
        )
        self.assertEqual(
            set(training_view["denied_filenames"]), set(PRODUCT_DENIED_FILENAMES)
        )
        self.assertNotIn(PRODUCT_FILENAME, TRAINING_VIEW_FILENAMES)
        self.assertNotIn(PRODUCT_MANIFEST_FILENAME, TRAINING_VIEW_FILENAMES)
        self.assertEqual(self.products["training_view_access"], "DENIED")
        self.assertEqual(
            self.dataset["product_probe_commitments"],
            [record["record_cid"] for record in self.products["records"]],
        )
        training_sources = {
            pair["source"]
            for pair in [*self.preflight["fit"], *self.preflight["sealed"]]
        }
        self.assertTrue(
            training_sources.isdisjoint(
                {pair["source"] for pair in self.products["records"]}
            )
        )

    def test_all_semantic_cids_reproduce_and_population_is_deterministic(self) -> None:
        verify_artifact_cid(self.dataset, "dataset_cid")
        verify_artifact_cid(self.preflight, "preflight_cid")
        verify_artifact_cid(self.products, "product_probes_cid")
        verify_artifact_cid(self.dataset["census"], "census_cid")
        for pair in [
            *self.preflight["fit"],
            *self.preflight["sealed"],
            *self.products["records"],
        ]:
            verify_artifact_cid(pair, "record_cid")
            for query in pair["queries"]:
                verify_artifact_cid(query, "query_row_cid")
        rebuilt = build_paired_query_binding_semantic_population()
        self.assertEqual(
            tuple(artifact_bytes(value) for value in rebuilt),
            tuple(
                artifact_bytes(value)
                for value in (self.dataset, self.preflight, self.products)
            ),
        )

    def test_canonical_artifact_loader_reproduces_and_rejects_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "dataset.json"
            path.write_bytes(artifact_bytes(self.dataset))
            loaded = load_artifact(
                path, schema=DATASET_SCHEMA, cid_field="dataset_cid"
            )
            self.assertEqual(loaded, self.dataset)

            corrupt = deepcopy(self.dataset)
            corrupt["policy"] = "drifted"
            path.write_bytes(canonical_json_bytes(corrupt))
            with self.assertRaises(ValueError):
                load_artifact(path, schema=DATASET_SCHEMA, cid_field="dataset_cid")


class PairedQueryTokenizerBindingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tokenizer = _toy_tokenizer()
        cls.tokenizer_cid = cid_bytes(cls.tokenizer.to_str().encode("utf-8"))
        cls.dataset, cls.preflight, cls.products = build_paired_query_binding_population(
            cls.tokenizer, tokenizer_cid=cls.tokenizer_cid
        )

    def test_bound_population_exposes_the_mechanism_interface(self) -> None:
        self.assertEqual(self.dataset["binding"], "TOKENIZER_BOUND")
        self.assertEqual(self.preflight["binding"], "TOKENIZER_BOUND")
        census = self.dataset["tokenizer_census"]
        self.assertEqual(census["schema"], TOKENIZER_CENSUS_SCHEMA)
        self.assertTrue(census["passed"])
        self.assertEqual(census["tokenizer_cid"], self.tokenizer_cid)

        for partition in ("fit", "sealed"):
            for pair in self.preflight[partition]:
                verify_artifact_cid(pair, "record_cid")
                self.assertTrue(pair["source_prefix_identity_exact"])
                self.assertTrue(pair["candidate_anchor_identity_exact"])
                self.assertTrue(pair["all_candidate_states_before_query"])
                self.assertEqual(len(pair["queries"]), 2)
                self.assertEqual(len(pair["label_matrix"]), 2)
                left_ids = [BOS_TOKEN_ID, *pair["queries"][0]["token_ids"]]
                right_ids = [BOS_TOKEN_ID, *pair["queries"][1]["token_ids"]]
                prefix_count = pair["source_prefix_token_count"]
                self.assertEqual(left_ids[:prefix_count], right_ids[:prefix_count])
                self.assertEqual(
                    pair["candidate_terminal_indices"],
                    pair["queries"][0]["candidate_terminal_indices"],
                )
                self.assertEqual(
                    pair["candidate_terminal_indices"],
                    pair["queries"][1]["candidate_terminal_indices"],
                )
                self.assertEqual(
                    len(pair["candidate_terminal_indices"]),
                    len(pair["candidate_groups"]),
                )
                for query in pair["queries"]:
                    verify_artifact_cid(query, "query_row_cid")
                    self.assertEqual(
                        query["query_terminal_index"], len(query["token_ids"])
                    )
                    self.assertLessEqual(
                        query["positions_including_bos"],
                        MAX_POSITIONS_INCLUDING_BOS,
                    )
                    self.assertEqual(query["truncation"], "FORBIDDEN_NOT_USED")
                    self.assertGreaterEqual(
                        query["query_terminal_index"],
                        query["source_prefix_token_count"],
                    )
                    self.assertTrue(
                        all(
                            index < query["source_prefix_token_count"]
                            for index in query["candidate_terminal_indices"]
                        )
                    )
                    self.assertEqual(
                        query["candidate_terminal_bit_offsets_u32"],
                        [index * 32 for index in query["candidate_terminal_indices"]],
                    )
                    self.assertEqual(
                        query["query_terminal_bit_offset_u32"],
                        query["query_terminal_index"] * 32,
                    )

    def test_products_remain_separate_and_unbound_in_training_return(self) -> None:
        self.assertEqual(self.products["schema"], PRODUCT_SCHEMA)
        self.assertEqual(self.products["training_view_access"], "DENIED")
        for pair in self.products["records"]:
            self.assertNotIn("tokenizer_cid", pair)
            self.assertNotIn("token_ids", pair["queries"][0])
        self.assertIn("product", self.dataset["tokenizer_census"]["partitions"])
        self.assertEqual(
            self.dataset["tokenizer_census"]["product_text_access"],
            "PREPARATION_ONLY_DENIED_TO_TRAINING_VIEW",
        )


if __name__ == "__main__":
    unittest.main()
