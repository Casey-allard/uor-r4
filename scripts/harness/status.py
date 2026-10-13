"""status.py - what is true now.

Answers the owner's question "I never know if a skill is being used or when agents
are deployed": the milestone headlines, what the ledger actually recorded, what
GitHub says, and what the delivery hygiene state is.

Read-only. Every external call is bounded (20 s) and degrades to an explicit
``{"available": false, "reason": "<exact command>: <error>"}`` - never a zero that
reads like a measurement. Standard library only, no secret paths, and every
subprocess error string passes through ``core.redact`` before it can be printed
or cached.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from checkrules import Rule, RuleResult
from core import HarnessPaths, emit, load_json, now_iso, read_jsonl, redact, today, month
from registries import Milestone, Registry
from util import truncate

SCHEMA = 1

#: every external call is bounded to this many seconds
GH_TIMEOUT = 20
GIT_TIMEOUT = 20

#: a GitHub answer younger than this is reused instead of re-asked
CACHE_TTL_SECONDS = 15 * 60

#: live state lives here (gitignored); receipts live in docs/harness/ledger/
CACHE_NAME = "github.json"

#: keep the newest N comments per issue, truncated, so the cache stays small
COMMENT_KEEP = 5
COMMENT_CHARS = 2000

#: the delivery-hygiene floor (AGENTS.md: keep 30-70 GB free)
DISK_FLOOR_GIB = 30

#: session-goal.md section 5: a status card every 90 minutes or 3 merged PRs
STALE_HOURS = 24

#: the receipt kinds the ledger contract names (docs/harness/CONTRACT.md)
RECEIPT_KINDS = ("skill", "team", "heartbeat", "context", "prep", "pr")

MERGED_PR_LIMIT = 100
OPEN_PR_LIMIT = 50

#: at most this many per-branch `gh pr list` calls in one run (the rest are cached)
BRANCH_PR_LIMIT = 25

#: ignored paths that are regenerable and therefore do not block a worktree cleanup
REGENERABLE_IGNORED = ("target/", ".harness/")

_STATUS_CARD_RE = re.compile(r"\bM(\d+)\s*[:=]\s*(\d+)\s*/\s*(\d+)")


# --------------------------------------------------------------------------- time


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _parse_ts(value: Any) -> datetime | None:
    """Parse an ISO-8601 stamp (or epoch seconds). Returns None, never raises."""
    if value is None:
        return None
    if isinstance(value, (int, float)):
        try:
            return datetime.fromtimestamp(float(value), tz=timezone.utc)
        except (OverflowError, OSError, ValueError):
            return None
    text = str(value).strip()
    if not text:
        return None
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    try:
        dt = datetime.fromisoformat(text)
    except ValueError:
        return None
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt


def _age_hours(value: Any, now: datetime) -> float | None:
    dt = _parse_ts(value)
    if dt is None:
        return None
    return round((now - dt).total_seconds() / 3600.0, 2)


def _hours_text(hours: float | None) -> str:
    if hours is None:
        return "n/a"
    if hours < 1:
        return f"{hours * 60:.0f}m"
    if hours < 48:
        return f"{hours:.1f}h"
    return f"{hours / 24:.1f}d"


# ------------------------------------------------------------------------ process


def _cmd_text(cmd: list[str]) -> str:
    return " ".join(cmd)


def _run(cmd: list[str], cwd: Path | None, timeout: int) -> tuple[bool, str, str]:
    """Run a read-only command. Returns (ok, stdout, error). Never raises."""
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(cwd) if cwd else None,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except FileNotFoundError:
        return False, "", f"`{_cmd_text(cmd)}` could not run: executable not found ({cmd[0]})"
    except subprocess.TimeoutExpired:
        return False, "", f"`{_cmd_text(cmd)}` timed out after {timeout}s"
    except OSError as exc:
        return False, "", f"`{_cmd_text(cmd)}` could not run: {type(exc).__name__}: {exc}"
    stdout = proc.stdout or ""
    if proc.returncode != 0:
        err = redact(((proc.stderr or "") + (proc.stdout or "")).strip().replace("\n", " "))[:240]
        return False, stdout, f"`{_cmd_text(cmd)}` exited {proc.returncode}: {err or 'no output'}"
    return True, stdout, ""


def gh_json(subargs: list[str], repo: Path | None = None, timeout: int = GH_TIMEOUT) -> tuple[bool, Any, str]:
    """Run ``gh <subargs>`` and parse JSON. Returns (ok, data, error). Never raises.

    This is the single gh entry point, so a test can monkeypatch it and no other
    code path can make an unbounded or unredacted call.
    """
    ok, out, err = _run(["gh", *subargs], repo, timeout)
    if not ok:
        return False, None, err
    try:
        return True, json.loads(out or "null"), ""
    except json.JSONDecodeError as exc:
        return False, None, f"`{_cmd_text(['gh', *subargs])}` returned unparseable JSON: {exc}"


def _git(subargs: list[str], paths: HarnessPaths, timeout: int = GIT_TIMEOUT) -> tuple[bool, str, str]:
    return _run(["git", *subargs], paths.repo, timeout)


def _repo_slug(paths: HarnessPaths) -> str:
    ok, out, _ = _git(["remote", "get-url", "origin"], paths)
    if not ok:
        return ""
    m = re.search(r"github\.com[:/]+([^/\s]+/[^/\s]+?)(?:\.git)?\s*$", out.strip())
    return m.group(1) if m else ""


def _default_branch(paths: HarnessPaths) -> str:
    """The trunk branch (never an orphan, never deleted)."""
    ok, out, _ = _git(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"], paths)
    if ok and out.strip():
        return out.strip().split("/")[-1]
    return "main"


# -------------------------------------------------------------------------- cache


def cache_path(paths: HarnessPaths) -> Path:
    return paths.state / CACHE_NAME


def _cache_load(paths: HarnessPaths) -> dict[str, Any]:
    path = cache_path(paths)
    try:
        if not path.exists():
            return {}
        data = load_json(path)
        return data if isinstance(data, dict) else {}
    except (OSError, ValueError):
        return {}


def _cache_save(paths: HarnessPaths, cache: dict[str, Any]) -> None:
    """Best-effort cache write; a read-only checkout must not break status."""
    try:
        from core import dump_json

        dump_json(cache_path(paths), cache)
    except (OSError, ValueError, TypeError):
        pass


def _fresh(item: dict[str, Any] | None, now: datetime) -> bool:
    if not isinstance(item, dict):
        return False
    age = _age_hours(item.get("fetched_at"), now)
    return age is not None and age * 3600.0 <= CACHE_TTL_SECONDS


# ---------------------------------------------------------------------- github io


def github_prs(paths: HarnessPaths, refresh: bool = False, now: datetime | None = None) -> dict[str, Any]:
    """Open PRs and merged PRs, from gh or from a cache younger than 15 minutes."""
    now = now or _now()
    cache = _cache_load(paths)
    cached = cache.get("prs")
    if not refresh and _fresh(cached, now):
        out = dict(cached)
        out["from_cache"] = True
        out["age_seconds"] = int((_age_hours(cached.get("fetched_at"), now) or 0) * 3600)
        return out

    stamp = now_iso()
    open_ok, open_prs, open_err = gh_json(
        ["pr", "list", "--state", "open", "--limit", str(OPEN_PR_LIMIT), "--json", "number,title,headRefName,url,createdAt,updatedAt"],
        paths.repo,
    )
    merged_ok, merged_prs, merged_err = gh_json(
        [
            "pr",
            "list",
            "--state",
            "merged",
            "--limit",
            str(MERGED_PR_LIMIT),
            "--json",
            "number,title,headRefName,headRefOid,url,mergedAt",
        ],
        paths.repo,
    )
    if open_ok and merged_ok:
        section: dict[str, Any] = {
            "available": True,
            "reason": "",
            "fetched_at": stamp,
            "from_cache": False,
            "open": list(open_prs or []),
            "merged": list(merged_prs or []),
            "open_limit": OPEN_PR_LIMIT,
            "merged_limit": MERGED_PR_LIMIT,
        }
    else:
        section = {
            "available": False,
            "reason": open_err or merged_err or "gh failed with no error text",
            "fetched_at": stamp,
            "from_cache": False,
            "open": [],
            "merged": [],
            "open_limit": OPEN_PR_LIMIT,
            "merged_limit": MERGED_PR_LIMIT,
        }
    cache["schema"] = SCHEMA
    cache["prs"] = section
    _cache_save(paths, cache)
    out = dict(section)
    out["age_seconds"] = 0
    return out


def _trim_comments(comments: Any) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for c in list(comments or [])[-COMMENT_KEEP:]:
        if not isinstance(c, dict):
            continue
        out.append(
            {
                "createdAt": c.get("createdAt") or c.get("created_at") or "",
                "author": ((c.get("author") or {}).get("login") if isinstance(c.get("author"), dict) else c.get("user", {}).get("login") if isinstance(c.get("user"), dict) else ""),
                "body": redact(str(c.get("body") or "")[:COMMENT_CHARS]),
            }
        )
    return out


def github_issues(
    paths: HarnessPaths,
    numbers: list[int],
    refresh: bool = False,
    with_comments: bool = False,
    now: datetime | None = None,
) -> dict[str, Any]:
    """Milestone-issue metadata (and optionally the newest comments), one gh call per issue."""
    now = now or _now()
    cache = _cache_load(paths)
    items: dict[str, Any] = dict(cache.get("issues") or {})
    errors: list[str] = []
    fetched = 0

    for num in numbers:
        key = str(num)
        item = items.get(key)
        if not refresh and _fresh(item, now) and (not with_comments or item.get("comments") is not None):
            continue
        fields = "number,title,state,updatedAt,url"
        if with_comments:
            fields += ",comments"
        ok, data, err = gh_json(["issue", "view", key, "--json", fields], paths.repo)
        fetched += 1
        if not ok or not isinstance(data, dict):
            errors.append(err or f"`gh issue view {key}` returned no issue object")
            continue
        entry = {
            "number": data.get("number", num),
            "title": str(data.get("title", "")),
            "state": str(data.get("state", "")),
            "updatedAt": str(data.get("updatedAt", "")),
            "url": str(data.get("url", "")),
            "fetched_at": now_iso(),
        }
        if with_comments:
            entry["comments"] = _trim_comments(data.get("comments"))
        elif isinstance(item, dict) and item.get("comments") is not None:
            entry["comments"] = item["comments"]
        items[key] = entry

    cache["schema"] = SCHEMA
    cache["issues"] = items
    _cache_save(paths, cache)

    have = [items.get(str(n)) for n in numbers]
    missing = [n for n, it in zip(numbers, have) if not isinstance(it, dict)]
    if missing:
        return {
            "available": False,
            "reason": "; ".join(errors) or f"no data for issue(s) {', '.join(str(n) for n in missing)}",
            "fetched_at": now_iso(),
            "from_cache": False,
            "issues": {str(n): it for n, it in zip(numbers, have) if isinstance(it, dict)},
            "missing": missing,
        }
    from_cache = fetched == 0 and bool(cache.get("issues")) and not refresh
    return {
        "available": True,
        "reason": "",
        "fetched_at": now_iso(),
        "from_cache": from_cache,
        "issues": {str(n): it for n, it in zip(numbers, have) if isinstance(it, dict)},
        "missing": [],
    }


def github_branch_prs(paths: HarnessPaths, branches: list[str], refresh: bool = False, now: datetime | None = None) -> dict[str, Any]:
    """The PR (any state) whose head is each local branch, cached per branch.

    Older merged PRs sit outside the newest-100 window of `gh pr list --state merged`,
    so a local branch can look clean while its PR is long merged. One bounded call per
    branch, at most BRANCH_PR_LIMIT per run, cached for 15 minutes.
    """
    now = now or _now()
    cache = _cache_load(paths)
    store: dict[str, Any] = dict(cache.get("branch_prs") or {})
    queried = 0
    errors: list[str] = []
    for name in branches:
        if not name:
            continue
        item = store.get(name)
        if not refresh and _fresh(item, now):
            continue
        if queried >= BRANCH_PR_LIMIT:
            errors.append(f"only the first {BRANCH_PR_LIMIT} branches were queried this run")
            break
        ok, data, err = gh_json(
            ["pr", "list", "--head", name, "--state", "all", "--limit", "3", "--json", "number,title,state,headRefOid,mergedAt,url"],
            paths.repo,
        )
        queried += 1
        if not ok:
            errors.append(err or f"`gh pr list --head {name}` failed")
            continue
        rows = [r for r in (data or []) if isinstance(r, dict)]
        merged = [r for r in rows if str(r.get("state", "")).upper() == "MERGED"]
        store[name] = {"fetched_at": now_iso(), "pr": (merged or rows or [None])[0]}
    cache["schema"] = SCHEMA
    cache["branch_prs"] = store
    _cache_save(paths, cache)
    have = {name: store.get(name) for name in branches if store.get(name)}
    return {
        "available": bool(have) or not errors,
        "reason": "; ".join(errors),
        "queried": queried,
        "items": have,
    }


# ----------------------------------------------------------------------- receipts


def _read_ledger_file(paths: HarnessPaths, month_str: str) -> list[dict[str, Any]]:
    try:
        return read_jsonl(paths.ledger_dir / f"{month_str}.jsonl")
    except OSError:
        return []


def _read_receipts(paths: HarnessPaths, month_str: str) -> tuple[list[dict[str, Any]], str]:
    """Read one month of receipts. Prefers the receipts module, falls back to the JSONL."""
    try:
        import receipts as R  # noqa: PLC0415 - optional sibling module

        rows = list(R.read(paths, month=month_str))
        return rows, ""
    except ImportError:
        return _read_ledger_file(paths, month_str), ""
    except Exception as exc:  # noqa: BLE001 - receipts.py is another module and may be mid-edit
        note = f"receipts.read failed ({type(exc).__name__}: {exc}); read the ledger file directly"
        return _read_ledger_file(paths, month_str), note


def _previous_month(month_str: str) -> str:
    try:
        dt = datetime.strptime(month_str, "%Y-%m")
    except ValueError:
        return month_str
    first = dt.replace(day=1)
    return (first - timedelta(days=1)).strftime("%Y-%m")


def _receipt_rows(paths: HarnessPaths, now: datetime) -> tuple[list[dict[str, Any]], str]:
    """This month plus (when needed) the previous month, newest first."""
    current = month()
    rows, note = _read_receipts(paths, current)
    seen = {current}
    if len(rows) < COMMENT_KEEP:
        prev = _previous_month(current)
        if prev not in seen:
            older, older_note = _read_receipts(paths, prev)
            rows = rows + older
            note = note or older_note
    rows = [r for r in rows if isinstance(r, dict)]
    rows.sort(key=lambda r: str(r.get("ts", "")), reverse=True)
    return rows, note


def _receipts_section(paths: HarnessPaths, now: datetime) -> dict[str, Any]:
    rows, note = _receipt_rows(paths, now)
    current, day = month(), today()
    by_kind_month = _count_by_kind(rows)
    by_kind_today = _count_by_kind([r for r in rows if str(r.get("ts", ""))[:10] == day])
    used = {"skill": {}, "team": {}}
    for r in rows:
        kind = str(r.get("kind", ""))
        if kind == "skill":
            sid = r.get("skill") or r.get("skill_id") or r.get("id")
            if sid:
                used["skill"][str(sid)] = used["skill"].get(str(sid), 0) + 1
        elif kind == "team":
            rid = r.get("role") or r.get("role_id")
            if rid:
                used["team"][str(rid)] = used["team"].get(str(rid), 0) + 1
    ledger_file = paths.ledger_dir / f"{current}.jsonl"
    return {
        "available": True,
        "reason": note,
        "ledger": {"dir": str(paths.ledger_dir), "month_file": str(ledger_file), "exists": ledger_file.exists()},
        "today": {"date": day, "total": len([r for r in rows if str(r.get("ts", ""))[:10] == day]), "by_kind": by_kind_today},
        "month": {"month": current, "total": len([r for r in rows if str(r.get("ts", ""))[:7] == current]), "by_kind": by_kind_month},
        "last5": rows[:COMMENT_KEEP],
        "used_ids": used,
    }


def _count_by_kind(rows: list[dict[str, Any]]) -> dict[str, int]:
    counts = {kind: 0 for kind in RECEIPT_KINDS}
    other = 0
    for r in rows:
        kind = str(r.get("kind", ""))
        if kind in counts:
            counts[kind] += 1
        else:
            other += 1
    if other:
        counts["other"] = other
    return counts


def _milestone_matches(receipt: dict[str, Any], milestone: Milestone) -> bool:
    raw = str(receipt.get("milestone", "")).strip().lower()
    if not raw:
        return False
    return raw in (milestone.key.lower(), milestone.id.lower(), str(milestone.issue))


def _milestone_receipts(rows: list[dict[str, Any]], milestone: Milestone, kind: str | None = None) -> list[dict[str, Any]]:
    out = [r for r in rows if _milestone_matches(r, milestone)]
    if kind:
        out = [r for r in out if str(r.get("kind", "")) == kind]
    return out


# --------------------------------------------------------------------- milestones


def _mechanism_briefs(paths: HarnessPaths, milestone: Milestone) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for mid in milestone.mechanisms:
        path = paths.mechanisms_dir / f"{mid}.md"
        idea = ""
        try:
            if path.is_file():
                idea = _first_bullet(path.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            idea = ""
        out.append({"id": mid, "brief": str(path), "exists": path.is_file(), "idea": truncate(idea, 150)})
    return out


def _first_bullet(text: str) -> str:
    idx = text.find("The idea (first principles)")
    body = text[idx:] if idx >= 0 else text
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("- "):
            return line[2:].replace("**", "").strip()
    return ""


def _headline_text(milestone: Milestone) -> str:
    """`Milestone.headline_line()`, with the qualitative form rendered properly.

    The live registry now carries a qualitative headline for M4 (`text` plus
    `target_display`, no numeric `current`), which `headline_line()` renders as
    "softmax at runtime None/1". Fall back to the registry's own text in that case.
    """
    line = milestone.headline_line()
    head = milestone.headline or {}
    if "None" not in line:
        return line
    text = str(head.get("text", "")).strip()
    if not text:
        return line
    display = str(head.get("target_display", "")).strip()
    if display:
        return f"{milestone.id}: {text} -> target {display}"
    if head.get("target") is not None:
        return f"{milestone.id}: {text} -> target {head.get('target')}"
    return f"{milestone.id}: {text}"


def _milestones_section(paths: HarnessPaths, reg: Registry) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    out: list[dict[str, Any]] = []
    slug = _repo_slug(paths)
    for m in reg.milestones.all():
        headline = m.headline or {}
        out.append(
            {
                "key": m.key,
                "id": m.id,
                "issue": m.issue,
                "lab": m.lab,
                "status": m.status,
                "title": m.title,
                "headline": _headline_text(m),
                "headline_fields": {
                    "label": headline.get("label", ""),
                    "current": headline.get("current"),
                    "target": headline.get("target"),
                    "of": headline.get("of"),
                    "unit": headline.get("unit", ""),
                    "direction": headline.get("direction", ""),
                    "vacuous": bool(headline.get("vacuous", False)),
                    "text": headline.get("text", ""),
                    "target_display": headline.get("target_display", ""),
                    "note": headline.get("note", ""),
                },
                "target": headline.get("target"),
                "lines": [dict(l) for l in m.lines],
                "line_counts": m.line_counts(),
                "panel": m.panel,
                "mechanisms": _mechanism_briefs(paths, m),
                "next": m.next,
                "url": f"https://github.com/{slug}/issues/{m.issue}" if slug else f"issue #{m.issue}",
            }
        )
    meta: dict[str, Any] = {"path": str(paths.milestones_json), "tracker": dict(reg.milestones.tracker or {}), "note": ""}
    try:
        raw = load_json(paths.milestones_json)
        if isinstance(raw, dict):
            meta["note"] = str(raw.get("note", ""))
    except (OSError, ValueError):
        pass
    return out, meta


def _rules_section(reg: Registry) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for rule in reg.milestones.owner_rules or []:
        if not isinstance(rule, dict):
            continue
        rows.append(
            {
                "id": str(rule.get("id", "")),
                "rule": truncate(str(rule.get("rule", "")), 200),
                "scope": str(rule.get("scope", "")),
            }
        )
    return rows


# ------------------------------------------------------------------------- github


def _milestone_for_issue(reg: Registry, issue: int) -> str:
    for m in reg.milestones.all():
        if m.issue == issue:
            return m.id
    tracker = (reg.milestones.tracker or {}).get("issue")
    if tracker is not None and int(tracker) == issue:
        return "tracker"
    return ""


def _milestone_ref(title: str, reg: Registry) -> dict[str, Any]:
    """Parse the milestone reference from a PR title: '(References #2029)' -> M1."""
    numbers = [int(n) for n in re.findall(r"#(\d+)", title or "")]
    for num in numbers:
        mid = _milestone_for_issue(reg, num)
        if mid:
            return {"milestone": mid, "issue": num}
    for m in reg.milestones.all():
        if re.search(rf"\b{re.escape(m.id)}\b", title or "", re.IGNORECASE):
            return {"milestone": m.id, "issue": m.issue}
    return {"milestone": "", "issue": numbers[0] if numbers else None}


def _pr_row(pr: dict[str, Any], reg: Registry, now: datetime) -> dict[str, Any]:
    ref = _milestone_ref(str(pr.get("title", "")), reg)
    merged_at = pr.get("mergedAt")
    created_at = pr.get("createdAt")
    stamp = merged_at or created_at
    return {
        "number": pr.get("number"),
        "title": str(pr.get("title", "")),
        "branch": str(pr.get("headRefName", "")),
        "url": str(pr.get("url", "")),
        "milestone": ref["milestone"],
        "milestone_issue": ref["issue"],
        "age_hours": _age_hours(stamp, now),
        "created_at": str(created_at or ""),
        "merged_at": str(merged_at or ""),
    }


def _prs_section(paths: HarnessPaths, reg: Registry, refresh: bool, now: datetime) -> dict[str, Any]:
    snap = github_prs(paths, refresh=refresh, now=now)
    if not snap.get("available"):
        return {
            "available": False,
            "reason": str(snap.get("reason", "")),
            "fetched_at": snap.get("fetched_at", ""),
            "from_cache": bool(snap.get("from_cache")),
            "age_seconds": snap.get("age_seconds"),
            "open": {"count": 0, "items": []},
            "merged_48h": {"count": 0, "items": []},
            "merged_limit": MERGED_PR_LIMIT,
        }
    open_rows = [_pr_row(p, reg, now) for p in snap.get("open") or [] if isinstance(p, dict)]
    merged_rows = [_pr_row(p, reg, now) for p in snap.get("merged") or [] if isinstance(p, dict)]
    recent = [r for r in merged_rows if r["age_hours"] is not None and r["age_hours"] <= 48]
    recent.sort(key=lambda r: r["age_hours"] or 0)
    open_rows.sort(key=lambda r: str(r.get("created_at", "")), reverse=True)
    return {
        "available": True,
        "reason": str(snap.get("reason", "")),
        "fetched_at": snap.get("fetched_at", ""),
        "from_cache": bool(snap.get("from_cache")),
        "age_seconds": snap.get("age_seconds"),
        "open": {"count": len(open_rows), "items": open_rows},
        "merged_48h": {"count": len(recent), "items": recent},
        "merged_limit": MERGED_PR_LIMIT,
    }


# ----------------------------------------------------------------------- delivery


def _worktree_items(paths: HarnessPaths, origin_main: str | None) -> tuple[bool, str, list[dict[str, Any]]]:
    """Raw worktree facts (git only, read-only). Cleanup candidacy is decided later."""
    ok, out, err = _git(["worktree", "list", "--porcelain"], paths)
    if not ok:
        return False, err, []
    blocks: list[dict[str, Any]] = []
    current: dict[str, Any] = {}
    for line in out.splitlines():
        if not line.strip():
            if current:
                blocks.append(current)
                current = {}
            continue
        key, _, value = line.partition(" ")
        current[key] = value.strip()
    if current:
        blocks.append(current)

    items: list[dict[str, Any]] = []
    for index, block in enumerate(blocks):
        path = block.get("worktree", "")
        head = block.get("HEAD", "")
        branch = block.get("branch", "").replace("refs/heads/", "")
        item: dict[str, Any] = {
            "path": path,
            "head": head,
            "branch": branch,
            "main": index == 0,
            "current": bool(path) and Path(path).resolve() == paths.repo.resolve(),
            "clean": None,
            "ignored_data": [],
            "ignored_count": 0,
            "blocking_ignored": False,
            "contained_in_origin_main": None,
            "reapable": False,
            "cleanup_candidate": False,
            "newest_write_hours": None,
            "idle": None,
            "stale": False,
        }
        if path and Path(path).is_dir():
            if not item["main"]:
                # The main checkout holds the owner's ignored material by design; scanning it
                # is slow and its result is never used.
                st_ok, st_out, _ = _git(["-C", path, "status", "--porcelain", "--ignored"], paths)
                if st_ok:
                    lines = [l for l in st_out.splitlines() if l.strip()]
                    item["clean"] = not [l for l in lines if not l.startswith("!!")]
                    ignored = [l[3:].strip() for l in lines if l.startswith("!!")]
                    item["ignored_count"] = len(ignored)
                    blockers = [p for p in ignored if not p.startswith(REGENERABLE_IGNORED)]
                    item["blocking_ignored"] = bool(blockers)
                    item["ignored_data"] = blockers[:10]
                hours = _newest_write_hours(path)
                item["newest_write_hours"] = hours
                item["idle"] = None if hours is None else hours >= 24
            else:
                item["clean"] = True
            if head and origin_main:
                anc_ok, _, _ = _git(["merge-base", "--is-ancestor", head, "origin/main"], paths)
                item["contained_in_origin_main"] = bool(anc_ok)
        items.append(item)
    return True, "", items


def _newest_write_hours(path: str, skip: tuple[str, ...] = (".git", "target", "node_modules", ".venv")) -> float | None:
    """Hours since the newest file under a worktree. None when unknown.

    Stops as soon as a file written within 24h is found (the only question the caller
    asks), and gives up rather than walk an unbounded tree.
    """
    now = datetime.now(timezone.utc).timestamp()
    newest: float | None = None
    count = 0
    try:
        for root, dirs, files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in skip]
            for name in files:
                count += 1
                if count > 200000:
                    return None
                try:
                    mtime = os.stat(os.path.join(root, name)).st_mtime
                except OSError:
                    continue
                age = (now - mtime) / 3600.0
                if newest is None or age < newest:
                    newest = age
                if newest < 24:
                    return round(newest, 2)
    except OSError:
        return None
    if newest is None:
        return None
    return round(newest, 2)


def _delivery_section(paths: HarnessPaths, now: datetime) -> dict[str, Any]:
    section: dict[str, Any] = {
        "available": True,
        "reason": "",
        "branch": "",
        "worktree": str(paths.repo),
        "repo": str(paths.repo),
        "origin_main": {"ref": "origin/main", "commit": "", "subject": "", "note": "last fetched value; run `git fetch --prune origin` before trusting it"},
        "branch_pr_check": {"available": False, "reason": "not attempted", "branches": 0, "queried": 0, "per_branch_limit": BRANCH_PR_LIMIT},
        "branches_with_merged_pr": [],
        "worktrees": {"available": False, "reason": "not attempted", "count": 0, "items": [], "cleanup_candidates": [], "stale_count": 0},
        "disk": {"available": False, "reason": "not attempted"},
    }

    ok_branch, branch, _ = _git(["rev-parse", "--abbrev-ref", "HEAD"], paths)
    section["branch"] = branch.strip() if ok_branch else ""

    ok_main, main_sha, main_err = _git(["rev-parse", "origin/main"], paths)
    origin_main = main_sha.strip() if ok_main else None
    if origin_main:
        ok_sub, subject, _ = _git(["log", "-1", "--format=%h %s", "origin/main"], paths)
        section["origin_main"]["commit"] = origin_main
        section["origin_main"]["subject"] = truncate(subject.strip() if ok_sub else "", 120)
    else:
        section["available"] = False
        section["reason"] = main_err or "cannot resolve origin/main"

    # ---- local branches (git only)
    trunk = _default_branch(paths)
    section["trunk"] = trunk
    ok_refs, refs_out, refs_err = _git(["for-each-ref", "--format=%(refname:short)\t%(objectname)", "refs/heads"], paths)
    local_branches: list[tuple[str, str]] = []
    if ok_refs:
        for line in refs_out.splitlines():
            name, _, sha = line.partition("\t")
            if name.strip():
                local_branches.append((name.strip(), sha.strip()))
    deliverable_branches = [name for name, _ in local_branches if name not in (trunk, "master")]

    # ---- the PR whose head is each local branch (one bounded, cached gh call per branch)
    branch_check = github_branch_prs(paths, deliverable_branches, now=now)
    by_branch: dict[str, dict[str, Any]] = {}
    for name, item in (branch_check.get("items") or {}).items():
        pr = item.get("pr") if isinstance(item, dict) else None
        if isinstance(pr, dict):
            by_branch[name] = pr
    section["branch_pr_check"] = {
        "available": bool(branch_check.get("available")),
        "reason": str(branch_check.get("reason", "")),
        "branches": len(local_branches),
        "queried": branch_check.get("queried", 0),
        "per_branch_limit": BRANCH_PR_LIMIT,
    }
    if not ok_refs:
        section["branch_pr_check"]["available"] = False
        section["branch_pr_check"]["reason"] = refs_err or "`git for-each-ref refs/heads` failed"

    # ---- completed work left on a branch
    orphans: list[dict[str, Any]] = []
    for name, sha in local_branches:
        if name in (trunk, "master"):
            continue
        pr = by_branch.get(name)
        if not pr or str(pr.get("state", "")).upper() != "MERGED":
            continue
        orphans.append(
            {
                "branch": name,
                "head": sha,
                "pr": pr.get("number"),
                "pr_title": str(pr.get("title", "")),
                "merged_at": str(pr.get("mergedAt", "")),
                "head_matches": bool(sha) and sha == str(pr.get("headRefOid", "")),
                "current": name == section["branch"],
                "fix": f"git branch -D {name} && git push origin --delete {name}",
            }
        )
    orphans.sort(key=lambda o: str(o.get("merged_at", "")), reverse=True)
    section["branches_with_merged_pr"] = orphans

    # ---- worktrees
    wt_ok, wt_err, items = _worktree_items(paths, origin_main)
    in_worktree: dict[str, list[str]] = {}
    for item in items:
        if item.get("branch"):
            in_worktree.setdefault(item["branch"], []).append(item["path"])
    for o in orphans:
        o["checked_out_in"] = in_worktree.get(o["branch"], [])

    if wt_ok:
        for item in items:
            if item["main"]:
                continue
            pr = by_branch.get(item.get("branch") or "")
            item["merged_pr_head"] = bool(
                pr and str(pr.get("state", "")).upper() == "MERGED" and item.get("head") == str(pr.get("headRefOid", ""))
            )
            item["reapable"] = bool(item["contained_in_origin_main"] or item["merged_pr_head"])
            item["reapable_reason"] = (
                "merged-pr-head" if item["merged_pr_head"] else ("contained-in-origin-main" if item["contained_in_origin_main"] else "")
            )
            # A worktree that is merely an ancestor of origin/main can be brand new (created
            # for work that has not started): only a merged PR head, or 24h of silence, makes
            # it completed work left behind rather than someone's live checkout.
            item["cleanup_candidate"] = bool(
                item["clean"]
                and item["reapable"]
                and not item["blocking_ignored"]
                and (item["merged_pr_head"] or item["idle"] is True)
            )
            item["stale"] = bool(item["cleanup_candidate"] or item["idle"])
        candidates = [i for i in items if i["cleanup_candidate"]]
        section["worktrees"] = {
            "available": True,
            "reason": "",
            "count": len(items),
            "items": items,
            "cleanup_candidates": candidates,
            "stale_count": len([i for i in items if i["stale"]]),
            "definition": (
                "cleanup candidate = a non-main worktree that is clean, holds no ignored data other than "
                "target/ or .harness/, and either (a) its HEAD is the exact head of a merged PR or (b) its HEAD "
                "is in origin/main and nothing under it was written for 24h (a fresh worktree at origin/main is "
                "someone's live checkout, not completed work); stale also counts a worktree untouched for 24h"
            ),
        }
    else:
        section["worktrees"] = {
            "available": False,
            "reason": wt_err or "`git worktree list --porcelain` failed",
            "count": 0,
            "items": [],
            "cleanup_candidates": [],
            "stale_count": 0,
            "definition": "cleanup candidate = a non-main worktree that is clean and already in origin/main",
        }

    # ---- disk
    try:
        usage = shutil.disk_usage(str(paths.repo))
        gib = 1024 ** 3
        section["disk"] = {
            "available": True,
            "reason": "",
            "path": str(paths.repo),
            "free_bytes": usage.free,
            "free_gib": round(usage.free / gib, 1),
            "total_gib": round(usage.total / gib, 1),
            "floor_gib": DISK_FLOOR_GIB,
            "ok": usage.free >= DISK_FLOOR_GIB * gib,
        }
    except OSError as exc:
        section["disk"] = {
            "available": False,
            "reason": f"shutil.disk_usage({paths.repo}): {type(exc).__name__}: {exc}",
        }
    return section


# -------------------------------------------------------------------------- stale


CARD_MARKERS = ("status card", "blocked card", "pivot card", "headline")
HEADLINE_WINDOW = 120


def _newest_card(comments: list[dict[str, Any]]) -> tuple[dict[str, Any], str] | None:
    """The newest comment that looks like a status card (newest first)."""
    ordered = sorted(comments or [], key=lambda c: str(c.get("createdAt", "")), reverse=True)
    for c in ordered:
        body = str(c.get("body", ""))
        low = body.lower()
        if any(marker in low for marker in CARD_MARKERS):
            return c, body
    return None


def _card_reading(comments: list[dict[str, Any]], milestone: Milestone) -> dict[str, Any]:
    """What the newest status card on the issue says about this milestone's number.

    Primary form: `M<n>: <n>/<n>`. Fallback: the numbers right after a `headline`
    marker, which is how the labs actually write the card today.
    """
    card = _newest_card(comments)
    if card is None:
        return {"kind": "no_card", "ts": "", "text": "", "current": None, "of": None}
    comment, body = card
    ts = str(comment.get("createdAt", ""))
    strict = re.search(rf"\b{re.escape(milestone.id)}\s*[:=]\s*(\d+)\s*/\s*(\d+)", body)
    if strict:
        return {"kind": "milestone_number", "ts": ts, "text": strict.group(0), "current": int(strict.group(1)), "of": int(strict.group(2))}
    marker = re.search(r"(?i)headline\s*[:\-]?", body)
    if not marker:
        return {"kind": "card_no_number", "ts": ts, "text": truncate(body, HEADLINE_WINDOW), "current": None, "of": None}
    fragment = body[marker.end() : marker.end() + HEADLINE_WINDOW]
    numbers = re.search(r"(\d+)\s*/\s*(\d+)", fragment)
    yes_no = None if numbers else re.search(r"(?i)\b(yes|no)\b", fragment)
    return {
        "kind": "headline_number" if numbers else "headline_text",
        "ts": ts,
        "text": truncate(fragment, HEADLINE_WINDOW),
        "current": int(numbers.group(1)) if numbers else None,
        "of": int(numbers.group(2)) if numbers else None,
        "yes_no": yes_no.group(1).lower() if yes_no else "",
    }


def _registry_numbers(milestone: Milestone) -> list[tuple[int, int]]:
    """Every (current, of) the registry claims for this milestone."""
    pairs: list[tuple[int, int]] = []
    for row in [milestone.headline or {}, *(milestone.secondary or [])]:
        if not isinstance(row, dict):
            continue
        cur, of = row.get("current"), row.get("of")
        if isinstance(cur, (int, float)) and isinstance(of, (int, float)):
            pairs.append((int(cur), int(of)))
    return pairs


def _stale_section(paths: HarnessPaths, reg: Registry, issues: dict[str, Any], rows: list[dict[str, Any]], now: datetime) -> dict[str, Any]:
    issue_items: list[dict[str, Any]] = []
    if issues.get("available"):
        for m in reg.milestones.all():
            item = (issues.get("issues") or {}).get(str(m.issue)) or {}
            age = _age_hours(item.get("updatedAt"), now)
            issue_items.append(
                {
                    "milestone": m.id,
                    "issue": m.issue,
                    "updated_at": str(item.get("updatedAt", "")),
                    "age_hours": age,
                    "stale": bool(age is not None and age > STALE_HOURS),
                }
            )
    receipt_items: list[dict[str, Any]] = []
    for m in reg.milestones.all():
        mine = _milestone_receipts(rows, m)
        last_any = max((str(r.get("ts", "")) for r in mine), default="")
        heartbeats = _milestone_receipts(rows, m, kind="heartbeat")
        last_hb = max((str(r.get("ts", "")) for r in heartbeats), default="")
        hb_24h = [r for r in heartbeats if (_age_hours(r.get("ts"), now) or 1e9) <= 24]
        posted_24h = [r for r in hb_24h if r.get("posted")]
        age_any = _age_hours(last_any, now) if last_any else None
        last_hb_row = max(heartbeats, key=lambda r: str(r.get("ts", "")), default={}) if heartbeats else {}
        receipt_items.append(
            {
                "milestone": m.id,
                "key": m.key,
                "issue": m.issue,
                "status": m.status,
                "count_24h": len([r for r in mine if (_age_hours(r.get("ts"), now) or 1e9) <= 24]),
                "last_receipt": last_any,
                "last_receipt_age_hours": age_any,
                "last_heartbeat": last_hb,
                "last_heartbeat_age_hours": _age_hours(last_hb, now) if last_hb else None,
                "heartbeats_24h": len(hb_24h),
                "posted_heartbeats_24h": len(posted_24h),
                "last_heartbeat_posted": bool(last_hb_row.get("posted")),
                "stale": bool(age_any is None or age_any > STALE_HOURS),
            }
        )
    stale_ids = [i["milestone"] for i in issue_items if i["stale"]] + [i["milestone"] for i in receipt_items if i["stale"] and i["milestone"] not in [x["milestone"] for x in issue_items if x["stale"]]]
    return {
        "threshold_hours": STALE_HOURS,
        "issues": {
            "available": bool(issues.get("available")),
            "reason": str(issues.get("reason", "")),
            "items": issue_items,
        },
        "milestones": {
            "available": True,
            "reason": "",
            "definition": "stale = no receipt of any kind for this milestone in the last 24h",
            "items": receipt_items,
        },
        "stale_milestones": stale_ids,
    }


# -------------------------------------------------------------------- build_status


def build_status(paths: HarnessPaths, reg: Registry, refresh: bool = False) -> dict[str, Any]:
    """One JSON-serialisable dict: milestones, rules, receipts, prs, delivery, stale."""
    now = _now()
    data: dict[str, Any] = {
        "schema": SCHEMA,
        "generated_at": now_iso(),
        "repo": str(paths.repo),
        "refresh": bool(refresh),
    }

    try:
        milestones, meta = _milestones_section(paths, reg)
        data["milestones"] = milestones
        data["registry"] = meta
    except Exception as exc:  # noqa: BLE001 - status must never raise
        data["milestones"] = []
        data["registry"] = {"path": str(paths.milestones_json), "tracker": {}, "note": f"registry read failed: {type(exc).__name__}: {exc}"}

    try:
        data["rules"] = _rules_section(reg)
    except Exception as exc:  # noqa: BLE001
        data["rules"] = []
        data["rules_error"] = f"{type(exc).__name__}: {exc}"

    rows: list[dict[str, Any]] = []
    try:
        receipts, _ = _receipt_rows(paths, now)
        rows = receipts
        data["receipts"] = _receipts_section(paths, now)
    except Exception as exc:  # noqa: BLE001
        data["receipts"] = {
            "available": False,
            "reason": f"ledger read failed: {type(exc).__name__}: {exc}",
            "ledger": {"dir": str(paths.ledger_dir), "month_file": "", "exists": False},
            "today": {"date": today(), "total": 0, "by_kind": {}},
            "month": {"month": month(), "total": 0, "by_kind": {}},
            "last5": [],
            "used_ids": {"skill": {}, "team": {}},
        }

    try:
        prs = _prs_section(paths, reg, refresh, now)
    except Exception as exc:  # noqa: BLE001
        prs = {
            "available": False,
            "reason": f"github read failed: {type(exc).__name__}: {exc}",
            "fetched_at": "",
            "from_cache": False,
            "age_seconds": None,
            "open": {"count": 0, "items": []},
            "merged_48h": {"count": 0, "items": []},
            "merged_limit": MERGED_PR_LIMIT,
        }
    data["prs"] = prs

    try:
        data["delivery"] = _delivery_section(paths, now)
    except Exception as exc:  # noqa: BLE001
        data["delivery"] = {
            "available": False,
            "reason": f"delivery read failed: {type(exc).__name__}: {exc}",
            "branch": "",
            "worktree": str(paths.repo),
            "repo": str(paths.repo),
            "origin_main": {"ref": "origin/main", "commit": "", "subject": "", "note": ""},
            "branch_pr_check": {"available": False, "reason": "delivery read failed", "checked": 0, "merged_limit": MERGED_PR_LIMIT},
            "branches_with_merged_pr": [],
            "worktrees": {"available": False, "reason": "delivery read failed", "count": 0, "items": [], "cleanup_candidates": [], "stale_count": 0},
            "disk": {"available": False, "reason": "delivery read failed"},
        }

    try:
        numbers = [m.issue for m in reg.milestones.all()]
        issues = github_issues(paths, numbers, refresh=refresh, with_comments=False, now=now)
        data["stale"] = _stale_section(paths, reg, issues, rows, now)
    except Exception as exc:  # noqa: BLE001
        data["stale"] = {
            "threshold_hours": STALE_HOURS,
            "issues": {"available": False, "reason": f"{type(exc).__name__}: {exc}", "items": []},
            "milestones": {"available": False, "reason": f"{type(exc).__name__}: {exc}", "definition": "", "items": []},
            "stale_milestones": [],
        }

    return data


# ------------------------------------------------------------------- render_status


def _unavailable(reason: str) -> str:
    return f"UNAVAILABLE: {reason or 'no reason given'}"


def _render_milestones(data: dict[str, Any], lines: list[str]) -> None:
    lines.append("1. MILESTONES")
    milestones = data.get("milestones") or []
    if not milestones:
        lines.append("   " + _unavailable(str((data.get("registry") or {}).get("note", "registry read failed"))))
    for m in milestones:
        lines.append(f"   {m.get('id')} #{m.get('issue')} [{m.get('lab') or 'unassigned'}, {m.get('status') or 'unknown'}] {m.get('headline')}")
        bits = []
        if m.get("line_counts"):
            bits.append(f"line: {m['line_counts']}")
        if m.get("panel"):
            bits.append(f"panel: {m['panel']}")
        if bits:
            lines.append("      " + " · ".join(bits))
        briefs = m.get("mechanisms") or []
        if briefs:
            rendered = ", ".join(f"{b.get('id')}{'' if b.get('exists') else ' (BRIEF MISSING)'}" for b in briefs)
            lines.append(f"      mechanisms: {rendered}")
        if m.get("next"):
            lines.append(f"      next: {truncate(str(m['next']), 220)}")
    rules = data.get("rules") or []
    if rules:
        lines.append("   owner rules (docs/harness/milestones.json):")
        for r in rules:
            lines.append(f"      {r.get('id')}: {truncate(str(r.get('rule', '')), 150)}")
    lines.append("")


def _render_receipts(data: dict[str, Any], lines: list[str]) -> None:
    lines.append("2. RECEIPTS / VISIBILITY")
    r = data.get("receipts") or {}
    if not r.get("available"):
        lines.append("   " + _unavailable(str(r.get("reason", "ledger unreadable"))))
    else:
        ledger = r.get("ledger") or {}
        today_part = r.get("today") or {}
        month_part = r.get("month") or {}
        lines.append(
            f"   {today_part.get('date')}: {today_part.get('total', 0)} receipts"
            f" ({_kind_text(today_part.get('by_kind'))}) · {month_part.get('month')}: {month_part.get('total', 0)}"
            f" ({_kind_text(month_part.get('by_kind'))})"
        )
        if not ledger.get("exists"):
            lines.append(f"   note: {ledger.get('month_file')} does not exist yet - no receipt has ever been recorded")
        used = r.get("used_ids") or {}
        lines.append(f"   skills used: {_id_counts(used.get('skill'))} · agents dispatched: {_id_counts(used.get('team'))}")
        last5 = r.get("last5") or []
        if not last5:
            lines.append("   last 5 receipts: none")
        for rec in last5[:5]:
            lines.append(f"   - {rec.get('ts')} {rec.get('kind')}: {truncate(_receipt_brief(rec), 150)}")
    stale = data.get("stale") or {}
    issues = stale.get("issues") or {}
    if issues.get("available"):
        flagged = [i for i in issues.get("items") or [] if i.get("stale")]
        if flagged:
            txt = ", ".join(f"{i['milestone']} #{i['issue']} ({_hours_text(i.get('age_hours'))})" for i in flagged)
            lines.append(f"   milestone issues not updated in 24h: {txt}")
        else:
            lines.append("   milestone issues: all four updated within 24h")
    else:
        lines.append("   milestone issue staleness: " + _unavailable(str(issues.get("reason", ""))))
    ms = stale.get("milestones") or {}
    if ms.get("available"):
        flagged = [i for i in ms.get("items") or [] if i.get("stale")]
        if flagged:
            txt = ", ".join(
                f"{i['milestone']} ({'no receipt on record' if not i.get('last_receipt') else 'last receipt ' + _hours_text(i.get('last_receipt_age_hours')) + ' ago'})"
                for i in flagged
            )
            lines.append(f"   milestones with no receipt in 24h: {txt}")
        else:
            lines.append("   milestones with no receipt in 24h: none")
        printed_only = [
            i["milestone"]
            for i in ms.get("items") or []
            if i.get("heartbeats_24h") and not i.get("posted_heartbeats_24h")
        ]
        if printed_only:
            lines.append(
                "   heartbeat cards emitted locally but never posted on the issue in 24h: "
                + ", ".join(printed_only)
                + " (post with `heartbeat --milestone <m> --post`, or paste the card)"
            )
    lines.append("")


def _kind_text(counts: Any) -> str:
    if not isinstance(counts, dict) or not counts:
        return "no kinds"
    return ", ".join(f"{k} {v}" for k, v in counts.items())


def _id_counts(counts: Any) -> str:
    if not isinstance(counts, dict) or not counts:
        return "none in this ledger month"
    return ", ".join(f"{k} x{v}" for k, v in sorted(counts.items()))


def _receipt_brief(rec: dict[str, Any]) -> str:
    parts = []
    for key in ("milestone", "skill", "role", "count", "outcome", "task", "headline", "line", "note"):
        value = rec.get(key)
        if value not in (None, "", []):
            parts.append(f"{key}={value}")
    return " ".join(parts) or "(no fields)"


def _render_delivery(data: dict[str, Any], lines: list[str]) -> None:
    lines.append("3. DELIVERY")
    d = data.get("delivery") or {}
    if not d.get("available"):
        lines.append("   " + _unavailable(str(d.get("reason", ""))))
    origin = d.get("origin_main") or {}
    lines.append(
        f"   branch {d.get('branch') or '?'} · worktree {d.get('worktree') or '?'}"
        f" · origin/main {str(origin.get('commit', ''))[:8] or '?'} ({origin.get('note', '')})"
    )
    check = d.get("branch_pr_check") or {}
    if not check.get("available"):
        lines.append("   branches with a merged PR: " + _unavailable(str(check.get("reason", ""))))
    else:
        orph = d.get("branches_with_merged_pr") or []
        if not orph:
            lines.append("   branches with a merged PR still present: none")
        else:
            lines.append(f"   branches with a merged PR still present: {len(orph)}")
            for o in orph[:3]:
                moved = "" if o.get("head_matches") else " (branch head has moved on)"
                where = f" [checked out in {', '.join(o['checked_out_in'])}]" if o.get("checked_out_in") else ""
                lines.append(f"      {o.get('branch')} -> PR #{o.get('pr')} merged {str(o.get('merged_at', ''))[:10]}{moved}{where}")
                lines.append(f"         fix: {o.get('fix')}")
            if len(orph) > 3:
                lines.append(f"      (+{len(orph) - 3} more)")
    wt = d.get("worktrees") or {}
    if not wt.get("available"):
        lines.append("   worktrees: " + _unavailable(str(wt.get("reason", ""))))
    else:
        cands = wt.get("cleanup_candidates") or []
        lines.append(
            f"   worktrees: {wt.get('count', 0)} total · {len(cands)} clean and already in origin/main"
            f" (cleanup candidates) · {wt.get('stale_count', 0)} stale"
        )
        for c in cands[:3]:
            here = " (this worktree)" if c.get("current") else ""
            lines.append(f"      remove {c.get('path')} [{c.get('branch') or 'detached'}]{here}")
        if len(cands) > 3:
            lines.append(f"      (+{len(cands) - 3} more)")
        lines.append(f"      definition: {wt.get('definition', '')}")
    disk = d.get("disk") or {}
    if not disk.get("available"):
        lines.append("   disk: " + _unavailable(str(disk.get("reason", ""))))
    else:
        verdict = "ok" if disk.get("ok") else f"BELOW the {disk.get('floor_gib')} GiB floor"
        lines.append(f"   disk: {disk.get('free_gib')} GiB free of {disk.get('total_gib')} GiB ({verdict})")
    lines.append("")


def _render_github(data: dict[str, Any], lines: list[str]) -> None:
    lines.append("4. GITHUB")
    prs = data.get("prs") or {}
    if not prs.get("available"):
        lines.append("   PRs: " + _unavailable(str(prs.get("reason", ""))))
    else:
        age = prs.get("age_seconds")
        source = f"cached {int(age) // 60}m ago" if prs.get("from_cache") and age is not None else "fetched now"
        lines.append(f"   PRs ({source}, docs cached at .harness/github.json):")
        open_part = prs.get("open") or {}
        merged_part = prs.get("merged_48h") or {}
        lines.append(f"      open {open_part.get('count', 0)} · merged in the last 48h {merged_part.get('count', 0)}")
        for p in (merged_part.get("items") or [])[:5]:
            ref = p.get("milestone") or (f"#{p['milestone_issue']}" if p.get("milestone_issue") else "no milestone ref")
            lines.append(f"      #{p.get('number')} [{ref}] {_hours_text(p.get('age_hours'))} old: {truncate(str(p.get('title', '')), 80)}")
        for p in (open_part.get("items") or [])[:5]:
            ref = p.get("milestone") or (f"#{p['milestone_issue']}" if p.get("milestone_issue") else "no milestone ref")
            lines.append(f"      OPEN #{p.get('number')} [{ref}] {_hours_text(p.get('age_hours'))} old: {truncate(str(p.get('title', '')), 80)}")
    lines.append("")


def render_status(data: dict[str, Any]) -> str:
    """A compact, numbered view a human reads in 20 seconds."""
    if not isinstance(data, dict):
        return f"uor-harness status\n{_unavailable('build_status returned no data')}"
    lines: list[str] = []
    lines.append(f"uor-harness status - {data.get('generated_at', 'no timestamp')}")
    registry = data.get("registry") or {}
    note = truncate(str(registry.get("note", "")), 160)
    lines.append(f"source: {registry.get('path', 'docs/harness/milestones.json')} ({len(data.get('milestones') or [])} milestones)" + (f" · {note}" if note else ""))
    lines.append("")
    _render_milestones(data, lines)
    _render_receipts(data, lines)
    _render_delivery(data, lines)
    _render_github(data, lines)
    return "\n".join(lines).rstrip() + "\n"


# ----------------------------------------------------------------------- heartbeat


def _pick_line(milestone: Milestone, override: str | None) -> tuple[str, str]:
    """Return (line name, count text). Default: the line nearest its 3/3 pivot."""
    lines = [l for l in (milestone.lines or []) if isinstance(l, dict)]
    chosen: dict[str, Any] | None = None
    if override:
        for l in lines:
            if str(l.get("name", "")).lower() == override.strip().lower():
                chosen = l
                break
    if chosen is None and lines:
        chosen = max(lines, key=lambda l: _line_ratio(l))
    name = override.strip() if override else str((chosen or {}).get("name", "no line registered"))
    count = f"{chosen.get('count', 0)}/{chosen.get('of', 3)}" if chosen else "0/3"
    return name, count


def _line_ratio(line: dict[str, Any]) -> float:
    try:
        return float(line.get("count", 0)) / max(1.0, float(line.get("of", 3)))
    except (TypeError, ValueError):
        return 0.0


def _how_headline_moves(milestone: Milestone) -> str:
    body = _headline_text(milestone)
    prefix = f"{milestone.id}: "
    if body.startswith(prefix):
        body = body[len(prefix) :]
    return f"moves the headline {body}"


def _record_heartbeat(paths: HarnessPaths, milestone: Milestone, line_name: str, count: str, next_text: str, card: str, posted: bool) -> tuple[dict[str, Any] | None, str]:
    try:
        import receipts as R  # noqa: PLC0415 - optional sibling module

        record = R.record(
            paths,
            "heartbeat",
            milestone=milestone.key,
            headline=_headline_text(milestone),
            line=line_name,
            count=count,
            next=next_text,
            posted=bool(posted),
        )
        return (record if isinstance(record, dict) else None), ""
    except ImportError:
        return None, "receipts module not present; heartbeat not recorded in docs/harness/ledger/"
    except Exception as exc:  # noqa: BLE001 - the card matters more than the ledger
        return None, f"receipt not recorded ({type(exc).__name__}: {exc})"


def _post_card(paths: HarnessPaths, issue: int, card: str) -> tuple[bool, str, str]:
    cmd = ["gh", "issue", "comment", str(issue), "--body-file", "-"]
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(paths.repo),
            input=card,
            capture_output=True,
            text=True,
            timeout=GH_TIMEOUT,
            check=False,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return False, f"`gh issue comment {issue} --body-file -` could not run: {type(exc).__name__}: {exc}", ""
    out = redact((proc.stdout or "").strip())
    if proc.returncode != 0:
        err = redact(((proc.stderr or "") + (proc.stdout or "")).strip().replace("\n", " "))[:240]
        return False, f"`gh issue comment {issue} --body-file -` exited {proc.returncode}: {err or 'no output'}", out
    return True, "", out


def heartbeat(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    """The 90-minute card: `headline · line · count n/3 · next step`."""
    key = getattr(args, "milestone", None)
    milestone = reg.milestones.get(key) if key else None
    if milestone is None:
        emit(f"heartbeat: unknown milestone {key!r}; known: {', '.join(sorted(reg.milestones.by_key))}", sys.stderr)
        return 2

    line_name, count = _pick_line(milestone, getattr(args, "line", None))
    override_count = getattr(args, "count", None)
    if override_count:
        count = str(override_count) if "/" in str(override_count) else f"{override_count}/{_line_of(milestone) or 3}"
    next_text = getattr(args, "next_step", None) or milestone.next or "(no next step registered)"
    how = _how_headline_moves(milestone)

    card = " · ".join(
        [
            f"{milestone.id} #{milestone.issue}",
            _headline_text(milestone),
            f"line: {line_name}",
            f"count {count}",
            f"next: {next_text}",
            f"[{how}]",
        ]
    )

    posted = False
    post_command = f"gh issue comment {milestone.issue} --body-file -"
    post_error = ""
    if getattr(args, "post", False):
        posted, post_error, _ = _post_card(paths, milestone.issue, card)

    record, receipt_note = _record_heartbeat(paths, milestone, line_name, count, next_text, card, posted)

    payload = {
        "milestone": milestone.key,
        "id": milestone.id,
        "issue": milestone.issue,
        "headline": _headline_text(milestone),
        "line": line_name,
        "count": count,
        "next": next_text,
        "how_headline_moves": how,
        "card": card,
        "posted": posted,
        "post_command": post_command,
        "receipt": record,
        "receipt_note": receipt_note,
    }
    if getattr(args, "json", False):
        emit(json.dumps(payload, indent=2, ensure_ascii=False))
    else:
        emit(card)
        if post_error:
            emit(f"post failed: {post_error}", sys.stderr)
        elif posted:
            emit(f"posted to #{milestone.issue} via `{post_command}`", sys.stderr)
        if receipt_note:
            emit(receipt_note, sys.stderr)
        elif record:
            emit(f"receipt recorded ({record.get('ts', '')}) in {paths.ledger_dir}", sys.stderr)
    if getattr(args, "post", False) and not posted:
        return 1
    return 0


def _line_of(milestone: Milestone) -> Any:
    lines = [l for l in (milestone.lines or []) if isinstance(l, dict)]
    if not lines:
        return None
    return max(lines, key=_line_ratio).get("of")


# --------------------------------------------------------------------------- rules


def _rule_heartbeat_receipts(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    """No heartbeat receipt in 24h for an in-progress milestone."""
    out: list[RuleResult] = []
    try:
        rows, _ = _receipt_rows(paths, _now())
    except Exception as exc:  # noqa: BLE001
        return [
            RuleResult(
                rule_id="status.heartbeat_receipts",
                severity="warn",
                ok=False,
                message=f"cannot read the ledger: {type(exc).__name__}: {exc}",
                path=str(paths.ledger_dir),
                fix="python3 scripts/harness/cli.py skill use <id>  # or fix docs/harness/ledger/",
            )
        ]
    now = _now()
    for m in reg.milestones.all():
        if m.status != "in-progress":
            continue
        heartbeats = _milestone_receipts(rows, m, kind="heartbeat")
        last = max((str(r.get("ts", "")) for r in heartbeats), default="")
        age = _age_hours(last, now) if last else None
        if age is not None and age <= STALE_HOURS:
            continue
        seen = f"last heartbeat {_hours_text(age)} ago" if last else "no heartbeat receipt on record"
        out.append(
            RuleResult(
                rule_id="status.heartbeat_receipts",
                severity="warn",
                ok=False,
                message=f"{m.id} #{m.issue} is in-progress and {seen} (docs/labs/session-goal.md 5: a card every 90 minutes or 3 merged PRs)",
                path="docs/harness/milestones.json",
                fix=f"python3 scripts/harness/cli.py heartbeat --milestone {m.key}",
                meta={"milestone": m.key, "issue": m.issue, "last_heartbeat": last, "age_hours": age},
            )
        )
    return out


def _rule_registry_vs_github(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    """The registry headline must agree with the newest card on the milestone issue."""
    numbers = [m.issue for m in reg.milestones.all()]
    try:
        snap = github_issues(paths, numbers, with_comments=True)
    except Exception as exc:  # noqa: BLE001
        return [
            RuleResult(
                rule_id="status.registry_vs_github",
                severity="info",
                ok=True,
                message=f"GitHub not checked: {type(exc).__name__}: {exc}",
                path=str(paths.milestones_json),
            )
        ]
    if not snap.get("available"):
        return [
            RuleResult(
                rule_id="status.registry_vs_github",
                severity="info",
                ok=True,
                message=f"GitHub unavailable, registry not cross-checked: {snap.get('reason', '')}",
                path=str(paths.milestones_json),
                meta={"github": False},
            )
        ]
    out: list[RuleResult] = []
    for m in reg.milestones.all():
        item = (snap.get("issues") or {}).get(str(m.issue)) or {}
        comments = item.get("comments")
        if comments is None:
            out.append(
                RuleResult(
                    rule_id="status.registry_vs_github",
                    severity="info",
                    ok=True,
                    message=f"{m.id} #{m.issue}: no comments cached for this issue; not compared",
                    path=str(paths.milestones_json),
                )
            )
            continue
        reading = _card_reading(comments, m)
        head = m.headline or {}
        reg_cur, reg_of = head.get("current"), head.get("of")
        reg_text = str(head.get("text", "") or "")
        reg_repr = f"{head.get('label', 'headline')} {'?' if reg_cur is None else reg_cur}{'/' + str(reg_of) if reg_of else ''}"
        registry_pairs = _registry_numbers(m)
        if reading["kind"] == "no_card":
            out.append(
                RuleResult(
                    rule_id="status.registry_vs_github",
                    severity="info",
                    ok=True,
                    message=f"{m.id} #{m.issue}: no status card found in the last {len(comments)} comments; nothing to compare",
                    path=str(paths.milestones_json),
                    meta={"milestone": m.key, "issue": m.issue, "compared": False},
                )
            )
            continue
        if reading["current"] is None:
            # A qualitative card (yes/no) agrees when the registry records the same answer.
            if reading.get("yes_no") and reading["yes_no"] in reg_text.lower():
                continue
            if reading.get("yes_no") and (reg_of is not None or reg_cur is not None):
                out.append(
                    RuleResult(
                        rule_id="status.registry_vs_github",
                        severity="warn",
                        ok=False,
                        message=(
                            f"{m.id} #{m.issue}: registry states a number ({reg_repr}), "
                            f"the newest card ({reading['ts']}) answers {reading['yes_no']!r}: {reading['text']!r}"
                        ),
                        path="docs/harness/milestones.json",
                        fix=f"reconcile docs/harness/milestones.json with issue #{m.issue} (or make the card state `{m.id}: n/n`)",
                        meta={"milestone": m.key, "issue": m.issue, "card": reading},
                    )
                )
                continue
            out.append(
                RuleResult(
                    rule_id="status.registry_vs_github",
                    severity="info",
                    ok=True,
                    message=(
                        f"{m.id} #{m.issue}: newest card ({reading['ts']}) has no machine-readable number; "
                        f"registry says {reg_repr}; card text: {reading['text']!r}"
                    ),
                    path=str(paths.milestones_json),
                    meta={"milestone": m.key, "issue": m.issue, "compared": False, "card": reading},
                )
            )
            continue
        if (reading["current"], reading["of"]) in registry_pairs:
            continue
        out.append(
            RuleResult(
                rule_id="status.registry_vs_github",
                severity="warn",
                ok=False,
                message=(
                    f"{m.id} #{m.issue}: registry says {reg_repr}, newest card ({reading['ts']}) says "
                    f"{reading['current']}/{reading['of']} ({reading['kind']} in {reading['text']!r})"
                ),
                path="docs/harness/milestones.json",
                fix=f"reconcile docs/harness/milestones.json with issue #{m.issue}, then re-run `python3 scripts/harness/cli.py status --refresh`",
                meta={
                    "milestone": m.key,
                    "issue": m.issue,
                    "registry": [reg_cur, reg_of],
                    "card": [reading["current"], reading["of"]],
                    "card_ts": reading["ts"],
                },
            )
        )
    return out


def _rule_no_orphan_branches(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    """Completed work left on a branch: orphan local branches and reapable worktrees."""
    now = _now()
    try:
        delivery = _delivery_section(paths, now)
    except Exception as exc:  # noqa: BLE001
        return [
            RuleResult(
                rule_id="status.no_orphan_branches",
                severity="info",
                ok=True,
                message=f"delivery state not readable: {type(exc).__name__}: {exc}",
                path=str(paths.repo),
            )
        ]
    out: list[RuleResult] = []
    check = delivery.get("branch_pr_check") or {}
    if check.get("available"):
        for o in delivery.get("branches_with_merged_pr") or []:
            moved = "" if o.get("head_matches") else " (its head has moved on since that merge)"
            where = f"; checked out in {', '.join(o['checked_out_in'])}" if o.get("checked_out_in") else ""
            out.append(
                RuleResult(
                    rule_id="status.no_orphan_branches",
                    severity="warn",
                    ok=False,
                    message=(
                        f"local branch {o.get('branch')} still exists but PR #{o.get('pr')} merged "
                        f"{str(o.get('merged_at', ''))[:10]}{moved}{where}"
                    ),
                    path=str(paths.repo),
                    fix=str(o.get("fix", "")),
                    meta={"kind": "branch", "branch": o.get("branch"), "pr": o.get("pr")},
                )
            )
    else:
        out.append(
            RuleResult(
                rule_id="status.no_orphan_branches",
                severity="info",
                ok=True,
                message=f"merged-PR branch check skipped: {check.get('reason', 'gh unavailable')}",
                path=str(paths.repo),
            )
        )
    worktrees = delivery.get("worktrees") or {}
    if worktrees.get("available"):
        for c in worktrees.get("cleanup_candidates") or []:
            here = " (this worktree; run the command from another checkout)" if c.get("current") else ""
            why = (
                "its HEAD is the exact head of a merged PR"
                if c.get("reapable_reason") == "merged-pr-head"
                else "it holds no commits beyond origin/main and nothing has been written in it for 24h"
            )
            out.append(
                RuleResult(
                    rule_id="status.no_orphan_branches",
                    severity="warn",
                    ok=False,
                    message=(
                        f"worktree {c.get('path')} [{c.get('branch') or 'detached'}] is clean and completed work "
                        f"was left behind: {why}{here}"
                    ),
                    path=str(c.get("path", "")),
                    fix=f"git worktree remove {c.get('path')}"
                    + (f" && git branch -D {c.get('branch')}" if c.get("branch") else "")
                    + " && scripts/storage/uor-hygiene --apply",
                    meta={"kind": "worktree", "path": c.get("path"), "branch": c.get("branch"), "why": c.get("reapable_reason")},
                )
            )
    else:
        out.append(
            RuleResult(
                rule_id="status.no_orphan_branches",
                severity="info",
                ok=True,
                message=f"worktree check skipped: {worktrees.get('reason', 'git unavailable')}",
                path=str(paths.repo),
            )
        )
    return out


def register_rules() -> list[Rule]:
    """Rules this module owns. `check` functions never raise."""
    return [
        Rule(
            id="status.heartbeat_receipts",
            description="every in-progress milestone posts a status card at least every 24h",
            severity="warn",
            check=_rule_heartbeat_receipts,
            rationale="docs/labs/session-goal.md 5 (owner direction, D21): without these cards nobody outside the lab can tell what the lab is doing",
            scope="repo",
        ),
        Rule(
            id="status.registry_vs_github",
            description="the registry headline agrees with the newest status card on the milestone issue",
            severity="warn",
            check=_rule_registry_vs_github,
            rationale="docs/harness/milestones.json is a cache of the live GitHub contract; a silent divergence makes every lab's headline wrong",
            mistake="headline-number-not-regenerated",
            scope="repo",
        ),
        Rule(
            id="status.no_orphan_branches",
            description="no local branch whose PR merged, and no clean worktree already in origin/main",
            severity="warn",
            check=_rule_no_orphan_branches,
            rationale="AGENTS.md (owner, 9 October 2026): branches are temporary transport; there are no standing branches",
            scope="repo",
        ),
    ]
