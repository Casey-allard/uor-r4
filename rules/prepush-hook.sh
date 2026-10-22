#!/bin/sh
# >>> uor-harness pre-push v1 (managed by crates/uor-r4-harness) >>>
# Installed by: uor-harness git install-hooks --install   (owner only)
# Remove with:  rm "$(git rev-parse --git-path hooks)/pre-push"
#
# Runs Git LFS's own pre-push step first, on the same stdin, then
# uor-harness git push-check on the same refs. The harness step runs
# loop pr-check --local for every pushed <lab>/... branch and skips others.
# It exits 0 when uor-harness is not on PATH, and when the harness itself
# cannot produce a result (exit other than 0 or 1).
set -u
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
tmp=$(mktemp "${TMPDIR:-/tmp}/uor-prepush.XXXXXX") || tmp=
trap '[ -n "$tmp" ] && rm -f "$tmp"' EXIT
if [ -n "$tmp" ]; then cat > "$tmp"; fi
if ! command -v git-lfs >/dev/null 2>&1; then
    printf >&2 '\n%s\n\n' "This repository is configured for Git LFS but 'git-lfs' was not found on your path. The loop pre-push hook chains to Git LFS; install it, or remove the hook deliberately."
    exit 2
fi
if [ -n "$tmp" ]; then
    git lfs pre-push "$@" < "$tmp" || exit $?
else
    git lfs pre-push "$@"
fi
command -v uor-harness >/dev/null 2>&1 || exit 0
if [ -n "$tmp" ]; then
    uor-harness git push-check --repo "$root" < "$tmp"
    rc=$?
else
    uor-harness git push-check --repo "$root"
    rc=$?
fi
if [ "$rc" -eq 1 ]; then
    echo "uor-harness: push blocked; fix the policy failure."
    exit 1
fi
exit 0
# <<< uor-harness pre-push v1 <<<
