#!/usr/bin/env bash
set -euo pipefail
top=$HOME/uor-codex
cd "$top/20261118-paired-role-fence/fence"
p=$top/20261118-content-support
old=$top/20261118-role-boundary-count
lab=tree/docs/loop/tasks/written-state-count-model/codex-12
test -s "$lab/FENCE-FIT.md"
stage=${1:?primary or repeat}
case "$stage" in primary|repeat) ;; *) exit 2 ;; esac
domain=${2:?historical, fresh-oracle, prior-nominee or paired-role}
fence=${FENCE_EXEC:-bin/assertion-fence-final}
mkdir -p "results/$stage"
out=$PWD/results/$stage/$domain
mkdir "$out"
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
device() {
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$@"
    flock -u 8
}
writer() {
    local trace=$1 dst=$2
    jq -ce '.arm="action" | .rows=[.rows[] | {prefix_sha256,continuation}]' "$trace" > "$dst-picks.json"
    jq -nc --arg m "$(sha256sum "$model"|cut -d' ' -f1)" --arg b "$(sha256sum "$bare"|cut -d' ' -f1)" \
        --arg p "$(sha256sum "$dst-picks.json"|cut -d' ' -f1)" '[{model:$m,bare:$b,picks:$p}]' > "$dst-manifest.json"
    local sum
    sum=$(sha256sum "$dst-manifest.json"|cut -d' ' -f1)
    device "$old/bin/action-writer" entity-writer-action "$dst-manifest.json" "$sum" \
        "$model" "$bare" "$dst-picks.json" "$dst-writer.json"
}
select_new() {
    device "$fence" apply bin/fence-device "$model" "$span" "$exact" "$bare" "$1" "$2"
}
historical() {
    for split in held unseen transfer; do
        case "$split" in
            held) bare=$p/panels/held-blind.json ;;
            unseen) bare=$old/panels/unseen-blind.json ;;
            transfer) bare=$top/20261118-continuation-eos/panels/transfer-blind.json ;;
        esac
        for arm in query both; do
            local base=$out/seed$n-$split-$arm
            select_new "$p/nominee/results/seed$n-$split/$arm-continuation.json" "$base-trace.json"
            writer "$base-trace.json" "$base"
        done
    done
    bare=$p/panels/held-blind.json
    for arm in query-agree agree; do
        local previous=$p/nominee/agreement/results/seed$n-held-$arm
        local consensus=$out/seed$n-held-$arm
        mkdir "$consensus"
        cp "$previous/sites.json" "$consensus/sites.json"
        for k in $(seq 0 $(($(jq -r .max "$previous/sites.json")-1))); do
            local base=$consensus/rank-$k
            select_new "$previous/rank-$k-cont.json" "$base-cont.json"
            writer "$base-cont.json" "$base"
        done
        local request=query
        [[ $arm != agree ]] || request=both
        device "$p/nominee/agreement/bin/agree" adjudicate "$p/nominee/agreement/bin/agree-device" \
            "$bare" "$p/nominee/results/seed$n-held-$request.json" "$consensus" "$arm" "$consensus/selection.json"
        jq -ce '.arm="action" | .schema="uor-r4.entity-pair-chain/1" | .rows=[.rows[]|{prefix_sha256,continuation:{source,reason:0,connector_mask:0,onset_mask:0}}]' \
            "$consensus/selection.json" > "$consensus/trace.json"
        writer "$consensus/trace.json" "$consensus"
    done
}
fresh_oracle() {
    local pref=$top/20261118-paired-role-fence/role/panels/fresh
    bare=$pref-blind.json
    local base=$out/seed$n-fresh-oracle
    jq -en --slurpfile b "$bare" --slurpfile g "$pref-grade.json" '$b[0]==[$g[0][]|{prefix}]' >/dev/null
    local bank
    bank=$(sha256sum "$span"|cut -d' ' -f1)
    local digest
    digest=$(sha256sum "$bare"|cut -d' ' -f1)
    local art
    art=$(sha256sum "$model"|cut -d' ' -f1)
    # The historical blind request contract binds packed prefix hashes. The grade
    # projection can pass only query and fact coordinates, never a value address.
    "$fence" anchors "$bare" "$pref-grade.json" "$art" "$bank" "$base-oracle.json"
    device "$old/bin/chain" cross role "$model" "$content" "$span" "$exact" "$bare" "$base-oracle.json" "$base-old.json"
    select_new "$base-old.json" "$base-new.json"
    writer "$base-old.json" "$base-old"
    writer "$base-new.json" "$base-new"
}
fresh_nominee() {
    local new=$top/20261118-paired-role-fence
    local panels=$new/role/panels
    local controls=$new/writer/controls
    local occurrence=$p/nominee/bin
    local agree=$p/nominee/agreement/bin
    local learned=$p/nominee/results/seed$n-bank.json
    local token=$top/20261118-value-connectors/inputs/tokenizer.json
    [[ $n == 1 ]] || token=$top/20261118-value-connectors/inputs/tokenizer-seed-2.json
    test -s "$controls/frozen.sha256"
    for split in held fresh; do
        bare=$panels/$split-blind.json
        local tag=seed$n-$split
        local base=$out/$tag
        mkdir "$base"
        local prev=$base/nominee.json
        if [[ "$domain" == prior-nominee ]]; then
            device "$occurrence/occurrence" blind "$occurrence/occurrence-device" "$model" "$span" \
                "$old/results/fit/seed$n.json" "$token" "$old/results/action-fit/seed$n.json" \
                "$bare" "$controls/role/$tag.json" "$learned" - "$base"
            jq -ce '{arm:"both",artifact_sha256:.artifact,bare_sha256:.bare,state_sha256:.bank,rows}' \
                "$base/both-diagnostic.json" > "$prev"
        else
            local request_dir=${3:?missing paired-role handoffs}
            "$fence" nominate "$bare" "$request_dir/$tag.json" "$span" "$learned" "$prev"
        fi
        local past=$base/old future=$base/fenced
        mkdir "$past" "$future"
        device "$agree/agree" enumerate "$agree/agree-device" "$span" "$learned" "$bare" "$prev" "$past"
        cp "$past/sites.json" "$future/sites.json"
        local max
        max=$(jq -r .max "$past/sites.json")
        for ((k=0;k<max;k++)); do
            device "$old/bin/chain" cross role "$model" "$content" "$span" "$exact" \
                "$bare" "$past/rank-$k-override.json" "$past/rank-$k-cont.json"
            select_new "$past/rank-$k-cont.json" "$future/rank-$k-cont.json"
            writer "$past/rank-$k-cont.json" "$past/rank-$k"
            writer "$future/rank-$k-cont.json" "$future/rank-$k"
        done
        for dir in "$past" "$future"; do
            device "$agree/agree" adjudicate "$agree/agree-device" "$bare" "$prev" "$dir" "agree" "$dir/selection.json"
            jq -ce '.arm="action" | .schema="uor-r4.entity-pair-chain/1" |
                .rows=[.rows[]|{prefix_sha256,continuation:{source,reason:0,connector_mask:0,onset_mask:0}}]' \
                "$dir/selection.json" > "$dir/final-trace.json"
        done
    done
}
for n in 1 2; do
    model=$top/20261118-value-connectors/inputs/seed$n.d11
    span=$top/20261118-span-connectors/results/span-01/seed$n-state.json
    exact=$top/20261118-value-connectors/results/value-01/seed$n-state.json
    content=$top/20261118-content-selector/results/selector-run-01/seed$n/trained.json
    case "$domain" in
        historical) historical ;;
        fresh-oracle) fresh_oracle ;;
        prior-nominee|paired-role) fresh_nominee "$@" ;;
        *) exit 2 ;;
    esac
done
(cd "$out"; find . -type f -name '*.json' -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
