#!/usr/bin/env bash
# Capture on Wick, then verify all words before any grade process opens its labels.
set -euo pipefail
umask 077
root="$HOME/uor-codex/20261118-query-region-pick/pick"
top="$HOME/uor-codex"
previous="$top/20261118-paired-role-fence"
fixture="$previous/ft-dev"
seed="$top/20261118-value-connectors/inputs/seed1.d11"
bin="$root/target/release/pick-reachability"
cuda="$root/bin/context-score-device-exact"
out="$root/${1:?required new capture name}"
phase="${2:?required phase capture or grade}"
case "$phase" in
  capture|grade) ;;
  *) exit 2 ;;
esac
carriers=(one-percent full-minus-fit)
stems=("$previous/count/primary-1-v2/run/depth3-equal" "$previous/count/primary-100/run/depth4-equal")
run_gpu() {
  local logfile="$1"
  shift
  (
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    nvidia-smi --query-gpu=timestamp,name,driver_version,memory.used,memory.free --format=csv --loop-ms=200       > "$logfile.gpu.csv" &
    monitor=$!
    trap 'kill "$monitor" 2>/dev/null || true; wait "$monitor" 2>/dev/null || true' EXIT
    /usr/bin/time -v -o "$logfile.time" "$@"
  ) 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
}
if [[ "$phase" == capture ]]; then
  mkdir "$out"
  cd "$out"
  sha256sum "$bin" "$cuda" > executables.sha256
  printf '%s  %s\n'     04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d "$seed"     b544f22f44cbb670aba39289233821504712d30f7d0ca7c2450cada551c02e51 "${stems[0]}.d11"     ff199225c52bd3364b938e61d496ad14bd66d559b9265d399a47ad6101181567 "${stems[0]}.reload.words"     d8744ebba0ec11470c22dcd6371a0cb4ce479c32c9e7fdc829d056cb20b9eea2 "${stems[1]}.d11"     0f8a039e257ef4016fb6f5eaedeb4c7ba1907b368fa62d6f30f3aa2a055feef2 "${stems[1]}.reload.words"     08893b3f749d2a062be7c81860ad5dbe095138e96e9fc221322f55eb90ef0906 "$root/SHEET.md"     > frozen.sha256
  sha256sum --check frozen.sha256
  mkdir finite
  "$bin" finite "$seed" finite > finite/weights.json
  run_gpu finite/pick "$bin" gpu "$seed" finite/prefixes.json finite/pick.gpu > finite/pick.json
  for i in "${!carriers[@]}"; do
    name="${carriers[$i]}"
    stem="${stems[$i]}"
    run_gpu "finite/$name" "$cuda" support "$stem.reload.words" finite/suffixes.u16 "finite/$name.gpu" 1
    "$bin" verify "$seed" "$stem.d11" "$stem.reload.words" finite/prefixes.json finite/pick.gpu       "finite/$name.gpu" "finite/$name.blind.json"
  done
  run_gpu finite/swap "$bin" gpu-swap "$seed" finite/prefixes.json finite/swap.gpu > finite/swap.json
  run_gpu finite/shift "$bin" gpu "$seed" finite/shifted-prefixes.json finite/shift.gpu > finite/shift.json
  for fault in swap shift; do
    if "$bin" verify "$seed" "${stems[0]}.d11" "${stems[0]}.reload.words" finite/prefixes.json       "finite/$fault.gpu" finite/one-percent.gpu "finite/$fault-unexpected.json"       > "finite/reject-$fault.log" 2>&1; then
      printf 'fault %s unexpectedly verified\n' "$fault" >&2
      exit 1
    fi
    grep -F 'identity Pick full-word mismatch' "finite/reject-$fault.log"
  done
  "$bin" prefixes "$fixture" .
  run_gpu pick "$bin" gpu "$seed" prefixes.json pick.gpu > pick.json
  for i in "${!carriers[@]}"; do
    name="${carriers[$i]}"
    stem="${stems[$i]}"
    run_gpu "$name" "$cuda" support "$stem.reload.words" suffixes.u16 "$name.gpu" 1
    /usr/bin/time -v -o "$name.reload.time" "$bin" verify "$seed" "$stem.d11" "$stem.reload.words"       prefixes.json pick.gpu "$name.gpu" "$name.blind.json"
  done
  sha256sum canonical-prefixes.json prefixes.json suffixes.u16 *.gpu *.snapshot.json *.blind.json     finite/{weights.json,prefixes.json,shifted-prefixes.json,suffixes.u16,*.gpu,*.snapshot.json,*.blind.json,reject-*.log}     > blind.sha256
  sha256sum --check blind.sha256
else
  cd "$out"
  test -s blind.sha256
  sha256sum --check blind.sha256
  sha256sum --check executables.sha256
  for i in "${!carriers[@]}"; do
    name="${carriers[$i]}"
    stem="${stems[$i]}"
    sha=$(awk -v file="$name.blind.json" '$2 == file {print $1}' blind.sha256)
    test -n "$sha"
    test ! -e "$name.result.json"
    /usr/bin/time -v -o "$name.grade.time" "$bin" grade "$fixture" "$stem.d11" prefixes.json "$name.gpu"       "$name.blind.json" "$sha" "$name.grades.jsonl" > "$name.result.json"
  done
  sha256sum *.result.json *.grades.jsonl > grades.sha256
  sha256sum --check grades.sha256
fi
