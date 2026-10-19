#!/bin/bash
# Seal each arm's review into its plan and run attention-check-plan (metadata only; no release).
# Usage: a8-seal.sh RECEIPT_URL   (the #2029 comment that carries the review verdict)
set -eu
URL=$1
B=/root/build/target-sm120/release
H=/root/build/payload-tool-target/release/a8-plan-payload
A=/workspace/a8-dose
for arm in ${2:-relation frozen-zero}; do
  D="$A/plans/a8-$arm-seed1.plan.json.draft"; P="$A/plans/a8-$arm-seed1.plan.json"; RV="$A/authorization/a8-$arm-seed1.review.json"
  pay=$("$H" "$D" | jq -r .reviewed_plan_payload_sha256)
  jq -n --slurpfile p "$D" --arg pay "$pay" --arg url "$URL" '{reviewer:"Claude", verdict:"APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF",
     scope:$p[0].scope, execution:$p[0].execution, reviewed_plan_payload_sha256:$pay, receipt_url:$url}' > "$RV"
  rs=$(sha256sum "$RV" | cut -c1-64)
  jq --arg rs "$rs" '.independent_review.sha256 = $rs' "$D" > "$P"
  [ "$("$H" "$P" | jq -r .reviewed_plan_payload_sha256)" = "$pay" ] || { echo "payload changed after sealing: FAIL"; exit 2; }
  echo "== $arm review=$rs payload=$pay"
  "$B/discrete-stack" attention-check-plan "$P" | tee "$A/plans/a8-$arm-seed1.check-plan.json" | jq -c '{status, scope, plan_sha256, owner_release, training, compiled_identity_checked}'
done
sha256sum "$A"/plans/*.plan.json "$A"/authorization/*.json
echo SEALED
