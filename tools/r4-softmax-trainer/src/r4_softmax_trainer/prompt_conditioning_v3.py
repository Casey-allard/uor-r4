"""Third independently frozen matched-prompt criterion for issue #973.

This module does two things only:

* deterministically commits a population strictly after the revealed V2 boundary; and
* scores the same continuation under its own and a matched foreign prompt.

Matched prompts share their last four tokens.  Consequently, the paired
contrast cannot be attributed to target difficulty or to a local four-token
language model.  The complete V3 population has its own schemas and storage
namespace, is sealed before candidate fitting, and is opened only after the
qualified V1 and layerwise-readout candidate artifact CIDs have been fixed.
Prior population files are read only to verify and bind the exact V1+V2
story-CID exclusion set before V3 selection.
"""

from __future__ import annotations

import json
import math
import os
import stat
import struct
from collections import defaultdict
from collections.abc import Callable, Collection, Iterable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol

import torch
from blake3 import blake3
from torch import Tensor

from .data import iter_canonical_stories, story_split, verify_source
from .provenance import canonical_json_bytes, cid_bytes, cid_file

ISSUE = 973
POLICY = "R4RetainedPromptSwapContrastV3"

SOURCE_REPOSITORY = "roneneldan/TinyStories"
SOURCE_REVISION = "f54c09fd23315a6f9c86f9dc80f725de7d8f9c64"
SOURCE_FILENAME = "TinyStoriesV2-GPT4-train.txt"
SOURCE_BYTES = 2_227_753_162
SOURCE_SHA256 = "6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443"
TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
SPLIT_POLICY_CID = (
    "blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa"
)

PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL = 241_074
V1_POPULATION_CID = (
    "blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340"
)
V2_POPULATION_CID = (
    "blake3:258f143eedbbb7067dc512db929a42166ad8a492fc059542409f419a3b46942e"
)
REQUIRED_PRIOR_POPULATION_CIDS = (V1_POPULATION_CID, V2_POPULATION_CID)
REQUIRED_EXCLUDED_STORY_CIDS = 1_024
REQUIRED_EXCLUDED_STORY_CIDS_CID = (
    "blake3:d33160871a87a9cec7912f394d7546b63f5ebc6323eacc58a65244e9f1148c1c"
)
PROMPT_TOKENS = 48
CONTINUATION_TOKENS = 16
SHARED_PROMPT_TAIL_TOKENS = 4
PAIR_COUNT = 256
DIRECTION_COUNT = PAIR_COUNT * 2
SCORED_TARGET_TOKENS = DIRECTION_COUNT * CONTINUATION_TOKENS
BOS_TOKEN_ID = 0
SPECIAL_TOKEN_IDS = frozenset({0, 1, 2})
MAX_TOKEN_ID = 4_095

POPULATION_SCHEMA = "uor-r4.retained-prompt-swap-population/3"
COMMITMENT_SCHEMA = "uor-r4.retained-prompt-swap-commitment/3"
REVEAL_SCHEMA = "uor-r4.retained-prompt-swap-reveal/3"
SCORE_SCHEMA = "uor-r4.retained-prompt-swap-score/3"
DECISION_SCHEMA = "uor-r4.retained-prompt-swap-decision/3"

WORK_RELATIVE_PATH = "prompt-conditioning-v3"
SEALED_DIRECTORY_RELATIVE_PATH = f"{WORK_RELATIVE_PATH}/sealed"
POPULATION_RELATIVE_PATH = f"{SEALED_DIRECTORY_RELATIVE_PATH}/population.json"
COMMITMENT_RELATIVE_PATH = f"{WORK_RELATIVE_PATH}/population-commitment.json"
REVEAL_RELATIVE_PATH = f"{WORK_RELATIVE_PATH}/reveal.json"

ABSOLUTE_GAIN_THRESHOLD = math.log(2.0) / CONTINUATION_TOKENS
CAPACITY_GAIN_THRESHOLD = math.log(1.5) / CONTINUATION_TOKENS
WIN_THRESHOLD = 308
STATE_OFF_TOLERANCE = 1e-7

VERDICT_PASS = "PROMPT_CONDITIONING_CAPACITY_PASS"
VERDICT_ABSOLUTE_NO_CAPACITY_GAIN = "PROMPT_CONDITIONING_ABSOLUTE_NO_CAPACITY_GAIN"
VERDICT_PARTIAL = "PROMPT_CONDITIONING_PARTIAL"
VERDICT_FAIL = "PROMPT_CONDITIONING_CAPACITY_FAIL"
VERDICT_INVALID = "INVALID_PROMPT_CONTRAST"


class PromptConditioningPopulationUnavailable(RuntimeError):
    """The pinned source cannot satisfy the frozen population contract."""

    terminal = "UNAVAILABLE_PROMPT_CONDITIONING_POPULATION"


class _Encoding(Protocol):
    ids: Sequence[int]


class _Tokenizer(Protocol):
    def encode(self, sequence: str, add_special_tokens: bool = True) -> _Encoding: ...


def _is_blake3_cid(value: Any) -> bool:
    if not isinstance(value, str) or not value.startswith("blake3:"):
        return False
    digest = value.removeprefix("blake3:")
    return len(digest) == 64 and all(
        character in "0123456789abcdef" for character in digest
    )


def _normalize_required_story_cid_exclusions(
    values: Collection[str],
) -> tuple[str, ...]:
    if isinstance(values, (str, bytes)):
        raise TypeError("excluded story CIDs must be a collection of CIDs")
    normalized = tuple(sorted(values))
    if (
        len(normalized) != REQUIRED_EXCLUDED_STORY_CIDS
        or len(set(normalized)) != REQUIRED_EXCLUDED_STORY_CIDS
        or any(not _is_blake3_cid(value) for value in normalized)
        or cid_bytes(canonical_json_bytes(list(normalized)))
        != REQUIRED_EXCLUDED_STORY_CIDS_CID
    ):
        raise ValueError("excluded story CIDs differ from the exact V1+V2 union")
    return normalized


def _require_int(value: Any, *, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise ValueError(f"{label} must be an integer >= {minimum}")
    return value


def _token_tuple(value: Sequence[int], *, length: int, label: str) -> tuple[int, ...]:
    if len(value) != length:
        raise ValueError(f"{label} must contain exactly {length} token IDs")
    normalized: list[int] = []
    for token_id in value:
        token = _require_int(token_id, label=f"{label} token", minimum=0)
        if token > MAX_TOKEN_ID or token in SPECIAL_TOKEN_IDS:
            raise ValueError(f"{label} contains an ineligible token ID")
        normalized.append(token)
    return tuple(normalized)


@dataclass(frozen=True, slots=True)
class PromptConditioningRecord:
    """One story-derived prompt and its untouched continuation."""

    source_story_ordinal: int
    story_cid: str
    prompt_token_ids: tuple[int, ...]
    continuation_token_ids: tuple[int, ...]

    def __post_init__(self) -> None:
        _require_int(
            self.source_story_ordinal,
            label="source story ordinal",
            minimum=PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL + 1,
        )
        if not _is_blake3_cid(self.story_cid):
            raise ValueError("story CID must be a lowercase BLAKE3 CID")
        object.__setattr__(
            self,
            "prompt_token_ids",
            _token_tuple(
                self.prompt_token_ids,
                length=PROMPT_TOKENS,
                label="prompt",
            ),
        )
        object.__setattr__(
            self,
            "continuation_token_ids",
            _token_tuple(
                self.continuation_token_ids,
                length=CONTINUATION_TOKENS,
                label="continuation",
            ),
        )

    @property
    def prompt_tail(self) -> tuple[int, ...]:
        return self.prompt_token_ids[-SHARED_PROMPT_TAIL_TOKENS:]

    def record(self) -> dict[str, Any]:
        return {
            "source_story_ordinal": self.source_story_ordinal,
            "story_cid": self.story_cid,
            "prompt_token_ids": list(self.prompt_token_ids),
            "continuation_token_ids": list(self.continuation_token_ids),
        }

    @classmethod
    def from_record(cls, value: Mapping[str, Any]) -> PromptConditioningRecord:
        if set(value) != {
            "source_story_ordinal",
            "story_cid",
            "prompt_token_ids",
            "continuation_token_ids",
        }:
            raise ValueError("prompt-conditioning record fields differ")
        prompt = value["prompt_token_ids"]
        continuation = value["continuation_token_ids"]
        if not isinstance(prompt, list) or not isinstance(continuation, list):
            raise TypeError("prompt-conditioning token fields must be lists")
        return cls(
            source_story_ordinal=value["source_story_ordinal"],
            story_cid=value["story_cid"],
            prompt_token_ids=tuple(prompt),
            continuation_token_ids=tuple(continuation),
        )


@dataclass(frozen=True, slots=True)
class PromptConditioningPair:
    """Two distinct prompts with an identical four-token local tail."""

    pair_index: int
    left: PromptConditioningRecord
    right: PromptConditioningRecord

    def __post_init__(self) -> None:
        _require_int(self.pair_index, label="pair index")
        if self.left.story_cid == self.right.story_cid:
            raise ValueError("paired stories must have distinct CIDs")
        if self.left.prompt_tail != self.right.prompt_tail:
            raise ValueError("paired prompts must share their final four tokens")
        if self.left.prompt_token_ids == self.right.prompt_token_ids:
            raise ValueError("paired complete prompts must differ")
        if self.left.continuation_token_ids == self.right.continuation_token_ids:
            raise ValueError("paired continuations must differ")

    def record(self) -> dict[str, Any]:
        return {
            "pair_index": self.pair_index,
            "shared_prompt_tail_token_ids": list(self.left.prompt_tail),
            "left": self.left.record(),
            "right": self.right.record(),
        }

    @classmethod
    def from_record(cls, value: Mapping[str, Any]) -> PromptConditioningPair:
        if set(value) != {
            "pair_index",
            "shared_prompt_tail_token_ids",
            "left",
            "right",
        }:
            raise ValueError("prompt-conditioning pair fields differ")
        left = value["left"]
        right = value["right"]
        tail = value["shared_prompt_tail_token_ids"]
        if not isinstance(left, Mapping) or not isinstance(right, Mapping):
            raise TypeError("prompt-conditioning pair records are malformed")
        if not isinstance(tail, list):
            raise TypeError("shared prompt tail must be a token list")
        pair = cls(
            pair_index=value["pair_index"],
            left=PromptConditioningRecord.from_record(left),
            right=PromptConditioningRecord.from_record(right),
        )
        if list(pair.left.prompt_tail) != tail:
            raise ValueError("shared prompt tail record differs from the prompts")
        return pair


SELECTION_CONTRACT = {
    "split": "development (BLAKE3 raw-story digest mod 100 in 90..94)",
    "source_order": "strictly ascending after revealed V2 boundary 241074",
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
        "reject the exact CID-bound union of all 512 V1 and all 512 V2 stories"
    ),
    "stop": "after 256 completed pairs",
}


@dataclass(frozen=True, slots=True)
class PromptConditioningPopulation:
    """The complete frozen matched-prompt population."""

    pairs: tuple[PromptConditioningPair, ...]
    last_source_story_ordinal: int
    eligible_stories_examined: int
    excluded_story_cids: tuple[str, ...]

    def __post_init__(self) -> None:
        if len(self.pairs) != PAIR_COUNT:
            raise ValueError(
                f"prompt-conditioning population requires {PAIR_COUNT} pairs"
            )
        if tuple(pair.pair_index for pair in self.pairs) != tuple(range(PAIR_COUNT)):
            raise ValueError("prompt-conditioning pair indexes are not canonical")
        story_cids = [
            record.story_cid
            for pair in self.pairs
            for record in (pair.left, pair.right)
        ]
        if (
            len(story_cids) != DIRECTION_COUNT
            or len(set(story_cids)) != DIRECTION_COUNT
        ):
            raise ValueError(
                "prompt-conditioning population requires 512 distinct stories"
            )
        exclusions = _normalize_required_story_cid_exclusions(self.excluded_story_cids)
        object.__setattr__(self, "excluded_story_cids", exclusions)
        if not set(story_cids).isdisjoint(exclusions):
            raise ValueError("V3 population reuses a V1 or V2 story CID")
        maximum_ordinal = max(
            record.source_story_ordinal
            for pair in self.pairs
            for record in (pair.left, pair.right)
        )
        if self.last_source_story_ordinal != maximum_ordinal:
            raise ValueError("population last source ordinal differs from its records")
        _require_int(
            self.eligible_stories_examined,
            label="eligible stories examined",
            minimum=DIRECTION_COUNT,
        )

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
                "population_cids": list(REQUIRED_PRIOR_POPULATION_CIDS),
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
    def from_manifest(cls, value: Mapping[str, Any]) -> PromptConditioningPopulation:
        if set(value) != {
            "schema",
            "source",
            "tokenizer_cid",
            "split_policy_cid",
            "prior_revealed_last_source_story_ordinal",
            "prior_population_exclusions",
            "selection",
            "population",
            "pairs",
        }:
            raise ValueError("prompt-conditioning population manifest fields differ")
        source = value["source"]
        exclusions = value["prior_population_exclusions"]
        population = value["population"]
        raw_pairs = value["pairs"]
        if (
            value["schema"] != POPULATION_SCHEMA
            or source
            != {
                "repository": SOURCE_REPOSITORY,
                "revision": SOURCE_REVISION,
                "filename": SOURCE_FILENAME,
                "bytes": SOURCE_BYTES,
                "sha256": SOURCE_SHA256,
            }
            or value["tokenizer_cid"] != TOKENIZER_CID
            or value["split_policy_cid"] != SPLIT_POLICY_CID
            or value["prior_revealed_last_source_story_ordinal"]
            != PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL
            or not isinstance(exclusions, Mapping)
            or set(exclusions)
            != {
                "population_cids",
                "story_cids",
                "story_cid_count",
                "story_cid_set_cid",
            }
            or exclusions.get("population_cids") != list(REQUIRED_PRIOR_POPULATION_CIDS)
            or exclusions.get("story_cid_count") != REQUIRED_EXCLUDED_STORY_CIDS
            or exclusions.get("story_cid_set_cid") != REQUIRED_EXCLUDED_STORY_CIDS_CID
            or not isinstance(exclusions.get("story_cids"), list)
            or value["selection"] != SELECTION_CONTRACT
            or not isinstance(population, Mapping)
            or not isinstance(raw_pairs, list)
        ):
            raise ValueError("prompt-conditioning population contract differs")
        pairs = tuple(
            PromptConditioningPair.from_record(pair)
            for pair in raw_pairs
            if isinstance(pair, Mapping)
        )
        if len(pairs) != len(raw_pairs):
            raise ValueError("prompt-conditioning population has a malformed pair")
        candidate = cls(
            pairs=pairs,
            last_source_story_ordinal=population.get("last_source_story_ordinal"),
            eligible_stories_examined=population.get("eligible_stories_examined"),
            excluded_story_cids=tuple(exclusions["story_cids"]),
        )
        if candidate.manifest() != dict(value):
            raise ValueError(
                "prompt-conditioning population does not reproduce canonically"
            )
        return candidate


def select_prompt_conditioning_population(
    indexed_stories: Iterable[tuple[int, bytes]],
    tokenizer: _Tokenizer,
    *,
    excluded_story_cids: Collection[str],
) -> PromptConditioningPopulation:
    """Select the first frozen V3 256 pairs from canonical indexed stories.

    The caller supplies global source ordinals, allowing focused synthetic tests
    without materializing the 241,075-story prefix.  Production callers must
    enumerate :func:`data.iter_canonical_stories` from ordinal zero.
    """

    exclusions = _normalize_required_story_cid_exclusions(excluded_story_cids)
    excluded = frozenset(exclusions)
    pending: dict[tuple[int, ...], list[PromptConditioningRecord]] = defaultdict(list)
    pairs: list[PromptConditioningPair] = []
    eligible = 0
    previous_ordinal = -1

    for source_ordinal, story in indexed_stories:
        ordinal = _require_int(source_ordinal, label="source story ordinal")
        if ordinal <= previous_ordinal:
            raise ValueError("source story ordinals must be strictly increasing")
        previous_ordinal = ordinal
        if not isinstance(story, bytes) or not story:
            raise ValueError("canonical stories must be nonempty bytes")
        if ordinal <= PRIOR_REVEALED_LAST_SOURCE_STORY_ORDINAL:
            continue
        if story_split(story) != "dev":
            continue
        story_digest = blake3(story).digest()
        story_cid = f"blake3:{story_digest.hex()}"
        if story_cid in excluded:
            continue
        try:
            text = story.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            continue
        encoded = tokenizer.encode(text, add_special_tokens=False)
        token_ids = list(encoded.ids)
        required = PROMPT_TOKENS + CONTINUATION_TOKENS
        if len(token_ids) < required:
            continue
        selected = token_ids[:required]
        try:
            prompt = _token_tuple(
                selected[:PROMPT_TOKENS], length=PROMPT_TOKENS, label="prompt"
            )
            continuation = _token_tuple(
                selected[PROMPT_TOKENS:],
                length=CONTINUATION_TOKENS,
                label="continuation",
            )
        except ValueError:
            continue

        eligible += 1
        candidate = PromptConditioningRecord(
            source_story_ordinal=ordinal,
            story_cid=story_cid,
            prompt_token_ids=prompt,
            continuation_token_ids=continuation,
        )
        waiting = pending[candidate.prompt_tail]
        matched_index = next(
            (
                index
                for index, existing in enumerate(waiting)
                if existing.prompt_token_ids != candidate.prompt_token_ids
                and existing.continuation_token_ids != candidate.continuation_token_ids
            ),
            None,
        )
        if matched_index is None:
            waiting.append(candidate)
            continue
        existing = waiting.pop(matched_index)
        pairs.append(
            PromptConditioningPair(
                pair_index=len(pairs),
                left=existing,
                right=candidate,
            )
        )
        if len(pairs) == PAIR_COUNT:
            return PromptConditioningPopulation(
                pairs=tuple(pairs),
                last_source_story_ordinal=ordinal,
                eligible_stories_examined=eligible,
                excluded_story_cids=exclusions,
            )

    raise PromptConditioningPopulationUnavailable(
        f"pinned population ended with {len(pairs)}/{PAIR_COUNT} matched pairs"
    )


def select_prompt_conditioning_population_from_source(
    source_path: Path,
    tokenizer_path: Path,
    *,
    excluded_story_cids: Collection[str],
) -> PromptConditioningPopulation:
    """Verify and load the pinned source/tokenizer, then run the selector."""

    if source_path.is_symlink() or not source_path.is_file():
        raise ValueError(
            "prompt-conditioning source must be a regular non-symlink file"
        )
    if tokenizer_path.is_symlink() or not tokenizer_path.is_file():
        raise ValueError(
            "prompt-conditioning tokenizer must be a regular non-symlink file"
        )
    verify_source(source_path)
    if cid_file(tokenizer_path) != TOKENIZER_CID:
        raise ValueError(
            "prompt-conditioning tokenizer differs from the frozen artifact"
        )
    from tokenizers import Tokenizer

    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    return select_prompt_conditioning_population(
        enumerate(iter_canonical_stories(source_path)),
        tokenizer,
        excluded_story_cids=excluded_story_cids,
    )


def _write_exclusive(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as handle:
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())


def _with_self_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    observed = value.get(field)
    unsigned = dict(value)
    unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_json(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"expected a regular non-symlink JSON file: {path}")
    try:
        payload = path.read_bytes()
        value = json.loads(payload.decode("utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read canonical JSON: {path}") from error
    if not isinstance(value, dict) or canonical_json_bytes(value) != payload:
        raise ValueError(f"JSON file is not canonical: {path}")
    return value


def _prior_population_story_cids(
    path: Path,
    *,
    expected_cid: str,
    expected_schema: str,
) -> frozenset[str]:
    if path.is_symlink() or not path.is_file():
        raise ValueError("prior prompt population must be a regular non-symlink file")
    resolved = path.resolve()
    value = _read_json(resolved)
    population = value.get("population")
    pairs = value.get("pairs")
    if (
        cid_bytes(canonical_json_bytes(value)) != expected_cid
        or value.get("schema") != expected_schema
        or not isinstance(population, Mapping)
        or population.get("pairs") != PAIR_COUNT
        or population.get("directions") != DIRECTION_COUNT
        or not isinstance(pairs, list)
        or len(pairs) != PAIR_COUNT
    ):
        raise ValueError("prior prompt population differs from its exact freeze")
    story_cids: list[str] = []
    for pair_index, pair in enumerate(pairs):
        if not isinstance(pair, Mapping) or pair.get("pair_index") != pair_index:
            raise ValueError("prior prompt population pair order differs")
        for side in ("left", "right"):
            record = pair.get(side)
            story_cid = record.get("story_cid") if isinstance(record, Mapping) else None
            if not _is_blake3_cid(story_cid):
                raise ValueError("prior prompt population has an invalid story CID")
            story_cids.append(story_cid)
    if len(story_cids) != DIRECTION_COUNT or len(set(story_cids)) != DIRECTION_COUNT:
        raise ValueError("prior prompt population does not contain 512 unique stories")
    return frozenset(story_cids)


def load_required_prior_story_cids(
    v1_population_path: Path,
    v2_population_path: Path,
) -> frozenset[str]:
    """Load and bind the exact revealed V1+V2 story-CID exclusion union."""

    v1 = _prior_population_story_cids(
        v1_population_path,
        expected_cid=V1_POPULATION_CID,
        expected_schema="uor-r4.retained-prompt-swap-population/1",
    )
    v2 = _prior_population_story_cids(
        v2_population_path,
        expected_cid=V2_POPULATION_CID,
        expected_schema="uor-r4.retained-prompt-swap-population/2",
    )
    combined = frozenset((*v1, *v2))
    _normalize_required_story_cid_exclusions(combined)
    return combined


def seal_prompt_conditioning_population(
    root: Path,
    population: PromptConditioningPopulation,
) -> dict[str, Any]:
    """Write the population once and make its directory unreadable."""

    root = root.resolve()
    sealed_directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    population_path = root / POPULATION_RELATIVE_PATH
    commitment_path = root / COMMITMENT_RELATIVE_PATH
    reveal_path = root / REVEAL_RELATIVE_PATH
    if any(
        path.exists() or path.is_symlink()
        for path in (sealed_directory, commitment_path, reveal_path)
    ):
        raise FileExistsError("prompt-conditioning population is create-once")

    population_payload = canonical_json_bytes(population.manifest())
    population_cid = cid_bytes(population_payload)
    _write_exclusive(population_path, population_payload)
    commitment = _with_self_cid(
        {
            "schema": COMMITMENT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "population": {
                "path": POPULATION_RELATIVE_PATH,
                "bytes": len(population_payload),
                "cid": population_cid,
                "pairs": PAIR_COUNT,
                "directions": DIRECTION_COUNT,
                "scored_target_tokens": SCORED_TARGET_TOKENS,
            },
            "access_policy": "directory mode 000 until one-time reveal",
        },
        "commitment_cid",
    )
    _write_exclusive(commitment_path, canonical_json_bytes(commitment))
    sealed_directory.chmod(0)
    return commitment


def _validate_commitment(
    commitment: Mapping[str, Any],
    *,
    sealed_directory: Path,
    expected_directory_mode: int,
) -> Mapping[str, Any]:
    _verify_self_cid(commitment, "commitment_cid")
    population = commitment.get("population")
    if (
        commitment.get("schema") != COMMITMENT_SCHEMA
        or commitment.get("issue") != ISSUE
        or commitment.get("policy") != POLICY
        or commitment.get("access_policy") != "directory mode 000 until one-time reveal"
        or not isinstance(population, Mapping)
        or set(population)
        != {"path", "bytes", "cid", "pairs", "directions", "scored_target_tokens"}
        or population.get("path") != POPULATION_RELATIVE_PATH
        or not _is_blake3_cid(population.get("cid"))
        or population.get("pairs") != PAIR_COUNT
        or population.get("directions") != DIRECTION_COUNT
        or population.get("scored_target_tokens") != SCORED_TARGET_TOKENS
        or isinstance(population.get("bytes"), bool)
        or not isinstance(population.get("bytes"), int)
        or population.get("bytes", 0) <= 0
        or sealed_directory.is_symlink()
        or not sealed_directory.is_dir()
        or stat.S_IMODE(sealed_directory.stat().st_mode) != expected_directory_mode
    ):
        raise ValueError("prompt-conditioning commitment differs from the freeze")
    return population


def load_prompt_conditioning_commitment(root: Path) -> dict[str, Any]:
    """Verify the public commitment without opening the sealed population."""

    root = root.resolve()
    commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
    _validate_commitment(
        commitment,
        sealed_directory=root / SEALED_DIRECTORY_RELATIVE_PATH,
        expected_directory_mode=0,
    )
    return commitment


def _artifact_cid(value: str, *, label: str) -> str:
    if not _is_blake3_cid(value):
        raise ValueError(f"{label} must be a lowercase BLAKE3 CID")
    return value


def reveal_prompt_conditioning_population(
    root: Path,
    *,
    baseline_artifact_cid: str,
    candidate_artifact_cid: str,
) -> PromptConditioningPopulation:
    """Open the committed population after both artifact CIDs are fixed.

    The create-once marker remains immutable.  Repeating this call with the
    exact same artifact bindings only repairs an interruption between writing
    that marker and reopening the sealed directory; different bindings fail.
    """

    root = root.resolve()
    reveal_path = root / REVEAL_RELATIVE_PATH
    baseline_cid = _artifact_cid(
        baseline_artifact_cid,
        label="baseline artifact CID",
    )
    candidate_cid = _artifact_cid(
        candidate_artifact_cid,
        label="candidate artifact CID",
    )
    if reveal_path.exists() or reveal_path.is_symlink():
        commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
        reveal = _read_json(reveal_path)
        _verify_self_cid(reveal, "reveal_cid")
        sealed_directory = root / SEALED_DIRECTORY_RELATIVE_PATH
        if sealed_directory.is_symlink() or not sealed_directory.is_dir():
            raise ValueError("prompt-conditioning sealed directory differs")
        directory_mode = stat.S_IMODE(sealed_directory.stat().st_mode)
        if directory_mode not in (0, 0o700):
            raise ValueError("prompt-conditioning sealed directory mode differs")
        commitment_population = _validate_commitment(
            commitment,
            sealed_directory=sealed_directory,
            expected_directory_mode=directory_mode,
        )
        if (
            reveal.get("schema") != REVEAL_SCHEMA
            or reveal.get("issue") != ISSUE
            or reveal.get("policy") != POLICY
            or reveal.get("commitment_cid") != commitment.get("commitment_cid")
            or reveal.get("population_cid") != commitment_population.get("cid")
            or reveal.get("baseline_artifact_cid") != baseline_cid
            or reveal.get("candidate_artifact_cid") != candidate_cid
            or reveal.get("reveal_count") != 1
        ):
            raise ValueError("existing prompt-conditioning reveal binding differs")
        if directory_mode == 0:
            sealed_directory.chmod(0o700)
        return load_revealed_prompt_conditioning_population(root)
    commitment = load_prompt_conditioning_commitment(root)
    reveal = _with_self_cid(
        {
            "schema": REVEAL_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "commitment_cid": commitment["commitment_cid"],
            "population_cid": commitment["population"]["cid"],
            "baseline_artifact_cid": baseline_cid,
            "candidate_artifact_cid": candidate_cid,
            "reveal_count": 1,
        },
        "reveal_cid",
    )
    _write_exclusive(reveal_path, canonical_json_bytes(reveal))
    sealed_directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    sealed_directory.chmod(0o700)
    return load_revealed_prompt_conditioning_population(root)


def load_revealed_prompt_conditioning_population(
    root: Path,
) -> PromptConditioningPopulation:
    """Load and verify a population only after its create-once reveal marker."""

    root = root.resolve()
    commitment = _read_json(root / COMMITMENT_RELATIVE_PATH)
    commitment_population = _validate_commitment(
        commitment,
        sealed_directory=root / SEALED_DIRECTORY_RELATIVE_PATH,
        expected_directory_mode=0o700,
    )
    reveal = _read_json(root / REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    if (
        reveal.get("schema") != REVEAL_SCHEMA
        or reveal.get("issue") != ISSUE
        or reveal.get("policy") != POLICY
        or reveal.get("commitment_cid") != commitment.get("commitment_cid")
        or reveal.get("population_cid") != commitment_population.get("cid")
        or reveal.get("reveal_count") != 1
        or not _is_blake3_cid(reveal.get("baseline_artifact_cid"))
        or not _is_blake3_cid(reveal.get("candidate_artifact_cid"))
    ):
        raise ValueError("prompt-conditioning reveal marker differs from the freeze")
    population_path = root / POPULATION_RELATIVE_PATH
    value = _read_json(population_path)
    payload = canonical_json_bytes(value)
    if (
        len(payload) != commitment_population["bytes"]
        or cid_bytes(payload) != commitment_population["cid"]
    ):
        raise ValueError(
            "revealed prompt-conditioning population differs from commitment"
        )
    return PromptConditioningPopulation.from_manifest(value)


@dataclass(frozen=True, slots=True)
class PromptDirection:
    pair_index: int
    side: str
    own_prompt: tuple[int, ...]
    crossed_prompt: tuple[int, ...]
    continuation: tuple[int, ...]


def prompt_directions(
    population: PromptConditioningPopulation,
) -> tuple[PromptDirection, ...]:
    directions: list[PromptDirection] = []
    for pair in population.pairs:
        directions.extend(
            (
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
    return tuple(directions)


@dataclass(frozen=True, slots=True)
class PromptConditioningScore:
    """One model/mode result over the exact population."""

    attention_off: bool
    directions: int
    scored_target_tokens: int
    mean_gain_nats_per_token: float
    wins: int
    own_nll_nats_per_token: float
    crossed_nll_nats_per_token: float
    maximum_paired_logits_delta: float
    forbidden_reads: int
    scored_logprob_trace_cid: str

    def record(self) -> dict[str, Any]:
        return {
            "schema": SCORE_SCHEMA,
            "attention_off": self.attention_off,
            "directions": self.directions,
            "scored_target_tokens": self.scored_target_tokens,
            "mean_gain_nats_per_token": self.mean_gain_nats_per_token,
            "wins": self.wins,
            "own_nll_nats_per_token": self.own_nll_nats_per_token,
            "crossed_nll_nats_per_token": self.crossed_nll_nats_per_token,
            "maximum_paired_logits_delta": self.maximum_paired_logits_delta,
            "forbidden_reads": self.forbidden_reads,
            "scored_logprob_trace_cid": self.scored_logprob_trace_cid,
        }


def _model_logits(
    output: Any,
    *,
    expected_batch: int,
    attention_off: bool,
) -> tuple[Tensor, int]:
    logits = getattr(output, "logits", output)
    if not isinstance(logits, Tensor) or logits.ndim != 3:
        raise ValueError(
            "prompt-conditioning model must return [batch,time,vocab] logits"
        )
    expected_time = 1 + PROMPT_TOKENS + CONTINUATION_TOKENS - 1
    if tuple(logits.shape[:2]) != (expected_batch, expected_time):
        raise ValueError(
            "prompt-conditioning model logits have the wrong batch/time shape"
        )
    if not torch.isfinite(logits).all().item():
        raise ValueError("prompt-conditioning model returned nonfinite logits")
    audit = getattr(output, "audit", None)
    forbidden_reads = getattr(audit, "forbidden_reads", None)
    if (
        isinstance(forbidden_reads, bool)
        or not isinstance(forbidden_reads, int)
        or forbidden_reads < 0
    ):
        raise ValueError("prompt-conditioning model omitted its forbidden-read audit")
    if hasattr(audit, "attention_off"):
        audited_attention_off = audit.attention_off
    elif hasattr(audit, "state_off"):
        audited_attention_off = audit.state_off
    else:
        raise ValueError("prompt-conditioning model omitted its attention-mode audit")
    if (
        not isinstance(audited_attention_off, bool)
        or audited_attention_off is not attention_off
    ):
        raise ValueError("prompt-conditioning model audit reports the wrong mode")
    return logits, forbidden_reads


def _sequence(prompt: Sequence[int], continuation: Sequence[int]) -> list[int]:
    return [BOS_TOKEN_ID, *prompt, *continuation[:-1]]


def score_prompt_conditioning(
    model: Any,
    population: PromptConditioningPopulation,
    *,
    attention_off: bool,
    direction_batch_size: int = 8,
    device: torch.device | str = "cpu",
) -> PromptConditioningScore:
    """Score every own/crossed direction in deterministic population order."""

    if (
        isinstance(direction_batch_size, bool)
        or not isinstance(direction_batch_size, int)
        or direction_batch_size < 1
    ):
        raise ValueError("direction batch size must be a positive integer")
    selected_device = torch.device(device)
    if hasattr(model, "eval"):
        model.eval()

    directions = prompt_directions(population)
    gains: list[float] = []
    own_log_probabilities: list[float] = []
    crossed_log_probabilities: list[float] = []
    maximum_delta = 0.0
    forbidden_reads = 0
    trace = blake3()

    with torch.inference_mode():
        for start in range(0, len(directions), direction_batch_size):
            batch = directions[start : start + direction_batch_size]
            rows: list[list[int]] = []
            targets: list[tuple[int, ...]] = []
            for direction in batch:
                rows.append(_sequence(direction.own_prompt, direction.continuation))
                rows.append(_sequence(direction.crossed_prompt, direction.continuation))
                targets.append(direction.continuation)
            inputs = torch.tensor(rows, dtype=torch.long, device=selected_device)
            output = model(inputs, attention_off=attention_off)
            logits, call_forbidden_reads = _model_logits(
                output,
                expected_batch=len(rows),
                attention_off=attention_off,
            )
            forbidden_reads += call_forbidden_reads
            suffix_logits = logits[
                :, PROMPT_TOKENS : PROMPT_TOKENS + CONTINUATION_TOKENS, :
            ]
            suffix_logits = suffix_logits.double()
            log_probabilities = torch.log_softmax(suffix_logits, dim=-1)

            for offset, target_tuple in enumerate(targets):
                own_row = offset * 2
                crossed_row = own_row + 1
                target = torch.tensor(
                    target_tuple,
                    dtype=torch.long,
                    device=log_probabilities.device,
                )
                if int(target.max().item()) >= int(log_probabilities.shape[-1]):
                    raise ValueError(
                        "prompt-conditioning target lies outside model vocabulary"
                    )
                own = log_probabilities[own_row].gather(1, target[:, None])[:, 0]
                crossed = log_probabilities[crossed_row].gather(1, target[:, None])[
                    :, 0
                ]
                own_values = [float(value) for value in own.detach().cpu().tolist()]
                crossed_values = [
                    float(value) for value in crossed.detach().cpu().tolist()
                ]
                own_log_probabilities.extend(own_values)
                crossed_log_probabilities.extend(crossed_values)
                gain = (
                    math.fsum(
                        left - right
                        for left, right in zip(own_values, crossed_values, strict=True)
                    )
                    / CONTINUATION_TOKENS
                )
                gains.append(gain)
                direction = batch[offset]
                trace.update(
                    struct.pack(
                        ">IB",
                        direction.pair_index,
                        0 if direction.side == "left" else 1,
                    )
                )
                for left, right in zip(own_values, crossed_values, strict=True):
                    trace.update(struct.pack("<dd", left, right))
                paired_delta = float(
                    (suffix_logits[own_row] - suffix_logits[crossed_row])
                    .abs()
                    .max()
                    .detach()
                    .cpu()
                )
                maximum_delta = max(maximum_delta, paired_delta)

    if len(gains) != DIRECTION_COUNT:
        raise RuntimeError("prompt-conditioning scorer did not cover every direction")
    own_nll = -math.fsum(own_log_probabilities) / SCORED_TARGET_TOKENS
    crossed_nll = -math.fsum(crossed_log_probabilities) / SCORED_TARGET_TOKENS
    mean_gain = math.fsum(gains) / DIRECTION_COUNT
    if not all(
        math.isfinite(value)
        for value in (own_nll, crossed_nll, mean_gain, maximum_delta)
    ):
        raise ValueError("prompt-conditioning aggregate is nonfinite")
    return PromptConditioningScore(
        attention_off=attention_off,
        directions=DIRECTION_COUNT,
        scored_target_tokens=SCORED_TARGET_TOKENS,
        mean_gain_nats_per_token=mean_gain,
        wins=sum(gain > 0.0 for gain in gains),
        own_nll_nats_per_token=own_nll,
        crossed_nll_nats_per_token=crossed_nll,
        maximum_paired_logits_delta=maximum_delta,
        forbidden_reads=forbidden_reads,
        scored_logprob_trace_cid=f"blake3:{trace.hexdigest()}",
    )


@dataclass(frozen=True, slots=True)
class PromptConditioningDecision:
    verdict: str
    population_cid: str
    reveal_cid: str
    baseline_artifact_cid: str
    candidate_artifact_cid: str
    baseline: PromptConditioningScore
    candidate: PromptConditioningScore
    baseline_state_off: PromptConditioningScore
    candidate_state_off: PromptConditioningScore
    gates: Mapping[str, bool]

    def record(self) -> dict[str, Any]:
        return {
            "schema": DECISION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "population_cid": self.population_cid,
            "reveal_cid": self.reveal_cid,
            "artifacts": {
                "baseline": self.baseline_artifact_cid,
                "candidate": self.candidate_artifact_cid,
            },
            "formula": (
                "g_d=(log P(y_d|p_d)-log P(y_d|paired_prompt_d))/16; "
                "G=mean over 512 bidirectional contrasts"
            ),
            "thresholds": {
                "candidate_mean_gain_nats_per_token": ABSOLUTE_GAIN_THRESHOLD,
                "candidate_minus_baseline_gain_nats_per_token": (
                    CAPACITY_GAIN_THRESHOLD
                ),
                "candidate_directional_wins": WIN_THRESHOLD,
                "state_off_tolerance": STATE_OFF_TOLERANCE,
                "candidate_own_nll_must_not_exceed_baseline": True,
            },
            "baseline": self.baseline.record(),
            "candidate": self.candidate.record(),
            "baseline_state_off": self.baseline_state_off.record(),
            "candidate_state_off": self.candidate_state_off.record(),
            "gates": dict(self.gates),
            "verdict": self.verdict,
        }


def _scores_equal(
    primary: PromptConditioningScore,
    replay: PromptConditioningScore,
) -> bool:
    return primary.record() == replay.record()


def evaluate_prompt_conditioning(
    *,
    population: PromptConditioningPopulation,
    reveal_cid: str,
    baseline_artifact_cid: str,
    candidate_artifact_cid: str,
    baseline_factory: Callable[[], Any],
    candidate_factory: Callable[[], Any],
    direction_batch_size: int = 8,
    device: torch.device | str = "cpu",
) -> PromptConditioningDecision:
    """Run enabled replay and state-off controls, then apply the frozen verdict."""

    bound_reveal_cid = _artifact_cid(reveal_cid, label="reveal CID")
    bound_baseline_cid = _artifact_cid(
        baseline_artifact_cid,
        label="baseline artifact CID",
    )
    bound_candidate_cid = _artifact_cid(
        candidate_artifact_cid,
        label="candidate artifact CID",
    )

    score_arguments = {
        "population": population,
        "direction_batch_size": direction_batch_size,
        "device": device,
    }
    baseline = score_prompt_conditioning(
        baseline_factory(),
        attention_off=False,
        **score_arguments,
    )
    baseline_replay = score_prompt_conditioning(
        baseline_factory(),
        attention_off=False,
        **score_arguments,
    )
    candidate = score_prompt_conditioning(
        candidate_factory(),
        attention_off=False,
        **score_arguments,
    )
    candidate_replay = score_prompt_conditioning(
        candidate_factory(),
        attention_off=False,
        **score_arguments,
    )
    baseline_state_off = score_prompt_conditioning(
        baseline_factory(),
        attention_off=True,
        **score_arguments,
    )
    candidate_state_off = score_prompt_conditioning(
        candidate_factory(),
        attention_off=True,
        **score_arguments,
    )

    baseline_replay_exact = _scores_equal(baseline, baseline_replay)
    candidate_replay_exact = _scores_equal(candidate, candidate_replay)
    forbidden_reads_zero = all(
        score.forbidden_reads == 0
        for score in (
            baseline,
            baseline_replay,
            candidate,
            candidate_replay,
            baseline_state_off,
            candidate_state_off,
        )
    )
    baseline_state_off_collapsed = (
        abs(baseline_state_off.mean_gain_nats_per_token) <= STATE_OFF_TOLERANCE
        and baseline_state_off.maximum_paired_logits_delta <= STATE_OFF_TOLERANCE
    )
    candidate_state_off_collapsed = (
        abs(candidate_state_off.mean_gain_nats_per_token) <= STATE_OFF_TOLERANCE
        and candidate_state_off.maximum_paired_logits_delta <= STATE_OFF_TOLERANCE
    )
    candidate_absolute_gain = (
        candidate.mean_gain_nats_per_token >= ABSOLUTE_GAIN_THRESHOLD
    )
    candidate_capacity_gain = (
        candidate.mean_gain_nats_per_token - baseline.mean_gain_nats_per_token
        >= CAPACITY_GAIN_THRESHOLD
    )
    candidate_any_gain = (
        candidate.mean_gain_nats_per_token > baseline.mean_gain_nats_per_token
    )
    candidate_wins = candidate.wins >= WIN_THRESHOLD
    own_nll_nonregression = (
        candidate.own_nll_nats_per_token <= baseline.own_nll_nats_per_token
    )
    gates = {
        "baseline_replay_exact": baseline_replay_exact,
        "candidate_replay_exact": candidate_replay_exact,
        "forbidden_reads_zero": forbidden_reads_zero,
        "baseline_state_off_collapsed": baseline_state_off_collapsed,
        "candidate_state_off_collapsed": candidate_state_off_collapsed,
        "candidate_absolute_gain": candidate_absolute_gain,
        "candidate_capacity_gain": candidate_capacity_gain,
        "candidate_any_gain": candidate_any_gain,
        "candidate_directional_wins": candidate_wins,
        "candidate_own_nll_nonregression": own_nll_nonregression,
    }
    valid = all(
        gates[name]
        for name in (
            "baseline_replay_exact",
            "candidate_replay_exact",
            "forbidden_reads_zero",
            "baseline_state_off_collapsed",
            "candidate_state_off_collapsed",
        )
    )
    if not valid:
        verdict = VERDICT_INVALID
    elif all(
        gates[name]
        for name in (
            "candidate_absolute_gain",
            "candidate_capacity_gain",
            "candidate_directional_wins",
            "candidate_own_nll_nonregression",
        )
    ):
        verdict = VERDICT_PASS
    elif all(
        gates[name]
        for name in (
            "candidate_absolute_gain",
            "candidate_directional_wins",
            "candidate_own_nll_nonregression",
        )
    ):
        verdict = VERDICT_ABSOLUTE_NO_CAPACITY_GAIN
    elif candidate_any_gain and own_nll_nonregression:
        verdict = VERDICT_PARTIAL
    else:
        verdict = VERDICT_FAIL

    return PromptConditioningDecision(
        verdict=verdict,
        population_cid=population.population_cid,
        reveal_cid=bound_reveal_cid,
        baseline_artifact_cid=bound_baseline_cid,
        candidate_artifact_cid=bound_candidate_cid,
        baseline=baseline,
        candidate=candidate,
        baseline_state_off=baseline_state_off,
        candidate_state_off=candidate_state_off,
        gates=gates,
    )
