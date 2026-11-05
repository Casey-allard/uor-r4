#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
release_root='https://github.com/UOR-Foundation/uor-r4/releases/download/studio-native-runner-v0.1.0'
runtime_home="${UOR_TAURI_RUNTIME_HOME:-$repo_root/src-tauri}"
binary_name='uor-chat-aarch64-apple-darwin'
archive_name='studio-local.tar.gz'
binary_sha='f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e'
archive_sha='bf78084e09d03736d3de5cfbe98040e632cc102946f6396434f12314b9ede04d'
stack_sha='d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df'
tokenizer_sha='6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b'
catalog_sha='96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a'

if [ "${1:-}" = '--check' ]; then
    printf 'sidecar: %s (%s)\n' "$binary_name" "$binary_sha"
    printf 'archive: %s (%s)\n' "$archive_name" "$archive_sha"
    printf 'externalBin: ["binaries/uor-chat"]\n'
    printf 'runtime: %s/runtime/studio\n' "$runtime_home"
    exit 0
fi
[ "$#" -eq 0 ] || { printf 'usage: prepare-tauri-runtime.sh [--check]\n' >&2; exit 2; }
for command_name in curl shasum tar; do
    command -v "$command_name" >/dev/null 2>&1 || { printf 'Required command is unavailable: %s\n' "$command_name" >&2; exit 1; }
done

verify_hash() {
    local path="$1" expected="$2" actual
    actual="$(shasum -a 256 "$path")"
    actual="${actual%% *}"
    [ "$actual" = "$expected" ] || { printf 'SHA-256 mismatch for %s\nexpected: %s\nactual:   %s\n' "$path" "$expected" "$actual" >&2; return 1; }
}

fetch_verified() {
    local url="$1" destination="$2" expected="$3"
    local partial="${destination}.part.$$"
    if [ -f "$destination" ] && verify_hash "$destination" "$expected" 2>/dev/null; then
        return 0
    fi
    mkdir -p "$(dirname "$destination")"
    if ! curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 "$url" --output "$partial"; then
        rm -f "$partial"
        return 1
    fi
    if ! verify_hash "$partial" "$expected"; then
        rm -f "$partial"
        return 1
    fi
    mv "$partial" "$destination"
}

fetch_verified "$release_root/uor-chat-arm64-apple-darwin" "$runtime_home/binaries/$binary_name" "$binary_sha"
fetch_verified "$release_root/$archive_name" "$runtime_home/runtime/$archive_name" "$archive_sha"
chmod 755 "$runtime_home/binaries/$binary_name"

"$repo_root/scripts/studio/runtime-files.sh" copy "$repo_root/web/studio" "$runtime_home/runtime/studio" --allow-generated-missing
tar -xzf "$runtime_home/runtime/$archive_name" -C "$runtime_home/runtime" 'studio/pkg' 'studio/models.json'
verify_hash "$runtime_home/runtime/studio/pkg/synthetic-stack.lut" "$stack_sha"
verify_hash "$runtime_home/runtime/studio/pkg/tokenizer.json" "$tokenizer_sha"
verify_hash "$runtime_home/runtime/studio/models.json" "$catalog_sha"
"$repo_root/scripts/studio/runtime-files.sh" --verify "$runtime_home/runtime/studio"
: > "$runtime_home/runtime/studio/SHA256SUMS"
while IFS= read -r rel; do
    if [ -f "$runtime_home/runtime/studio/$rel" ]; then
        shasum -a 256 "$runtime_home/runtime/studio/$rel" | sed "s|$runtime_home/runtime/studio/||" >> "$runtime_home/runtime/studio/SHA256SUMS"
    fi
done < <("$repo_root/scripts/studio/runtime-files.sh" --list)
"$repo_root/scripts/studio/generate-tauri-icons.sh"
printf 'Tauri runtime prepared under %s\n' "$runtime_home"
