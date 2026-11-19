#!/usr/bin/env bash
# Each candidate follows the previously sealed source handoff format. Controls use the
# same role carrier identity and original immutable test prefixes.
set -euo pipefail
cd "$HOME/uor-codex/20261118-paired-role-fence/writer"
split=${1:?held or fresh}; arm=${2:?arm}; n=${3:?seed}; handoff=${4:-}
case "$split" in held|fresh);; *) exit 2;; esac
test -s FIT.md
(cd ../role/panels; sha256sum --status -c SHA256SUMS)
(cd controls; sha256sum --status -c frozen.sha256)
sha256sum --status -c executable.sha256
top="$HOME/uor-codex"
prior=$top/20261118-role-boundary-count
base=$top/20261118-value-connectors
model=$base/inputs/seed$n.d11
key=seed$n-$split
bare=../role/panels/$split-blind.json
role=controls/role/$key.json
d=results/$split/$arm/seed$n
mkdir -p "$(dirname "$d")"
mkdir "$d"
case "$arm" in
  role|none) args=(--argjson p '[]' --argjson grade '[]');;
  old) args=(--slurpfile p "controls/old/$key.json" --argjson grade '[]');;
  right|wrong) args=(--slurpfile grade "../role/panels/$split-grade.json" --argjson p '[]');;
  *) args=(--slurpfile p "$handoff" --argjson grade '[]');;
esac
jq -ce --arg arm "$arm" "${args[@]}" -f picks.jq "$role" > "$d/picks.json"
jq -cn --arg m "$(sha256sum "$model"|cut -d' ' -f1)" --arg b "$(sha256sum "$bare"|cut -d' ' -f1)" \
  --arg p "$(sha256sum "$d/picks.json"|cut -d' ' -f1)" '[{model:$m,bare:$b,picks:$p}]' > "$d/manifest.json"
sum=$(sha256sum "$d/manifest.json" | cut -d' ' -f1)
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
for attempt in primary repeat; do
  flock 8
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
  "$prior/bin/action-writer" entity-writer-action "$d/manifest.json" "$sum" "$model" "$bare" "$d/picks.json" "$d/$attempt.json"
  flock -u 8
done
cmp "$d/primary.json" "$d/repeat.json"
