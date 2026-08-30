"""Repository-local default paths for untracked #1014 bulk artifacts."""

from __future__ import annotations

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

