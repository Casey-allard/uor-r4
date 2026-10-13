"""Core utilities for the research harness.

Standard library only. No model imports, no network imports at module scope.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA = 1


# --------------------------------------------------------------------------- paths


@dataclass(frozen=True)
class HarnessPaths:
    """Explicit roots, so the library is usable and testable from anywhere."""

    repo: Path
    harness: Path
    state: Path

    @classmethod
    def discover(cls, repo: str | os.PathLike[str] | None = None, state: str | os.PathLike[str] | None = None) -> "HarnessPaths":
        root = Path(repo).resolve() if repo is not None else _find_repo_root()
        harness = root / "docs" / "harness"
        st = Path(state).resolve() if state is not None else root / ".harness"
        return cls(repo=root, harness=harness, state=st)

    @property
    def milestones_json(self) -> Path:
        return self.harness / "milestones.json"

    @property
    def roles_json(self) -> Path:
        return self.harness / "roles.json"

    @property
    def panels_dir(self) -> Path:
        return self.harness / "panels"

    @property
    def skills_dir(self) -> Path:
        return self.harness / "skills"

    @property
    def mistakes_dir(self) -> Path:
        return self.harness / "mistakes"

    @property
    def ledger_dir(self) -> Path:
        return self.harness / "ledger"

    @property
    def decisions_md(self) -> Path:
        return self.repo / "docs" / "integration" / "DECISIONS.md"

    @property
    def session_goal_md(self) -> Path:
        return self.repo / "docs" / "labs" / "session-goal.md"

    @property
    def current_state_md(self) -> Path:
        return self.repo / "docs" / "integration" / "current-state.md"

    @property
    def mechanisms_dir(self) -> Path:
        return self.repo / "docs" / "mechanisms"

    @property
    def policy_json(self) -> Path:
        return self.repo / "docs" / "integration" / "agent-execution-policy.json"


def _find_repo_root(start: Path | None = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    for cand in [cur, *cur.parents]:
        if (cand / ".git").exists():
            return cand
    return cur


def which(name: str) -> str | None:
    from shutil import which as _w

    return _w(name)


def run(cmd: list[str], cwd: Path | None = None, timeout: int = 20) -> tuple[int, str]:
    """Run a command, never raising. Returns (returncode, stdout+stderr)."""
    try:
        p = subprocess.run(
            cmd,
            cwd=str(cwd) if cwd else None,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except (OSError, subprocess.SubprocessError) as exc:  # pragma: no cover - environment
        return 127, str(exc)


# ----------------------------------------------------------------------------- time


def today() -> str:
    return datetime.now(timezone.utc).astimezone().strftime("%Y-%m-%d")


def month() -> str:
    return datetime.now(timezone.utc).astimezone().strftime("%Y-%m")


def now_iso() -> str:
    return datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")


def timestamp_slug() -> str:
    return datetime.now(timezone.utc).astimezone().strftime("%Y%m%dT%H%M%S")


# ---------------------------------------------------------------------- front matter


_FM_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n?(.*)$", re.DOTALL)


def parse_front_matter(text: str) -> tuple[dict[str, Any], str]:
    """Parse a small YAML subset: scalars, inline lists, block lists, nested maps.

    Deliberately not a YAML implementation: it accepts exactly the shapes used by
    the registries (front matter and mistake headers), and raises ValueError on
    anything else so a malformed registry fails loudly instead of silently.
    """
    m = _FM_RE.match(text.lstrip("\ufeff"))
    if not m:
        return {}, text
    head, body = m.group(1), m.group(2)
    return _parse_yaml_subset(head), body


def _coerce(scalar: str) -> Any:
    s = scalar.strip()
    if not s:
        return ""
    if len(s) >= 2 and s[0] == s[-1] and s[0] in "\"'":
        return s[1:-1]
    low = s.lower()
    if low in ("true", "yes"):
        return True
    if low in ("false", "no"):
        return False
    if low in ("null", "~"):
        return None
    if s.startswith("[") and s.endswith("]"):
        inner = s[1:-1].strip()
        if not inner:
            return []
        return [_coerce(part) for part in _split_inline(inner)]
    try:
        if re.fullmatch(r"-?\d+", s):
            return int(s)
        if re.fullmatch(r"-?\d+\.\d+", s):
            return float(s)
    except ValueError:  # pragma: no cover - guarded by regex
        pass
    return s


def _split_inline(inner: str) -> list[str]:
    parts, buf, quote = [], [], ""
    for ch in inner:
        if quote:
            buf.append(ch)
            if ch == quote:
                quote = ""
            continue
        if ch in "\"'":
            quote = ch
            buf.append(ch)
            continue
        if ch == ",":
            parts.append("".join(buf))
            buf = []
            continue
        buf.append(ch)
    if buf:
        parts.append("".join(buf))
    return parts


def _parse_yaml_subset(text: str) -> dict[str, Any]:
    root: dict[str, Any] = {}
    # stack of (indent, container)
    stack: list[tuple[int, Any]] = [(-1, root)]
    for raw in text.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        line = raw.strip()
        while len(stack) > 1 and indent <= stack[-1][0]:
            stack.pop()
        container = stack[-1][1]
        if line.startswith("- "):
            if not isinstance(container, list):
                raise ValueError(f"list item outside a list: {line!r}")
            item = line[2:].strip()
            if ":" in item and not item.startswith(("\"", "'", "[")):
                key, _, rest = item.partition(":")
                entry: dict[str, Any] = {key.strip(): _coerce(rest)}
                container.append(entry)
                stack.append((indent, entry))
            else:
                container.append(_coerce(item))
            continue
        key, sep, rest = line.partition(":")
        if not sep:
            raise ValueError(f"not a key: value line: {line!r}")
        key = key.strip()
        rest = rest.strip()
        if rest:
            container[key] = _coerce(rest)
            continue
        # look ahead is not available; decide by the next non-empty line's shape
        nxt = _peek_next(text, raw)
        child: Any = [] if (nxt or "").strip().startswith("- ") else {}
        container[key] = child
        stack.append((indent, child))
    return root


def _peek_next(text: str, current: str) -> str | None:
    lines = text.splitlines()
    try:
        idx = lines.index(current)
    except ValueError:  # pragma: no cover - current always came from text
        return None
    for line in lines[idx + 1 :]:
        if line.strip() and not line.lstrip().startswith("#"):
            return line
    return None


# ------------------------------------------------------------------------- policies


# Files that must never be read or printed by harness tooling.
SECRET_PATTERNS = (
    re.compile(r"config\.toml$", re.IGNORECASE),
)

_REDACT_RE = re.compile(
    r"(?i)\b(gh[pousr]_[A-Za-z0-9]{8,}|sk-[A-Za-z0-9_\-]{8,}|rpa_[A-Za-z0-9]{8,})\b"
)


def redact(text: str) -> str:
    """Redact anything token-shaped before it can reach a transcript or ledger."""
    return _REDACT_RE.sub("<redacted>", text)


def is_secret_path(path: Path) -> bool:
    s = str(path)
    if "runpod" in s and path.name == "config.toml":
        return True
    if "/.dsh/.credentials.yaml" in s or s.endswith(".credentials.yaml"):
        return True
    return any(p.search(path.name) for p in SECRET_PATTERNS)


# ----------------------------------------------------------------------------- json


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def dump_json(path: Path, obj: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    with tmp.open("w", encoding="utf-8") as fh:
        json.dump(obj, fh, indent=2, ensure_ascii=False)
        fh.write("\n")
    tmp.replace(path)


def append_jsonl(path: Path, record: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    out: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return out


# ------------------------------------------------------------------------ formatting


def human_tokens(text: str) -> int:
    """Rough token count (chars/4 + lines). Used only for context cost reporting."""
    chars = len(text)
    return max(1, chars // 4)


@dataclass
class Source:
    """One loaded context source, with why it was selected."""

    path: str
    reason: str
    tokens: int = 0
    section: str = ""


@dataclass
class ContextPack:
    """An assembled brief: an ordered list of markdown blocks plus its provenance."""

    title: str
    blocks: list[tuple[str, str]] = field(default_factory=list)  # (heading, body)
    sources: list[Source] = field(default_factory=list)
    dropped: list[tuple[str, str]] = field(default_factory=list)  # (path, reason)

    def add(self, heading: str, body: str, path: str, reason: str, section: str = "") -> None:
        body = body.strip()
        if not body:
            self.dropped.append((path, f"empty content ({reason})"))
            return
        self.blocks.append((heading, body))
        self.sources.append(Source(path=path, reason=reason, tokens=human_tokens(body), section=section))

    @property
    def tokens(self) -> int:
        return sum(s.tokens for s in self.sources) + human_tokens(self.title)

    def render(self) -> str:
        parts = [f"# {self.title}", ""]
        for heading, body in self.blocks:
            parts.append(f"## {heading}")
            parts.append(body)
            parts.append("")
        parts.append("---")
        parts.append("*Context selected by `scripts/harness/cli.py`. Sources, in load order:*")
        for s in self.sources:
            extra = f" ({s.reason})" if s.reason else ""
            parts.append(f"- `{s.path}`{extra} - ~{s.tokens} tokens")
        if self.dropped:
            parts.append("")
            parts.append("*Not loaded, and why:*")
            for path, reason in self.dropped:
                parts.append(f"- `{path}` - {reason}")
        parts.append("")
        parts.append(f"*Total ~{self.tokens} tokens.*")
        return "\n".join(parts)


def emit(text: str, out: Any = None) -> None:
    (out or sys.stdout).write(text if text.endswith("\n") else text + "\n")
