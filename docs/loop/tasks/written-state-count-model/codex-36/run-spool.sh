#!/usr/bin/env bash
# Execute one reviewed, device-bound CUDA count materialization on Wick.
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
root=$(realpath "$1")
go=$(realpath "$2")
go_sha=$3
printf '%s  %s\n' "$go_sha" "$go" | sha256sum --strict -c -
cd "$root"
field(){ jq -er --arg f "$1" '.[$f]' "$go"; }
check(){ printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
test "$(field decision)" = GO
test "$(field host)" = wick
dose=$(field dose)
case "$dose" in c128|c) ;; *) exit 2;; esac
binary=$(field executable)
corpus=$(field corpus)
census=$(field census)
dest=$(field output)
check "$binary" "$(field executable_sha256)"
check "$census" "$(field census_sha256)"
check "$(field fit)" "$(field fit_sha256)"
check "$(field plan)" "$(field plan_sha256)"
check "$(field sheet)" "$(field sheet_sha256)"
test ! -e "$dest"
test "$(nvidia-smi --query-gpu=uuid --format=csv,noheader)" = "$(field gpu_uuid)"
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 8.9
export CUDA_VISIBLE_DEVICES="$(field gpu_uuid)" CUDA_DISABLE_PTX_JIT=1
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -n 8 || { echo "shared GPU lock busy"; exit 75; }
foreign=$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)
if [[ -n "$foreign" ]]; then echo "GPU occupied by $foreign"; exit 75; fi
# The producer hashes the entire corpus against the dose's compiled-in pin before dispatch.
test "$(jq -er .train_sha256 "$(field plan)")" = "$(field corpus_sha256)"
test "$(jq -er .dose "$(field plan)")" = "$dose"
test "$(df -PB1 "$root" | awk 'END{print $4}')" -ge "$(field min_disk_bytes)"
exec /usr/bin/time -v -o "$dest.time" "$binary" spool "$dose" "$corpus" "$census" "$dest"
