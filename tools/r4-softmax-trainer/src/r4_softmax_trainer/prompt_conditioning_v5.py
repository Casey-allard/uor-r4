"""Fifth create-once matched-prompt population for issue #973.

V5 preserves the already qualified V4 contrast: two story prompts share their
last four tokens, while their complete prompts and continuations differ.  The
only population changes are the public post-V4 source boundary and the exact
V1-through-V4 story-CID exclusion union.  Selection is deterministic and is
not invoked until the terminal campaign's create-once preparation step.
"""

from __future__ import annotations

from collections import defaultdict
from collections.abc import Collection, Iterable, Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol

from blake3 import blake3

from .data import iter_canonical_stories, story_split, verify_source
from .prompt_conditioning_v4 import (
    BOS_TOKEN_ID,
    CONTINUATION_TOKENS,
    DIRECTION_COUNT,
    MAX_TOKEN_ID,
    PAIR_COUNT,
    PROMPT_TOKENS,
    SHARED_PROMPT_TAIL_TOKENS,
    SOURCE_BYTES,
    SOURCE_FILENAME,
    SOURCE_REPOSITORY,
    SOURCE_REVISION,
    SOURCE_SHA256,
    SPECIAL_TOKEN_IDS,
    SPLIT_POLICY_CID,
    TOKENIZER_CID,
    PromptConditioningPair,
    PromptConditioningRecord,
    PromptDirection,
    _prior_population_story_cids,
)
from .provenance import canonical_json_bytes, cid_bytes, cid_file


POLICY = "R4RetainedPromptSwapContrastV5"
POPULATION_SCHEMA = "uor-r4.retained-prompt-swap-population/5"
PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL = 409_546
REQUIRED_EXCLUDED_STORY_CIDS = 2_048
REQUIRED_EXCLUDED_STORY_CIDS_CID = (
    "blake3:c926c19deaae20a17b05fc3c5eddc099324d9b531bbfd83ac992a5ef02ede092"
)
SCORED_TARGET_TOKENS = DIRECTION_COUNT * CONTINUATION_TOKENS

V1_POPULATION_CID = (
    "blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340"
)
V2_POPULATION_CID = (
    "blake3:258f143eedbbb7067dc512db929a42166ad8a492fc059542409f419a3b46942e"
)
V3_POPULATION_CID = (
    "blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93"
)
V4_POPULATION_CID = (
    "blake3:cc9a1c40fe753e269ea31edd804c32b2a0c208ef20fceb1167636d6f28d7da11"
)
PRIOR_POPULATION_CONTRACTS = (
    (V1_POPULATION_CID, "uor-r4.retained-prompt-swap-population/1"),
    (V2_POPULATION_CID, "uor-r4.retained-prompt-swap-population/2"),
    (V3_POPULATION_CID, "uor-r4.retained-prompt-swap-population/3"),
    (V4_POPULATION_CID, "uor-r4.retained-prompt-swap-population/4"),
)

SELECTION_CONTRACT = {
    "split": "development (BLAKE3 raw-story digest mod 100 in 90..94)",
    "source_order": "strictly ascending after revealed V4 boundary 409546",
    "utf8": "strict",
    "content_tokens_required": PROMPT_TOKENS + CONTINUATION_TOKENS,
    "special_token_ids_forbidden_in_first_64": sorted(SPECIAL_TOKEN_IDS),
    "prompt_tokens": PROMPT_TOKENS,
    "continuation_tokens": CONTINUATION_TOKENS,
    "pair_key": "last 4 prompt token IDs",
    "pairing": (
        "current eligible story pairs with earliest unpaired same-key story "
        "whose full prompt and continuation both differ; remove matched pending story"
    ),
    "story_cid_exclusion": (
        "reject the CID-bound union of all 2048 V1, V2, V3, and V4 stories"
    ),
    "stop": "after 256 completed pairs",
}


class _Encoding(Protocol):
    ids: Collection[int]


class _Tokenizer(Protocol):
    def encode(self, sequence: str, add_special_tokens: bool = True) -> _Encoding: ...


class PromptConditioningPopulationUnavailable(RuntimeError):
    """The pinned source cannot satisfy the frozen V5 population contract."""


def _is_cid(value: object) -> bool:
    if not isinstance(value, str) or not value.startswith("blake3:"):
        return False
    digest = value.removeprefix("blake3:")
    return len(digest) == 64 and all(char in "0123456789abcdef" for char in digest)


def _token_tuple(values: Collection[int], *, length: int) -> tuple[int, ...]:
    if len(values) != length:
        raise ValueError("V5 token field has the wrong length")
    result: list[int] = []
    for value in values:
        if (
            isinstance(value, bool)
            or not isinstance(value, int)
            or not 0 <= value <= MAX_TOKEN_ID
            or value in SPECIAL_TOKEN_IDS
        ):
            raise ValueError("V5 token field contains an ineligible token")
        result.append(value)
    return tuple(result)


def load_required_prior_story_cids(
    population_paths: Collection[Path],
) -> tuple[str, ...]:
    """Bind and merge the exact four already revealed populations."""

    paths = tuple(population_paths)
    if len(paths) != len(PRIOR_POPULATION_CONTRACTS):
        raise ValueError("V5 requires exactly four prior population paths")
    combined: set[str] = set()
    for path, (expected_cid, expected_schema) in zip(
        paths, PRIOR_POPULATION_CONTRACTS, strict=True
    ):
        values = _prior_population_story_cids(
            path,
            expected_cid=expected_cid,
            expected_schema=expected_schema,
        )
        if combined.intersection(values):
            raise ValueError("prior prompt populations reuse a story CID")
        combined.update(values)
    ordered = tuple(sorted(combined))
    if len(ordered) != REQUIRED_EXCLUDED_STORY_CIDS or any(
        not _is_cid(value) for value in ordered
    ) or cid_bytes(canonical_json_bytes(list(ordered))) != REQUIRED_EXCLUDED_STORY_CIDS_CID:
        raise ValueError("V1-through-V4 story exclusion union differs")
    return ordered


@dataclass(frozen=True, slots=True)
class PromptConditioningPopulationV5:
    """The complete deterministic V5 prompt population."""

    pairs: tuple[PromptConditioningPair, ...]
    last_source_story_ordinal: int
    eligible_stories_examined: int
    excluded_story_cids: tuple[str, ...]

    def __post_init__(self) -> None:
        if (
            len(self.pairs) != PAIR_COUNT
            or tuple(pair.pair_index for pair in self.pairs) != tuple(range(PAIR_COUNT))
        ):
            raise ValueError("V5 requires 256 canonically indexed pairs")
        story_cids = tuple(
            record.story_cid
            for pair in self.pairs
            for record in (pair.left, pair.right)
        )
        if len(story_cids) != DIRECTION_COUNT or len(set(story_cids)) != DIRECTION_COUNT:
            raise ValueError("V5 requires 512 distinct prompt stories")
        exclusions = tuple(sorted(self.excluded_story_cids))
        if (
            len(exclusions) != REQUIRED_EXCLUDED_STORY_CIDS
            or len(set(exclusions)) != REQUIRED_EXCLUDED_STORY_CIDS
            or any(not _is_cid(value) for value in exclusions)
            or cid_bytes(canonical_json_bytes(list(exclusions)))
            != REQUIRED_EXCLUDED_STORY_CIDS_CID
        ):
            raise ValueError("V5 story exclusion set differs")
        object.__setattr__(self, "excluded_story_cids", exclusions)
        if set(story_cids).intersection(exclusions):
            raise ValueError("V5 reuses a V1-through-V4 story")
        ordinals = tuple(
            record.source_story_ordinal
            for pair in self.pairs
            for record in (pair.left, pair.right)
        )
        if any(
            isinstance(value, bool)
            or not isinstance(value, int)
            or value <= PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
            for value in ordinals
        ):
            raise ValueError("every V5 record ordinal must be an integer after V4")
        maximum = max(
            ordinals
        )
        if (
            isinstance(self.last_source_story_ordinal, bool)
            or not isinstance(self.last_source_story_ordinal, int)
            or isinstance(self.eligible_stories_examined, bool)
            or not isinstance(self.eligible_stories_examined, int)
            or self.last_source_story_ordinal != maximum
            or maximum <= PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
            or self.eligible_stories_examined < DIRECTION_COUNT
        ):
            raise ValueError("V5 source-order witness differs")

    @property
    def excluded_story_cids_cid(self) -> str:
        return cid_bytes(canonical_json_bytes(list(self.excluded_story_cids)))

    def manifest(self) -> dict[str, Any]:
        return {
            "schema": POPULATION_SCHEMA,
            "source": {
                "repository": SOURCE_REPOSITORY,
                "revision": SOURCE_REVISION,
                "filename": SOURCE_FILENAME,
                "bytes": SOURCE_BYTES,
                "sha256": SOURCE_SHA256,
            },
            "tokenizer_cid": TOKENIZER_CID,
            "split_policy_cid": SPLIT_POLICY_CID,
            "prior_revealed_last_source_story_ordinal": (
                PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
            ),
            "prior_population_exclusions": {
                "population_cids": [value[0] for value in PRIOR_POPULATION_CONTRACTS],
                "story_cids": list(self.excluded_story_cids),
                "story_cid_count": REQUIRED_EXCLUDED_STORY_CIDS,
                "story_cid_set_cid": REQUIRED_EXCLUDED_STORY_CIDS_CID,
            },
            "selection": dict(SELECTION_CONTRACT),
            "population": {
                "pairs": PAIR_COUNT,
                "directions": DIRECTION_COUNT,
                "scored_target_tokens": SCORED_TARGET_TOKENS,
                "last_source_story_ordinal": self.last_source_story_ordinal,
                "eligible_stories_examined": self.eligible_stories_examined,
            },
            "pairs": [pair.record() for pair in self.pairs],
        }

    @property
    def population_cid(self) -> str:
        return cid_bytes(canonical_json_bytes(self.manifest()))

    @classmethod
    def from_manifest(cls, value: Mapping[str, Any]) -> PromptConditioningPopulationV5:
        pairs_value = value.get("pairs")
        exclusions = value.get("prior_population_exclusions")
        population = value.get("population")
        if (
            value.get("schema") != POPULATION_SCHEMA
            or not isinstance(pairs_value, list)
            or not isinstance(exclusions, Mapping)
            or not isinstance(exclusions.get("story_cids"), list)
            or not isinstance(population, Mapping)
        ):
            raise ValueError("V5 population manifest is malformed")
        pairs = tuple(
            PromptConditioningPair.from_record(pair)
            for pair in pairs_value
            if isinstance(pair, Mapping)
        )
        if len(pairs) != len(pairs_value):
            raise ValueError("V5 population contains a malformed pair")
        candidate = cls(
            pairs=pairs,
            last_source_story_ordinal=population.get("last_source_story_ordinal"),
            eligible_stories_examined=population.get("eligible_stories_examined"),
            excluded_story_cids=tuple(exclusions["story_cids"]),
        )
        if candidate.manifest() != dict(value):
            raise ValueError("V5 population does not reproduce canonically")
        return candidate


def select_prompt_conditioning_population(
    indexed_stories: Iterable[tuple[int, bytes]],
    tokenizer: _Tokenizer,
    *,
    excluded_story_cids: Collection[str],
) -> PromptConditioningPopulationV5:
    """Select the first 256 eligible pairs after the exact V4 boundary."""

    exclusions = tuple(sorted(excluded_story_cids))
    if len(exclusions) != REQUIRED_EXCLUDED_STORY_CIDS or len(set(exclusions)) != len(
        exclusions
    ) or cid_bytes(canonical_json_bytes(list(exclusions))) != REQUIRED_EXCLUDED_STORY_CIDS_CID:
        raise ValueError("V5 exclusions must contain the exact 2048-story union")
    excluded = frozenset(exclusions)
    pending: dict[tuple[int, ...], list[PromptConditioningRecord]] = defaultdict(list)
    pairs: list[PromptConditioningPair] = []
    eligible = 0
    previous_ordinal = -1
    for source_ordinal, story in indexed_stories:
        if (
            isinstance(source_ordinal, bool)
            or not isinstance(source_ordinal, int)
            or source_ordinal <= previous_ordinal
        ):
            raise ValueError("V5 source ordinals must be strictly increasing integers")
        previous_ordinal = source_ordinal
        if not isinstance(story, bytes) or not story:
            raise ValueError("V5 canonical stories must be nonempty bytes")
        if source_ordinal <= PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL:
            continue
        if story_split(story) != "dev":
            continue
        story_cid = f"blake3:{blake3(story).hexdigest()}"
        if story_cid in excluded:
            continue
        try:
            text = story.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            continue
        token_ids = list(tokenizer.encode(text, add_special_tokens=False).ids)
        if len(token_ids) < PROMPT_TOKENS + CONTINUATION_TOKENS:
            continue
        try:
            prompt = _token_tuple(token_ids[:PROMPT_TOKENS], length=PROMPT_TOKENS)
            continuation = _token_tuple(
                token_ids[PROMPT_TOKENS : PROMPT_TOKENS + CONTINUATION_TOKENS],
                length=CONTINUATION_TOKENS,
            )
        except ValueError:
            continue
        eligible += 1
        candidate = PromptConditioningRecord(
            source_story_ordinal=source_ordinal,
            story_cid=story_cid,
            prompt_token_ids=prompt,
            continuation_token_ids=continuation,
        )
        waiting = pending[candidate.prompt_tail]
        match = next(
            (
                index
                for index, existing in enumerate(waiting)
                if existing.prompt_token_ids != candidate.prompt_token_ids
                and existing.continuation_token_ids != candidate.continuation_token_ids
            ),
            None,
        )
        if match is None:
            waiting.append(candidate)
            continue
        existing = waiting.pop(match)
        pairs.append(
            PromptConditioningPair(
                pair_index=len(pairs),
                left=existing,
                right=candidate,
            )
        )
        if len(pairs) == PAIR_COUNT:
            return PromptConditioningPopulationV5(
                pairs=tuple(pairs),
                last_source_story_ordinal=source_ordinal,
                eligible_stories_examined=eligible,
                excluded_story_cids=exclusions,
            )
    raise PromptConditioningPopulationUnavailable(
        f"pinned source ended with {len(pairs)}/{PAIR_COUNT} V5 pairs"
    )


def select_prompt_conditioning_population_from_source(
    source_path: Path,
    tokenizer_path: Path,
    *,
    excluded_story_cids: Collection[str],
) -> PromptConditioningPopulationV5:
    if (
        source_path.is_symlink()
        or not source_path.is_file()
        or tokenizer_path.is_symlink()
        or not tokenizer_path.is_file()
    ):
        raise ValueError("V5 source and tokenizer must be regular non-symlink files")
    verify_source(source_path)
    if cid_file(tokenizer_path) != TOKENIZER_CID:
        raise ValueError("V5 tokenizer differs from the frozen artifact")
    from tokenizers import Tokenizer

    return select_prompt_conditioning_population(
        enumerate(iter_canonical_stories(source_path)),
        Tokenizer.from_file(str(tokenizer_path)),
        excluded_story_cids=excluded_story_cids,
    )


def prompt_directions(
    population: PromptConditioningPopulationV5,
) -> tuple[PromptDirection, ...]:
    return tuple(
        direction
        for pair in population.pairs
        for direction in (
            PromptDirection(
                pair_index=pair.pair_index,
                side="left",
                own_prompt=pair.left.prompt_token_ids,
                crossed_prompt=pair.right.prompt_token_ids,
                continuation=pair.left.continuation_token_ids,
            ),
            PromptDirection(
                pair_index=pair.pair_index,
                side="right",
                own_prompt=pair.right.prompt_token_ids,
                crossed_prompt=pair.left.prompt_token_ids,
                continuation=pair.right.continuation_token_ids,
            ),
        )
    )


__all__ = [
    "BOS_TOKEN_ID",
    "CONTINUATION_TOKENS",
    "DIRECTION_COUNT",
    "PAIR_COUNT",
    "POLICY",
    "POPULATION_SCHEMA",
    "PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL",
    "PROMPT_TOKENS",
    "PromptConditioningPopulationUnavailable",
    "PromptConditioningPopulationV5",
    "SCORED_TARGET_TOKENS",
    "load_required_prior_story_cids",
    "prompt_directions",
    "select_prompt_conditioning_population",
    "select_prompt_conditioning_population_from_source",
]
