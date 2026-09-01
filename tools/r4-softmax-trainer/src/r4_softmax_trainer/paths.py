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
