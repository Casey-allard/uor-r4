#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261119-value-ranking"
driver=src/docs/loop/tasks/written-state-count-model/codex-17/capture/run-wick.sh
pins() {
    sha256sum --status -c launch.sha256
    (cd ../20261118-ordered-role-transfer && sha256sum --status -c ../20261119-value-ranking/chain.sha256)
}
pins
mkdir capture
for phase in primary repeat; do
    for split in fit held; do
        workers=()
        for seed in 1 2; do
            spec="$PWD/specs/$split-seed$seed.json"
            seal=$(awk -v key="specs/$split-seed$seed.json" '$2==key{print $1}' launch.sha256)
            out="$PWD/capture/$phase-$split-seed$seed"
            /usr/bin/time -v -o "$out.time" bash "$driver" "$spec" "$seal" "$out" > "$out.log" 2>&1 &
            workers+=("$!")
        done
        rc=0
        for worker in "${workers[@]}"; do wait "$worker" || rc=1; done
        test "$rc" = 0
        pins
    done
done
for split in fit held; do
    for seed in 1 2; do
        for phase in primary repeat; do
            dir="capture/$phase-$split-seed$seed"
            (cd "$dir" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) > "$dir.all.sha256"
            (cd "$dir" && sha256sum --status -c "../$phase-$split-seed$seed.all.sha256")
        done
        cmp "capture/primary-$split-seed$seed/cold.sha256" "capture/repeat-$split-seed$seed/cold.sha256"
        # Retain all reports. If a path is embedded in a receipt, classify the
        # difference before qualifying individual trajectories or fit inputs.
        diff -u "capture/primary-$split-seed$seed.all.sha256" "capture/repeat-$split-seed$seed.all.sha256" > "capture/$split-seed$seed.all.diff" || test "$?" = 1
    done
done
