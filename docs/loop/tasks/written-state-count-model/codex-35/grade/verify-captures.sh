#!/usr/bin/env bash
# Verify graded inputs, both native orientations and both cold/device repeats before releasing the key.
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
root=$(realpath "$1")
split=$(realpath "$2")
device=$(realpath "$3")
primary=$(realpath "$4")
cold=$(realpath "$5")
frozen=$(realpath "$6")
tools_manifest=$(realpath "$7")
expected_tools_sha=$8
receipt=$(realpath "${9}")
receipt_sha=${10}
out=${11}
verify_file() { printf '%s  %s\n' "$1" "$2" | sha256sum --strict -c -; }
verify_file "$receipt_sha" "$receipt"
check_pin() { verify_file "$(jq -er --arg key "$1" '.[$key]' "$receipt")" "$2"; }
jq -e '.decision=="GO" and .phase=="offline-grade"' "$receipt" >/dev/null
check_pin tools_sha256 "$tools_manifest"
check_pin serve_go "$primary/go.json"
check_pin serve_go "$cold/go.json"
check_pin captures "$primary/captures.sha256"
check_pin captures "$cold/captures.sha256"
check_pin device_results "$device/results.sha256"
check_pin fact_packets "$split/fact-prefix/packets.sha256"
check_pin candidate_packets "$split/candidate-prefix/packets.sha256"
for g in fact-prefix candidate-prefix; do check_pin indexes "$split/$g/indexes.sha256"; done
check_pin offline_manifest "$frozen/offline/manifest.sha256"
check_pin blind_manifest "$frozen/held/manifest.sha256"
check_pin grade_script "$0"
printf '%s  %s\n' "$expected_tools_sha" "$tools_manifest" | sha256sum --strict -c -
(cd "$root" && sha256sum --strict --status -c "$tools_manifest")
mkdir "$out"
out=$(realpath "$out")
(cd "$device" && sha256sum --strict --status -c results.sha256)
for geometry in fact-prefix candidate-prefix; do
    (cd "$split/$geometry" && sha256sum --strict --status -c packets.sha256 && sha256sum --strict --status -c indexes.sha256)
done
for dir in "$primary" "$cold"; do
    (cd "$dir" && sha256sum --strict --status -c captures.sha256)
done
cmp "$primary/captures.sha256" "$cold/captures.sha256"
for prefix in fact candidate; do
    geometry=$prefix-prefix
    for mode in ordinary supplied; do
        packet=$mode
        if [[ $mode == supplied ]]; then packet=diagnostic; fi
        for pass in primary cold; do
            capture=$primary
            if [[ $pass == cold ]]; then capture=$cold; fi
            base="$out/$prefix-$mode-$pass"
            "$root/device/paired-value-decode" "$device/$geometry/$pass/$packet.out" > "$base.decoded.json"
            "$root/grade/target/release/d35-parity" "$split/$geometry/$packet.index.json" "$base.decoded.json" \
                "$capture/$mode/${prefix}_true" "$capture/$mode/${prefix}_reversed" > "$base.parity.json"
        done
    done
done
(cd "$out" && sha256sum *.parity.json *.decoded.json > parity.sha256)
# The key lives outside serving, projection and CUDA inputs. It can enter only the following offline grade.
(cd "$frozen/offline" && sha256sum --strict --status -c manifest.sha256)
jq -e '.arms' "$receipt" > "$out/arm-hashes.json"
grade="$root/grade-v5/target/release"
"$grade/d35-value-grade" "$frozen/offline/key.json" "$primary" "$cold" "$out/arm-hashes.json" > "$out/graded.json"
"$grade/d35-summary" "$out/graded.json" > "$out/summary.json"
(cd "$out" && sha256sum arm-hashes.json parity.sha256 graded.json summary.json > results.sha256)
