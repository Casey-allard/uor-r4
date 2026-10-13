"""The regression arena: the lineage of evaluated geometric configurations.

Nobody here is racing transformers. D23/D25 forbid prohibited comparators, and the owner's
3 October direction withdrew transformer controls. What this project actually needs is the
opposite of a leaderboard: **a regression check**. Every candidate is an edit to the geometry,
and the question is never "is it better than the last one on average" but "what did it break,
and what did it fix, on the frozen panels".

So the arena is a lineage, not a ranking:

* every evaluated configuration is one **row**, with the scope it was measured at;
* each row names its **parent** - the configuration it was derived from - so the records form
  the actual tree of how the model evolved;
* the report is **row by row against the parent**, never an aggregate. A candidate that gains
  0.02 BPB while losing three memory rows is a regression with a headline, and averaging
  those away is exactly the failure the historical pitfalls warn about.

Scope discipline is enforced, not hoped for. A run with unknown noise, one seed, or a bolted-on
mechanism is recorded as such and cannot be read as decisive. The arena reports `UNAVAILABLE`
for anything it cannot compare and never invents a delta.

Read-only: this module reports. It does not run a model, choose a candidate, or retire anything.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from core import HarnessPaths, emit
from checkrules import Rule, RuleResult, register as register_rule
from registries import Registry
from util import read_text, truncate

HISTORY = "docs/harness/arena.json"

#: verdicts a row may carry. "NOT_PROMOTED" is a real outcome, not a failure.
VERDICTS = ("KEEP", "REJECT", "NOT_YET_PROMOTED", "NOT_RUN", "UNAVAILABLE")
#: how much a row can be trusted to support a decision
CONFIDENCE = ("decisive", "provisional", "scoped-reading", "uninterpretable")


@dataclass
class Candidate:
    id: str
    artifact: str
    milestone: str
    date: str
    parent: str | None
    verdict: str
    confidence: str
    mechanism: str
    panel: str
    seeds: int | None
    rows: int | None
    scope: str
    scores: dict[str, Any] = field(default_factory=dict)
    notes: str = ""
    record: str = ""

    @property
    def decisive(self) -> bool:
        return self.confidence == "decisive"


@dataclass
class Arena:
    rows: list[Candidate]
    raw: dict[str, Any]

    def get(self, cid: str) -> Candidate | None:
        for c in self.rows:
            if c.id == cid:
                return c
        return None

    def children(self, cid: str) -> list[Candidate]:
        return [c for c in self.rows if c.parent == cid]

    def lineage(self, cid: str) -> list[Candidate]:
        """Walk from a root down to `cid`, so a comparison has the full chain."""
        chain: list[Candidate] = []
        seen: set[str] = set()
        cur = self.get(cid)
        while cur is not None and cur.id not in seen:
            seen.add(cur.id)
            chain.append(cur)
            cur = self.get(cur.parent) if cur.parent else None
        return list(reversed(chain))


def load(paths: HarnessPaths) -> Arena:
    path = paths.repo / HISTORY
    if not path.exists():
        return Arena(rows=[], raw={"schema": 1, "candidates": []})
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return Arena(rows=[], raw={"schema": 1, "candidates": []})
    rows = []
    for c in raw.get("candidates", []):
        rows.append(
            Candidate(
                id=str(c.get("id", "")),
                artifact=str(c.get("artifact", "")),
                milestone=str(c.get("milestone", "")),
                date=str(c.get("date", "")),
                parent=c.get("parent"),
                verdict=str(c.get("verdict", "")),
                confidence=str(c.get("confidence", "")),
                mechanism=str(c.get("mechanism", "")),
                panel=str(c.get("panel", "")),
                seeds=c.get("seeds"),
                rows=c.get("rows"),
                scope=str(c.get("scope", "")),
                scores=dict(c.get("scores") or {}),
                notes=str(c.get("notes", "")),
                record=str(c.get("record", "")),
            )
        )
    return Arena(rows=rows, raw=raw)


_TRANSFER_RE = re.compile(r"(?:fresh\s+)?transfer\s*[:=]?\s*[0-9]", re.IGNORECASE)


def _reports_transfer(cand: Candidate) -> bool:
    """A KEEP must report fresh transfer. Look for evidence, not for the word.

    Checking `"transfer" in scope` is not enough: "no transfer reported" contains the word.
    """
    if cand.scores.get("transfer") is not None:
        return True
    if "transfer" in cand.scores:          # explicitly recorded, even as 0
        return True
    return bool(_TRANSFER_RE.search(cand.scope) or _TRANSFER_RE.search(cand.notes))


def bar_meta(paths: HarnessPaths) -> dict[str, dict[str, str]]:
    """The declared direction and unit of every bar, from the registry."""
    try:
        raw = json.loads((paths.repo / HISTORY).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return {k: dict(v) for k, v in (raw.get("bars") or {}).items()}


_BARS: dict[str, dict[str, str]] = {}


def _direction(key: str) -> str:
    return _BARS.get(key, {}).get("direction", "undeclared")


def _unit(key: str) -> str:
    return _BARS.get(key, {}).get("unit", "")


def _is_better(key: str, candidate: float, parent: float) -> bool | None:
    """True when improved, False when regressed, None when unchanged or direction unknown.

    Direction is declared per bar. Without it, a 92% increase in candidates scored per query
    reads as a gain - which is exactly the mistake this function exists to prevent. An equal
    value is neither a gain nor a regression, in either direction.
    """
    if candidate == parent:
        return None
    direction = _direction(key)
    if direction == "higher_is_better":
        return candidate > parent
    if direction == "lower_is_better":
        return candidate < parent
    return None


# ------------------------------------------------------------------------ comparison


def compare(paths: HarnessPaths, reg: Registry, cid: str) -> dict[str, Any]:
    """Row-by-row comparison of one candidate against its parent.

    Returns a structure that names every bar gained, lost and unchanged, plus the reasons a
    comparison cannot be made. It never collapses the bars into one number.
    """
    global _BARS
    _BARS = bar_meta(paths)
    arena = load(paths)
    cand = arena.get(cid)
    if cand is None:
        return {"ok": False, "reason": f"no candidate {cid!r} in {HISTORY}", "candidate": cid}

    parent = arena.get(cand.parent) if cand.parent else None
    result: dict[str, Any] = {
        "ok": True,
        "candidate": cand.id,
        "artifact": cand.artifact,
        "milestone": cand.milestone,
        "parent": parent.id if parent else None,
        "verdict": cand.verdict,
        "confidence": cand.confidence,
        "comparable": True,
        "gained": [],
        "lost": [],
        "unchanged": [],
        "unavailable": [],
        "blockers": [],
    }

    # scope gates: what stops this row from supporting a decision
    if cand.confidence == "uninterpretable":
        result["comparable"] = False
        result["blockers"].append("the row is recorded as uninterpretable; fix the configuration before reading it")
    if not cand.scope:
        result["blockers"].append("no scope recorded: an artifact/data/steps scope must be stated for the result to have one")
    if cand.confidence == "decisive":
        seeds, rows = cand.seeds or 0, cand.rows or 0
        if seeds < 2 and rows < 200:
            result["blockers"].append(
                f"claimed decisive with {seeds} seed(s) and {rows} row(s); a decisive run needs >=2 seeds or >=200 fresh rows (D22)"
            )
    if cand.verdict == "KEEP" and not _reports_transfer(cand):
        result["blockers"].append(
            "a development KEEP must report fresh transfer (D22): record a 'transfer' bar or state "
            "'fresh transfer: <number>' in the scope"
        )

    if parent is None:
        result["comparable"] = False
        result["reason"] = "this is a root row: there is no parent to regress against"
        return result

    if cand.panel and parent.panel and cand.panel != parent.panel:
        result["comparable"] = False
        result["blockers"].append(
            f"panel changed between parent and candidate ({parent.panel} -> {cand.panel}); "
            "a comparison across panels is not a comparison"
        )

    keys = sorted(set(cand.scores) | set(parent.scores))
    for key in keys:
        a, b = parent.scores.get(key), cand.scores.get(key)
        if isinstance(a, (int, float)) and isinstance(b, (int, float)):
            better = _is_better(key, b, a)
            direction = _direction(key)
            entry = {
                "bar": key,
                "parent": a,
                "candidate": b,
                "delta": round(b - a, 6),
                "direction": direction,
                "unit": _unit(key),
            }
            if better is None and direction == "undeclared":
                entry["note"] = "bar direction is not declared, so this cannot be called a gain or a regression"
            if better:
                result["gained"].append(entry)
            elif better is False:
                result["lost"].append(entry)
            else:
                result["unchanged"].append(entry)
        elif b is None:
            result["unavailable"].append({"bar": key, "parent": a, "candidate": None,
                                          "why": "the candidate does not report this bar"})
        elif a is None:
            result["unavailable"].append({"bar": key, "parent": None, "candidate": b,
                                          "why": "the parent does not report this bar"})
    return result


def render(result: dict[str, Any]) -> str:
    if not result.get("ok"):
        return f"arena: {result['reason']}"
    out: list[str] = []
    out.append(f"candidate {result['candidate']}  ({result['milestone']}, {result['verdict']}, {result['confidence']})")
    out.append(f"  artifact: {result['artifact']}")
    out.append(f"  parent:   {result['parent'] or '(root row)'}")
    for b in result.get("blockers", []):
        out.append(f"  BLOCKER: {b}")
    if not result.get("comparable"):
        out.append("  not comparable - see blockers above")
        return "\n".join(out)
    out.append("")
    out.append("  row-by-row against the parent (never an aggregate):")
    for label, key in (("gained", "gained"), ("lost", "lost"), ("unchanged", "unchanged")):
        for e in result[key]:
            arrow = "+" if e["delta"] > 0 else "" if e["delta"] == 0 else ""
            unit = f" {e['unit']}" if e.get("unit") else ""
            note = f"  [{e['note']}]" if e.get("note") else ""
            out.append(
                f"    {label:<9} {e['bar']:<24} {e['parent']} -> {e['candidate']}  ({arrow}{e['delta']})"
                f"  {e.get('direction','')}{unit}{note}"
            )
    for e in result["unavailable"]:
        out.append(f"    UNAVAILABLE {e['bar']:<24} {e['why']}")
    if result["lost"]:
        out.append("")
        out.append(f"  {len(result['lost'])} bar(s) regressed. A gain elsewhere does not offset them;")
        out.append("  report both, and say which one the decision turns on.")
    return "\n".join(out)


# ------------------------------------------------------------------------------- rules


def register_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="arena.rows_valid",
                description="every arena row has an id, artifact, milestone, verdict and confidence",
                severity="error",
                check=_check_rows_valid,
                rationale="a row missing its scope cannot support a decision",
                mistake="measurement-is-not-mechanism-evidence",
            )
        ),
        register_rule(
            Rule(
                id="arena.parents_exist",
                description="every named parent is a row in the arena",
                severity="error",
                check=_check_parents_exist,
                rationale="a broken lineage makes the comparison meaningless",
            )
        ),
        register_rule(
            Rule(
                id="arena.decisive_power",
                description="a row claiming decisive confidence meets the D22 minimum test power",
                severity="warn",
                check=_check_decisive_power,
                rationale="D22 section 3: >=2 seeds or >=200 fresh rows",
                mistake="seed-lottery-variance-is-not-a-mechanism-result",
            )
        ),
        register_rule(
            Rule(
                id="arena.knows_what_it_cannot_measure",
                description="the arena states its own limits",
                severity="warn",
                check=_check_stated_limits,
                rationale="an instrument that does not name its blind spots gets read as complete",
            )
        ),
    ]


def _check_rows_valid(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    arena = load(paths)
    out: list[RuleResult] = []
    seen: set[str] = set()
    for c in arena.rows:
        if c.id in seen:
            out.append(_fail("arena.rows_valid", f"duplicate candidate id {c.id!r}", HISTORY))
        seen.add(c.id)
        for fieldname in ("id", "artifact", "milestone", "verdict", "confidence"):
            if not getattr(c, fieldname):
                out.append(_fail("arena.rows_valid", f"candidate {c.id or '(no id)'} has no {fieldname}", HISTORY))
        if c.verdict and c.verdict not in VERDICTS:
            out.append(_fail("arena.rows_valid", f"candidate {c.id}: verdict {c.verdict!r} is not one of {VERDICTS}", HISTORY))
        if c.confidence and c.confidence not in CONFIDENCE:
            out.append(_fail("arena.rows_valid", f"candidate {c.id}: confidence {c.confidence!r} is not one of {CONFIDENCE}", HISTORY))
    return out


def _check_parents_exist(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    arena = load(paths)
    ids = {c.id for c in arena.rows}
    out: list[RuleResult] = []
    for c in arena.rows:
        if c.parent and c.parent not in ids:
            out.append(_fail("arena.parents_exist", f"candidate {c.id} names parent {c.parent!r}, which is not a row", HISTORY))
        if c.parent == c.id:
            out.append(_fail("arena.parents_exist", f"candidate {c.id} is its own parent", HISTORY))
    # a cycle other than self-parenting
    for c in arena.rows:
        seen, cur = set(), c
        while cur.parent and cur.parent in ids:
            if cur.parent in seen:
                out.append(_fail("arena.parents_exist", f"parent cycle through {c.id}", HISTORY))
                break
            seen.add(cur.parent)
            cur = arena.get(cur.parent)  # type: ignore[assignment]
    return out


def _check_decisive_power(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    for c in load(paths).rows:
        if c.confidence != "decisive":
            continue
        if (c.seeds or 0) < 2 and (c.rows or 0) < 200:
            out.append(
                _fail(
                    "arena.decisive_power",
                    f"candidate {c.id} claims decisive with {c.seeds or 0} seed(s) and {c.rows or 0} row(s)",
                    HISTORY,
                    fix="record it as provisional, or run >=2 seeds / >=200 fresh rows (D22)",
                )
            )
    return out


def _check_stated_limits(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    raw = load(paths).raw
    limits = str(raw.get("limits", "")).strip()
    if len(limits) < 40:
        return [
            _fail(
                "arena.knows_what_it_cannot_measure",
                "the arena does not state what it cannot measure, so it will be read as complete",
                HISTORY,
                fix='add a "limits" string to docs/harness/arena.json naming the blind spots',
            )
        ]
    return []


def _fail(rule_id: str, message: str, path: str, fix: str = "") -> RuleResult:
    return RuleResult(rule_id=rule_id, severity="error", ok=False, message=message, path=path, fix=fix)


# -------------------------------------------------------------------------------- cli


def main(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    action = getattr(args, "action", "board")
    arena = load(paths)

    if action == "board":
        if not arena.rows:
            emit(f"arena: no rows recorded yet ({HISTORY})")
            emit("  a row is one evaluated geometric configuration; add them as they are measured")
            return 0
        if getattr(args, "json", False):
            emit(json.dumps({"candidates": [c.__dict__ for c in arena.rows], "limits": arena.raw.get("limits", "")}, indent=2))
            return 0
        emit(f"the regression arena - {len(arena.rows)} evaluated configuration(s)")
        emit(f"{'id':<26} {'milestone':<10} {'verdict':<16} {'confidence':<16} parent")
        emit("-" * 90)
        for c in sorted(arena.rows, key=lambda c: c.date):
            emit(f"{c.id:<26} {c.milestone:<10} {c.verdict:<16} {c.confidence:<16} {c.parent or '-'}")
        limits = str(arena.raw.get("limits", "")).strip()
        emit("")
        emit(f"limits: {truncate(limits, 300) if limits else 'NOT STATED'}")
        emit("")
        emit("compare one against its parent:  cli.py arena compare <id>")
        return 0

    if action == "compare":
        result = compare(paths, reg, args.id)
        if getattr(args, "json", False):
            emit(json.dumps(result, indent=2))
        else:
            emit(render(result))
        return 0 if result.get("ok") else 1

    if action == "lineage":
        chain = arena.lineage(args.id)
        if not chain:
            emit(f"arena: no lineage for {args.id!r}", file=__import__("sys").stderr)
            return 1
        for i, c in enumerate(chain):
            emit(f"{'  ' * i}{c.id}  ({c.milestone}, {c.verdict})")
        return 0

    return 2
