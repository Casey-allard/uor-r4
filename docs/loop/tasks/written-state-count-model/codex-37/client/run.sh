#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/run.sh"
cd "$stage"
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key]|strings|select(length>0)' GO.json; }
check GO.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 ordinary D11 closure'
for file in run.sh SHEET.md FIT.md stage.sha256; do
    key=${file%%.*}
    check "$file" "$(field "${key,,}_sha256")"
done
sha256sum --strict -c stage.sha256
sha256sum --quiet --strict -c source.sha256

runs=/mnt/plexpool/uor-r4-corpus/runs
root="$runs/20261120-pruned-r-fit-37"
test "$stage" = "$root/client"
fit=$(field fit_result)
test "$(dirname "$fit")" = "$root/fit"
[[ "$(basename "$fit")" =~ ^real-[a-z0-9-]+$ ]]
check "$fit/results.sha256" "$(field fit_result_sha256)"
(cd "$fit" && sha256sum --strict -c results.sha256)
test "$(jq -er .pick_mode "$fit/GO.json")" = identity

trunk="$runs/20261120-complete-c1-31/normal-5ea09ea47/fixtures/seed1.d11"
tokenizer="$runs/20261120-complete-c1-31/normal-5ea09ea47/fixtures/tokenizer-seed1.json"
counts="$runs/20261120-whole-context-36/pruned-full-v1/compact.counts"
check "$tokenizer" a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377
output=$(field output)
test "$(dirname "$output")" = "$stage"
[[ "$(basename "$output")" =~ ^result-[a-z0-9-]+$ ]]
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 8388608
test "$(df --output=avail -B1 "$stage" | awk 'NR==2 {print $1}')" -ge 10737418240
mkdir "$output"
cp GO.json SHEET.md FIT.md prompts.tsv stage.sha256 "$output/"
export CUDA_VISIBLE_DEVICES=

bound='def bound($b):
    .artifact_sha256==$b.artifact_sha256 and .p1_trunk_sha256==$b.trunk_sha256
    and .p1_state_sha256==$b.state_sha256 and .p1_components==$b.components
    and .read_enabled==true;'
for arm in a1 a1s; do
    dir="$output/$arm"
    "$stage/d37-client-check" "$fit/identity" "$arm" "$trunk" "$counts" "$tokenizer" "$dir"
    artifact="$dir/artifact.d11"
    readback="$dir/readback.json"
    mkdir "$dir/ids" "$dir/text"
    # Choose one distinct substantive prefix from each of the first 64 records.
    # Supplement it with explicit causal gates and both batch-transition probes.
    jq -ce '
        .probes as $p |
        (reduce range(1;65) as $record ([]; . as $chosen |
            ($p | map(select(.record==$record and (.prefix|length)>=8)) |
                sort_by(.step.context_tokens) | reverse |
                map(select(.prefix as $prefix | all($chosen[]; .prefix!=$prefix))) |
                first // error("missing distinct causal probe for original record \($record)")) as $probe |
            . + [$probe])) as $panel |
        ($panel + [$p[]|select(.target==1)][0:1]
        + [$p[]|select(.step.context_tokens==192)][0:1]
        + ($p|group_by(.read_class)|map(first))
        + [$p[]|select(.ordinal==4096 or .ordinal==65536)] + $p[-2:]) |
        unique_by(.ordinal)[]' "$readback" > "$dir/client-probes.jsonl"
    jq -se '
        {probes:length, original_records:(map(.record)|unique|length),
           distinct_nonempty_prefixes:(map(.prefix|select(length>0))|unique|length)} |
        select(.original_records>=64 and .distinct_nonempty_prefixes>=64)
        ' "$dir/client-probes.jsonl" > "$dir/client-coverage.json"
    while IFS=$'\t' read -r name expected prompt; do
        actual="$dir/text/$name.json"
        direct="$dir/text/$name-ids.json"
        options=(--discrete "$artifact" --temperature 0 --max-tokens 12 --read-mode enabled --p1-trace)
        "$stage/uor-chat" "${options[@]}" --tokenizer "$tokenizer" --protocol 2 --say "$prompt" > "$actual"
        ids=$(jq -er '.prompt_ids|join(",")' "$actual")
        "$stage/uor-chat" "${options[@]}" --prompt-ids "$ids" > "$direct"
        jq -e --slurpfile readback "$readback" --slurpfile direct "$direct" "$bound
            bound(\$readback[0]) and (\$direct[0]|bound(\$readback[0]))
            and .prompt_ids==\$direct[0].prompt_ids and .generated_ids==\$direct[0].generated_ids
            and .p1_steps==\$direct[0].p1_steps and .generated_ids==[.p1_steps[].token]
            and ((.generated_ids|length)==12 or .generated_ids[-1]==1)
            and (.generated_ids[:-1]|index(1)==null)
        " "$actual" >/dev/null
        gold=$("$stage/d37-client-check" gold "$tokenizer" "$expected")
        jq -c --arg id "$name" --arg gold "$expected" --arg prompt "$prompt" --argjson key "$gold" '
            .generated_ids as $actual |
            (([($actual|length),($key.ids|length)]|max) as $n |
                first(range(0;$n) | select($actual[.] != $key.ids[.])) // null) as $wrong |
            {case:$id,prompt:$prompt,expected:$gold,output:.text,generated_ids, eos:(.generated_ids[-1]==1),
              expected_ids:$key.ids,expected_eos:$key.eos,tokenizer_sha256:$key.tokenizer_sha256,
              exact_ids_and_eos:($actual==$key.ids),
              first_wrong:(if $wrong==null then null else {
                 index:$wrong,expected_id:$key.ids[$wrong],actual_id:$actual[$wrong],
                 missing_output:($actual[$wrong]==null)
              } end),
              exact_word:(.text|gsub("^\\s+|\\s+$";"") == $gold)} |
            . + {exact_word_and_eos:(.exact_word and .eos)}' "$actual"
    done < prompts.tsv > "$dir/text/grades.jsonl"
    jq -se 'length==2 and .[0].prompt_ids!=.[1].prompt_ids' \
        "$dir/text/copper.json" "$dir/text/silver.json" >/dev/null
    (cd "$dir/text" && sha256sum *.json grades.jsonl > results.sha256)
done

for arm in a1 a1s; do
    dir="$output/$arm"
    artifact="$dir/artifact.d11"
    readback="$dir/readback.json"
    while read -r probe; do
        ordinal=$(jq -er .ordinal <<< "$probe")
        ids=$(jq -er '.prefix|join(",")' <<< "$probe")
        actual="$dir/ids/$ordinal.json"
        "$stage/uor-chat" --discrete "$artifact" --prompt-ids "$ids" --temperature 0 \
            --max-tokens 1 --read-mode enabled --p1-trace > "$actual"
        jq -e --argjson probe "$probe" --slurpfile readback "$readback" "$bound
            bound(\$readback[0]) and .prompt_ids==\$probe.prefix
            and .generated_ids==[\$probe.step.token] and .p1_steps==[\$probe.step]
        " "$actual" >/dev/null || { echo "$arm ordinary replay differs at original ordinal $ordinal" >&2; exit 1; }
    done < "$dir/client-probes.jsonl"
    (cd "$dir" && sha256sum artifact.d11 readback.json client-probes.jsonl client-coverage.json ids/*.json text/* > results.sha256)
done
(cd "$output" && sha256sum GO.json SHEET.md FIT.md a1/results.sha256 a1s/results.sha256 > results.sha256)
