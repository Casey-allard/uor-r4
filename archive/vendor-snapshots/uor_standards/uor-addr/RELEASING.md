# Releasing uor-addr

> Operational guide for cutting a release. The mechanics are
> automated by [.github/workflows/release.yml](.github/workflows/release.yml);
> this document explains *what to do* (version bumps, secrets, tag
> push, post-release verification) and *what happens* (full job DAG +
> registry surface).

## Distribution surface

Each release publishes to **three crate registries** + **two language
registries** + attaches **N prebuilt artifacts** to the GitHub Release.
Every distribution target produces the same 71-byte ASCII
`sha256:<64hex>` κ-label byte-for-byte (pinned by CF-C\* / CF-W\* in
[CONFORMANCE.md](CONFORMANCE.md)).

| Distribution channel | Artifact | Source crate / package |
|---|---|---|
| [crates.io](https://crates.io) | `uor-addr` (Rust crate) | `crates/uor-addr/` |
| [crates.io](https://crates.io) | `uor-addr-c` (Rust crate; emits a C ABI cdylib + staticlib) | `crates/uor-addr-c/` |
| [crates.io](https://crates.io) | `uor-addr-wasm` (Rust crate; emits a WASM Component Model artifact) | `crates/uor-addr-wasm/` |
| [npm](https://www.npmjs.com/) | `@uor-foundation/uor-addr` (ESM + `.d.ts`) | `bindings/npm/` |
| [PyPI](https://pypi.org/) | `uor-addr` (per-platform wheels) | `bindings/python/` |
| GitHub Release | `uor_addr_wasm.wasm` — WASM Component Model artifact | built once in `release` job |
| GitHub Release | `uor_addr.h` — auto-generated C header | built once via `cbindgen` |
| GitHub Release | `libuor_addr_c-{linux-x86_64,linux-aarch64,macos-x86_64,macos-aarch64,windows-x86_64}-vX.Y.Z.{so,dylib,dll}` — per-platform prebuilt C ABI | matrix `prebuilt-c` job |

## Polyglot strategy

`uor-addr` ships **two FFI paths** that coexist:

1. **WASM Component Model** — primary polyglot path. Single
   `.wasm` artifact consumable from any language with a wasm
   runtime + Component Model support. The npm binding wraps this
   via [`jco`](https://github.com/bytecodealliance/jco).
2. **C ABI** — native FFI fallback for languages whose wasm
   runtimes don't yet ship Component Model support (Python's
   `wasmtime-py` as of v24.x is the current motivating example).
   Also the path for embedded toolchains and any C/C++ caller.
   The Python binding wraps this via stdlib `ctypes`.

Future language bindings should pick whichever path their ecosystem
runtime supports best:

- JS / TS / Deno / Bun → npm via `jco`-transpiled WASM.
- Python → PyPI via `ctypes` + C ABI (until wasmtime-py adds
  Component Model support; then pivot to WASM).
- Go → either `wasmtime-go` + WASM, or `cgo` + C ABI.
- .NET → either `Wasmtime.NET` + WASM, or P/Invoke + C ABI.
- Ruby → either `wasmtime-rb` + WASM, or `ffi` gem + C ABI.
- Embedded C/C++ / Cortex-M / ESP32 → C ABI only (`uor-addr-c`
  builds for `thumbv7em-none-eabihf` with no allocator).

## Cutting a release

### 1. Bump the release version (one edit + one script)

The single source of truth is `Cargo.toml`'s `[workspace.package].version`.
Every Rust crate in the workspace inherits it via `version.workspace =
true`; downstream bindings sync from it.

```bash
# 1. Edit the workspace version (one field, one file).
sed -i 's/^version       = ".*"/version       = "X.Y.Z"/' Cargo.toml
# (Or open Cargo.toml and edit `[workspace.package].version` by hand.)

# 2. Propagate to the non-Rust bindings + inter-crate dep pins.
python3 tools/sync-versions.py

# 3. Regenerate Cargo.lock (cargo picks up the new versions).
cargo update -w

# 4. Verify nothing drifted.
just version-sync
```

`tools/sync-versions.py` writes the new version into:

- `Cargo.toml`'s `[workspace.dependencies]` `uor-addr* { version = … }` entries,
- `bindings/npm/package.json`'s `"version"` field,
- `bindings/python/pyproject.toml`'s `[project] version` field.

CI's `version-sync` job runs the same script in `--check` mode on
every PR — drift between any of these and `[workspace.package].version`
fails the gate before merge.

### 2. Tag + push

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

The push fires [`release.yml`](.github/workflows/release.yml).

### 3. Watch the workflow

The job DAG:

```text
release
  ↓
  ├─ prebuilt-c (5-way matrix: linux × 2, macos × 2, windows)
  │    ↓
  │    publish-pypi (5-way matrix; downloads prebuilt-c artifacts)
  │
  └─ publish-npm
```

Steps inside the `release` job:

1. Validates the tag (`vX.Y.Z`) matches every crate + binding version.
2. Runs the V&V gate equivalent (fmt, clippy, workspace tests, embedded build, wasm build, doc).
3. `cargo publish --dry-run` for all three crates.
4. Creates the GitHub Release with auto-generated notes.
5. Publishes to crates.io in dependency order with registry-index wait.
6. Attaches `uor_addr_wasm.wasm` + `uor_addr.h` to the release.

Then `prebuilt-c` (matrix) builds the native libraries per OS+arch, uploads to the release, and uploads each to workflow artifacts for the `publish-pypi` matrix to consume.

`publish-npm` builds the wasm component, jco-transpiles, smoke-tests, and publishes with provenance attestation enabled.

`publish-pypi` per-platform: downloads its matching prebuilt-c artifact, bundles into the Python package, builds a platform-tagged wheel, smoke-tests, uploads to the release, publishes to PyPI via trusted publisher.

### 4. Verify

Within ~10 minutes of the workflow completing all jobs:

- crates.io shows `uor-addr` / `uor-addr-c` / `uor-addr-wasm` at the new version.
- docs.rs builds the docs for `uor-addr` (automatic on publish).
- `npm view @uor-foundation/uor-addr version` returns `X.Y.Z`.
- `pip install uor-addr==X.Y.Z` succeeds and the bundled .so/.dylib/.dll round-trips:
  ```bash
  python -c 'from uor_addr import kappa; assert kappa.json_address(b"{}") == "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"'
  ```
- The GitHub Release page shows: `uor_addr_wasm.wasm`, `uor_addr.h`, five `libuor_addr_c-*-vX.Y.Z.*` files, five `uor_addr-X.Y.Z-py3-none-*.whl` files.

## Secrets required

Configured under repo Settings → Secrets and variables → Actions:

| Secret | Used by | Provisioned via |
|---|---|---|
| `CARGO_REGISTRY_TOKEN` | crates.io publishes | <https://crates.io/me> → "API Tokens" |
| `NPM_TOKEN` | `npm publish` | <https://www.npmjs.com/settings/<user>/tokens> — granular token scoped to `@uor-foundation/uor-addr` |
| *(none)* for PyPI | `publish-pypi` uses **trusted publishing** | <https://pypi.org/manage/account/publishing/> — configure the `UOR-Foundation/uor-addr` repo + `release.yml` workflow + `publish-pypi` environment (no token needed) |
| *(none)* for npm provenance | `npm publish --provenance` uses GitHub OIDC | requires `id-token: write` permission (already set on the job) |

`GITHUB_TOKEN` is supplied automatically by Actions; it covers the `gh release upload` calls.

## Idempotence

Every publish step is idempotent:

- **crates.io publishes** — each step queries `crates.io/api/v1/crates/<name>/<version>` first; skips if the version is already on the registry.
- **npm publish** — checks `npm view @uor-foundation/uor-addr@<version> version`; skips if already published.
- **PyPI publish** — `pypa/gh-action-pypi-publish` with `skip-existing: true`.
- **GH Release uploads** — `gh release upload … --clobber` overwrites prior attempts.

A re-run of a partially-failed release picks up where it left off without manual cleanup.

## Yanking a release

If a release ships with a defect, yank (don't delete):

```bash
# crates.io
cargo yank --version X.Y.Z uor-addr
cargo yank --version X.Y.Z uor-addr-c
cargo yank --version X.Y.Z uor-addr-wasm

# npm
npm deprecate @uor-foundation/uor-addr@X.Y.Z "yanked: see vX.Y.Z+1"

# PyPI
# PyPI does not support yank; the next release supersedes.
```

Then ship X.Y.Z+1 with the fix.
