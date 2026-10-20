# The desktop client (specification)

The desktop client is a planned app you install on your laptop. It shows the same interface as the web client, but it runs the model as native code rather than inside a browser, which should make it faster. It can open your project folders, it remembers your conversations between sessions in a durable local store, and it works with no internet connection. The target machine is an M1-class laptop. **This page is a specification, not a description of working software.** No desktop app project exists in the repository yet, and there is no trained model for it to run. The owner asked for a spec only in this round (decision 8, 19 October 2026). Nothing ships until a trained model passes milestones M1 to M8.

Labels used below: **proof** (derived from definitions or code), **measurement** (a recorded run with its artifact), **hypothesis** (untested). Words in this spec such as *must* are requirements, not claims about existing code. See the [glossary](../start/glossary.md).

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
| **Interface** | The Studio page, [`index.html`](../../index.html), shared with the web client | Yes, one 7,107-line file. It detects Tauri (`window.__TAURI__`) and calls `native_read_file`, `native_write_file`, `native_git_status`, `native_run_terminal_command` and `native_list_local_models`, falling back to browser APIs when they are absent |
| **Tauri project** | `src-tauri/` with `tauri.conf.json` and the Rust commands above | No. Nothing in the repository defines those commands |
| **Model engine** | [`crates/uor-r4-integer`](../../crates/uor-r4-integer/) linked as a library: the same code as the `uor-chat` command-line program | Yes, as a library and CLI, verified only on test and synthetic models |
| **Memory store** | [`crates/uor-r4-object-store`](../../crates/uor-r4-object-store/) | Yes, as a tested library; no crate uses it yet |
| **File-access layer** | New Rust commands limited to folders the user grants | No |
| **Model manager** | New: lists, verifies and loads `UORLUT01` files; checks for updates | No |

Sources: reading `index.html` and the repository tree, and the [object-store dossier](../mechanisms/object-store.md).

## Data flow

1. **Install.** The installer carries the app and, optionally, the model file. If the model is not bundled, the first run downloads it once ([update and download](#update-and-model-download)).
2. **Load.** The model manager reads the file, checks its SHA-256 against the release pin, and passes the bytes to the engine. The `UORLUT01` reader checks every offset and length and returns an error rather than crashing on a malformed file ([`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs); **proof** by reading the code).
3. **Chat.** The interface sends the user's text to a Rust command. The engine steps token by token with integer adds, shifts and table look-ups and streams tokens back to the page.
4. **Remember.** At the end of a turn, the app saves the session state as a `UR4SESS1` snapshot and stores conversation bytes in the object store. The store cuts bytes into 32-byte pieces arranged in a hashed three-way tree, so repeated content is stored once and any byte range can be read back and checked ([object-store dossier](../mechanisms/object-store.md)). Storing session snapshots in the store is a design note, not yet done.
5. **Resume.** On reopening, the app restores the snapshot, and the conversation continues token for token. That is the M3 restore bar ([project track](../integration/project-track.md), M3).
6. **Work on files.** When the user asks about code, the app reads files only from granted folders and passes their text to the model as context.

## Why native, and how fast

The browser runs the engine as WebAssembly in one worker. A native build can use the processor's vector units (NEON on Apple silicon) and several threads. The project already has native table kernels for NEON and AVX2, though they belong to an older, frozen engine (`crates/uor-r4-simd`). **That native will be faster is a hypothesis.** Nobody has timed WebAssembly against native on the same file. No decode speed exists for any D11 artifact. The only native D11 speeds on record are from older float models served exactly by the integer engine: 68 to 80 tokens per second on 8 threads for a 29M model, which is evaluation throughput over 64 scored windows rather than decode ([D11 serving record](../../archive/labs/d11-serving-2026-10-10/README.md)), and 184.58 tokens per second on one thread for input-driven forward passes, which is not free-running generation either ([lut-serving dossier](../mechanisms/lut-serving.md)). M5 is the milestone that measures laptop cost: ms per token, peak memory, mJ per token and weights read per token ([project track](../integration/project-track.md), M5).

Serving on the user's laptop is allowed. The charter says serving the exported D11 file on client devices, including Apple silicon, is the product, and D24's GPU-only rule does not apply ([charter](../../CHARTER.md), section 3.6).

## Durable memory (D19)

D19 makes grounded conversation and durable memory the first priority ([D19](../integration/DECISIONS.md#d19--grounded-conversation-and-durable-memory-first)). The object store was split out as its own crate for this purpose on 14 October (D34). What it has shown, all on test data (**measurement**):

- 60 tests pass, and a pure-Python client pulled a 256 KiB object byte-identically from the Rust server ([conformance record](../labs/uorobj01-conformance-2026-10-18/README.md)).
- A 23-byte read of a 32,768-byte object touched 8 authenticated nodes, 384 bytes ([object-store dossier](../mechanisms/object-store.md)).
- Repeated content compresses well (7,776 bytes stored in 380), but unique content expands about 2.17 times (32,768 bytes become 70,972) ([D19 store record](../../archive/labs/d19-object-store-2026-10-15/README.md)).

Never tested: any model reading from the store, the M3 memory bars on a real model, and storage cost on real chat logs ([object-store dossier](../mechanisms/object-store.md)). The store keeps exact identity (what was said, where, in which version) separate from any geometric address, as the charter requires (section 3.12).

## Security model: local only

| Risk | Requirement |
|---|---|
| Data leaving the device | No network access except the optional, user-visible update check. No telemetry. No remote model calls (as in M7). The shared Studio page currently loads CDN assets and sends web-search queries to Wikipedia and DuckDuckGo ([web client](./web-client.md)), so the desktop build must self-host every asset and drop or gate web search, or the "only an update check" claim is false |
| Broad file access | The Studio already calls `native_write_file`, so write access exists as well as read access. The backend may read and write only folders the user has explicitly granted, using Tauri's capability allowlist. No access outside them |
| Running shell commands | The Studio already calls `native_run_terminal_command`. This is the riskiest command. It must be off by default, show each command to the user before it runs, and be limited to a granted folder |
| A tampered model file | Load only files whose SHA-256 matches a pinned release hash. The reader already rejects malformed structure |
| A malformed saved session | The `UR4SESS1` parser checks varint counts before allocating and returns an error on truncated input ([record](../labs/wasm-serving-runtime-2026-10-18/README.md)) |
| Memory-store sync between machines | Pins prove integrity, not authentication. Peer sync stays off unless the user sets it up, and is out of scope for the first release ([object-store dossier](../mechanisms/object-store.md)) |
| Unsafe code | Keep `forbid(unsafe_code)` in the runtime and format crates ([charter](../../CHARTER.md), section 3.13) |

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
