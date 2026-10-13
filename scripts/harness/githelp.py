"""githelp.py - delivery hygiene.

Two things only:

* ``install_hooks`` writes an idempotent, clearly-marked pre-commit hook that runs
  the harness staged check and blocks **only** on ``error`` severity. It refuses to
  overwrite a pre-commit hook the harness did not write.
* ``status`` reports the delivery state of the current branch: branch, upstream PR,
  whether its merge commit is contained in freshly fetched ``origin/main``, the
  worktree, cleanliness, and the exact next delivery command.

Read-only except for ``install_hooks``, which writes exactly one file, under
``$(git rev-parse --git-path hooks)``. Standard library only; subprocess output is
redacted before it is printed.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

from checkrules import Rule  # noqa: F401  (module contract: every module exposes register_rules)
from core import HarnessPaths, emit, redact
from registries import Registry  # noqa: F401  (module contract)
from util import truncate

GH_TIMEOUT = 20
GIT_TIMEOUT = 20
FETCH_TIMEOUT = 30

#: both lines are searched for; a hook without them is never touched
HOOK_START = "# >>> uor-harness pre-commit v1 (managed by scripts/harness/githelp.py) >>>"
HOOK_END = "# <<< uor-harness pre-commit v1 <<<"

HOOK_SCRIPT = f"""#!/bin/sh
{HOOK_START}
# Installed by: python3 scripts/harness/cli.py git install-hooks
# Remove with:  rm "$(git rev-parse --git-path hooks)/pre-commit"
#
# Runs `python3 scripts/harness/cli.py check --staged` and prints what it found.
# It blocks the commit ONLY on error severity; warn and info print and pass.
# It never modifies staged content. Safe to run in a linked worktree.
# It exits 0 (commit allowed) when python3 or scripts/harness/ is not present,
# and when the harness itself cannot produce a readable result.
set -u
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
[ -f "$root/scripts/harness/cli.py" ] || exit 0
command -v python3 >/dev/null 2>&1 || exit 0
state="$root/.harness"
mkdir -p "$state" 2>/dev/null || exit 0
tmp="$state/precommit-$$.json"
trap 'rm -f "$tmp"' EXIT INT TERM
python3 "$root/scripts/harness/cli.py" check --staged --json >"$tmp" 2>/dev/null
python3 - "$tmp" <<'PY'
import json
import sys

try:
    with open(sys.argv[1], "r", encoding="utf-8") as fh:
        data = json.load(fh)
except Exception:
    sys.exit(0)  # no readable result: never block a commit on infrastructure

results = [r for r in (data.get("results") or []) if isinstance(r, dict) and not r.get("ok")]
errors = [r for r in results if r.get("severity") == "error"]
for r in results:
    if r.get("severity") != "error":
        print("harness %s: %s: %s" % (r.get("severity", "warn"), r.get("rule_id", ""), r.get("message", "")))
for r in errors:
    print("harness ERROR: %s: %s" % (r.get("rule_id", ""), r.get("message", "")))
    if r.get("fix"):
        print("      fix: %s" % r["fix"])
if errors:
    print("uor-harness: %d error(s); commit blocked. Bypass (not recommended): git commit --no-verify" % len(errors))
    sys.exit(1)
if results:
    print("uor-harness: %d warning(s); commit allowed." % len(results))
sys.exit(0)
PY
{HOOK_END}
"""


# ------------------------------------------------------------------------ process


def _cmd_text(cmd: list[str]) -> str:
    return " ".join(cmd)


def _run(cmd: list[str], cwd: Path | None, timeout: int, input_text: str | None = None) -> tuple[bool, str, str]:
    """Run a command with a bounded timeout. Returns (ok, stdout, error). Never raises."""
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(cwd) if cwd else None,
            input=input_text,
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


def _git(subargs: list[str], paths: HarnessPaths, timeout: int = GIT_TIMEOUT) -> tuple[bool, str, str]:
    return _run(["git", *subargs], paths.repo, timeout)


def _gh(subargs: list[str], paths: HarnessPaths, timeout: int = GH_TIMEOUT) -> tuple[bool, Any, str]:
    ok, out, err = _run(["gh", *subargs], paths.repo, timeout)
    if not ok:
        return False, None, err
    try:
        return True, json.loads(out or "null"), ""
    except json.JSONDecodeError as exc:
        return False, None, f"`{_cmd_text(['gh', *subargs])}` returned unparseable JSON: {exc}"


# --------------------------------------------------------------------------- hooks


def _hooks_dir(paths: HarnessPaths) -> tuple[Path | None, str]:
    """The directory git actually reads hooks from (the common dir in a worktree)."""
    ok, out, err = _git(["rev-parse", "--git-path", "hooks"], paths)
    if not ok:
        return None, err or "`git rev-parse --git-path hooks` failed"
    raw = out.strip()
    if not raw:
        return None, "`git rev-parse --git-path hooks` returned nothing"
    path = Path(raw)
    if not path.is_absolute():
        path = (paths.repo / path).resolve()
    return path, ""


def install_hooks(args: Any, paths: HarnessPaths) -> int:
    """Install/refresh the marked pre-commit hook. Never clobbers a foreign hook."""
    hooks_dir, err = _hooks_dir(paths)
    if hooks_dir is None:
        emit(f"cannot locate the hooks directory: {err}", sys.stderr)
        return 1
    hook = hooks_dir / "pre-commit"
    existed = hook.exists() or hook.is_symlink()
    previous = ""
    if hook.is_symlink():
        emit(f"CONFLICT: {hook} is a symlink; refusing to replace it.", sys.stderr)
        emit(f"  resolve it yourself, or remove it and re-run: python3 scripts/harness/cli.py git install-hooks", sys.stderr)
        return 2
    if existed:
        try:
            previous = hook.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            emit(f"CONFLICT: cannot read the existing pre-commit hook {hook}: {type(exc).__name__}: {exc}", sys.stderr)
            return 2
        if HOOK_START not in previous:
            first = previous.splitlines()[0] if previous.splitlines() else "(empty file)"
            emit(f"CONFLICT: {hook} already exists and was not written by the harness (no marker line).", sys.stderr)
            emit(f"  marker looked for: {HOOK_START}", sys.stderr)
            emit(f"  existing file starts: {truncate(first, 120)}", sys.stderr)
            emit(f"  merge it by hand, or remove it and re-run: python3 scripts/harness/cli.py git install-hooks", sys.stderr)
            return 2
    try:
        hooks_dir.mkdir(parents=True, exist_ok=True)
        hook.write_text(HOOK_SCRIPT, encoding="utf-8")
        os.chmod(hook, 0o755)
    except OSError as exc:
        emit(f"cannot write {hook}: {type(exc).__name__}: {exc}", sys.stderr)
        return 1
    action = "updated" if existed and previous != HOOK_SCRIPT else ("unchanged" if existed else "installed")
    emit(f"{action}: {hook}")
    emit("  runs: python3 scripts/harness/cli.py check --staged")
    emit("  blocks: error severity only (warn/info print and pass); exits 0 without python3 or the harness")
    if existed and action == "unchanged":
        emit("  already up to date; nothing was appended")
    return 0


# ---------------------------------------------------------------------- git status


def _worktree_facts(paths: HarnessPaths) -> dict[str, Any]:
    ok, out, err = _git(["status", "--porcelain"], paths)
    if not ok:
        return {"available": False, "reason": err or "`git status --porcelain` failed", "clean": None, "lines": [], "counts": {}}
    lines = [l for l in out.splitlines() if l.strip()]
    staged = len([l for l in lines if l[:1] not in (" ", "?", "!")])
    unstaged = len([l for l in lines if len(l) > 1 and l[1:2] not in (" ", "?")])
    untracked = len([l for l in lines if l.startswith("??")])
    return {
        "available": True,
        "reason": "",
        "clean": not lines,
        "lines": [truncate(l, 120) for l in lines[:20]],
        "counts": {"total": len(lines), "staged": staged, "unstaged": unstaged, "untracked": untracked},
    }


def _pr_facts(paths: HarnessPaths, branch: str, fresh_main: str | None) -> dict[str, Any]:
    fields = "number,title,state,url,mergeCommit,headRefName,headRefOid,baseRefName,isDraft"
    ok, data, err = _gh(["pr", "view", "--json", fields], paths)
    if not ok or not isinstance(data, dict):
        return {
            "available": False,
            "reason": err or "`gh pr view` returned no pull request",
            "number": None,
            "state": "",
            "contained_in_origin_main": None,
            "checked_ref": "",
        }
    merge_commit = data.get("mergeCommit") or {}
    sha = str(merge_commit.get("oid", "")) if isinstance(merge_commit, dict) else ""
    checked_ref, checked_sha = "", ""
    if sha:
        checked_ref, checked_sha = "merge commit", sha
    elif data.get("headRefOid"):
        checked_ref, checked_sha = "PR head", str(data.get("headRefOid"))
    contained: bool | None = None
    if checked_sha and fresh_main:
        anc_ok, _, _ = _git(["merge-base", "--is-ancestor", checked_sha, "origin/main"], paths)
        contained = bool(anc_ok)
    return {
        "available": True,
        "reason": "",
        "number": data.get("number"),
        "title": str(data.get("title", "")),
        "state": str(data.get("state", "")),
        "is_draft": bool(data.get("isDraft", False)),
        "url": str(data.get("url", "")),
        "branch": str(data.get("headRefName", branch)),
        "base": str(data.get("baseRefName", "")),
        "merge_commit": sha,
        "checked_ref": checked_ref,
        "checked_sha": checked_sha,
        "contained_in_origin_main": contained,
    }


def _delivery_state(branch: str, facts: dict[str, Any], pr: dict[str, Any]) -> str:
    if facts.get("available") and facts.get("clean") is False:
        return "dirty-worktree"
    if not pr.get("available"):
        return "no-pr"
    state = str(pr.get("state", "")).upper()
    if state == "MERGED":
        if pr.get("contained_in_origin_main") is True:
            return "merged-verified"
        if pr.get("contained_in_origin_main") is False:
            return "merged-unverified"
        return "merged-unknown"
    if state == "CLOSED":
        return "pr-closed"
    if state in ("OPEN",):
        return "pr-open"
    return "unknown"


def _next_command(paths: HarnessPaths, branch: str, state: str, pr: dict[str, Any], worktree: str) -> tuple[str, list[str]]:
    """The exact next delivery command, plus the rest of the chain."""
    number = pr.get("number")
    if state == "dirty-worktree":
        return (
            'git add <the paths you changed> && git commit -m "<message>"',
            ["AGENTS.md: stage named paths, not `git add -A`.", "then follow the state below."],
        )
    if state == "no-pr":
        push = f"git push -u origin {branch}"
        create = "gh pr create --fill --base main"
        return push, [create, "then wait for the required check and merge through the queue."]
    if state == "pr-open":
        checks = f"gh pr checks {number} --watch" if number else "gh pr checks --watch"
        merge = f"gh pr merge {number} --squash --auto" if number else "gh pr merge --squash --auto"
        return checks, [merge, "then verify the merge commit on fresh origin/main."]
    if state == "pr-closed":
        return (
            f"gh pr view {number} --json state,closedAt,url",
            [
                "the PR was closed without merging: reopen it, or preserve the source as a patch + INDEX row in docs/history/branch-archive/.",
                "do not delete the branch until that is decided.",
            ],
        )
    if state in ("merged-verified", "merged-unknown"):
        branch_delete = f"git branch -D {branch}" if branch else "git branch -D <branch>"
        remote_delete = f"git push origin --delete {branch}" if branch else "git push origin --delete <branch>"
        return (
            f"git worktree remove {worktree} && {branch_delete} && {remote_delete} && scripts/storage/uor-hygiene --apply",
            ["run it from another checkout (this command removes the worktree it is run in)."],
        )
    if state == "merged-unverified":
        return (
            "git fetch --prune origin && git log -1 --format='%H %s' origin/main",
            [
                "the PR is merged but its merge commit is NOT contained in the fetched origin/main.",
                "keep the branch and worktree until that is explained; do not delete anything yet.",
            ],
        )
    return ("git status", ["state not recognised; report this output rather than acting on it."])


def status(args: Any, paths: HarnessPaths) -> int:
    """Delivery state of the current branch, with the exact next command."""
    ok_branch, branch_out, branch_err = _git(["rev-parse", "--abbrev-ref", "HEAD"], paths)
    branch = branch_out.strip() if ok_branch else ""
    ok_head, head_out, _ = _git(["rev-parse", "--short", "HEAD"], paths)
    head = head_out.strip() if ok_head else ""
    ok_wt, wt_out, _ = _git(["rev-parse", "--show-toplevel"], paths)
    worktree = wt_out.strip() if ok_wt else str(paths.repo)

    facts = _worktree_facts(paths)

    fetch_ok, _, fetch_err = _git(["fetch", "--prune", "origin"], paths, timeout=FETCH_TIMEOUT)
    ok_main, main_out, main_err = _git(["rev-parse", "origin/main"], paths)
    fresh_main = main_out.strip() if ok_main else None
    subject = ""
    if fresh_main:
        ok_sub, sub_out, _ = _git(["log", "-1", "--format=%h %s", "origin/main"], paths)
        subject = truncate(sub_out.strip() if ok_sub else "", 100)

    upstream = ""
    ok_up, up_out, _ = _git(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"], paths)
    if ok_up:
        upstream = up_out.strip()
    ahead = behind = None
    if fresh_main:
        ok_lr, lr_out, _ = _git(["rev-list", "--left-right", "--count", "origin/main...HEAD"], paths)
        if ok_lr and lr_out.split():
            parts = lr_out.split()
            if len(parts) == 2:
                behind, ahead = int(parts[0]), int(parts[1])

    pr = _pr_facts(paths, branch, fresh_main)
    state = _delivery_state(branch, facts, pr)
    next_command, next_steps = _next_command(paths, branch, state, pr, worktree)

    data: dict[str, Any] = {
        "branch": branch,
        "head": head,
        "worktree": worktree,
        "clean": facts.get("clean"),
        "changes": facts.get("counts", {}),
        "changes_available": facts.get("available", False),
        "changes_reason": facts.get("reason", ""),
        "fetch": {"command": "git fetch --prune origin", "ok": fetch_ok, "reason": fetch_err},
        "origin_main": {
            "available": bool(fresh_main),
            "reason": main_err if not fresh_main else "",
            "commit": fresh_main or "",
            "subject": subject,
        },
        "upstream": upstream,
        "ahead": ahead,
        "behind": behind,
        "pr": pr,
        "state": state,
        "next_command": next_command,
        "next_steps": next_steps,
        "notes": [],
    }
    if not fetch_ok:
        data["notes"].append("origin/main may be stale: the fetch failed, so containment uses the last fetched ref")

    if getattr(args, "json", False):
        emit(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        lines = [f"uor-harness git status - branch {branch or '?'} @ {head or '?'}"]
        if facts.get("available"):
            counts = facts.get("counts", {})
            cleanliness = "clean" if facts.get("clean") else f"{counts.get('total', 0)} uncommitted change(s) ({counts.get('staged', 0)} staged, {counts.get('unstaged', 0)} unstaged, {counts.get('untracked', 0)} untracked)"
            lines.append(f"1. worktree {worktree} - {cleanliness}")
        else:
            lines.append(f"1. worktree {worktree} - UNAVAILABLE: {facts.get('reason', '')}")
        if fresh_main:
            fetched = "just fetched" if fetch_ok else f"last fetch FAILED: {fetch_err}"
            lines.append(f"2. origin/main {fresh_main[:8]} ({subject}) - {fetched}")
        else:
            lines.append(f"2. origin/main - UNAVAILABLE: {main_err or 'cannot resolve origin/main'}")
        if pr.get("available"):
            draft = " (draft)" if pr.get("is_draft") else ""
            contained = {True: "contained in origin/main", False: "NOT contained in origin/main", None: "containment unknown"}[pr.get("contained_in_origin_main")]
            lines.append(f"3. PR #{pr.get('number')} {pr.get('state')}{draft}: {truncate(str(pr.get('title', '')), 80)}")
            lines.append(f"   {pr.get('checked_ref') or 'no merge ref'} {str(pr.get('checked_sha', ''))[:8]} - {contained}")
        else:
            lines.append(f"3. PR - none for this branch ({pr.get('reason', 'gh unavailable')})")
        if upstream:
            lines.append(f"4. upstream {upstream} - ahead {ahead} / behind {behind} of origin/main")
        else:
            lines.append(f"4. upstream - none (branch is not pushed)")
        lines.append(f"5. state: {state}")
        lines.append(f"6. next: {next_command}")
        for step in next_steps:
            lines.append(f"   then: {step}")
        for note in data["notes"]:
            lines.append(f"   note: {note}")
        emit("\n".join(lines))

    return 1 if state == "merged-unverified" else 0


def main(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    """Contract entry point; `cli.py` calls install_hooks/status directly."""
    action = getattr(args, "action", None)
    if action == "install-hooks":
        return install_hooks(args, paths)
    if action == "status":
        return status(args, paths)
    emit("githelp: usage: git install-hooks | git status [--json]", sys.stderr)
    return 2


def register_rules() -> list[Rule]:
    """No rules: this module is the hook and the delivery report, not a guard."""
    return []
