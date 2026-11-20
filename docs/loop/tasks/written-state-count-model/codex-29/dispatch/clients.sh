#!/bin/bash
set -euo pipefail
chat=${1:?uor-chat}; replay=${2:?d29-writer-replay}; tokenizer=${3:?tokenizer}; root=${4:?absolute run directory}
driver="$(dirname "$0")/../../codex-27/dispatch/client.sh"
mkdir "$root/clients-primary" "$root/clients-cold" "$root/grades"
for pass in primary cold; do
    for arm in baseline calibrated shuffled; do
        dest="$root/clients-$pass/$arm"
        "$driver" "$chat" "$root/carriers/$arm.d11" "$tokenizer" "$root/panel-all/held.blind.tsv" "$dest"
        if [[ "$pass" == primary ]]; then
            "$replay" "$root/carriers/$arm.d11" "$root/carriers/baseline.d11" "$root/inputs/fitted.bank" "$root/panel-all" "$dest" "$root/grades/$arm"
        else
            diff -qr "$dest" "$root/clients-primary/$arm"
        fi
    done
done
touch "$root/dispatch/clients-complete"
