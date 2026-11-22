#!/usr/bin/env bash
set -euo pipefail
# Build crates/uor-r4-mvm with the browser wasm feature into web/studio/pkg-mvm/.

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
crate_dir="$repo_root/crates/uor-r4-mvm"
out_dir="$repo_root/web/studio/pkg-mvm"

mkdir -p "$out_dir"

wasm_pack_bin="${WASM_PACK_BIN:-$(command -v wasm-pack || true)}"
if [ -z "$wasm_pack_bin" ] && [ -x "$HOME/.cargo/bin/wasm-pack" ]; then
    wasm_pack_bin="$HOME/.cargo/bin/wasm-pack"
fi
[ -n "$wasm_pack_bin" ] || { echo "wasm-pack not found in PATH or ~/.cargo/bin" >&2; exit 1; }

RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=-simd128" \
    "$wasm_pack_bin" build "$crate_dir" \
    --target web \
    --no-opt \
    --out-dir "$out_dir" \
    -- --locked --features wasm

rm -f "$out_dir/.gitignore" "$out_dir/README.md"
test -s "$out_dir/uor_r4_mvm.js"
test -s "$out_dir/uor_r4_mvm_bg.wasm"
printf 'built MVM WASM package in %s\n' "$out_dir"
