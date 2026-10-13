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

#: The live tier. All four labs run on one machine here, so they can share a real-time
#: directory that git never sees - no commit lag, no merge conflicts, and a peer's state is
#: visible within the second it is written. The durable tier (docs/harness/) remains the
#: record, refreshed from here. Live is preferred when present; durable is the fallback, and
#: the reader always says which one it used.
LIVE_ENV = "UOR_HARNESS_LIVE"
LIVE_DEFAULT = ("~/.local/share/uor-r4/harness")
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


def main_worktree(paths: HarnessPaths) -> Path:
    """The main checkout's root, even when called from a linked worktree.

    Claims must land in ONE shared, committable store. A claim written into a worktree's own
    `docs/harness/claims.json` is invisible to every other lab, and the worktree -- and with it
    the claim -- is deleted by routine cleanup. That is measured, not theoretical: two Kimi
    claims from 12 October left receipts in `kimi-2026-10.jsonl` while the shared store stayed
    empty, because the claim lived and died with a worktree.

    `git worktree list` reports the main worktree first. On any failure, fall back to the
    caller's own root, which is correct when that already is the main checkout.
    """
    rc, out = run(["git", "-C", str(paths.repo), "worktree", "list", "--porcelain"], timeout=20)
    if rc == 0:
        for line in out.splitlines():
            if line.startswith("worktree "):
                candidate = Path(line[len("worktree ") :].strip())
                if candidate.is_dir():
                    return candidate
    return paths.repo


def claims_file(paths: HarnessPaths) -> Path:
    """The committed claims store, always in the main checkout so every lab can see it."""
    return main_worktree(paths) / "docs" / "harness" / "claims.json"


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


# --------------------------------------------------------------------------- live tier


def live_dir(paths: HarnessPaths) -> Path:
    """Where live cross-lab state lives. Shared by every lab on this machine."""
    import os

    raw = os.environ.get(LIVE_ENV) or LIVE_DEFAULT
    return Path(raw).expanduser()


def live_presence_path(paths: HarnessPaths, lab: str) -> Path:
    return live_dir(paths) / "presence" / f"{slug(lab)}.json"


def publish_live(paths: HarnessPaths, rec: Presence) -> Path:
    """Write one lab's live presence atomically (rename, so a reader never sees a half file)."""
    target = live_presence_path(paths, rec.lab)
    target.parent.mkdir(parents=True, exist_ok=True)
    tmp = target.with_suffix(".tmp")
    tmp.write_text(json.dumps(asdict(rec), ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")
    tmp.replace(target)
    return target


def read_live(paths: HarnessPaths, lab: str) -> Presence | None:
    data = _load(live_presence_path(paths, lab))
    return _to_presence(data) if data else None


def all_live(paths: HarnessPaths) -> list[Presence]:
    d = live_dir(paths) / "presence"
    if not d.is_dir():
        return []
    out = []
    for f in sorted(d.glob("*.json")):
        data = _load(f)
        if data:
            out.append(_to_presence(data))
    return out


def live_claims_path(paths: HarnessPaths) -> Path:
    return live_dir(paths) / "claims.json"


def sync_live_to_durable(paths: HarnessPaths) -> dict[str, Any]:
    """Copy live presence into the committed tier, so it survives a restart and reaches git.

    This is the bridge between the two tiers. It is deliberately explicit rather than
    automatic: the live tier is written on every heartbeat (cheap, uncommitted), and the
    durable tier is refreshed when a lab wants the record to show it.
    """
    written = []
    for p in all_live(paths):
        _save(presence_path(paths, p.lab), asdict(p))
        written.append(p.lab)
    return {"synced": written, "live_dir": str(live_dir(paths))}


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
    #: which tier answered this read: "live" or "durable". Reporting only, never saved.
    source: str = ""

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
    # Durable tier is the record; the live tier is what peers see immediately. Both are
    # written from the same record, so they cannot disagree about what a lab is doing.
    _save(presence_path(paths, lab), asdict(rec))
    try:
        publish_live(paths, rec)
    except OSError:  # pragma: no cover - a read-only or absent share must not break a check-in
        pass
    return rec


def touch_presence(paths: HarnessPaths, lab: str, *, session: str = "") -> Presence:
    """Refresh a lab's timestamp without changing what it is doing.

    This is what a per-prompt hook needs: a cheap "still here" that does not overwrite the
    intent published by the last `heartbeat`. `write_presence` with no `doing` would inherit
    the existing text anyway, so the difference is honesty about intent - a touch says only
    "this session is active", not "this is what it is working on".

    Never raises for a missing live directory: a hook running on every prompt must not break
    the prompt.
    """
    if not lab:
        raise ValueError("a lab name is required")
    current = read_presence(paths, lab)
    rec = Presence(
        lab=slug(lab),
        updated=now_iso(),
        milestone=current.milestone if current else "",
        branch=_current_branch(paths),
        worktree=str(paths.repo),
        doing=current.doing if current else "",
        next=current.next if current else "",
        host=_hostname(),
        session=session or (current.session if current else ""),
        source="live",
    )
    try:
        publish_live(paths, rec)
    except OSError:
        pass
    # the durable tier is deliberately NOT written here: a per-prompt touch would churn a
    # committed file on every keystroke. `heartbeat` owns the record; `sync-live` bridges.
    return rec


def read_presence(paths: HarnessPaths, lab: str) -> Presence | None:
    """Live tier first (fresher, uncommitted), durable second. `source` says which answered."""
    live = read_live(paths, lab)
    if live is not None:
        live.source = "live"
        return live
    data = _load(presence_path(paths, lab))
    if not data:
        return None
    durable = _to_presence(data)
    durable.source = "durable"
    return durable


def all_presence(paths: HarnessPaths) -> list[Presence]:
    """Every lab from both tiers, newest record wins, stale last."""
    merged: dict[str, Presence] = {}
    for p in all_live(paths):
        p.source = "live"
        merged[p.lab] = p
    if presence_dir(paths).is_dir():
        for f in sorted(presence_dir(paths).glob("*.json")):
            data = _load(f)
            if not data:
                continue
            p = _to_presence(data)
            p.source = "durable"
            prior = merged.get(p.lab)
            if prior is None:
                merged[p.lab] = p
                continue
            # Live wins ties: it is the fresher channel by construction, and only the durable
            # copy can be older than the file it mirrors. Replacing on equal age would label a
            # live record "durable", which is what this comment is here to prevent.
            prior_age, durable_age = prior.age_minutes or 1e9, p.age_minutes or 1e9
            if durable_age < prior_age - 1e-6:
                merged[p.lab] = p
    return sorted(merged.values(), key=lambda p: (p.stale, p.lab))


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
                id="multilab.claim_not_merged",
                description="no claim is held on a branch whose work has already merged",
                severity="warn",
                check=_check_claim_not_merged,
                rationale=(
                    "a claim is a statement that work is in progress. One that outlives its merge "
                    "says another lab is still working on something that is already in main, which "
                    "is worse than no claim at all - it deters work that is safe to start. "
                    "Proposed by the PM in docs/harness/pm-rules.md R1"
                ),
            )
        ),
        register_rule(
            Rule(
                id="multilab.claim_stale",
                description="no claim is held for more than a day without a heartbeat",
                severity="warn",
                check=_check_claim_stale,
                rationale=(
                    "a claim nobody is working under blocks the branch for everyone else. The PM "
                    "proposed a day as the point where holding it stops being information and "
                    "starts being a lock (docs/harness/pm-rules.md R1/R7). Release it, or check in "
                    "so the claim is visibly live"
                ),
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


#: How long a claim may be held with no heartbeat before it reads as a lock rather than a signal.
#: Proposed by the PM (pm-rules.md R1): a day.
CLAIM_ABANDONED_HOURS = 24


def _branch_is_merged(paths: HarnessPaths, branch: str) -> bool | None:
    """Whether `branch` has a MERGED pull request. `None` when it cannot be established.

    Evidence, not inference. The first version treated "not on origin" as merged, which is wrong:
    a branch that was never pushed looks identical, so a lab holding a claim on unpushed work would
    have been told to release it. The check must never say "merged" without a merge to point at.

    Three signals, strongest first:

    1. **A merged PR for this branch.** The direct evidence, and the only one that can prove a
       squash merge (which leaves no ancestry link for `merge-base --is-ancestor` to find).
    2. **No commits ahead of origin/main**, when the branch still exists locally or on the remote.
       Content proof for the case where a branch was merged and kept.
    3. Otherwise `None` — "not established" — and the rule stays quiet rather than guessing.
    """
    import subprocess

    def git(*args: str) -> str:
        try:
            return subprocess.run(
                ["git", "-C", str(paths.repo), *args],
                capture_output=True, text=True, timeout=20, check=False,
            ).stdout.strip()
        except (OSError, subprocess.SubprocessError):
            return ""

    # 1. A merged PR is the direct evidence, and the only thing that proves a squash merge. Asked
    #    FIRST, because a stale local ref makes the content comparison below lie in the other
    #    direction: a lab that has just claimed a branch and pushed it may not have it in the local
    #    remote-tracking refs yet, so the branch looks "gone", and asking GitHub for a PR on a
    #    branch pushed seconds ago can come back empty before the push is fully visible.
    #
    #    That race produced a false positive on live work: `kimi/plan-refresh-2026-10-13` was
    #    claimed and flagged as merged within the same couple of minutes, then the warning vanished
    #    once the ref arrived. A rule that cries wolf on a lab's live branch is worse than no rule.
    try:
        from status import gh_json
    except ImportError:  # pragma: no cover - status ships with the harness
        pass
    else:
        ok, data, _err = gh_json(
            ["pr", "list", "--head", branch, "--state", "merged", "--limit", "1", "--json", "number,state"],
            paths.repo,
        )
        if ok:
            if data:
                return True
            # No merged PR. If the branch is visible, it is unmerged work; if it is not visible
            # either, the honest answer is "not established" rather than "merged".
            for candidate in (f"refs/remotes/origin/{branch}", f"refs/heads/{branch}"):
                if git("rev-parse", "--verify", "--quiet", candidate):
                    return False
            return None

    # 2. GitHub unreachable: fall back to content, which is safe in the direction that matters.
    #    Nothing ahead of main means the work is in main, whatever route put it there.
    for candidate in (f"refs/remotes/origin/{branch}", f"refs/heads/{branch}"):
        if git("rev-parse", "--verify", "--quiet", candidate):
            ahead = git("rev-list", "--count", f"origin/main..{candidate}")
            return int(ahead) == 0 if ahead.isdigit() else None
    return None


def _check_claim_not_merged(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    out: list[RuleResult] = []
    for branch, held in load_claims(paths).items():
        merged = _branch_is_merged(paths, branch)
        if merged is not True:
            continue
        out.append(
            RuleResult(
                rule_id="multilab.claim_not_merged",
                severity="warn",
                ok=False,
                message=(
                    f"{held.get('lab') or 'nobody'} still holds a claim on {branch!r}, but its work "
                    f"is already in origin/main"
                ),
                path="docs/harness/claims.json",
                fix=(
                    f"release it: python3 scripts/harness/cli.py team release --branch {branch} "
                    f"--lab {held.get('lab') or '<lab>'}"
                ),
                meta={"branch": branch, "lab": held.get("lab", "")},
            )
        )
    return out


def _check_claim_stale(paths: HarnessPaths, reg: Any) -> list[RuleResult]:  # noqa: ANN401
    """A claim held past the abandonment window with no heartbeat from its lab.

    The heartbeat is what distinguishes "holding the branch while working" from "holding it and
    gone". A lab that checks in keeps its claims; one that does not shows up here.
    """
    from datetime import datetime, timedelta, timezone

    now = datetime.now(timezone.utc)
    presence = {p.lab: p for p in all_presence(paths)}
    out: list[RuleResult] = []
    for branch, held in load_claims(paths).items():
        lab = held.get("lab") or ""
        claimed_at = _parse_claimed_at(held.get("claimed_at", ""))
        if claimed_at is None:
            continue  # an unreadable timestamp is reported by claims_one_owner's sibling checks
        age = now - claimed_at
        if age < timedelta(hours=CLAIM_ABANDONED_HOURS):
            continue
        p = presence.get(lab)
        fresh = False
        if p is not None and p.age_minutes is not None:
            fresh = p.age_minutes <= PRESENCE_TTL_MINUTES
        if fresh:
            continue  # the lab is visibly working; the claim is information, not a lock
        hours = int(age.total_seconds() // 3600)
        out.append(
            RuleResult(
                rule_id="multilab.claim_stale",
                severity="warn",
                ok=False,
                message=(
                    f"{lab or 'nobody'} has held {branch!r} for {hours}h with no live heartbeat"
                    + ("" if p is not None else " and has published no presence at all")
                ),
                path="docs/harness/claims.json",
                fix=(
                    "release it if you are done, or run `cli.py heartbeat -m <milestone>` so the "
                    "claim is visibly live; the PM resolves a contested claim (pm-rules.md R7)"
                ),
                meta={"branch": branch, "lab": lab, "hours": hours},
            )
        )
    return out


def _parse_claimed_at(text: str):  # noqa: ANN202 - datetime | None
    """Parse a claim timestamp in either offset form.

    `datetime.fromisoformat` did not accept the `Z` suffix until Python 3.11, and this harness
    supports 3.9 (verified on 3.9.6). That matters more than a compatibility nicety: the Rust
    implementation writes `Z` timestamps, so without this a claim written in Rust is unreadable by
    the Python harness on 3.9 — the two implementations would disagree about a claim's age, and the
    staleness rule would silently stop working. Found by running the suite on 3.9 after it passed on
    3.15.
    """
    from datetime import datetime, timezone

    if not text:
        return None
    normalised = text.strip()
    if normalised.endswith(("Z", "z")):
        normalised = normalised[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(normalised)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed


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
            emit("  an empty board is normal until someone checks in.")
            emit("  publish yours with:  export UOR_LAB=<lab> && cli.py heartbeat -m <milestone>")
            return 0
        emit(f"{len(rows)} lab(s) known (stale after {PRESENCE_TTL_MINUTES}m) · live tier: {live_dir(paths)}")
        emit(f"{'lab':<12} {'state':<20} {'via':<8} {'milestone':<8} {'branch':<24} doing")
        emit("-" * 108)
        for p in rows:
            emit(f"{p.lab:<12} {p.state:<20} {p.source:<8} {p.milestone:<8} {p.branch[:24]:<24} {p.doing[:28]}")
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

    if action == "touch":
        lab = getattr(args, "lab", "") or ""
        if not lab:
            emit("multilab touch: --lab or $UOR_LAB is required", file=__import__("sys").stderr)
            return 2
        rec = touch_presence(paths, lab, session=getattr(args, "session", "") or "")
        if getattr(args, "json", False):
            emit(json.dumps(asdict(rec), indent=2))
        else:
            emit(f"{rec.lab}: live, updated {rec.updated}" + (f" (doing: {rec.doing})" if rec.doing else ""))
        return 0

    if action == "sync-live":
        result = sync_live_to_durable(paths)
        if getattr(args, "json", False):
            emit(json.dumps(result, indent=2))
        else:
            if result["synced"]:
                emit(f"copied live presence for {', '.join(result['synced'])} into the committed tier")
            else:
                emit(f"no live presence to sync (looked in {result['live_dir']})")
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
