#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 4 ]]; then
    echo "usage: $0 PROBE UOR_CHAT STUDY_ROOT NEW_OUTPUT_DIR" >&2
    exit 2
fi
probe=$(realpath "$1")
chat=$(realpath "$2")
root=$(realpath "$3")
mkdir "$4"
out=$(realpath "$4")
old="$HOME/uor-codex/20261118-role-boundary-count/bin/action-writer"
gate="$HOME/uor-codex/20261118-query-region-pick/writer/run-wick.sh"
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
sha256sum "$chat" "$probe" "$old" > "$out/executables.sha256"
test "$(sha256sum "$old" | cut -d' ' -f1)" = 004cc78a1d0b616c15986c2032b6c07f661536851c4d24a627d6eb6d4258e592
(cd "$root/assets"; sha256sum --status -c SHA256SUMS)
for seed in 1 2; do
    trunk="$HOME/uor-codex/20261118-value-connectors/inputs/seed$seed.d11"
    /usr/bin/time -f 'wall_s=%e peak_rss_kib=%M' -o "$out/s$seed-prepare.time" \
        "$probe" prepare "$root" "$out" "$trunk" "$seed"
    for count in 1 32; do
        dir="$out/s$seed-$count"
        manifest=$(sha256sum "$dir/manifest.json" | cut -d' ' -f1)
        flock "$lock" /usr/bin/time -f 'wall_s=%e peak_rss_kib=%M' -o "$dir/writer.time" \
            bash "$gate" gpu "$old" entity-writer-action "$dir/manifest.json" "$manifest" \
            "$trunk" "$dir/bare.json" "$dir/picks.json" "$dir/writer.json"
        for mode in 0 1; do
            /usr/bin/time -f 'wall_s=%e peak_rss_kib=%M' -o "$out/s$seed-m$mode-public-$count.time" \
                "$probe" public "$root" "$out" "$chat" "$seed" "$mode" "$count"
        done
        "$probe" verify "$out" "$seed" "$count"
    done
    "$probe" positive "$root" "$out" "$chat" "$seed"
done
(cd "$out"; find . -type f -regextype posix-extended -regex '.*\.(d11|json|time)' -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
