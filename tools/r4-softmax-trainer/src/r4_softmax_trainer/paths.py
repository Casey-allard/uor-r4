"""Repository-local default paths for untracked training artifacts."""

from __future__ import annotations

import os
from pathlib import Path


def repository_root(start: Path | None = None) -> Path:
    candidate = (start or Path(__file__)).resolve()
    if candidate.is_file():
        candidate = candidate.parent
    for directory in (candidate, *candidate.parents):
        if (directory / "Cargo.toml").is_file() and (directory / "AGENTS.md").is_file():
            return directory
    raise RuntimeError("could not locate uor-r4 repository root")


def default_research_root() -> Path:
    return repository_root() / ".uor-models" / "research" / "issue-1014"


def default_continuation_root() -> Path:
    return repository_root() / ".uor-models" / "research" / "issue-1017"


def default_capacity_root() -> Path:
    return repository_root() / ".uor-models" / "research" / "issue-1019"


def model_store_root() -> Path:
    """Honor the shared model store when commands run from an isolated worktree."""
    configured = os.environ.get("UOR_MODEL_STORE")
    if configured:
        return Path(configured).expanduser().resolve()
    return repository_root() / ".uor-models"


def default_grounding_predecessor_root() -> Path:
    return model_store_root() / "research" / "issue-1017" / "export"


def default_grounding_root() -> Path:
    return model_store_root() / "research" / "issue-954"


def default_source_span_pointer_root() -> Path:
    return model_store_root() / "research" / "issue-954" / "source-span-pointer"


def default_source_relation_head_root() -> Path:
    return model_store_root() / "research" / "issue-954" / "source-relation-head"


def default_attended_relation_adapter_root() -> Path:
    return model_store_root() / "research" / "issue-954" / "attended-relation-adapter"


def default_joint_candidate_margin_root() -> Path:
    return model_store_root() / "research" / "issue-954" / "joint-candidate-margin"


def default_paired_query_binding_root() -> Path:
    return model_store_root() / "research" / "issue-954" / "paired-query-binding"


def default_group_retention_root() -> Path:
    return model_store_root() / "research" / "issue-973-group-retention"


def default_group_retention_source_root() -> Path:
    return model_store_root() / "research" / "issue-1017"


def default_group_retention_decoder_root() -> Path:
    return model_store_root() / "research" / "issue-973-group-retention-decoder-v1"


def default_group_retention_decoder_cpu_recovery_root() -> Path:
    return (
        model_store_root()
        / "research"
        / "issue-973-group-retention-decoder-v1-cpu-recovery"
    )


def default_language_path_source_root() -> Path:
    """Return the shared, nonsealed #1019 source-data root."""

    return model_store_root() / "research" / "issue-1019"


def default_language_path_geometry() -> Path:
    """Return the qualified exact-H4 geometry inherited by the new rung."""

    return (
        model_store_root()
        / "research"
        / "issue-973-group-retention-decoder-v1-cpu-recovery"
        / "geometry"
        / "r4-group-address-geometry.json"
    )


def default_language_path_root() -> Path:
    """Return #973's compact retained language-path experiment root."""

    return model_store_root() / "research" / "issue-973-retained-language-path-v1"


def default_paired_h4_prompt_capacity_root() -> Path:
    """Return #973's independently frozen paired-H4 capacity root."""

    return (
        model_store_root()
        / "research"
        / "issue-973-paired-h4-prompt-capacity-v1"
    )


def default_paired_h4_prompt_capacity_predecessor() -> Path:
    """Return the immutable qualified retained-language predecessor."""

    return default_language_path_root()


def default_paired_h4_prompt_capacity_source_train() -> Path:
    """Return the verified nonsealed #1019 train-token store."""

    return default_language_path_source_root() / "tokens" / "train.u16"


def default_paired_h4_prompt_capacity_raw_source() -> Path:
    """Return the pinned raw TinyStories source used only for prompt freezing."""

    return (
        model_store_root()
        / "research"
        / "issue-1014"
        / "raw"
        / "TinyStoriesV2-GPT4-train.txt"
    )


def default_direct_retained_readout_root() -> Path:
    """Return #973's independently frozen readout-only campaign root."""

    return model_store_root() / "research" / "issue-973-direct-retained-readout-v1"


def default_direct_retained_readout_predecessor() -> Path:
    """Return the immutable qualified retained-language predecessor."""

    return default_language_path_root()


def default_direct_retained_readout_source_train() -> Path:
    """Return the verified nonsealed #1019 train-token store."""

    return default_language_path_source_root() / "tokens" / "train.u16"


def default_direct_retained_readout_source_train_index() -> Path:
    """Return the canonical #1019 train-story index."""

    return default_language_path_source_root() / "indexes" / "train.jsonl"


def default_direct_retained_readout_raw_source() -> Path:
    """Return the pinned raw source used only to seal prompt contrast V2."""

    return default_paired_h4_prompt_capacity_raw_source()
