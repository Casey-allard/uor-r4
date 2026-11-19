#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
region=../region
out=${1:?fresh verification directory}
mkdir "$out"
nvidia-smi --query-gpu=name,uuid,driver_version --format=csv,noheader > "$out/device.txt"
sha256sum bin/query-region-writer > "$out/instrument.txt"
for split in held fresh nuisance; do
    jq -en --slurpfile a "$region/panels/$split-blind.json" --slurpfile g "$region/panels/$split-grade.json" \
        '$a[0]==[$g[0][]|{prefix}]' >/dev/null
    for seed in 1 2; do
        tag=seed$seed-$split
        for stage in primary repeat; do
            (cd "$stage/$tag"; sha256sum --status -c SHA256SUMS)
        done
        cmp "primary/$tag/SHA256SUMS" "repeat/$tag/SHA256SUMS"
        jq -c --arg tag "$tag" --slurpfile wa "primary/$tag/original/work.json" \
            --slurpfile wi "primary/$tag/identity/work.json" --slurpfile wr "primary/$tag/region/work.json" \
            --slurpfile wd "primary/$tag/adverse/work.json" --slurpfile wo "primary/$tag/role-oracle/work.json" \
            --slurpfile identity "primary/$tag/identity.json" '
            def device:
              {observed_trajectories:(.actual_finals + ([.ranks[].actual] | add // 0)),
               device_feedback_steps:(.actual_final_steps + ([.ranks[].actual_steps] | add // 0)),
               rank_uses:([.ranks[].uses] | add // 0)};
            def armnames: ["original","region","adverse","role_oracle"];
            def scored($arm):
              {rows:length,exact:(map(select(.[$arm].exact))|length),
               unsupported_copy:(map(select(.expected==null and .[$arm].copied))|length)};
            def arms: . as $d | [armnames[] as $arm | {key:$arm,value:($d|scored($arm))}] | from_entries;
            {tag:$tag,overall,identity:$identity[0],
             direction:([.rows[]|. + {direction:(.id|split("-")|map(select(.=="0" or .=="1"))|last)}]
                        |group_by(.direction)|map({key:(.[0].direction // "other"), value:arms})|from_entries),
             pair_exact:([armnames[] as $arm | {key:$arm,value:(.rows|map(select(.expected!=null))|
                         group_by([.pair,(.id|split("-")|last|select(.=="plain" or .=="trailing"))])|
                         map(select(length==2 and all(.[]; .[$arm].exact)))|length)}]|from_entries),
             device:
            {original:($wa[0]|device),identity:($wi[0]|device),region:($wr[0]|device),
             adverse:($wd[0]|device),role_oracle:($wo[0]|device)}}' "primary/$tag/grade.json" > "$out/$tag.json"
        if [[ $split != nuisance ]]; then
            jq -c --arg tag "$tag" --slurpfile cohort "$region/panels/$tag-intrusions.json" \
                --slurpfile role "$region/primary/$tag-region.json" '
                .rows as $rows | [$cohort[0][] as $c | $rows[$c.row] as $r |
                 if $r.id != $c.id or $r.prefix_sha256 != $c.prefix_sha256 then error("intrusion row binding") else
                   {id:$r.id,expected:$r.expected,before:$c.prior,expected_request:$c.query,
                    actual_request:([$role[0].rows[$c.row]|.query_start,.query_end]),
                    original:$r.original,region:$r.region,role_oracle:$r.role_oracle}
                 end] as $r |
                {tag:$tag,rows:($r|length),correct_endpoint:($r|map(select(.actual_request==.expected_request))|length),
                 exact_reply_recovered:($r|map(select(.region.exact and (.original.exact|not)))|length),
                 unsupported_new_copies:($r|map(select(.expected==null and .region.copied and (.original.copied|not)))|length),
                 examples:([$r[]|select(.region.exact and (.original.exact|not))]|.[0:2])}' \
                "primary/$tag/grade.json" > "$out/$tag-intrusions.json"
        fi
        jq -c --arg tag "$tag" --slurpfile grades "$region/panels/$split-grade.json" '
          .rows as $rows |
          [range($rows|length) as $i | $rows[$i] as $r |
            if $r.expected==null and ($r.original.copied|not) and ($r.original.ids|length)>1 then
              {id:$r.id,prompt:$grades[0][$i].prompt,original:$r.original,query_region:$r.region,role_oracle:$r.role_oracle}
            else empty end][0] as $abstention |
          [$rows[]|select(.region.exact and (.original.exact|not))][0] as $repair |
          {tag:$tag,abstention:$abstention,repair:$repair}' "primary/$tag/grade.json" > "$out/$tag-examples.json"
    done
done
(cd "$out"; sha256sum *.json > SHA256SUMS)
