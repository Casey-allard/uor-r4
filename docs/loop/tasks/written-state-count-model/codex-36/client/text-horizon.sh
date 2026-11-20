#!/usr/bin/env bash
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
sheet=$(realpath "$(dirname "$0")/HORIZON.md")
client=$(realpath "$1")
d11=$(realpath "$2")
tokenizer=$(realpath "$3")
mkdir "$4"
cd "$4"
sha256sum "$sheet" "$client" "$d11" "$tokenizer" > inputs.sha256
mapfile -t texts < <(awk -F' \\| ' '/^\| (Distant|Near) /{sub(/ \|$/,"",$2); print $2}' "$sheet")
test "${#texts[@]}" -eq 4
for i in "${!texts[@]}"; do
    options=(--discrete "$d11" --temperature 0 --max-tokens 1 --read-mode enabled --p1-trace)
    "$client" "${options[@]}" --tokenizer "$tokenizer" --protocol 2 --say "${texts[i]}" > "$i-text.json"
    ids=$(jq -er '.prompt_ids|select(length>=10)|join(",")' "$i-text.json")
    "$client" "${options[@]}" --prompt-ids "$ids" > "$i-ids.json"
    jq -e --slurpfile by_ids "$i-ids.json" '
      (.p1_steps|length) == 1 and (.generated_ids|length) == 1 and
      .prompt_ids == $by_ids[0].prompt_ids and .generated_ids == $by_ids[0].generated_ids and
      .p1_steps == $by_ids[0].p1_steps
    ' "$i-text.json" > /dev/null
done
jq -se '
  def compare($a;$b):
    {different_prompts: ($a.prompt_ids != $b.prompt_ids),
     suffixes: [$a.prompt_ids[-10:],$b.prompt_ids[-10:]],
     same_suffix: ($a.prompt_ids[-10:] == $b.prompt_ids[-10:]),
     same_mass_vector: ($a.p1_steps[0].mass_sha256 == $b.p1_steps[0].mass_sha256),
     mass_sha256: [$a.p1_steps[0].mass_sha256,$b.p1_steps[0].mass_sha256],
     deepest_present_order: [($a,$b) | (.p1_steps[0].bucket % 176 / 16 | floor)],
     winners: [$a.generated_ids[0],$b.generated_ids[0]],
     text: [$a.text,$b.text],
     maxima: [$a.p1_steps[0].mass_q48,$b.p1_steps[0].mass_q48],
     ties: [$a.p1_steps[0].top_ties,$b.p1_steps[0].top_ties]};
  {distant:compare(.[0];.[1]),near:compare(.[2];.[3])} |
  if (.distant.different_prompts and .near.different_prompts and .distant.same_suffix and
      .distant.same_mass_vector and (.near.same_suffix|not) and
      (.distant|.winners[0]==.winners[1] and .maxima[0]==.maxima[1] and .ties[0]==.ties[1]))
  then . else error("text horizon or positive-control premise failed: \(.)") end
' {0,1,2,3}-text.json > result.json
sha256sum ./*.json > results.sha256
