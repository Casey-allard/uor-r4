"""PR body lint: the delivery contract, machine-checked.

`docs/labs/session-goal.md` §3 requires every PR to declare
``Line: <name> · count n/3 · headline before -> after``; D21 makes a pivot card the
next action at 3/3; D22 requires a pre-registration before any run that produces
numbers; CONTRIBUTING.md § Delivery cadence requires a dated record, a current-state
entry and exact-head checks. Until this module existed nothing read the PR body, so
counts drifted silently and the pre-registration gate was skipped without a trace.

This module parses the body, checks it against the frozen registry
(`docs/harness/milestones.json`) and the files on disk, and can render a compliant
body back out. It never runs a model, never needs the network, and never raises out
of a check: a malformed body is a list of problems, not an exception.

    python3 scripts/harness/cli.py lint check --body-file .github/pull_request_template.md
    python3 scripts/harness/cli.py lint format --body-file my-pr-body.md
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

from core import HarnessPaths, redact
from registries import Milestone, Registry
from util import truncate

# --------------------------------------------------------------------------- rule ids
#
# Cited as guards by docs/harness/mistakes/*.md, so an id here is an interface.

LOGICAL_RULES = (
    "prlint.parse",
    "prlint.line_declared",
    "prlint.count_matches_registry",
    "prlint.headline_present",
    "prlint.headline_vs_registry",
    "prlint.pivot_at_three",
    "prlint.decision_present",
    "prlint.record_path",
    "prlint.current_state_entry",
    "prlint.prereg_before_compute",
    "prlint.softmax_ban",
    "prlint.no_cpu_model_compute",
    "prlint.checks_at_head",
    "prlint.org_rules",
)

#: rules the engine runs when no PR body is in hand. A gate is *declared* here and
#: raised by `check_pr`; this list is what makes the id resolvable (and therefore
#: what lets a lesson cite it in `guard:`), and what `mistakes.guard_covered`
#: checks against. Keep it equal to LOGICAL_RULES.
GATE_RULES = LOGICAL_RULES

DECISIONS = ("KEEP", "REJECT", "NOT YET PROMOTED")
CHECKLIST_COUNT = 8  # .github/pull_request_template.md
MILESTONE_ISSUES = range(2029, 2037)

_COMMENT_RE = re.compile(r"<!--.*?-->", re.DOTALL)
_SEP_RE = re.compile(r"\s*(?:·|\||—|–|::)\s*")
_ARROW_RE = re.compile(r"\s*(?:→|->|-->|=>)\s*")
_HEADLINE_LABEL_RE = re.compile(r"(?i)\bheadline\b\s*[:\-]?\s*")
_COUNT_RE = re.compile(r"(?i)\bcount\b\s*[:\-]?\s*(?P<n>\d+)?\s*(?:/\s*(?P<of>\d+))?")
#: segments that are template scaffolding, not a line name
_COUNT_SEGMENT_RE = re.compile(r"(?i)^count\b\s*[:\-]?\s*(?:\d+\s*)?(?:/\s*\d+)?\s*\.?$")
_ISSUE_RE = re.compile(r"#(\d{3,6})\b")
_CMD_RE = re.compile(
    r"(?im)\b(python3|cargo|pytest|unittest|uv run|docker|just|rustup)\b[^\n]{0,120}?(?:\s-\S|/)"
    r"|^\s*(?:\$|>)\s*(?:python3|cargo|pytest|python|uv|docker|just)\b"
)
_BOX_RE = re.compile(r"(?m)^\s*[-*+]\s*\[(?P<state>[ xX]?)\]\s*(?P<label>.+?)\s*$")
#: the template's own text sits in unchecked boxes. An unticked box is an
#: instruction, not a claim: "Any model compute here is covered by a linked
#: pre-registration carrying `TEST FITNESS: FIT`" must not satisfy the D22 gate.
_CHECKBOX_LINE_RE = re.compile(r"(?m)^[ \t]*[-*+][ \t]*\[[ xX]?\][^\n]*$")
_LAB_RECORD_RE = re.compile(r"docs/labs/([A-Za-z0-9._-]+)/README\.md")
_EVIDENCE_RE = re.compile(r"(?:docs|archive)/evidence/[A-Za-z0-9._/-]+")
#: the charter's per-line notebook page (owner, 19 October 2026, Q4): one row per result.
_NOTEBOOK_RE = re.compile(r"docs/research/notebook/[A-Za-z0-9._-]+\.md")
_DATED_CELL_RE = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
#: Q2: builds, finite kernel tests and host checks need no pre-registration ...
_HOST_CHECK_RE = re.compile(
    r"(?i)\b(?:cargo|rustc|clippy|fmt|unittest|pytest|builds?|built|compil\w*|tests?|lint|parity|checks?|kernel|finite|host)\b"
)
#: ... unless the line is about GPU training or evaluation, or a milestone claim.
_RESEARCH_RUN_RE = re.compile(
    r"(?i)\b(?:train\w*|fine-?tun\w*|evaluat\w*|eval|bpb|nll|perplexit\w*|accuracy|panel|headline|milestone|seeds?|steps?|fit|fits|fitting|scor\w*|loss|top-?\d|held-?out|qualif\w*|reference)\b"
)
#: Q1: serving the exported D11 integer artifact on a client device is the product, not research compute ...
_DEVICE_SERVING_RE = re.compile(
    r"(?i)\bd11\b|\bexport(?:ed)?\b[^\n]{0,24}\bartifact\b|\binteger\s+artifact\b|\bserv(?:e|es|ed|ing)\b|\bwasm\b|\bapple\s+silicon\b|\bclient\s+devices?\b|\bon-?device\b"
)
#: ... while CPU training, evaluation, qualification and reference execution stay flagged (D24).
_RESEARCH_CPU_RE = re.compile(
    r"(?i)\b(?:train\w*|fine-?tun\w*|evaluat\w*|eval|qualif\w*|reference|parity|fit|fits|fitting|scor\w*|bpb|nll|perplexit\w*|panel|headline|milestone|accuracy|recall|held-?out)\b"
)
#: A host-side fallback is what D24 exists to catch; device serving never is one, so such a line
#: is never excused.
_HOST_FALLBACK_RE = re.compile(r"(?i)host[- ]?(?:side[- ]?)?fall-?back")

# D22 pre-registration references: a prep sheet id, a milestone comment link, or a verdict.
_PREREG_MARKERS = (
    re.compile(r"(?i)\bTEST\s+FITNESS\s*:?\s*(?:FIT|NOT[ _]FIT)"),
    re.compile(r"(?i)\bpre-?regist(?:er|ration|ered)\b"),
    re.compile(r"(?i)#issuecomment-\d+"),
    re.compile(r"(?i)\bprep\s+(?:sheet|id|card)\b|\bsheet\s+[a-z0-9][a-z0-9._-]{3,}\b"),
)

# D23: delegate the scan, never reimplement it. Phrases that mean "a softmax is still here".
_SOFTMAX_PHRASES = re.compile(r"(?i)\bsoftmax(?:es)?\b|log[-\s]?softmax|normalized exponential")

# D24: CPU model compute presented as an accepted result.
_CPU_COMPUTE_RE = re.compile(
    r"(?i)\bcpu[- ](?:training|train|fit|fitting|evaluation|eval|inference|run|compute)\b"
    r"|\btrain(?:ed|ing)?\s+on\s+(?:the\s+)?(?:cpu|laptop)\b"
    r"|\b(?:ran|running|runs|executed|measured)\s+on\s+(?:the\s+)?(?:cpu|laptop|host)\b"
    r"|\bhost[- ](?:side\s+)?fallback\b"
    r"|\bhost\s+fallback\b"
)
_QUALITY_NEAR_RE = re.compile(
    r"(?i)\bbpb\b|\bnll\b|\bperplexit|\baccuracy\b|\bexact\b|\brecall\b|\bscore[sd]?\b"
    r"|\brows?\b|\bpanel\b|\bheadline\b|\bcheck_pass\b"
)
_DEFECT_WORDS_RE = re.compile(r"(?i)\bdefect\b|\bnot\s+evidence\b|\bblocked\b|\bforbidden\b|\bviolat")

# Body text that says "a model produced numbers" (the D22 gate trigger).
_RUN_WORDS_RE = re.compile(
    r"(?i)\b(trained|training run|fine-?tuned|fit|fits|evaluated|evaluation|scored|ran|run|runs|steps|seeds)\b"
)
_NUMBER_RE = re.compile(r"\d")

_PROSE_HEADLINE_WORDS = re.compile(
    r"(?i)\b(improved|better|worse|unchanged|moved|higher|lower|up|down|same|flat|n/?a|pending|tbd|none)\b"
)


# ----------------------------------------------------------------------------- model


@dataclass
class Headline:
    """One side of `headline before -> after`, as numbers when they are numbers."""

    label: str = ""
    value: Any = None
    of: Any = None
    text: str = ""

    @property
    def numeric(self) -> bool:
        return isinstance(self.value, (int, float))

    def render(self) -> str:
        if not self.numeric:
            return self.text.strip()
        of = f"/{self.of}" if self.of not in (None, "") else ""
        return f"{self.label} {_fmt_num(self.value)}{of}".strip()


@dataclass
class ChecklistItem:
    label: str
    checked: bool


@dataclass
class Problem:
    rule: str
    severity: str
    message: str
    fix: str = ""

    def as_dict(self) -> dict[str, str]:
        return asdict(self)


@dataclass
class PRFields:
    """What the body actually declares. Empty strings mean 'not declared'."""

    line_name: str = ""
    line_count: int | None = None
    line_of: int | None = None
    headline_label: str = ""
    headline_before: Headline = field(default_factory=Headline)
    headline_after: Headline = field(default_factory=Headline)
    milestone_ref: str = ""
    milestone_issue: int | None = None
    milestone_issues: list[int] = field(default_factory=list)
    decision: str = ""
    record_paths: list[str] = field(default_factory=list)
    record_path: str = ""
    references: list[int] = field(default_factory=list)
    closes: list[int] = field(default_factory=list)
    checklist: list[ChecklistItem] = field(default_factory=list)
    prereg: str = ""
    serials: list[str] = field(default_factory=list)
    softmax_claims: list[str] = field(default_factory=list)
    compute_claims: list[str] = field(default_factory=list)
    cpu_claims: list[str] = field(default_factory=list)
    command_hits: int = 0
    has_pivot: bool = False
    claims_current_state: bool = False
    body_text: str = ""
    raw: str = ""

    @property
    def count_text(self) -> str:
        if self.line_count is None:
            return ""
        of = self.line_of if self.line_of is not None else 3
        return f"{self.line_count}/{of}"

    @property
    def checked(self) -> int:
        return sum(1 for c in self.checklist if c.checked)


def _fmt_num(value: Any) -> str:
    if isinstance(value, float):
        return f"{value:g}"
    return str(value)


def _norm(text: str) -> str:
    return " ".join(re.split(r"[^a-z0-9]+", text.lower())).strip()


# --------------------------------------------------------------------------- parsing


def strip_comments(body: str) -> str:
    """Remove HTML comments, and with them any unrendered template instructions."""
    return _COMMENT_RE.sub(" ", body or "")


def parse_pr_body(body: str) -> dict[str, Any]:
    """Extract the fillable PR fields from a body (rendered, or the raw template).

    Tolerates the template's HTML comments and both `·` and `-` separators. Never
    raises: anything unreadable simply comes back empty, and `check_pr` reports it.
    """
    raw = body or ""
    clean = strip_comments(raw)
    clean = clean.replace("\r\n", "\n").replace("\r", "\n")
    prose = _prose(clean)
    fields = PRFields(raw=raw, body_text=clean)

    # --- the Line: declaration (session-goal §3)
    line_value = ""
    m = re.search(r"(?im)^[ \t>*-]*line\s*[:\-][ \t]*(.*)$", clean)
    line_region = ""
    if m:
        line_value = m.group(1).strip()
        after = clean[m.end() :]
        stop = re.search(r"(?m)^\s{0,3}#{1,6}\s+\S", after)
        line_region = after[: stop.start()] if stop else after
        if not line_region.strip():
            # value wrapped onto the next line
            nxt = re.search(r"(?m)^[ \t]*(\S.*)$", after)
            if nxt and re.search(r"(?i)\bcount\b", nxt.group(1)):
                line_region = nxt.group(1).strip()
    count_m = _COUNT_RE.search(line_value) or _COUNT_RE.search(line_region)
    if count_m and count_m.group("n"):
        fields.line_count = int(count_m.group("n"))
        fields.line_of = int(count_m.group("of")) if count_m.group("of") else None

    prefix = line_value
    if count_m and count_m.start() < len(line_value):
        prefix = line_value[: count_m.start()]
    segments = [s.strip(" \t:*-") for s in _SEP_RE.split(prefix) if s.strip(" \t:*-")]
    if segments:
        name = segments[0]
        name = re.sub(r"(?i)^line\s*[:\-]\s*", "", name).strip()
        if name and not re.fullmatch(r"(?i)(n/?a|tbd|none|milestone|line)", name) and not _COUNT_SEGMENT_RE.match(name):
            fields.line_name = name

    fields.headline_before, fields.headline_after, hl_label = _parse_headline(line_value, line_region)
    fields.headline_label = hl_label

    # --- milestone reference (#2029-#2036), preferring the milestone the body is about
    issues = [int(i) for i in _ISSUE_RE.findall(clean)]
    candidates = _dedupe([i for i in issues if i in MILESTONE_ISSUES])
    fields.milestone_issues = candidates
    if candidates:
        fields.milestone_issue = candidates[0]
        fields.milestone_ref = f"#{candidates[0]}"
    elif issues:
        fields.milestone_ref = f"#{issues[0]}"

    # --- decision
    upper = clean.upper()
    for decision in ("NOT YET PROMOTED", "KEEP", "REJECT"):
        if decision in upper:
            fields.decision = decision
            break

    # --- records
    for path in _LAB_RECORD_RE.findall(clean):
        fields.record_paths.append(f"docs/labs/{path}/README.md")
    fields.record_paths.extend(_EVIDENCE_RE.findall(clean))
    fields.record_paths.extend(_NOTEBOOK_RE.findall(clean))
    fields.record_paths = _dedupe(fields.record_paths)
    # A dated record first (unchanged), then the line's notebook page (Q4).
    fields.record_path = next((p for p in fields.record_paths if re.search(r"\d{4}-\d{2}-\d{2}", p)), "") or next(
        (p for p in fields.record_paths if _NOTEBOOK_RE.fullmatch(p)), ""
    )

    # --- issue references
    for m2 in re.finditer(r"(?i)\b(References|Closes|Fixes)\s*#?(\d{3,6})", clean):
        kind, num = m2.group(1).lower(), int(m2.group(2))
        (fields.references if kind == "references" else fields.closes).append(num)
    fields.references = _dedupe(fields.references)
    fields.closes = _dedupe(fields.closes)

    # --- checklist
    for c in _BOX_RE.finditer(clean):
        label = " ".join(c.group("label").split())
        fields.checklist.append(ChecklistItem(label=truncate(label, 90), checked=c.group("state").lower() == "x"))

    # --- pre-registration (D22), from prose only: an unticked template box is
    # not the author asserting that a pre-registration exists.
    for marker in _PREREG_MARKERS:
        hit = marker.search(prose)
        if hit:
            fields.prereg = hit.group(0)
            break
    fields.serials = _dedupe(re.findall(r"\b\d{9,12}\b", prose))

    # --- softmax claims (D23) and compute claims (D22/D24)
    fields.softmax_claims = _dedupe([m3.group(0) for m3 in _SOFTMAX_PHRASES.finditer(clean)])
    for line in prose.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if _RUN_WORDS_RE.search(stripped) and _NUMBER_RE.search(stripped):
            fields.compute_claims.append(truncate(stripped, 120))
    fields.cpu_claims = _cpu_claims(prose)
    fields.command_hits = len(_CMD_RE.findall(prose))

    fields.has_pivot = bool(re.search(r"(?i)pivot\s*(?:card|recorded|posted|point|line)", clean))
    fields.claims_current_state = _claims_current_state(prose, fields.checklist)
    return _to_dict(fields)


def _prose(clean: str) -> str:
    """The body without its checklist lines."""
    return _CHECKBOX_LINE_RE.sub("", clean)


def _claims_current_state(prose: str, checklist: list[ChecklistItem]) -> bool:
    """A claim is a *ticked* current-state checklist item, or a statement that names
    the file with the entry (``docs/integration/current-state.md`` - newest-first entry)."""
    if any(c.checked and "current-state.md" in c.label for c in checklist):
        return True
    for line in prose.splitlines():
        if "current-state.md" not in line:
            continue
        if re.search(r"(?i)\b(entry|entries|newest|top|added|updated|new)\b", line):
            return True
    return False


def _has_result_row(text: str) -> bool:
    """A notebook result row (Q4): a markdown table row whose first cell is a ``YYYY-MM-DD`` date.

    Mirrors ``prgates::has_result_row`` in the Rust harness."""
    for line in text.split("\n"):
        line = line.strip()
        if not line.startswith("|"):
            continue
        if _DATED_CELL_RE.fullmatch(line[1:].split("|", 1)[0].strip()):
            return True
    return False


def _research_compute_claims(claims: list[Any]) -> list[Any]:
    """Q2: drop build, finite-kernel-test and host-check lines that are not about training/evaluation."""
    return [c for c in claims if not (_HOST_CHECK_RE.search(str(c)) and not _RESEARCH_RUN_RE.search(str(c)))]


def _research_cpu_claims(claims: list[Any]) -> list[Any]:
    """Q1: drop lines that serve or benchmark the exported D11 artifact on a client device."""
    return [
        c
        for c in claims
        if _HOST_FALLBACK_RE.search(str(c))
        or not (_DEVICE_SERVING_RE.search(str(c)) and not _RESEARCH_CPU_RE.search(str(c)))
    ]


def _dedupe(items: list[Any]) -> list[Any]:
    out: list[Any] = []
    for item in items:
        if item not in out:
            out.append(item)
    return out


def _cpu_claims(text: str) -> list[str]:
    """D24: a CPU/laptop model run offered as an accepted result."""
    out: list[str] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        hit = _CPU_COMPUTE_RE.search(stripped)
        if not hit:
            continue
        tail = stripped[hit.end() : hit.end() + 160]
        accepted = _QUALITY_NEAR_RE.search(tail) or _QUALITY_NEAR_RE.search(stripped)
        if accepted and not _DEFECT_WORDS_RE.search(stripped):
            out.append(truncate(stripped, 120))
    return _dedupe(out)


def _split_value(raw: str) -> str:
    """Cut a value at the next separator, or at a label such as 'decision:'."""
    cut = len(raw)
    sep = _SEP_RE.search(raw)
    if sep:
        cut = min(cut, sep.start())
    lab = re.search(r"\s+(?=[A-Z][A-Za-z ]{2,18}:)", raw)
    if lab:
        cut = min(cut, lab.start())
    return raw[:cut].strip(" \t:*-")


def _parse_headline(line_value: str, region: str) -> tuple[Headline, Headline, str]:
    src = line_value if _ARROW_RE.search(line_value) or _HEADLINE_LABEL_RE.search(line_value) else region
    if not src:
        return Headline(), Headline(), ""
    hl = _HEADLINE_LABEL_RE.search(src)
    label = ""
    if hl:
        tail = src[hl.end() :]
        lab = re.search(r"([A-Za-z][A-Za-z0-9]*)\s*\d+\s*/\s*\d+", tail)
        if lab:
            label = lab.group(1)
        src = tail
    parts = _ARROW_RE.split(src, maxsplit=1)
    if len(parts) == 2:
        before_text = _split_value(parts[0])
        after_text = _split_value(parts[1])
    else:
        before_text = _split_value(src)
        after_text = ""
    before = _make_headline(label, before_text)
    after = _make_headline(label, after_text)
    if not label:
        label = before.label or after.label
    return before, after, label


def _make_headline(label: str, text: str) -> Headline:
    text = (text or "").strip()
    if not text:
        return Headline(label=label)
    m = re.search(r"(\d+(?:\.\d+)?)\s*(?:/\s*(\d+))?", text)
    if not m:
        return Headline(label=label, text=_split_value(text))
    value: Any = float(m.group(1)) if "." in m.group(1) else int(m.group(1))
    of: Any = int(m.group(2)) if m.group(2) else None
    return Headline(label=label, value=value, of=of, text=_split_value(text))


def _to_dict(fields: PRFields) -> dict[str, Any]:
    data = asdict(fields)
    data["count_text"] = fields.count_text
    data["checked"] = fields.checked
    return data


def _as_headline(value: Any) -> Headline:
    """`parse_pr_body` returns plain dicts; checks want the Headline view back."""
    if isinstance(value, Headline):
        return value
    if isinstance(value, dict):
        return Headline(
            label=str(value.get("label") or ""),
            value=value.get("value"),
            of=value.get("of"),
            text=str(value.get("text") or ""),
        )
    return Headline()


def _checked_items(fields: dict[str, Any]) -> list[ChecklistItem]:
    out: list[ChecklistItem] = []
    for item in fields.get("checklist") or []:
        if isinstance(item, ChecklistItem):
            out.append(item)
        elif isinstance(item, dict):
            out.append(ChecklistItem(label=str(item.get("label") or ""), checked=bool(item.get("checked"))))
    return out


# -------------------------------------------------------------------------- lookups


def _find_milestone(reg: Registry, fields: dict[str, Any]) -> Milestone | None:
    """The milestone the body is about: the issue whose line or headline is declared.

    A body may cite several milestone issues (it often cites the tracker too). Score
    the candidates against the declared line name and headline rather than trusting
    the first number that appears.
    """
    issues = [int(i) for i in (fields.get("milestone_issues") or [])]
    if fields.get("milestone_issue") is not None and int(fields["milestone_issue"]) not in issues:
        issues.insert(0, int(fields["milestone_issue"]))
    declared = str(fields.get("line_name") or "")
    body = str(fields.get("body_text") or "")
    headline_values = [
        h.get("value")
        for h in (fields.get("headline_before") or {}, fields.get("headline_after") or {})
        if isinstance(h, dict)
    ]

    best: tuple[int, Milestone] | None = None
    for issue in issues:
        milestone = next((m for m in reg.milestones.all() if m.issue == issue), None)
        if milestone is None:
            continue
        score = 0
        if declared and _find_line(milestone, declared) is not None:
            score += 3
        if milestone.id.lower() in body.lower():
            score += 1
        _, current, _ = _registry_headline(milestone)
        if current is not None and any(_same_number(v, current) for v in headline_values if v is not None):
            score += 2
        if best is None or score > best[0]:
            best = (score, milestone)
    if best is not None:
        return best[1]

    ref = str(fields.get("milestone_ref") or "").lstrip("#")
    if ref:
        m = reg.milestones.get(ref)
        if m is not None:
            return m
    name = _norm(str(fields.get("line_name") or ""))
    if name:
        for m in reg.milestones.all():
            if m.id.lower() in name:
                return m
    return None


def _find_line(milestone: Milestone | None, declared: str) -> dict[str, Any] | None:
    if milestone is None:
        return None
    want = _norm(declared)
    if not want:
        return None
    lines = [l for l in (milestone.lines or []) if isinstance(l, dict)]
    for line in lines:
        if _norm(str(line.get("name", ""))) == want:
            return line
    for line in lines:
        other = _norm(str(line.get("name", "")))
        if other and (other in want or want in other):
            return line
    want_tokens = {t for t in want.split() if len(t) > 3}
    best: tuple[int, dict[str, Any]] | None = None
    for line in lines:
        other_tokens = {t for t in _norm(str(line.get("name", ""))).split() if len(t) > 3}
        score = len(want_tokens & other_tokens)
        if score and (best is None or score > best[0]):
            best = (score, line)
    return best[1] if best else None


def _registry_headline(milestone: Milestone) -> tuple[str, Any, Any]:
    h = milestone.headline or {}
    return str(h.get("label", "")), h.get("current"), h.get("of")


def _same_number(a: Any, b: Any) -> bool:
    try:
        return abs(float(a) - float(b)) < 1e-9
    except (TypeError, ValueError):
        return a == b


def _target_matches_registry(after: Headline, milestone: Milestone) -> bool:
    label, current, of = _registry_headline(milestone)
    if not after.numeric or current is None:
        return False
    if not _same_number(after.value, current):
        return False
    return of in (None, "") or after.of in (None, "") or _same_number(after.of, of)


# --------------------------------------------------------------------------- checks


def check_pr(body: str, paths: HarnessPaths, reg: Registry) -> dict[str, Any]:
    """Check a PR body. Returns ok / problems / summary / fields. Never raises."""
    try:
        fields = parse_pr_body(body)
    except Exception as exc:  # noqa: BLE001 - a lint tool never breaks a run
        return {
            "ok": False,
            "problems": [Problem("prlint.parse", "error", f"body could not be parsed: {type(exc).__name__}: {exc}", "check the body text").as_dict()],
            "summary": "lint: body unreadable",
            "fields": {},
        }

    milestone = _find_milestone(reg, fields)
    declared = str(fields.get("line_name") or "")
    line = _find_line(milestone, declared) if declared else None
    before = _as_headline(fields.get("headline_before"))
    after = _as_headline(fields.get("headline_after"))
    count = fields.get("line_count")

    problems: list[Problem] = []
    add = problems.append

    # --- 1. a line and a count are declared (session-goal §3)
    missing: list[str] = []
    if not declared:
        missing.append("Line name")
    if count is None:
        missing.append("count n/3")
    if missing:
        add(
            Problem(
                "prlint.line_declared",
                "error",
                f"the body does not declare {', '.join(missing)}",
                "Line: <name> · count n/3 · headline <before> → <after>",
            )
        )

    # --- 2. the declared count against the registry
    if count is not None and line is not None:
        reg_count = line.get("count")
        if isinstance(reg_count, int) and count < reg_count:
            add(
                Problem(
                    "prlint.count_matches_registry",
                    "warn",
                    f"declares {count}/{line.get('of', 3)} but docs/harness/milestones.json has {reg_count}/{line.get('of', 3)} for line '{line.get('name')}'",
                    "declare the current count, or move the headline and reset it explicitly (D21: only a moved headline resets)",
                )
            )
    elif count is not None and milestone is not None and (milestone.lines or []) and declared:
        names = ", ".join(str(l.get("name")) for l in milestone.lines if isinstance(l, dict))
        add(
            Problem(
                "prlint.count_matches_registry",
                "warn",
                f"line {declared!r} is not in {milestone.id}'s registry lines ({names})",
                f"rename the line to one of: {names} - renaming does not reset the count (D21)",
            )
        )

    # --- 3. headline present, as numbers
    if not after.numeric:
        add(
            Problem(
                "prlint.headline_present",
                "error",
                "no `headline <before> → <after>` with numbers (prose is not a headline)",
                "headline " + (before.render() if before.numeric else "<number/of>") + " → <number/of>",
            )
        )

    # --- 4. the after value against the registry
    if after.numeric:
        label, current, of = _registry_headline(milestone) if milestone else ("", None, None)
        stated_move = before.numeric and not _same_number(before.value, after.value)
        if milestone is not None and current is not None and not _target_matches_registry(after, milestone) and not stated_move:
            suffix = f"/{of}" if of else ""
            add(
                Problem(
                    "prlint.headline_vs_registry",
                    "warn",
                    f"after {after.render()} does not match the registry headline for {milestone.id} #{milestone.issue} ({label} {_fmt_num(current)}{suffix}) and no move is stated",
                    f"state the move (`headline {before.render()} → after`) or correct the registry in docs/harness/milestones.json",
                )
            )

    # --- 5. 3/3 forces a pivot card (D21 §1)
    reg_count = line.get("count") if isinstance(line, dict) else None
    at_three = (isinstance(count, int) and count >= 3) or (isinstance(reg_count, int) and reg_count >= 3)
    if at_three and not fields.get("has_pivot"):
        where = f"{count}/3 declared" if isinstance(count, int) and count >= 3 else f"registry count {reg_count}/3"
        add(
            Problem(
                "prlint.pivot_at_three",
                "error",
                f"line '{declared or '?'}' is at {where} and the body announces no pivot",
                f"this PR is a pivot card on #{milestone.issue if milestone else '<milestone issue>'}: stop and archive the line, or name the one decisive run (D21); then set count 0/3",
            )
        )

    # --- 6. a decision (CONTRIBUTING § Delivery cadence)
    if not fields.get("decision"):
        add(
            Problem(
                "prlint.decision_present",
                "error",
                "no KEEP / REJECT / NOT YET PROMOTED decision in the body",
                "add `Decision: KEEP` (or REJECT, or NOT YET PROMOTED) to the Result section",
            )
        )

    # --- 7. the record path: a notebook row (Q4), a dated record, or archived evidence
    record = str(fields.get("record_path") or "")
    if not record:
        add(
            Problem(
                "prlint.record_path",
                "warn",
                "no record path (docs/research/notebook/<line>.md, docs/labs/<topic>-<YYYY-MM-DD>/README.md or archive/evidence/…) is named",
                "add one result row to docs/research/notebook/<line>.md and name it in the checklist (or a dated record docs/labs/<topic>-"
                + _today()
                + "/README.md when the run seals artifacts)",
            )
        )
    elif not paths.repo.joinpath(record).exists():
        add(
            Problem(
                "prlint.record_path",
                "warn",
                f"record path {record} is named but does not exist on disk",
                f"write {record} in this PR (delivery cadence) or correct the path",
            )
        )
    else:
        # Every named notebook must carry a dated row, not only the chosen record: a dated artifact
        # path wins record_path, and must not hide a notebook that was never written (review of #2798).
        for notebook in fields.get("record_paths") or []:
            if not _NOTEBOOK_RE.fullmatch(str(notebook)):
                continue
            try:
                text = paths.repo.joinpath(notebook).read_bytes().decode("utf-8", errors="replace")
            except OSError:
                text = ""
            if not _has_result_row(text):
                add(
                    Problem(
                        "prlint.record_path",
                        "warn",
                        f"notebook {notebook} is named but has no dated result row",
                        "append the result as one row (Date | What ran | Number | Bar | Verdict | Artifact)",
                    )
                )

    # --- 8. the current-state entry
    if fields.get("claims_current_state"):
        issue = fields.get("milestone_issue")
        topic = ""
        if record:
            parts = Path(record).parts
            if "labs" in parts:
                idx = parts.index("labs")
                if idx + 1 < len(parts):
                    topic = parts[idx + 1]
        text = _current_state_text(paths)
        if text is None:
            add(
                Problem(
                    "prlint.current_state_entry",
                    "warn",
                    "the body claims a docs/integration/current-state.md entry, but that file is missing",
                    "add the file with a newest-first entry, or drop the claim",
                )
            )
        else:
            hit = ""
            if issue is not None and re.search(rf"#{issue}\b|\b{issue}\b", text):
                hit = f"#{issue}"
            elif topic and topic.lower() in text.lower():
                hit = topic
            if not hit:
                what = f"#{issue}" if issue is not None else (topic or "the record topic")
                add(
                    Problem(
                        "prlint.current_state_entry",
                        "warn",
                        f"the body claims a current-state entry but docs/integration/current-state.md does not mention {what}",
                        f"add the newest-first entry citing {what} to docs/integration/current-state.md, or drop the claim",
                    )
                )
            elif not issue and not topic:
                add(
                    Problem(
                        "prlint.current_state_entry",
                        "warn",
                        "the body claims a current-state entry, but neither a milestone issue nor a dated record topic is named to check it against",
                        "name the milestone (#2029-#2036) or the record topic in the body",
                    )
                )

    # --- 9. D22: pre-registration before compute
    compute_claims = _research_compute_claims(fields.get("compute_claims") or [])
    if compute_claims and not fields.get("prereg"):
        sample = truncate(str(compute_claims[0]), 80)
        add(
            Problem(
                "prlint.prereg_before_compute",
                "error",
                f"the body reports model numbers ({sample}) with no pre-registration reference: no TEST FITNESS verdict, no `#issuecomment-<id>` milestone comment, no prep sheet id",
                "post the bar, base artifact, data split, steps and cost on the milestone issue first, then cite it here (`Prereg: #issuecomment-<id>`, `TEST FITNESS: FIT`) - D22 §5d/§5e",
            )
        )

    # --- 10. D23 softmax interlock (delegated)
    softmax_problems = _check_softmax(paths, fields)
    problems.extend(softmax_problems)

    # --- 11. D24 no CPU model compute
    cpu_claims = _research_cpu_claims(fields.get("cpu_claims") or [])
    if cpu_claims:
        sample = truncate(str(cpu_claims[0]), 90)
        add(
            Problem(
                "prlint.no_cpu_model_compute",
                "warn",
                f"the body presents CPU or host-fallback model compute as an accepted result: {sample}",
                "run model compute on GPU via scripts/pod/uor-pod and fix a host-side fallback with a kernel; CPU holds data preparation and grading only (D24)",
            )
        )

    # --- 12. exact-head checks
    checks_box = any(c.checked and re.search(r"(?i)exact head|checks? run", c.label) for c in _checked_items(fields))
    if not checks_box and not fields.get("command_hits"):
        add(
            Problem(
                "prlint.checks_at_head",
                "warn",
                "the exact-head-checks box is not ticked and the body lists no commands",
                "tick `Checks run at this exact head` and paste the commands with their outcomes",
            )
        )

    # --- 13. org rules
    if not fields.get("references") and not fields.get("closes"):
        add(
            Problem(
                "prlint.org_rules",
                "info",
                "no `References #N` or `Closes #N` line, so the PR is not linked to its issue",
                "end the body with `References #<issue>` (or `Closes #<issue>` only when the acceptance is complete)",
            )
        )

    errors = [p for p in problems if p.severity == "error"]
    warnings = [p for p in problems if p.severity == "warn"]
    summary = (
        f"lint {declared or '(no line)'} {fields.get('count_text') or 'n/3'}: "
        f"{'OK' if not errors else 'FAIL'} - {len(errors)} error, {len(warnings)} warn"
    )
    if not problems:
        summary += " (contract complete)"
    return {
        "ok": not errors,
        "problems": [p.as_dict() for p in problems],
        "summary": summary,
        "fields": fields,
    }


def _today() -> str:
    from core import today

    return today()


def _current_state_text(paths: HarnessPaths) -> str | None:
    p = paths.current_state_md
    if not p.exists():
        return None
    try:
        return p.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None


def _check_softmax(paths: HarnessPaths, fields: dict[str, Any]) -> list[Problem]:
    """D23: delegate to scripts/check_no_softmax.py. Never reimplement its scan."""
    claims: list[str] = list(fields.get("softmax_claims") or [])
    rel = "../check_no_softmax.py"
    script = paths.repo / "scripts" / "check_no_softmax.py"
    if not script.exists():
        if not claims:
            return []
        return [
            Problem(
                "prlint.softmax_ban",
                "error",
                f"the body names softmax ({', '.join(claims[:3])}) and the D23 interlock scripts/check_no_softmax.py is missing",
                "restore scripts/check_no_softmax.py and run it: python3 scripts/check_no_softmax.py",
            )
        ]

    module = _load_softmax_module(script)
    if module is None or not callable(getattr(module, "scan", None)):
        return [
            Problem(
                "prlint.softmax_ban",
                "error",
                "cannot call the D23 interlock; run it directly: python3 scripts/check_no_softmax.py",
                "python3 scripts/check_no_softmax.py",
            )
        ]

    try:
        current, baseline = _softmax_scan(module, script)
    except Exception as exc:  # noqa: BLE001
        return [
            Problem(
                "prlint.softmax_ban",
                "error",
                f"scripts/check_no_softmax.py could not run: {type(exc).__name__}: {exc}",
                "python3 scripts/check_no_softmax.py",
            )
        ]
    growth = sorted(p for p, n in current.items() if n > baseline.get(p, 0))
    out: list[Problem] = []
    if growth:
        listed = ", ".join(growth[:3]) + ("…" if len(growth) > 3 else "")
        out.append(
            Problem(
                "prlint.softmax_ban",
                "error",
                f"scripts/check_no_softmax.py reports {len(growth)} file(s) with softmax above baseline: {listed}",
                "python3 scripts/check_no_softmax.py; replace the site with a geometric mechanism, then lower the baseline with --update-baseline",
            )
        )
    elif claims and not re.search(r"(?i)\bno\b|\bnot\b|\bwithout\b|\bremov|\breplac|\bban\b|\bprohibit|\bforbid|\bcensus\b", " ".join(claims)):
        out.append(
            Problem(
                "prlint.softmax_ban",
                "info",
                f"the body names softmax ({', '.join(claims[:3])}); the ratchet is clean, but D23 requires the whole executed path to be traced",
                f"run python3 scripts/check_no_softmax.py and scripts/softmax_path_check.py; cite `{rel}` in the body",
            )
        )
    return out


def _gate_ids() -> tuple[str, ...]:
    """The gate names `check_pr` actually raises, read out of this module's source.

    `prlint.gates_declared` compares these to `LOGICAL_RULES`, so renaming a gate
    cannot leave a lesson citing an id that no longer exists.
    """
    import ast

    source = Path(__file__).read_text(encoding="utf-8")
    tree = ast.parse(source)
    found: set[str] = set()
    for node in ast.walk(tree):
        names: set[str] = set()
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            names = {node.name}
        elif isinstance(node, ast.Assign):
            names = {t.id for t in node.targets if isinstance(t, ast.Name)}
        if not (names & {"check_pr", "_check_softmax"}):
            continue
        for inner in ast.walk(node):
            if isinstance(inner, ast.Constant) and isinstance(inner.value, str) and inner.value.startswith("prlint."):
                found.add(inner.value.split(".", 1)[1])
    return tuple(sorted(found))


_GATE_IDS = _gate_ids()


_SOFTMAX_MODULES: dict[str, Any] = {}
_SOFTMAX_SCANS: dict[tuple[str, int], tuple[dict[str, int], dict[str, int]]] = {}


def _load_softmax_module(script: Path) -> Any:
    """Import the interlock by path so its scan is called, not copied."""
    key = str(script)
    if key in _SOFTMAX_MODULES:
        return _SOFTMAX_MODULES[key]
    try:
        spec = importlib.util.spec_from_file_location("check_no_softmax", script)
        if spec is None or spec.loader is None:
            return None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception:  # noqa: BLE001 - reported as a problem by the caller
        return None
    _SOFTMAX_MODULES[key] = module
    return module


def _softmax_scan(module: Any, script: Path) -> tuple[dict[str, int], dict[str, int]]:
    """Cached because the ratchet walks crates/**; keyed on the script's mtime."""
    try:
        mtime = int(script.stat().st_mtime)
    except OSError:
        mtime = 0
    key = (str(script), mtime)
    hit = _SOFTMAX_SCANS.get(key)
    if hit is None:
        hit = (module.scan(), module.load_baseline())
        _SOFTMAX_SCANS.clear()
        _SOFTMAX_SCANS[key] = hit
    return hit


# -------------------------------------------------------------------------- rendering

_CHECKLIST = (
    ("Checks run at this exact head (commands and outcomes posted on this PR)", False),
    ("Result recorded: one row in docs/research/notebook/<line>.md (a dated docs/labs/<topic>-<YYYY-MM-DD>/ folder only when a run seals artifacts)", False),
    ("docs/integration/NOW.md updated if the next action changed", False),
    ("Every document this change makes stale is updated", False),
    ("STATUS.md / ROADMAP.md / #2028 table updated, or state unchanged", False),
    ("README updated in its existing sections, or nothing a reader relies on changed", False),
    ("cargo run -q --locked -p uor-r4-claim-wording -- . passes", False),
    ("After merge: verify on fresh origin/main, delete the branch and worktree, uor-hygiene --apply", False),
)


def format_pr_body(fields: dict[str, Any]) -> str:
    """Render a compliant PR body from parsed fields (`lint format`)."""
    f = fields or {}
    before = _headline_from(f.get("headline_before"))
    after = _headline_from(f.get("headline_after"))
    count = f.get("line_count")
    of = f.get("line_of") or 3
    line_name = str(f.get("line_name") or "<line name>")
    count_text = f"{count}/{of}" if isinstance(count, int) else f"<n>/{of}"
    headline = f"{before or '<number/of>'} → {after or '<number/of>'}"

    milestone = f.get("milestone_ref") or ""
    decision = str(f.get("decision") or "NOT YET PROMOTED")
    record = str(f.get("record_path") or "docs/labs/<topic>-<YYYY-MM-DD>/README.md")
    prereg = str(f.get("prereg") or "")
    issues = [i for i in (f.get("references") or []) + (f.get("closes") or [])]
    ref = f"References #{issues[0]}" if issues else "References #<issue>"

    lines = [
        "## Line and headline (D21, CHARTER.md)",
        "",
        f"Line: {line_name} · count {count_text} · headline {headline}",
        "",
        "## What and why",
        "",
        str(f.get("what") or "<the change, the question it answers" + (f", the milestone ({milestone})" if milestone else "") + ">"),
        "",
        "## Result",
        "",
        f"Decision: {decision}",
        "",
    ]
    if prereg:
        lines += [f"Pre-registration: {prereg}", ""]
    body_tail = [
        "## Checklist",
        "",
    ]
    for label, _ in _CHECKLIST:
        if label.startswith("Result recorded") and "<" not in record:
            label = f"{label} - `{record}`"
        body_tail.append(f"- [ ] {label}")
    body_tail += ["", ref, ""]
    return redact("\n".join(lines + body_tail))


def _headline_from(value: Any) -> str:
    if isinstance(value, Headline):
        return value.render() if value.numeric else value.text.strip()
    if isinstance(value, dict):
        return Headline(
            label=str(value.get("label", "")),
            value=value.get("value"),
            of=value.get("of"),
            text=str(value.get("text", "")),
        ).render()
    return ""


# ------------------------------------------------------------------------------- main


def register_rules() -> list[Any]:
    """The PR gates, registered so their ids resolve without a body in hand.

    A PR body is an argument, not a file in the tree, so `checkrules.run` cannot
    evaluate these gates from the repo alone. They are still registered: the id
    must exist for a lesson to cite it as its `guard` (`mistakes.guard_covered`
    resolves against `checkrules.rules()`), and `prlint.softmax_ban` has a real
    repo-scope job - the D23 interlock must be present and callable.
    """
    from checkrules import Rule, RuleResult

    declared = ", ".join(LOGICAL_RULES)
    checks = {rid: _gate_pattern(rid) for rid in GATE_RULES}

    def check_gates(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
        missing = [rid for rid, pattern in checks.items() if pattern is None]
        if missing:
            return [
                RuleResult(
                    rule_id="prlint.gates_declared",
                    severity="error",
                    ok=False,
                    message=f"gate(s) declared but not implemented: {', '.join(missing)}",
                    path="scripts/harness/prlint.py",
                    fix="implement the gate in check_pr() or remove it from LOGICAL_RULES",
                )
            ]
        return [
            RuleResult(
                rule_id="prlint.gates_declared",
                severity="info",
                ok=True,
                message=f"{len(checks)} PR gates declared; run `lint check --body-file <body>` to evaluate one: {declared}",
            )
        ]

    def check_interlock(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
        script = paths.repo / "scripts" / "check_no_softmax.py"
        if not script.exists():
            return [
                RuleResult(
                    rule_id="prlint.softmax_ban",
                    severity="error",
                    ok=False,
                    message="the D23 interlock scripts/check_no_softmax.py is missing",
                    path="scripts/check_no_softmax.py",
                    fix="restore the interlock: D23 (no softmax) is enforced by it",
                )
            ]
        module = _load_softmax_module(script)
        if module is None or not callable(getattr(module, "scan", None)):
            return [
                RuleResult(
                    rule_id="prlint.softmax_ban",
                    severity="error",
                    ok=False,
                    message="scripts/check_no_softmax.py exists but exposes no callable scan()",
                    path="scripts/check_no_softmax.py",
                    fix="run it directly (python3 scripts/check_no_softmax.py) and fix the script",
                )
            ]
        return []

    return [
        Rule(
            id="prlint.gates_declared",
            description=f"every declared PR gate ({declared}) is implemented",
            severity="error",
            check=check_gates,
            rationale="an id a lesson cites as its guard must exist, or the lesson is a note",
        ),
        Rule(
            id="prlint.softmax_ban",
            description="the D23 softmax interlock exists and is callable, and is delegated to, not reimplemented",
            severity="error",
            check=check_interlock,
            rationale="D23; scripts/check_no_softmax.py is the interlock and prlint must call it",
        ),
    ]


def _gate_pattern(rule_id: str) -> Any:
    """A gate is implemented when the code that raises its id is still present."""
    needle = rule_id.split(".", 1)[-1]
    return needle if needle in _GATE_IDS else None


def main(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    """`lint check` / `lint format`. Prints its own output; returns an exit code."""
    from core import emit

    body = ""
    body_file = getattr(args, "body_file", None)
    if body_file:
        try:
            body = Path(body_file).read_text(encoding="utf-8")
        except OSError as exc:
            emit(f"cannot read {body_file}: {exc}", sys.stderr)
            return 2
    elif getattr(args, "body", None):
        body = str(args.body)
    else:
        emit("provide --body or --body-file", sys.stderr)
        return 2

    action = getattr(args, "action", "check") or "check"
    if action == "format":
        emit(format_pr_body(parse_pr_body(body)))
        return 0

    result = check_pr(body, paths, reg)
    if getattr(args, "json", False):
        emit(json.dumps(result, indent=2, default=str))
    else:
        emit(result["summary"])
        for problem in result["problems"]:
            emit(f"  [{problem['severity']}] {problem['rule']}: {problem['message']}")
            if problem.get("fix"):
                emit(f"      fix: {problem['fix']}")
    return 0 if result["ok"] else 1
