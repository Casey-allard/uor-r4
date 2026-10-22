#!/bin/sh
# >>> uor-harness commit-msg v1 (managed by crates/uor-r4-harness) >>>
# Installed by: uor-harness git install-hooks --install   (owner only)
# Remove with:  rm "$(git rev-parse --git-path hooks)/commit-msg"
#
# On a <registered-lab>/... branch, with UOR_LAB set, checks the message for
# the loop's Lab: and Order: trailers. Merge, Revert, fixup! and squash!
# messages pass. It exits 0 (commit allowed) when uor-harness is not on PATH,
# and when the harness itself cannot produce a result (exit other than 0 or 1).
set -u
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
command -v uor-harness >/dev/null 2>&1 || exit 0
out=$(uor-harness git check-message "$1" --repo "$root" 2>&1)
rc=$?
[ -n "$out" ] && printf '%s\n' "$out"
if [ "$rc" -eq 1 ]; then
    echo "uor-harness: commit refused; fix the message (loop work is not bypassed)."
    exit 1
fi
exit 0
# <<< uor-harness commit-msg v1 <<<
