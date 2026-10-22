# The desktop client

The desktop client is a pre-alpha native shell you install on your laptop. It shows the same interface as the web client, but it runs the model engine as native Rust code rather than inside a browser worker. It opens only user-granted project folders, remembers your conversations between sessions in a durable local object store, and runs 100% offline with zero external CDN requests. The target machine is an M1-class laptop. **No trained model exists for it to run.** The project is pre-alpha: 0 of 8 milestones are complete, and the `src-tauri/` desktop shell is verified only on the 469,252-byte `mk-synthetic-stack` (`seed = 5`) synthetic stand-in artifact ([record](../labs/infra-desktop-client-tauri/README.md)). Nothing ships until a trained model passes milestones M1 to M8.

Labels used below: **proof** (derived from definitions or code), **measurement** (a recorded run with its artifact), **hypothesis** (untested). See the [glossary](../start/glossary.md).

![A laptop outline split in two. Left half, "webview": the shared Studio interface (editor, chat, model manager). Right half, "Rust core": four boxes — the integer model engine (uor-r4-integer), the memory store (uor-r4-object-store), a scoped file-access layer, and a model manager with an update checker. Arrows between the halves are labelled "Tauri commands". The model file and the memory store sit on a disk icon inside the laptop. A single dashed arrow leaves the laptop, labelled "optional: check for a new model release", and nothing else crosses the border.](../figures/products/desktop-client-architecture.svg)

*Same interface as the web client; the engine, the memory and the file access run as native Rust inside the app. Only an optional update check uses the network.*

## Goals and non-goals

**Goals.**

1. Run the released `UORLUT01` model natively, under the D11 integer contract, on an M1-class laptop.
2. Share the web client's interface, so there is one interface to maintain.
3. Give the model access to the user's chosen project folders, for coding help.
4. Keep durable conversation memory on disk (the D19 line), so a conversation can be closed and resumed exactly.
5. Work fully offline after install.

**Non-goals.** No cloud model, no account, no telemetry, no server component, and no second model format. Training and evaluation stay on GPUs ([charter](../../CHARTER.md), section 3.5).

## Architecture

The app is built with **Tauri**: a small native shell that shows a web page in the system's own webview and runs a Rust backend beside it. The page calls the backend through named commands. Rust is the project's default language ([charter](../../CHARTER.md), section 3.11), so the backend can link the project's crates directly.

| Component | Built from | Exists today? |
|---|---|---|
| **Interface** | The Studio page, [`index.html`](../../web/studio/index.html), shared with the web client, plus self-hosted offline vendor shims ([`uor_offline_vendor.js`](../../web/studio/assets/js/uor_offline_vendor.js) and [`uor_offline_fonts.css`](../../web/studio/assets/css/uor_offline_fonts.css)) | Yes. It detects `window.__TAURI__.core` (or connects via `--serve-studio` loopback HTTP `/__tauri_invoke` authenticated with a per-launch `X-UOR-Desktop-Token` and `Host`/`Origin`/`Content-Type: application/json` checks) and calls `native_list_granted_folders`, `native_read_file`, `native_write_file`, `native_list_directory`, `native_git_status`, `native_run_terminal_command`, `native_list_local_models`, `native_load_model`, `native_step_tokens`, `native_save_session_durable` and `native_restore_session_durable`, streaming tokens step-by-step and falling back to browser APIs when absent |
| **Desktop backend (`src-tauri/`)** | [`src-tauri/`](../../src-tauri/) with [`tauri.conf.json`](../../src-tauri/tauri.conf.json), [`capabilities/default.json`](../../src-tauri/capabilities/default.json), [`build.rs`](../../src-tauri/build.rs) and [`src/lib.rs`](../../src-tauri/src/lib.rs) | In progress (`#2826`): the zero-GUI-crate native Rust backend (`DesktopBackend`), `--stdio` JSON-RPC bridge, and `--serve-studio` loopback Studio host are built and verified across 6 integration controls ([record](../labs/infra-desktop-client-tauri/README.md)); linking the `tauri` WebView crate (`tauri::Builder`, `#[tauri::command]`) awaits vendor-fetching the Tauri v2 crate tree into `Cargo.lock` |
| **Model engine** | [`crates/uor-r4-integer`](../../crates/uor-r4-integer/) linked as a library: the same `IntegerStackModel` / `LutStackSession` engine as `uor-chat` and `serving-wasm`, plus [`mk-synthetic-stack.rs`](../../crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs) compiled via [`src-tauri/build.rs`](../../src-tauri/build.rs) | Yes, linked directly into `src-tauri` and verified on the `seed = 5` synthetic stand-in (`32`-token trajectory matches `docs/wasm-serving/verify.mjs` token-for-token) |
| **Memory store** | [`crates/uor-r4-object-store`](../../crates/uor-r4-object-store/) (`UORMEM01` + `UR4SESS1` envelope) | Yes, linked into `src-tauri`: `native_save_session_durable` and `native_restore_session_durable` fold and verify `UORMEM01` envelopes + `UR4SESS1` session snapshots in `uor-r4-object-store::Store` inside user-granted folders |
| **File-access layer** | `granted_folders` allowlist + symlink refusal (`reject_if_symlink`, `reject_symlinks_between_root_and_target`) in [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs) | Yes: `native_read_file` (capped at `16 MiB`), `native_write_file`, `native_list_directory`, `native_git_status` (`-c core.fsmonitor=false -c core.hooksPath=/dev/null`) and `set_durable_store_dir` canonicalize paths, refuse symlinks/dangling symlinks, and refuse any target outside user-granted roots |
| **Model manager** | `native_list_local_models` and `native_load_model` in [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs) | Yes: stream-checks the 8-byte `b"UORLUT01"` magic header before reading, skips unpinned/non-`UORLUT01` files so unrelated `.bin` files cannot hide pinned models, and checks SHA-256 against `PinRegistry` before passing bytes to `format.rs` |

Sources: [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs), [`web/studio/index.html`](../../web/studio/index.html), the [desktop client record](../labs/infra-desktop-client-tauri/README.md), and the [object-store dossier](../mechanisms/object-store.md).

## Data flow

1. **Install.** The installer carries the app and, optionally, the model file. If the model is not bundled, the first run downloads it once ([update and download](#update-and-model-download)).
2. **Load.** The model manager reads the file, checks its SHA-256 against the release pin before parsing, and passes the verified bytes to `LoadedLutStack::from_bytes`. The `UORLUT01` reader checks every offset and length and returns an error rather than crashing on a malformed file ([`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs); **proof** by reading the code).
3. **Chat.** The interface sends the user's prompt tokens to `native_step_tokens`. The engine steps token by token with integer adds, shifts and table look-ups (`LutStackSession::step_token`) and streams tokens back to the page.
4. **Remember.** At the end of a turn, `native_save_session_durable` snapshots the session state as `UR4SESS1` (`9,782 B` after 32 steps on the synthetic stand-in), wraps the conversation history and snapshot in a `UORMEM01` envelope (`encode_memory_envelope` + `fold_memory_envelope`), and stores the content-addressed records (`446` records on the 32-token self-test, `object_version = bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca`) in `uor-r4-object-store::Store` ([record](../labs/infra-desktop-client-tauri/README.md)).
5. **Resume.** On reopening, `native_restore_session_durable` reconstructs and verifies the `UORMEM01` envelope from the local store (`verify_memory_envelope`), loads the embedded `UR4SESS1` snapshot back into `LutStackSession::from_session_bytes`, and continues generation token-for-token (`16 + 16 == 32` verified in `durable_memory_object_store_close_and_resume_continues_token_for_token`). That is the M3 restore contract on the synthetic stand-in; M3 itself remains `0/8` until a trained model passes the full M3 acceptance panel ([project track](../integration/project-track.md), M3).
6. **Work on files.** When the user launches the host with `--grant <dir>` (or selects a folder through a host-owned dialog), `native_grant_folder` canonicalizes and registers it in `GrantedFolders` (`native_grant_folder` is host-owned and refused over page RPC); `native_read_file`, `native_write_file`, `native_list_directory` and `native_git_status` refuse any path outside granted roots.

## Why native, and how fast

The browser runs the engine as WebAssembly in one worker. A native build can use the processor's vector units (NEON on Apple silicon) and several threads. The project already has native table kernels for NEON and AVX2, though they belong to an older, frozen engine (`crates/uor-r4-simd`). **That native will be faster is a hypothesis.** Nobody has timed WebAssembly against native on the same file. No decode speed exists for any D11 artifact. The only native D11 speeds on record are from older float models served exactly by the integer engine: 68 to 80 tokens per second on 8 threads for a 29M model, which is evaluation throughput over 64 scored windows rather than decode ([D11 serving record](../../archive/labs/d11-serving-2026-10-10/README.md)), and 184.58 tokens per second on one thread for input-driven forward passes, which is not free-running generation either ([lut-serving dossier](../mechanisms/lut-serving.md)). M5 is the milestone that measures laptop cost: ms per token, peak memory, mJ per token and weights read per token ([project track](../integration/project-track.md), M5).

Serving on the user's laptop is allowed. The charter says serving the exported D11 file on client devices, including Apple silicon, is the product, and D24's GPU-only rule does not apply ([charter](../../CHARTER.md), section 3.6).

## Durable memory (D19)

D19 makes grounded conversation and durable memory the first priority ([D19](../integration/DECISIONS.md#d19--grounded-conversation-and-durable-memory-first)). The object store was split out as its own crate for this purpose on 14 October (D34). What it has shown, all on test data (**measurement**):

- 60 tests pass, and a pure-Python client pulled a 256 KiB object byte-identically from the Rust server ([conformance record](../labs/uorobj01-conformance-2026-10-18/README.md)).
- A 23-byte read of a 32,768-byte object touched 8 authenticated nodes, 384 bytes ([object-store dossier](../mechanisms/object-store.md)).
- Repeated content compresses well (7,776 bytes stored in 380), but unique content expands about 2.17 times (32,768 bytes become 70,972) ([D19 store record](../../archive/labs/d19-object-store-2026-10-15/README.md)).
- `src-tauri` folds and restores `UORMEM01` + `UR4SESS1` session state through `uor-r4-object-store::Store` (`446` records for a 32-token synthetic session, `9,782 B` `UR4SESS1` snapshot), and resumes generation token-for-token across process close/reopen ([desktop client record](../labs/infra-desktop-client-tauri/README.md)).

Never tested: any trained model reading from the store, the M3 memory bars on a trained model, and storage cost on real chat logs ([object-store dossier](../mechanisms/object-store.md)). The store keeps exact identity (what was said, where, in which version) separate from any geometric address, as the charter requires (section 3.12).

## Security model: local only

| Risk | Requirement and implementation (`#2826`) |
|---|---|
| Data leaving the device | No network access except an optional, user-visible update check. No telemetry. No remote model calls (as in M7). [`web/studio/index.html`](../../web/studio/index.html) self-hosts all vendor functionality in [`uor_offline_vendor.js`](../../web/studio/assets/js/uor_offline_vendor.js) and [`uor_offline_fonts.css`](../../web/studio/assets/css/uor_offline_fonts.css) (`0` external `http(s)://` URLs), gates `callGithubApi` off in desktop/offline mode while preserving online browser mode, and [`tauri.conf.json`](../../src-tauri/tauri.conf.json) sets `connect-src 'self' ipc: http://ipc.localhost` (`'unsafe-inline'` in `script-src` is retained for existing inline DOM event attributes in `index.html`) |
| Broad file access | `native_grant_folder` is host-owned (`--grant <dir>` CLI flag on `uor-desktop-client`, refused over page RPC). `native_read_file` (capped at `16 MiB`), `native_write_file`, `native_list_directory`, `native_git_status` (`-c core.fsmonitor=false -c core.hooksPath=/dev/null`) and `set_durable_store_dir` check canonical paths against `granted_folders` in [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs). Paths outside user-granted roots, `..` traversal, symlinks and dangling symlinks (`DesktopClientError::SymlinkRefused`) are refused. On `--serve-studio 127.0.0.1:<port>`, `POST /__tauri_invoke` additionally requires a per-launch random `X-UOR-Desktop-Token`, `Host: 127.0.0.1:<port>`, `Origin: http://127.0.0.1:<port>`, and `Content-Type: application/json` to block cross-origin browser requests and DNS rebinding |
| Running shell commands | `native_run_terminal_command` is disabled by default (`terminal_enabled = false`), requires host-owned enablement (`native_set_terminal_enabled` is not callable via page RPC) plus host-owned per-command approval (`host_approve_terminal_command`) and matching page confirmation, executes only the allowlisted binary (`cargo`, `git`, `ls`, `pwd`, `cat`, `head`, `tail`, `wc`, `grep`, `rg`, `find`, `echo`, `uor-chat`, `uor-harness`) directly via `Command::new(program).args(...)` (no `sh -c`), and confines every path-like argument (`/`-containing, `.`-prefixed, or `~`-prefixed) to `granted_folders` |
| A tampered model file | `native_load_model` verifies `b"UORLUT01"` magic and checks SHA-256 against `PinRegistry` before calling `IntegerStackModel::from_bytes`; `native_list_local_models` stream-reads the 8-byte header first and skips unpinned files so unrelated `.bin` files do not abort discovery |
| A malformed saved session | The `UR4SESS1` parser checks varint counts before allocating and returns an error on truncated input ([record](../labs/wasm-serving-runtime-2026-10-18/README.md)), and `verify_memory_envelope` authenticates the `UORMEM01` store tree before restore |
| Memory-store sync between machines | Pins prove integrity, not authentication. Peer sync stays off unless the user sets it up, and is out of scope for the first release ([object-store dossier](../mechanisms/object-store.md)) |
| Unsafe code | `#![forbid(unsafe_code)]` is enforced in [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs) as well as the runtime and format crates ([charter](../../CHARTER.md), section 3.13) |

### Offline vendor shim scope (`uor_offline_vendor.js`)

To eliminate all 9 external CDN `<script>` and `<link>` dependencies from [`web/studio/index.html`](../../web/studio/index.html) without committing multi-megabyte minified third-party bundles, [`web/studio/assets/js/uor_offline_vendor.js`](../../web/studio/assets/js/uor_offline_vendor.js) provides self-hosted local shims verified by [`web/studio/verify_desktop_client.mjs`](../../web/studio/verify_desktop_client.mjs):

- **`monaco.editor` + AMD `require`:** `<textarea>`-backed code editor supporting `createModel`, `setModelLanguage`, `getModelLanguage`, `create`, and a line-by-line LCS unified diff viewer (`createDiffEditor` rendering `.uor-diff-del` removed lines, `.uor-diff-ins` added lines, and `getLineChanges()`), without full Monaco token syntax highlighting or IntelliSense.
- **`marked`:** Renders fenced code blocks, headings (`#`–`####`), unordered and ordered lists, blockquotes, pipe tables, links (blocking `javascript:` URLs), bold, italics, and inline code.
- **`katex` + `renderMathInElement`:** Formats fractions (`\frac{a}{b}`), square roots (`\sqrt{x}`), superscripts (`^`), subscripts (`_`), Greek letters, and mathematical operators into structured `.katex` spans.
- **`THREE`:** 2D-canvas software 3D projection shim (`Scene`, `PerspectiveCamera`, `WebGLRenderer`, `Points`, `LineSegments`, `Line`, `OrbitControls`, `BufferGeometry`, `Vector3`, `Color`) driving the Studio's 3D lattice visualizer without external WebGL bundle fetches.
- **`pdfjsLib`:** Extracts printable ASCII text runs from simple PDF buffers and emits an explicit offline notice that binary PDF stream/font rasterization is not bundled offline.

## Update and model download

- **Model files are versioned releases.** Each is one `UORLUT01` file with a published SHA-256 ([releases](./releases-and-artifacts.md)). The app never runs an unpinned file.
- **First download.** Fetch the file once, verify the hash, store it locally. After that the app needs no network.
- **App updates.** Use Tauri's signed updater, or manual install. Signing keys and the hosting choice are open questions.
- **Delta updates for models (hypothesis).** The object store can send only the pieces that changed. In one test a receiver holding a slightly older 1 MiB object fetched 22 records, 1,014 bytes, in 3 rounds. But a cold pull of unique content through the store is far larger than a compressed download (5,725,408 bytes against 39,920 for a 2 MiB artifact), so the supported recipe is a compressed download first, then deltas ([object-store dossier](../mechanisms/object-store.md)). Whether deltas help for real model files is untested.

## Milestones it depends on

| Milestone | What the desktop client needs from it |
|---|---|
| M3 `#2031` | Durable memory that passes provenance, correction, abstention, reset and restart, with identical restore |
| M4 `#2032` | A trained model exported to `UORLUT01` and served by `uor-chat`, which is the engine the app embeds |
| M5 `#2033` | Measured laptop cost: speed, memory, energy |
| M7 `#2035` | The shared interface, and bit-identical tokens across library, command line and browser |
| M8 `#2036` | A versioned, checksum-pinned release to download |

## Open questions

1. **Superseding the old plan.** [`archive/integration/frontend-port-plan.md`](../../archive/integration/frontend-port-plan.md) (historical, 4 September) says Tauri should not be the main client. The owner's 19 October decision replaces it, and that file needs a supersession note.
2. **Sharing one interface.** The Studio is one 7,107-line HTML file with inline scripts. Does it need to be split into modules before two shells can share it cleanly?
3. **The terminal command.** Keep it (with consent and folder limits) or drop it from the first release?
4. **Where memory meets the model.** Which comes first: storing session snapshots in the object store, or exact quotations from the store feeding the model's output? This is the object store's open path to ACTIVE status ([object-store dossier](../mechanisms/object-store.md)).
5. **Threads and D11.** The five D11 rules say nothing about threads. Whatever kernels the app ships, threaded or vectorized, must still pass the zero-multiply audit.
6. **Engine format.** The legacy Studio also calls `native_list_local_models`, written for an older model format. The model manager must accept only `UORLUT01`.

## Go deeper

- [Web client](./web-client.md), which shares the interface; [products overview](./README.md).
- [Exact object store dossier](../mechanisms/object-store.md) and its [protocol](../../crates/uor-r4-object-store/PROTOCOL.md).
- [Serving and export](../model/serving-and-export.md); [integer serving dossier](../mechanisms/lut-serving.md).
- [Charter](../../CHARTER.md), sections 2 and 3.6.
