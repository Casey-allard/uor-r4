"""Focused decision and execution-contract tests for the #973 V5 terminal."""

from __future__ import annotations

import unittest
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import torch
from blake3 import blake3

from r4_softmax_trainer import predictive_block_delta_terminal_campaign as campaign
from r4_softmax_trainer import prompt_conditioning_v5


def _score(
    mode: str,
    *,
    gain: float,
    own_nll: float,
    positive_directions: int = campaign.DIRECTION_COUNT,
) -> campaign.TerminalPromptScore:
    gains = tuple(
        gain if index < positive_directions else -0.001
        for index in range(campaign.DIRECTION_COUNT)
    )
    mean = sum(gains) / len(gains)
    return campaign.TerminalPromptScore(
        mode=mode,
        directions=campaign.DIRECTION_COUNT,
        targets=campaign.SCORED_TARGET_TOKENS,
        mean_gain_nats_per_token=mean,
        wins=positive_directions,
        own_nll_nats_per_token=own_nll,
        foreign_nll_nats_per_token=own_nll + mean,
        forbidden_reads=0,
        work_signature=(1, 2, 3),
        trace_cid="blake3:" + "1" * 64,
        suffix_logits_trace_cid="blake3:" + "2" * 64,
        head_logits_trace_cid="blake3:" + "3" * 64,
        direction_gains_nats_per_token=gains,
    )


def _fresh(ce: float, top1: float) -> dict:
    return {
        "ce_nats": ce,
        "top1_rate": top1,
        "forbidden_reads": 0,
    }


def _plan_record(plan: campaign.ExecutionPlan, seconds: float) -> dict:
    arms = {}
    for arm in campaign.ARMS:
        arms[arm] = {
            "ok": True,
            "result": {
                "probe_vector": [1.0, 2.0],
                "mean_train_step_seconds": seconds,
                "artifact_export_seconds": 0.01,
                "evaluation_batch_seconds": 0.001,
                "peak_memory_bytes": 100,
                "memory_budget_bytes": 10_000,
                "initial_binding_cid": "blake3:" + "4" * 64,
                "mechanics": {"passed": True},
                "sealed_prompt_reads": 0,
                "sealed_heldout_reads": 0,
            },
        }
    return {"plan": plan.identity(), "arms": arms}


class _SyntheticLanguageAudit:
    forbidden_reads = 0

    def __init__(self, batch_size: int, *, drift: int = 0) -> None:
        self.batch_size = batch_size
        self.drift = drift

    def work_signature(self) -> tuple[int, ...]:
        return (
            self.batch_size,
            self.batch_size * campaign.CONTEXT,
            2,
            4,
            120,
            self.batch_size * 7 + self.drift,
        )


class _SyntheticLanguageModel:
    def __init__(self, *, drift_on_tail: bool = False) -> None:
        self.drift_on_tail = drift_on_tail

    def eval(self) -> _SyntheticLanguageModel:
        return self

    def __call__(self, token_ids: torch.Tensor, *, intervention: str) -> object:
        batch_size, time = token_ids.shape
        drift = int(self.drift_on_tail and batch_size == 1)
        return SimpleNamespace(
            logits=torch.zeros((batch_size, time, 8), dtype=torch.float32),
            audit=_SyntheticLanguageAudit(batch_size, drift=drift),
        )


class PredictiveBlockDeltaTerminalTests(unittest.TestCase):
    def test_original_unavailable_cache_remains_readable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = campaign._write_unavailable(
                root,
                reason={"ok": False, "error": {"type": "ValueError", "reason": "first"}},
                phase="SCORING",
            )
            cached = campaign._write_unavailable(
                root,
                reason="must not replace the frozen failure",
                phase="FIT",
            )
        self.assertEqual(cached, first)

    def test_cached_scoring_recovery_failure_is_provenance_checked(self) -> None:
        recovery = {
            "recovery_cid": "blake3:" + "1" * 64,
            "scoring_implementation": {"tree_cid": "blake3:" + "2" * 64},
        }
        reason = {
            "ok": False,
            "error": {"type": "ValueError", "reason": "scoring failed"},
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = campaign._write_scoring_recovery_unavailable(
                root,
                recovery=recovery,
                reason=reason,
            )
            self.assertEqual(
                campaign._write_scoring_recovery_unavailable(
                    root,
                    recovery=recovery,
                    reason=reason,
                ),
                first,
            )
            different_recovery = {
                **recovery,
                "recovery_cid": "blake3:" + "3" * 64,
            }
            with self.assertRaisesRegex(
                ValueError,
                "cached V5 scoring-recovery unavailable differs",
            ):
                campaign._write_scoring_recovery_unavailable(
                    root,
                    recovery=different_recovery,
                    reason=reason,
                )

    def test_historical_implementation_requires_the_exact_frozen_envelope(
        self,
    ) -> None:
        files = [
            {
                "bytes": 1,
                "cid": "blake3:" + "1" * 64,
                "path": "src/example.py",
            }
        ]
        historical = {"files": files, "tree_cid": campaign.tree_cid(files)}
        current = {
            "files": [
                {
                    "bytes": 1,
                    "cid": "blake3:" + "2" * 64,
                    "path": "src/current.py",
                }
            ]
        }
        current["tree_cid"] = campaign.tree_cid(current["files"])
        with (
            patch.object(
                campaign,
                "FROZEN_V5_FIT_IMPLEMENTATION_TREE_CID",
                historical["tree_cid"],
            ),
            patch.object(
                campaign,
                "trainer_implementation_contract",
                return_value=current,
            ),
        ):
            self.assertEqual(
                campaign._validate_bound_implementation(
                    historical,
                    envelope_cid="frozen-envelope",
                    frozen_envelope_cid="frozen-envelope",
                ),
                historical,
            )
            with self.assertRaisesRegex(ValueError, "neither current nor frozen"):
                campaign._validate_bound_implementation(
                    historical,
                    envelope_cid="different-envelope",
                    frozen_envelope_cid="frozen-envelope",
                )
            tampered = {"files": [{**files[0], "bytes": 2}], "tree_cid": historical["tree_cid"]}
            with self.assertRaisesRegex(ValueError, "does not reproduce"):
                campaign._validate_bound_implementation(
                    tampered,
                    envelope_cid="frozen-envelope",
                    frozen_envelope_cid="frozen-envelope",
                )

    def test_completed_fit_batch_does_not_rewrite_the_wall_ledger(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for arm in campaign.ARMS:
                path = root / "arms" / arm / "result.json"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("complete", encoding="utf-8")
            with (
                patch.object(campaign, "_load_fit_budget", return_value=123.5),
                patch.object(
                    campaign,
                    "_load_arm_result",
                    side_effect=lambda _root, arm: {"arm": arm},
                ),
                patch.object(
                    campaign,
                    "_write_fit_budget",
                    side_effect=AssertionError("fit ledger rewritten"),
                ),
            ):
                result = campaign._run_fit_batch(
                    root,
                    campaign.ELIGIBLE_PLANS[2],
                    campaign.ARMS,
                    resume=True,
                    wall_seconds=campaign.HARD_WALL_SECONDS,
                )
        self.assertTrue(result["ok"])
        self.assertTrue(result["cached"])
        self.assertEqual(result["aggregate_wall_seconds"], 123.5)

    def test_language_scoring_aggregates_a_short_tail_batch(self) -> None:
        windows = SimpleNamespace(
            windows=[[0] * (campaign.CONTEXT + 1) for _ in range(5)]
        )
        with (
            patch.object(campaign, "FRESH_HELDOUT_WINDOWS", 5),
            patch.object(
                campaign,
                "FRESH_HELDOUT_DECISIONS",
                5 * campaign.CONTEXT,
            ),
            patch.object(campaign, "BATCH_SIZE", 4),
            patch.object(campaign, "VOCAB_SIZE", 8),
        ):
            result = campaign._evaluate_language(
                _SyntheticLanguageModel(),
                windows,
                mode="geometric",
                device=torch.device("cpu"),
            )

        self.assertEqual(result["rows"], 5 * campaign.CONTEXT)
        self.assertEqual(
            result["work_signature"],
            [5, 5 * campaign.CONTEXT, 2, 4, 120, 35],
        )

    def test_language_scoring_rejects_real_tail_work_drift(self) -> None:
        windows = SimpleNamespace(
            windows=[[0] * (campaign.CONTEXT + 1) for _ in range(5)]
        )
        with (
            patch.object(campaign, "FRESH_HELDOUT_WINDOWS", 5),
            patch.object(
                campaign,
                "FRESH_HELDOUT_DECISIONS",
                5 * campaign.CONTEXT,
            ),
            patch.object(campaign, "BATCH_SIZE", 4),
            patch.object(campaign, "VOCAB_SIZE", 8),
            self.assertRaisesRegex(
                ValueError,
                "geometric V5 language work changed between batches",
            ),
        ):
            campaign._evaluate_language(
                _SyntheticLanguageModel(drift_on_tail=True),
                windows,
                mode="geometric",
                device=torch.device("cpu"),
            )

    def test_frozen_v5_boundaries_and_prior_union(self) -> None:
        self.assertEqual(
            prompt_conditioning_v5.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL,
            409_546,
        )
        self.assertEqual(prompt_conditioning_v5.REQUIRED_EXCLUDED_STORY_CIDS, 2_048)
        self.assertEqual(
            prompt_conditioning_v5.REQUIRED_EXCLUDED_STORY_CIDS_CID,
            "blake3:c926c19deaae20a17b05fc3c5eddc099324d9b531bbfd83ac992a5ef02ede092",
        )
        self.assertEqual(campaign.FRESH_HELDOUT_SOURCE_OFFSET_TOKENS, 156_282_226)
        self.assertEqual(campaign.FRESH_HELDOUT_TOKENS, 249_986)
        self.assertEqual(campaign.FRESH_HELDOUT_STORY_CIDS, 1_242)
        self.assertEqual(campaign.OPTIMIZER_STEPS, 2_730)
        self.assertEqual(campaign.ARMS, ("geometric", "plain", "additive"))

    def test_execution_selector_uses_fastest_eligible_cpu_plan(self) -> None:
        records = [
            _plan_record(campaign.ELIGIBLE_PLANS[0], 0.12),
            _plan_record(campaign.ELIGIBLE_PLANS[1], 0.08),
            _plan_record(campaign.ELIGIBLE_PLANS[2], 0.05),
        ]
        selection = campaign.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertEqual(
            selection["selected_plan"]["name"],
            "cpu-accelerate-2x4t-concurrent",
        )
        self.assertTrue(
            selection["selected_projection"]["byte_identical_initialization"]
        )

    def test_execution_selector_serializes_one_failed_plan_without_nonfinite_values(
        self,
    ) -> None:
        records = [
            _plan_record(campaign.ELIGIBLE_PLANS[0], 0.12),
            _plan_record(campaign.ELIGIBLE_PLANS[1], 0.08),
            _plan_record(campaign.ELIGIBLE_PLANS[2], 0.05),
        ]
        records[1]["arms"]["plain"] = {
            "ok": False,
            "error": {"type": "RuntimeError", "reason": "test-only failure"},
        }
        selection = campaign.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertIsNone(
            selection["plans"][1]["projection"]["projected_training_seconds"]
        )
        campaign.canonical_json_bytes(selection)

    def test_wrong_source_store_fails_before_slice_read(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "train.u16"
            path.write_bytes(b"\x00\x00")
            with self.assertRaisesRegex(ValueError, "differs from #1019"):
                campaign._read_u16_slice(path, offset_tokens=0, token_count=1)

    def test_v5_selector_boundary_pairing_and_canonical_round_trip(self) -> None:
        boundary = prompt_conditioning_v5.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
        excluded_story = b"excluded story"
        exclusions = tuple(
            sorted(
                (
                    f"blake3:{blake3(excluded_story).hexdigest()}",
                    "blake3:" + "f" * 64,
                )
            )
        )
        exclusion_cid = prompt_conditioning_v5.cid_bytes(
            prompt_conditioning_v5.canonical_json_bytes(list(exclusions))
        )
        common_tail = [31, 32, 33, 34]
        tokens = {
            "left story": [100] * 44 + common_tail + [200] * 16,
            "right story": [101] * 44 + common_tail + [201] * 16,
        }
        tokenizer = Mock()
        tokenizer.encode.side_effect = lambda text, add_special_tokens=False: Mock(
            ids=tokens[text]
        )
        indexed = (
            (boundary, b"ignored boundary"),
            (boundary + 1, excluded_story),
            (boundary + 2, b"left story"),
            (boundary + 3, b"right story"),
        )
        with (
            patch.object(prompt_conditioning_v5, "REQUIRED_EXCLUDED_STORY_CIDS", 2),
            patch.object(
                prompt_conditioning_v5,
                "REQUIRED_EXCLUDED_STORY_CIDS_CID",
                exclusion_cid,
            ),
            patch.object(prompt_conditioning_v5, "PAIR_COUNT", 1),
            patch.object(prompt_conditioning_v5, "DIRECTION_COUNT", 2),
            patch.object(prompt_conditioning_v5, "SCORED_TARGET_TOKENS", 32),
            patch.object(prompt_conditioning_v5, "story_split", return_value="dev"),
        ):
            population = prompt_conditioning_v5.select_prompt_conditioning_population(
                indexed, tokenizer, excluded_story_cids=exclusions
            )
            self.assertEqual(population.last_source_story_ordinal, boundary + 3)
            self.assertEqual(
                population.pairs[0].left.story_cid,
                f"blake3:{blake3(b'left story').hexdigest()}",
            )
            self.assertEqual(
                prompt_conditioning_v5.PromptConditioningPopulationV5.from_manifest(
                    population.manifest()
                ).manifest(),
                population.manifest(),
            )
            with self.assertRaises(ValueError):
                prompt_conditioning_v5.select_prompt_conditioning_population(
                    indexed, tokenizer, excluded_story_cids=exclusions[:1]
                )
            with self.assertRaisesRegex(ValueError, "strictly increasing"):
                prompt_conditioning_v5.select_prompt_conditioning_population(
                    tuple(reversed(indexed)), tokenizer, excluded_story_cids=exclusions
                )

    def test_v2_authorization_fails_before_any_v5_source_access(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "new-v5-root"
            blocked = RuntimeError("authorization blocked")
            predecessor = Mock(side_effect=AssertionError("predecessor opened"))
            heldout = Mock(side_effect=AssertionError("V5 heldout opened"))
            selector = Mock(side_effect=AssertionError("V5 prompt source opened"))
            with (
                patch.object(campaign, "_verify_v2_authorization", side_effect=blocked),
                patch.object(campaign, "_verify_predecessor", predecessor),
                patch.object(campaign, "_read_u16_slice", heldout),
                patch.object(
                    campaign,
                    "select_prompt_conditioning_population_from_source",
                    selector,
                ),
                self.assertRaisesRegex(RuntimeError, "authorization blocked"),
            ):
                campaign.prepare_predictive_block_delta_terminal(
                    root=root,
                    predecessor_root=Path("/not-opened/predecessor"),
                    source_train_path=Path("/not-opened/train.u16"),
                    source_train_index_path=Path("/not-opened/train.jsonl"),
                    raw_source_path=Path("/not-opened/source.txt"),
                    prior_population_paths=(),
                    frame_sidecar_path=Path("/not-opened/frames.json"),
                    v2_result_path=Path("/not-opened/v2.json"),
                    pooled_comparator_root=Path("/not-opened/pooled"),
                )
            predecessor.assert_not_called()
            heldout.assert_not_called()
            selector.assert_not_called()

    def test_terminal_decision_separates_capacity_geometry_and_delta(self) -> None:
        scores = {
            "geometric": _score("geometric", gain=0.10, own_nll=2.0),
            "v1": _score("v1", gain=0.0, own_nll=2.5, positive_directions=0),
            "pooled": _score("pooled", gain=0.01, own_nll=2.4),
            "plain": _score("plain", gain=0.0, own_nll=2.2, positive_directions=0),
            "transport_permuted": _score(
                "transport_permuted", gain=0.0, own_nll=2.2, positive_directions=0
            ),
            "additive": _score(
                "additive", gain=0.0, own_nll=2.1, positive_directions=0
            ),
            "state_off": _score(
                "state_off", gain=0.0, own_nll=2.5, positive_directions=0
            ),
        }
        fresh = {
            "geometric": _fresh(2.0, 0.30),
            "v1": _fresh(2.02, 0.30),
            "pooled": _fresh(2.01, 0.30),
        }
        decision = campaign.terminal_decision(
            scores=scores, fresh=fresh, mechanics={"passed": True}
        )
        self.assertTrue(decision["capacity_positive"])
        self.assertTrue(decision["geometry_positive"])
        self.assertTrue(decision["delta_overwrite_positive"])
        self.assertEqual(
            decision["verdict"],
            "PREDICTIVE_GEOMETRIC_CAPACITY_AND_ATTRIBUTION_PASS",
        )
        self.assertEqual(
            campaign._terminal_next_action(decision, {"passed": True}),
            "FREEZE_ONE_BOUNDED_AUTONOMOUS_GENERATION_RUNG",
        )
        invalid = campaign.terminal_decision(
            scores=scores, fresh=fresh, mechanics={"passed": False}
        )
        self.assertEqual(invalid["verdict"], "INVALID_PREDICTIVE_V5_TERMINAL")
        self.assertFalse(invalid["capacity_positive"])
        self.assertFalse(invalid["geometry_positive"])
        self.assertFalse(invalid["delta_attribution"]["claimed"])
        self.assertEqual(
            campaign._terminal_next_action(invalid, {"passed": False}),
            "STOP_WITHOUT_GENERATION",
        )

    def test_unstable_additive_cannot_create_trivial_delta_attribution(self) -> None:
        scores = {
            "geometric": _score("geometric", gain=0.10, own_nll=2.0),
            "v1": _score("v1", gain=0.0, own_nll=2.5, positive_directions=0),
            "pooled": _score("pooled", gain=0.01, own_nll=2.4),
            "plain": _score("plain", gain=0.0, own_nll=2.2, positive_directions=0),
            "transport_permuted": _score(
                "transport_permuted", gain=0.0, own_nll=2.2, positive_directions=0
            ),
            "additive": _score(
                "additive", gain=-1.0, own_nll=2.56, positive_directions=0
            ),
            "state_off": _score(
                "state_off", gain=0.0, own_nll=2.5, positive_directions=0
            ),
        }
        decision = campaign.terminal_decision(
            scores=scores,
            fresh={
                "geometric": _fresh(2.0, 0.30),
                "v1": _fresh(2.02, 0.30),
                "pooled": _fresh(2.01, 0.30),
            },
            mechanics={"passed": True},
        )
        self.assertFalse(decision["delta_overwrite_positive"])
        self.assertFalse(decision["delta_attribution"]["claimed"])
        self.assertEqual(
            decision["delta_attribution"]["verdict"],
            "ADDITIVE_CONTROL_NO_STABLE_CAPACITY",
        )

    def test_state_own_nll_nonregression_is_a_capacity_gate(self) -> None:
        scores = {
            "geometric": _score("geometric", gain=0.10, own_nll=2.6),
            "v1": _score("v1", gain=0.0, own_nll=2.5, positive_directions=0),
            "pooled": _score("pooled", gain=0.0, own_nll=2.7, positive_directions=0),
            "plain": _score("plain", gain=0.0, own_nll=2.7, positive_directions=0),
            "transport_permuted": _score(
                "transport_permuted", gain=0.0, own_nll=2.7, positive_directions=0
            ),
            "additive": _score("additive", gain=0.0, own_nll=2.7),
            "state_off": _score(
                "state_off", gain=0.0, own_nll=2.5, positive_directions=0
            ),
        }
        decision = campaign.terminal_decision(
            scores=scores,
            fresh={
                "geometric": _fresh(2.0, 0.30),
                "v1": _fresh(2.0, 0.30),
                "pooled": _fresh(2.0, 0.30),
            },
            mechanics={"passed": True},
        )
        self.assertFalse(decision["capacity"]["gates"]["state_own_nll_nonregression"])
        self.assertFalse(decision["capacity_positive"])


if __name__ == "__main__":
    unittest.main()
