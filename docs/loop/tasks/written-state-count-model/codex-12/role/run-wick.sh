#!/usr/bin/env bash
set -euo pipefail
# Run from this unit's dedicated root. A new output basename is required.
[[ $# == 1 && "$1" =~ ^[A-Za-z0-9-]+$ ]]
out=$1
base=$HOME/uor-codex
value=$base/20261118-value-connectors
span=$base/20261118-span-connectors
old=$base/20261118-role-boundary-count
prior=$base/20261118-content-support
continuation=$base/20261118-continuation-eos
check_inputs() {
    test -s ROLE-FIT.md
    sha256sum -c release.sha256
    (cd panels; sha256sum -c SHA256SUMS)
}
resolve() {
    case "$1" in
        held|fresh) tape=panels/$1-blind.json ;;
        historical) tape=$prior/panels/held-blind.json ;;
        mixed) tape=$old/panels/action-held-blind.json ;;
        unseen) tape=$old/panels/unseen-blind.json ;;
        transfer) tape=$continuation/panels/transfer-blind.json ;;
        *) return 2 ;;
    esac
}
seed() {
    model=$value/inputs/seed$1.d11
    tokenizer=$value/inputs/tokenizer.json
    [[ $1 == 1 ]] || tokenizer=$value/inputs/tokenizer-seed-2.json
    span_state=$span/results/span-01/seed$1-state.json
}
check_inputs
mkdir "$out"
exec 9>"$base/20261118-diagnostics/wick-codex-gpu.lock"
device() {
    flock 9
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$@"
    flock -u 9
}
nvidia-smi --query-gpu=name,uuid,driver_version --format=csv > "$out/device.txt"
date -u -Iseconds > "$out/start.txt"
device bin/paired-role-device finite bin/device > "$out/finite.txt"
for n in 1 2; do
    seed "$n"
    device bin/paired-role-device fit bin/device "$model" "$tokenizer" "$span_state" panels/train-blind.json panels/train-grade.json "$out/seed$n.role" > "$out/seed$n-fit.txt"
    for p in held fresh historical mixed unseen transfer; do
        resolve "$p"
        device bin/paired-role-device blind bin/device "$model" "$tokenizer" "$out/seed$n.role" "$tape" "$out/seed$n-$p.json" > "$out/seed$n-$p-parity.txt"
    done
done
seed 2
if bin/paired-role-device blind bin/device "$model" "$tokenizer" "$out/seed1.role" panels/held-blind.json "$out/rejected.json" > "$out/rejection.txt" 2>&1; then
    echo 'cross-seed bank was accepted' >&2
    exit 2
fi
grep -q 'role bank model/tokenizer binding differs' "$out/rejection.txt"
(cd "$out"; sha256sum seed*.role seed*.json > decisions.sha256)
date -u -Iseconds > "$out/end.txt"
