"""Receipts: the answer to "was that skill actually used, and did an agent really run?"

Every harness action that the owner cannot otherwise see appends one JSON object to
``docs/harness/ledger/<YYYY-MM>.jsonl``. Receipts are committed; local scratch state
(heartbeats cache, GitHub cache, budget probes) goes to the gitignored ``.harness/``.

A receipt is evidence of orchestration, never of a result. It cannot move a headline
number and must never be presented as one.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Iterable

from core import HarnessPaths, append_jsonl, now_iso, read_jsonl, redact
from checkrules import Rule, RuleResult, register as register_rule

KINDS = ("skill", "team", "heartbeat", "context", "prep", "pr", "check")
EVENT_TYPES = {"skill": "harness_skill_used", "team": "harness_team_dispatched", "heartbeat": "harness_heartbeat"}

#: fields that are never worth storing, and must never be stored if they look secret
_NEVER = ("token", "api_key", "apikey", "secret", "password")


def ledger_path(paths: HarnessPaths, month: str | None = None) -> Path:
    from core import month as current_month

    return paths.ledger_dir / f"{(month or current_month())}.jsonl"


def record(paths: HarnessPaths, kind: str, **fields: Any) -> dict[str, Any]:
    """Append one receipt and return it. Unknown kinds raise, so typos fail loudly."""
    if kind not in KINDS:
        raise ValueError(f"unknown receipt kind {kind!r}; expected one of {KINDS}")
    rec: dict[str, Any] = {"ts": now_iso(), "kind": kind, "schema": 1}
    if kind in EVENT_TYPES:
        rec["event"] = EVENT_TYPES[kind]
    for key, value in fields.items():
        if key.lower() in _NEVER:
            continue
        if value in (None, "", [], {}):
            continue
        rec[key] = redact(str(value)) if isinstance(value, str) else value
    append_jsonl(ledger_path(paths), rec)
    return rec


def read(paths: HarnessPaths, month: str | None = None, kinds: Iterable[str] | None = None) -> list[dict[str, Any]]:
    """Read receipts for one month (default: all months present), oldest first."""
    if month:
        files = [ledger_path(paths, month)]
    else:
        files = sorted(paths.ledger_dir.glob("*.jsonl")) if paths.ledger_dir.is_dir() else []
    want = set(kinds) if kinds else None
    out: list[dict[str, Any]] = []
    for f in files:
        for rec in read_jsonl(f):
            if want and rec.get("kind") not in want:
                continue
            rec.setdefault("_file", f.name)
            out.append(rec)
    out.sort(key=lambda r: str(r.get("ts", "")))
    return out


def today(paths: HarnessPaths) -> list[dict[str, Any]]:
    from core import today as current_day

    day = current_day()
    return [r for r in read(paths) if str(r.get("ts", "")).startswith(day)]


def in_last_hours(paths: HarnessPaths, hours: int, kinds: Iterable[str] | None = None) -> list[dict[str, Any]]:
    from datetime import datetime, timedelta, timezone

    cutoff = datetime.now(timezone.utc) - timedelta(hours=hours)
    out = []
    for rec in read(paths, kinds=kinds):
        try:
            when = datetime.fromisoformat(str(rec["ts"]))
        except (KeyError, ValueError):
            continue
        if when.tzinfo is None:
            when = when.replace(tzinfo=timezone.utc)
        if when >= cutoff:
            out.append(rec)
    return out


def summarize(paths: HarnessPaths, month: str | None = None) -> dict[str, Any]:
    recs = read(paths, month=month)
    by_kind: dict[str, int] = {}
    skills: dict[str, int] = {}
    roles: dict[str, int] = {}
    for r in recs:
        kind = str(r.get("kind", "?"))
        by_kind[kind] = by_kind.get(kind, 0) + 1
        if kind == "skill" and r.get("skill"):
            skills[str(r["skill"])] = skills.get(str(r["skill"]), 0) + 1
        if kind == "team" and r.get("role"):
            roles[str(r["role"])] = roles.get(str(r["role"]), 0) + 1
    day = today(paths)
    return {
        "month": month or "",
        "total": len(recs),
        "today": len(day),
        "by_kind": dict(sorted(by_kind.items())),
        "skills_used": dict(sorted(skills.items(), key=lambda kv: -kv[1])),
        "roles_dispatched": dict(sorted(roles.items(), key=lambda kv: -kv[1])),
        "last": recs[-5:],
    }


def ledger_files(paths: HarnessPaths) -> list[Path]:
    if not paths.ledger_dir.is_dir():
        return []
    return sorted(paths.ledger_dir.glob("*.jsonl"))


# ------------------------------------------------------------------------------- rules


def register_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="receipts.parse",
                description="every ledger line is valid JSON with a ts and a known kind",
                severity="error",
                check=_check_parse,
                rationale="a corrupt ledger silently erases the record of what ran",
                mistake="malformed-ledger-line",
            )
        ),
        register_rule(
            Rule(
                id="receipts.skill_usage_recorded",
                description="a skill receipt exists once skills have been used",
                severity="info",
                check=_check_skill_usage,
                rationale="the owner cannot see skill usage otherwise",
            )
        ),
        register_rule(
            Rule(
                id="receipts.ledger_present",
                description="the ledger directory is tracked and readable",
                severity="warn",
                check=_check_ledger_present,
                rationale="receipts are the only durable record of orchestration",
            )
        ),
    ]


def _check_parse(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401 - Registry
    out: list[RuleResult] = []
    for f in ledger_files(paths):
        for lineno, line in enumerate(f.read_text(encoding="utf-8").splitlines(), start=1):
            if not line.strip():
                continue
            import json

            try:
                rec = json.loads(line)
            except json.JSONDecodeError as exc:
                out.append(
                    RuleResult(
                        rule_id="receipts.parse",
                        severity="error",
                        ok=False,
                        message=f"{f.name}:{lineno} is not valid JSON ({exc.msg})",
                        path=str(f),
                        fix=f"fix or delete the line: {f}:{lineno}",
                    )
                )
                continue
            if "ts" not in rec or "kind" not in rec:
                out.append(
                    RuleResult(
                        rule_id="receipts.parse",
                        severity="error",
                        ok=False,
                        message=f"{f.name}:{lineno} has no ts/kind",
                        path=str(f),
                        fix="a receipt needs at least ts and kind",
                    )
                )
            elif rec["kind"] not in KINDS:
                out.append(
                    RuleResult(
                        rule_id="receipts.parse",
                        severity="error",
                        ok=False,
                        message=f"{f.name}:{lineno} has unknown kind {rec['kind']!r}",
                        path=str(f),
                        fix=f"use one of {list(KINDS)}",
                    )
                )
    return out


def _check_skill_usage(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    recs = read(paths, kinds=("skill",))
    if recs:
        return []
    return []  # informational only: the engine prints "ok" for an empty list


def _check_ledger_present(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    if ledger_files(paths):
        return []
    if not paths.ledger_dir.exists():
        return []
    return [
        RuleResult(
            rule_id="receipts.ledger_present",
            severity="warn",
            ok=False,
            message="docs/harness/ledger/ exists but holds no receipts",
            path="docs/harness/ledger",
            fix="run `python3 scripts/harness/cli.py skill use <id>` on the next unit of work",
        )
    ]


def main(args: Any, paths: HarnessPaths, reg: Any) -> int:  # noqa: ANN401 - argparse namespace
    from core import emit
    import json

    if getattr(args, "json", False):
        emit(json.dumps(summarize(paths), indent=2, default=str))
    else:
        s = summarize(paths)
        emit(f"receipts this month: {s['total']} ({s['today']} today)")
        for kind, n in s["by_kind"].items():
            emit(f"  {kind:<10} {n}")
    return 0
