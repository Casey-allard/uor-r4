#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-ordered-role-transfer"
cmp oracle-primary/cold.sha256 oracle-repeat/cold.sha256
for stage in oracle-primary oracle-repeat; do
    (cd "$stage" && sha256sum -c cold.sha256)
done
for seed in 1 2; do
    jq -ec '([.rows[]|select(.exact==true)]|length) as $n | if (.rows|length)==256 and $n>=248 then {exact:$n,rows:256} else error("oracle floor failed: \($n)/256") end' "oracle-primary/seed$seed/oracle-grade.json"
done
