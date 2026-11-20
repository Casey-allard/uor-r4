#!/bin/bash
set -euo pipefail
q=${1:?absolute scratch}; chat=${2:?ordinary uor-chat}; tokenizer=${3:?protocol 2 tokenizer}
d26=${4:?frozen recurrent carrier}; d29=${5:?local reference carrier}
audit=${6:?d30-role-read}
test -s "$q/GO.json"
driver="$(dirname "$0")/../../codex-27/dispatch/client.sh"
cat "$q/panel/held.blind.tsv" "$q/panel/role.blind.tsv" > "$q/ordinary.blind.tsv"
test "$(wc -l < "$q/ordinary.blind.tsv")" -eq 41
for pass in ordinary ordinary-cold; do
    mkdir "$q/$pass"
    for arm in d26 d29; do
        model="$d26"; if [[ $arm == d29 ]]; then model="$d29"; fi
        "$driver" "$chat" "$model" "$tokenizer" "$q/ordinary.blind.tsv" "$q/$pass/$arm"
        if [[ $pass == ordinary ]]; then
            "$audit" "$model" "$q/panel/role.gold.json" "$q/$pass/$arm" "$q/role-$arm.json"
        else
            diff -qr "$q/ordinary/$arm" "$q/$pass/$arm"
        fi
    done
done
touch "$q/ordinary-complete"
