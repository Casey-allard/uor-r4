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


# --------------------------------------------------------------- merged-branch cleanup
#
# Owner direction, 12 October 2026: merge branches into main through the PR and do not leave
# them behind. GitHub auto-delete is off for this repository, so a merged PR leaves a stale
# local branch with a "[gone]" upstream and, if it had a worktree, the whole checkout behind it.
# This finds those safely: a branch is removable only when its head is contained in
# origin/main AND it is not checked out in any worktree. Other labs' merged branches are
# reported, never removed unless asked for by name.


def _git_raw(args: list[str], cwd: Path | None = None) -> tuple[int, str]:
    """Git with an exit code and combined output. Distinct from the module's `_run`, which
    returns (ok, stdout, err) and is used by the hook installer."""
    try:
        p = subprocess.run(args, cwd=str(cwd) if cwd else None, capture_output=True, text=True, timeout=60, check=False)
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except (OSError, subprocess.SubprocessError) as exc:  # pragma: no cover
        return 127, str(exc)


def merged_branches(root: Path) -> dict[str, list[str]]:
    """Split local branches into removable / checked-out / unmerged.

    Two merge shapes have to be recognised, and only the first is an ancestor test:

    * a merge commit, where the branch head IS an ancestor of origin/main;
    * a **squash merge**, which is what this repository's queue uses - the branch head is
      never an ancestor, so `merge-base --is-ancestor` is false even though every change
      landed. Those are identified by their upstream being `[gone]`, which means the remote
      branch was deleted, plus no commits in origin/main..branch that are not already in main.

    A branch is only ever listed as removable when it is not checked out in any worktree.
    """
    rc, out = _git_raw(["git", "-C", str(root), "for-each-ref", "--format=%(refname:short)", "refs/heads/"])
    branches = [b for b in out.splitlines() if b.strip() and b.strip() != "main"]

    rc, out = _git_raw(["git", "-C", str(root), "worktree", "list", "--porcelain"])
    held: set[str] = set()
    for line in out.splitlines():
        if line.startswith("branch refs/heads/"):
            held.add(line[len("branch refs/heads/") :].strip())
    prunable_worktrees = "prunable" in out

    rc, up = _git_raw(["git", "-C", str(root), "for-each-ref", "--format=%(refname:short)\t%(upstream:track)",
                   "refs/heads/"])
    gone: set[str] = set()
    for line in up.splitlines():
        if "\t" in line:
            name, track = line.split("\t", 1)
            if "gone" in track:
                gone.add(name.strip())

    removable, checked_out, unmerged = [], [], []
    for b in branches:
        if b in held:
            checked_out.append(b)
            continue
        rc, _ = _git_raw(["git", "-C", str(root), "merge-base", "--is-ancestor", b, "origin/main"])
        if rc == 0:
            removable.append(b)          # a real merge: the head is in main
            continue
        if b in gone:
            # A squash-merged branch is not an ancestor of main, so it has to be verified by
            # CONTENT: if a merged PR for this branch exists and that PR's merge commit carries
            # exactly the branch's tree, then everything on the branch is already in main and
            # nothing unique is lost. Without that proof the branch is left alone.
            verdict = _squash_merged_proof(root, b)
            if verdict is True:
                removable.append(b)
            else:
                unmerged.append(b)
            continue
        unmerged.append(b)
    return {
        "removable": sorted(removable),
        "checked_out": sorted(checked_out),
        "unmerged": sorted(unmerged),
        "stale_worktree_refs": prunable_worktrees,
    }


def _squash_merged_proof(root: Path, branch: str) -> bool:
    """True when a merged PR exists for `branch` and its merge commit carries the same tree.

    This is the check that makes deleting a squash-merged branch safe. `git branch -d` cannot
    do it - it only understands ancestry, and a squash merge has none.
    """
    rc, out = _git_raw(["gh", "pr", "list", "--head", branch, "--state", "merged",
                        "--json", "number,mergeCommit", "--limit", "1"], root)
    if rc != 0 or not out.strip():
        return False
    try:
        data = json.loads(out)
    except json.JSONDecodeError:
        return False
    if not data:
        return False
    mc = (data[0].get("mergeCommit") or {}).get("oid")
    if not mc:
        return False
    rc, _ = _git_raw(["git", "-C", str(root), "cat-file", "-e", f"{mc}^{{tree}}"])
    if rc != 0:
        return False
    rc, bt = _git_raw(["git", "-C", str(root), "rev-parse", f"{branch}^{{tree}}"])
    rc2, mt = _git_raw(["git", "-C", str(root), "rev-parse", f"{mc}^{{tree}}"])
    return rc == 0 and rc2 == 0 and bt.strip() == mt.strip()


def cmd_clean_merged(args: argparse.Namespace, paths: HarnessPaths) -> int:
    state = merged_branches(paths.repo)
    if getattr(args, "json", False):
        print(json.dumps(state, indent=2))
        return 0
    print(f"local branches: {len(state['removable'])} merged into origin/main and not checked out, "
          f"{len(state['checked_out'])} in a worktree, {len(state['unmerged'])} unmerged")
    if state["removable"]:
        print()
        print("merged and safe to delete:")
        for b in state["removable"]:
            print(f"  {b}")
    if state["unmerged"]:
        print()
        print("NOT merged - kept (their owners decide):")
        for b in state["unmerged"]:
            print(f"  {b}")
    if not getattr(args, "apply", False):
        if state["removable"]:
            print()
            print(f"delete them with:  cli.py git clean-merged --apply")
        return 0
    removed = 0
    for b in state["removable"]:
        # `git branch -d` understands ancestry only and so refuses every squash-merged branch,
        # which is most of them here. The removal decision was already made above, on proof:
        # an ancestor head, or a merged PR whose merge commit carries this exact tree. -D
        # applies that decision; reflog and origin keep the commits recoverable.
        rc, out = _git_raw(["git", "-C", str(paths.repo), "branch", "-D", b])
        if rc == 0:
            removed += 1
            print(f"  deleted {b}")
        else:
            print(f"  KEPT {b}: {out.strip()}")
    print()
    print(f"deleted {removed} merged branch(es); worktrees were untouched")
    return 0

def delete_merged_branch(paths: HarnessPaths, branch: str, apply: bool = False) -> dict:
    """Delete a branch locally AND on the remote, but only if its merge is PROVEN.

    This exists because of a real mistake. I merged a pull request with `gh pr merge --auto`, polled
    its state, and then ran the remote delete **unconditionally**. The PR had auto-closed as DIRTY
    midway through the poll; the loop exited without noticing, the delete ran anyway, and the only
    remote copy of unmerged work was removed. The work survived in a local commit and was recovered,
    but nothing in the command sequence prevented the loss - and a script that deletes a branch
    without asking whether the merge happened will eventually delete work that cannot be recovered.

    So the proof comes first and the answer can be no:

      * a MERGED pull request whose merge commit is an ancestor of origin/main, or
      * the branch content is entirely in origin/main (the squash-merge case)

    Anything else is refused with the reason, because an unmerged branch on the remote is the last
    copy of that work in most cases.

    `apply=False` reports what would happen and changes nothing.
    """
    import subprocess

    def git(*args: str) -> tuple[int, str]:
        try:
            proc = subprocess.run(
                ["git", "-C", str(paths.repo), *args],
                capture_output=True, text=True, timeout=30, check=False,
            )
            return proc.returncode, (proc.stdout + proc.stderr).strip()
        except (OSError, subprocess.SubprocessError) as exc:
            return 1, f"{type(exc).__name__}: {exc}"

    # `safe_to_delete` is the ONLY flag the caller acts on. An earlier version used `merged` for
    # this and a guard returned early without clearing it, so a REFUSED branch still printed
    # "would delete". One flag with one meaning makes that impossible.
    result: dict = {"branch": branch, "safe_to_delete": False, "merged": False, "reason": "", "deleted": False}

    # 0. A LIVE CLAIM IS CHECKED FIRST, and it does not matter whether the PR merged.
    #
    #    A merged PR is not proof a branch is finished: a lab can merge and keep going, or hold the
    #    branch while a follow-up lands. Gatekeeping on the merge first also meant an unreachable
    #    GitHub returned before this guard ran, so a claimed branch could be deleted whenever the
    #    network was down - the worst arrangement, since the protection failed exactly when the
    #    evidence was weakest. A claim needs no network to read.
    # 1b. somebody may still be WORKING here. A merged PR is not proof the branch is finished: a
    #     lab can merge, then keep going on the same branch, or hold it while a follow-up lands.
    #     This check is here because the first version recommended deleting
    #     `kimi/plan-refresh-2026-10-13` - merged, yes, but under Kimi's live claim, and a harness
    #     that deletes a peer's working branch is worse than one that leaves a stale branch behind.
    try:
        import multilab

        held = multilab.load_claims(paths).get(branch)
    except Exception:  # noqa: BLE001 - an advisory guard must not break the check
        held = None
    if held:
        owner = held.get("lab") or "an unidentified lab"
        # CLEAR the flag, not just the reason. This is the same mistake twice in one function: a
        # guard that explains itself but leaves `safe_to_delete` set still tells the caller to go
        # ahead. The flag is what acts; the reason is only what a human reads.
        result["safe_to_delete"] = False
        result["reason"] = (
            f"{owner} holds a live claim on this branch - merged or not, its owner decides when it "
            f"is finished. Release the claim first if the work is done: "
            f"cli.py team release --branch {branch} --lab {owner}"
        )
        return result

    # 1. a merged PR, and its merge commit actually on main
    try:
        from status import gh_json

        ok, data, err = gh_json(
            ["pr", "list", "--head", branch, "--state", "merged", "--limit", "1",
             "--json", "number,mergeCommit"],
            paths.repo,
        )
        if not ok:
            result["reason"] = f"cannot ask GitHub about the PR ({err or 'unknown error'}), and an unverified merge is not a merge"
            return result
        if data:
            merge_oid = ((data[0].get("mergeCommit") or {}) or {}).get("oid", "")
            num = data[0].get("number")
            if merge_oid:
                rc, _ = git("merge-base", "--is-ancestor", merge_oid, "origin/main")
                if rc == 0:
                    result.update(
                        merged=True,
                        safe_to_delete=True,
                        reason=f"PR #{num} merged as {merge_oid[:9]}, on main",
                    )
                else:
                    result["reason"] = (
                        f"PR #{num} is marked merged ({merge_oid[:9]}) but that commit is NOT "
                        f"on origin/main"
                    )
                    return result
            else:
                result["reason"] = f"PR #{num} is merged but reports no merge commit"
                return result
        else:
            result["reason"] = "no merged PR for this branch"
            return result
    except ImportError:  # pragma: no cover - status ships with the harness
        result["reason"] = "the status module is unavailable, so the merge cannot be proven"
        return result

    # 2. even with a merged PR, the remote ref must actually be gone-or-equal, not different work
    # The branch must hold nothing main lacks. Compared with a DIRECT diff, not ancestry or a
    # merge-base diff, because a squash merge leaves the branch's commits with no ancestor in main:
    # `rev-list --count main..branch` reports 2 and `git diff main...branch` reports 9 lines for a
    # branch whose content is byte-identical to main. Both would refuse forever.
    #
    # `git diff main..branch` is empty exactly when the branch adds nothing, which is the property
    # that makes deleting it safe - and it is the same content proof `clean-merged` already uses.
    rc, out = git("rev-parse", "--verify", "--quiet", f"refs/remotes/origin/{branch}")
    if rc == 0:
        rc2, diff = git("diff", "--stat", f"origin/main..origin/{branch}")
        if rc2 == 0 and diff.strip():
            result["reason"] = (
                f"origin/{branch} holds content main does not ({diff.splitlines()[-1].strip()}) - "
                f"the merge did not carry it"
            )
            result["merged"] = False
            result["safe_to_delete"] = False
            return result

    if not apply:
        return result

    # local first, then remote: a failed remote delete leaves the local copy for the next attempt
    rc, out = git("branch", "-D", branch)
    if rc == 0:
        result["deleted"] = True
    else:
        result["local_error"] = out
    rc, out = git("push", "origin", "--delete", branch)
    result["remote_deleted"] = rc == 0
    if rc != 0 and "remote ref does not exist" not in out:
        result["remote_error"] = out
    return result


def remote_branches_without_pr(paths, limit: int = 20) -> list[dict]:
    """Remote branches that have no PR and are not merged into origin/main.

    `branches_with_merged_pr` answers "did this land?"; this answers the other half, which is what
    a stale branch list is usually made of: work that was pushed and then never opened as a PR. It
    is deliberately advisory - such a branch holds unmerged work, so the answer is archive-or-PR,
    never delete, and only its owner can decide which.
    """
    import json as _json
    import subprocess

    def _run(*args: str) -> str:
        try:
            return subprocess.run(
                ["git", "-C", str(paths.repo), *args],
                capture_output=True, text=True, timeout=30, check=False,
            ).stdout.strip()
        except (OSError, subprocess.SubprocessError):
            return ""

    listing = _run(
        "for-each-ref", "--sort=-committerdate", "--format=%(refname:short)|%(committerdate:short)",
        "refs/remotes/origin/",
    )
    out: list[dict] = []
    for line in listing.splitlines():
        if "|" not in line:
            continue
        ref, updated = line.split("|", 1)
        name = ref[len("origin/"):]
        if name in ("main", "HEAD", "gh-pages") or name.startswith("HEAD"):
            # gh-pages is published by GitHub itself, not delivery transport; flagging it would be
            # noise that trains people to ignore this rule.
            continue
        ahead = _run("rev-list", "--count", f"origin/main..origin/{name}")
        behind = _run("rev-list", "--count", f"origin/{name}..origin/main")
        if not ahead.isdigit() or int(ahead) == 0:
            continue  # nothing ahead of main: fully merged, safe to remove on its own merits
        # gh_json lives in status.py; import it lazily so this module stays importable on its own.
        try:
            from status import gh_json as _gh_json
        except ImportError:  # pragma: no cover - status ships with the harness
            return out
        ok, data, _err = _gh_json(
            ["pr", "list", "--head", name, "--state", "all", "--limit", "1", "--json", "number"],
            paths.repo,
        )
        if ok and data:
            continue  # it has a PR; that path is handled by the merged-PR check
        out.append({"branch": name, "ahead": int(ahead), "behind": int(behind) if behind.isdigit() else 0,
                    "updated": updated})
        if len(out) >= limit:
            break
    return out

def checkout_preserving_claims(paths: HarnessPaths, target: str, create: bool = False) -> dict:
    """Switch branches without losing an uncommitted claim.

    `docs/harness/claims.json` is TRACKED, because a claim has to reach the other labs, and it is
    APPENDED to by every `team claim` and `team release`. So the ordinary sequence — claim a branch,
    then check something out — fails with:

        error: Your local changes to the following files would be overwritten by checkout:
            docs/harness/claims.json

    That happened three times in one session, and each time the workaround was to copy the file
    aside by hand. It is worth a command because **the wrong workaround is the easy one**:
    `git checkout -f` or `git checkout -- docs/harness/claims.json` discards the claim, and a claim
    nobody has committed is still a lab saying "I am working here". Silently dropping it is exactly
    the collision this plane exists to prevent.

    So: read the modified claims, check out, write them back. The claim survives on the new branch,
    still uncommitted, exactly as the caller left it.
    """
    import subprocess

    import multilab

    # `claims_file` lives in multilab, which resolves it to the MAIN checkout so a claim made from a
    # linked worktree still lands in the shared store.
    claims = multilab.claims_file(paths)
    preserved: str | None = None
    if claims.is_file():
        rc, dirty = _git_raw(["git", "-C", str(paths.repo), "status", "--porcelain", "--", str(claims)])
        if rc == 0 and dirty.strip():
            preserved = claims.read_text(encoding="utf-8")

    args = ["git", "-C", str(paths.repo), "checkout"]
    if create:
        args += ["-b", target]
    else:
        args.append(target)
    rc, out = _git_raw(args)
    if rc != 0:
        return {"target": target, "switched": False, "claims_preserved": False, "error": out.strip()}

    restored = False
    if preserved is not None:
        claims.write_text(preserved, encoding="utf-8")
        restored = True
    return {
        "target": target,
        "switched": True,
        "claims_preserved": restored,
        "note": (
            "an uncommitted claim was carried across the checkout"
            if restored
            else "no uncommitted claim to carry"
        ),
    }
