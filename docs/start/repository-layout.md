# How the repository is laid out

Every top-level folder has one job. This page lists them, says which ones are about to move, and explains three names that are easy to confuse. It describes the repository as of 20 October 2026; the owner approved a layout pass that day, and this page changes with each step of it.

## Folders

| Folder | What it holds |
|---|---|
| `archive/` | Every dated record and retired file, byte for byte as it was. Nothing in it is edited. [`archive/CATALOG.md`](../../archive/CATALOG.md) indexes it. |
| `crates/` | The Rust code, one crate per folder: the model, the serving runtime, the harness and the tools around them. |
| `data/` | The small evaluation panels kept in git. Large inputs (corpora, checkpoints) live in the private Hugging Face store instead. |
| `docs/` | The documentation you are reading, plus the live lab records under `docs/labs/` and the harness files under `docs/harness/`. |
| `projects/` | Side projects related to UOR-R4 but not part of the model. See the [projects page](../../projects/README.md). |
| `scripts/` | Shell entry points: GPU pods (`scripts/pod/`), storage and hygiene (`scripts/storage/`), harness wrappers. |
| `third_party/` | Copies of outside Rust crates the build patches in. Their contents are pinned by hash. |
| `tools/` | `lab-runner` (a workspace crate), notes for the `uor-knowledge` server, and one historical trainer (below). |
| `web/` | The web client. `web/studio/` is the Studio page served at `/studio/` on the documentation site. |
| `xtask/` | Repository tooling, run as `cargo xtask <command>`: link checks, the archive check, figures, the catalog and the site build. |
| `model/` | The requirements ledger: requirement ids, authorities and `ledger.toml`. The `repo-model` and `repo-conformance` crates read it to produce `CONFORMANCE.md`. **Moving to** `conformance/`. |
| `features/` | Behaviour specifications (`.feature` files) for those requirements. **Moving to** `conformance/features/`. |
| `models/` | Four reference tokenizer files that older tests compare against. **Moving to** `data/tokenizers/`. |
| `src/`, `tests/`, `profiles/` | The root Rust package, `uor-r4-wasm-router`: its library, its tests and its server profiles. **Moving to** `crates/uor-r4-wasm-router/`, so the top level holds only workspace settings. |

One more change is planned: the normative reference pages that sit loose in `docs/` will gather under `docs/reference/`, except for the few that code pins by path and hash, which stay where they are.

`tools/r4-softmax-trainer/` stays for now. It is a historical trainer that uses a banned method. One file in it is compiled into the core crate, and that core source file is itself pinned by hash in a recorded evidence table. Moving the trainer would break that record. It will move once that binding is retired.

## Hidden folders at the top

| Folder | Why it is there |
|---|---|
| `.github/` | Continuous integration and the pull-request template. |
| `.cargo/` | Cargo build settings for the WebAssembly target. |
| `.devcontainer/` | Editor container settings. Editors look for it at the top level. |
| `.agents/` | Shared skills for coding agents, in the layout agent tools expect. |
| `.kimi-code/`, `.codex-lab/` | Onboarding notes for two of the labs. |
| `.uor-models/` | The runtime's local model store. Only a manifest is tracked; the contents are gitignored. About ninety Rust source files read this path, so it stays. |

## Files at the top

`README.md`, `CHARTER.md` (the rules), `AGENTS.md` (onboarding for agents), `CONTRIBUTING.md`, `STATUS.md`, `ROADMAP.md`, `SECURITY.md`, `LICENSE`, `CITATION.cff` and the generated `CONFORMANCE.md`, plus build settings: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `Justfile` and `book.toml` (the documentation site).

## Three names that are easy to confuse

- `model/` is a **requirements ledger**, not a language model. It is moving to `conformance/` to end the confusion.
- `models/` holds **reference tokenizers** used by older tests. It is moving to `data/tokenizers/`.
- `.uor-models/` is the **runtime model store** on each machine. It is gitignored; its contents come from the private Hugging Face store.

## What is not in git

Build output (`target/`), each lab's scratch folder (`local/`), working folders for open pull requests (`.worktrees/`), large corpora (`*.u16`) and the runtime model store are gitignored on purpose. Where each kind of file lives, and the routine that keeps the laptop clean, are described in `docs/labs/workspace.md`.
