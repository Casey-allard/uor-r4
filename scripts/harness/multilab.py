"""Multi-lab coordination: presence, committed claims, and per-lab ledger shards.

The problem this module exists for. One checkout with a local `.harness/claims.json` and one
shared `docs/harness/ledger/<month>.jsonl` works for a single lab and breaks at four:

* a claim held in a gitignored file is invisible to every other lab, so "who owns this branch"
  is unanswerable;
* one shared ledger file is appended by everyone, so every concurrent PR conflicts on it - the
  author of this module resolved that union by hand twice in one session, which does not scale;
* nothing anywhere answers "what is lab X doing right now", because a receipt only exists after
  it has been committed in a PR.

The design that fixes all three, in the order of how much it matters:

1. **Presence is a per-lab file, refreshed by `heartbeat`.** `docs/harness/presence/<lab>.json`
   says where a lab is, what it is working on, and when it last checked in. Stale after
   `PRESENCE_TTL_MINUTES`; the reader says STALE rather than showing a false "running".
2. **Claims are committed.** `docs/harness/claims.json`, one entry per branch, so any lab can see
   who holds what before touching a branch. Still advisory in the filesystem sense - what enforces
   it is `status.branch_claimed` on every check and the pre-commit hook.
3. **The ledger is sharded per lab.** `docs/harness/ledger/<lab>-<month>.jsonl`. Two labs never
   write the same file, so no merge conflicts; a lab's own history stays in one file per month.

Honest limits, stated because a coordination layer that oversells itself is worse than none:

* This is **not** real-time. Presence is as fresh as the last heartbeat, and a receipt is visible
  only once committed. GitHub is the bus and `main` is the truth; there is no socket anywhere.
* It cannot see across vendor CLIs. A lab that never runs the harness is simply absent.
* A presence file that stops being refreshed says STALE, and STALE is not "idle" - it means the
  lab may be working without checking in, which is exactly what the session contract forbids.
"""

from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from core import HarnessPaths, now_iso, run
from checkrules import Rule, RuleResult, register as register_rule

#: a presence file older than this is reported STALE, never as a live lab
PRESENCE_TTL_MINUTES = 30
#: labs known to the project. A lab may still identify itself; this is for reporting order.
KNOWN_LABS = ("kimi", "claude", "codex", "deepseek")

_SLUG = re.compile(r"[^a-z0-9._-]+")


# ------------------------------------------------------------------------------- helpers


def slug(name: str) -> str:
    """A safe file stem for a lab name."""
    s = _SLUG.sub("-", str(name).strip().lower()).strip("-")
    return s or "unknown"


def presence_dir(paths: HarnessPaths) -> Path:
    return paths.harness / "presence"


def presence_path(paths: HarnessPaths, lab: str) -> Path:
    return presence_dir(paths) / f"{slug(lab)}.json"


def claims_file(paths: HarnessPaths) -> Path:
    return paths.harness / "claims.json"


def shard_path(paths: HarnessPaths, lab: str, month: str | None = None) -> Path:
    from core import month as current_month

    return paths.ledger_dir / f"{slug(lab)}-{(month or current_month())}.jsonl"


def _load(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return data if isinstance(data, dict) else {}


def _save(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(data, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")
    tmp.replace(path)


def _age_minutes(stamp: str) -> float | None:
    try:
        when = datetime.fromisoformat(stamp)
    except (TypeError, ValueError):
        return None
    if when.tzinfo is None:
        when = when.replace(tzinfo=timezone.utc)
    return (datetime.now(timezone.utc) - when).total_seconds() / 60.0


# ------------------------------------------------------------------------------ presence


@dataclass
class Presence:
    lab: str
    updated: str
    milestone: str = ""
    branch: str = ""
    worktree: str = ""
    doing: str = ""
    next: str = ""
    host: str = ""
    session: str = ""
    schema: int = 1

    @property
    def age_minutes(self) -> float | None:
        return _age_minutes(self.updated)

    @property
    def stale(self) -> bool:
        age = self.age_minutes
        return age is None or age > PRESENCE_TTL_MINUTES

    @property
    def state(self) -> str:
        if self.stale:
            age = self.age_minutes
            shown = "unknown age" if age is None else f"{age:.0f}m old"
            return f"STALE ({shown})"
        return f"live ({self.age_minutes:.0f}m ago)"


def write_presence(
    paths: HarnessPaths,
    lab: str,
    *,
    milestone: str = "",
    doing: str = "",
    next_step: str = "",
    branch: str | None = None,
    worktree: str | None = None,
    session: str = "",
) -> Presence:
    """Refresh this lab's presence file. Called by `heartbeat`, so it cannot be forgotten."""
    if not lab:
        raise ValueError("a lab name is required: presence without an owner is not presence")
    current = read_presence(paths, lab)
    rec = Presence(
        lab=slug(lab),
        updated=now_iso(),
        milestone=milestone or (current.milestone if current else ""),
        branch=branch if branch is not None else _current_branch(paths),
        worktree=worktree or str(paths.repo),
        doing=doing or (current.doing if current else ""),
        next=next_step or (current.next if current else ""),
        host=_hostname(),
        session=session,
    )
    _save(presence_path(paths, lab), asdict(rec))
    return rec


def read_presence(paths: HarnessPaths, lab: str) -> Presence | None:
    data = _load(presence_path(paths, lab))
    if not data:
        return None
    return _to_presence(data)


def all_presence(paths: HarnessPaths) -> list[Presence]:
    if not presence_dir(paths).is_dir():
        return []
    out = []
    for f in sorted(presence_dir(paths).glob("*.json")):
        data = _load(f)
        if data:
            out.append(_to_presence(data))
    return sorted(out, key=lambda p: (p.stale, p.lab))


def _to_presence(data: dict[str, Any]) -> Presence:
    known = {f for f in Presence.__dataclass_fields__}
    return Presence(**{k: v for k, v in data.items() if k in known})


def _current_branch(paths: HarnessPaths) -> str:
    rc, out = run(["git", "-C", str(paths.repo), "branch", "--show-current"], timeout=10)
    return out.strip() if rc == 0 else ""


def _hostname() -> str:
    import socket

    try:
        return socket.gethostname()
    except OSError:  # pragma: no cover - environment
        return ""


# -------------------------------------------------------------------------------- claims

CLAIM_STALE_MINUTES = 12 * 60  # a claim held without activity for half a day is suspect


@dataclass
class Claim:
    branch: str
    lab: str
    claimed_at: str
    head: str = ""
    note: str = ""
    worktree: str = ""

    @property
    def age_minutes(self) -> float | None:
        return _age_minutes(self.claimed_at)

    @property
    def stale(self) -> bool:
        age = self.age_minutes
        return age is not None and age > CLAIM_STALE_MINUTES


def load_claims(paths: HarnessPaths) -> dict[str, dict[str, Any]]:
    return _load(claims_file(paths))


def save_claims(paths: HarnessPaths, claims: dict[str, dict[str, Any]]) -> None:
    _save(claims_file(paths), claims)


def claim(paths: HarnessPaths, branch: str, lab: str, note: str = "") -> tuple[bool, dict[str, Any]]:
    """Claim a branch. Refuses when another lab holds it and the holder is not stale."""
    if not branch:
        raise ValueError("a branch name is required")
    if not lab:
        raise ValueError("a lab is required: a claim without an owner is not a claim")
    claims = load_claims(paths)
    held = claims.get(branch)
    if held and held.get("lab") and held["lab"] != slug(lab):
        c = Claim(**{k: v for k, v in held.items() if k in Claim.__dataclass_fields__})
        if not c.stale:
            return False, held
    rc, out = run(["git", "-C", str(paths.repo), "rev-parse", "--short", "HEAD"], timeout=10)
    entry = {
        "branch": branch,
        "lab": slug(lab),
        "claimed_at": now_iso(),
        "head": out.strip() if rc == 0 else "",
        "note": note,
        "worktree": str(paths.repo),
    }
    claims[branch] = entry
    save_claims(paths, claims)
    return True, entry


def release(paths: HarnessPaths, branch: str, lab: str = "") -> tuple[bool, dict[str, Any]]:
    claims = load_claims(paths)
    held = claims.get(branch)
    if not held:
        return False, {}
    if lab and held.get("lab") and held["lab"] != slug(lab):
        return False, held
    del claims[branch]
    save_claims(paths, claims)
    return True, held


def claim_of(paths: HarnessPaths, branch: str) -> dict[str, Any] | None:
    return load_claims(paths).get(branch)


# -------------------------------------------------------------------------------- ledger


def append_receipt(paths: HarnessPaths, lab: str, record: dict[str, Any]) -> Path:
    """Append one receipt to this lab's shard. Returns the shard path.

    Sharded by lab so two labs never write the same file; that is the whole point (see the
    module docstring). A lab that has not identified itself writes to `unknown-<month>` rather
    than to a shared file, so the conflict is still avoided and the gap is visible.
    """
    shard = shard_path(paths, lab or "unknown")
    shard.parent.mkdir(parents=True, exist_ok=True)
    with shard.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
    return shard


def read_shards(paths: HarnessPaths, month: str | None = None) -> list[dict[str, Any]]:
    """Every receipt from every shard, oldest first, with its lab and file recorded."""
    if not paths.ledger_dir.is_dir():
        return []
    pattern = f"*-{month}.jsonl" if month else "*.jsonl"
    out: list[dict[str, Any]] = []
    for f in sorted(paths.ledger_dir.glob(pattern)):
        lab = f.name.rsplit("-", 1)[0]
        for line in f.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            rec.setdefault("lab", lab if lab != "unknown" else rec.get("lab", ""))
            rec["_file"] = f.name
            out.append(rec)
    out.sort(key=lambda r: str(r.get("ts", "")))
    return out


def legacy_ledgers(paths: HarnessPaths) -> list[Path]:
    """Unsharded ledgers from before sharding: `<YYYY-MM>.jsonl` with no lab prefix."""
    if not paths.ledger_dir.is_dir():
        return []
    return sorted(p for p in paths.ledger_dir.glob("*.jsonl") if re.fullmatch(r"\d{4}-\d{2}\.jsonl", p.name))


# --------------------------------------------------------------------------------- rules


def register_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="multilab.presence_fresh",
                description="every lab with a presence file has refreshed it recently",
                severity="warn",
                check=_check_presence_fresh,
                rationale="a stale presence file is not 'idle'; it means the lab is working without checking in",
            )
        ),
        register_rule(
            Rule(
                id="multilab.claims_one_owner",
                description="no branch is claimed by two labs",
                severity="error",
                check=_check_claims_one_owner,
                rationale="one branch, one owner - the rule the single-checkout policy depends on",
            )
        ),
        register_rule(
            Rule(
                id="multilab.no_shared_ledger",
                description="receipts are sharded per lab, not written to one shared file",
                severity="warn",
                check=_check_no_shared_ledger,
                rationale="one shared ledger conflicts on every concurrent PR; that does not scale past two labs",
                mistake="shared-ledger-conflicts",
            )
        ),
        register_rule(
            Rule(
                id="multilab.ledger_lab_recorded",
                description="every sharded receipt names its lab",
                severity="warn",
                check=_check_lab_recorded,
                rationale="a receipt with no lab cannot be attributed, which is the visibility the shards exist for",
            )
        ),
    ]


def _check_presence_fresh(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    out: list[RuleResult] = []
    for p in all_presence(paths):
        if p.stale:
            out.append(
                RuleResult(
                    rule_id="multilab.presence_fresh",
                    severity="warn",
                    ok=False,
                    message=f"lab {p.lab}: presence is {p.state}"
                            + (f" (last: {p.doing})" if p.doing else ""),
                    path=str(presence_path(paths, p.lab).relative_to(paths.repo)),
                    fix=f"the lab refreshes it with: cli.py heartbeat -m <milestone> --lab {p.lab}",
                )
            )
    return out


def _check_claims_one_owner(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    # the file is keyed by branch, so a double claim is only possible if a lab name is empty
    out: list[RuleResult] = []
    for branch, held in load_claims(paths).items():
        if not held.get("lab"):
            out.append(
                RuleResult(
                    rule_id="multilab.claims_one_owner",
                    severity="error",
                    ok=False,
                    message=f"branch {branch!r} is claimed with no lab",
                    path="docs/harness/claims.json",
                    fix="re-claim it with --lab <lab>, or release it",
                )
            )
    return out


def _check_no_shared_ledger(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    legacy = legacy_ledgers(paths)
    if not legacy:
        return []
    return [
        RuleResult(
            rule_id="multilab.no_shared_ledger",
            severity="warn",
            ok=False,
            message=f"{len(legacy)} unsharded ledger file(s) remain: {', '.join(p.name for p in legacy[:3])}",
            path="docs/harness/ledger",
            fix="run: cli.py multilab migrate-ledger   (moves receipts into per-lab shards, keeping history)",
        )
    ]


def _check_lab_recorded(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    out: list[RuleResult] = []
    for rec in read_shards(paths):
        if not rec.get("lab"):
            out.append(
                RuleResult(
                    rule_id="multilab.ledger_lab_recorded",
                    severity="warn",
                    ok=False,
                    message=f"{rec.get('_file')}: a receipt has no lab ({rec.get('kind', '?')} at {rec.get('ts', '?')})",
                    path=f"docs/harness/ledger/{rec.get('_file', '')}",
                    fix="identity comes from `--lab` or $UOR_LAB; pass one when recording",
                )
            )
    return out


# ------------------------------------------------------------------------------ migration


def migrate_ledger(paths: HarnessPaths) -> dict[str, Any]:
    """Move unsharded receipts into per-lab shards, preserving every line and its order."""
    moved: dict[str, int] = {}
    for f in legacy_ledgers(paths):
        month = f.stem
        lines = [l for l in f.read_text(encoding="utf-8").splitlines() if l.strip()]
        buckets: dict[str, list[str]] = {}
        for line in lines:
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                # never drop an unreadable line: keep it in an explicit quarantine shard
                buckets.setdefault("unreadable", []).append(line)
                continue
            lab = slug(str(rec.get("lab") or rec.get("actor") or "unknown"))
            if rec.get("lab") != lab:
                rec["lab"] = lab
            buckets.setdefault(lab, []).append(json.dumps(rec, ensure_ascii=False, sort_keys=True))
        for lab, rows in buckets.items():
            target = shard_path(paths, lab, month)
            existing = target.read_text(encoding="utf-8").splitlines() if target.exists() else []
            merged = [l for l in existing if l.strip()] + rows
            target.parent.mkdir(parents=True, exist_ok=True)
            # keep chronological order where the timestamps allow it
            def key(line: str) -> str:
                try:
                    return str(json.loads(line).get("ts", ""))
                except json.JSONDecodeError:
                    return ""
            target.write_text("\n".join(sorted(merged, key=key)) + "\n", encoding="utf-8")
            moved[f"{lab}-{month}"] = moved.get(f"{lab}-{month}", 0) + len(rows)
        f.unlink()
    return {"moved": moved, "total": sum(moved.values())}


def main(args: Any, paths: HarnessPaths, reg: Any) -> int:  # noqa: ANN401
    from core import emit

    action = getattr(args, "action", "presence")
    if action == "presence":
        rows = all_presence(paths)
        if getattr(args, "json", False):
            emit(json.dumps([asdict(p) | {"state": p.state} for p in rows], indent=2))
            return 0
        if not rows:
            emit("no lab has published presence yet")
            emit("  a lab publishes it by running: cli.py heartbeat -m <milestone> --lab <lab>")
            return 0
        emit(f"{len(rows)} lab(s) known (stale after {PRESENCE_TTL_MINUTES}m)")
        emit(f"{'lab':<12} {'state':<22} {'milestone':<8} {'branch':<28} doing")
        emit("-" * 104)
        for p in rows:
            emit(f"{p.lab:<12} {p.state:<22} {p.milestone:<8} {p.branch[:28]:<28} {p.doing[:34]}")
        return 0

    if action == "claims":
        claims = load_claims(paths)
        if getattr(args, "json", False):
            emit(json.dumps(claims, indent=2))
            return 0
        if not claims:
            emit("no branch claims recorded (docs/harness/claims.json)")
            return 0
        emit(f"{len(claims)} branch claim(s)")
        for br, held in sorted(claims.items()):
            flag = " STALE" if Claim(**{k: v for k, v in held.items() if k in Claim.__dataclass_fields__}).stale else ""
            emit(f"  {br:<44} lab={held.get('lab', '?'):<10} since {held.get('claimed_at', '?')}{flag}")
        return 0

    if action == "migrate-ledger":
        result = migrate_ledger(paths)
        if getattr(args, "json", False):
            emit(json.dumps(result, indent=2))
        else:
            emit(f"migrated {result['total']} receipt(s) into per-lab shards")
            for name, n in sorted(result["moved"].items()):
                emit(f"  {name}: {n}")
        return 0

    return 2
