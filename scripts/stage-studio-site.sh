#!/usr/bin/env bash
# Stage the Studio web client (web/studio/) into a published-site directory.
#
# Usage:
#   scripts/stage-studio-site.sh [--typed] <studio-src> <dest-dir> [public_model_url] [public_model_label] \
#       [--trained-manifest FILE]
#
# <studio-src> is the built web/studio tree; it must already contain the wasm-pack output and the
# generated demo artifacts (none of these are committed; see web/studio/README.md):
#   pkg/uor_r4_integer.js, pkg/uor_r4_integer_bg.wasm, pkg/synthetic-stack.lut, pkg/tokenizer.json
#
# --typed selects <studio-src>/dist and the typed runtime allowlist instead of the legacy source.
#
# The helper:
#   1. replaces <dest-dir> wholesale (no stale files from an older Studio can survive a publish;
#      the live gh-pages /studio was the retired 13 Oct app before this change);
#   2. copies only the runtime files, not the dev-only verification harness, tools or README;
#   3. writes <dest-dir>/models.json, schema "uor-r4.studio-models/1": the bundled synthetic demo
#      plus, when given, one optional public-model URL slot (http/https only), plus any trained
#      models named by --trained-manifest;
#   4. refuses a file over GitHub Pages' 100 MB per-file limit, and prints file count and size.
#
# --trained-manifest FILE names a JSON array of one to four trained model entries:
#   [{ "id": "fbs-dual-c131-seed1", "label": "...", "file": "/path/seed-1.uorlut",
#      "hf_url": "https://huggingface.co/<org>/<repo>/resolve/main/<file>.uorlut",
#      "tokenizer": "/path/seed-1.tokenizer.json",
#      "audit": "/path/final.audit.json", "audit_sha256": "<64 lowercase hex>",
#      "note": "..." }]
# Each entry is copied to <dest>/models/<id>.uorlut with <id>.tokenizer.json beside it, and TWO
# published cards are written: the same-origin copy (url "./models/<id>.uorlut") and the Hugging
# Face URL (cross-origin, user-initiated, CORS required). The tokenizer is validated against the
# artifact (stack_sha256 + vocab) and the served-forward audit receipt is validated against the
# artifact before anything is copied. Remote-only entries must pin both artifact and receipt.
# Paths are local files; nothing is fetched from the network.
#
# It reads nothing from the network and writes only under <dest-dir>.
set -euo pipefail
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

usage() {
    echo "usage: $0 [--typed] <studio-src> <dest-dir> [public_model_url] [public_model_label] [--trained-manifest FILE]" >&2
    exit 2
}

typed=0
if [ "${1:-}" = '--typed' ]; then
    typed=1
    shift
fi
[ "$#" -ge 2 ] || usage
src=$1
dest=$2
shift 2
if [ "$typed" = 1 ]; then
    src=$src/dist
fi
public_url=''
public_label='Public model'
trained_manifest=''
positional=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --trained-manifest)
            [ "$#" -ge 2 ] || usage
            trained_manifest=$2
            shift 2
            ;;
        --public-model-url)
            [ "$#" -ge 2 ] || usage
            public_url=$2
            shift 2
            ;;
        --public-model-label)
            [ "$#" -ge 2 ] || usage
            public_label=$2
            shift 2
            ;;
        --*)
            echo "stage-studio-site: unknown option $1" >&2
            usage
            ;;
        *)
            positional=$((positional + 1))
            if [ "$positional" -eq 1 ]; then
                public_url=$1
            elif [ "$positional" -eq 2 ]; then
                public_label=$1
            else
                usage
            fi
            shift
            ;;
    esac
done
if [ -n "$trained_manifest" ] && [ ! -f "$trained_manifest" ]; then
    echo "stage-studio-site: --trained-manifest file not found: $trained_manifest" >&2
    exit 1
fi

[ -f "$src/index.html" ] || { echo "stage-studio-site: $src/index.html is missing" >&2; exit 1; }
for required in \
    pkg/uor_r4_integer.js \
    pkg/uor_r4_integer_bg.wasm \
    pkg/synthetic-stack.lut \
    pkg/tokenizer.json; do
    if [ ! -f "$src/$required" ]; then
        echo "stage-studio-site: $src/$required is missing; build the wasm bundle and the demo artifacts first (web/studio/README.md)" >&2
        exit 1
    fi
done
case "$public_url" in
    '') ;;
    http://*|https://*) ;;
    *) echo "stage-studio-site: public_model_url must be an http(s) URL, got '$public_url'" >&2; exit 1 ;;
esac
if [ -n "$public_url" ] && [ -z "$public_label" ]; then
    echo "stage-studio-site: public_model_label must not be empty when public_model_url is set" >&2
    exit 1
fi

if [ "$typed" = 1 ]; then
    "$repo_root/scripts/studio/runtime-files.sh" copy "$src" "$dest" --typed
else
    "$repo_root/scripts/studio/runtime-files.sh" copy "$src" "$dest" --allow-generated-missing
fi
rm -f "$dest/models.json"

node "$repo_root/scripts/studio/catalog.mjs" "$dest" "$public_url" "$public_label" "$trained_manifest" \
    "$repo_root/scripts/validate_served_forward_receipt.py"

if [ "$typed" = 1 ]; then
    "$repo_root/scripts/studio/runtime-files.sh" --verify "$dest" --typed
else
    "$repo_root/scripts/studio/runtime-files.sh" --verify "$dest"
fi

if find "$dest" -type f -size +100M | grep -q .; then
    echo "stage-studio-site: a file in $dest is over GitHub Pages' 100 MB per-file limit:" >&2
    find "$dest" -type f -size +100M >&2
    exit 1
fi

files=$(find "$dest" -type f | wc -l | tr -d ' ')
bytes=$(du -sk "$dest" | awk '{print $1 * 1024}')
echo "staged Studio: $files file(s), $bytes byte(s), manifest $(du -h "$dest/models.json" | cut -f1)"
