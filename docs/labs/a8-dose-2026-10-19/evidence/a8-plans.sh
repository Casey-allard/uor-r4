#!/bin/bash
# Build the two A8 owner plans from actual build identities (no model work, no release).
# Plan fields mirror Codex's rows_v1 plan template; identities come from attention-build-identity.
set -eu
SRC=/root/build/src-2c015db9f01d3ba3a7d79889a052b1345d9da48f
B=/root/build/target-sm120/release
H=/root/build/payload-tool-target/release/a8-plan-payload
CFG=$SRC/docs/labs/a8-training-configs-2026-10-19/evidence
SHEET=$SRC/docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md
A=/workspace/a8-dose
mkdir -p "$A/plans" "$A/authorization"
sheet_sha=$(sha256sum "$SHEET" | cut -c1-64)
[ "$sheet_sha" = 3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b ] || { echo "sheet SHA mismatch"; exit 2; }
for arm in relation frozen-zero; do
  C="$CFG/a8-$arm-seed1.json"; P="$A/plans/a8-$arm-seed1.plan.json"
  id=$("$B/discrete-stack" attention-build-identity "$C")
  echo "$id" > "$A/plans/a8-$arm-seed1.identity.json"
  case $arm in relation) scope=m1-attention-learning-injective-6600bdb1306d-relation-k8-seed1-batches0..16404;;
               frozen-zero) scope=m1-attention-learning-injective-6600bdb1306d-relation-frozen-zero-k8-seed1-batches0..16404;; esac
  jq -n --slurpfile cfg "$C" --argjson id "$id" --arg scope "$scope" --arg sheet "$SHEET" --arg sheet_sha "$sheet_sha" \
     --arg review "$A/authorization/a8-$arm-seed1.review.json" --arg state "$A/state/a8-$arm-seed1" \
     --arg tok "$A/fresh-sealed-v2/tokens.u16" --arg mask "$A/fresh-sealed-v2/response_mask.u8" '{
    schema:"uor-r4.attention-learning-owner-plan/2", status:"HELD_OWNER_ACTION_REQUIRED", scope:$scope,
    execution:{base_source:"ef9d30999700be11e103f96590e9e3b83b6807c7", reviewed_source_commit:"2c015db9f01d3ba3a7d79889a052b1345d9da48f",
      source_sha256:$id.compiled.source_sha256, kernel_sha256:$id.compiled.kernel_sha256, geometry_sha256:$id.compiled.geometry_sha256,
      executable_sha256:$id.executable_sha256, driver_source_sha256:$id.driver_source_sha256,
      config_sha256:$id.config_sha256, lineage_sha256:$id.lineage_sha256, sheet_sha256:$sheet_sha},
    config:$cfg[0], sheet:{path:$sheet, sha256:$sheet_sha},
    independent_review:{path:$review, sha256:"0000000000000000000000000000000000000000000000000000000000000000"},
    transfer:{split:"dev", tokens:{path:$tok, sha256:"0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8"},
      mask:{path:$mask, sha256:"a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e"},
      expected_payload_tokens:44678, expected_eligible_targets:14479},
    run_state_directory:$state,
    cost_contract:{max_batches:16404, checkpoint_every_batches:64, baseline_dev_evaluations:1, q1_dev_evaluations:1,
      final_dev_evaluations:1, relation_off_dev_evaluations:1, final_transfer_evaluations:1, relation_off_transfer_evaluations:0}}' > "$P.draft"
  "$H" "$P.draft" | tee "$A/plans/a8-$arm-seed1.payload.json"
done
echo PLANS_DRAFTED
