#!/usr/bin/env bash
# Independent pre-fit test of the fixed downstream path. No learned bank is loaded.
set -euo pipefail
cd "$HOME/uor-codex/20261118-ordered-role-transfer"
stage=${1:?specify fresh stage}
[[ "$stage" == oracle-primary || "$stage" == oracle-repeat ]]
sha256sum -c driver/oracle-pins.sha256
sha256sum -c ../20261118-paired-role-fence/fence/identity/INPUTS.sha256
(cd assets && sha256sum -c SHA256SUMS)
(cd panel && sha256sum -c panel-inputs.sha256)
sha256sum -c "$HOME/uor-codex/20261118-query-region-pick/writer/PREDECESSORS.sha256"
mkdir "$stage"
w=$PWD/bins/writer-c
for seed in 1 2; do
    out=$PWD/$stage/seed$seed
    mkdir "$out"
    for arm in oracle wrong; do
        "$w" select "assets/seed$seed-paired.d11" panel/held-blind.json "$arm" panel/held-bounds.json grading/held-grades.json "$out/$arm.json"
    done
    "$w" chain "$HOME/uor-codex" "$seed" panel/held-blind.json "$out/device" "$out/oracle.json" "$out/wrong.json"
    for arm in oracle wrong; do
        "$w" grade "$out/$arm.json" "$out/device" grading/held-grades.json - "$out/$arm-grade.json"
    done
    jq -e '.rows|length==256 and all(.[];.stratum=="supported")' "$out/oracle-grade.json"
done
(cd "$stage" && sha256sum seed*/*grade.json seed*/device/nominees/chosen.json > cold.sha256)
