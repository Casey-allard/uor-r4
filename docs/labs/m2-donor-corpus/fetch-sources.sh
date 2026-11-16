#!/usr/bin/env bash
# fetch-sources.sh OUT_DIR: download the prompt bank's source files (claude-400) at
# pinned dataset revisions, and check each one against Hugging Face's LFS sha256.
# Only user turns are later taken from these files (src/bank.rs); every assistant text
# in them is ignored. Licences, read on each card on 15 Nov 2026, are listed below.
set -euo pipefail
OUT=${1:?usage: fetch-sources.sh OUT_DIR}
mkdir -p "$OUT"

# name|repo|revision|licence|path glob (grep -E on the tree path)
SOURCES='wildchat|allenai/WildChat-4.8M|c827c6df8fcf008219ffaffa4d1dd77491099367|odc-by|^data/train-000[0-8][02468]-of-00086\.parquet$
numina|project-numina/NuminaMath-1.5|1b05109f9e5c1ad06c0663519502416c30b300f8|apache-2.0|^data/train-.*\.parquet$
openthoughts|open-thoughts/OpenThoughts-114k|bd093c3994fd54d2390985b66988ddf282a55eb6|apache-2.0|^data/train-.*\.parquet$
oasst2|OpenAssistant/oasst2|179dd21fc55192153d94adb0e0ce8f69e222bf75|apache-2.0|^data/train-.*\.parquet$
gsm8k|openai/gsm8k|740312add88f781978c0658806c59bc2815b9866|mit|^main/train-.*\.parquet$'

MANIFEST="$OUT/sources.tsv"
printf 'name\trepo\trevision\tlicence\tpath\tbytes\tsha256\n' > "$MANIFEST.new"
while IFS='|' read -r name repo rev lic glob; do
  dir=$(dirname "$(echo "$glob" | sed -e 's/^\^//' -e 's/\\//g')")
  mkdir -p "$OUT/$name"
  curl -sf "https://huggingface.co/api/datasets/$repo/tree/$rev/$dir" |
    jq -r --arg g "$glob" '.[] | select(.type == "file") | select(.path | test($g)) | "\(.path) \(.size) \(.lfs.oid)"' |
    while read -r path size oid; do
      dest="$OUT/$name/$(basename "$path")"
      if [ ! -f "$dest" ] || [ "$(shasum -a 256 "$dest" | cut -d' ' -f1)" != "$oid" ]; then
        curl -sfL --retry 5 -o "$dest.tmp" "https://huggingface.co/datasets/$repo/resolve/$rev/$path"
        mv "$dest.tmp" "$dest"
      fi
      got=$(shasum -a 256 "$dest" | cut -d' ' -f1)
      [ "$got" = "$oid" ] || { echo "sha256 mismatch: $dest" >&2; exit 1; }
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$name" "$repo" "$rev" "$lic" "$path" "$size" "$got" >> "$MANIFEST.new"
      echo "ok $name $path"
    done
done <<< "$SOURCES"
mv "$MANIFEST.new" "$MANIFEST"
echo "manifest: $MANIFEST"
