"""Shared helpers for harness modules.

Tiny on purpose: anything that two modules would otherwise duplicate lives here.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from core import ContextPack, HarnessPaths  # noqa: F401  (re-exported for modules)

MISSING = "MISSING"
NOT_RUN = "NOT_RUN"
ABSENT = "ABSENT"


@dataclass
class Result:
    """A module's answer to one question, with the exact call that produced it."""

    ok: bool
    message: str
    data: dict[str, Any] = field(default_factory=dict)
    command: str = ""
    cwd: str = ""
    returncode: int | None = None


def rel(path: Path | str, root: Path) -> str:
    try:
        return str(Path(path).resolve().relative_to(Path(root).resolve()))
    except (ValueError, OSError):
        return str(path)


def truncate(text: str, limit: int = 400) -> str:
    text = " ".join(text.split())
    if len(text) <= limit:
        return text
    return text[: limit - 1].rstrip() + "…"


def bullet(lines: list[str], indent: str = "- ") -> str:
    return "\n".join(f"{indent}{line}" for line in lines)


def read_text(path: Path, limit: int | None = None) -> str:
    if not path.exists() or not path.is_file():
        return ""
    text = path.read_text(encoding="utf-8", errors="replace")
    if limit is not None and len(text) > limit:
        return text[:limit]
    return text


def first_markdown_section(text: str, heading: str) -> str:
    """Return the body of `## <heading>` (or `###`), or "".

    A heading inside a blockquote, a fenced code block, or an HTML comment is not a
    section boundary: session-goal.md carries `## Owner direction` lines inside
    quoted context, and treating those as boundaries truncates the contract.
    """
    want = re.compile(rf"^#{{2,4}}\s+{re.escape(heading)}\s*$", re.IGNORECASE)
    any_head = re.compile(r"^#{2,4}\s+\S")
    lines = text.splitlines()
    start: int | None = None
    for i, line in enumerate(lines):
        if want.match(line.strip()):
            start = i + 1
            break
    if start is None:
        return ""
    out: list[str] = []
    in_fence = False
    in_comment = False
    for line in lines[start:]:
        stripped = line.strip()
        if stripped.startswith("<!--"):
            in_comment = True
        if in_comment:
            out.append(line)
            if "-->" in stripped:
                in_comment = False
            continue
        if stripped.startswith("```"):
            in_fence = not in_fence
            out.append(line)
            continue
        if line.startswith(">"):
            # a blockquote is quoted text, never a section boundary
            out.append(line)
            continue
        if not in_fence and any_head.match(line) and not re.match(r"^\s{4,}\S", line):
            break
        out.append(line)
    return "\n".join(out).strip()


def markdown_sections(text: str, level: int = 2) -> list[tuple[str, str]]:
    """Split markdown into (heading, body) pairs at the given heading level."""
    pat = re.compile(rf"^#{{{level}}}\s+(.+?)\s*$", re.MULTILINE)
    marks = list(pat.finditer(text))
    out: list[tuple[str, str]] = []
    for i, m in enumerate(marks):
        end = marks[i + 1].start() if i + 1 < len(marks) else len(text)
        out.append((m.group(1).strip(), text[m.end() : end].strip()))
    return out


def indent_block(text: str, prefix: str = "  ") -> str:
    return "\n".join(prefix + line if line.strip() else line for line in text.splitlines())


def yes_no(value: bool) -> str:
    return "yes" if value else "no"
