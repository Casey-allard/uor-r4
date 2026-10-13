"""The lesson registry: learn from a mistake once, on purpose.

A lesson is stored once under `docs/harness/mistakes/<id>.md`, retrieved
automatically when a future task touches the same files or topic
(`mistakes find -f <files>`), and - where a machine can decide it - turned into
a guard rule that runs forever.

The guard column is the point. `guard: <rule id>` binds a lesson to a rule in the
harness engine; `mistakes.guard_covered` fails when a `severity: block` lesson
names a rule id that does not exist, so a lesson cannot be written down and then
quietly forgotten. `guard: none: <reason>` is honest and allowed.

Lesson files use the strict front-matter subset `core.parse_front_matter` accepts
(scalars, inline lists, block lists) and exactly four body sections:

    ## What happened
    ## Symptom
    ## Lesson
    ## How to apply

Everything in this module is read-only except `capture`, which writes one file
into `docs/harness/mistakes/` and refuses to overwrite it without `--force`.

    python3 scripts/harness/cli.py mistakes list
    python3 scripts/harness/cli.py mistakes find -f scripts/harness/prlint.py
    python3 scripts/harness/cli.py mistakes capture --id my-lesson --title "..." --evidence docs/labs/...
"""

from __future__ import annotations

import fnmatch
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable

from checkrules import Rule, RuleResult
from core import HarnessPaths, emit, parse_front_matter, redact, today
from registries import Mistake, Registry, match_mistakes
from util import truncate

SEVERITIES = ("block", "warn", "info")
CLASSES = ("process", "evidence", "compute", "measurement", "delivery", "coordination")
STATUSES = ("active", "resolved", "superseded")
SECTIONS = ("What happened", "Symptom", "Lesson", "How to apply")
READMES = ("README.md", "README.MD", "readme.md")
ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]{2,63}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
HEADING_RE = re.compile(r"^(#{2,4})\s+(.+?)\s*$", re.MULTILINE)

#: the rules this module owns (also its public interface)
LOGICAL_RULES = (
    "mistakes.registry_valid",
    "mistakes.guard_covered",
    "mistakes.evidence_exists",
    "mistakes.stale_check",
)


# ------------------------------------------------------------------------------ data


@dataclass
class Lesson:
    """One lesson plus the structural errors that stop it being usable."""

    path: Path
    fm: dict[str, Any] = field(default_factory=dict)
    body: str = ""
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    @property
    def id(self) -> str:
        return str(self.fm.get("id", "") or self.path.stem)

    @property
    def severity(self) -> str:
        return str(self.fm.get("severity", "") or "")

    @property
    def trigger(self) -> list[str]:
        return _as_list(self.fm.get("trigger"))

    @property
    def paths(self) -> list[str]:
        return _as_list(self.fm.get("paths"))

    @property
    def milestones(self) -> list[str]:
        return _as_list(self.fm.get("milestones"))

    @property
    def guard(self) -> str:
        return str(self.fm.get("guard", "") or "")

    @property
    def evidence(self) -> list[str]:
        return _as_list(self.fm.get("evidence"))

    @property
    def status(self) -> str:
        return str(self.fm.get("status", "active") or "active")

    @property
    def class_(self) -> str:
        return str(self.fm.get("class", "") or "")


def _as_list(value: Any) -> list[str]:
    if value is None or value == "":
        return []
    if isinstance(value, list):
        return [str(v) for v in value]
    return [str(value)]


def read_lesson(path: Path) -> Lesson:
    """Parse one lesson file. Structural problems go into `Lesson.errors`."""
    lesson = Lesson(path=path)
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        lesson.errors.append(f"cannot read the file: {exc}")
        return lesson
    if not text.lstrip().startswith("---"):
        lesson.errors.append("no front matter: the file must start with '---'")
        return lesson
    try:
        fm, body = parse_front_matter(text)
    except ValueError as exc:
        lesson.errors.append(f"front matter does not parse: {exc}")
        return lesson
    lesson.fm = fm
    lesson.body = body.strip()
    lesson.errors.extend(_front_matter_errors(lesson))
    lesson.warnings.extend(_body_warnings(lesson))
    return lesson


def _front_matter_errors(lesson: Lesson) -> list[str]:
    fm = lesson.fm
    errors: list[str] = []
    for key in ("id", "title", "date", "severity", "class", "trigger", "paths", "evidence", "status"):
        if key not in fm or fm.get(key) in (None, "", []):
            errors.append(f"front matter is missing '{key}'")
    if not str(fm.get("mechanism", "") or "").strip():
        lesson.warnings.append("'mechanism' is empty: the lesson is not recalled by mechanism name")
    if str(fm.get("id", "")).lower() != lesson.path.stem.lower():
        errors.append(f"id {fm.get('id')!r} must equal the file stem {lesson.path.stem!r}")
    if not ID_RE.match(str(fm.get("id", ""))):
        errors.append(f"id {fm.get('id')!r} is not a kebab-case id")
    if str(fm.get("severity", "")) not in SEVERITIES:
        errors.append(f"severity must be one of {SEVERITIES}, got {fm.get('severity')!r}")
    if str(fm.get("class", "")) not in CLASSES:
        errors.append(f"class must be one of {CLASSES}, got {fm.get('class')!r}")
    if str(fm.get("status", "")) not in STATUSES:
        errors.append(f"status must be one of {STATUSES}, got {fm.get('status')!r}")
    if not DATE_RE.match(str(fm.get("date", ""))):
        errors.append(f"date must be YYYY-MM-DD, got {fm.get('date')!r}")
    guard = str(fm.get("guard", "") or "")
    if guard and not guard.startswith("none") and "." not in guard:
        errors.append(f"guard {guard!r} must be a dotted rule id or 'none: <reason>'")
    for key in ("trigger", "paths", "evidence"):
        value = fm.get(key)
        if value not in (None, "", []) and not isinstance(value, list):
            errors.append(f"'{key}' must be an inline or block list")
    if isinstance(fm.get("trigger"), list) and len(fm["trigger"]) < 2:
        errors.append("'trigger' needs at least two phrases, or the lesson is not retrievable")
    if isinstance(fm.get("paths"), list) and not fm["paths"]:
        errors.append("'paths' is empty: name a glob so the lesson is recalled")
    return errors


def _body_warnings(lesson: Lesson) -> list[str]:
    warnings: list[str] = []
    headings = [m.group(2).strip() for m in HEADING_RE.finditer(lesson.body)]
    for section in SECTIONS:
        if section not in headings:
            warnings.append(f"body is missing '## {section}'")
    for section in ("Lesson", "Symptom"):
        if not _section(lesson.body, section):
            warnings.append(f"'## {section}' is empty, so registries.Mistake.{section.lower()} reads nothing")
    extra = [h for h in headings if h not in SECTIONS]
    if extra:
        warnings.append(f"unexpected section(s): {', '.join(sorted(set(extra)))} (only {', '.join(SECTIONS)} are read)")
    return warnings


def _section(body: str, name: str) -> str:
    m = re.search(rf"^#{{2,4}}\s+{re.escape(name)}\s*$", body, re.MULTILINE | re.IGNORECASE)
    if not m:
        return ""
    rest = body[m.end() :]
    nxt = re.search(r"^#{2,4}\s+\S", rest, re.MULTILINE)
    return (rest[: nxt.start()] if nxt else rest).strip()


def load_lessons(paths: HarnessPaths) -> list[Lesson]:
    """Every mistake file, parsed, with its structural errors attached."""
    if not paths.mistakes_dir.is_dir():
        return []
    out: list[Lesson] = []
    for f in sorted(paths.mistakes_dir.glob("*.md")):
        if f.name in READMES or f.name.upper() == "README.MD":
            continue
        out.append(read_lesson(f))
    return out


def lesson_to_mistake(lesson: Lesson) -> Mistake:
    """The `registries.Mistake` view of a lesson (what `reg.mistakes` holds)."""
    fm = lesson.fm
    return Mistake(
        id=lesson.id,
        title=str(fm.get("title", lesson.id)),
        path=lesson.path,
        body=lesson.body,
        date=str(fm.get("date", "")),
        severity=lesson.severity if lesson.severity in SEVERITIES else "warn",
        class_=str(fm.get("class", "")),
        trigger=lesson.trigger,
        paths=lesson.paths,
        milestones=lesson.milestones,
        mechanism=str(fm.get("mechanism", "")),
        guard=lesson.guard,
        evidence=lesson.evidence,
        status=lesson.status,
    )


def _contract_mistakes(paths: HarnessPaths) -> list[Mistake]:
    """The `registries.Mistake` view of every lesson (used by `mistakes find` fixtures)."""
    return [lesson_to_mistake(lesson) for lesson in load_lessons(paths)]


# ------------------------------------------------------------------- path resolution


def resolve_path(paths: HarnessPaths, value: str) -> Path | None:
    """Resolve an evidence/record path relative to the repo, then the lesson."""
    if not value:
        return None
    p = Path(value)
    if p.is_absolute():
        return p if p.exists() else None
    direct = paths.repo / p
    if direct.exists():
        return direct
    in_dir = paths.mistakes_dir / p
    if in_dir.exists():
        return in_dir
    return None


#: directories a lesson never needs to match against. Excluding them keeps the set small even
#: when the checkout carries large dormant trees (research/ alone is ~11k files).
_INDEX_SKIP = (".git", "target", "node_modules", ".venv", "__pycache__", ".worktrees", "local", ".harness")


def _file_index(paths: HarnessPaths) -> list[str]:
    """Repo paths for the stale check, from git rather than the filesystem.

    This used to `rglob("*")` the whole tree - about 11k files - and then run three fnmatch
    calls per file for every lesson, which took ~46 seconds on Python 3.9 and dominated the
    entire `check`. `git ls-files` is the already-built index of exactly the files that matter,
    and it excludes ignored build output and scratch state for free.
    """
    import subprocess

    try:
        proc = subprocess.run(
            ["git", "-C", str(paths.repo), "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
            capture_output=True, text=True, timeout=30, check=False,
        )
        if proc.returncode == 0 and proc.stdout:
            out = []
            for rel in proc.stdout.split("\0"):
                if not rel:
                    continue
                if any(part in _INDEX_SKIP for part in rel.split("/")):
                    continue
                out.append(rel)
            return out
    except (OSError, subprocess.SubprocessError):  # pragma: no cover - environment
        pass
    # fallback: the filesystem walk, if git is unavailable for any reason
    root = paths.repo
    out = []
    for p in root.rglob("*"):
        if not p.is_file():
            continue
        rel = p.relative_to(root)
        if any(part in _INDEX_SKIP for part in rel.parts):
            continue
        out.append(rel.as_posix())
    return out


def _patterns_match_index(index: Iterable[str], patterns: Iterable[str]) -> bool:
    """Does any pattern match any indexed path? Cheap checks first.

    The hot path is: 20 lessons x 11k paths. The original form ran three fnmatch calls per
    (path, pattern) pair, which is tens of millions of calls. Most patterns are plain literal
    paths or directory prefixes, so those resolve with `in` / `startswith` before any glob is
    attempted, and a glob is only tried when the pattern actually contains a wildcard.
    """
    pats: list[tuple[str, str | None]] = []
    globs: list[str] = []
    for raw in patterns:
        pat = str(raw).strip().lstrip("./")
        if not pat:
            continue
        if "*" in pat or "?" in pat or "[" in pat:
            if pat.endswith("/**"):
                pats.append((pat, pat[:-3].rstrip("/") + "/"))
            else:
                globs.append(pat)
        else:
            pats.append((pat, None))
    if not pats and not globs:
        return True  # an empty pattern list is not a staleness finding
    paths = list(index)
    for pat, prefix in pats:
        for rel in paths:
            if rel == pat or rel.endswith("/" + pat) or (prefix and rel.startswith(prefix)):
                return True
            if pat in rel:
                return True
    for glob in globs:
        for rel in paths:
            if fnmatch.fnmatch(rel, glob):
                return True
    return False


# ------------------------------------------------------------------------------- rules


def _check_registry_valid(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    lessons = load_lessons(paths)
    seen: dict[str, Path] = {}
    for lesson in lessons:
        for error in lesson.errors:
            out.append(
                RuleResult(
                    rule_id="mistakes.registry_valid",
                    severity="error",
                    ok=False,
                    message=f"{lesson.path.name}: {error}",
                    path=str(lesson.path),
                    fix="fix the front matter: see docs/harness/mistakes/README.md for the exact template",
                )
            )
        if lesson.id in seen:
            out.append(
                RuleResult(
                    rule_id="mistakes.registry_valid",
                    severity="error",
                    ok=False,
                    message=f"duplicate lesson id {lesson.id!r} in {lesson.path.name} and {seen[lesson.id].name}",
                    path=str(lesson.path),
                    fix="give one of them a unique id (the file stem must equal the id)",
                )
            )
        seen.setdefault(lesson.id, lesson.path)
    return out


def _check_evidence_exists(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    for lesson in load_lessons(paths):
        if lesson.errors and not lesson.evidence:
            continue  # registry_valid already reports the unreadable file
        for value in lesson.evidence:
            if resolve_path(paths, value) is None:
                out.append(
                    RuleResult(
                        rule_id="mistakes.evidence_exists",
                        severity="error",
                        ok=False,
                        message=f"{lesson.id}: evidence path {value} does not exist",
                        path=str(lesson.path),
                        fix="cite a path that exists (repo-relative), or remove the claim",
                    )
                )
    return out


def _check_guard_covered(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    known = _known_rule_ids()
    out: list[RuleResult] = []
    for lesson in load_lessons(paths):
        if lesson.severity != "block" or lesson.errors:
            continue
        guard = lesson.guard
        if not guard:
            out.append(
                RuleResult(
                    rule_id="mistakes.guard_covered",
                    severity="warn",
                    ok=False,
                    message=f"{lesson.id} is severity block with no guard",
                    path=str(lesson.path),
                    fix="name a rule id, or 'none: <reason why no machine can decide it>'",
                )
            )
        elif guard.startswith("none"):
            if len(guard.split(":", 1)[-1].strip()) < 4:
                out.append(
                    RuleResult(
                        rule_id="mistakes.guard_covered",
                        severity="warn",
                        ok=False,
                        message=f"{lesson.id} guard {guard!r} gives no reason",
                        path=str(lesson.path),
                        fix="write 'none: <reason>'",
                    )
                )
        elif guard not in known:
            out.append(
                RuleResult(
                    rule_id="mistakes.guard_covered",
                    severity="warn",
                    ok=False,
                    message=f"{lesson.id} names guard {guard!r}, which is not a rule id in the harness engine",
                    path=str(lesson.path),
                    fix="use a real rule id (python3 scripts/harness/cli.py check) or 'none: <reason>'",
                )
            )
    return out


def _check_stale(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    index = _file_index(paths)
    out: list[RuleResult] = []
    for lesson in load_lessons(paths):
        if lesson.errors or not lesson.paths:
            continue
        if not _patterns_match_index(index, lesson.paths):
            out.append(
                RuleResult(
                    rule_id="mistakes.stale_check",
                    severity="info",
                    ok=False,
                    message=f"{lesson.id}: paths {lesson.paths} match no file in the repo",
                    path=str(lesson.path),
                    fix="update or delete the lesson: a pattern that matches nothing never recalls it",
                )
            )
    return out


def _known_rule_ids() -> set[str]:
    """Every rule id the engine can register, without importing anything twice."""
    import checkrules

    known = {r.id for r in checkrules.rules()}
    for name in checkrules.RULE_MODULES:
        try:
            module = __import__(name)
        except Exception:  # noqa: BLE001 - a module that cannot load has no ids
            continue
        getter = getattr(module, "register_rules", None)
        if getter is None:
            continue
        try:
            known.update(r.id for r in getter())
        except Exception:  # noqa: BLE001
            continue
    # prlint's gates are raised against a PR body, which the engine never has; it
    # declares them as data so a lesson can cite one as its guard.
    try:
        import prlint

        known.update(getattr(prlint, "GATE_RULES", ()))
    except Exception:  # noqa: BLE001
        pass
    return known


def register_rules() -> list[Rule]:
    return [
        Rule(
            id="mistakes.registry_valid",
            description="every lesson file parses, ids are unique, and the front matter is complete",
            severity="error",
            check=_check_registry_valid,
            rationale="a lesson the registry cannot read is a lesson nobody will ever be recalled",
            mistake="",
        ),
        Rule(
            id="mistakes.evidence_exists",
            description="every cited evidence path exists on disk",
            severity="error",
            check=_check_evidence_exists,
            rationale="a lesson is only as good as the record behind it (evidence class)",
        ),
        Rule(
            id="mistakes.guard_covered",
            description="every block-severity lesson names a guard rule that exists, or explains why none can",
            severity="warn",
            check=_check_guard_covered,
            rationale="D21/D22: a written rule that nothing executes is how counts drifted before",
        ),
        Rule(
            id="mistakes.stale_check",
            description="every lesson's paths still match a file in the repo",
            severity="info",
            check=_check_stale,
            rationale="a stale glob silently stops recalling the lesson",
        ),
    ]


# ------------------------------------------------------------------------- selection


def render_matches(
    reg: Registry,
    files: Iterable[str] = (),
    query: str = "",
    milestone: str | None = None,
) -> list[dict[str, Any]]:
    """Which lessons apply to these files/query/milestone, with why."""
    files = [str(f) for f in files]
    ms = [milestone] if milestone else []
    hits = match_mistakes(reg.mistakes, files=files, query=query, milestones=ms)
    out: list[dict[str, Any]] = []
    for mistake, why in hits:
        out.append(
            {
                "id": mistake.id,
                "title": mistake.title,
                "severity": mistake.severity,
                "class": mistake.class_,
                "guard": mistake.guard,
                "why": why,
                "lesson": mistake.lesson,
                "evidence": list(mistake.evidence),
                "path": str(mistake.path),
            }
        )
    return out


def format_matches(matches: list[dict[str, Any]]) -> str:
    if not matches:
        return "no lesson matches this task (docs/harness/mistakes/README.md shows the trigger format)"
    lines = [f"{len(matches)} lesson(s) apply to this task:", ""]
    for m in matches:
        lines.append(f"  [{m['severity']}] {m['id']} - {m['title']}")
        lines.append(f"      why: {m['why']}")
        if m.get("lesson"):
            lines.append(f"      lesson: {truncate(m['lesson'], 220)}")
        if m.get("guard"):
            lines.append(f"      guard: {m['guard']}")
        if m.get("evidence"):
            lines.append(f"      evidence: {', '.join(m['evidence'][:2])}")
    return "\n".join(lines)


def render_index(mistakes: Iterable[Mistake]) -> str:
    rows = sorted(mistakes, key=lambda m: ({"block": 0, "warn": 1, "info": 2}.get(m.severity, 3), m.id))
    if not rows:
        return "no lessons yet: add docs/harness/mistakes/<id>.md (see docs/harness/mistakes/README.md)"
    out = [f"{len(rows)} lessons in docs/harness/mistakes/ (block first - read these before you touch their paths)", ""]
    counted: dict[str, int] = {}
    for m in rows:
        counted[m.severity] = counted.get(m.severity, 0) + 1
    out.append("  " + ", ".join(f"{k} {counted[k]}" for k in SEVERITIES if k in counted))
    out.append("")
    for m in rows:
        guard = m.guard or "(none)"
        out.append(f"  [{m.severity:5}] {m.id:44} {truncate(m.title, 60)}")
        out.append(f"          class {m.class_ or '?':12} guard {truncate(guard, 40)}  paths {truncate(', '.join(m.paths), 60)}")
    return "\n".join(out)


# --------------------------------------------------------------------------- capture

_LESSON_TEMPLATE = """---
{front_matter}---

## What happened

{what}

## Symptom

{symptom}

## Lesson

{lesson}

## How to apply

{apply}
"""


def build_lesson_text(
    *,
    lesson_id: str,
    title: str,
    severity: str = "warn",
    class_: str = "process",
    trigger: Iterable[str] = (),
    paths: Iterable[str] = (),
    milestones: Iterable[str] = (),
    mechanism: str = "",
    guard: str = "",
    evidence: Iterable[str] = (),
    status: str = "active",
    date: str = "",
    symptom: str = "",
    lesson: str = "",
    what: str = "",
    apply: str = "",
) -> str:
    """Render a lesson file. Placeholders keep it honest and registry-valid."""
    triggers = [str(t) for t in trigger if str(t).strip()]
    path_globs = [str(p) for p in paths if str(p).strip()]
    ev = [str(e) for e in evidence if str(e).strip()]
    if not triggers:
        triggers = [lesson_id.replace("-", " "), truncate(title, 40)]
    if not path_globs:
        path_globs = ["docs/harness/**"]
    if not ev:
        ev = ["docs/labs/README.md"]

    fm = {
        "id": lesson_id,
        "title": title,
        "date": date or today(),
        "severity": severity,
        "class": class_,
        "trigger": triggers,
        "paths": path_globs,
        "milestones": [str(m) for m in milestones if str(m).strip()],
        "mechanism": mechanism,
        "guard": guard or f"none: captured by hand on {date or today()}; no rule decides it yet",
        "evidence": ev,
        "status": status,
    }
    front_matter = "".join(f"{key}: {_yaml(fm[key])}\n" for key in fm)
    return _LESSON_TEMPLATE.format(
        front_matter=front_matter,
        what=what.strip() or "(not recorded yet: what the run or delivery actually did)",
        symptom=symptom.strip() or "(not recorded yet: what someone would see that made them look this up)",
        lesson=lesson.strip() or "(not recorded yet: the transferable rule this lesson teaches)",
        apply=apply.strip() or "(not recorded yet: what to do differently, with the exact command or check)",
    )


def _yaml(value: Any) -> str:
    if isinstance(value, list):
        if not value:
            return "[]"
        return "[" + ", ".join(_scalar(v) for v in value) + "]"
    return _scalar(value)


def _scalar(value: Any) -> str:
    text = str(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if text == "":
        return '""'
    if re.fullmatch(r"[A-Za-z0-9_./*<>#-]+", text) and ":" not in text and not text.startswith("-"):
        return text
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def capture(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    """Write one lesson from CLI flags. Refuses to overwrite without --force."""
    raw_id = str(getattr(args, "id", "") or "").strip()
    title = str(getattr(args, "title", "") or "").strip()
    if not raw_id or not title:
        emit("mistakes capture needs --id and --title", sys.stderr)
        return 2
    if not ID_RE.match(raw_id):
        emit(f"--id {raw_id!r} must be kebab-case (a-z, 0-9, '-')", sys.stderr)
        return 2
    lesson_id = raw_id

    severity = str(getattr(args, "severity", "warn") or "warn")
    class_ = str(getattr(args, "class_", "process") or "process")
    if severity not in SEVERITIES:
        emit(f"--severity must be one of {SEVERITIES}", sys.stderr)
        return 2
    if class_ not in CLASSES:
        emit(f"--class must be one of {CLASSES}", sys.stderr)
        return 2

    target = paths.mistakes_dir / f"{lesson_id}.md"
    if target.exists() and not getattr(args, "force", False):
        emit(f"{target} exists; pass --force to overwrite it", sys.stderr)
        return 1

    text = build_lesson_text(
        lesson_id=lesson_id,
        title=title,
        severity=severity,
        class_=class_,
        trigger=getattr(args, "trigger", ()) or (),
        paths=getattr(args, "paths", ()) or (),
        milestones=getattr(args, "milestones", ()) or (),
        mechanism=str(getattr(args, "mechanism", "") or ""),
        guard=str(getattr(args, "guard", "") or ""),
        evidence=getattr(args, "evidence", ()) or (),
        symptom=str(getattr(args, "symptom", "") or ""),
        lesson=str(getattr(args, "lesson", "") or ""),
    )
    text = redact(text)

    # A capture must never write a file `mistakes.registry_valid` rejects.
    problems, warnings = _draft_problems(paths, reg, target, text)
    if problems:
        emit(f"refusing to write {target}: the draft is not a valid lesson", sys.stderr)
        for problem in problems:
            emit(f"  {problem}", sys.stderr)
        return 1
    for warning in warnings:
        emit(f"note: {warning}", sys.stderr)

    existed = target.exists()
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")
    check = read_lesson(target)
    if check.errors:  # pragma: no cover - guarded above
        target.unlink(missing_ok=True)
        emit(f"wrote and removed {target}: {check.errors[0]}", sys.stderr)
        return 1
    emit(f"{'overwrote' if existed else 'wrote'} {target}")
    emit(f"  severity {severity}, class {class_}, guard {check.guard}")
    emit(f"  read it back: python3 scripts/harness/cli.py mistakes show {lesson_id}")
    emit("  check: python3 scripts/harness/cli.py check --only mistakes")
    return 0


def _draft_problems(paths: HarnessPaths, reg: Registry, target: Path, text: str) -> tuple[list[str], list[str]]:
    """What `mistakes.registry_valid` and `evidence_exists` would say about the draft.

    Errors refuse the write. Warnings are reported and allowed: a captured lesson
    with no mechanism (or no evidence) is honest, just less useful.
    """
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        trial = Path(tmp) / target.name
        trial.write_text(text, encoding="utf-8")
        lesson = read_lesson(trial)
        problems = [f"{target.name}: {e}" for e in lesson.errors]
        warnings = [f"{target.name}: {w}" for w in lesson.warnings]
        for value in lesson.evidence:
            if resolve_path(paths, value) is None:
                problems.append(f"{target.name}: evidence path {value} does not exist")
        others = [l for l in load_lessons(paths) if l.path.resolve() != target.resolve()]
        if any(l.id == lesson.id for l in others):
            problems.append(f"{target.name}: id {lesson.id!r} is already used by another lesson")
        return problems, warnings


# ------------------------------------------------------------------------------- main


def main(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    """`mistakes list|find|show|capture`. Prints its own output."""
    import json

    action = getattr(args, "action", "list") or "list"
    if action == "list":
        if getattr(args, "json", False):
            emit(json.dumps({"mistakes": [{"id": m.id, "severity": m.severity, "title": m.title, "guard": m.guard, "class": m.class_} for m in reg.mistakes]}, indent=2))
        else:
            emit(render_index(reg.mistakes))
        return 0
    if action == "find":
        matches = render_matches(reg, files=getattr(args, "files", ()) or (), query=getattr(args, "query", "") or "", milestone=getattr(args, "milestone", None))
        if getattr(args, "json", False):
            emit(json.dumps({"matches": matches}, indent=2))
        else:
            emit(format_matches(matches))
        return 0
    if action == "show":
        from registries import find_mistake

        mistake = find_mistake(reg.mistakes, getattr(args, "id", None) or "") if getattr(args, "id", None) else None
        if mistake is None:
            emit(f"unknown mistake {getattr(args, 'id', None)!r}", sys.stderr)
            return 2
        emit(mistake.path.read_text(encoding="utf-8"))
        return 0
    if action == "capture":
        return capture(args, paths, reg)
    return 2
