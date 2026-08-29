#!/usr/bin/env bash
set -euo pipefail

experiment_repo="$(git rev-parse --show-toplevel)"
cd "${experiment_repo}"

experiment_revision="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain=v1 --untracked-files=no)" ]]; then
  echo "refusing to run: tracked checkout is not clean at ${experiment_revision}" >&2
  exit 1
fi

experiment_manifest="${experiment_repo}/docs/intrinsic_lorentz_r4_attention_partition_973.json"
experiment_output="/Users/casey.allard/uor-r4/.uor-models/research/issue-973-intrinsic-lorentz-r4/cad3dfd17159fdacc5c40e38753109c11764117e3c960f42b9b198d5731272a1/result.attempt-02-checkpoint-float-roundtrip.json"
experiment_events="$(mktemp -t uor-r4-973-build.XXXXXX)"
trap 'rm -f "${experiment_events}"' EXIT

UOR_R4_973_INTRINSIC_IMPLEMENTATION_REVISION="${experiment_revision}" \
  cargo test -p uor-r4-core --release --offline \
  --test intrinsic_lorentz_r4_attention_973 --no-run \
  --message-format=json-render-diagnostics >"${experiment_events}"

experiment_binary="$(python3 - "${experiment_events}" <<'PY'
import json
import sys

matches = []
with open(sys.argv[1], encoding="utf-8") as events:
    for line in events:
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = event.get("target", {})
        executable = event.get("executable")
        if (
            event.get("reason") == "compiler-artifact"
            and target.get("name") == "intrinsic_lorentz_r4_attention_973"
            and "test" in target.get("kind", [])
            and executable
        ):
            matches.append(executable)
if len(matches) != 1:
    raise SystemExit(f"expected one exact test executable, observed {matches!r}")
print(matches[0])
PY
)"

# The in-process deadline is checked between bounded operations. SIGALRM is an
# independent 75-minute wall watchdog on the exact test process itself.
exec /usr/bin/perl -e 'alarm shift; exec @ARGV or die "exec failed: $!\n"' 4500 \
  env \
  TLESS_CANONICAL_DETERMINISTIC=1 \
  UOR_R4_973_INTRINSIC_IMPLEMENTATION_REVISION="${experiment_revision}" \
  UOR_R4_973_INTRINSIC_MANIFEST="${experiment_manifest}" \
  UOR_R4_973_INTRINSIC_OUTPUT="${experiment_output}" \
  UOR_R4_973_INTRINSIC_WORKERS=8 \
  "${experiment_binary}" \
  --exact intrinsic_lorentz_r4_full_decoder_decision \
  --ignored --nocapture --test-threads=1
