from __future__ import annotations

import math
import stat
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest import mock

from blake3 import blake3
from r4_softmax_trainer import prompt_conditioning_v4 as subject
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


def _exclusions(*required: str) -> tuple[str, ...]:
    values = set(required)
    index = 1
    while len(values) < subject.REQUIRED_EXCLUDED_STORY_CIDS:
        values.add(f"blake3:{index:064x}")
        index += 1
    return tuple(sorted(values))


@contextmanager
def _bound_exclusions(values: tuple[str, ...]):
    witness = cid_bytes(canonical_json_bytes(list(values)))
    with mock.patch.object(subject, "REQUIRED_EXCLUDED_STORY_CIDS_CID", witness):
        yield


def _population(
    excluded_story_cids: tuple[str, ...],
) -> subject.PromptConditioningPopulation:
    pairs: list[subject.PromptConditioningPair] = []
    first = subject.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL + 1
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
        excluded_story_cids=excluded_story_cids,
    )


def _prior_manifest(
    schema: str,
    story_cids: tuple[str, ...],
) -> dict[str, object]:
    pairs = []
    for pair_index in range(subject.PAIR_COUNT):
        offset = pair_index * 2
        pairs.append(
            {
                "pair_index": pair_index,
                "left": {"story_cid": story_cids[offset]},
                "right": {"story_cid": story_cids[offset + 1]},
            }
        )
    return {
        "schema": schema,
        "population": {
            "pairs": subject.PAIR_COUNT,
            "directions": subject.DIRECTION_COUNT,
        },
        "pairs": pairs,
    }


def _score(
    gains: tuple[float, ...],
    *,
    own_nll: float,
    attention_off: bool = False,
    maximum_delta: float = 1.0,
    forbidden_reads: int = 0,
) -> subject.PromptConditioningScore:
    mean = math.fsum(gains) / subject.DIRECTION_COUNT
    return subject.PromptConditioningScore(
        attention_off=attention_off,
        directions=subject.DIRECTION_COUNT,
        scored_target_tokens=subject.SCORED_TARGET_TOKENS,
        mean_gain_nats_per_token=mean,
        wins=sum(gain > 0.0 for gain in gains),
        own_nll_nats_per_token=own_nll,
        crossed_nll_nats_per_token=own_nll + mean,
        maximum_paired_logits_delta=maximum_delta,
        forbidden_reads=forbidden_reads,
        scored_logprob_trace_cid=f"blake3:{'a' * 64}",
        direction_gains_nats_per_token=gains,
    )


class PromptConditioningV4Tests(unittest.TestCase):
    def test_contract_has_an_isolated_v4_identity_and_unchanged_law(self) -> None:
        self.assertEqual(subject.POLICY, "R4RetainedPromptSwapContrastV4")
        self.assertEqual(
            subject.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL,
            324_230,
        )
        self.assertEqual(
            subject.REQUIRED_PRIOR_POPULATION_CIDS,
            (
                subject.V1_POPULATION_CID,
                subject.V2_POPULATION_CID,
                subject.V3_POPULATION_CID,
            ),
        )
        self.assertEqual(subject.REQUIRED_EXCLUDED_STORY_CIDS, 1_536)
        self.assertEqual(
            subject.REQUIRED_EXCLUDED_STORY_CIDS_CID,
            "blake3:e8d02abcf9ab326545afa80c5191285ec37110cf73f0d389cd6a2f75fcd5c121",
        )
        self.assertTrue(subject.POPULATION_SCHEMA.endswith("/4"))
        self.assertTrue(subject.COMMITMENT_SCHEMA.endswith("/4"))
        self.assertTrue(subject.REVEAL_SCHEMA.endswith("/4"))
        self.assertTrue(subject.SCORE_SCHEMA.endswith("/4"))
        self.assertEqual(
            subject.ASSOCIATIVE_DECISION_SCHEMA,
            "uor-r4.associative-capacity-decision/1",
        )
        self.assertEqual(
            subject.GEOMETRY_DECISION_SCHEMA,
            "uor-r4.geometry-attribution-decision/1",
        )
        self.assertEqual(
            subject.POPULATION_RELATIVE_PATH,
            "evaluation/sealed/prompt-population.json",
        )
        self.assertEqual(
            subject.FRESH_HELDOUT_RELATIVE_PATH,
            "evaluation/sealed/fresh-heldout.u16",
        )
        self.assertEqual(subject.COMMITMENT_RELATIVE_PATH, "evaluation/commitment.json")
        self.assertEqual(subject.REVEAL_RELATIVE_PATH, "evaluation/reveal.json")
        self.assertEqual(subject.PAIR_COUNT, 256)
        self.assertEqual(subject.DIRECTION_COUNT, 512)
        self.assertEqual(subject.SCORED_TARGET_TOKENS, 8_192)
        self.assertEqual(subject.WIN_THRESHOLD, 308)
        self.assertEqual(
            subject.ABSOLUTE_GAIN_THRESHOLD,
            math.log(2.0) / subject.CONTINUATION_TOKENS,
        )
        self.assertEqual(
            subject.CAPACITY_GAIN_THRESHOLD,
            math.log(1.5) / subject.CONTINUATION_TOKENS,
        )

    def test_selector_starts_strictly_after_v3_and_excludes_union(self) -> None:
        boundary = subject.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
        excluded_story = f"story-{boundary + 1}".encode()
        exclusions = _exclusions(
            f"blake3:{blake3(excluded_story).hexdigest()}",
        )
        indexed = [
            (ordinal, f"story-{ordinal}".encode())
            for ordinal in range(
                boundary,
                boundary + subject.DIRECTION_COUNT + 2,
            )
        ]
        with (
            _bound_exclusions(exclusions),
            mock.patch.object(subject, "story_split", return_value="dev"),
        ):
            population = subject.select_prompt_conditioning_population(
                indexed,
                _SyntheticTokenizer(),
                excluded_story_cids=exclusions,
            )
            replay = subject.PromptConditioningPopulation.from_manifest(
                population.manifest()
            )

        self.assertEqual(
            population.pairs[0].left.source_story_ordinal,
            boundary + 2,
        )
        self.assertEqual(
            population.last_source_story_ordinal,
            boundary + subject.DIRECTION_COUNT + 1,
        )
        self.assertEqual(population, replay)
        self.assertEqual(
            population.manifest()["prior_population_exclusions"]["story_cid_count"],
            1_536,
        )

    def test_prior_loader_verifies_all_three_populations_and_exact_union(self) -> None:
        exclusions = _exclusions()
        chunks = tuple(
            exclusions[start : start + subject.DIRECTION_COUNT]
            for start in range(0, len(exclusions), subject.DIRECTION_COUNT)
        )
        schemas = tuple(
            f"uor-r4.retained-prompt-swap-population/{version}"
            for version in (1, 2, 3)
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths = tuple(root / f"v{version}.json" for version in (1, 2, 3))
            payloads = tuple(
                canonical_json_bytes(_prior_manifest(schema, story_cids))
                for schema, story_cids in zip(schemas, chunks, strict=True)
            )
            for path, payload in zip(paths, payloads, strict=True):
                path.write_bytes(payload)
            with (
                _bound_exclusions(exclusions),
                mock.patch.object(subject, "V1_POPULATION_CID", cid_bytes(payloads[0])),
                mock.patch.object(subject, "V2_POPULATION_CID", cid_bytes(payloads[1])),
                mock.patch.object(subject, "V3_POPULATION_CID", cid_bytes(payloads[2])),
            ):
                observed = subject.load_required_prior_story_cids(*paths)
                self.assertEqual(observed, frozenset(exclusions))
                with self.assertRaisesRegex(
                    ValueError,
                    "differs from its exact freeze",
                ):
                    subject.load_required_prior_story_cids(
                        paths[1],
                        paths[0],
                        paths[2],
                    )

    def test_staged_seal_and_reveal_bind_all_three_artifacts(self) -> None:
        exclusions = _exclusions()
        heldout_payload = b"heldout"
        heldout_cid = cid_bytes(heldout_payload)
        with _bound_exclusions(exclusions):
            population = _population(exclusions)
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                staged = subject.stage_prompt_conditioning_population(
                    root,
                    population,
                )
                sealed = root / subject.SEALED_DIRECTORY_RELATIVE_PATH
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0o700)
                heldout_path = root / subject.FRESH_HELDOUT_RELATIVE_PATH
                with self.assertRaisesRegex(ValueError, "companion differs"):
                    subject.seal_prompt_conditioning_population(
                        root,
                        population,
                        heldout_relative_path=subject.FRESH_HELDOUT_RELATIVE_PATH,
                        heldout_bytes=len(heldout_payload),
                        heldout_cid=heldout_cid,
                    )
                heldout_path.write_bytes(b"wrong!!")
                with self.assertRaisesRegex(ValueError, "companion differs"):
                    subject.seal_prompt_conditioning_population(
                        root,
                        population,
                        heldout_relative_path=subject.FRESH_HELDOUT_RELATIVE_PATH,
                        heldout_bytes=len(heldout_payload),
                        heldout_cid=heldout_cid,
                    )
                heldout_path.write_bytes(heldout_payload)
                commitment = subject.seal_prompt_conditioning_population(
                    root,
                    population,
                    heldout_relative_path=subject.FRESH_HELDOUT_RELATIVE_PATH,
                    heldout_bytes=len(heldout_payload),
                    heldout_cid=heldout_cid,
                )
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0)
                self.assertEqual(staged, commitment["population"])
                self.assertEqual(
                    commitment["fresh_heldout"],
                    {
                        "path": subject.FRESH_HELDOUT_RELATIVE_PATH,
                        "bytes": len(heldout_payload),
                        "cid": heldout_cid,
                    },
                )
                self.assertEqual(
                    subject.load_prompt_conditioning_commitment(root),
                    commitment,
                )
                revealed = subject.reveal_prompt_conditioning_population(
                    root,
                    baseline_artifact_cid=f"blake3:{'a' * 64}",
                    geometric_artifact_cid=f"blake3:{'b' * 64}",
                    pooled_artifact_cid=f"blake3:{'c' * 64}",
                )
                self.assertEqual(revealed, population)
                sealed.chmod(0)
                recovered = subject.reveal_prompt_conditioning_population(
                    root,
                    baseline_artifact_cid=f"blake3:{'a' * 64}",
                    geometric_artifact_cid=f"blake3:{'b' * 64}",
                    pooled_artifact_cid=f"blake3:{'c' * 64}",
                )
                self.assertEqual(recovered, population)
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0o700)
                with self.assertRaisesRegex(ValueError, "binding differs"):
                    subject.reveal_prompt_conditioning_population(
                        root,
                        baseline_artifact_cid=f"blake3:{'a' * 64}",
                        geometric_artifact_cid=f"blake3:{'b' * 64}",
                        pooled_artifact_cid=f"blake3:{'d' * 64}",
                    )
                heldout_path.write_bytes(b"wrong!!")
                with self.assertRaisesRegex(ValueError, "companion differs"):
                    subject.load_revealed_prompt_conditioning_population(root)

    def test_associative_capacity_is_decided_per_arm(self) -> None:
        zeros = (0.0,) * subject.DIRECTION_COUNT
        v1 = _score(zeros, own_nll=4.0)
        learned = _score((0.08,) * subject.DIRECTION_COUNT, own_nll=3.0)
        state_off = _score(
            zeros,
            own_nll=4.0,
            attention_off=True,
            maximum_delta=0.0,
        )

        decision = subject.associative_capacity_decision(learned, v1, state_off)

        self.assertEqual(decision.verdict, subject.VERDICT_PASS)
        self.assertTrue(decision.gates["score_absolute_gain"])
        self.assertTrue(decision.gates["score_capacity_gain_over_v1"])
        self.assertTrue(decision.gates["score_directional_wins"])
        self.assertTrue(decision.gates["score_own_nll_nonregression"])
        self.assertEqual(
            decision.record()["schema"],
            subject.ASSOCIATIVE_DECISION_SCHEMA,
        )
        invalid = subject.associative_capacity_decision(learned, v1, v1)
        self.assertEqual(invalid.verdict, subject.VERDICT_INVALID)

    def test_geometry_attribution_requires_both_paired_controls(self) -> None:
        pooled = _score((0.0,) * subject.DIRECTION_COUNT, own_nll=4.0)
        deranged = _score((0.01,) * subject.DIRECTION_COUNT, own_nll=4.0)
        geometric = _score((0.08,) * subject.DIRECTION_COUNT, own_nll=3.0)

        decision = subject.geometry_attribution_decision(
            geometric,
            pooled,
            deranged,
        )

        self.assertEqual(decision.verdict, subject.GEOMETRY_ATTRIBUTION_PASS)
        self.assertEqual(
            decision.geometric_over_pooled_directional_improvements,
            subject.DIRECTION_COUNT,
        )
        self.assertEqual(
            decision.geometric_over_deranged_directional_improvements,
            subject.DIRECTION_COUNT,
        )
        self.assertEqual(
            decision.record()["schema"],
            subject.GEOMETRY_DECISION_SCHEMA,
        )

        only_307_improvements = (0.4,) * 307 + (-0.1,) * (
            subject.DIRECTION_COUNT - 307
        )
        unpaired = subject.geometry_attribution_decision(
            _score(only_307_improvements, own_nll=3.0),
            pooled,
            pooled,
        )
        self.assertGreaterEqual(
            unpaired.geometric_minus_pooled_gain_nats_per_token,
            subject.CAPACITY_GAIN_THRESHOLD,
        )
        self.assertEqual(
            unpaired.geometric_over_pooled_directional_improvements,
            307,
        )
        self.assertEqual(unpaired.verdict, subject.GEOMETRY_ATTRIBUTION_FAIL)


if __name__ == "__main__":
    unittest.main()
