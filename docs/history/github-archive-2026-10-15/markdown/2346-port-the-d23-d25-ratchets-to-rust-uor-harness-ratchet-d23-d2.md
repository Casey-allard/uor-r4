# #2346 — Port the D23/D25 ratchets to Rust: uor-harness ratchet d23|d25; D23 baseline 1648 -> 1647 (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T16:31:07Z
- **Updated:** 2026-10-13T16:37:48Z
- **Closed:** 2026-10-13T16:32:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2346
- **Merged:** 2026-10-13T16:32:55Z
- **Merge commit:** 0909f82b3ab9aded3b6353b24b4f77b0a4f512fe
- **Base / Head:** main / UOR-Foundation:claude/rust-ratchets-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2028 · D23 / D25 / D30 (Rust only)

**Line:** tooling · no model line · headlines unchanged. No model compute.

## What changed
- **New command:** `uor-harness ratchet d23|d25 [--update-baseline] [--list] [--quiet]` (`crates/uor-r4-harness/src/ratchet.rs`). It ports `scripts/check_no_{softmax,matmul}.py` with the same scan, comment/string stripper, baseline files and verdicts. `--update-baseline` may only lower counts.
- **No self-matching.** The banned word and the tensor-core needles are assembled with `concat!`, so the checker never counts against itself.
- **D23 baseline lowered 1648 → 1647.** `cuda_stack_kernels.rs` goes 6 → 5, the site #2344 removed.
- Releases the stale `claude/starling-rank-read-2026-10-13` claim (merged as #2338).

## Parity and checks at this head
- **D23:** the Rust rewrite of the 144-file baseline differs from the Python file only by the intended 6 → 5 line.
- **D25:** the rewritten 26-file, 93-site baseline is byte-identical.
- **Tests:** `cargo test -p uor-r4-harness` passes 138 + 1, including 6 new ratchet tests. `clippy --all-targets` and `fmt --check` are clean.
- **At this head:** `uor-harness ratchet d23` gives OK at 1647/1647, and `ratchet d25` gives OK at 93/93.

## Not here
- **DeepSeek (harness owner):** repoint prlint and CI at the Rust command, then retire the Python scripts (#2028 6150176928).
- **Lexical limit:** the ratchets cannot see a dense contraction written as scalar loops (Codex's #2344 findings). The quarantine of `selected_potential.cu` and `native_context.cu` follows in the next Claude PR.

Record: `docs/labs/rust-ratchets-2026-10-13/README.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6150278658 — 2026-10-13T16:31:12Z — Casey-allard

**Exact-head APPROVE at `a0e4ade83`** (self-review, owner rule 09-30). At this head: `cargo test -p uor-r4-harness` 138+1 pass (6 new ratchet tests), clippy/fmt clean, `uor-harness ratchet d23` OK 1647/1647, `ratchet d25` OK 93/93; baseline parity with the Python files per the record.


---
_Comment/review ids in this file: 6150278658_

