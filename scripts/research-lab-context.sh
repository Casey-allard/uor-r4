#!/usr/bin/env bash
# Read-only receipt. No fetch, build, model execution, credentials or mutations.
set -euo pipefail

usage() {
  printf '%s\n' 'Usage: bash scripts/research-lab-context.sh [--github]'
}
github=false
if [[ $# -gt 1 ]]; then usage >&2; exit 2; fi
if [[ $# -eq 1 ]]; then
  case "$1" in
    --github) github=true ;;
    --help|-h) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
fi
root=$(git rev-parse --show-toplevel)
cd "$root"
printf '# UOR-R4 fourth-lab context receipt\n\n'
date -u '+Observed UTC: %Y-%m-%dT%H:%M:%SZ'
printf 'Worktree: %s\n' "$root"
printf 'HEAD: '; git rev-parse HEAD
printf 'Local origin/main cache: '
git rev-parse --verify origin/main || printf 'UNAVAILABLE\n'
printf '\nRefresh origin/main separately before a consequential decision.\n'
printf '\n## Worktree changes\n'
git status --short --branch
printf '\n## Authority and operating-file SHA256\n'
for path in AGENTS.md README.md docs/integration/project-track.md \
  docs/integration/current-state.md docs/integration/model-direction-2026-09.md \
  docs/PROJECT_MAP.md docs/integration/agent-execution-policy.json \
  .codex-lab/README.md; do
  if [[ -f "$path" ]]; then shasum -a 256 "$path";
  else printf 'UNAVAILABLE %s\n' "$path"; fi
done
printf '\n## Shared checkouts; activity/ownership requires verification\n'
git worktree list --porcelain
printf '\n## Physical storage; directory sums are not free space\n'
df -k "$root"
printf '\n## Build/model process names; commands intentionally excluded\n'
if command -v rg >/dev/null 2>&1; then
  ps -axo pid,ppid,etime,rss,comm | rg 'cargo|rustc|uor-r4-training|r4-integer|uor-chat' || true
else
  printf 'UNAVAILABLE: rg not installed; inspect active jobs before work.\n'
fi
if "$github"; then
  printf '\n## Live GitHub coordination\n'
  if ! command -v gh >/dev/null 2>&1; then
    printf 'UNAVAILABLE: gh not installed\n'; exit 1
  fi
  gh pr list --state open --limit 20 --json number,title,headRefName,headRefOid,updatedAt
  for issue in 820 973; do
    gh issue view "$issue" --json number,title,state,updatedAt,comments \
      --jq '{number,title,state,updatedAt,recentComments:(.comments[-3:] | map({url,createdAt,body}))}'
  done
else
  printf '\nGitHub NOT_QUERIED; use --github for live ownership/results.\n'
fi
printf '\nRead source and evidence before deciding. This receipt is navigation, not model qualification.\n'
