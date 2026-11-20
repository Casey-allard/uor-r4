#!/usr/bin/env bash
# Summarize all sixteen separate untraced serving processes on the registered panel.
set -euo pipefail
run=$1
for profile in "$run"/timing/*.stderr; do
    stem=${profile%.stderr}
    name=${stem##*/}
    read -r seconds peak_kib < "$stem.resources"
    jq -ce --argjson id "${name%%-*}" --arg mode "${name#*-}" --argjson elapsed "$seconds" \
        --argjson peak "$peak_kib" --slurpfile response "$stem.json" '
        select(.schema=="uor-r4.p1-client-timing/1" and .traced==false and
               .tokens==($response[0].generated_ids|length) and .decode_ns>0 and .open_and_prefill_ns>0) |
        {id:$id,mode:$mode,process_seconds:$elapsed,peak_rss_kib:$peak,tokens,open_and_prefill_ns,decode_ns}' "$profile"
done | jq -se '
    def median: sort | . as $a | length as $n | ($a[($n-1)/2|floor]+$a[$n/2|floor])/2;
    if ((length!=16) or (map([.id,.mode])|unique|length)!=16 or (map(.mode)|sort|group_by(.)|map(length))!=[8,8])
    then error("missing or repeated untraced process measurements") else
    {scope:"Wick one-shot processes, OS page cache uncontrolled, decoding starts after load and prefill",
     raw:.,summary:[group_by(.mode)[] |
       {mode:.[0].mode,processes:length,tokens:(map(.tokens)|add),median_open_prefill_seconds:(map(.open_and_prefill_ns)|median/1e9),
        median_process_seconds:(map(.process_seconds)|median),max_rss_kib:(map(.peak_rss_kib)|max),
        decode_tokens_per_second:((map(.tokens)|add)*1e9/(map(.decode_ns)|add)),
        end_to_end_tokens_per_second:((map(.tokens)|add)/(map(.process_seconds)|add))}]} end'
