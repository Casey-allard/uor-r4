#!/usr/bin/env bash
# Read the two native artifacts in ordinary text mode and compare raw device witnesses.
set -euo pipefail
: "${BIN:?}" "${TOK:?}" "${M:?model directory}" "${OUT:?fresh directory}" "${UNIT:?codex-12}"
mkdir "$OUT"
q=$UNIT/../codex-11/client/requests.json
for depth in 3 4; do
  arm=depth$depth-equal
  fraction=one-percent
  [[ $depth != 4 ]] || fraction=full-train
  model=$M/depth$depth.d11
  result=$UNIT/count/wick/$fraction/result.json
  wanted=$(jq -er --arg arm "$arm" '.arms[]|select(.arm==$arm)|.d11_sha256' "$result")
  for i in 0 1 2 3; do
    prompt=$(jq -er ".[$i].prompt" "$q")
    "$BIN" --discrete "$model" --tokenizer "$TOK" --say "$prompt" --max-tokens 16 > "$OUT/$depth-$i.json"
    jq -ce --arg sha "$wanted" --argjson i "$i" --arg arm "$arm" --slurpfile q "$q" '
      if .artifact_sha256!=$sha then error("wrong carrier") end | . as $t | $q[0][$i] as $g |
      {arm:$arm,case:$i,prompt:$g.prompt,shared_suffix_ids:.prompt_ids[-7:],expected:$g.expected,generated_ids,text,stop,
      first_wrong:([range([(.generated_ids|length),($g.expected|length)]|max)|select($t.generated_ids[.]!=$g.expected[.])]|first),first_mass_q32:.generated_mass_q32[0],artifact_sha256}' "$OUT/$depth-$i.json" >> "$OUT/text.jsonl"
  done
  jq -c --arg arm "$arm" '.arms[]|select(.arm==$arm)|.scores.examples[]|select(.recent[-1]==79)' "$result" > "$OUT/$depth-witness.jsonl"
  while read -r witness; do
    prefix=$(jq -r '.recent|join(",")' <<< "$witness")
    "$BIN" --discrete "$model" --prompt-ids "$prefix" --max-tokens 1 | jq -ce --arg sha "$wanted" --argjson r "$witness" '
        if .artifact_sha256!=$sha or .generated_ids!=[$r.winner] or .generated_mass_q32!=[$r.winner_q32] then error("device/runtime mismatch") end |
        {artifact_sha256,prompt_ids,generated_ids,generated_mass_q32,device_index:$r.index}' >> "$OUT/witnesses.jsonl"
  done < "$OUT/$depth-witness.jsonl"
done
