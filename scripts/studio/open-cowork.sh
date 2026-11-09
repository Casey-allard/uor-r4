#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
target_dir="${CARGO_TARGET_DIR:-$repo_root/target/cowork}"
case "$target_dir" in
    /*) ;;
    *) target_dir="$repo_root/$target_dir" ;;
esac
app="$target_dir/release/bundle/macos/UOR Harness.app"
identity='org.uor.r4.desktop.cowork'

if [ "${1:-}" = '--check' ]; then
    printf 'target: %s\n' "$target_dir"
    printf 'app: %s\n' "$app"
    printf 'identity: %s\n' "$identity"
    printf 'frontend: web/studio/dist\n'
    exit 0
fi
[ "$#" -eq 0 ] || { printf 'usage: open-cowork.sh [--check]\n' >&2; exit 2; }
for command_name in cargo npm node plutil; do
    command -v "$command_name" >/dev/null 2>&1 || { printf 'Required command is unavailable: %s\n' "$command_name" >&2; exit 1; }
done
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR="$target_dir"
bash "$repo_root/scripts/studio/prepare-tauri-runtime.sh"
cargo tauri build --bundles app --features gui
[ -x "$app/Contents/MacOS/uor-desktop-client" ] || { printf 'Missing cowork executable: %s\n' "$app" >&2; exit 1; }
actual_identity=$(plutil -extract CFBundleIdentifier raw "$app/Contents/Info.plist")
[ "$actual_identity" = "$identity" ] || { printf 'Unexpected application identity: %s\n' "$actual_identity" >&2; exit 1; }
shasum -a 256 "$app/Contents/MacOS/uor-desktop-client"
if [ "${UOR_COWORK_NO_OPEN:-0}" = '1' ]; then
    printf 'Cowork build verified; launch skipped by UOR_COWORK_NO_OPEN=1\n'
    exit 0
fi
exec "$app/Contents/MacOS/uor-desktop-client"
