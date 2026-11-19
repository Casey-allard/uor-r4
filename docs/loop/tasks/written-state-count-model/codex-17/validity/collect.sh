#!/usr/bin/env bash
# Materialize the sealed per-seed eligibility receipts after the blind stage.
# No training, device scoring or grading of generated answers happens here.
set -euo pipefail
umask 077
[[ $# == 3 ]] || { echo "usage: collect.sh WICK_STAGE BINDER_STAGE NEW_DIRECTORY" >&2; exit 2; }
stage=$(cd "$1" && pwd -P)
built=$(cd "$2" && pwd -P)
here=$(cd "$(dirname "$0")" && pwd -P)
out=$3
[[ "$out" == /* && ! -e "$out" ]]
mkdir "$out"
cd "$stage/capture"
for phase in primary repeat; do
    for split in fit held; do
        for seed in 1 2; do
            dir="$phase-$split-seed$seed"
            [[ -s "$dir.all.sha256" ]]
            (cd "$dir" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) |
                cmp - "$dir.all.sha256"
        done
    done
done
sha256sum {primary,repeat}-{fit,held}-seed{1,2}.all.sha256 > "$out/raw-index.sha256"
sha256sum -c "$out/raw-index.sha256" > "$out/raw-verification.log"
for seed in 1 2; do
    for split in fit held; do
        cmp "primary-$split-seed$seed/cold.sha256" "repeat-$split-seed$seed/cold.sha256"
        primary="$stage/capture/primary-$split-seed$seed"
        repeat="$stage/capture/repeat-$split-seed$seed"
        # Original manifests are verified above. Every byte outside the three
        # path-addressed read inventories must match between independent runs.
        audit_pattern='  \./(no-read|requested-diagnostic|public)/archive-audit\.json$'
        cmp <(grep -Ev "$audit_pattern" "$primary.all.sha256") <(grep -Ev "$audit_pattern" "$repeat.all.sha256")
        for lane in public no-read requested-diagnostic; do
            for root in "$primary" "$repeat"; do
                audit="$root/$lane/archive-audit.json"
                jq -e '.schema=="uor-r4.legacy-assertion-capture-audit/1" and (.reads_sha256|length)>0
                     and all(.reads_sha256|to_entries[]; (.key|startswith("/") and (test("[\\r\\n]")|not))
                         and (.value|test("^[0-9a-f]{64}$")))' "$audit" >/dev/null
                jq -r '.reads_sha256|to_entries[]|.value+"  "+.key' "$audit" | sha256sum --status -c -
                # Separate namespaces prevent substitution of an external
                # path for a phase-relative one. All digest values survive.
                jq -cS --arg prefix "$root/" '.reads_sha256 |=
                  (to_entries | map({key:(if (.key|startswith($prefix)) then
                    ["run",(.key|ltrimstr($prefix))] else ["external",.key] end),value}) | sort_by(.key))' \
                    "$audit" > "$out/$(basename "$root")-$lane.normalized.json"
            done
            cmp "$out/$(basename "$primary")-$lane.normalized.json" "$out/$(basename "$repeat")-$lane.normalized.json"
        done
    done
    case $seed in
        1) draft=135f297e948973a201fc02909e0eb70421179c3e38337bdbcabb811fd7407e57;;
        2) draft=273991aab09aa31a1426baace23d5dda80a5bd7c89bbd846c9984d23e53e3d67;;
    esac
    for phase in primary repeat; do
        receipt="$out/$phase-seed$seed"
        "$here/bind-capture.sh" "$stage" "$seed" "$phase" "$built/specs/draft-seed$seed.json" \
            "$draft" "$out/raw-index.sha256" "$receipt.spec.json"
        sha=$(sha256sum "$receipt.spec.json" | cut -d' ' -f1)
        "$here/check-pinned.sh" "$receipt.spec.json" "$sha" "$receipt.validity.json" || {
            # An explicit false gate is a valid measurement of testability.
            jq -e '.eligible_for_deciding_run==false' "$receipt.validity.json" >/dev/null
        }
    done
    cmp "$out/primary-seed$seed.validity.json" "$out/repeat-seed$seed.validity.json"
done
(cd "$out" && sha256sum *.json raw-index.sha256 > receipt.sha256 && sha256sum --status -c receipt.sha256)
