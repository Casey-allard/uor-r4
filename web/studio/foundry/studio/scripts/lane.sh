#!/bin/bash
# The full lane, replayed from the committed tree on Linux x86_64 (WSL or CI).
#
#   1. LexLean, the compiler PrismPM vendors, at the PrismPM commit this repo pins:
#      lock, check, build, verify the authored model under Lean 4.32.1
#      (leanchecker replay, exact per declaration axiom policy).
#   2. lean4-prod, also as PrismPM vendors it: export every definition root to
#      kernel LCNF twice (byte identical), then generate Rust twice (byte
#      identical) into generated/hologram_forge_core.rs.
#   3. Compile the generated Rust for the host and for wasm32-unknown-unknown.
#
# Needs: git, cargo (rustup), python3, and the elan toolchain
# leanprover/lean4:v4.32.1 (installed on demand; the only network use besides
# the first PrismPM clone).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${FORGE_LANE_WORK:-$HOME/.cache/hologram-forge-lane}"
PRISMPM_REV="$(tr -d '\n' < "$ROOT/PRISMPM_REV")"
mkdir -p "$WORK"

# 0. PrismPM at the pinned commit, its vendored LexLean built once.
if [ ! -d "$WORK/PrismPM/.git" ]; then
  git clone -q https://github.com/UOR-Foundation/PrismPM.git "$WORK/PrismPM"
fi
git -C "$WORK/PrismPM" checkout -q "$PRISMPM_REV"
if [ ! -x "$WORK/PrismPM/target/release/lexlean" ]; then
  (cd "$WORK/PrismPM" && cargo build --release -q --locked --manifest-path vendor/lexlean/Cargo.toml --target-dir target)
fi
LX="$WORK/PrismPM/target/release/lexlean"

# 0b. PrismPM's archive code is vendored into core/src (holo/*.rs verbatim; error.rs minus the one
#     LexLean conversion), so a published application is composed by PrismPM's own writer. Refuse a
#     build whose copy differs from PrismPM's at the pinned commit (tools/prismpm_vendor.sha256).
python3 "$ROOT/tools/prismpm_vendor.py" "$WORK/PrismPM"

# 0c. The brand: the Hologram brand kit's tokens, vendored at BRAND_KIT_REV, generate every stylesheet
#     the shell serves. Refuse a build whose vendored kit files, generated stylesheets, copied marks or
#     fonts drifted, or that names a colour by hand anywhere in the shell.
python3 "$ROOT/tools/brand_kit.py" --check

# 1. Lean toolchain.
export PATH="$HOME/.elan/bin:$PATH"
if ! elan toolchain list 2>/dev/null | grep -q 'leanprover/lean4:v4.32.1'; then
  command -v elan >/dev/null || {
    curl -sSfL https://elan.lean-lang.org/elan-init.sh | sh -s -- -y --default-toolchain none
    export PATH="$HOME/.elan/bin:$PATH"
  }
  elan toolchain install leanprover/lean4:v4.32.1
fi

# 2. LexLean over the committed source. `lock` must report current: the
#    committed lock is the authority, never rewritten here.
cd "$ROOT"
lake env lean --version >/dev/null 2>&1 || true
"$LX" lock | grep -q 'is current' || { echo "lexlean.lock is not current; run lexlean lock and commit it" >&2; exit 1; }
"$LX" check
# The build directory is content addressed; take the one this build reports, never the newest by
# time, or a restored model would export a stale sibling's module.
BUILD_ID=$("$LX" build | tee /dev/stderr | grep -o "build/[0-9a-f]*" | head -1 | cut -d/ -f2)
[ -n "$BUILD_ID" ] || { echo "lexlean build reported no build id" >&2; exit 1; }
# The attestation, like the build, is the one verify reports, not the newest by time.
ATTESTATION_ID=$("$LX" verify | tee /dev/stderr | grep -o "attestation [0-9a-f]*" | head -1 | cut -d' ' -f2)
[ -n "$ATTESTATION_ID" ] || { echo "lexlean verify reported no attestation" >&2; exit 1; }
echo "attestation: $ROOT/.lexlean/verified/$ATTESTATION_ID/attestation.json"

# 3. Export workspace: the generated Lean module beside vendored lean4-prod.
WS="$WORK/export-ws"
rm -rf "$WS"; mkdir -p "$WS/lean4-prod"
tar xf "$WORK/PrismPM/vendor/lean4-prod/lean.tar" -C "$WS/lean4-prod"
# Every module the project declares (lexlean.toml entrypoints, one module per file, in that order):
# copied beside lean4-prod, built and replayed by leanchecker, and exported together.
MODULES=$(grep -o '"src/[A-Za-z0-9_]*\.lex\.tex"' "$ROOT/lexlean.toml" | sed 's#"src/##; s#\.lex\.tex"##')
[ -n "$MODULES" ] || { echo "lexlean.toml lists no entrypoints" >&2; exit 1; }
mkdir -p "$WS/HologramForge"
LAKE_ROOTS=""; CHECK=""; EXPORT_MODULES=""
for m in $MODULES; do
  cp "$ROOT/.lexlean/build/$BUILD_ID/modules/HologramForge/$m.lean" "$WS/HologramForge/"
  LAKE_ROOTS="$LAKE_ROOTS${LAKE_ROOTS:+, }\"HologramForge.$m\""
  CHECK="$CHECK HologramForge.$m"
  EXPORT_MODULES="$EXPORT_MODULES --module HologramForge.$m"
done
cat > "$WS/lakefile.toml" <<EOF
name = "forge_verify"
version = "0.1.0"

[[lean_lib]]
name = "PrismGenerated"
roots = [$LAKE_ROOTS]
EOF
printf 'leanprover/lean4:v4.32.1\n' > "$WS/lean-toolchain"
cd "$WS"
lake build PrismGenerated
# shellcheck disable=SC2086
lake env leanchecker $CHECK
cd "$WS/lean4-prod"
lake build prod-export
export LEAN_PATH="$WS/.lake/build/lib/lean"
# model/roots.txt names every definition root as Module.name, strictly sorted, as the exporter demands.
ROOTS=""
for r in $(tr '\n' ' ' < "$ROOT/model/roots.txt"); do
  ROOTS="$ROOTS --root HologramForge.$r"
done
# shellcheck disable=SC2086
lake exe prod-export $EXPORT_MODULES $ROOTS --ir-module HologramForge --out "$WS/export-a"
# shellcheck disable=SC2086
lake exe prod-export $EXPORT_MODULES $ROOTS --ir-module HologramForge --out "$WS/export-b"
cmp "$WS/export-a/kernel.ir" "$WS/export-b/kernel.ir"
cmp "$WS/export-a/roots.json" "$WS/export-b/roots.json"
cmp "$WS/export-a/coverage.json" "$WS/export-b/coverage.json"

# 4. Rust generation over the vendored prod crates, twice.
DRIVER="$WORK/codegen"
rm -rf "$DRIVER" "$WORK/prod-rust"; mkdir -p "$DRIVER/src"
cp -r "$WORK/PrismPM/vendor/lean4-prod/rust" "$WORK/prod-rust"
cat > "$DRIVER/Cargo.toml" <<'EOF'
[package]
name = "codegen-driver"
version = "0.1.0"
edition = "2021"

[dependencies]
prod-ir = { path = "../prod-rust/prod-ir" }
prod-codegen = { path = "../prod-rust/prod-codegen" }

[workspace]
EOF
cat > "$DRIVER/src/main.rs" <<'EOF'
fn main() {
    let path = std::env::args().nth(1).expect("kernel.ir path");
    let out = std::env::args().nth(2).expect("output path");
    let text = std::fs::read_to_string(&path).expect("read kernel.ir");
    let (rest, module) = prod_ir::parser::parse_module(&text).expect("parse kernel.ir");
    assert!(rest.trim().is_empty(), "trailing kernel.ir bytes");
    let a = prod_codegen::generate_module(&module).expect("generate a");
    let b = prod_codegen::generate_module(&module).expect("generate b");
    assert_eq!(a, b, "generation is not deterministic");
    std::fs::write(&out, &a).expect("write generated.rs");
}
EOF
(cd "$DRIVER" && cargo build --release -q)
"$DRIVER/target/release/codegen-driver" "$WS/export-a/kernel.ir" "$WS/generated.rs"

# 5. Compare or refresh the committed artifacts.
mkdir -p "$ROOT/generated"
compare() { cmp <(tr -d '\r' < "$1") <(tr -d '\r' < "$2") && echo "match: $2"; }
if [ "${LANE_WRITE:-0}" = "1" ]; then
  cp "$WS/export-a/kernel.ir" "$ROOT/generated/kernel.ir"
  cp "$WS/export-a/roots.json" "$ROOT/generated/roots.json"
  cp "$WS/export-a/coverage.json" "$ROOT/generated/coverage.json"
  cp "$WS/generated.rs" "$ROOT/generated/hologram_forge_core.rs"
  echo "wrote generated/ (review and commit)"
else
  compare "$WS/export-a/kernel.ir" "$ROOT/generated/kernel.ir"
  compare "$WS/export-a/roots.json" "$ROOT/generated/roots.json"
  compare "$WS/export-a/coverage.json" "$ROOT/generated/coverage.json"
  compare "$WS/generated.rs" "$ROOT/generated/hologram_forge_core.rs"
fi

# 6. The generated core for the host and for wasm32; the wasm is what the page loads.
cd "$ROOT/core"
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo build --release -q
cargo build --release -q --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hologram_forge_core.wasm "$ROOT/shell/core.wasm"
ls -la "$ROOT/shell/core.wasm"

# 7. The page is a projection of the verified View: index.html, the web manifest and the shell
#    hash list are written from view() by the projector. Compare mode refuses drift in the words.
cargo run --release -q --bin project-shell
cd "$ROOT"
if [ "${LANE_WRITE:-0}" != "1" ]; then
  git diff --exit-code -- shell/index.html shell/holo.html || { echo "shell/index.html or shell/holo.html drifted from the projected View; run LANE_WRITE=1 ./scripts/lane.sh and commit" >&2; exit 1; }
fi

# 8. The evidence a published application carries (shell/provenance.json): derived from the
#    attestation, the build manifest, PrismPM's dependency register and stdlib release, its pinned
#    hologram-live and uor-hologram commits, the export, the generated core, the packaged crate and
#    the shell closure. Precached by the worker beside manifest.json; outside the closure because the
#    closure digest is one of its fields. Nothing in it is typed in; a field the lane cannot derive
#    stops the lane.
python3 tools/provenance.py "$WORK/PrismPM" "$ATTESTATION_ID" "$BUILD_ID" "$WS/export-a"
echo "lane: green"
