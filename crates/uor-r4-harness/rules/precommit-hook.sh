#!/bin/sh
# >>> uor-harness pre-commit v2 (managed by crates/uor-r4-harness) >>>
# Installed by: uor-harness git install-hooks
# Remove with:  rm "$(git rev-parse --git-path hooks)/pre-commit"
#
# Runs `uor-harness check --staged` and prints what it found.
# It blocks the commit ONLY on error severity (exit 1); warn and info print and pass.
# It never modifies staged content. Safe to run in a linked worktree.
# It exits 0 (commit allowed) when uor-harness is not on PATH, and when the
# harness itself cannot produce a result (any exit other than 0 or 1).
set -u
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
command -v uor-harness >/dev/null 2>&1 || exit 0
out=$(uor-harness check --staged --repo "$root" 2>&1)
rc=$?
# Quiet only when the check exited 0 and its summary (the last line) counts no failure.
last=$(printf '%s\n' "$out" | tail -n 1)
case "$rc:$last" in
    "0:"*" rules, 0 failed ("*) ;;
    *) printf '%s\n' "$out" ;;
esac
if [ "$rc" -eq 1 ]; then
    echo "uor-harness: error(s) found; commit blocked. Bypass (not recommended): git commit --no-verify"
    exit 1
fi
exit 0
# <<< uor-harness pre-commit v2 <<<
