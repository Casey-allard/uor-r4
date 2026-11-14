<!-- freshness: sources=bake-off,crates/uor-r4-float-train,crates/uor-r4-wasm-router,rules,scripts/pod,scripts/storage,src-tauri,tools/r4-softmax-trainer,web/studio checked=a70d6189df3368575e745d351ed692cb000e9388 -->
# How the repository is laid out

Every top-level folder has one job. This page lists them and the files at the top, and explains three names that are easy to confuse. The folders that the layout pass the owner approved on 20 October 2026 moved are named where they now live.

## Folders

| Folder | What it holds |
|---|---|
| `archive/` | Every dated record and retired file, byte for byte as it was. Nothing in it is edited. [`archive/CATALOG.md`](../../archive/CATALOG.md) indexes it. |
| `bake-off/` | Frozen bake-off records: for each contest, the socket (what a candidate takes and hands out), the control, and for the scored round the scoreboard and the pick. Today it holds the three reply-control rounds and `reply-control-v3.json`. |
| `crates/` | The Rust code, one crate per folder: the model, the serving runtime, the harness and the tools around them. `crates/uor-r4-float-train/` is the offline float training crate of the float-trained FBS trunk (D37; never part of serving). `crates/uor-r4-wasm-router/` is the browser-facing library (with its tests and server profiles); until 20 October 2026 it was the root package in `src/`, `tests/` and `profiles/`. |
| `data/` | The small evaluation panels (`data/panels/`) and four reference tokenizer descriptors that older tests compare against (`data/tokenizers/`, the old top-level `models/`). Large inputs (corpora, checkpoints) live in the private Hugging Face store instead. |
| `docs/` | The documentation you are reading, the reference documents (indexed in `docs/reference/`), the live lab records under `docs/labs/` and the harness files under `docs/harness/`. |
| `projects/` | Side projects related to UOR-R4 but not part of the model. See the [projects page](../../projects/README.md). |
| `rules/` | The two git hook scripts (`commit-msg-hook.sh`, `prepush-hook.sh`) that `uor-harness git install-hooks` installs. |
| `scripts/` | Shell entry points: GPU pods (`scripts/pod/`), storage and hygiene (`scripts/storage/`), harness wrappers. |
| `src-tauri/` | The desktop client's native Rust backend (Tauri). It is a separate Cargo package, excluded from the root workspace so workspace builds need no GUI libraries. |
| `third_party/` | Copies of outside code the build patches in: Rust crates such as candle and cudarc, and one reference implementation. Each records its upstream and file hashes. |
| `tools/` | `lab-runner` (a workspace crate), the small `uor-knowledge` server (a Python package and its notes), and one historical trainer (below). |
| `web/` | The web client. `web/studio/` is the Studio page served at `/studio/` on the documentation site. |
| `xtask/` | Repository tooling, run as `cargo xtask <command>`: link checks, the archive check, figures, the catalog and the site build. |
| `conformance/` | The requirements ledger (`ledger.toml`, `ids.toml`, `authorities.toml`) and its behaviour specifications (`features/suites/*.feature`). The `repo-model` and `repo-conformance` crates read it to produce `CONFORMANCE.md`. Until 20 October 2026 these were the top-level `model/` and `features/` folders. |
| `tests/` | End-to-end fixtures (`tests/e2e/`). The Rust tests live with their crates; the retired Python harness tests are archived under `archive/tests/harness/`. |

Reference documents (definitions, contracts, decision records and design specifications) are indexed in [`docs/reference/`](../reference/README.md). The decision records and design specifications moved there on 20 October 2026. The normative pages that code compiles in, or that evidence tables pin by path and hash, stay at their original paths in `docs/`, and the index links them where they are.

`tools/r4-softmax-trainer/` stays for now. It is a historical trainer that uses a banned method. One file in it is compiled into the core crate, and that core source file is itself pinned by hash in a recorded evidence table. Moving the trainer would break that record. It will move once that binding is retired.

## Hidden folders at the top

| Folder | Why it is there |
|---|---|
| `.github/` | Continuous integration and the pull-request template. |
| `.cargo/` | Cargo build settings for the WebAssembly target. |
| `.devcontainer/` | Editor container settings. Editors look for it at the top level. |
| `.agents/` | Shared skills for coding agents, in the layout agent tools expect. |
| `.kimi-code/`, `.codex-lab/` | Onboarding notes for the Kimi lab, and the Codex lab's old onboarding page, superseded by `CHARTER.md` and kept for history. |
| `.uor-models/` | The runtime's local model store. Only a manifest is tracked; the contents are gitignored. About ninety Rust source files read this path, so it stays. |

## Files at the top

`README.md`, `CHARTER.md` (the rules), `AGENTS.md` (onboarding for agents), `PLAN.md` (the current plan), `STATE.md` and `HANDOFF.md` (rendered by `uor-harness loop render`), `CONTRIBUTING.md`, `STATUS.md` (retired; it points to `STATE.md`), `ROADMAP.md`, `SECURITY.md`, `LICENSE`, `CITATION.cff` and the generated `CONFORMANCE.md`, plus build settings: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `Justfile` and `book.toml` (the documentation site).

## Names that are easy to confuse

- `conformance/` is a **requirements ledger**, not a language model. It was called `model/` until 20 October 2026 and was renamed to end that confusion.
- `data/tokenizers/` holds **reference tokenizer descriptors** used by older tests. It was the top-level `models/` until 20 October 2026.
- `.uor-models/` is the **runtime model store** on each machine. It is gitignored; its contents come from the private Hugging Face store.

## What is not in git

Build output (`target/`), each lab's scratch folder (`local/`), working folders for open pull requests (`.worktrees/`), large corpora (`*.u16`) and the runtime model store are gitignored on purpose. Where each kind of file lives, and the routine that keeps the laptop clean, are described in `docs/labs/workspace.md`.
