#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd -P)
[[ $# == 1 || $# == 2 ]]
mkdir "$1"
cd "$1"
for ((i=0; i<94; i++)); do
    printf '000000000a0000000b0000000c0000000d0000000e0000000f000000%02x000000' "$((16+i))" |
        xxd -r -p | shasum -a 256 | cut -d' ' -f1
done | jq -Rs 'split("\n")[:-1]' > hashes.json
jq -n --slurpfile hashes hashes.json -f "$here/finite.jq" > fixtures.json
for key in fg extra hg fb hb; do jq ".$key" fixtures.json > "$key"; done
for split in f h; do
    hash=$(shasum -a 256 "${split}b" | cut -d' ' -f1)
    jq --arg hash "$hash" ".${split}c | .blind_sha256=\$hash" fixtures.json > "${split}c"
done
check() { "$here/check-inputs.sh" fc "$1" fb hb fg extra hg "$2"; }
check hc pass.json
jq -e '.matched.confirmed_structural_ceiling==8 and .fit.original_literal_reachable==88' pass.json
# One changed candidate class breaks the signature equality only.
jq '.rows[0].assertions[0].legacy_candidates[0].slot_classes[0]=2' hc > shifted
if check shifted shifted.json; then exit 1; fi
jq -e '.matched.groups[0].distinct_structures==2 and .held.source_in_rank_domain==36' shifted.json
# Gold still writes from the original tape, but is absent from the ranked set.
jq '.rows[0].assertions[0].legacy_candidates[0].writer_reachable=false' hc > missing
if check missing missing.json; then exit 1; fi
jq -e '.held.original_literal_reachable==36 and .held.source_in_rank_domain==35' missing.json
if [[ $# == 2 ]]; then
    args=()
    for key in fb hb fc hc fg hg extra checker query binder; do
        case "$key" in
            checker) p="$here/check-inputs.sh";;
            query) p="$here/input.jq";;
            binder) p="$2";;
            *) p="$PWD/$key";;
        esac
        sha=$(shasum -a 256 "$p" | cut -d' ' -f1)
        args+=(--argjson "$key" "$(jq -n --arg path "$p" --arg sha256 "$sha" '{path:$path,sha256:$sha256}')")
    done
    jq -n "${args[@]}" '{schema:"uor-r4.rank-validity-inputs/1",files:$ARGS.named}' > manifest
    seal=$(shasum -a 256 manifest | cut -d' ' -f1)
    "$here/check-pinned.sh" manifest "$seal" "$PWD/qualified.json"
    cmp pass.json qualified.json
    jq '.[0].prefix[6]+=1' hb > hb-changed
    rebased=$(shasum -a 256 hb-changed | cut -d' ' -f1)
    jq --arg sha "$rebased" '.blind_sha256=$sha' hc > hc-rebased
    if "$2" verify-input-rows hb-changed hc-rebased hg -; then exit 1; fi
    jq '.[41].id=null' hg > hg-bad
    if "$2" verify-input-rows hb hc hg-bad -; then exit 1; fi
    # Change two labels to different live candidates, preserving both original
    # tapes and the multiset of target offsets; the external key pin must fail.
    jq '.[0].gold_start=3 | .[0].expected=[12,13,1] |
        .[1].gold_start=2 | .[1].expected=[11,12,13,1]' hg > switched-key
    cp switched-key hg
    if "$here/check-pinned.sh" manifest "$seal" "$PWD/false-result.json"; then exit 1; fi
    test ! -e false-result.json
fi
