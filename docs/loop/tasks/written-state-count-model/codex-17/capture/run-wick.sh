#!/usr/bin/env bash
# After the run's FIT, launch the existing public reader and frozen CUDA
# chain. Exactly seven explicitly pinned dependencies are in the stage spec.
set -euo pipefail
umask 077
[[ $# == 3 ]] || { echo "usage: run-wick.sh ABS_SPEC SPEC_SHA ABS_NEW_DIR" >&2; exit 2; }
spec="$1"
spec_hash="$2"
out="$3"
[[ "$spec" == /* && "$out" == /* && "$spec_hash" =~ ^[0-9a-f]{64}$ ]]
printf '%s  %s\n' "$spec_hash" "$spec" | sha256sum --check --status
jq -e '
  keys == ["inputs","schema","seed","wick_root"]
  and .schema=="uor-r4.requested-capture-stage/1"
  and (.seed==1 or .seed==2)
  and (.wick_root|type=="string" and startswith("/"))
  and (.inputs|keys)==["base","blind","capture","paired","resolved","span","writer"]
  and all(.inputs[];
      keys==["path","sha256"]
      and (.path|type=="string" and startswith("/") and (test("[\\n\\r]")|not))
      and (.sha256|type=="string" and test("^[0-9a-f]{64}$")))
' "$spec" >/dev/null
resolve() {
    local key="$1" expected filename
    filename=$(jq -er --arg key "$key" '.inputs[$key].path' "$spec")
    expected=$(jq -er --arg key "$key" '.inputs[$key].sha256' "$spec")
    printf '%s  %s\n' "$expected" "$filename" | sha256sum --check --status
    printf '%s\n' "$filename"
}
cap=$(resolve capture)
writer=$(resolve writer)
base=$(resolve base)
paired=$(resolve paired)
span=$(resolve span)
blind=$(resolve blind)
resolved=$(resolve resolved)
resolved_sha=$(jq -er '.inputs.resolved.sha256' "$spec")
seed=$(jq -er .seed "$spec")
wick=$(jq -er .wick_root "$spec")
mkdir "$out"
cd "$out"
cp "$spec" stage-spec.json
"$writer" select "$paired" "$blind" public - - "$out/public.json"
"$writer" select "$paired" "$blind" no-read - - "$out/no-read.json"
"$cap" requested "$paired" "$blind" "$out/public.json" "$resolved" "$resolved_sha" "$out/requested-diagnostic.json"
# The chain itself locks the GPU for the AOT commands and supplies the
# historical base/exact/span state. Do not hold that same lock around it.
"$writer" chain "$wick" "$seed" "$blind" "$out/chain" \
    "$out/public.json" "$out/no-read.json" "$out/requested-diagnostic.json"
for lane in public no-read requested-diagnostic; do
    extra=()
    if [[ $lane == requested-diagnostic ]]; then
        extra=("$out/public.json" "$resolved" "$resolved_sha")
    fi
    "$cap" capture "$base" "$paired" "$span" "$blind" "$out/$lane.json" "$out/chain" \
        "$out/$lane" "${extra[@]}"
done
sha256sum stage-spec.json public.json no-read.json requested-diagnostic.json \
    chain/receipt.json chain/nominees/chosen.json \
    {public,no-read,requested-diagnostic}/{candidates.json,legacy-word-classes.u8,packed-codebook.u16le} \
    > cold.sha256
sha256sum --check cold.sha256
