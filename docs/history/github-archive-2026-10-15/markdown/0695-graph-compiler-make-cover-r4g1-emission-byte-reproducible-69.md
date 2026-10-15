# #695 — graph-compiler: make cover.r4g1 emission byte-reproducible (#694)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T04:20:05Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-14T04:39:38Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/695
- **Merged:** 2026-08-14T04:39:38Z
- **Merge commit:** c900646ebe47d578d2ea66b5210600bbf46b592f
- **Base / Head:** main / UOR-Foundation:i694-cover-r4g1-byte-determinism
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #694.

## Problem
`emit_r4g1` built the per-region emission shortlist in `std::collections::HashMap` (`freq`, `bigram_freq`, `candidate_weights`) and sorted it with `sort_by_key(|&(_, w)| Reverse(w))` — a **weight-only** key. HashMap iteration is per-instance random-seeded, and `Reverse(weight)` is not a total order, so equal-weight tokens (and the token kept at the 64-candidate `truncate(64)` boundary) were ordered by per-run HashMap state. That perturbed the EMIT section bytes and flipped the artifact's content-addressed section CID, so `cover.r4g1` was not byte-reproducible across runs — the ~32-byte section-CID diff #694 reported.

The HEAD digests (teacher_cid, corpus_construction_cid, compiler-version) and the induced region structure / held-out recall were already deterministic; only the emission ordering leaked a seed to the wire.

## Fix
- Build `freq` / `bigram_freq` / `candidate_weights` as `BTreeMap` (key-order iteration).
- Sort the shortlist by a **total order**: weight descending, then token id ascending — so both ordering and the truncation boundary are seed-independent.

No behavior change beyond determinism: weights are unchanged (sums are order-independent), and the same top-64 tokens are emitted, now in a stable order.

## Verification
- `cargo fmt --check`, `cargo clippy --workspace --all-targets`: clean.
- `cargo test -p uor-r4-graph-compiler`: 144 tests pass.
- Real cover pipeline over the pinned GPT-2 snapshot, two runs (incl. a fresh process): `cover.r4g1` reproduces κ `blake3:aa591432…` **bit for bit**.
- The #670 increment-2 canary (`gpt2_cover_certifies_source_parity_rows`) is tightened from a size + structural-determinism check to a **byte-exact** `assert_eq!(bytes_a, bytes_b)`; presence-gated, so it stays a vacuous UNAVAILABLE pass in CI where the snapshot is absent.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

