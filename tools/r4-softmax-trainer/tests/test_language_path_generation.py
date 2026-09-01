from __future__ import annotations

import copy
import json
import tempfile
import unittest
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch

from r4_softmax_trainer import language_path_generation as subject
from r4_softmax_trainer.cli import parser
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


@dataclass(frozen=True)
class _Audit:
    batch_size: int = 1
    token_steps: int = 1
    layers: int = subject.LAYERS
    heads: int = subject.HEADS
    group_size: int = 120
    transported_state_values: int = subject.STATE_VALUES
    occupancy_slot_reads: int = subject.LAYERS * 120
    attention_slot_scores: int = subject.LAYERS * subject.HEADS * 120
    attention_value_reads: int = subject.LAYERS * subject.HEADS * 120 * subject.HEAD_DIM
    key_delta_writes: int = subject.LAYERS * subject.HEADS * subject.HEAD_DIM
    value_delta_writes: int = subject.LAYERS * subject.HEADS * subject.HEAD_DIM
    vocabulary_scores: int = subject.VOCAB_SIZE
    state_off: bool = False
    implementation: str = "direct"
    forbidden_reads: int = 0


class _Model:
    def __init__(
        self,
        next_token: int = subject.EOS_TOKEN_ID,
        *,
        finite_logits: bool = True,
    ) -> None:
        self.next_token = next_token
        self.finite_logits = finite_logits
        self.seen: list[int] = []
        self.initial_states = 0

    def initial_state(self, batch_size: int, *, device: object, dtype: object) -> int:
        if batch_size != 1 or device != torch.device("cpu") or dtype != torch.float32:
            raise AssertionError("unexpected fake-state request")
        self.initial_states += 1
        return 0

    def step(
        self,
        token_ids: torch.Tensor,
        state: int,
        *,
        attention_off: bool,
    ) -> SimpleNamespace:
        if attention_off or token_ids.shape != (1,):
            raise AssertionError("generation did not use direct attention-on step input")
        self.seen.append(int(token_ids.item()))
        logits = torch.full((1, subject.VOCAB_SIZE), -1000.0, dtype=torch.float32)
        logits[0, self.next_token] = 1000.0
        if not self.finite_logits:
            logits[0, 0] = float("nan")
        return SimpleNamespace(logits=logits, final_state=state + 1, audit=_Audit())


class _Tokenizer:
    def decode(self, ids: list[int], skip_special_tokens: bool = True) -> str:
        selected = (
            [token for token in ids if token not in (0, 1, 2)]
            if skip_special_tokens
            else ids
        )
        return " ".join(str(token) for token in selected)

    def decode_bytes(self, ids: list[int] | tuple[int, ...]) -> bytes:
        return " ".join(str(token) for token in ids).encode("utf-8")


class _InvalidRawDecoder:
    def decode_bytes(self, ids: list[int] | tuple[int, ...]) -> bytes:
        return b"\xff" if ids else b""


class _PromptRawDecoder:
    def __init__(self, *, corrupt_index: int | None = None) -> None:
        self.corrupt_index = corrupt_index

    def decode_bytes(self, ids: list[int] | tuple[int, ...]) -> bytes:
        for prompt in subject.PROMPTS:
            if tuple(ids) == prompt.token_ids:
                suffix = b"!" if prompt.index == self.corrupt_index else b""
                return prompt.text.encode("utf-8") + suffix
        return b""


class _SequenceSelector:
    def __init__(self, tokens: list[int]) -> None:
        self.tokens = tokens
        self.cursor = 0

    def __call__(
        self, logits: torch.Tensor | list[float], sampler: subject.SplitMix64
    ) -> int:
        del logits, sampler
        token = self.tokens[self.cursor]
        self.cursor += 1
        return token


class FrozenContractTests(unittest.TestCase):
    def test_public_prompts_and_cli_are_exact(self) -> None:
        self.assertEqual(len(subject.PROMPTS), 5)
        self.assertEqual(
            cid_bytes(canonical_json_bytes(subject.prompt_contract())),
            subject.EXPECTED_PROMPT_CONTRACT_CID,
        )
        self.assertEqual(
            [len(prompt.token_ids) for prompt in subject.PROMPTS],
            [9, 8, 10, 11, 9],
        )
        self.assertTrue(
            all(
                not set(prompt.token_ids) & {0, 1, 2}
                and 1 + len(prompt.token_ids) + subject.MAX_NEW_TOKENS
                <= subject.CONTEXT
                for prompt in subject.PROMPTS
            )
        )
        arguments = parser().parse_args(
            ["--root", "/tmp/language-path", "generate-language-path"]
        )
        self.assertEqual(arguments.command, "generate-language-path")

    def test_manifest_verification_names_only_tokenizer_and_geometry(self) -> None:
        manifest = {
            "schema": subject.DATA_MANIFEST_SCHEMA,
            "policy": "R4RetainedLanguagePathV1",
            "manifest_cid": subject.EXPECTED_PREPARATION_MANIFEST_CID,
            "geometry": {
                "artifact_cid": subject.EXPECTED_GEOMETRY_ARTIFACT_CID,
                "file_cid": subject.EXPECTED_GEOMETRY_FILE_CID,
            },
            "source": {"tokenizer_cid": subject.EXPECTED_TOKENIZER_CID},
        }
        with (
            mock.patch.object(
                subject, "_require_regular_path", return_value=Path("/root/manifest.json")
            ),
            mock.patch.object(subject, "verify_manifest_envelope", return_value=manifest),
            mock.patch.object(subject, "verify_artifact_subset") as subset,
        ):
            self.assertIs(subject._verify_data_manifest(Path("/root")), manifest)
        subset.assert_called_once_with(
            manifest,
            artifact_root=Path("/root"),
            relative_paths=(
                subject.TOKENIZER_RELATIVE_PATH,
                subject.GEOMETRY_RELATIVE_PATH,
            ),
        )

    def test_bytelevel_raw_decoder_preserves_invalid_bytes_and_added_literals(self) -> None:
        tokenizer_json = {
            "model": {
                "type": "BPE",
                "vocab": {"<|bos|>": 0, "<|eos|>": 1, "<|unk|>": 2, "ÿ": 3},
            },
            "decoder": {"type": "ByteLevel"},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"},
            ],
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tokenizer.json"
            path.write_text(json.dumps(tokenizer_json), encoding="utf-8")
            decoder = subject.ByteLevelRawDecoder.from_tokenizer_json(path)
        self.assertEqual(decoder.decode_bytes([3]), b"\xff")
        self.assertEqual(decoder.decode_bytes([0, 2]), b"<|bos|><|unk|>")

    def test_public_prompts_must_round_trip_as_exact_bytes(self) -> None:
        subject._validate_prompt_bytes(_PromptRawDecoder())
        with self.assertRaisesRegex(ValueError, "prompt 3"):
            subject._validate_prompt_bytes(_PromptRawDecoder(corrupt_index=3))

    def test_archived_executed_runner_is_byte_exact_and_required(self) -> None:
        self.assertEqual(
            subject._verify_executed_runner_archive(),
            subject.EXECUTED_RUNNER_RECORD,
        )
        with (
            mock.patch.object(subject, "cid_file", return_value="blake3:" + "f" * 64),
            self.assertRaisesRegex(ValueError, "archived executed generation runner"),
        ):
            subject._verify_executed_runner_archive()


class SamplerTests(unittest.TestCase):
    def test_splitmix_and_equal_logit_sampler_match_fixed_vectors(self) -> None:
        self.assertEqual(subject.SplitMix64(0).next_u64(), 0xE220_A839_7B1D_CDAF)
        sampler = subject.SplitMix64(subject.SEED)
        logits = [0.0] * 64
        self.assertEqual(
            [subject.sample_top_k_q32(logits, sampler) for _ in range(8)],
            [29, 21, 19, 10, 23, 34, 0, 26],
        )

    def test_total_order_and_rust_positive_rounding_are_explicit(self) -> None:
        self.assertEqual(
            [token for token, _ in subject._top_k_q32_weights([-0.0, 0.0])],
            [1, 0],
        )
        self.assertEqual(subject._positive_rust_round(2.5), 3)
        self.assertEqual(subject._positive_rust_round(2.49), 2)

    def test_nonuniform_sampler_matches_compiled_rust_reference(self) -> None:
        logits = [
            -0.0,
            0.0,
            1.25,
            -2.5,
            1.25,
            0.125,
            -0.125,
            3.0,
            2.75,
            -7.0,
            0.5,
            0.50000006,
            1.0,
            -1.0,
            2.0,
            1.9999999,
        ]
        sampler = subject.SplitMix64(subject.SEED)
        self.assertEqual(
            [subject.sample_top_k_q32(logits, sampler) for _ in range(16)],
            [15, 8, 8, 7, 8, 4, 7, 14, 8, 8, 8, 15, 10, 13, 8, 8],
        )


class RolloutTests(unittest.TestCase):
    def setUp(self) -> None:
        self.prompt = subject.GenerationPrompt(0, "test", (35,))
        self.tokenizer = _Tokenizer()

    def test_eos_is_recorded_and_never_fed(self) -> None:
        model = _Model()
        record = subject._rollout(
            self.prompt,
            raw_decoder=self.tokenizer,
            model_factory=lambda: model,
            select_token=_SequenceSelector([subject.EOS_TOKEN_ID]),
        )
        self.assertEqual(record["generated_token_ids"], [subject.EOS_TOKEN_ID])
        self.assertEqual(record["fed_back_generated_token_ids"], [])
        self.assertEqual(record["stop_reason"], "eos")
        self.assertEqual(model.seen, [subject.BOS_TOKEN_ID, 35])

    def test_existing_transcript_cannot_append_after_eos_or_cycle(self) -> None:
        with self.assertRaisesRegex(ValueError, "continues after selected EOS"):
            subject._expected_stop([subject.EOS_TOKEN_ID, 5])
        with self.assertRaisesRegex(ValueError, "continues after a short cycle"):
            subject._expected_stop([5, 5, 5, 6])

    def test_period_two_cycle_stops_after_third_repeat_without_terminal_feed(self) -> None:
        model = _Model()
        sequence = [5, 6, 5, 6, 5, 6]
        record = subject._rollout(
            self.prompt,
            raw_decoder=self.tokenizer,
            model_factory=lambda: model,
            select_token=_SequenceSelector(sequence),
        )
        self.assertEqual(record["generated_token_ids"], sequence)
        self.assertEqual(record["short_cycle_period"], 2)
        self.assertEqual(record["stop_reason"], {"short_cycle": {"period": 2}})
        self.assertEqual(model.seen, [subject.BOS_TOKEN_ID, 35, *sequence[:-1]])

    def test_maximum_horizon_self_feeds_only_first_63_outputs(self) -> None:
        model = _Model()
        sequence = list(range(100, 100 + subject.MAX_NEW_TOKENS))
        record = subject._rollout(
            self.prompt,
            raw_decoder=self.tokenizer,
            model_factory=lambda: model,
            select_token=_SequenceSelector(sequence),
        )
        self.assertEqual(record["generated_token_ids"], sequence)
        self.assertEqual(record["stop_reason"], "maximum_new_tokens")
        self.assertEqual(model.seen, [subject.BOS_TOKEN_ID, 35, *sequence[:-1]])
        self.assertEqual(record["audit"]["positions_executed"], 65)

    def test_nonfinite_logits_fail_before_any_selection(self) -> None:
        model = _Model(finite_logits=False)
        with self.assertRaisesRegex(RuntimeError, "nonfinite logits"):
            subject._rollout(
                self.prompt,
                raw_decoder=self.tokenizer,
                model_factory=lambda: model,
                select_token=_SequenceSelector([subject.EOS_TOKEN_ID]),
            )

    def test_invalid_raw_byte_is_preserved_and_fails_utf8_without_throwing(self) -> None:
        model = _Model()
        record = subject._rollout(
            self.prompt,
            raw_decoder=_InvalidRawDecoder(),
            model_factory=lambda: model,
            select_token=_SequenceSelector([97, subject.EOS_TOKEN_ID]),
        )
        self.assertFalse(record["utf8_decodable"])
        self.assertEqual(record["response_utf8_hex"], "ff")
        self.assertEqual(record["response_text"], "\N{REPLACEMENT CHARACTER}")


class ResultTests(unittest.TestCase):
    @staticmethod
    def _implementation() -> dict[str, object]:
        files = [
            {
                "path": "src/r4_softmax_trainer/language_path_generation.py",
                "bytes": 1,
                "cid": "blake3:" + "a" * 64,
            }
        ]
        return {"files": files, "tree_cid": subject.tree_cid(files)}

    @staticmethod
    def _environment() -> dict[str, object]:
        return {
            "platform": "Darwin",
            "backend": "cpu",
            "blas": "Apple Accelerate",
            "threads": 4,
            "workers": 1,
            "dtype": "float32",
            "deterministic_algorithms": True,
            "cuda": "FORBIDDEN",
            "torch": "test",
            "numpy": "test",
            "tokenizers": "test",
            "safetensors": "test",
        }

    @staticmethod
    def _official_historical_implementation() -> dict[str, object]:
        files = [
            {"path": path, "bytes": byte_count, "cid": cid}
            for path, (byte_count, cid) in subject.EXPECTED_MODEL_DEPENDENCIES.items()
        ]
        files.append(dict(subject.EXECUTED_RUNNER_RECORD))
        files.sort(key=lambda record: record["path"])
        implementation = {"files": files, "tree_cid": subject.tree_cid(files)}
        if implementation["tree_cid"] != subject.EXECUTED_IMPLEMENTATION_TREE_CID:
            raise AssertionError("historical test fixture tree drifted")
        return implementation

    def test_five_primary_and_fresh_replay_are_exact_and_create_once(self) -> None:
        models: list[_Model] = []
        first_random_values: list[int] = []

        def factory() -> _Model:
            model = _Model()
            models.append(model)
            return model

        def select_eos(
            logits: torch.Tensor | list[float], sampler: subject.SplitMix64
        ) -> int:
            del logits
            first_random_values.append(sampler.next_u64())
            return subject.EOS_TOKEN_ID

        input_evidence = {"parent_result_cid": subject.EXPECTED_PARENT_RESULT_CID}
        implementation = self._implementation()
        result = subject._execute_generation(
            raw_decoder=_Tokenizer(),
            model_factory=factory,
            input_evidence=input_evidence,
            implementation=implementation,
            environment=self._environment(),
            select_token=select_eos,
        )
        self.assertEqual(len(models), 10)
        self.assertEqual(len(set(first_random_values)), 1)
        self.assertTrue(all(model.initial_states == 1 for model in models))
        self.assertEqual(result["primary"], result["replay"])
        self.assertTrue(result["replay_equality"]["exact"])
        self.assertEqual(result["access"]["fresh_model_artifact_loads"], 10)
        self.assertEqual(result["access"]["source_data_reads"], 0)
        self.assertEqual(result["verdict"], "AUTONOMOUS_GENERATION_SMOKE_COMPLETE")
        subject._verify_self_cid(result, "result_cid")
        self.assertIsNone(subject._historical_runner_for_result(result))
        subject._validate_frozen_result(
            result,
            raw_decoder=_Tokenizer(),
            input_evidence=input_evidence,
            implementation=implementation,
            executed_runner_record=subject._historical_runner_for_result(result),
        )

        alternate = copy.deepcopy(result)
        alternate["decode"]["seed"] += 1
        alternate.pop("result_cid")
        alternate = subject._with_self_cid(alternate, "result_cid")
        with self.assertRaisesRegex(ValueError, "frozen derived evidence"):
            subject._validate_frozen_result(
                alternate,
                raw_decoder=_Tokenizer(),
                input_evidence=input_evidence,
                implementation=implementation,
            )

        audit_alias = copy.deepcopy(result)
        for arm in ("primary", "replay"):
            for record in audit_alias[arm]:
                record["audit"]["future_token_reads"] = False
                record.pop("transcript_cid")
                record["transcript_cid"] = cid_bytes(canonical_json_bytes(record))
        audit_alias["replay_equality"]["primary_cid"] = cid_bytes(
            canonical_json_bytes(audit_alias["primary"])
        )
        audit_alias["replay_equality"]["replay_cid"] = cid_bytes(
            canonical_json_bytes(audit_alias["replay"])
        )
        audit_alias.pop("result_cid")
        audit_alias["result_cid"] = cid_bytes(canonical_json_bytes(audit_alias))
        with self.assertRaisesRegex(ValueError, "audit differs"):
            subject._validate_frozen_result(
                audit_alias,
                raw_decoder=_Tokenizer(),
                input_evidence=input_evidence,
                implementation=implementation,
            )

        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "result.json"
            subject._write_exclusive_json(output, result)
            with self.assertRaises(FileExistsError):
                subject._write_exclusive_json(output, result)

    def test_invalid_utf8_is_a_create_once_terminal_invalid_result(self) -> None:
        input_evidence = {"parent_result_cid": subject.EXPECTED_PARENT_RESULT_CID}
        implementation = self._implementation()
        result = subject._execute_generation(
            raw_decoder=_InvalidRawDecoder(),
            model_factory=_Model,
            input_evidence=input_evidence,
            implementation=implementation,
            environment=self._environment(),
            select_token=_SequenceSelector([subject.EOS_TOKEN_ID] * 10),
        )
        self.assertFalse(result["mechanical_passed"])
        self.assertEqual(result["verdict"], "INVALID_AUTONOMOUS_GENERATION_SMOKE")
        subject._validate_frozen_result(
            result,
            raw_decoder=_InvalidRawDecoder(),
            input_evidence=input_evidence,
            implementation=implementation,
        )

    def test_existing_result_pins_executed_runner_but_revalidates_common_code(self) -> None:
        historical = self._official_historical_implementation()
        result = subject._execute_generation(
            raw_decoder=_Tokenizer(),
            model_factory=_Model,
            input_evidence={"parent_result_cid": subject.EXPECTED_PARENT_RESULT_CID},
            implementation=historical,
            environment=self._environment(),
            select_token=_SequenceSelector([subject.EOS_TOKEN_ID] * 10),
        )
        current = copy.deepcopy(historical)
        current_runner = next(
            record
            for record in current["files"]
            if record["path"] == subject.GENERATOR_RELATIVE_PATH
        )
        current_runner["bytes"] = 99_999
        current_runner["cid"] = "blake3:" + "b" * 64
        current["tree_cid"] = subject.tree_cid(current["files"])
        selected_runner = subject._historical_runner_for_result(result)
        self.assertEqual(selected_runner, subject.EXECUTED_RUNNER_RECORD)
        subject._validate_frozen_result(
            result,
            raw_decoder=_Tokenizer(),
            input_evidence={"parent_result_cid": subject.EXPECTED_PARENT_RESULT_CID},
            implementation=current,
            executed_runner_record=selected_runner,
        )

        changed_common = copy.deepcopy(current)
        changed_common["files"][0]["cid"] = "blake3:" + "c" * 64
        changed_common["tree_cid"] = subject.tree_cid(changed_common["files"])
        with self.assertRaisesRegex(ValueError, "verified dependencies"):
            subject._validate_frozen_result(
                result,
                raw_decoder=_Tokenizer(),
                input_evidence={"parent_result_cid": subject.EXPECTED_PARENT_RESULT_CID},
                implementation=changed_common,
                executed_runner_record=subject.EXECUTED_RUNNER_RECORD,
            )


if __name__ == "__main__":
    unittest.main()
