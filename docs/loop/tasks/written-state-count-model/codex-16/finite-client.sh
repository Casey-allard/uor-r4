#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261119-fresh-boundary-validity"
old=$HOME/uor-codex/20261118-ordered-role-transfer
out=${1:?fresh output directory}
mkdir "$out"
text='Aster kept cobalt. Dahlia kept ivory. Which item belongs to Aster?'
for seed in 1 2; do
    for read in enabled no_read; do
        "$old/native-finite/source/target/release/uor-chat" --discrete "$old/native-finite/positive/positive-s$seed-1.d11" --tokenizer "$old/assets/tokenizer-seed$seed.json" --say "$text" --protocol 2 --max-tokens 16 --temperature 0 --read-mode "$read" > "$out/seed$seed-$read.json"
    done
    jq -en --slurpfile on "$out/seed$seed-enabled.json" --slurpfile off "$out/seed$seed-no_read.json" '($on[0].selected.source==7 and $on[0].generated_ids==[3566,421,86,1] and ($on[0].text|ltrimstr(" "))=="cobalt" and $on[0].stop=="eos" and $off[0].selected.source==null and $off[0].generated_ids!=$on[0].generated_ids) or error("plant or disabled read failed")'
done
