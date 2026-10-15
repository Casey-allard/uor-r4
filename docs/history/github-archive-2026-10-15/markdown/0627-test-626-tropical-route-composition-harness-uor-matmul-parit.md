# #627 — test(#626): tropical route-composition harness + uor-matmul parity pin

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T00:42:11Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-12T01:01:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/627
- **Merged:** 2026-08-12T01:01:29Z
- **Merge commit:** 4f7f1a049f34f09d3e655a3b75b8406101e6ac07
- **Base / Head:** main / UOR-Foundation:i626-tropical-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of #626 (sub-issue of #622 / campaign #589). Ships the pre-registered experiment harness and the ecosystem parity pin; the measured result lands on #626 as a comment when the run completes (in flight at PR time).

## What

- `crates/uor-r4-graph-cli/tests/tropical_2hop_626.rs` — the #626 harness, `#[ignore]`d and fixture-gated with the #457-harness conventions (same `load_inputs`/split/support/bits rules, so rows are cross-comparable). Arms: NULL (root prior — must reproduce the #457 unigram floor or the instrument asserts broken), 1H and 2H tropical route-then-emit (token-dependent via destination-region ΔE), and a deranged-destination instrument null (deterministic rotation, no RNG). The tropical selection retains route witnesses; the harness prints them.
- `tropical_parity_with_uor_matmul` — un-ignored, runs in every `cargo test`: the harness's max-plus fold must agree exactly with `uor_matmul_core::dot_tropical_ref` on a deterministic corpus (finite mixes, -inf absorption in every slot pattern, empty folds, extreme magnitudes, an exhaustive small grid, and truncation ≡ -inf padding per their CK-17/A-6).
- `uor-matmul-core` (rev `b13c984`, default-features off, no_std, single bytemuck dep) added as a **dev-dependency of uor-r4-graph-cli only** — #622 item 2 exactly as scoped by the legality memo: dev-only semantic reference, never a dependency of shipped code. `deny.toml` allow-git gains the uor-matmul source with a comment marking it dev-only.

## Why

The #622 legality memo established the tropical path is P-4-legal arithmetic and the E_f seat is empty at serving (token-independent ⇒ argmax-neutral, score_runtime's own record). #626's first comment pre-registers the exit rule; this PR is the instrument. Runtime adoption is NOT part of this PR and only becomes a proposal if the pre-registered gate goes positive.

## Breaking changes

None. No shipped crate's dependency graph changes (dev-dep only); no runtime code; P-4 scan and allocation census untouched.

## Verification

1. `cargo test -p uor-r4-graph-cli --test tropical_2hop_626` → parity test green, experiment ignored (exit 0)
2. Four local gates on the branch: fmt 0, clippy `-D warnings` 0, no_std ladder unaffected, `cargo test --workspace` 0
3. `cargo deny` source allowlist extended deliberately (dev-only comment at the site)
4. Experiment run: `R4_TROP_HELD=5000 cargo test --release -p uor-r4-graph-cli --test tropical_2hop_626 -- --ignored --nocapture` — result posts to #626 with the pre-registered verdict either way


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

