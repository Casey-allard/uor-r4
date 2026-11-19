#!/usr/bin/env bash
# One fresh process per text question or device witness.
set -euo pipefail
: "${BIN:?uor-chat}" "${TOK:?bound tokenizer}" "${ROOT:?codex-11 root}" "${DATA:?D11 directory}" "${OUT:?fresh output directory}" "${BASE:?original trunk}"
mkdir "$OUT"
for arm in full pruned restored base; do
  model=$DATA/$arm.d11
  [[ $arm != base ]] || model=$BASE
  if [[ $arm != base ]]; then
    if [[ $arm == restored ]]; then
      jq '.score.examples' "$ROOT/equal-bytes/primary.json" > "$OUT/$arm-witness.json"
    else
      jq --arg name "$arm-fit" '.scores[]|select(.arm==$name)|.examples' "$ROOT/count/result.json" > "$OUT/$arm-witness.json"
    fi
    while read -r prev; do
      target="$OUT/$arm-$prev.json"
      "$BIN" --discrete "$model" --prompt-ids "55,$prev" --max-tokens 16 > "$target"
      jq -ce --argjson prev "$prev" --slurpfile ref "$OUT/$arm-witness.json" '
        . as $t | ($ref[0]|map(select(.previous==$prev))|first) as $r |
        if .generated_ids[0]!=$r.device_top1 or .generated_mass_q32[0]!=$r.winner_q32 then error("device mismatch") else
          [range(.generated_ids|length) as $i | (if $i==0 then $prev else $t.generated_ids[$i-1] end) as $p |
           ($ref[0]|map(select(.previous==$p))|first) as $w | select($w!=null) |
           if $t.generated_ids[$i]!=$w.device_top1 or $t.generated_mass_q32[$i]!=$w.winner_q32 then error("feedback mismatch") else $i end] as $checked |
           {previous:$prev,generated_ids,generated_mass_q32,device_steps:($checked|length)} end' "$target" >> "$OUT/$arm-raw.jsonl"
    done < <(jq -r 'map(.previous)|unique[]' "$OUT/$arm-witness.json")
  fi
  for i in 0 1 2 3; do
    prompt=$(jq -er ".[$i].prompt" "$ROOT/client/requests.json")
    file="$OUT/$arm-text$i.json"
    "$BIN" --discrete "$model" --tokenizer "$TOK" --say "$prompt" --max-tokens 16 > "$file"
    jq -c --slurpfile q "$ROOT/client/requests.json" --argjson i "$i" --arg arm "$arm" '
      . as $t|$q[0][$i] as $case |
      {arm:$arm,case:$i,prompt:$case.prompt,prompt_tail:.prompt_ids[-1],expected:$case.expected,generated_ids,text,stop,
       first_wrong:([range([(.generated_ids|length),($case.expected|length)]|max)|select($t.generated_ids[.]!=$case.expected[.])]|first),
       first_mass_q32:.generated_mass_q32[0],artifact_sha256}' "$file" >> "$OUT/text.jsonl"
  done
done
jq -se 'group_by(.arm)|all(.[];.[0].arm=="base" or ((map(.prompt_tail)|unique|length)==1 and (map(.generated_ids)|unique|length)==1))' "$OUT/text.jsonl" >/dev/null
cp "$TOK" "$OUT/altered-tokenizer.json"
printf ' ' >> "$OUT/altered-tokenizer.json"
if "$BIN" --discrete "$DATA/full.d11" --tokenizer "$OUT/altered-tokenizer.json" --say 'What is two plus two?' --max-tokens 1 > "$OUT/bad-tokenizer.log" 2>&1; then
  exit 1
fi
grep -F 'does not bind this tokenizer SHA-256' "$OUT/bad-tokenizer.log"
for a in full pruned restored; do
  jq -s --rawfile tsv "$ROOT/client/$a.tsv" -f "$ROOT/client/confirm-winners.jq" "$OUT/$a-raw.jsonl" > "$OUT/$a-device.json"
  cmp "$ROOT/client/$a-device.json" "$OUT/$a-device.json"
done
