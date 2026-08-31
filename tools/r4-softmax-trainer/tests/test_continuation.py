"""Focused contract tests for the one frozen #1017 continuation."""

from __future__ import annotations

import json
import math
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import numpy as np
import torch

from r4_softmax_trainer.continuation import (
    CONTINUATION_OPTIMIZER_STEPS,
    CONTINUATION_TRAIN_TOKENS,
    CUMULATIVE_OPTIMIZER_STEPS,
    CUMULATIVE_TRAIN_TOKENS,
    ENABLED_PREFIX_SCHEMA,
    INHERITED_OPTIMIZER_STEP,
    INHERITED_RUN_CONTRACT_CID,
    INHERITED_TRAIN_TOKENS,
    PYTHON_ENABLED_PREFIX_RELATIVE_PATH,
    REVEAL_OPENED_RELATIVE_PATH,
    REVEAL_RESULT_RELATIVE_PATH,
    RUST_ENABLED_QUALIFICATION_SCHEMA,
    ContinuationConfig,
    _load_elapsed_ledger,
    _select_resume_checkpoint,
    _validate_enabled_rust_report,
    _validate_inherited_checkpoint_envelope,
    _validate_optimizer_state,
    _write_enabled_prefix_fixture,
    _write_elapsed_ledger,
    _write_reveal_opened_marker,
    _write_unavailable_mps_budget,
    continuation_batch_index,
    phase_two_learning_rate,
    reveal_continuation,
)
from r4_softmax_trainer.continuation_data import TOKEN_RELATIVE_PATHS
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


class FrozenContinuationArithmeticTests(unittest.TestCase):
    def test_exact_step_and_token_arithmetic(self) -> None:
        config = ContinuationConfig()
        config.validate()
        self.assertEqual(config.tokens_per_optimizer_step, 16_384)
        self.assertEqual(
            config.tokens_per_optimizer_step * CONTINUATION_OPTIMIZER_STEPS,
            CONTINUATION_TRAIN_TOKENS,
        )
        self.assertEqual(
            INHERITED_TRAIN_TOKENS + CONTINUATION_TRAIN_TOKENS,
            CUMULATIVE_TRAIN_TOKENS,
        )
        self.assertEqual(
            INHERITED_OPTIMIZER_STEP + CONTINUATION_OPTIMIZER_STEPS,
            CUMULATIVE_OPTIMIZER_STEPS,
        )

    def test_phase_two_schedule_has_exact_frozen_boundaries(self) -> None:
        config = ContinuationConfig()
        self.assertEqual(phase_two_learning_rate(0, config), 3e-5)
        self.assertEqual(phase_two_learning_rate(100, config), 3e-4)
        self.assertTrue(
            math.isclose(
                phase_two_learning_rate(CONTINUATION_OPTIMIZER_STEPS, config),
                3e-5,
                rel_tol=0.0,
                abs_tol=1e-15,
            )
        )
        self.assertGreater(phase_two_learning_rate(1, config), 3e-5)
        self.assertLess(phase_two_learning_rate(101, config), 3e-4)
        with self.assertRaises(ValueError):
            phase_two_learning_rate(CONTINUATION_OPTIMIZER_STEPS + 1, config)

    def test_config_cannot_be_tuned(self) -> None:
        with self.assertRaisesRegex(ValueError, "exact frozen"):
            ContinuationConfig(seed=1015).validate()

    def test_sampler_ledger_continues_after_1014_on_only_the_fresh_store(self) -> None:
        config = ContinuationConfig()
        self.assertEqual(
            continuation_batch_index(1, 0, config),
            INHERITED_OPTIMIZER_STEP * config.gradient_accumulation_steps,
        )
        self.assertEqual(
            continuation_batch_index(1, 3, config),
            INHERITED_OPTIMIZER_STEP * config.gradient_accumulation_steps + 3,
        )

    def test_wall_terminal_preserves_partial_only_as_uninterpretable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            status = _write_unavailable_mps_budget(
                root,
                run_contract_cid="blake3:" + "1" * 64,
                continuation_step=100,
                elapsed_seconds=18_900.0,
                config=ContinuationConfig(),
            )
            self.assertEqual(status["terminal"], "UNAVAILABLE_MPS_BUDGET")
            self.assertFalse(status["partial_checkpoint_interpretable"])
            self.assertFalse(status["selection_export_or_reveal_permitted"])
            unsigned = dict(status)
            result_cid = unsigned.pop("result_cid")
            self.assertEqual(result_cid, cid_bytes(canonical_json_bytes(unsigned)))

    def test_elapsed_ledger_is_signed_and_resume_selects_highest_step(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            latest_path = root / "latest.pt"
            best_path = root / "best.pt"
            latest_path.touch()
            best_path.touch()
            run_contract_cid = "blake3:" + "1" * 64
            ledger = _write_elapsed_ledger(
                root,
                run_contract_cid=run_contract_cid,
                continuation_step=105,
                elapsed_seconds=123.5,
                config=ContinuationConfig(),
            )
            self.assertEqual(
                _load_elapsed_ledger(
                    root,
                    run_contract_cid=run_contract_cid,
                    config=ContinuationConfig(),
                ),
                ledger,
            )
            with mock.patch(
                "r4_softmax_trainer.continuation._continuation_checkpoint_step",
                side_effect=lambda path, **_kwargs: 100 if path == latest_path else 105,
            ):
                self.assertEqual(
                    _select_resume_checkpoint(
                        latest_path=latest_path,
                        best_path=best_path,
                        run_contract_cid=run_contract_cid,
                        config=ContinuationConfig(),
                    ),
                    best_path,
                )


class InheritedStateTests(unittest.TestCase):
    @staticmethod
    def _optimizer(step: int = INHERITED_OPTIMIZER_STEP) -> torch.optim.AdamW:
        parameters = [torch.nn.Parameter(torch.zeros(())) for _ in range(56)]
        optimizer = torch.optim.AdamW(
            parameters,
            lr=3e-5,
            betas=(0.9, 0.95),
            eps=1e-8,
            weight_decay=0.1,
        )
        for parameter in parameters:
            optimizer.state[parameter] = {
                "step": torch.tensor(float(step)),
                "exp_avg": torch.zeros_like(parameter),
                "exp_avg_sq": torch.zeros_like(parameter),
            }
        return optimizer

    def test_exact_adamw_state_is_admitted(self) -> None:
        _validate_optimizer_state(
            self._optimizer(),
            expected_step=INHERITED_OPTIMIZER_STEP,
            expected_learning_rate=3e-5,
        )

    def test_partial_or_wrong_step_adamw_state_is_rejected(self) -> None:
        optimizer = self._optimizer(step=INHERITED_OPTIMIZER_STEP - 1)
        with self.assertRaisesRegex(ValueError, "state steps"):
            _validate_optimizer_state(
                optimizer,
                expected_step=INHERITED_OPTIMIZER_STEP,
                expected_learning_rate=3e-5,
            )
        optimizer.state.pop(next(iter(optimizer.state)))
        with self.assertRaisesRegex(ValueError, "56 parameter tensors"):
            _validate_optimizer_state(
                optimizer,
                expected_step=INHERITED_OPTIMIZER_STEP,
                expected_learning_rate=3e-5,
            )

    def test_inherited_envelope_rejects_token_or_step_drift(self) -> None:
        checkpoint = {
            "schema": "uor-r4-softmax-trainer-checkpoint/1",
            "run_contract_cid": INHERITED_RUN_CONTRACT_CID,
            "run_contract": {
                "model": ContinuationConfigModel.contract(),
            },
            "optimizer_step": INHERITED_OPTIMIZER_STEP,
            "tokens_seen": INHERITED_TRAIN_TOKENS,
        }
        with mock.patch(
            "r4_softmax_trainer.continuation.cid_bytes",
            return_value=INHERITED_RUN_CONTRACT_CID,
        ):
            _validate_inherited_checkpoint_envelope(checkpoint)
            checkpoint["tokens_seen"] -= 1
            with self.assertRaisesRegex(ValueError, "token count"):
                _validate_inherited_checkpoint_envelope(checkpoint)


class ContinuationConfigModel:
    """Avoid importing a second spelling of the frozen model contract in tests."""

    @staticmethod
    def contract() -> dict[str, object]:
        from r4_softmax_trainer.constants import FROZEN_MODEL_CONFIG

        return FROZEN_MODEL_CONFIG.as_contract()


class EnabledOnlyBoundaryTests(unittest.TestCase):
    def test_python_prefix_fixture_has_exact_rust_enabled_only_shape(self) -> None:
        class EnabledOnlyModel:
            def __call__(self, inputs: torch.Tensor) -> SimpleNamespace:
                logits = torch.zeros((1, inputs.shape[1], 4096), dtype=torch.float32)
                logits[0, -1, 17] = 2.0
                return SimpleNamespace(logits=logits)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dev_path = root / TOKEN_RELATIVE_PATHS["dev"]
            dev_path.parent.mkdir(parents=True)
            dev_path.write_bytes(np.arange(32, dtype="<u2").tobytes())
            store = SimpleNamespace(tokens=np.arange(32, dtype=np.uint16))
            fixture = _write_enabled_prefix_fixture(
                root,
                model=EnabledOnlyModel(),
                dev_store=store,
                device=torch.device("cpu"),
                weights_cid="blake3:" + "1" * 64,
            )
            self.assertEqual(fixture["schema"], ENABLED_PREFIX_SCHEMA)
            self.assertEqual(fixture["enabled"]["top1_token_id"], 17)
            self.assertNotIn("attention_off", fixture)
            self.assertEqual(
                set(fixture),
                {
                    "schema",
                    "weights_cid",
                    "token_store_cid",
                    "prefix_token_ids",
                    "maximum_absolute_logit_delta_limit",
                    "enabled",
                    "result_cid",
                },
            )
            unsigned = dict(fixture)
            result_cid = unsigned.pop("result_cid")
            self.assertEqual(result_cid, cid_bytes(canonical_json_bytes(unsigned)))
            self.assertTrue((root / PYTHON_ENABLED_PREFIX_RELATIVE_PATH).is_file())

    @staticmethod
    def _selection_and_prefix() -> tuple[dict[str, object], dict[str, object]]:
        selection: dict[str, object] = {
            "continuation_dataset_manifest_cid": "blake3:" + "1" * 64,
            "continuation_training_view_manifest_cid": "blake3:" + "2" * 64,
            "split_policy_cid": "blake3:" + "3" * 64,
            "run_contract_cid": "blake3:" + "4" * 64,
            "selected_checkpoint_cid": "blake3:" + "5" * 64,
            "weights_cid": "blake3:" + "6" * 64,
            "tokenizer_cid": "blake3:" + "7" * 64,
        }
        prefix: dict[str, object] = {
            "result_cid": "blake3:" + "8" * 64,
            "token_store_cid": "blake3:" + "9" * 64,
            "prefix_token_ids": list(range(32)),
        }
        return selection, prefix

    def test_rust_admission_rejects_any_attention_off_arm(self) -> None:
        selection, prefix = self._selection_and_prefix()
        report = {
            "schema": RUST_ENABLED_QUALIFICATION_SCHEMA,
            "issue": 1017,
            "qualification_passed": True,
            "attention_off_executions": 0,
            "provenance": {
                "dataset_manifest_cid": selection["continuation_dataset_manifest_cid"],
                "training_view_manifest_cid": selection[
                    "continuation_training_view_manifest_cid"
                ],
                "split_policy_cid": selection["split_policy_cid"],
                "run_contract_cid": selection["run_contract_cid"],
                "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
                "weights_cid": selection["weights_cid"],
                "tokenizer_cid": selection["tokenizer_cid"],
            },
            "evaluation_input": {
                "python_prefix_result_cid": prefix["result_cid"],
                "token_store_cid": prefix["token_store_cid"],
                "prefix_token_ids": prefix["prefix_token_ids"],
            },
            "enabled_prefix_parity": {
                "passed": True,
                "identical_top1": True,
                "maximum_absolute_logit_delta_within_limit": True,
                "maximum_absolute_logit_delta": 0.001,
            },
            "enabled": {
                "audit": {
                    "selected_layer_count": 6,
                    "all_layers_selected": True,
                    "causal_audits_exact": 1,
                    "projection_audits_exact": 1,
                    "r4_audits_exact": 1,
                    "output_policy_audits_exact": 1,
                    "future_reads": 0,
                    "zeroed_applications": 0,
                }
            },
            "source_read_audit": {
                "provider_calls": 0,
                "ollama_calls": 0,
                "prior_trace_reads": 0,
            },
        }
        _validate_enabled_rust_report(report, selection=selection, prefix=prefix)
        report["enabled_prefix_parity"]["maximum_absolute_logit_delta"] = 0.005
        with self.assertRaisesRegex(ValueError, "prefix parity failed"):
            _validate_enabled_rust_report(report, selection=selection, prefix=prefix)
        report["enabled_prefix_parity"]["maximum_absolute_logit_delta"] = 0.001
        report["attention_off"] = None
        with self.assertRaisesRegex(ValueError, "prohibited attention-off"):
            _validate_enabled_rust_report(report, selection=selection, prefix=prefix)

    def test_reveal_marker_makes_a_second_open_impossible(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selection = {
                "manifest_cid": "blake3:" + "1" * 64,
                "selected_checkpoint_cid": "blake3:" + "2" * 64,
                "continuation_dataset_manifest_cid": "blake3:" + "3" * 64,
            }
            admission = {"manifest_cid": "blake3:" + "4" * 64}
            marker = _write_reveal_opened_marker(
                root, selection=selection, parity_admission=admission
            )
            self.assertEqual(marker["terminal"], "SEALED_CONFIRMATION_OPEN_INITIATED")
            self.assertTrue((root / REVEAL_OPENED_RELATIVE_PATH).is_file())
            with self.assertRaisesRegex(FileExistsError, "already opened"):
                _write_reveal_opened_marker(
                    root, selection=selection, parity_admission=admission
                )


class RevealExecutionTests(unittest.TestCase):
    def test_failed_enabled_nll_is_frozen_without_running_attention_off(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run_contract = {"trainer_implementation": {"tree_cid": "tree"}}
            run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
            (root / "continuation-training-result.json").write_text(
                json.dumps(
                    {
                        "run_contract": run_contract,
                        "run_contract_cid": run_contract_cid,
                    }
                ),
                encoding="utf-8",
            )
            selection = {
                "manifest_cid": "blake3:" + "1" * 64,
                "selected_checkpoint_cid": "blake3:" + "2" * 64,
                "continuation_dataset_manifest_cid": "blake3:" + "3" * 64,
                "continuation_training_view_manifest_cid": "blake3:" + "4" * 64,
                "split_policy_cid": "blake3:" + "5" * 64,
                "weights_cid": "blake3:" + "6" * 64,
                "tokenizer_cid": "blake3:" + "7" * 64,
            }
            parity = {"manifest_cid": "blake3:" + "8" * 64}
            dataset = {
                "manifest_cid": selection["continuation_dataset_manifest_cid"],
                "split_policy_cid": selection["split_policy_cid"],
            }
            prompts = [
                {
                    "story_cid": f"blake3:{index:064x}",
                    "token_ids": [index + 10] * 24,
                    "text": f"prompt {index}",
                }
                for index in range(5)
            ]

            class FakeModel:
                def to(self, _device: torch.device) -> "FakeModel":
                    return self

            class FakeStore:
                tokens = np.zeros(249_880, dtype=np.uint16)
                scored_next_tokens = 249_856

            class FakeTokenizer:
                def decode(self, token_ids: list[int], *, skip_special_tokens: bool) -> str:
                    del skip_special_tokens
                    return f"prompt {token_ids[0] - 10}"

                def encode(self, text: str, *, add_special_tokens: bool) -> SimpleNamespace:
                    del add_special_tokens
                    index = int(text.rsplit(" ", 1)[1])
                    return SimpleNamespace(ids=[index + 10] * 24)

            evaluate_enabled = mock.Mock(return_value=1.50)
            with (
                mock.patch(
                    "r4_softmax_trainer.continuation._load_frozen_continuation_selection",
                    return_value=selection,
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.load_enabled_parity_admission",
                    return_value=parity,
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.trainer_implementation_contract",
                    return_value={"tree_cid": "tree"},
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.require_mps",
                    return_value=torch.device("cpu"),
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.R4SoftmaxForCausalLM",
                    return_value=FakeModel(),
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation._load_continuation_checkpoint"
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation._write_reveal_opened_marker",
                    return_value={"result_cid": "blake3:" + "9" * 64},
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.open_sealed_confirmation",
                    return_value={"result_cid": "blake3:" + "a" * 64},
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.load_continuation_dataset_manifest",
                    return_value=dataset,
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.TokenStore", return_value=FakeStore()
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.evaluate", evaluate_enabled
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.Tokenizer.from_file",
                    return_value=FakeTokenizer(),
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation._load_sealed_prompt_fixture",
                    return_value=prompts,
                ),
                mock.patch(
                    "r4_softmax_trainer.continuation.write_bound_manifest",
                    side_effect=lambda _path, payload, **_kwargs: payload,
                ),
            ):
                manifest = reveal_continuation(root)

            evaluate_enabled.assert_called_once()
            self.assertEqual(evaluate_enabled.call_args.kwargs, {})
            self.assertEqual(manifest["terminal"], "FAIL_ENABLED_NLL")
            result = json.loads((root / REVEAL_RESULT_RELATIVE_PATH).read_text())
            self.assertEqual(result["terminal"], "FAIL_ENABLED_NLL")
            self.assertEqual(result["attention_off_executions"], 0)
            self.assertTrue(result["quality_failure_is_frozen_not_retriable"])


if __name__ == "__main__":
    unittest.main()
