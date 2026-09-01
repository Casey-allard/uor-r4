from __future__ import annotations

import stat
import tempfile
import unittest
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch
from blake3 import blake3

from r4_softmax_trainer import prompt_conditioning as subject
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


class _Encoding:
    def __init__(self, ids: list[int]) -> None:
        self.ids = ids


class _SyntheticTokenizer:
    def encode(self, sequence: str, add_special_tokens: bool = True) -> _Encoding:
        if add_special_tokens:
            raise AssertionError("selector inserted tokenizer special tokens")
        ordinal = int(sequence.removeprefix("story-"))
        side = ordinal % 2
        prompt = [10 + side, *([12 + side] * 43), 30, 31, 32, 33]
        continuation = [20 + side] * subject.CONTINUATION_TOKENS
        return _Encoding([*prompt, *continuation])


def _story_cid(label: str) -> str:
    return f"blake3:{blake3(label.encode('utf-8')).hexdigest()}"


EVALUATION_IDENTITY = {
    "reveal_cid": f"blake3:{'c' * 64}",
    "baseline_artifact_cid": f"blake3:{'a' * 64}",
    "candidate_artifact_cid": f"blake3:{'b' * 64}",
}


def _population() -> subject.PromptConditioningPopulation:
    pairs: list[subject.PromptConditioningPair] = []
    first = subject.PRIOR_DEVELOPMENT_LAST_SOURCE_STORY_ORDINAL + 1
    for pair_index in range(subject.PAIR_COUNT):
        ordinal = first + pair_index * 2
        left = subject.PromptConditioningRecord(
            source_story_ordinal=ordinal,
            story_cid=_story_cid(f"left-{pair_index}"),
            prompt_token_ids=(10, *([12] * 43), 30, 31, 32, 33),
            continuation_token_ids=(20,) * subject.CONTINUATION_TOKENS,
        )
        right = subject.PromptConditioningRecord(
            source_story_ordinal=ordinal + 1,
            story_cid=_story_cid(f"right-{pair_index}"),
            prompt_token_ids=(11, *([13] * 43), 30, 31, 32, 33),
            continuation_token_ids=(21,) * subject.CONTINUATION_TOKENS,
        )
        pairs.append(
            subject.PromptConditioningPair(
                pair_index=pair_index,
                left=left,
                right=right,
            )
        )
    return subject.PromptConditioningPopulation(
        pairs=tuple(pairs),
        last_source_story_ordinal=first + subject.DIRECTION_COUNT - 1,
        eligible_stories_examined=subject.DIRECTION_COUNT,
    )


@dataclass(frozen=True)
class _Audit:
    forbidden_reads: int = 0
    state_off: bool = False


class _ConditionalModel:
    def __init__(
        self,
        strength: float,
        *,
        state_off_strength: float = 0.0,
        forbidden_reads: int = 0,
    ) -> None:
        self.strength = strength
        self.state_off_strength = state_off_strength
        self.forbidden_reads = forbidden_reads
        self.eval_calls = 0

    def eval(self) -> _ConditionalModel:
        self.eval_calls += 1
        return self

    def __call__(
        self,
        token_ids: torch.Tensor,
        *,
        attention_off: bool,
    ) -> SimpleNamespace:
        batch, time = token_ids.shape
        logits = torch.zeros((batch, time, 64), dtype=torch.float32)
        strength = self.state_off_strength if attention_off else self.strength
        for row, prompt_marker in enumerate(token_ids[:, 1].tolist()):
            preferred = 20 if prompt_marker == 10 else 21
            logits[row, :, preferred] = strength
        return SimpleNamespace(
            logits=logits,
            audit=_Audit(
                forbidden_reads=self.forbidden_reads,
                state_off=attention_off,
            ),
        )


class _PositionBoundConditionalModel(_ConditionalModel):
    def __call__(
        self,
        token_ids: torch.Tensor,
        *,
        attention_off: bool,
    ) -> SimpleNamespace:
        batch, time = token_ids.shape
        logits = torch.zeros((batch, time, 64), dtype=torch.float32)
        strength = self.state_off_strength if attention_off else self.strength
        for row, prompt_marker in enumerate(token_ids[:, 1].tolist()):
            preferred = 20 if prompt_marker == 10 else 21
            logits[
                row,
                subject.PROMPT_TOKENS : subject.PROMPT_TOKENS
                + subject.CONTINUATION_TOKENS,
                preferred,
            ] = strength
        return SimpleNamespace(
            logits=logits,
            audit=_Audit(
                forbidden_reads=self.forbidden_reads,
                state_off=attention_off,
            ),
        )


class PopulationSelectionTests(unittest.TestCase):
    def test_selector_is_exact_fresh_development_and_canonical(self) -> None:
        boundary = subject.PRIOR_DEVELOPMENT_LAST_SOURCE_STORY_ORDINAL
        indexed = [
            (ordinal, f"story-{ordinal}".encode())
            for ordinal in range(boundary, boundary + subject.DIRECTION_COUNT + 1)
        ]
        with mock.patch.object(subject, "story_split", return_value="dev"):
            population = subject.select_prompt_conditioning_population(
                indexed,
                _SyntheticTokenizer(),
            )
            replay = subject.select_prompt_conditioning_population(
                indexed,
                _SyntheticTokenizer(),
            )

        self.assertEqual(len(population.pairs), 256)
        self.assertEqual(len(subject.prompt_directions(population)), 512)
        self.assertEqual(population.eligible_stories_examined, 512)
        self.assertEqual(population.pairs[0].left.source_story_ordinal, boundary + 1)
        self.assertEqual(
            population.last_source_story_ordinal,
            boundary + subject.DIRECTION_COUNT,
        )
        self.assertTrue(
            all(
                pair.left.prompt_tail == pair.right.prompt_tail
                and pair.left.prompt_token_ids != pair.right.prompt_token_ids
                and pair.left.continuation_token_ids
                != pair.right.continuation_token_ids
                for pair in population.pairs
            )
        )
        self.assertEqual(population.population_cid, replay.population_cid)
        self.assertEqual(
            population.population_cid,
            cid_bytes(canonical_json_bytes(population.manifest())),
        )
        self.assertEqual(
            subject.PromptConditioningPopulation.from_manifest(population.manifest()),
            population,
        )

    def test_selector_fails_closed_when_exact_population_is_unavailable(self) -> None:
        boundary = subject.PRIOR_DEVELOPMENT_LAST_SOURCE_STORY_ORDINAL
        indexed = [
            (boundary + 1, f"story-{boundary + 1}".encode()),
            (boundary + 2, f"story-{boundary + 2}".encode()),
        ]
        with (
            mock.patch.object(subject, "story_split", return_value="dev"),
            self.assertRaises(subject.PromptConditioningPopulationUnavailable),
        ):
            subject.select_prompt_conditioning_population(
                indexed,
                _SyntheticTokenizer(),
            )

    def test_source_entrypoint_verifies_source_and_tokenizer_before_selection(self) -> None:
        boundary = subject.PRIOR_DEVELOPMENT_LAST_SOURCE_STORY_ORDINAL
        stories = [
            f"story-{ordinal}".encode()
            for ordinal in range(boundary + subject.DIRECTION_COUNT + 1)
        ]
        with tempfile.TemporaryDirectory() as temporary:
            source_path = Path(temporary) / "source.txt"
            tokenizer_path = Path(temporary) / "tokenizer.json"
            source_path.write_bytes(b"synthetic source")
            tokenizer_path.write_bytes(b"synthetic tokenizer")
            with (
                mock.patch.object(subject, "verify_source") as verify_source,
                mock.patch.object(
                    subject,
                    "cid_file",
                    return_value=subject.TOKENIZER_CID,
                ) as cid_file,
                mock.patch.object(
                    subject,
                    "iter_canonical_stories",
                    return_value=iter(stories),
                ),
                mock.patch(
                    "tokenizers.Tokenizer.from_file",
                    return_value=_SyntheticTokenizer(),
                ) as from_file,
                mock.patch.object(subject, "story_split", return_value="dev"),
            ):
                population = subject.select_prompt_conditioning_population_from_source(
                    source_path,
                    tokenizer_path,
                )
        self.assertEqual(len(population.pairs), subject.PAIR_COUNT)
        verify_source.assert_called_once_with(source_path)
        cid_file.assert_called_once_with(tokenizer_path)
        from_file.assert_called_once_with(str(tokenizer_path))


class PopulationSealingTests(unittest.TestCase):
    def test_population_is_create_once_committed_sealed_and_revealed_once(self) -> None:
        population = _population()
        baseline_cid = f"blake3:{'a' * 64}"
        candidate_cid = f"blake3:{'b' * 64}"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commitment = subject.seal_prompt_conditioning_population(root, population)
            sealed = root / subject.SEALED_DIRECTORY_RELATIVE_PATH
            self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0)
            self.assertEqual(
                commitment["population"]["cid"],
                population.population_cid,
            )
            self.assertEqual(
                subject.load_prompt_conditioning_commitment(root),
                commitment,
            )
            with self.assertRaises(FileExistsError):
                subject.seal_prompt_conditioning_population(root, population)

            revealed = subject.reveal_prompt_conditioning_population(
                root,
                baseline_artifact_cid=baseline_cid,
                candidate_artifact_cid=candidate_cid,
            )
            self.assertEqual(revealed, population)
            self.assertEqual(
                subject.load_revealed_prompt_conditioning_population(root),
                population,
            )
            with self.assertRaises((FileExistsError, ValueError)):
                subject.reveal_prompt_conditioning_population(
                    root,
                    baseline_artifact_cid=baseline_cid,
                    candidate_artifact_cid=candidate_cid,
                )


class PairedScoringTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.population = _population()

    def test_candidate_passes_effect_size_win_replay_and_state_off_gates(self) -> None:
        decision = subject.evaluate_prompt_conditioning(
            population=self.population,
            **EVALUATION_IDENTITY,
            baseline_factory=lambda: _ConditionalModel(0.10),
            candidate_factory=lambda: _ConditionalModel(0.50),
            direction_batch_size=32,
        )

        self.assertEqual(decision.verdict, subject.VERDICT_PASS)
        self.assertEqual(decision.candidate.wins, subject.DIRECTION_COUNT)
        self.assertAlmostEqual(
            decision.candidate.mean_gain_nats_per_token,
            0.50,
            places=7,
        )
        self.assertAlmostEqual(
            decision.baseline.mean_gain_nats_per_token,
            0.10,
            places=7,
        )
        self.assertLessEqual(
            decision.candidate.own_nll_nats_per_token,
            decision.baseline.own_nll_nats_per_token,
        )
        self.assertEqual(decision.baseline_state_off.mean_gain_nats_per_token, 0.0)
        self.assertEqual(decision.candidate_state_off.mean_gain_nats_per_token, 0.0)
        self.assertEqual(decision.candidate_state_off.maximum_paired_logits_delta, 0.0)
        self.assertTrue(all(decision.gates.values()))
        record = decision.record()
        self.assertEqual(
            record["thresholds"]["candidate_mean_gain_nats_per_token"],
            subject.ABSOLUTE_GAIN_THRESHOLD,
        )
        self.assertEqual(record["verdict"], subject.VERDICT_PASS)
        self.assertEqual(record["reveal_cid"], EVALUATION_IDENTITY["reveal_cid"])
        self.assertEqual(
            record["artifacts"],
            {
                "baseline": EVALUATION_IDENTITY["baseline_artifact_cid"],
                "candidate": EVALUATION_IDENTITY["candidate_artifact_cid"],
            },
        )

    def test_scorer_uses_exact_continuation_logit_positions(self) -> None:
        score = subject.score_prompt_conditioning(
            _PositionBoundConditionalModel(0.5),
            self.population,
            attention_off=False,
            direction_batch_size=64,
        )
        self.assertAlmostEqual(score.mean_gain_nats_per_token, 0.5, places=7)

    def test_generic_fluency_cannot_pass_the_paired_intervention(self) -> None:
        decision = subject.evaluate_prompt_conditioning(
            population=self.population,
            **EVALUATION_IDENTITY,
            baseline_factory=lambda: _ConditionalModel(0.0),
            candidate_factory=lambda: _ConditionalModel(0.0),
            direction_batch_size=64,
        )
        self.assertEqual(decision.verdict, subject.VERDICT_FAIL)
        self.assertEqual(decision.candidate.mean_gain_nats_per_token, 0.0)
        self.assertEqual(decision.candidate.wins, 0)
        self.assertFalse(decision.gates["candidate_absolute_gain"])
        self.assertFalse(decision.gates["candidate_capacity_gain"])

    def test_positive_gain_below_absolute_threshold_is_partial_only(self) -> None:
        decision = subject.evaluate_prompt_conditioning(
            population=self.population,
            **EVALUATION_IDENTITY,
            baseline_factory=lambda: _ConditionalModel(0.01),
            candidate_factory=lambda: _ConditionalModel(0.02),
            direction_batch_size=64,
        )
        self.assertEqual(decision.verdict, subject.VERDICT_PARTIAL)
        self.assertTrue(decision.gates["candidate_any_gain"])
        self.assertFalse(decision.gates["candidate_absolute_gain"])
        self.assertFalse(decision.gates["candidate_capacity_gain"])

    def test_model_mode_audit_is_explicit_and_matches_the_requested_mode(self) -> None:
        logits = torch.zeros((1, 64, 64), dtype=torch.float32)
        with self.assertRaisesRegex(ValueError, "omitted its attention-mode audit"):
            subject._model_logits(
                SimpleNamespace(
                    logits=logits,
                    audit=SimpleNamespace(forbidden_reads=0),
                ),
                expected_batch=1,
                attention_off=False,
            )
        with self.assertRaisesRegex(ValueError, "reports the wrong mode"):
            subject._model_logits(
                SimpleNamespace(
                    logits=logits,
                    audit=SimpleNamespace(forbidden_reads=0, state_off=True),
                ),
                expected_batch=1,
                attention_off=False,
            )

    def test_state_off_or_replay_failure_invalidates_without_model_verdict(self) -> None:
        state_leak = subject.evaluate_prompt_conditioning(
            population=self.population,
            **EVALUATION_IDENTITY,
            baseline_factory=lambda: _ConditionalModel(0.10),
            candidate_factory=lambda: _ConditionalModel(
                0.50,
                state_off_strength=0.20,
            ),
            direction_batch_size=64,
        )
        self.assertEqual(state_leak.verdict, subject.VERDICT_INVALID)
        self.assertFalse(state_leak.gates["candidate_state_off_collapsed"])

        strengths = iter((0.50, 0.60, 0.50))
        replay_drift = subject.evaluate_prompt_conditioning(
            population=self.population,
            **EVALUATION_IDENTITY,
            baseline_factory=lambda: _ConditionalModel(0.10),
            candidate_factory=lambda: _ConditionalModel(next(strengths)),
            direction_batch_size=64,
        )
        self.assertEqual(replay_drift.verdict, subject.VERDICT_INVALID)
        self.assertFalse(replay_drift.gates["candidate_replay_exact"])


if __name__ == "__main__":
    unittest.main()
