"""Focused contract checks for #1043's data and reveal boundary."""

from __future__ import annotations

import stat
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from blake3 import blake3

from r4_softmax_trainer import position_kv_binding_data as subject
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


class _TinyTokenizer:
    def __init__(self) -> None:
        self._decoded: dict[tuple[int, ...], str] = {}

    def encode(self, text: str, add_special_tokens: bool = True) -> SimpleNamespace:
        digest = blake3(text.encode("utf-8")).digest()
        ids = tuple(2 + value % 200 for value in digest[:12])
        self._decoded[ids] = text
        return SimpleNamespace(ids=list(ids))

    def decode(self, ids: list[int], skip_special_tokens: bool = False) -> str:
        return self._decoded.get(tuple(ids), "<invalid serialized tokens>")


def _tiny_token_ids() -> dict[str, int]:
    values = {
        lexeme: 256 + index
        for index, lexeme in enumerate((*subject.KEY_LEXICON, *subject.VALUE_LEXICON))
    }
    values["unknown"] = subject.UNKNOWN_TOKEN_ID
    return values


class PositionKVDeterminismTests(unittest.TestCase):
    def test_mqar_is_causal_partitioned_deterministic_and_directly_recoverable(self) -> None:
        construction = subject._generate_mqar_examples(count=32, terminal=False)
        terminal = subject._generate_mqar_examples(count=16, terminal=True)
        self.assertEqual(
            construction,
            subject._generate_mqar_examples(count=32, terminal=False),
        )
        self.assertEqual({len(row.input_ids) for row in construction}, {subject.CONTEXT})
        self.assertEqual({len(row.query_positions) for row in construction}, {8})
        self.assertTrue(
            all(
                subject.mqar_pair_partition(key, value) != 0
                for row in construction
                for key, value in zip(row.binding_keys, row.binding_values, strict=True)
            )
        )
        self.assertTrue(
            all(
                subject.mqar_pair_partition(key, value) == 0
                for row in terminal
                for key, value in zip(row.binding_keys, row.binding_values, strict=True)
            )
        )
        oracle = subject.serialization_oracle(terminal, ())
        self.assertEqual(oracle.mqar_correct, 128)
        self.assertEqual(oracle.mqar_total, 128)
        self.assertEqual(oracle.ambiguous_bindings, 0)
        self.assertEqual(oracle.missing_bindings, 0)
        mutated_tokens = list(terminal[0].input_ids)
        mutated_tokens[1] = (
            subject.MQAR_VALUE_MIN
            if mutated_tokens[1] != subject.MQAR_VALUE_MIN
            else subject.MQAR_VALUE_MIN + 1
        )
        mutated = replace(
            terminal[0],
            input_ids=tuple(mutated_tokens),
            sequence_cid=subject._sequence_cid(mutated_tokens, terminal[0].label_ids),
        )
        mutated_oracle = subject.serialization_oracle((mutated, *terminal[1:]), ())
        self.assertLess(mutated_oracle.mqar_correct, mutated_oracle.mqar_total)
        controls = subject.binding_permuted_examples(terminal)
        self.assertEqual(len(controls), len(terminal))
        self.assertTrue(
            all(control.answers == native.answers for control, native in zip(controls, terminal, strict=True))
        )
        self.assertTrue(
            all(control.sequence_cid != native.sequence_cid for control, native in zip(controls, terminal, strict=True))
        )

    def test_english_templates_are_partitioned_and_terminal_queries_are_independent(self) -> None:
        tokenizer = _TinyTokenizer()
        token_ids = _tiny_token_ids()
        construction, no_history = subject._generate_english_examples(
            tokenizer=tokenizer,  # type: ignore[arg-type]
            token_ids=token_ids,
            terminal=False,
            history_count=12,
            no_history_count=4,
        )
        terminal, terminal_no_history = subject._generate_english_examples(
            tokenizer=tokenizer,  # type: ignore[arg-type]
            token_ids=token_ids,
            terminal=True,
            history_count=8,
            no_history_count=8,
        )
        self.assertEqual([row.family_index for row in construction[:6]], [0, 1, 2, 0, 1, 2])
        self.assertEqual({row.answers for row in no_history}, {(subject.UNKNOWN_TOKEN_ID,)})
        self.assertEqual({row.answers for row in terminal_no_history}, {(subject.UNKNOWN_TOKEN_ID,)})
        self.assertTrue(
            all(
                subject.english_pair_partition(key, value) != 0
                for row in construction
                for key, value in row.binding_names
            )
        )
        self.assertTrue(
            all(
                subject.english_pair_partition(key, value) == 0
                for row in terminal
                for key, value in row.binding_names
            )
        )
        for world in range(4):
            rows = [row for row in terminal if row.world_index == world]
            self.assertEqual(len(rows), 2)
            self.assertNotEqual(rows[0].query_keys, rows[1].query_keys)
            self.assertNotEqual(rows[0].sequence_cid, rows[1].sequence_cid)
        oracle = subject.serialization_oracle(
            (),
            terminal,
            tokenizer=tokenizer,  # type: ignore[arg-type]
            token_ids=token_ids,
        )
        self.assertEqual(oracle.english_correct, 8)
        self.assertEqual(oracle.english_total, 8)
        mutated_tokens = list(terminal[0].input_ids)
        mutated_tokens[0] = subject.VOCAB_SIZE - 1
        mutated = replace(
            terminal[0],
            input_ids=tuple(mutated_tokens),
            sequence_cid=subject._sequence_cid(mutated_tokens, terminal[0].label_ids),
        )
        mutated_oracle = subject.serialization_oracle(
            (),
            (mutated, *terminal[1:]),
            tokenizer=tokenizer,  # type: ignore[arg-type]
            token_ids=token_ids,
        )
        self.assertLess(mutated_oracle.english_correct, mutated_oracle.english_total)
        disjoint = subject._population_disjointness((), (), construction, terminal)
        self.assertTrue(all(value is True for key, value in disjoint.items() if key.endswith("intersection")))

    def test_natural_orders_and_post_v5_story_windows_are_exact(self) -> None:
        self.assertEqual(
            subject.deterministic_natural_replay_ordinals(window_count=8, take=4),
            (5, 1, 2, 0),
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "index.jsonl"
            records = [
                {
                    "capacity_story_ordinal": ordinal,
                    "source_story_ordinal": 10 + ordinal,
                    "story_cid": f"blake3:{ordinal + 1:064x}",
                    "story_token_offset": ordinal * 400,
                    "story_token_count": 300,
                }
                for ordinal in range(4)
            ]
            payload = b"".join(canonical_json_bytes(record) for record in records)
            path.write_bytes(payload)
            excluded = records[2]["story_cid"]
            with mock.patch.multiple(
                subject,
                FRESH_HELDOUT_TRAIN_INDEX_CID=cid_bytes(payload),
                FRESH_HELDOUT_LAST_CAPACITY_STORY=1,
                FRESH_HELDOUT_LAST_SOURCE_STORY=11,
            ):
                coordinates = subject._terminal_natural_coordinates(
                    path,
                    excluded_story_cids=(excluded,),
                    count=2,
                )
            self.assertEqual(
                coordinates,
                (
                    (1_200, records[3]["story_cid"], 13),
                    (1_321, records[3]["story_cid"], 13),
                ),
            )


class PositionKVLeakageBoundaryTests(unittest.TestCase):
    @staticmethod
    def _write_envelope(
        path: Path,
        body: dict[str, object],
        cid_field: str,
    ) -> dict[str, object]:
        value = subject._with_self_cid(body, cid_field)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(canonical_json_bytes(value))
        return value

    def _write_completed_fit_envelopes(
        self,
        *,
        root: Path,
        manifest: dict[str, object],
        artifact: Path,
    ) -> None:
        implementation = {"tree_cid": "blake3:" + "1" * 64}
        preparation = self._write_envelope(
            root / subject.CAMPAIGN_PREPARATION_RELATIVE_PATH,
            {
                "schema": subject.CAMPAIGN_PREPARATION_SCHEMA,
                "issue": subject.ISSUE,
                "policy": subject.POLICY,
                "implementation": implementation,
                "data_manifest": manifest,
                "data_manifest_cid": manifest["manifest_cid"],
            },
            "preparation_cid",
        )
        preflight = self._write_envelope(
            root / subject.CAMPAIGN_PREFLIGHT_RELATIVE_PATH,
            {
                "schema": subject.CAMPAIGN_PREFLIGHT_SCHEMA,
                "issue": subject.ISSUE,
                "policy": subject.POLICY,
                "preparation_cid": preparation["preparation_cid"],
                "data_manifest_cid": manifest["manifest_cid"],
                "implementation": implementation,
                "passed": True,
                "terminal_payload_reads": 0,
            },
            "preflight_cid",
        )
        run_contract = {
            "policy": subject.POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "implementation": implementation,
            "plan": {"name": "test-cpu", "threads": 1},
            "optimizer": {
                "steps": subject.CAMPAIGN_OPTIMIZER_STEPS,
                "batch_size": 16,
                "composition": {
                    "natural": 8,
                    "mqar": 4,
                    "english_history": 3,
                    "english_no_history": 1,
                },
                "checkpoint_selection": "NONE",
            },
            "terminal_payload": "SEALED_UNOPENED",
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        }
        started = self._write_envelope(
            root / subject.CAMPAIGN_STARTED_RELATIVE_PATH,
            {
                "schema": subject.CAMPAIGN_STARTED_SCHEMA,
                "issue": subject.ISSUE,
                "policy": subject.POLICY,
                "preparation_cid": preparation["preparation_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "implementation": implementation,
                "run_contract": run_contract,
                "run_contract_cid": cid_bytes(canonical_json_bytes(run_contract)),
                "terminal_payload_reads": 0,
            },
            "started_cid",
        )
        self._write_envelope(
            root / subject.CAMPAIGN_FIT_RELATIVE_PATH,
            {
                "schema": subject.CAMPAIGN_FIT_SCHEMA,
                "issue": subject.ISSUE,
                "policy": subject.POLICY,
                "started_cid": started["started_cid"],
                "preparation_cid": preparation["preparation_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "run_contract_cid": started["run_contract_cid"],
                "implementation": implementation,
                "plan": run_contract["plan"],
                "completed_steps": subject.CAMPAIGN_OPTIMIZER_STEPS,
                "presentations": {
                    "natural": subject.NATURAL_CONSTRUCTION_WINDOWS,
                    "mqar": subject.MQAR_CONSTRUCTION_SEQUENCES,
                    "english_history": subject.ENGLISH_CONSTRUCTION_HISTORY,
                    "english_no_history": subject.ENGLISH_CONSTRUCTION_NO_HISTORY,
                },
                "first_loss": {"total": 1.0},
                "final_loss": {"total": 0.5},
                "loss_trace_cid": "blake3:" + "2" * 64,
                "elapsed_seconds": 1.0,
                "artifact": {
                    "path": subject.CAMPAIGN_ARTIFACT_RELATIVE_PATH,
                    "bytes": artifact.stat().st_size,
                    "cid": subject.cid_file(artifact),
                },
                "work": {
                    "target_reads": (
                        subject.NATURAL_CONSTRUCTION_DECISIONS
                        + subject.MQAR_CONSTRUCTION_DECISIONS
                        + subject.ENGLISH_CONSTRUCTION_HISTORY
                        + subject.ENGLISH_CONSTRUCTION_NO_HISTORY
                    ),
                    "provider_calls": 0,
                    "teacher_calls": 0,
                    "future_reads": 0,
                    "forbidden_reads": 0,
                },
                "terminal_payload_reads_before_artifact_cid": 0,
                "optimizer_steps_after_reveal": 0,
            },
            "fit_cid",
        )

    def test_prepare_is_create_once_and_terminal_requires_a_real_final_artifact(self) -> None:
        tokenizer = _TinyTokenizer()
        token_ids = _tiny_token_ids()
        one_window = bytes(subject.WINDOW_TOKENS * 2)
        natural_construction_selection = {
            "schema": subject.NATURAL_SELECTION_SCHEMA,
            "split": "construction",
            "payload_cid": cid_bytes(one_window),
        }
        natural_terminal_selection = {
            "schema": subject.NATURAL_SELECTION_SCHEMA,
            "split": "terminal",
            "payload_cid": cid_bytes(one_window),
        }
        identities = {
            "NATURAL_CONSTRUCTION_WINDOWS": 1,
            "NATURAL_TERMINAL_WINDOWS": 1,
            "MQAR_CONSTRUCTION_SEQUENCES": 2,
            "MQAR_CONSTRUCTION_DECISIONS": 16,
            "MQAR_TERMINAL_SEQUENCES": 2,
            "MQAR_TERMINAL_DECISIONS": 16,
            "ENGLISH_CONSTRUCTION_HISTORY": 3,
            "ENGLISH_CONSTRUCTION_NO_HISTORY": 1,
            "ENGLISH_TERMINAL_WORLDS": 1,
            "ENGLISH_TERMINAL_HISTORY": 2,
            "ENGLISH_TERMINAL_NO_HISTORY": 2,
            "CAMPAIGN_ARTIFACT_BYTES": len(b"fixed-final-artifact"),
        }
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root = base / "prepared"
            fake_tokenizer_path = base / "tokenizer.json"
            fake_tokenizer_path.write_text("{}", encoding="utf-8")
            with (
                mock.patch.multiple(subject, **identities),
                mock.patch.object(subject, "_validate_frozen_arithmetic"),
                mock.patch.object(
                    subject,
                    "validate_tokenizer",
                    return_value=(tokenizer, token_ids),
                ),
                mock.patch.object(
                    subject,
                    "_materialize_natural_construction",
                    return_value=(one_window, natural_construction_selection),
                ),
                mock.patch.object(
                    subject,
                    "_materialize_natural_terminal",
                    return_value=(one_window, natural_terminal_selection),
                ),
            ):
                prepared = subject.prepare_position_kv_binding_data(
                    output_root=root,
                    retained_language_root=base / "retained",
                    source_root=base / "source",
                    tokenizer_path=fake_tokenizer_path,
                    excluded_story_cids=("blake3:" + "f" * 64,),
                )
                self.assertEqual(len(prepared.construction.mqar), 2)
                no_history_identity = prepared.commitment[
                    "english_no_history_serialization"
                ]
                self.assertEqual(no_history_identity["rows"], 2)
                self.assertGreaterEqual(no_history_identity["unique_inputs"], 1)
                self.assertIn("not independent", no_history_identity["role"])
                sealed = root / subject.SEALED_DIRECTORY_RELATIVE_PATH
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0)
                with self.assertRaises(ValueError):
                    subject.reveal_position_kv_binding_terminal(
                        root,
                        final_artifact_path=base / "missing.safetensors",
                    )
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0)
                with self.assertRaises(FileExistsError):
                    subject.prepare_position_kv_binding_data(
                        output_root=root,
                        retained_language_root=base / "retained",
                        source_root=base / "source",
                        tokenizer_path=fake_tokenizer_path,
                        excluded_story_cids=("blake3:" + "f" * 64,),
                    )
                loose_artifact = base / "model.safetensors"
                loose_artifact.write_bytes(b"fixed-final-artifact")
                with self.assertRaisesRegex(ValueError, "bound final artifact path"):
                    subject.reveal_position_kv_binding_terminal(
                        root,
                        final_artifact_path=loose_artifact,
                    )
                artifact = root / subject.CAMPAIGN_ARTIFACT_RELATIVE_PATH
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(b"fixed-final-artifact")
                with self.assertRaises(ValueError):
                    subject.reveal_position_kv_binding_terminal(
                        root,
                        final_artifact_path=artifact,
                    )
                self._write_completed_fit_envelopes(
                    root=root,
                    manifest=prepared.manifest,  # type: ignore[arg-type]
                    artifact=artifact,
                )
                terminal = subject.reveal_position_kv_binding_terminal(
                    root,
                    final_artifact_path=artifact,
                )
                self.assertEqual(len(terminal.mqar), 2)
                self.assertEqual(len(terminal.mqar_binding_permuted), 2)
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0o700)
                replay = subject.load_revealed_position_kv_binding_terminal(
                    root,
                    final_artifact_path=artifact,
                )
                self.assertEqual(replay.final_artifact_cid, terminal.final_artifact_cid)
                other = base / "other.safetensors"
                other.write_bytes(b"another-final-artifact")
                with self.assertRaisesRegex(ValueError, "artifact"):
                    subject.load_revealed_position_kv_binding_terminal(
                        root,
                        final_artifact_path=other,
                    )


if __name__ == "__main__":
    unittest.main()
