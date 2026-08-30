#!/usr/bin/env bash
set -euo pipefail

experiment_repo="$(git rev-parse --show-toplevel)"
cd "${experiment_repo}"

experiment_revision="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain=v1 --untracked-files=no)" ]]; then
  echo "refusing to run: tracked checkout is not clean at ${experiment_revision}" >&2
  exit 1
fi

experiment_partition="${experiment_repo}/docs/helm_d_learned_manifold_r4_construction_partition_973.json"
experiment_attempt="${UOR_R4_973_SCORE_CENTROID_ATTEMPT:-attempt-01}"
if [[ ! "${experiment_attempt}" =~ ^attempt-[0-9][0-9]$ ]]; then
  echo "refusing to run: attempt must have the form attempt-NN" >&2
  exit 1
fi

experiment_evidence_root="/Users/casey.allard/uor-r4/.uor-models/research/issue-973-helm-d-score-centroid-localization/5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde"
experiment_checkpoint="${experiment_evidence_root}/${experiment_attempt}.checkpoint.json"
experiment_output="${experiment_evidence_root}/${experiment_attempt}.result.json"
experiment_target_commitment="${experiment_evidence_root}/${experiment_attempt}.target-commitments.json"
experiment_events="$(mktemp -t uor-r4-973-score-centroid-build.XXXXXX)"
trap 'rm -f "${experiment_events}"' EXIT

mkdir -p "${experiment_evidence_root}"
if [[ -e "${experiment_checkpoint}" || -e "${experiment_output}" || -e "${experiment_target_commitment}" ]]; then
  echo "refusing to overwrite evidence for ${experiment_attempt}" >&2
  exit 1
fi

cargo test -p uor-r4-core --release --offline \
  --test helm_d_learned_manifold_r4_construction_973 --no-run \
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
            and target.get("name") == "helm_d_learned_manifold_r4_construction_973"
            and "test" in target.get("kind", [])
            and executable
        ):
            matches.append(executable)
if len(matches) != 1:
    raise SystemExit(f"expected one exact test executable, observed {matches!r}")
print(matches[0])
PY
)"

# This locally held commitment freeze is the only operation allowed to open the
# eight audit targets before the decision. It runs from the exact protected
# executable and keeps the one-token CIDs in the local evidence cache; they
# are never committed to Git where their small preimage space could leak them.
env \
  UOR_R4_973_SCORE_CENTROID_PARTITION="${experiment_partition}" \
  UOR_R4_973_SCORE_CENTROID_TARGET_COMMITMENT_OUTPUT="${experiment_target_commitment}" \
  "${experiment_binary}" \
  --exact freeze_helm_d_score_centroid_localization_r4_v1_targets \
  --ignored --nocapture --test-threads=1

# The in-process contract stops after the two-document preflight when its
# scientific gate fails. SIGALRM independently caps an admitted full run at
# 80 minutes; the release build above is deliberately outside that allowance.
exec /usr/bin/perl -e 'alarm shift; exec @ARGV or die "exec failed: $!\n"' 4800 \
  env \
  TLESS_CANONICAL_DETERMINISTIC=1 \
  UOR_R4_973_SCORE_CENTROID_PARTITION="${experiment_partition}" \
  UOR_R4_973_SCORE_CENTROID_TARGET_COMMITMENT="${experiment_target_commitment}" \
  UOR_R4_973_SCORE_CENTROID_CHECKPOINT="${experiment_checkpoint}" \
  UOR_R4_973_SCORE_CENTROID_OUTPUT="${experiment_output}" \
  "${experiment_binary}" \
  --exact helm_d_score_centroid_localization_r4_v1_decision \
  --ignored --nocapture --test-threads=1
