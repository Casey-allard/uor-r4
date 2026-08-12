#!/usr/bin/env bash
# setup_hooks.sh — one-time hook setup for a fresh clone.
#
# Run this once after cloning:
#   bash setup_hooks.sh
#
# What it does:
#   1. Points git at .githooks/ so all three hooks are active:
#        post-checkout      -- prints research context on branch switch
#        prepare-commit-msg -- pre-fills commit messages with [RR-###]
#        pre-push           -- validates canonical research state before push
#   2. Ensures the hooks are executable.
#
# How to disable a hook without removing it:
#   chmod -x .githooks/<hook-name>
#
# How to bypass pre-push when docs lag behind code:
#   git push --no-verify

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Configuring git to use .githooks/ ..."
git -C "$REPO_ROOT" config core.hooksPath .githooks

echo "Ensuring hooks are executable ..."
chmod +x "$REPO_ROOT/.githooks/post-checkout"
chmod +x "$REPO_ROOT/.githooks/prepare-commit-msg"
chmod +x "$REPO_ROOT/.githooks/pre-push"

echo ""
echo "Done. Active hooks:"
echo "  post-checkout       — prints research context when you switch branches"
echo "  prepare-commit-msg  — pre-fills commit messages with [RR-###]"
echo "  pre-push            — validates canonical research state before push"
echo ""
echo "Quick-start:"
echo "  make help       list all make targets"
echo "  make state      validate research state right now"
echo "  make branch     show context for the current branch"
