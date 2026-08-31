"""Create-once corpus construction for the frozen #1019 capacity rung.

The training stream starts at the canonical beginning of the existing train
split.  Development and confirmation start strictly after #1017's last
source-story ordinal.  The full #1017 manifest is verified, but none of its
sealed token, index, or prompt files is opened.
"""

from __future__ import annotations

import json
import os
import shutil
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from blake3 import blake3

from .constants import (
    CAPACITY_MODEL_CONFIG,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
    TINYSTORIES_BYTES,
    TINYSTORIES_FILENAME,
    TINYSTORIES_REPOSITORY,
    TINYSTORIES_REVISION,
    TINYSTORIES_SHA256,
    TINYSTORIES_URL,
)
from .continuation_data import _capped_ids, _prompt_candidate
from .data import (
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
    IndexWriter,
    StorySummary,
    U16Writer,
    iter_canonical_stories,
    story_split,
    validate_tokenizer_json,
    verify_source,
)
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    verify_artifact_subset,
    verify_manifest_envelope,
    write_bound_manifest,
)


ISSUE = 1019
CAPACITY_DATASET_MANIFEST_NAME = "capacity-dataset-manifest.json"
CAPACITY_TRAINING_VIEW_MANIFEST_NAME = "capacity-training-view-manifest.json"
CAPACITY_DATASET_MANIFEST_SCHEMA = "uor-r4-softmax-trainer-capacity-dataset/1"
CAPACITY_TRAINING_VIEW_MANIFEST_SCHEMA = (
    "uor-r4-softmax-trainer-capacity-training-view/1"
)
SEALED_DENIAL_SCHEMA = "uor-r4-softmax-trainer-capacity-sealed-denial/1"
SEALED_PROMPT_SCHEMA = "uor-r4-softmax-trainer-capacity-sealed-prompts/1"

PREDECESSOR_DATASET_MANIFEST_CID = (
    "blake3:5f709a9ef886801c55c799cd6f684774dc87a3e9e192f148d254c3d20a394aec"
)
PREDECESSOR_SPLIT_POLICY_CID = (
    "blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa"
)
PREDECESSOR_DEV_LAST_SOURCE_STORY_ORDINAL = 47_293
PREDECESSOR_TEST_LAST_SOURCE_STORY_ORDINAL = 48_856
TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)

TRAIN_TOKEN_CAP = 275_251_200
DEV_TOKEN_CAP = 250_000
TEST_TOKEN_CAP = 249_880
TOKEN_CAPS = {"train": TRAIN_TOKEN_CAP, "dev": DEV_TOKEN_CAP, "test": TEST_TOKEN_CAP}

TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"
TOKEN_RELATIVE_PATHS = {
    "train": "tokens/train.u16",
    "dev": "tokens/dev.u16",
    "test": "sealed-confirmation/test.u16",
}
INDEX_RELATIVE_PATHS = {
    "train": "indexes/train.jsonl",
    "dev": "indexes/dev.jsonl",
    "test": "sealed-confirmation/test-index.jsonl",
}
SEALED_DIRECTORY_RELATIVE_PATH = "sealed-confirmation"
SEALED_PROMPT_RELATIVE_PATH = "sealed-confirmation/prompts.json"
SEALED_DENIAL_RELATIVE_PATH = "preflight/sealed-confirmation-denial.json"

PREVIOUS_PROMPT_CIDS = frozenset(
    {
        "blake3:0000268631c8e69c7b66380e559bcf9b3cba35d034c3b8fea4d91a8995722d45",
        "blake3:000041b3c7fd5fc959421bcc59cf4b4bc5051f49f1e943d3938f31f2d5110e7d",
        "blake3:00006032bb21c0ed1ff8902efb50793bcd596341fcc6380f35aa7d2b33929e88",
        "blake3:0000ad9ff89a4a7cd105828d8e1ae952b49889444b3b3928fa3973e1ae7fe3d3",
        "blake3:00010aef5fb4f8edc9f7f9b29d644b4ab9c278bccd53b47b9584b2e01aca14ef",
        "blake3:000272da1d524963f9965109510d8529709b389d80815078e83f1c53ac696bf3",
        "blake3:0002c6820c016981b3621b54c0e2f18f7c9ad5cde09be614592301a620e20988",
        "blake3:00036e360a675dfa525138bd1375e3965f57d36345f50f221ccaed083f5c43cb",
        "blake3:00042fc3d212c3dbd2801d6a9bfbe51bdc30a2e97ce15f49ce537c72a7eb57ac",
        "blake3:0004586ddfce6012b8e733752f6f051ea0e7d5675ca47e0e83d2f7dcd998e139",
    }
)


class CapacityPopulationUnavailable(RuntimeError):
    terminal = "UNAVAILABLE_CAPACITY_POPULATION"


@dataclass(slots=True)
class _SplitState:
    summary: StorySummary = field(default_factory=StorySummary)
    stories: int = 0
    split_story_ordinal: int = 0
    first_story_cid: str | None = None
    last_story_cid: str | None = None
    first_source_story_ordinal: int | None = None
    last_source_story_ordinal: int | None = None
    truncated_final_story: bool = False
    cap_padding_tokens: int = 0


def _copy_verified(source: Path, destination: Path, expected_cid: str) -> None:
    if cid_file(source) != expected_cid:
        raise ValueError(f"source artifact CID mismatch: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.part")
    try:
        shutil.copyfile(source, temporary)
        if cid_file(temporary) != expected_cid:
            raise ValueError(f"copied artifact CID mismatch: {destination}")
        os.replace(temporary, destination)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def _consider_prompt(
    selected: list[dict[str, Any]], tokenizer: Any, story_cid: str, story: bytes
) -> None:
    if story_cid in PREVIOUS_PROMPT_CIDS:
        return
    if len(selected) == SEALED_PROMPT_COUNT and story_cid >= selected[-1]["story_cid"]:
        return
    candidate = _prompt_candidate(tokenizer, story_cid, story)
    if candidate is None:
        return
    selected.append(candidate)
    selected.sort(key=lambda record: str(record["story_cid"]))
    del selected[SEALED_PROMPT_COUNT:]


def _prior_boundaries(predecessor: dict[str, Any]) -> dict[str, int]:
    if predecessor.get("manifest_cid") != PREDECESSOR_DATASET_MANIFEST_CID:
        raise ValueError("#1017 dataset manifest differs from the frozen predecessor")
    if predecessor.get("split_policy_cid") != PREDECESSOR_SPLIT_POLICY_CID:
        raise ValueError("#1017 split policy differs from the frozen predecessor")
    splits = predecessor.get("splits")
    if not isinstance(splits, dict):
        raise ValueError("#1017 dataset manifest has no split records")
    boundaries: dict[str, int] = {"train": -1}
    for name in ("dev", "test"):
        split = splits.get(name)
        if not isinstance(split, dict) or not isinstance(
            split.get("last_source_story_ordinal"), int
        ):
            raise ValueError(f"#1017 {name} boundary is unavailable")
        boundaries[name] = int(split["last_source_story_ordinal"])
    if boundaries != {
        "train": -1,
        "dev": PREDECESSOR_DEV_LAST_SOURCE_STORY_ORDINAL,
        "test": PREDECESSOR_TEST_LAST_SOURCE_STORY_ORDINAL,
    }:
        raise ValueError("#1017 frozen source-story boundaries differ")
    return boundaries


def _manifest_paths(manifest: dict[str, Any], *, label: str) -> set[str]:
    records = manifest.get("artifacts")
    if not isinstance(records, list) or not all(
        isinstance(record, dict) and isinstance(record.get("path"), str)
        for record in records
    ):
        raise ValueError(f"{label} has malformed artifact records")
    paths = [str(record["path"]) for record in records]
    if len(paths) != len(set(paths)):
        raise ValueError(f"{label} repeats an artifact path")
    return set(paths)


def _validate_dataset_envelope(manifest: dict[str, Any]) -> None:
    predecessor = manifest.get("predecessor")
    source = manifest.get("source")
    splits = manifest.get("splits")
    budget = manifest.get("sealed_confirmation_budget")
    freshness = manifest.get("freshness")
    if not all(
        isinstance(value, dict)
        for value in (predecessor, source, splits, budget, freshness)
    ):
        raise ValueError("#1019 dataset omits a required object")
    expected_paths = {
        TOKENIZER_RELATIVE_PATH,
        *TOKEN_RELATIVE_PATHS.values(),
        *INDEX_RELATIVE_PATHS.values(),
        SEALED_PROMPT_RELATIVE_PATH,
    }
    if (
        manifest.get("schema") != CAPACITY_DATASET_MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or predecessor.get("issue") != 1017
        or predecessor.get("dataset_manifest_cid")
        != PREDECESSOR_DATASET_MANIFEST_CID
        or predecessor.get("split_policy_cid") != PREDECESSOR_SPLIT_POLICY_CID
        or predecessor.get("dev_last_source_story_ordinal")
        != PREDECESSOR_DEV_LAST_SOURCE_STORY_ORDINAL
        or predecessor.get("test_last_source_story_ordinal")
        != PREDECESSOR_TEST_LAST_SOURCE_STORY_ORDINAL
        or predecessor.get("sealed_artifact_reads") != 0
        or source.get("repository") != TINYSTORIES_REPOSITORY
        or source.get("revision") != TINYSTORIES_REVISION
        or source.get("filename") != TINYSTORIES_FILENAME
        or source.get("url") != TINYSTORIES_URL
        or source.get("bytes") != TINYSTORIES_BYTES
        or source.get("sha256") != TINYSTORIES_SHA256
        or not isinstance(source.get("stories_examined"), int)
        or source.get("stories_examined", 0) <= 0
        or manifest.get("split_policy_cid") != PREDECESSOR_SPLIT_POLICY_CID
        or cid_bytes(canonical_json_bytes(manifest.get("split_policy")))
        != PREDECESSOR_SPLIT_POLICY_CID
        or manifest.get("model_contract") != CAPACITY_MODEL_CONFIG.as_contract()
        or manifest.get("tokenizer_cid") != TOKENIZER_CID
        or budget
        != {
            "scored_store_token_ids": TEST_TOKEN_CAP,
            "prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_token_ids": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
            "hard_cap": 250_000,
        }
        or freshness.get("training_population")
        != "canonical train split from its beginning"
        or freshness.get("development_and_test")
        != "strictly after content-bound #1017 source ordinals"
        or freshness.get("excluded_published_prompt_cids")
        != sorted(PREVIOUS_PROMPT_CIDS)
        or freshness.get("excluded_published_prompt_story_count")
        != len(PREVIOUS_PROMPT_CIDS)
        or freshness.get("predecessor_sealed_paths_opened") != 0
        or _manifest_paths(manifest, label="#1019 dataset") != expected_paths
    ):
        raise ValueError("#1019 dataset identity differs")
    for name, token_cap in TOKEN_CAPS.items():
        split = splits.get(name)
        if not isinstance(split, dict):
            raise ValueError(f"#1019 {name} split is missing")
        expected_scored = (
            (token_cap - 1) // CAPACITY_MODEL_CONFIG.max_position_embeddings
        ) * CAPACITY_MODEL_CONFIG.max_position_embeddings
        prior = predecessor.get(f"{name}_last_source_story_ordinal")
        if name == "train":
            prior = None
        first = split.get("first_source_story_ordinal")
        last = split.get("last_source_story_ordinal")
        if (
            split.get("tokens") != token_cap
            or split.get("token_cap") != token_cap
            or not isinstance(split.get("stories"), int)
            or split.get("stories", 0) <= 0
            or not isinstance(first, int)
            or not isinstance(last, int)
            or first > last
            or split.get("predecessor_last_source_story_ordinal") != prior
            or split.get("source_ordinal_disjoint") is not True
            or split.get("complete_context_scored_next_tokens") != expected_scored
        ):
            raise ValueError(f"#1019 {name} split identity differs")
        if name in {"dev", "test"} and (not isinstance(prior, int) or first <= prior):
            raise ValueError(f"#1019 {name} overlaps its predecessor")


def _validate_training_view_envelope(
    manifest: dict[str, Any], dataset: dict[str, Any]
) -> set[str]:
    """Validate signed training-view semantics without requiring sealed mode 000."""
    _validate_dataset_envelope(dataset)
    commitment = manifest.get("sealed_confirmation_commitment")
    expected_training_paths = {
        TOKENIZER_RELATIVE_PATH,
        TOKEN_RELATIVE_PATHS["train"],
        TOKEN_RELATIVE_PATHS["dev"],
        INDEX_RELATIVE_PATHS["train"],
        INDEX_RELATIVE_PATHS["dev"],
        SEALED_DENIAL_RELATIVE_PATH,
    }
    dataset_records = {
        str(record["path"]): record for record in dataset.get("artifacts", [])
    }
    observed_training_paths = _manifest_paths(
        manifest, label="#1019 training view"
    )
    training_records = {
        str(record["path"]): record for record in manifest.get("artifacts", [])
    }
    shared_nonsealed_paths = {
        TOKENIZER_RELATIVE_PATH,
        TOKEN_RELATIVE_PATHS["train"],
        TOKEN_RELATIVE_PATHS["dev"],
        INDEX_RELATIVE_PATHS["train"],
        INDEX_RELATIVE_PATHS["dev"],
    }
    sealed_paths = [
        TOKEN_RELATIVE_PATHS["test"],
        INDEX_RELATIVE_PATHS["test"],
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    if (
        manifest.get("schema") != CAPACITY_TRAINING_VIEW_MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("dataset_manifest_cid") != dataset["manifest_cid"]
        or manifest.get("predecessor_dataset_manifest_cid")
        != PREDECESSOR_DATASET_MANIFEST_CID
        or manifest.get("split_policy") != dataset.get("split_policy")
        or manifest.get("split_policy_cid") != PREDECESSOR_SPLIT_POLICY_CID
        or manifest.get("model_contract") != CAPACITY_MODEL_CONFIG.as_contract()
        or manifest.get("tokenizer_cid") != TOKENIZER_CID
        or not isinstance(commitment, dict)
        or commitment.get("tokens") != TEST_TOKEN_CAP
        or commitment.get("prompt_tokens") != SEALED_PROMPT_TOKEN_COUNT
        or commitment.get("total_reveal_tokens")
        != TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT
        or commitment.get("artifacts")
        != [dataset_records[path] for path in sealed_paths]
        or commitment.get("access_policy")
        != "directory mode 000 until create-once reveal marker"
        or observed_training_paths != expected_training_paths
        or any(
            training_records.get(path) != dataset_records[path]
            for path in shared_nonsealed_paths
        )
    ):
        raise ValueError("#1019 training-view identity differs")
    return expected_training_paths


def _write_story(
    *,
    writer: U16Writer,
    index: IndexWriter,
    state: _SplitState,
    token_ids: list[int],
    story: bytes,
    story_cid: str,
    source_ordinal: int,
    split_ordinal: int,
    cap: int,
) -> bool:
    remaining = cap - writer.count
    if remaining <= 0:
        return False
    next_ids, consumed_story = _capped_ids(token_ids, remaining)
    if not consumed_story:
        writer.append(next_ids)
        state.cap_padding_tokens += len(next_ids)
        return False
    truncated = len(next_ids) < len(token_ids)
    offset = writer.count
    writer.append(next_ids)
    state.summary.add(story)
    state.stories += 1
    state.first_story_cid = state.first_story_cid or story_cid
    state.last_story_cid = story_cid
    state.first_source_story_ordinal = (
        source_ordinal
        if state.first_source_story_ordinal is None
        else state.first_source_story_ordinal
    )
    state.last_source_story_ordinal = source_ordinal
    state.truncated_final_story |= truncated
    index.append(
        {
            "content_token_count": max(0, len(next_ids) - 2),
            "content_token_offset": offset + 1,
            "capacity_story_ordinal": state.stories - 1,
            "source_story_ordinal": source_ordinal,
            "split_story_ordinal": split_ordinal,
            "story_cid": story_cid,
            "story_token_count": len(next_ids),
            "story_token_offset": offset,
            "truncated": truncated,
        }
    )
    return True


def _build_stores(
    source: Path,
    tokenizer: Any,
    root: Path,
    *,
    predecessor_boundaries: dict[str, int],
) -> dict[str, Any]:
    writers = {name: U16Writer(root / path) for name, path in TOKEN_RELATIVE_PATHS.items()}
    indexes = {name: IndexWriter(root / path) for name, path in INDEX_RELATIVE_PATHS.items()}
    states = {name: _SplitState() for name in TOKEN_RELATIVE_PATHS}
    selected_prompts: list[dict[str, Any]] = []
    full_test_stories = 0
    full_test_story_bytes = 0
    ordered_test_cids = blake3()
    excluded_previous_prompt_cids: set[str] = set()
    examined = 0
    try:
        for source_ordinal, story in enumerate(iter_canonical_stories(source)):
            examined += 1
            split = story_split(story)
            state = states[split]
            split_ordinal = state.split_story_ordinal
            state.split_story_ordinal += 1
            story_digest = blake3(story).digest()
            story_cid = f"blake3:{story_digest.hex()}"
            if split == "test":
                full_test_stories += 1
                full_test_story_bytes += len(story)
                ordered_test_cids.update(story_digest)

            if story_cid in PREVIOUS_PROMPT_CIDS:
                excluded_previous_prompt_cids.add(story_cid)
                continue

            if split in {"dev", "test"} and source_ordinal <= predecessor_boundaries[split]:
                continue
            writer = writers[split]
            if writer.count >= TOKEN_CAPS[split]:
                if split == "test":
                    _consider_prompt(selected_prompts, tokenizer, story_cid, story)
                continue

            text = story.decode("utf-8", errors="strict")
            content_ids = tokenizer.encode(text, add_special_tokens=False).ids
            token_ids = [BOS_TOKEN_ID, *content_ids, EOS_TOKEN_ID]
            consumed = _write_story(
                writer=writer,
                index=indexes[split],
                state=state,
                token_ids=token_ids,
                story=story,
                story_cid=story_cid,
                source_ordinal=source_ordinal,
                split_ordinal=split_ordinal,
                cap=TOKEN_CAPS[split],
            )
            if split == "test" and writer.count >= TEST_TOKEN_CAP and not consumed:
                _consider_prompt(selected_prompts, tokenizer, story_cid, story)

        counts = {name: writer.count for name, writer in writers.items()}
        if counts != TOKEN_CAPS or len(selected_prompts) != SEALED_PROMPT_COUNT:
            raise CapacityPopulationUnavailable(
                "UNAVAILABLE_CAPACITY_POPULATION: pinned source cannot satisfy exact caps/prompts"
            )
        if excluded_previous_prompt_cids != PREVIOUS_PROMPT_CIDS:
            raise CapacityPopulationUnavailable(
                "UNAVAILABLE_CAPACITY_POPULATION: pinned source does not contain every prior prompt"
            )
        for writer in writers.values():
            writer.finish()
        for index in indexes.values():
            index.finish()
        prompt_fixture: dict[str, Any] = {
            "schema": SEALED_PROMPT_SCHEMA,
            "issue": ISSUE,
            "selection": (
                "first 24 content tokens of the five lowest eligible test-story CIDs "
                "strictly after the #1019 NLL-store boundary"
            ),
            "excluded_previous_prompt_cids": sorted(PREVIOUS_PROMPT_CIDS),
            "revealed_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "tokenizer_cid": TOKENIZER_CID,
            "prompts": selected_prompts,
        }
        prompt_fixture["fixture_cid"] = cid_bytes(canonical_json_bytes(prompt_fixture))
        atomic_write_json(root / SEALED_PROMPT_RELATIVE_PATH, prompt_fixture)
    except BaseException:
        for writer in writers.values():
            if not writer.handle.closed:
                writer.abort()
        for index in indexes.values():
            if not index.handle.closed:
                index.abort()
        raise

    split_payload: dict[str, Any] = {}
    for name in ("train", "dev", "test"):
        state = states[name]
        if state.first_source_story_ordinal is None or state.last_source_story_ordinal is None:
            raise CapacityPopulationUnavailable(f"{name} population has no complete story range")
        prior_boundary = predecessor_boundaries[name]
        if name in {"dev", "test"} and state.first_source_story_ordinal <= prior_boundary:
            raise ValueError(f"{name} capacity population overlaps #1017")
        split_payload[name] = {
            "tokens": writers[name].count,
            "token_cap": TOKEN_CAPS[name],
            "stories": state.stories,
            "story_bytes": state.summary.story_bytes,
            "ordered_story_bytes_cid": f"blake3:{state.summary._digest.hexdigest()}",
            "first_story_cid": state.first_story_cid,
            "last_story_cid": state.last_story_cid,
            "first_source_story_ordinal": state.first_source_story_ordinal,
            "last_source_story_ordinal": state.last_source_story_ordinal,
            "predecessor_last_source_story_ordinal": prior_boundary if prior_boundary >= 0 else None,
            "source_ordinal_disjoint": name == "train" or state.first_source_story_ordinal > prior_boundary,
            "cap_padding_tokens": state.cap_padding_tokens,
            "truncated_final_story": state.truncated_final_story,
            "complete_context_scored_next_tokens": (
                (writers[name].count - 1) // CAPACITY_MODEL_CONFIG.max_position_embeddings
            )
            * CAPACITY_MODEL_CONFIG.max_position_embeddings,
            "boundary_policy": (
                "train starts at canonical train-split beginning; dev/test start after "
                "the content-bound #1017 last source-story ordinal"
            ),
        }
    return {
        "splits": split_payload,
        "stories_examined": examined,
        "excluded_previous_prompt_cids": sorted(excluded_previous_prompt_cids),
        "full_snapshot_test_population": {
            "stories": full_test_stories,
            "story_bytes": full_test_story_bytes,
            "ordered_full_story_cids_cid": f"blake3:{ordered_test_cids.hexdigest()}",
        },
    }


def _deny_sealed(root: Path, *, dataset_manifest_cid: str) -> dict[str, Any]:
    directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    directory.chmod(0)
    denied = False
    try:
        with (root / TOKEN_RELATIVE_PATHS["test"]).open("rb"):
            pass
    except PermissionError:
        denied = True
    if not denied:
        directory.chmod(0o700)
        raise RuntimeError("#1019 sealed-confirmation permission denial did not hold")
    value: dict[str, Any] = {
        "schema": SEALED_DENIAL_SCHEMA,
        "issue": ISSUE,
        "dataset_manifest_cid": dataset_manifest_cid,
        "directory": SEALED_DIRECTORY_RELATIVE_PATH,
        "directory_mode": "000",
        "read_attempt": "PERMISSION_DENIED",
        "sealed_paths": [
            TOKEN_RELATIVE_PATHS["test"],
            INDEX_RELATIVE_PATHS["test"],
            SEALED_PROMPT_RELATIVE_PATH,
        ],
        "training_or_selection_reads": 0,
        "prior_sealed_artifact_reads": 0,
    }
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(root / SEALED_DENIAL_RELATIVE_PATH, value)
    return value


def open_sealed_confirmation(root: Path) -> dict[str, Any]:
    load_capacity_training_view_manifest(root)
    denial = json.loads((root / SEALED_DENIAL_RELATIVE_PATH).read_text(encoding="utf-8"))
    unsigned = dict(denial)
    expected = unsigned.pop("result_cid", None)
    if (
        denial.get("schema") != SEALED_DENIAL_SCHEMA
        or expected != cid_bytes(canonical_json_bytes(unsigned))
        or denial.get("read_attempt") != "PERMISSION_DENIED"
        or denial.get("training_or_selection_reads") != 0
        or denial.get("prior_sealed_artifact_reads") != 0
    ):
        raise ValueError("#1019 sealed denial record does not reproduce")
    directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    if directory.stat().st_mode & 0o777 != 0:
        raise ValueError("#1019 sealed confirmation was unlocked before reveal")
    directory.chmod(0o700)
    return denial


def prepare_capacity_dataset(
    root: Path,
    *,
    predecessor_root: Path,
    source: Path | None = None,
    force: bool = False,
) -> dict[str, Any]:
    """Build #1019 while reading only #1017's public manifest and tokenizer."""
    from tokenizers import Tokenizer

    root = root.resolve()
    predecessor_root = predecessor_root.resolve()
    if force:
        raise ValueError(
            "#1019 population is create-once; use a new empty root instead of --force"
        )
    if root == predecessor_root:
        raise ValueError("#1019 root must differ from immutable #1017")
    manifest_path = root / CAPACITY_DATASET_MANIFEST_NAME
    training_view_path = root / CAPACITY_TRAINING_VIEW_MANIFEST_NAME
    if manifest_path.is_file() and training_view_path.is_file():
        dataset = verify_manifest_envelope(manifest_path)
        _validate_dataset_envelope(dataset)
        return {
            "dataset": dataset,
            "training_view": load_capacity_training_view_manifest(root),
        }
    sealed_directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    derived_roots = [
        root / "tokens",
        root / "indexes",
        root / "preflight",
        root / "checkpoints",
        root / "selection",
        root / "qualification",
        root / "reveal",
        root / "generations",
        root / "final",
        root / "export",
        sealed_directory,
        manifest_path,
        training_view_path,
    ]
    if any(path.exists() or path.is_symlink() for path in derived_roots):
        raise FileExistsError(
            "#1019 root contains partial or downstream evidence; use a new empty root"
        )

    predecessor = verify_manifest_envelope(
        predecessor_root / "continuation-dataset-manifest.json"
    )
    boundaries = _prior_boundaries(predecessor)
    source_path = (source or predecessor_root / "raw" / TINYSTORIES_FILENAME).resolve()
    if not source_path.is_file():
        fallback = predecessor_root.parent / "issue-1014" / "raw" / TINYSTORIES_FILENAME
        source_path = fallback.resolve()
    verify_source(source_path)
    tokenizer_source = predecessor_root / "tokenizer/tokenizer.json"
    root.mkdir(parents=True, exist_ok=True)
    _copy_verified(tokenizer_source, root / TOKENIZER_RELATIVE_PATH, TOKENIZER_CID)
    validate_tokenizer_json(root / TOKENIZER_RELATIVE_PATH)
    tokenizer = Tokenizer.from_file(str(root / TOKENIZER_RELATIVE_PATH))
    try:
        built = _build_stores(
            source_path,
            tokenizer,
            root,
            predecessor_boundaries=boundaries,
        )
        if built["full_snapshot_test_population"] != predecessor.get(
            "full_snapshot_test_population"
        ):
            raise ValueError("pinned source/split full test population does not reproduce")

        payload: dict[str, Any] = {
            "schema": CAPACITY_DATASET_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "predecessor": {
                "issue": 1017,
                "dataset_manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
                "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
                "dev_last_source_story_ordinal": boundaries["dev"],
                "test_last_source_story_ordinal": boundaries["test"],
                "sealed_artifact_reads": 0,
            },
            "source": {
                "repository": TINYSTORIES_REPOSITORY,
                "revision": TINYSTORIES_REVISION,
                "filename": TINYSTORIES_FILENAME,
                "url": TINYSTORIES_URL,
                "bytes": TINYSTORIES_BYTES,
                "sha256": TINYSTORIES_SHA256,
                "stories_examined": built["stories_examined"],
            },
            "split_policy": predecessor["split_policy"],
            "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
            "model_contract": CAPACITY_MODEL_CONFIG.as_contract(),
            "tokenizer_cid": TOKENIZER_CID,
            "splits": built["splits"],
            "sealed_confirmation_budget": {
                "scored_store_token_ids": TEST_TOKEN_CAP,
                "prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
                "total_revealed_token_ids": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
                "hard_cap": 250_000,
            },
            "full_snapshot_test_population": built["full_snapshot_test_population"],
            "freshness": {
                "training_population": "canonical train split from its beginning",
                "development_and_test": (
                    "strictly after content-bound #1017 source ordinals"
                ),
                "excluded_published_prompt_cids": built[
                    "excluded_previous_prompt_cids"
                ],
                "excluded_published_prompt_story_count": len(
                    built["excluded_previous_prompt_cids"]
                ),
                "predecessor_sealed_paths_opened": 0,
            },
        }
        all_paths = [
            TOKENIZER_RELATIVE_PATH,
            *TOKEN_RELATIVE_PATHS.values(),
            *INDEX_RELATIVE_PATHS.values(),
            SEALED_PROMPT_RELATIVE_PATH,
        ]
        dataset = write_bound_manifest(
            manifest_path,
            payload,
            artifact_root=root,
            relative_paths=all_paths,
        )
        denial = _deny_sealed(root, dataset_manifest_cid=str(dataset["manifest_cid"]))
        records = {str(record["path"]): record for record in dataset["artifacts"]}
        sealed_paths = [
            TOKEN_RELATIVE_PATHS["test"],
            INDEX_RELATIVE_PATHS["test"],
            SEALED_PROMPT_RELATIVE_PATH,
        ]
        training_paths = [
            TOKENIZER_RELATIVE_PATH,
            TOKEN_RELATIVE_PATHS["train"],
            TOKEN_RELATIVE_PATHS["dev"],
            INDEX_RELATIVE_PATHS["train"],
            INDEX_RELATIVE_PATHS["dev"],
            SEALED_DENIAL_RELATIVE_PATH,
        ]
        training_view = write_bound_manifest(
            training_view_path,
            {
                "schema": CAPACITY_TRAINING_VIEW_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "dataset_manifest_cid": dataset["manifest_cid"],
                "predecessor_dataset_manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
                "split_policy": predecessor["split_policy"],
                "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
                "model_contract": CAPACITY_MODEL_CONFIG.as_contract(),
                "tokenizer_cid": TOKENIZER_CID,
                "sealed_confirmation_commitment": {
                    "tokens": TEST_TOKEN_CAP,
                    "prompt_tokens": SEALED_PROMPT_TOKEN_COUNT,
                    "total_reveal_tokens": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
                    "artifacts": [records[path] for path in sealed_paths],
                    "access_policy": "directory mode 000 until create-once reveal marker",
                    "denial_result_cid": denial["result_cid"],
                },
            },
            artifact_root=root,
            relative_paths=training_paths,
        )
        return {"dataset": dataset, "training_view": training_view}
    except BaseException:
        if sealed_directory.exists() and not sealed_directory.is_symlink():
            sealed_directory.chmod(0)
        raise


def load_capacity_dataset_manifest(root: Path) -> dict[str, Any]:
    manifest = verify_manifest_envelope(root / CAPACITY_DATASET_MANIFEST_NAME)
    _validate_dataset_envelope(manifest)
    expected_paths = {
        TOKENIZER_RELATIVE_PATH,
        *TOKEN_RELATIVE_PATHS.values(),
        *INDEX_RELATIVE_PATHS.values(),
        SEALED_PROMPT_RELATIVE_PATH,
    }
    if _manifest_paths(manifest, label="#1019 dataset") != expected_paths:
        raise ValueError("#1019 dataset binds unexpected artifacts")
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=expected_paths,
    )
    return manifest


def load_capacity_training_view_manifest(root: Path) -> dict[str, Any]:
    """Verify only nonsealed #1019 inputs and the physical denial witness."""
    manifest = verify_manifest_envelope(root / CAPACITY_TRAINING_VIEW_MANIFEST_NAME)
    dataset = verify_manifest_envelope(root / CAPACITY_DATASET_MANIFEST_NAME)
    expected_training_paths = _validate_training_view_envelope(manifest, dataset)
    commitment = manifest.get("sealed_confirmation_commitment")
    sealed_paths = [
        TOKEN_RELATIVE_PATHS["test"],
        INDEX_RELATIVE_PATHS["test"],
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=expected_training_paths,
    )
    denial = json.loads((root / SEALED_DENIAL_RELATIVE_PATH).read_text(encoding="utf-8"))
    unsigned = dict(denial)
    expected = unsigned.pop("result_cid", None)
    if (
        denial.get("schema") != SEALED_DENIAL_SCHEMA
        or denial.get("issue") != ISSUE
        or expected != cid_bytes(canonical_json_bytes(unsigned))
        or denial.get("dataset_manifest_cid") != dataset["manifest_cid"]
        or denial.get("directory") != SEALED_DIRECTORY_RELATIVE_PATH
        or denial.get("directory_mode") != "000"
        or denial.get("read_attempt") != "PERMISSION_DENIED"
        or denial.get("sealed_paths") != sealed_paths
        or denial.get("training_or_selection_reads") != 0
        or denial.get("prior_sealed_artifact_reads") != 0
        or commitment.get("denial_result_cid") != denial.get("result_cid")
    ):
        raise ValueError("#1019 sealed denial does not reproduce")
    if (root / SEALED_DIRECTORY_RELATIVE_PATH).stat().st_mode & 0o777 != 0:
        raise ValueError("#1019 sealed confirmation is readable during training")
    return manifest
