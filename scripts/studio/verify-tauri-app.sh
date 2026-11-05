#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    printf 'usage: verify-tauri-app.sh <path-to-UOR-Harness.app>\n' >&2
    exit 2
fi
app=$1
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
[ -d "$app" ] || { printf 'App bundle is unavailable: %s\n' "$app" >&2; exit 1; }
case "$app" in *'.app') ;; *) printf 'Expected a .app bundle: %s\n' "$app" >&2; exit 1 ;; esac
[ -n "${UOR_DESKTOP_SHA256:-}" ] || { printf 'UOR_DESKTOP_SHA256 is required\n' >&2; exit 1; }
contents="$app/Contents"
required=(
    'Info.plist'
    'MacOS/uor-desktop-client'
    'MacOS/uor-chat'
    'Resources/pkg/synthetic-stack.lut'
    'Resources/pkg/tokenizer.json'
    'Resources/models.json'
    'Resources/icon.icns'
    'Resources/SHA256SUMS'
)
for rel in "${required[@]}"; do
    [ -f "$contents/$rel" ] || { printf 'Missing packaged resource: %s\n' "$rel" >&2; exit 1; }
done
check_hash() {
    local path=$1 expected=$2 actual
    actual=$(shasum -a 256 "$path" | cut -d' ' -f1)
    [ "$actual" = "$expected" ] || { printf 'Hash mismatch for %s: %s\n' "$path" "$actual" >&2; exit 1; }
}
check_hash "$contents/MacOS/uor-chat" 'f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e'
check_hash "$contents/Resources/pkg/synthetic-stack.lut" 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df'
check_hash "$contents/Resources/pkg/tokenizer.json" '6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b'
check_hash "$contents/Resources/models.json" '96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a'
check_hash "$contents/MacOS/uor-desktop-client" "$UOR_DESKTOP_SHA256"
cmp -s "$contents/Resources/icon.icns" "$repo_root/src-tauri/icons/icon.icns" || { printf 'Icon differs from the reviewed generated icon\n' >&2; exit 1; }
cmp -s "$contents/Resources/SHA256SUMS" "$repo_root/src-tauri/runtime/studio/SHA256SUMS" || { printf 'Runtime manifest differs from the reviewed staged frontend\n' >&2; exit 1; }
while IFS= read -r -d '' path; do
    rel=${path#"$app"/}
    case "$rel" in
        *'/src/'*|*'/tests/'*|*'/node_modules/'*|*'/package.json'|*'/tsconfig.json'|*'/vite.config.ts'|*'/README'*|*'/verify_'*|*'.d.ts')
            printf 'Development file in app bundle: %s\n' "$rel" >&2
            exit 1
            ;;
    esac
done < <(find "$app" -type f -print0)
[ -x "$contents/MacOS/uor-desktop-client" ] || { printf 'Desktop executable is not executable\n' >&2; exit 1; }
[ -x "$contents/MacOS/uor-chat" ] || { printf 'Sidecar is not executable\n' >&2; exit 1; }
printf 'PACKAGED APP CONTRACT PASSED: %s\n' "$app"
printf 'Sidecar, resources, catalog and icon match their reviewed hashes; development files are absent. Unsigned development verification only.\n'
