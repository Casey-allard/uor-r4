# #233 — Legacy pipeline documentation is stale (compile/store no longer part of the legacy chain)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @maurathat
- **Created:** 2026-07-27T03:26:39Z
- **Updated:** 2026-07-28T02:41:40Z
- **Closed:** 2026-07-28T02:41:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/233
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The documented legacy workflow (README.md, "Legacy benchmark and certification workflow") no longer matches the CLI at HEAD.

- `cargo run --release -- compile` now resolves to the HF graph-compiler path and fails without `--model` or `--source`: `ERROR r4: message=command failed, error=pass --model <HF_REPOSITORY> or --source <DIRECTORY>`
- `cargo run --release -- store` panics: `crates/uor-r4-graph-cli/src/lib.rs:1710:44: run 'cargo run --release -- compile' first` (exit 101)
- The repo's own `setup` output documents the current chain as `gen → certify → compare → compare-report`, with `certify` performing compile, store, certificate, and census internally.

Also worth recording in the same pass: the gen path is not byte-reproducible across machines or eras — this run produced 754 stories / 30,036 held-out against a certified 757 / 30,192, which bounds how exactly any downstream figure can reproduce.

Docs-only PR to follow against this issue.

## Environment

Commit `0ffb71b94d78ff722af089f1337a08fce42dccff` (HEAD of main, 2026-07-26). rustc 1.95.0, `--release`. macOS 26.5.1, arm64.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

