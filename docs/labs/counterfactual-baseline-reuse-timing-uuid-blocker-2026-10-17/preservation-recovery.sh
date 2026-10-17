#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/share/uor-r4/bin:$PATH"
export UOR_LAB=codex UOR_POD_SESSION=codex-cf1-reuse-timing-run2
export UOR_POD_CONFIG=/dev/null UOR_POD_GIT_DIR=/Users/casey.allard/uor-r4
export UOR_POD_VAST_BIN=/Users/casey.allard/uor-r4-local/bin/codex-uor-r4-pod-provider-driver2567
BASE=/Users/casey.allard/uor-r4-local/codex-cf1-reuse-timing-run2
POD_TOOL="$BASE/runner/uor-pod"
CLOUD_TOOL=/Users/casey.allard/.local/share/uor-r4/bin/cloud-store
POD=vast-56332437
SESSION=codex-cf1-reuse-timing-run2
cd "$BASE"
shasum -a256 -c REVIEW-SHA256SUMS
shasum -a256 -c EXTERNAL-SHA256SUMS
shasum -a256 -c FINAL-PIN-SHA256SUMS
"$POD_TOOL" renew "$POD" --lab codex --session "$SESSION" --hours 0.5
terminal=$("$POD_TOOL" ssh "$POD" 'cat /workspace/codex-cf1-reuse-timing-run2/command.exit')
[ "$terminal" = 1 ]
archive="$BASE/reuse-timing-run2-archive"
cmp qualified-gpu.csv "$archive/qualified-gpu.csv"
cp qualified-gpu.csv "$archive/local-operator/qualified-gpu.csv"
cp "$archive/OPERATOR-SHA256SUMS" "$archive/OPERATOR-SHA256SUMS.initial"
mkdir -p "$archive/preservation-recovery"
cp lease.log "$archive/preservation-recovery/lease-final-failed.log"
cp preservation-recovery.sh "$archive/preservation-recovery/"
cat > "$archive/preservation-recovery/README.md" <<'NOTE'
Preservation-only recovery after terminal setup refusal, no instrument binary invoked.
The reviewed operator omitted *.csv from its local-operator copy glob, so the
frozen REVIEW-SHA256SUMS failed on qualified-gpu.csv. The trap retained and
renewed vast-56332437; it did not upload, release or delete on failure.
Parent copied the exact frozen CSV after cmp with the complete remote archive.
Original operator bytes and initial manifest/failure log remain retained.
This recovery changes no execution condition, source, model input or dose.
All original and complete updated manifests are checked before full cloud
MD5 upload/readback; only then release/down is permitted. A recovery failure
leaves the identified pod retained for renewed recovery, never reruns the job.
NOTE
jq -c --arg s "$SESSION" 'select(.lab=="codex" and .session==$s)' "$HOME/.local/share/uor-r4/compute/ledger.jsonl" > "$archive/preservation-recovery/session-ledger.jsonl"
(cd "$archive" && find local-operator -type f -exec shasum -a256 {} + > OPERATOR-SHA256SUMS && find preservation-recovery -type f -exec shasum -a256 {} + > RECOVERY-SHA256SUMS && shasum -a256 -c SHA256SUMS && shasum -a256 -c INPUT-SHA256SUMS && shasum -a256 -c OPERATOR-SHA256SUMS.initial && shasum -a256 -c OPERATOR-SHA256SUMS && shasum -a256 -c RECOVERY-SHA256SUMS && (cd local-operator && shasum -a256 -c REVIEW-SHA256SUMS))
free_kib=$(df -Pk "$BASE" | awk 'NR==2 {print $4}')
archive_kib=$(du -sk "$archive" | awk '{print $1}')
[ "$free_kib" -ge $((31457280 + 2 * archive_kib + 32768)) ]
"$CLOUD_TOOL" put codex "$archive" | tee "$BASE/cloud-readback.log"
grep -Eq '^stored reuse-timing-run2-archive \([0-9]+ bytes, md5 [0-9a-f]{32}\)' "$BASE/cloud-readback.log"
"$POD_TOOL" release "$POD" --lab codex --session "$SESSION"
"$POD_TOOL" down "$POD" --lab codex --session "$SESSION" --confirm-archived
"$POD_TOOL" status > "$BASE/fleet-after.log" 2>&1
jq -c --arg s "$SESSION" 'select(.lab=="codex" and .session==$s)' "$HOME/.local/share/uor-r4/compute/ledger.jsonl" > "$BASE/session-ledger-final.jsonl"
cat "$BASE/fleet-after.log"
echo 'PRESERVATION_RECOVERY_COMPLETE; setup remained blocked and model dose0'
