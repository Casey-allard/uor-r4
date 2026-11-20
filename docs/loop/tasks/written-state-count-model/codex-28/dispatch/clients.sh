#!/bin/bash
# All six clients receive the identical unannotated text through protocol 2.
set -euo pipefail
chat=${1:?uor-chat}; verifier=${2:?d28-value-replay}; token=${3:?tokenizer}
models=${4:?carrier directory}; panel=${5:?final panel}; work=${6:?run root}
driver="$(dirname "$0")/../../codex-27/dispatch/client.sh"
mkdir -p "$work/clients-primary" "$work/clients-cold" "$work/grades"
for pass in primary cold; do
    for policy in frozen fitted augmented; do
        for enumeration in legacy whole; do
            arm="$policy-$enumeration"
            "$driver" "$chat" "$models/$arm.d11" "$token" "$panel/held.blind.tsv" "$work/clients-$pass/$arm"
            if [[ "$pass" == primary ]]; then
                "$verifier" grade "$models/$arm.d11" "$panel" "$work/clients-primary/$arm" "$work/grades/$arm"
            else
                diff -qr "$work/clients-primary/$arm" "$work/clients-cold/$arm"
            fi
        done
    done
done
