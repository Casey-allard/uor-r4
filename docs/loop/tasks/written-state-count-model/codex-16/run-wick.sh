#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261119-fresh-boundary-validity"
stage=${1:?primary or repeat}
seed=${2:?1 or 2}
[[ $stage == primary || $stage == repeat ]]
[[ $seed == 1 || $seed == 2 ]]
sha256sum --status -c driver/inputs.sha256
old=$HOME/uor-codex/20261118-ordered-role-transfer
(cd "$old" && sha256sum --status -c driver/oracle-pins.sha256)
(cd "$old" && sha256sum --status -c ../20261118-paired-role-fence/fence/identity/INPUTS.sha256)
m=$old/assets/seed$seed-paired.d11
tok=$old/assets/tokenizer-seed$seed.json
w=$old/bins/writer-c
chat=$old/native-finite/source/target/release/uor-chat
labels=$PWD/grading/held-grades.json
check=$PWD/driver/fresh-validity-check
reader=$PWD/grading/blind-semantic-v2.json
out=$PWD/$stage-seed$seed
mkdir "$out" "$out/blind"
cp panel/held-blind.json "$out/blind/held-blind.json"
blind=$out/blind/held-blind.json
bash driver/finite-client.sh "$out/client-calibration" > "$out/finite-calibration-pass.txt"
# This sibling is absent in primary and intentionally wrong in repeat. The
# public and no-read processes see the same complete unmarked prompts.
if [[ $stage == repeat ]]; then cp grading/mutated.json "$out/blind/held-grades.json"; fi
jq '[.[]|{start:0,end:(.prefix|length)}]' "$blind" > "$out/all-original-positions.json"
"$check" "$tok" "$blind" "$labels" "$reader" "$out/independent-preflight.json"
"$w" select "$m" "$blind" public - - "$out/public.json"
"$w" select "$m" "$blind" no-read - - "$out/no-read.json"
"$w" select "$m" "$blind" oracle "$out/all-original-positions.json" "$labels" "$out/requested.json"
jq -en --slurpfile p "$out/public.json" --slurpfile r "$out/requested.json" 'all($r[0].rows[]; . as $x | ($p[0].rows[$x.index].paired.role_words[2:] == $x.paired.role_words[2:]) and $x.original_role_words==$p[0].rows[$x.index].paired.role_words)' >/dev/null
# First exercise the two writer intervention controls through actual GPU
# autoregression. The label file hands over only a start; no end/IDs/EOS.
force() {
    local arm=$1
    flock "$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock" bash "$HOME/uor-codex/20261118-query-region-pick/writer/run-wick.sh" gpu "$w" replay "$HOME/uor-codex/20261118-role-boundary-count/bin/action-writer" "$HOME/uor-codex/20261118-value-connectors/inputs/seed$seed.d11" "$blind" "$out/$arm-run" "$out/$arm.json"
    mkdir "$out/$arm-run/nominees"
    ln "$out/$arm-run/gpu.json" "$out/$arm-run/nominees/chosen.json"
    "$w" grade "$out/$arm.json" "$out/$arm-run" "$labels" - "$out/$arm-grade.json"
}
for arm in forced-gold forced-rival; do
    jq -ecn --arg arm "$arm" --slurpfile original "$out/requested.json" --slurpfile labels "$labels" -f driver/sources.jq > "$out/$arm.json"
    force "$arm"
    jq -en --arg arm "$arm" --slurpfile labels "$labels" --slurpfile grade "$out/$arm-grade.json" --slurpfile trajectory "$out/$arm-run/gpu.json" -f driver/writer-validity.jq >"$out/$arm-cursor.json"
done
jq -e '(.rows|length)==24 and all(.rows[];.exact==true and .terminal_eos and .source_matches_requested_value)' "$out/forced-gold-grade.json" >/dev/null
jq -e '(.rows|length)==24 and all(.rows[];.exact==false)' "$out/forced-rival-grade.json" >/dev/null
"$w" grade "$out/forced-gold.json" "$out/forced-gold-run" grading/mutated.json - "$out/wrong-key-grade.json"
jq -e '(.rows|length)==24 and all(.rows[];.exact==false)' "$out/wrong-key-grade.json" >/dev/null
"$check" observed "$tok" "$labels" "$reader" - "$out/forced-gold-grade.json" "$out/forced-gold-independent.json"
"$check" observed "$tok" "$labels" "$reader" - "$out/forced-rival-grade.json" "$out/forced-rival-independent.json"
"$check" observed "$tok" grading/mutated.json "$reader" "$labels" "$out/wrong-key-grade.json" "$out/wrong-key-independent.json"
jq -ens --slurpfile g "$out/forced-gold-independent.json" --slurpfile m "$out/wrong-key-independent.json" '($g[0].counts.independently_exact==24 and $m[0].counts.active_label_exact==0 and $m[0].counts.independently_exact==24 and $m[0].generated_id_tape_sha256==$g[0].generated_id_tape_sha256 and $g[0].gpu_sha256==$m[0].gpu_sha256)' >/dev/null
"$w" chain "$HOME/uor-codex" "$seed" "$blind" "$out/chain" "$out/public.json" "$out/no-read.json" "$out/requested.json"
jq -ecs --arg arm word-start --slurpfile original "$out/requested.json" --slurpfile labels /dev/null -f driver/sources.jq "$out"/chain/level-*-chunk-*/nominees/rank-*-cont.json > "$out/word-start.json"
force word-start
"$w" grade "$out/requested.json" "$out/chain" "$labels" - "$out/requested-grade.json"
for arm in public no-read; do
    "$w" public "$chat" "$m" "$tok" "$blind" "$arm" "$out/say-$arm"
    "$w" grade "$out/$arm.json" "$out/chain" "$labels" "$out/say-$arm" "$out/$arm-grade.json"
done
for arm in word-start requested public no-read; do
    "$check" observed "$tok" "$labels" "$reader" - "$out/$arm-grade.json" "$out/$arm-independent.json"
done
(cd "$out" && sha256sum *-independent.json independent-preflight.json *-grade.json public.json no-read.json requested.json word-start.json chain/nominees/chosen.json *-cursor.json client-calibration/*.json *-run/gpu.json say-{public,no-read}/*.json > cold.sha256)
