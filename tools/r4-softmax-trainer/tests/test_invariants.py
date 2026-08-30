"""Dependency-light invariants for the frozen #1014 run contract."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from blake3 import blake3

from r4_softmax_trainer.constants import (
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
    FROZEN_MODEL_CONFIG,
    SEALED_PROMPT_TOKEN_COUNT,
    STORY_DELIMITER,
    TEST_REVEAL_TOTAL_CAP,
    TEST_TOKEN_CAP,
    TRAINING_VIEW_MANIFEST_SCHEMA,
)
from r4_softmax_trainer.data import (
    INDEX_RELATIVE_PATHS,
    TOKENIZER_RELATIVE_PATH,
    TOKEN_RELATIVE_PATHS,
    iter_canonical_stories,
    load_training_view_manifest,
    story_split,
)
from r4_softmax_trainer.provenance import (
    canonical_json_bytes,
    cid_bytes,
    verify_bound_manifest,
    write_bound_manifest,
)


class SplitContractTests(unittest.TestCase):
    def test_split_uses_full_digest_modulo(self) -> None:
        story = next(
            candidate
            for index in range(10_000)
            if (
                candidate := f"deterministic story {index}".encode()
            )
            and int.from_bytes(blake3(candidate).digest(), "big") % 100
            != int.from_bytes(blake3(candidate).digest(length=8), "big") % 100
        )
        bucket = int.from_bytes(blake3(story).digest(), "big") % 100
        expected = "train" if bucket < 90 else "dev" if bucket < 95 else "test"
        self.assertEqual(story_split(story), expected)

    def test_same_story_always_has_same_split(self) -> None:
        story = b"Once upon a time there was a geometric route."
        self.assertEqual(story_split(story), story_split(bytes(story)))

    def test_story_parser_is_chunk_boundary_independent(self) -> None:
        payload = b"  first story\n" + STORY_DELIMITER + b"\r\nsecond story  " + STORY_DELIMITER
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "stories.txt"
            source.write_bytes(payload)
            self.assertEqual(
                list(iter_canonical_stories(source, chunk_size=3)),
                [b"first story", b"second story"],
            )


class ProvenanceContractTests(unittest.TestCase):
    def test_canonical_json_ignores_mapping_insertion_order(self) -> None:
        left = {"z": 1, "a": [2, 3]}
        right = {"a": [2, 3], "z": 1}
        self.assertEqual(canonical_json_bytes(left), canonical_json_bytes(right))
        self.assertEqual(cid_bytes(canonical_json_bytes(left)), cid_bytes(canonical_json_bytes(right)))

    def test_manifest_binds_path_size_and_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "artifact.bin").write_bytes(b"bounded evidence")
            manifest_path = root / "manifest.json"
            written = write_bound_manifest(
                manifest_path,
                {"schema": "test/1"},
                artifact_root=root,
                relative_paths=["artifact.bin"],
            )
            verified = verify_bound_manifest(manifest_path, artifact_root=root)
            self.assertEqual(verified, written)
            (root / "artifact.bin").write_bytes(b"tampered evidence")
            with self.assertRaises(ValueError):
                verify_bound_manifest(manifest_path, artifact_root=root)

    def test_manifest_cid_is_reproducible(self) -> None:
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            manifests = []
            for directory in [first, second]:
                root = Path(directory)
                (root / "a").write_bytes(b"a")
                (root / "b").write_bytes(b"b")
                manifests.append(
                    write_bound_manifest(
                        root / "manifest.json",
                        {"schema": "test/1", "answer": 42},
                        artifact_root=root,
                        relative_paths=["b", "a"],
                    )
                )
            self.assertEqual(manifests[0], manifests[1])

    def test_training_view_never_opens_sealed_test_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            training_paths = [
                TOKENIZER_RELATIVE_PATH,
                TOKEN_RELATIVE_PATHS["train"],
                TOKEN_RELATIVE_PATHS["dev"],
                INDEX_RELATIVE_PATHS["train"],
                INDEX_RELATIVE_PATHS["dev"],
            ]
            for relative in training_paths:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode())
            sealed = root / TOKEN_RELATIVE_PATHS["test"]
            sealed.parent.mkdir(parents=True, exist_ok=True)
            sealed.write_bytes(b"must remain unopened")
            write_bound_manifest(
                root / "training-view-manifest.json",
                {
                    "schema": TRAINING_VIEW_MANIFEST_SCHEMA,
                    "dataset_manifest_cid": "blake3:" + "0" * 64,
                    "split_policy_cid": "blake3:" + "1" * 64,
                    "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
                    "sealed_test_commitment": {"path": TOKEN_RELATIVE_PATHS["test"]},
                },
                artifact_root=root,
                relative_paths=training_paths,
            )
            original_open = Path.open
            opened: list[str] = []

            def audited_open(path: Path, *args: object, **kwargs: object):
                opened.append(str(path))
                if "sealed-test" in path.parts:
                    raise AssertionError(f"sealed path opened: {path}")
                return original_open(path, *args, **kwargs)

            with mock.patch.object(Path, "open", audited_open):
                load_training_view_manifest(root)
            self.assertFalse(any("sealed-test" in path for path in opened))


class FrozenConfigTests(unittest.TestCase):
    def test_exact_r4_head_geometry(self) -> None:
        config = FROZEN_MODEL_CONFIG
        config.validate()
        self.assertEqual(config.hidden_size, 288)
        self.assertEqual(config.num_hidden_layers, 6)
        self.assertEqual(config.num_attention_heads, 6)
        self.assertEqual(config.num_key_value_heads, 6)
        self.assertEqual(config.head_dim, 48)
        self.assertEqual(config.r4_blocks_per_head, 12)
        self.assertEqual(config.head_dim, 4 * config.r4_blocks_per_head)
        self.assertEqual(config.intermediate_size, 768)
        self.assertEqual(config.max_position_embeddings, 256)
        self.assertEqual(config.vocab_size, 4096)

    def test_hugging_face_config_matches_rust_loader_contract(self) -> None:
        config = FROZEN_MODEL_CONFIG.as_hugging_face_config()
        self.assertEqual(config["model_type"], "llama")
        self.assertEqual(config["hidden_act"], "silu")
        self.assertEqual(config["rms_norm_eps"], 1e-5)
        self.assertEqual(config["rope_theta"], 10_000.0)
        self.assertIs(config["rope_scaling"], None)
        self.assertIs(config["rope_interleaved"], False)
        self.assertIs(config["attention_bias"], False)
        self.assertIs(config["mlp_bias"], False)
        self.assertIs(config["tie_word_embeddings"], True)
        self.assertEqual(config["bos_token_id"], BOS_TOKEN_ID)
        self.assertEqual(config["eos_token_id"], EOS_TOKEN_ID)

    def test_closed_form_parameter_count(self) -> None:
        config = FROZEN_MODEL_CONFIG
        embedding = config.vocab_size * config.hidden_size
        per_layer = (
            4 * config.hidden_size * config.hidden_size
            + 3 * config.hidden_size * config.intermediate_size
            + 2 * config.hidden_size
        )
        final_norm = config.hidden_size
        self.assertEqual(embedding + config.num_hidden_layers * per_layer + final_norm, 7_155_360)

    def test_contract_json_round_trip(self) -> None:
        contract = FROZEN_MODEL_CONFIG.as_contract()
        self.assertEqual(json.loads(canonical_json_bytes(contract)), contract)

    def test_sealed_reveal_budget_is_strict(self) -> None:
        self.assertEqual(TEST_TOKEN_CAP, 249_880)
        self.assertEqual(SEALED_PROMPT_TOKEN_COUNT, 120)
        self.assertEqual(TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT, TEST_REVEAL_TOTAL_CAP)


if __name__ == "__main__":
    unittest.main()
