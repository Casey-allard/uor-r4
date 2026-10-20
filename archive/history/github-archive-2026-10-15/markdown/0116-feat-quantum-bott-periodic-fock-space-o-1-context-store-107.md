# #116 — feat(quantum): Bott periodic Fock space O(1) context store (#107)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T17:30:33Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-07-23T17:39:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/116
- **Merged:** 2026-07-23T17:39:48Z
- **Merge commit:** d1b1a2aa12e6b4b1f4e696f0ba26804568f23a5c
- **Base / Head:** main / UOR-Foundation:issue-107-bott-fock-context
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #107

## What

`crates/uor-r4-core/src/transformerless/bott_fock.rs` — Bott periodic Fock space context store per `docs/r4_furey_quantum_geometric_plan.md` §Phase 4. Folds an arbitrary N-token context window into a fixed `[i16; 256]` state matrix: O(1) space and O(1) update time **by construction** (fixed array, one fixed 256-cell integer sweep per append, no KV cache, no allocation).

Update rule: contractive decay (each cell sheds a quarter) + position-sensitive injection (row embedding rotated by column index, XOR-mixed with the column embedding, quarter-strength, saturating). XOR/rotate/arithmetic-shift/saturating-add only — no mul/div/mod/float in the update path, machine-checked by a P-4-style self-scan test in the module.

Honesty note: the fold is lossy — an abstraction of the context, not a reversible encoding. Documented as such in the module.

## Gate F evidence (release benchmark, `tests/bott_fock_scaling.rs`)

| N (context tokens) | total | ns/append |
| ---: | ---: | ---: |
| 1,000 | 157.6 µs | 157 |
| 10,000 | 1.63 ms | 162 |
| 100,000 | 14.80 ms | 148 |
| 1,000,000 | 146.89 ms | 146 |

Per-append spread 146–162 ns (ratio ≈ 1.1) — flat across three orders of magnitude; the test asserts max ≤ 3× min, which still catches any accidental O(N) work. Always-on half of the test asserts store size never grows across 100k appends.

## Tests

- append determinism + token counting; fixed-point convergence under repeated embedding; order sensitivity; P-4 source self-scan
- `store_size_is_independent_of_token_count` (always-on), `append_latency_is_constant_across_context_lengths` (release, ignored by default)

Run: `cargo test -p uor-r4-core --release --test bott_fock_scaling -- --ignored --nocapture`

## Verification

- `cargo test --workspace --offline`: green
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
- `cargo fmt --check`: clean

Refs #103 (tracker), #105 (operator store conventions)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

