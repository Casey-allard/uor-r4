#!/usr/bin/env bash
# Build the Studio web client and publish (or dry-run publish) its runtime files to the docs
# site's /studio/, without changing any tracked file in the current checkout.
#
# The manual docs publisher replaces the book on gh-pages while preserving studio/.
# This helper stages and inspects a Studio update locally unless explicitly published.
#
# SAFETY
#   * default is a DRY RUN: builds, stages, makes a local commit in a throwaway worktree of the
#     target branch, prints the tree, and pushes NOTHING.
#   * --push is the only way anything reaches the remote, and it also requires --yes. It is meant
#     to be run by the owner/Lead from a clean, attached checkout of main after review.
#   * the current checkout's tracked files are untouched: build outputs live in the gitignored
#     web/studio/pkg/, and the publish commit is made in <tmp>/site and then discarded.
#
# USAGE
#   scripts/studio/build-and-publish.sh [options]
#     --typed                build and stage the Vite app with the public M1 predictor card
#                            (default: legacy; --trained-manifest replaces the typed card selection)
#     --target <branch>       gh-pages only; required explicitly for a live push
#     --push                  push the publish commit (otherwise dry run)
#     --no-build              reuse web/studio/pkg/ as built
#     --skip-verify           skip the Studio verification harnesses
#     --public-model-url U    optional public http(s) UORLUT01 URL to list in models.json
#     --public-model-label L  label for that URL (default: Public model)
#     --trained-manifest FILE stage one to four trained models from a JSON manifest
#                             (scripts/stage-studio-site.sh --trained-manifest); each may be
#                             bundled same-origin or listed remote-only by URL. Every trained
#                             entry must include a passing served-forward audit receipt.
#     --stage-dir DIR         copy the staged studio tree to DIR for inspection
#     --keep-worktree         keep the throwaway publish worktree and print its path
#   ENV:
#     WASM_PACK   wasm-pack binary to use (default: wasm-pack on PATH; pin 0.13.1 for CI parity)
#     CARGO       cargo binary (default: ~/.cargo/bin/cargo when present, to avoid PATH shims)
#     CHROME      Chrome/Chromium binary for the runtime page-load trap (auto-detected)
#
# It prints the exact commands for the owner to apply the same change by hand if preferred.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$root"

target=''
typed=0
push=0
yes=0
build=1
verify=1
public_url=''
public_label='Public model'
trained_manifest=''
stage_dir=''
keep_worktree=0

say() { printf '%s\n' "$*"; }
die() { printf 'build-and-publish: %s\n' "$*" >&2; exit 2; }

# Emit a rollback only after a successful push. Its explicit lease fails closed
# if another publication has advanced gh-pages, even after an intervening fetch.
publish_studio() {
    local checkout=$1 previous=$2 published rollback
    published=$(git -C "$checkout" rev-parse HEAD)
    if [ "$(git -C "$checkout" show -s --format=%P "$published")" != "$previous" ]; then
        echo "refusing to publish: the candidate must have the staged Pages head as its only parent" >&2
        return 1
    fi
    git -C "$checkout" push --force-with-lease="refs/heads/gh-pages:$previous" \
        origin "$published:refs/heads/gh-pages" || return 1
    rollback="git push --force-with-lease=refs/heads/gh-pages:$published origin $previous:refs/heads/gh-pages"
    say "PUSHED gh-pages -> $published"
    say "rollback if needed: $rollback"
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
        {
            echo "### Studio published to gh-pages"
            echo ""
            echo "- previous head: \`$previous\`"
            echo "- new head: \`$published\`"
            echo "- rollback: \`$rollback\`"
            echo "- the next docs-site book deploy keeps studio/"
        } >> "$GITHUB_STEP_SUMMARY"
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --typed) typed=1; shift ;;
        --target) target=${2:?--target needs a branch}; shift 2 ;;
        --push) push=1; shift ;;
        --yes) yes=1; shift ;;
        --no-build) build=0; shift ;;
        --skip-verify) verify=0; shift ;;
        --public-model-url) public_url=${2:?--public-model-url needs a value}; shift 2 ;;
        --public-model-label) public_label=${2:?--public-model-label needs a value}; shift 2 ;;
        --trained-manifest) trained_manifest=${2:?--trained-manifest needs a path}; shift 2 ;;
        --stage-dir) stage_dir=${2:?--stage-dir needs a path}; shift 2 ;;
        --keep-worktree) keep_worktree=1; shift ;;
        -h|--help) sed -n '1,60p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) die "unknown argument $1 (try --help)" ;;
    esac
done
if [ "$push" = 1 ] && [ -z "$target" ]; then
    die "--push needs an explicit --target gh-pages"
fi
target=${target:-gh-pages}
[ "$target" = gh-pages ] || die "--target must be gh-pages"
if [ "$push" = 1 ] && [ "$yes" != 1 ]; then
    die "--push also needs --yes: publishing is a deliberate action (dry run is the default)"
fi
branch=$(git symbolic-ref -q --short HEAD || true)
if [ "$push" = 1 ]; then
    [ "$branch" = main ] || die "refusing to push live: the checkout must be on branch main (it is '${branch:-detached}')"
    [ -z "$(git status --porcelain)" ] || die "refusing to push live from a dirty checkout"
    git fetch -q --no-prune --no-tags origin refs/heads/main:refs/remotes/origin/main ||
        die "cannot verify current origin/main"
    head_sha=$(git rev-parse HEAD)
    main_sha=$(git rev-parse origin/main 2>/dev/null || true)
    [ -n "$main_sha" ] || die "origin/main is not fetched; run git fetch origin main"
    [ "$head_sha" = "$main_sha" ] || die "refusing to push live: HEAD ($head_sha) is not origin/main ($main_sha)"
fi

cargo_bin=${CARGO:-}
if [ -z "$cargo_bin" ]; then
    if [ -x "$HOME/.cargo/bin/cargo" ]; then cargo_bin="$HOME/.cargo/bin/cargo"; else cargo_bin=cargo; fi
fi
wasm_pack=${WASM_PACK:-}
if [ -z "$wasm_pack" ]; then
    wasm_pack=$(command -v wasm-pack || true)
    [ -n "$wasm_pack" ] || die "wasm-pack not found; install it (0.13.1 for CI parity) or set WASM_PACK"
fi
node_bin=$(command -v node || true)
[ -n "$node_bin" ] || die "node not found"
[ -x "$cargo_bin" ] || die "cargo not found at $cargo_bin"

say "Studio build and publish"
say "  repo:    $root"
say "  target:  $target ($([ "$push" = 1 ] && echo 'PUSH (confirmed)' || echo 'dry run, nothing pushed'))"
say "  branch:  ${branch:-detached}"
if [ "$typed" = 1 ]; then
    say "  mode:    typed Vite dist"
else
    say "  mode:    legacy Studio source"
fi
say "  cargo:   $cargo_bin ($("$cargo_bin" --version))"
say "  wasm:    $wasm_pack ($("$wasm_pack" --version))"
say "  node:    $node_bin ($("$node_bin" --version))"


# ---------------------------------------------------------------------------
# 1. Build the wasm engine, the synthetic demo stack and its tokenizer.
pkg=web/studio/pkg
if [ "$build" = 1 ]; then
    say ""
    say "== build: wasm (crates/uor-r4-integer --features serving-wasm, wasm-pack 0.13.1 recipe)"
    rm -rf "$pkg"
    ( export PATH="$HOME/.cargo/bin:$PATH"
      RUSTFLAGS='-C target-feature=-simd128' "$wasm_pack" build crates/uor-r4-integer \
        --target web --no-opt --out-dir ../../web/studio/pkg -- --locked --features serving-wasm )
    say "== build: synthetic demo stack (seed 5) and stack-bound tokenizer"
    "$cargo_bin" run -q --locked -p uor-r4-integer --features serving-wasm \
        --bin mk-synthetic-stack -- 5 "$pkg/synthetic-stack.lut"
    "$node_bin" web/studio/tools/make_tokenizer.mjs "$pkg/synthetic-stack.lut" --out "$pkg/tokenizer.json"
    "$node_bin" web/studio/tools/make_tokenizer.mjs "$pkg/synthetic-stack.lut" --check "$pkg/tokenizer.json"
else
    [ -f "$pkg/uor_r4_integer_bg.wasm" ] || die "--no-build but $pkg/uor_r4_integer_bg.wasm is missing"
fi
for f in uor_r4_integer.js uor_r4_integer_bg.wasm synthetic-stack.lut tokenizer.json; do
    [ -f "$pkg/$f" ] || die "missing build output $pkg/$f"
done
ls -lh "$pkg"

# ---------------------------------------------------------------------------
# 2. Verify the page against the built bundle (the same bars the publish workflow should run).
if [ "$verify" = 1 ]; then
    say ""
    say "== verify: worker engine traps"
    "$node_bin" web/studio/verify_studio_worker.mjs
    say "== verify: static page-load references (zero external)"
    "$node_bin" web/studio/verify_studio_page.mjs --strict-router
    chrome=${CHROME:-}
    if [ -z "$chrome" ]; then
        for c in \
            '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
            '/Applications/Chromium.app/Contents/MacOS/Chromium' \
            /usr/bin/google-chrome /usr/bin/chromium /usr/bin/chromium-browser; do
            if [ -x "$c" ]; then chrome=$c; break; fi
        done
    fi
    if [ -n "$chrome" ]; then
        say "== verify: runtime page-load trap in $chrome (zero external)"
        "$node_bin" web/studio/verify_studio_page_load.mjs --strict-router
    else
        say "== verify: runtime page-load trap SKIPPED (no Chrome/Chromium found; set CHROME=...);"
        say "   the static scan above still ran. This is the one bar a headless browser is needed for."
    fi
    say "== verify: wasm instruction audit (D11 serving classes)"
    python3 scripts/audit_zero_matmul_wasm.py "$pkg/uor_r4_integer_bg.wasm" \
        --serving --call-graph --tap --build-commit "$(git rev-parse HEAD)"
else
    say "== verify skipped (--skip-verify)"
fi

if [ "$typed" = 1 ]; then
    say ""
    say "== build: typed Vite application"
    ( cd web/studio && npm ci && npm run build )
fi

# ---------------------------------------------------------------------------
# 3. Stage the Studio for publication (single source of truth: scripts/stage-studio-site.sh).
staging=$(mktemp -d "${TMPDIR:-/tmp}/studio-publish.XXXXXX")
cleanup() {
    if [ "$keep_worktree" = 1 ] && [ -n "${site_wt:-}" ] && [ -d "${site_wt:-}" ]; then
        say "kept publish worktree: $site_wt"
    elif [ -n "${site_wt:-}" ] && [ -d "${site_wt:-}" ]; then
        git -C "$root" worktree remove --force "$site_wt" >/dev/null 2>&1 || true
    fi
    if [ "$keep_worktree" = 1 ]; then
        say "kept staging dir: $staging"
    else
        rm -rf "$staging"
    fi
}
trap cleanup EXIT

say ""
if [ "$typed" = 1 ]; then
    say "== stage: web/studio/dist -> $staging/studio (+ models.json)"
    stage_args=(--typed web/studio "$staging/studio" "$public_url" "$public_label")
else
    say "== stage: web/studio -> $staging/studio (+ models.json)"
    stage_args=(web/studio "$staging/studio" "$public_url" "$public_label")
fi
if [ "$typed" = 1 ] && [ -z "$trained_manifest" ]; then
    trained_manifest="$root/scripts/studio/public-m1.json"
fi
if [ -n "$trained_manifest" ]; then
    stage_args+=(--trained-manifest "$trained_manifest")
fi
scripts/stage-studio-site.sh "${stage_args[@]}"
if [ -n "$stage_dir" ]; then
    rm -rf "$stage_dir"
    mkdir -p "$stage_dir"
    cp -R "$staging/studio/." "$stage_dir/"
    say "staged tree also at: $stage_dir"
fi

# ---------------------------------------------------------------------------
# 4. Make the publish commit in a throwaway worktree of the target branch, then (optionally) push.
say ""
say "== fetch origin $target"
git fetch -q --no-prune --no-tags origin "refs/heads/$target:refs/remotes/origin/$target" || die "git fetch origin $target failed; is the branch present?"
if ! git rev-parse --verify --quiet "refs/remotes/origin/$target" >/dev/null; then
    die "origin/$target does not exist"
fi
site_wt="$staging/site"
git worktree add -q --detach "$site_wt" "refs/remotes/origin/$target"
prev=$(git -C "$site_wt" rev-parse HEAD)
say "previous origin/$target head: $prev"
# Replace the whole Studio directory: no file from the retired 13 Oct app may survive a publish.
rm -rf "$site_wt/studio"
mkdir -p "$site_wt/studio"
cp -R "$staging/studio/." "$site_wt/studio/"

# The staged page is what ships: prove the browser's files are there and stale ones are gone.
for f in index.html models.json pkg/uor_r4_integer.js pkg/uor_r4_integer_bg.wasm \
         pkg/synthetic-stack.lut pkg/tokenizer.json; do
    [ -f "$site_wt/studio/$f" ] || die "staged $site_wt/studio/$f is missing"
done
if [ -e "$site_wt/studio/pkg/uor_r4_wasm_router_bg.wasm" ]; then
    die "retired router engine file survived staging: $site_wt/studio/pkg/uor_r4_wasm_router_bg.wasm"
fi
"$node_bin" -e "const m = require('$site_wt/studio/models.json'); if (m.schema !== 'uor-r4.studio-models/1' || !Array.isArray(m.models) || m.models.length === 0) { console.error('bad models.json'); process.exit(1); } console.log('models.json: ' + m.models.map((x) => x.id + ' (' + x.kind + ')').join(', '))"
"$node_bin" web/studio/tools/make_tokenizer.mjs "$site_wt/studio/pkg/synthetic-stack.lut" --check "$site_wt/studio/pkg/tokenizer.json"

# GitHub Pages hard limits: 100 MB per file, 1 GB per published site.
"$node_bin" - "$site_wt" <<'NODE'
const { readdirSync, statSync } = require('node:fs');
const { join } = require('node:path');
const root = process.argv[2];
let total = 0;
const files = [];
(function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) walk(path);
    else { const size = statSync(path).size; total += size; files.push([path, size]); }
  }
})(root);
const big = files.filter(([, size]) => size >= 100 * 1024 * 1024);
if (big.length) { console.error('over the 100 MB per-file Pages limit: ' + big.map(([p]) => p).join(', ')); process.exit(1); }
if (total >= 1024 * 1024 * 1024) { console.error('over the 1 GB Pages site limit: ' + total + ' bytes'); process.exit(1); }
console.log('site: ' + files.length + ' file(s), ' + total + ' byte(s) (Pages limit 1073741824)');
const studio = files.filter(([p]) => p.startsWith(join(root, 'studio')));
console.log('studio: ' + studio.length + ' file(s), ' + studio.reduce((n, [, s]) => n + s, 0) + ' byte(s)');
NODE

git -C "$site_wt" add -A
if git -C "$site_wt" diff --cached --quiet; then
    say "nothing changed: the target already carries this Studio build"
else
    git -C "$site_wt" -c user.name="studio-publish" -c user.email="studio-publish@users.noreply.github.com" \
        commit -q -m "studio: publish web/studio build from $(git rev-parse --short HEAD)$([ -n "$public_url" ] && echo " + public model" || echo '')"
    say ""
    say "== publish commit (local only unless --push)"
    git -C "$site_wt" show --stat --oneline --summary HEAD | sed -n '1,30p'
    say ""
    say "staged Studio tree ($(find "$site_wt/studio" -type f | wc -l | tr -d ' ') files):"
    ( cd "$site_wt" && find studio -maxdepth 3 -type f | sort | sed -n '1,40p' )
    if [ "$push" = 1 ]; then
        publish_studio "$site_wt" "$prev"
    else
        say ""
        say "DRY RUN: nothing pushed. To publish:"
        say "  Re-run this script with --target gh-pages --push --yes from a clean main checkout."
        say "The next docs-site book deploy keeps studio/: docs-site.yml deletes only non-studio root entries."
    fi
fi
say ""
say "done."
