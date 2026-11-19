#!/usr/bin/env bash
set -euo pipefail
umask 077
if [[ $# != 1 ]]; then
  echo 'usage: run-device.sh output-directory (run from pinned input directory)' >&2
  exit 2
fi
out=$1
mkdir "$out"
trap 'echo "$?" > "$out/exit"' EXIT
sha256sum --strict --check inputs.sha256 > "$out/input-check"
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv,noheader > "$out/device"

for case_name in fixture actual; do
  jq -ceS 'if type == "array" and length > 0 then . else error("empty differential") end' \
    "$case_name.oracle.json" > "$out/$case_name.expected.json"
  for pass in primary cold; do
    ./s2-differential-sm89 "$case_name.rows" "$case_name.queries.txt" |
      jq -ceS . > "$out/$case_name.$pass.json"
    cmp "$out/$case_name.expected.json" "$out/$case_name.$pass.json"
  done
done
(cd "$out" && sha256sum *.json > outputs.sha256)
