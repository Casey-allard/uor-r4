from __future__ import annotations

import stat
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest import mock

from blake3 import blake3
from r4_softmax_trainer import prompt_conditioning_v3 as subject
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


class PromptConditioningV3Tests(unittest.TestCase):
    def test_contract_has_an_isolated_v3_identity_and_unchanged_thresholds(
        self,
    ) -> None:
        self.assertEqual(subject.POLICY, "R4RetainedPromptSwapContrastV3")
        self.assertEqual(
            subject.PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL,
            241_074,
        )
        self.assertTrue(subject.POPULATION_SCHEMA.endswith("/3"))
        self.assertTrue(subject.COMMITMENT_SCHEMA.endswith("/3"))
        self.assertTrue(subject.REVEAL_SCHEMA.endswith("/3"))
        self.assertTrue(subject.SCORE_SCHEMA.endswith("/3"))
        self.assertTrue(subject.DECISION_SCHEMA.endswith("/3"))
        self.assertEqual(subject.WORK_RELATIVE_PATH, "prompt-conditioning-v3")
        self.assertEqual(subject.PAIR_COUNT, 256)
        self.assertEqual(subject.DIRECTION_COUNT, 512)
        self.assertEqual(subject.SCORED_TARGET_TOKENS, 8_192)
        self.assertEqual(subject.WIN_THRESHOLD, 308)

    def test_selector_skips_prior_cid_and_binds_exact_exclusion_union(self) -> None:
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
        self.assertEqual(population.eligible_stories_examined, 512)
        self.assertEqual(population, replay)
        self.assertEqual(
            population.manifest()["prior_population_exclusions"]["story_cid_count"],
            1_024,
        )

    def test_wrong_or_reused_exclusion_cids_fail_closed(self) -> None:
        exclusions = _exclusions()
        with self.assertRaisesRegex(ValueError, r"exact V1\+V2 union"):
            subject.select_prompt_conditioning_population(
                (),
                _SyntheticTokenizer(),
                excluded_story_cids=exclusions[:-1],
            )

        reused = _story_cid("left-0")
        exclusions = _exclusions(reused)
        with (
            _bound_exclusions(exclusions),
            self.assertRaisesRegex(ValueError, "reuses a V1 or V2 story CID"),
        ):
            _population(exclusions)

    def test_prior_loader_verifies_both_frozen_populations_and_union(self) -> None:
        exclusions = _exclusions()
        v1_story_cids = exclusions[: subject.DIRECTION_COUNT]
        v2_story_cids = exclusions[subject.DIRECTION_COUNT :]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            v1_path = root / "v1.json"
            v2_path = root / "v2.json"
            v1_payload = canonical_json_bytes(
                _prior_manifest(
                    "uor-r4.retained-prompt-swap-population/1",
                    v1_story_cids,
                )
            )
            v2_payload = canonical_json_bytes(
                _prior_manifest(
                    "uor-r4.retained-prompt-swap-population/2",
                    v2_story_cids,
                )
            )
            v1_path.write_bytes(v1_payload)
            v2_path.write_bytes(v2_payload)
            with (
                _bound_exclusions(exclusions),
                mock.patch.object(subject, "V1_POPULATION_CID", cid_bytes(v1_payload)),
                mock.patch.object(subject, "V2_POPULATION_CID", cid_bytes(v2_payload)),
            ):
                observed = subject.load_required_prior_story_cids(v1_path, v2_path)
                self.assertEqual(observed, frozenset(exclusions))
                with self.assertRaisesRegex(
                    ValueError,
                    "differs from its exact freeze",
                ):
                    subject.load_required_prior_story_cids(v2_path, v1_path)

    def test_seal_and_reveal_marker_recovers_only_the_same_binding(self) -> None:
        exclusions = _exclusions()
        with _bound_exclusions(exclusions):
            population = _population(exclusions)
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                commitment = subject.seal_prompt_conditioning_population(
                    root,
                    population,
                )
                sealed = root / subject.SEALED_DIRECTORY_RELATIVE_PATH
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0)
                self.assertEqual(
                    commitment["population"]["cid"],
                    population.population_cid,
                )
                revealed = subject.reveal_prompt_conditioning_population(
                    root,
                    baseline_artifact_cid=f"blake3:{'a' * 64}",
                    candidate_artifact_cid=f"blake3:{'b' * 64}",
                )
                self.assertEqual(revealed, population)
                sealed.chmod(0)
                recovered = subject.reveal_prompt_conditioning_population(
                    root,
                    baseline_artifact_cid=f"blake3:{'a' * 64}",
                    candidate_artifact_cid=f"blake3:{'b' * 64}",
                )
                self.assertEqual(recovered, population)
                self.assertEqual(stat.S_IMODE(sealed.stat().st_mode), 0o700)
                with self.assertRaisesRegex(ValueError, "binding differs"):
                    subject.reveal_prompt_conditioning_population(
                        root,
                        baseline_artifact_cid=f"blake3:{'a' * 64}",
                        candidate_artifact_cid=f"blake3:{'c' * 64}",
                    )


if __name__ == "__main__":
    unittest.main()
