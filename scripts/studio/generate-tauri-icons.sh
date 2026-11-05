#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source_svg="$repo_root/src-tauri/assets/uor-icon.svg"
icons_dir="${UOR_TAURI_RUNTIME_HOME:-$repo_root/src-tauri}/icons"
iconset="$icons_dir/uor.iconset"

[ -f "$source_svg" ] || { printf 'Icon source is missing: %s\n' "$source_svg" >&2; exit 1; }
command -v qlmanage >/dev/null 2>&1 || { printf 'qlmanage is required to render the reviewed vector icon\n' >&2; exit 1; }
command -v iconutil >/dev/null 2>&1 || { printf 'iconutil is required to package the macOS icon\n' >&2; exit 1; }

rm -rf "$iconset"
mkdir -p "$iconset" "$icons_dir"
for spec in 16x16 16x16@2x 32x32 32x32@2x 128x128 128x128@2x 256x256 256x256@2x 512x512 512x512@2x; do
    case "$spec" in
        16x16) pixels=16 ;;
        16x16@2x|32x32) pixels=32 ;;
        32x32@2x) pixels=64 ;;
        128x128) pixels=128 ;;
        128x128@2x|256x256) pixels=256 ;;
        256x256@2x|512x512) pixels=512 ;;
        512x512@2x) pixels=1024 ;;
    esac
    qlmanage -t -s "$pixels" -o "$iconset" "$source_svg" >/dev/null 2>&1
    mv "$iconset/uor-icon.svg.png" "$iconset/icon_${spec}.png"
done
cp "$iconset/icon_512x512.png" "$icons_dir/icon.png"
iconutil -c icns -o "$icons_dir/icon.icns" "$iconset"
rm -rf "$iconset"
printf 'Generated Tauri icons from %s\n' "$source_svg"
