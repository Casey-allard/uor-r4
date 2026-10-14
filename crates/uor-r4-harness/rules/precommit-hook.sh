#!/bin/sh
# >>> uor-harness pre-commit v1 (managed by scripts/harness/githelp.py) >>>
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
# <<< uor-harness pre-commit v1 <<<
