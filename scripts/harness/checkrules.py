"""The rule engine.

A rule is a small, named, machine-checkable statement about this repo. Rules exist so a
lesson learned once becomes a guard that runs forever, and so the owner can see *why* a
PR is not compliant rather than a wall of prose.

`register_rules()` is called on each feature module; a module that cannot be imported
does not silently disable its rules - the engine reports the failure as an error result.
"""

from __future__ import annotations

import importlib
from dataclasses import dataclass, field
from typing import Any, Callable, Iterable

from core import HarnessPaths
from registries import Registry

SEVERITIES = ("error", "warn", "info")
SEVERITY_ORDER = {"error": 0, "warn": 1, "info": 2}

#: feature modules that contribute rules
RULE_MODULES = ("briefing", "mistakes", "receipts", "prep", "prlint", "status", "budget", "arena")


@dataclass
class RuleResult:
    rule_id: str
    severity: str
    ok: bool
    message: str
    path: str = ""
    fix: str = ""
    meta: dict[str, Any] = field(default_factory=dict)

    def render(self) -> str:
        mark = "ok  " if self.ok else ("FAIL" if self.severity == "error" else "warn")
        loc = f" [{self.path}]" if self.path else ""
        out = f"{mark} {self.rule_id}{loc}: {self.message}"
        if not self.ok and self.fix:
            out += f"\n      fix: {self.fix}"
        return out


@dataclass
class Rule:
    id: str
    description: str
    severity: str
    check: Callable[[HarnessPaths, Registry], list[RuleResult]]
    rationale: str = ""
    mistake: str = ""
    scope: str = "repo"


def register(rule: Rule, into: list[Rule] | None = None) -> Rule:
    if rule.severity not in SEVERITIES:
        raise ValueError(f"rule {rule.id}: severity must be one of {SEVERITIES}")
    if rule.scope not in ("repo", "staged", "pr"):
        raise ValueError(f"rule {rule.id}: scope must be repo|staged|pr")
    if into is None:
        _REGISTRY.append(rule)
    else:
        into.append(rule)
    return rule


_REGISTRY: list[Rule] = []


def rules(scope: str | None = None) -> list[Rule]:
    """Repo-scope rules from this module plus every feature module."""
    found = list(_REGISTRY) + _own_rules()
    for name in RULE_MODULES:
        try:
            mod = importlib.import_module(name)
        except Exception as exc:  # noqa: BLE001 - reported, never raised
            found.append(
                Rule(
                    id=f"harness.module.{name}",
                    description=f"{name} rules load",
                    severity="error",
                    check=_broken_module(name, exc),
                    rationale="a module that cannot load must not silently disable its rules",
                )
            )
            continue
        getter = getattr(mod, "register_rules", None)
        if getter is None:
            continue
        try:
            found.extend(getter())
        except Exception as exc:  # noqa: BLE001
            found.append(
                Rule(
                    id=f"harness.module.{name}",
                    description=f"{name} rules register",
                    severity="error",
                    check=_broken_module(name, exc),
                )
            )
    if scope is not None:
        found = [r for r in found if r.scope == scope]
    return found


def _broken_module(name: str, exc: Exception) -> Callable[[HarnessPaths, Registry], list[RuleResult]]:
    def check(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
        return [
            RuleResult(
                rule_id=f"harness.module.{name}",
                severity="error",
                ok=False,
                message=f"{name} could not be loaded: {type(exc).__name__}: {exc}",
                fix=f"python3 -c \"import sys; sys.path.insert(0,'scripts/harness'); import {name}\"",
            )
        ]

    return check


def run(
    paths: HarnessPaths,
    reg: Registry,
    scope: str = "repo",
    only: Iterable[str] | None = None,
) -> list[RuleResult]:
    """Run rules and return results, violations first, then by rule id."""
    only_set = {o for o in (only or [])}
    out: list[RuleResult] = []
    for rule in rules(scope=scope):
        if only_set and not any(rule.id.startswith(o) for o in only_set):
            continue
        try:
            results = rule.check(paths, reg) or []
        except Exception as exc:  # noqa: BLE001 - a rule never breaks a run
            results = [
                RuleResult(
                    rule_id=rule.id,
                    severity="error",
                    ok=False,
                    message=f"rule raised {type(exc).__name__}: {exc}",
                    fix="fix the rule implementation",
                )
            ]
        if not results:
            results = [RuleResult(rule_id=rule.id, severity=rule.severity, ok=True, message="ok")]
        for r in results:
            if not isinstance(r, RuleResult):
                continue
            if not r.severity:
                r.severity = rule.severity
            if not r.rule_id:
                r.rule_id = rule.id
            out.append(r)
    out.sort(key=lambda r: (r.ok, SEVERITY_ORDER.get(r.severity, 3), r.rule_id, r.path))
    return out


def summarize(results: list[RuleResult]) -> dict[str, Any]:
    fails = [r for r in results if not r.ok]
    return {
        "checked": len(results),
        "failed": len(fails),
        "errors": sum(1 for r in fails if r.severity == "error"),
        "warnings": sum(1 for r in fails if r.severity == "warn"),
        "ok": not any(r.severity == "error" for r in fails),
    }


# ---------------------------------------------------------------- registry-level rules


def _own_rules() -> list[Rule]:
    return [
        Rule(
            id="harness.milestones.present",
            description="the milestone registry exists and covers m1-m4",
            severity="error",
            check=_check_milestones_present,
            rationale="the harness cannot brief a session without the live contract",
        ),
        Rule(
            id="harness.skills.nonempty",
            description="at least one skill is registered",
            severity="error",
            check=_check_skills_present,
            rationale="an empty skill registry means no reusable protocol is reachable",
        ),
        Rule(
            id="harness.mistakes.have_guards",
            description="every active mistake names a guard or explains why it has none",
            severity="warn",
            check=_check_mistake_guards,
            rationale="a lesson without a guard is a note that will be forgotten again",
        ),
    ]


def _check_milestones_present(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    missing = [k for k in ("m1", "m2", "m3", "m4") if reg.milestones.get(k) is None]
    if missing:
        return [
            RuleResult(
                rule_id="harness.milestones.present",
                severity="error",
                ok=False,
                message=f"milestone registry is missing {', '.join(missing)}",
                path="docs/harness/milestones.json",
                fix="add the missing milestone entries",
            )
        ]
    return []


def _check_skills_present(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    if reg.skills:
        return []
    return [
        RuleResult(
            rule_id="harness.skills.nonempty",
            severity="error",
            ok=False,
            message="no skills found under docs/harness/skills/*/SKILL.md",
            path="docs/harness/skills",
            fix="add at least one skill directory with SKILL.md front matter",
        )
    ]


def _check_mistake_guards(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    for m in reg.mistakes:
        if m.status != "active":
            continue
        if not m.guard:
            out.append(
                RuleResult(
                    rule_id="harness.mistakes.have_guards",
                    severity="warn",
                    ok=False,
                    message=f"{m.id} has no guard",
                    path=str(m.path),
                    fix="add a guard: line naming a rule id, or 'none: <reason>'",
                )
            )
        elif not m.guard.startswith("none") and "." not in m.guard:
            out.append(
                RuleResult(
                    rule_id="harness.mistakes.have_guards",
                    severity="warn",
                    ok=False,
                    message=f"{m.id} guard {m.guard!r} does not name a rule id",
                    path=str(m.path),
                    fix="use a dotted rule id such as prlint.record, or 'none: <reason>'",
                )
            )
    return out
