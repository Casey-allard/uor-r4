#!/bin/bash
# Studio's quick gates, in seconds, before anything is pushed: what CI would refuse first, found here.
#
#   ./scripts/quick.sh                  run them (from studio/)
#   ./scripts/quick.sh --install-hook   run them before every git push from this checkout
#
# Not the acceptance boundary: `just vv` in the locked SDK image is, and CI runs it. These are the gates that
# need only Node and Python, so a parse error, a hand-typed colour, a stale pin or a broken contract is seen
# in seconds rather than after a CI run. A gate whose inputs are not installed here (the renderer's or the
# live frame's node_modules, a renderer build) is named as not run, never passed silently. A change under
# app/ is named too: proving the kernel needs the SDK image, so CI's bootstrap does it.
set -uo pipefail
STUDIO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$STUDIO"

if [[ ${1:-} == --install-hook ]]; then
  hook="$(git rev-parse --git-path hooks)/pre-push"
  printf '#!/bin/bash\n# Installed by studio/scripts/quick.sh --install-hook.\nexec "%s/scripts/quick.sh"\n' "$STUDIO" >"$hook"
  chmod +x "$hook"
  echo "installed $hook"
  exit 0
fi

PY=$(command -v python3 || command -v python)
failed=() notrun=()
start=$(date +%s)
gate() {
  local name=$1; shift
  local log; log=$(mktemp)
  if "$@" >"$log" 2>&1; then printf '  ok    %s\n' "$name"; else printf '  FAIL  %s\n' "$name"; sed 's/^/        /' "$log" | tail -12; failed+=("$name"); fi
  rm -f "$log"
}
needs() { local name=$1 path=$2; shift 2; if [[ -e $path ]]; then gate "$name" "$@"; else printf '  --    %s (not run: %s is not here)\n' "$name" "$path"; notrun+=("$name"); fi; }

echo "studio quick gates"
gate "every shell script parses" node tools/shell_parse.mjs
gate "brand: kit, generated sheets, no typed colour" "$PY" tools/brand_kit.py --check
gate "the model's sources are what author.py writes" "$PY" tools/author.py --check
gate "kernel: the pinned guest replays every vector" node tools/kernel.mjs --check
gate "experience: the stylesheet agrees with the kernel" node tools/experience.mjs --check
gate "journey: prism-code's files, byte for byte" node tools/prism_journey.mjs --check
gate "live preview, table and edit" bash -c 'node tools/live_preview.mjs --check && node tools/live_table.mjs --check && node tools/live_edit.mjs --check'
gate "build look" node tools/build_look.mjs
gate "self-contained" node tools/self_contained.mjs
gate "unit tests (tools/*.test.mjs)" bash -c 'node --test tools/*.test.mjs'
needs "contracts: the host answers the renderer's own" vendor/prism/node_modules node tools/contract_shapes.mjs --check
needs "components: the kit's shadcn/ui, byte for byte" live-src/node_modules node tools/live_components.mjs --check
needs "performance: JS per route within the kernel's budgets" vendor/prism/dist/.vite/manifest.json node tools/perf_budget.mjs

base=$(git merge-base HEAD origin/main 2>/dev/null || true)
if [[ -n $base ]] && ! git diff --quiet "$base" -- ../app; then
  echo "  --    app/ changed: proving the kernel needs the SDK image (just vv); CI's bootstrap proves it"
fi

echo "$(( $(date +%s) - start )) s; ${#failed[@]} failed, ${#notrun[@]} not run here"
[[ ${#failed[@]} -eq 0 ]]
