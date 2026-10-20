#!/usr/bin/env bash
# DO NOT EXECUTE during preparation/review. Separate release + final pin required.
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/share/uor-r4/bin:$PATH"
export UOR_LAB=codex UOR_POD_SESSION=codex-cf1-reuse-finite-run3
export UOR_POD_VAST_BIN=/Users/casey.allard/uor-r4-local/bin/codex-uor-r4-pod-provider-driver2567
export UOR_POD_VAST_DRIVER_VERSION=595.71.05 UOR_POD_CONFIG=/dev/null UOR_POD_GIT_DIR=/Users/casey.allard/uor-r4
export UOR_POD_MAX_PODS=4 UOR_POD_MAX_RATE=8.00 UOR_POD_PROVIDERS=vast
BASE=/Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run3
POD_TOOL="$BASE/runner/uor-pod"
CLOUD_TOOL=/Users/casey.allard/.local/share/uor-r4/bin/cloud-store
GATE=/Users/casey.allard/uor-r4-local/codex-reuse-resource-refusal/gate1/gate2562-r1-archive
SESSION=codex-cf1-reuse-finite-run3
ARCHIVE="$BASE/reuse-finite-run3-archive"
REMOTE=/workspace/codex-cf1-reuse-finite-run3
POD='' RUN_ATTEMPTED=0
retain() {
  echo "RETAIN $POD: $*; rejoin and renew, never release/down while state or preservation is unknown" >&2
  "$POD_TOOL" renew "$POD" --lab codex --session "$SESSION" --hours 0.5 || echo 'Renewal failed: retained pod still requires immediate recovery' >&2
}
cleanup() {
  result=$?; trap - EXIT; set +e
  jq -c --arg s "$SESSION" 'select(.lab=="codex" and .session==$s)' "$HOME/.local/share/uor-r4/compute/ledger.jsonl" > "$BASE/session-ledger.jsonl"
  session_ledger=$?
  if [ -z "$POD" ]; then
    POD=$(jq -r --arg s "$SESSION" 'select(.event=="up" and .lab=="codex" and .session==$s) | .pod' "$HOME/.local/share/uor-r4/compute/ledger.jsonl" | tail -1)
  fi
  if [ -z "$POD" ]; then echo "No identified pod; exit=$result; inspect up/ledger if creation was interrupted"; exit "$result"; fi
  if [ "$RUN_ATTEMPTED" -eq 1 ]; then
    terminal=$("$POD_TOOL" ssh "$POD" "test -f '$REMOTE/command.exit' && cat '$REMOTE/command.exit'")
    observed=$?
    if [ "$observed" -ne 0 ] || ! [[ "$terminal" =~ ^[0-9]+$ ]] || [ "$terminal" -gt 255 ]; then
      retain 'RUN STATE UNKNOWN OR LIVE; atomic terminal marker not observed'; exit 1
    fi
  fi
  "$POD_TOOL" renew "$POD" --lab codex --session "$SESSION" --hours 0.5 || { retain 'renewal failed'; exit 1; }
  "$POD_TOOL" ssh "$POD" "bash -s -- '$REMOTE'" <<'REMOTE_ARCHIVE'
set -euo pipefail
out=$1
mkdir -p "$out/jobs"
if [ -d /workspace/uor-r4/jobs/codex ]; then cp -R /workspace/uor-r4/jobs/codex/. "$out/jobs/"; fi
cd "$out"
find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
REMOTE_ARCHIVE
  collected=$?
  remote_kib=$("$POD_TOOL" ssh "$POD" "du -sk '$REMOTE'" | awk '{print $1}')
  available_kib=$(df -Pk "$BASE" | awk 'NR==2 {print $4}')
  if ! [[ "$remote_kib" =~ ^[0-9]+$ && "$available_kib" =~ ^[0-9]+$ ]]; then retain 'archive size/capacity unknown'; exit 1; fi
  if [ "$available_kib" -lt $((31457280 + 2 * remote_kib + 32768)) ]; then retain 'archive plus cloud-stage copy would cross local30GiB floor'; exit 1; fi
  mkdir -p "$ARCHIVE"
  "$POD_TOOL" ssh "$POD" "tar -C '$REMOTE' -cf - ." | tar -C "$ARCHIVE" -xf -
  copied=$?
  mkdir -p "$ARCHIVE/local-operator"
  operator_copy=$session_ledger
  for file in "$BASE"/*.json "$BASE"/*.md "$BASE"/*.sh "$BASE"/*.log "$BASE"/*.txt "$BASE"/*.jsonl "$BASE"/*SHA256SUMS; do
    [ ! -f "$file" ] || cp "$file" "$ARCHIVE/local-operator/" || operator_copy=1
  done
  cp -R "$BASE/runner" "$ARCHIVE/local-operator/runner" || operator_copy=1
  (cd "$ARCHIVE" && find local-operator -type f -exec shasum -a 256 {} + > OPERATOR-SHA256SUMS)
  operator_manifest=$?
  if [ "$collected" -ne 0 ] || [ "$copied" -ne 0 ] || [ "$operator_copy" -ne 0 ] || [ "$operator_manifest" -ne 0 ]; then retain 'incomplete archive capture'; exit 1; fi
  if ! (cd "$ARCHIVE" && shasum -a 256 -c SHA256SUMS && shasum -a 256 -c OPERATOR-SHA256SUMS && shasum -a 256 -c INPUT-SHA256SUMS && (cd local-operator && shasum -a 256 -c REVIEW-SHA256SUMS)); then retain 'complete SHA256 verification failed'; exit 1; fi
  # Pinned cloud-store put uploads the whole tar, streams the complete remote
  # tar back through MD5, and writes its index only after equality. No sampling.
  if ! "$CLOUD_TOOL" put codex "$ARCHIVE" | tee "$BASE/cloud-readback.log"; then retain 'full cloud MD5 upload/readback failed'; exit 1; fi
  if ! grep -Eq '^stored reuse-finite-run3-archive \([0-9]+ bytes, md5 [0-9a-f]{32}\)' "$BASE/cloud-readback.log"; then retain 'cloud completion receipt missing'; exit 1; fi
  "$POD_TOOL" release "$POD" --lab codex --session "$SESSION" &&
    "$POD_TOOL" down "$POD" --lab codex --session "$SESSION" --confirm-archived || result=1
  "$POD_TOOL" status > "$BASE/fleet-after.log" 2>&1 || result=1
  cat "$BASE/fleet-after.log"
  echo "LEASE_RESULT=$result POD=$POD"
  exit "$result"
}
# Installed before any creation. Local failures cannot silently orphan an owned pod.
trap cleanup EXIT
cd "$BASE"
shasum -a 256 -c REVIEW-SHA256SUMS
# The preparer creates neither of these files. Only a separately reviewed release
# may supply them; the release URL must be the final pin's sole changed field.
# These files do not exist in the prospective packet. A later PM release must
# bind this exact readiness manifest, confirm hygiene75611 completed, and supply
# the final pin. No API is reached until these local gates all pass.
test -s readiness-verdict.md
test -s hygiene-complete.json
jq -e '.schema=="uor-r4.hygiene-admission/1" and .process_id==75611 and .completed==true and (.evidence_reference|type=="string" and length>0)' hygiene-complete.json >/dev/null
test -s execution-release.md
test -s finite-pin.json
release=$(jq -er '.execution_release_reference | select(type=="string" and startswith("https://github.com/UOR-Foundation/uor-r4/") and contains("#"))' finite-pin.json)
test "$release" != "$(jq -r .test_fitness_reference finite-pin.json)"
test "$release" != "$(jq -r .finite_acceptance_reference finite-pin.json)"
test "$release" != 'https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227632827'
test "$release" != 'https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227215454'
test "$release" != 'https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227893756'
cmp <(jq '.execution_release_reference=null' finite-pin.json) <(jq . finite-pin.review.json)
shasum -a 256 -c FINAL-PIN-SHA256SUMS
for binding in \
 "$UOR_POD_VAST_BIN:cb524800bf583d1bffc956516f430e74128dc999224b5970b294588521cdc13d" \
 "$POD_TOOL:1ae8e6021dc6853d2cf62571842c9d429b5d319acad8aa7d18a42ea889cc8fd9" \
 "$CLOUD_TOOL:2e07a318d58b2f5d2f4ec73a32073b657a9d26d5263bd9398e7a9a4663ad3319" \
 "$GATE/k16-timing:671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209"; do
  file=${binding%:*}; expected=${binding##*:}
  test "$(shasum -a 256 "$file" | awk '{print $1}')" = "$expected"
done
df -Pk "$BASE" | tee df-preflight.log
free_kib=$(df -Pk "$BASE" | awk 'NR==2 {print $4}')
[ "$free_kib" -ge 31457280 ] || { echo 'DISK FLOOR STOP: below30GiB'; exit 1; }
"$POD_TOOL" status > fleet-before.log 2>&1
cat fleet-before.log
# uor-pod up performs strict cross-provider fleet/rate admission and quotes the
# actual Vast offer before creation. No owner override, fallback or wait/retry.
"$POD_TOOL" up --provider vast --gpu 5090 --count 1 --lab codex --session "$SESSION" --purpose 'Released environment qualification 595.71.05 only;64 U/R finite attempts;16 refusals;apply0;no timing' --hours 0.5 --no-bootstrap --ref 5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2 --card "$release" > up.log 2>&1
cat up.log
POD=$(awk '/^Pod vast-[0-9]+ created/{print $2;exit}' up.log)
[ -n "$POD" ] || { echo 'Cannot identify own newly created pod'; exit 1; }
printf '%s\n' "$POD" > pod-id.txt
"$POD_TOOL" lease "$POD" --lab codex --session "$SESSION" --gpus 0 --purpose 'Finite environment qualification 595.71.05 only;64 attempts;16 refusals;separate release' --hours 0.5 --card "$release"
"$POD_TOOL" ssh "$POD" "mkdir -p '$REMOTE'"
"$POD_TOOL" ssh "$POD" "cat > '$REMOTE/k16-timing'; chmod 755 '$REMOTE/k16-timing'" < "$GATE/k16-timing"
for file in expected-identity.json finite-pin.json finite-pin.review.json execution-review.md execution-release.md run-remote.sh; do
  "$POD_TOOL" ssh "$POD" "cat > '$REMOTE/$file'" < "$BASE/$file"
done
{ printf '%s  k16-timing\n' 671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209; shasum -a 256 expected-identity.json finite-pin.json finite-pin.review.json execution-review.md execution-release.md run-remote.sh; } > INPUT-SHA256SUMS
"$POD_TOOL" ssh "$POD" "cat > '$REMOTE/INPUT-SHA256SUMS'" < INPUT-SHA256SUMS
RUN_ATTEMPTED=1
"$POD_TOOL" run "$POD" --lab codex --session "$SESSION" --gpu 0 -- bash "$REMOTE/run-remote.sh"
last_renew=$(date +%s)
while :; do
  state=$("$POD_TOOL" ssh "$POD" "if [ -f '$REMOTE/command.exit' ]; then cat '$REMOTE/command.exit'; else echo WAIT; fi")
  if [ "$state" != WAIT ]; then
    [[ "$state" =~ ^[0-9]+$ ]] || { echo 'Invalid terminal marker; retain for recovery'; exit 1; }
    "$POD_TOOL" ssh "$POD" "cat '$REMOTE/command.log'"
    exit "$state"
  fi
  if [ $(( $(date +%s) - last_renew )) -ge 600 ]; then
    "$POD_TOOL" renew "$POD" --lab codex --session "$SESSION" --hours 0.5
    last_renew=$(date +%s)
  fi
  echo "Finite environment qualification live on $POD; $(date -u +%FT%TZ)"
  sleep 10
done
