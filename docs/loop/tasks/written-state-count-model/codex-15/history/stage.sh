#!/usr/bin/env bash
# Stage known archived data; never invokes a model, writer, or GPU executable.
set -euo pipefail
[[ $# == 4 ]] || { echo 'usage: stage.sh HISTORY NEW_LAB ARCHIVES CUSTODY_TAR' >&2; exit 2; }
here=$(cd "$1" && pwd)
lab=$2
archives=$(cd "$3" && pwd)
custody_dir=$(cd "$(dirname "$4")" && pwd)
custody="$custody_dir/$(basename "$4")"
test ! -e "$lab"
test ! -e "$here/source"
(cd "$archives" && shasum -a 256 -c "$here/ARCHIVES.sha256")
digest=$(shasum -a 256 "$custody" | awk '{print $1}')
[[ "$digest" == "$(awk 'NR==1{print $1}' "$here/CUSTODY.sha256")" ]]
mkdir -p "$lab"/{span-connectors,value-connectors,continuation-eos}-20261118
tar -xzf "$archives/span-connectors-wick.tar.gz" -C "$lab/span-connectors-20261118" \
  panels/held-grade.json panels/prior-grade.json results/span-01/seed{1,2}-state.json
tar -xzf "$archives/value-connectors-wick.tar.gz" -C "$lab/value-connectors-20261118" \
  panels/fresh-grade.json results/value-01/seed{1,2}-{primary,fresh,prior}.json
tar -xzf "$archives/continuation-eos-wick.tar.gz" -C "$lab/continuation-eos-20261118" \
  transfer-01/transfer-grade.json results/transfer-baseline/seed{1,2}-value.json \
  results/writer-inputs/seed{1,2}-{primary,fresh,transfer,prior}-seal.json \
  results/continuation-01/seed{1,2}-{primary,fresh,transfer,prior}.json \
  results/continuation-repeat/seed{1,2}-{primary,fresh,transfer,prior}.json \
  results/writer-01/seed{1,2}-{primary,fresh,transfer,prior}{,-repeat}.json
readarray_or_read() {
  awk '{print $2}' "$here/SOURCE-PINS.sha256"
}
# The trust boundary is the pinned archive plus the exact expected member list.
while IFS= read -r member; do
  tar -xOzf "$custody" "$member" > /dev/null
done < <(readarray_or_read)
tar -xzf "$custody" -C "$here" $(readarray_or_read)
(cd "$here" && shasum -a 256 -c SOURCE-PINS.sha256)
