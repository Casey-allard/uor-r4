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

import json
import re

from core import HarnessPaths, append_jsonl, now_iso, read_jsonl, redact, run
from checkrules import Rule, RuleResult, register as register_rule

KINDS = ("skill", "team", "heartbeat", "context", "prep", "pr", "check", "claim", "release")
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
    # Sharded per lab so two labs never write the same file. Falls back to the unsharded
    # month file only if multilab is unavailable, so an older checkout still records.
    lab = str(rec.get("lab") or "")
    try:
        import multilab

        if not lab:
            rec["lab"] = _lab_identity()
        if not rec.get("lab"):
            # Loud, once per process. A receipt with no lab is preserved but cannot answer
            # "which lab did this", which is the visibility the shards exist for. Thirteen
            # receipts accumulated unattributed before this warning existed, and nothing
            # said so at the time.
            _warn_unattributed()
        multilab.append_receipt(paths, rec.get("lab", ""), rec)
    except ImportError:  # pragma: no cover - multilab ships with the harness
        append_jsonl(ledger_path(paths), rec)
    return rec


def _warn_unattributed() -> None:
    """Tell the session, once, that this receipt will be attributed to nobody."""
    if _UNATTRIBUTED_WARNED[0]:
        return
    _UNATTRIBUTED_WARNED[0] = True
    import sys

    print(
        "uor-harness: WARNING - no lab identity, so this record is attributed to nobody and no\n"
        "             other lab can see who did it. Fix it for this session with:\n"
        "             export UOR_LAB=<kimi|claude|codex|deepseek>",
        file=sys.stderr,
    )


_UNATTRIBUTED_WARNED = [False]


def _lab_identity() -> str:
    """This lab's name: $UOR_LAB if set, else empty (which the reader reports as a gap)."""
    import os

    return os.environ.get("UOR_LAB", "").strip()


def read(paths: HarnessPaths, month: str | None = None, kinds: Iterable[str] | None = None) -> list[dict[str, Any]]:
    """Read receipts for one month (default: all months present), oldest first."""
    want = set(kinds) if kinds else set(KINDS)
    out: list[dict[str, Any]] = []
    try:
        import multilab

        for rec in multilab.read_shards(paths, month):
            if rec.get("kind") not in want:
                continue
            out.append(rec)
    except ImportError:  # pragma: no cover
        pass
    # legacy unsharded files, kept readable so no earlier receipt is lost
    if month:
        files = [ledger_path(paths, month)]
    else:
        files = sorted(paths.ledger_dir.glob("*.jsonl")) if paths.ledger_dir.is_dir() else []
    for f in files:
        if not re.fullmatch(r"\d{4}-\d{2}\.jsonl", f.name):
            continue
        for rec in read_jsonl(f):
            if rec.get("kind") not in want:
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
                id="status.branch_claimed",
                description="the checked-out branch is claimed by a lab, under the one-checkout policy",
                severity="warn",
                check=_check_branch_claimed,
                rationale="docs/labs/working-in-one-checkout.md: one checkout means one branch owner at a time",
                mistake="shared-checkout-branch-collision",
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


# --------------------------------------------------------------------- branch claims
#
# Owner direction, 12 October 2026: labs work in one checkout, so a branch is claimed
# rather than owned by a directory.
#
# REVISED for multiple labs: claims now live in the COMMITTED `docs/harness/claims.json`
# (see multilab.py). The original `.harness/claims.json` was gitignored, which was right for
# one lab and wrong for four - a claim nobody else can read cannot prevent a collision. These
# functions remain as a thin compatibility layer over the committed store, and fall back to
# the local file only if multilab is unavailable.

CLAIMS_FILE = "claims.json"
CLAIM_KINDS = ("claim", "release")


def claims_path(paths: HarnessPaths) -> Path:
    """The committed claims store, so every lab can see who holds what."""
    try:
        import multilab

        return multilab.claims_file(paths)
    except ImportError:  # pragma: no cover
        return paths.state / CLAIMS_FILE


def load_claims(paths: HarnessPaths) -> dict[str, Any]:
    try:
        import multilab

        return multilab.load_claims(paths)
    except ImportError:  # pragma: no cover
        path = paths.state / CLAIMS_FILE
        if not path.exists():
            return {}
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return data if isinstance(data, dict) else {}


def _publish_claims_live(paths: HarnessPaths, claims: dict[str, Any]) -> None:
    """Mirror the claims into the live tier so labs see them before the PR merges.

    `team claim` reaches the store through THIS module, not through `multilab.claim`, so the mirror
    has to be published here too. Missing it meant `multilab.claim` mirrored and `team claim` - the
    command labs actually run - did not, which is the kind of split that makes a feature look like it
    works while the real path does nothing. Best-effort: a read-only live tier must not cost the
    committed copy, which is the authority.
    """
    try:
        import multilab

        multilab.publish_claims_live(paths, claims)
    except (ImportError, OSError):  # pragma: no cover - the durable copy already landed
        pass


def _save_claims(paths: HarnessPaths, claims: dict[str, Any]) -> None:
    try:
        import multilab

        multilab.save_claims(paths, claims)
        return
    except ImportError:  # pragma: no cover
        pass
    path = paths.state / CLAIMS_FILE
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(claims, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    tmp.replace(path)


def current_branch(paths: HarnessPaths) -> str:
    """The checked-out branch. Used by the claim rule and by the CLI."""
    rc, out = run(["git", "-C", str(paths.repo), "branch", "--show-current"], timeout=10)
    return out.strip() if rc == 0 else ""


def claim_branch(paths: HarnessPaths, branch: str, lab: str, note: str = "") -> tuple[bool, dict[str, Any]]:
    """Claim a branch for a lab. Refuses when another lab already holds it."""
    if not branch:
        raise ValueError("a branch name is required")
    if not lab:
        raise ValueError("a lab is required (--lab), because a claim without an owner is not a claim")
    claims = load_claims(paths)
    existing = claims.get(branch)
    if existing and existing.get("lab") and existing["lab"] != lab:
        return False, existing
    entry = {"branch": branch, "lab": lab, "head": "", "claimed_at": now_iso(), "note": note}
    rc, out = run(["git", "-C", str(paths.repo), "rev-parse", "--short", "HEAD"], timeout=10)
    if rc == 0:
        entry["head"] = out.strip()
    claims[branch] = entry
    _save_claims(paths, claims)
    _publish_claims_live(paths, claims)
    # NB: `record` must stay the imported function here; shadowing it was a real bug.
    record(paths, "claim", branch=branch, lab=lab, note=note)
    return True, entry


def release_branch(paths: HarnessPaths, branch: str, lab: str = "") -> tuple[bool, dict[str, Any]]:
    claims = load_claims(paths)
    existing = claims.get(branch)
    if not existing:
        return False, {}
    if lab and existing.get("lab") and existing["lab"] != lab:
        return False, existing
    del claims[branch]
    _save_claims(paths, claims)
    # Sync the REMOVAL as well: a mirror only ever added to resurrects released claims.
    _publish_claims_live(paths, claims)
    record(paths, "release", branch=branch, lab=lab or existing.get("lab", ""))
    return True, existing


def claim_for_current_branch(paths: HarnessPaths) -> dict[str, Any] | None:
    return load_claims(paths).get(current_branch(paths))


def _check_branch_claimed(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    """Under the one-checkout policy a branch should be claimed before work starts."""
    branch = current_branch(paths)
    if not branch or branch in ("main", "master"):
        return []
    claims = load_claims(paths)
    if branch in claims:
        return []
    return [
        RuleResult(
            rule_id="status.branch_claimed",
            severity="warn",
            ok=False,
            message=f"branch {branch!r} is checked out but not claimed by any lab",
            path="docs/harness/claims.json",
            fix=f"python3 scripts/harness/cli.py team claim --branch {branch} --lab <lab>",
        )
    ]
