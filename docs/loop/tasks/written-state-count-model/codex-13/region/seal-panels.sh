#!/usr/bin/env bash
set -euo pipefail
# Run from the isolated Wick root with both tokenizer regenerations available.
[[ $# == 2 ]]
a=$1
b=$2
p=docs/loop/tasks/written-state-count-model/codex-13/region
test ! -e panels/collection.sha256
(cd panels; sha256sum -c SHA256SUMS)
(cd baseline; sha256sum -c retained.sha256)
sha256sum -c frozen-binaries-inputs.sha256
diff -qr "$a" "$b"
for split in train held fresh; do
    for part in blind grade; do
        cmp "panels/$split-$part.json" "$a/$split-$part.json"
    done
done
for split in held fresh nuisance; do
    cp -n "$a/$split-regions.json" panels/
    if [[ $split == nuisance ]]; then
        cp -n "$a/$split-blind.json" "$a/$split-grade.json" panels/
    fi
    jq -e -s --arg split "$split" '
      (if $split=="nuisance" then 12 else 1056 end) as $n |
      all(.[]; length == $n) and (
      .[0] as $regions | .[1] as $grade | .[2] as $blind |
      all(range($n); . as $i |
        $regions[$i] as $r | $grade[$i] as $g |
        $r.prefix==$blind[$i].prefix and $r.prefix==$g.prefix
        and $r.prefix[:4]==[0,55,2728,28] and
        $r.start>=4 and $r.start<$r.end and $r.end<($r.prefix|length) and
        all(["query","excluded"][]; . as $role |
          $g[$role+"_start"] as $a | $g[$role+"_end"] as $b |
          ($a==null and $b==null) or ($r.start<=$a and $a<$b and $b<=$r.end))
      ))' panels/"$split"-{regions,grade,blind}.json >/dev/null
    jq -c '[.[] | {prefix, end:.start, start:([0, 2*.start-.end] | max)}]' \
        "panels/$split-regions.json" > "panels/$split-adverse.json"
done
for seed in 1 2; do
    for split in held fresh; do
        jq -c -s --arg split "$split" --argjson seed "$seed" '
          .[0].rows as $original | .[1] as $region | .[2] as $grade |
          [$grade | to_entries[] | .key as $i | .value as $g |
           $original[$i] as $o | $region[$i] as $r |
           select($g.query_start != null and $o.query_start != null and
                  $o.query_start >= 4 and $o.query_end <= $r.start) |
           {seed:$seed,split:$split,row:$i,id:$g.id,prefix_sha256:$o.prefix_sha256,
            prior:[$o.query_start,$o.query_end],query:[$g.query_start,$g.query_end],
            region:[$r.start,$r.end]}]' \
            "baseline/seed$seed-$split.json" "panels/$split-regions.json" \
            "panels/$split-grade.json" > "panels/seed$seed-$split-intrusions.json"
    done
done
{
    sha256sum "$p"/{SHEET.md,FIT.md,seal-panels.sh,run-wick.sh,device.cu,driver.rs,Cargo.lock}
    sha256sum crates/uor-r4-integer/src/paired_role.rs
    sha256sum crates/uor-r4-tokenizer/examples/span-connector-panel{.rs,/paired_role.rs,/action.rs,/support.rs}
    sha256sum generator-target/release/examples/span-connector-panel
    sha256sum bin/* baseline/retained.sha256 frozen-binaries-inputs.sha256 panels/*.json
} > panels/collection.sha256
