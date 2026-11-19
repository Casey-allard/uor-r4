#!/usr/bin/env bash
# From a staged directory containing uor-chat, seed{1,2}-role.d11 and
# seed{1,2}-tokenizer.json. U points at the checked-in client evidence.
set -euo pipefail
: "${U:?directory containing requests.json and check-words.jq}"
out=${1:?new output directory}
mkdir "$out"
for n in 1 2; do
  for i in 0 1 2 3 4 5 6; do
    prompt=$(jq -er ".[$i].prompt" "$U/requests.json")
    for a in enabled no_read; do
      file="$out/seed$n-case$i-$a"
      for run in primary repeat; do
        ./uor-chat --discrete "seed$n-role.d11" --tokenizer "seed$n-tokenizer.json" \
          --say "$prompt" --read-mode "$a" --max-tokens 16 > "$file-$run.json"
      done
      cmp "$file-primary.json" "$file-repeat.json"
      jq -c --arg arm "$a" --argjson case "$i" --argjson seed "$n" \
        --slurpfile requests "$U/requests.json" --slurpfile reference "$U/wick-reference.jsonl" \
        --slurpfile pack "$U/seed$n-pack.json" -f "$U/check-words.jq" "$file-primary.json"
    done
  done
done
