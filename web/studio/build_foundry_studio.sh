#!/usr/bin/env bash
set -euo pipefail
# Build the full Foundry Web Studio + real UOR-R4 integer WASM + RV64IMAC MVM Micro-Linux WASM
# and emit a self-contained static bundle in web/studio/dist/.

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
studio_root="$repo_root/web/studio"
foundry_root="$studio_root/foundry"
foundry_studio="$foundry_root/studio"
shell_dir="$foundry_studio/shell"
dist_dir="$studio_root/dist"

export PATH="$HOME/.cargo/bin:$PATH"

fetch_args=()
if [ -d /tmp/foundry-probe/studio ]; then
    fetch_args+=(--seed-dir /tmp/foundry-probe)
fi
echo "== 1/9 fetch locked Foundry binary assets =="
node "$repo_root/scripts/studio/fetch-foundry-assets.mjs" ${fetch_args[@]+"${fetch_args[@]}"}

echo "== 2/9 build UOR-R4 integer serving WASM + synthetic-stack.lut + tokenizer.json =="
bash "$studio_root/build_serve_wasm.sh" --no-verify

echo "== 3/9 build UOR-R4 RV64IMAC MVM WASM =="
bash "$studio_root/build_mvm_wasm.sh"

echo "== 4/9 stage UOR-R4 + MVM runtime packages into Foundry shell =="
rm -rf "$shell_dir/pkg" "$shell_dir/pkg-mvm" "$shell_dir/uor"
mkdir -p "$shell_dir/pkg" "$shell_dir/pkg-mvm" "$shell_dir/uor/js"
cp "$studio_root/pkg/uor_r4_integer.js" "$studio_root/pkg/uor_r4_integer_bg.wasm" "$studio_root/pkg/synthetic-stack.lut" "$studio_root/pkg/tokenizer.json" "$shell_dir/pkg/"
cp "$studio_root/pkg-mvm/uor_r4_mvm.js" "$studio_root/pkg-mvm/uor_r4_mvm_bg.wasm" "$shell_dir/pkg-mvm/"
cp "$studio_root/r4_worker.js" "$studio_root/coi-serviceworker.js" "$shell_dir/"
cp "$studio_root/assets/js/uor_model_worker.js" "$studio_root/assets/js/uor_tokenizer_artifact.js" "$shell_dir/uor/js/"

rm -rf "$shell_dir/models"
node -e "import('$repo_root/scripts/studio/catalog.mjs').then(m => m.writeStudioCatalog({ dest: '$shell_dir' }))"
node "$repo_root/scripts/studio/build-standards-index.mjs" "$shell_dir/standards-index.json"

echo "== 5/9 build Foundry core.wasm (hologram-forge-core) =="
cargo build --release --locked --manifest-path "$foundry_studio/core/Cargo.toml" --target wasm32-unknown-unknown
target_base="${CARGO_TARGET_DIR:-$foundry_studio/core/target}"
cp "$target_base/wasm32-unknown-unknown/release/hologram_forge_core.wasm" "$shell_dir/core.wasm"

echo "== 6/9 install Foundry JS deps (ignoring native Electron/sqlite scripts) & build Vite renderer =="
npm ci --prefix "$foundry_studio/runtime" --ignore-scripts --no-audit --no-fund
if [ ! -d "$foundry_studio/vendor/prism/node_modules" ] || [ ! -f "$foundry_studio/vendor/prism/node_modules/.package-lock.json" ]; then
    (
        cd "$foundry_studio/vendor/prism"
        ELECTRON_SKIP_BINARY_DOWNLOAD=1 npm ci --prefer-offline --ignore-scripts --engine-strict=false --force --no-audit --no-fund --loglevel=error
    )
else
    # Ensure npm's offline tarball store holds monaco-editor for stamp_shell.py.
    npm cache add monaco-editor@0.52.2 --prefer-offline >/dev/null 2>&1 || true
fi
(
    cd "$foundry_studio/vendor/prism"
    npx vite build --config vite.shell.config.mts --base ./ --outDir dist --emptyOutDir
)

echo "== 7/9 stamp Foundry shell assets, Monaco editor, esbuild-wasm compiler, and host.bundle.js =="
python3 "$foundry_studio/scripts/stamp_shell.py"

echo "== 8/9 project Foundry shell index.html, holo.html, manifest.json & seal closure =="
cargo run --release --locked --manifest-path "$foundry_studio/core/Cargo.toml" --bin project-shell
cp "$shell_dir/index.html" "$shell_dir/404.html"

python3 -c '
import hashlib, json, pathlib
foundry = pathlib.Path("'"$foundry_root"'")
shell = pathlib.Path("'"$shell_dir"'")
def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()
prov = {
    "schema": "hologram-forge/provenance/1",
    "source_repo": "UOR-Foundation/uor-r4",
    "foundry_upstream_commit": json.loads((pathlib.Path("'"$studio_root"'") / "foundry-assets.lock.json").read_text())["upstream_commit"],
    "kernel_ir_sha256": sha(foundry / "studio/generated/kernel.ir"),
    "coverage_sha256": sha(foundry / "studio/generated/coverage.json"),
    "roots_sha256": sha(foundry / "studio/generated/roots.json"),
    "generated_core_sha256": sha(foundry / "studio/generated/hologram_forge_core.rs"),
    "core_wasm_sha256": sha(shell / "core.wasm"),
    "browser_projection_sha256": json.loads((shell / "manifest.json").read_text())["closure"],
}
(shell / "provenance.json").write_text(json.dumps(prov, indent=2) + "\n", encoding="utf-8")
'

node "$repo_root/scripts/studio/seal-shell-closure.mjs" "$shell_dir" --template "$shell_dir/sw.template.js"

echo "== 9/9 stage clean bundle to web/studio/dist/ =="
rm -rf "$dist_dir"
mkdir -p "$dist_dir"
python3 -c '
import pathlib, shutil
src = pathlib.Path("'"$shell_dir"'")
dst = pathlib.Path("'"$dist_dir"'")
for p in sorted(src.rglob("*")):
    rel = p.relative_to(src)
    srel = str(rel).replace("\\", "/")
    if srel == "sw.template.js" or srel.endswith(".test.mjs") or srel.endswith(".test.js") or srel.endswith(".test.ts"):
        continue
    target = dst / rel
    if p.is_dir():
        target.mkdir(parents=True, exist_ok=True)
    elif p.is_file():
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(p, target)
'
for req in q/pkg/holospaces_web.js q/pkg/holospaces_web_bg.wasm core.wasm host.bundle.js manifest.json sw.js standards-index.json v13/uor_foundry_v13_wasm.js v13/uor_foundry_v13_wasm_bg.wasm; do
    [ -f "$dist_dir/$req" ] || { echo "build_foundry_studio: missing required dist file $req" >&2; exit 1; }
done
printf 'Built Foundry Studio in %s (%s files)\n' "$dist_dir" "$(find "$dist_dir" -type f | wc -l | tr -d ' ')"
