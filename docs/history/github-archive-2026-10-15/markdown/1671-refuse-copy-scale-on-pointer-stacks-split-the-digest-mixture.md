# #1671 — Refuse copy scale on pointer stacks; split the digest mixture hash; share MAX_POINTER_DIM (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T04:18:55Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-10-04T04:22:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1671
- **Merged:** 2026-10-04T04:22:30Z
- **Merge commit:** 01b7f57dae0f3f12bc73125675970f4ba0a5f5cb
- **Base / Head:** main / UOR-Foundation:claude/pointer-followups
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-ups to #1669 (integer pointer-copy head in both stack engines). References #820.

## Changes
1. **Copy scale vs pointer head** (`crates/uor-r4-integer/src/stack/session.rs`): `IntegerStackSession::set_copy_scale` now returns `Result<(), StackError>`. On a model with a pointer head it refuses any positive scale with the new `StackError::CopyScaleWithPointer`. Zero, or a negative scale that clamps to zero, is still accepted. `restore_state` also refuses a pointer-model snapshot whose `copy_scale_q16 != 0`. Without this, D11 would build the pointer mixture from boosted logits that D10 never has, which breaks D10 == D11. Plain models are unchanged; the existing copy-scale unit tests now `.expect` the `Ok`.
2. **`uor-r4-stack digest`**: `logits_sha256` hashes logits only again. A pointer model's record adds `mixture_sha256`: the SHA-256 of every step's Q30 mixture, little-endian `i32`, in order. `pointer_mixture_hashed` is kept, so a plain record has the same keys as on main; it now means that `mixture_sha256` is present.
3. **`MAX_POINTER_DIM`**: D10's `StackPointer::valid` now uses a new `uor_r4_lut::format::MAX_POINTER_DIM` (256) in place of the hard-coded `1..=256`. Neither crate depends on the other, so the oracle test asserts that the two constants are equal.

## Checks run at this head (macOS arm64 laptop, `CARGO_TARGET_DIR` outside the repo)
- `cargo fmt --all --check`: pass
- `cargo test -p uor-r4-integer --lib`: 204 passed
- `cargo test -p uor-r4-lut --lib`: 25 passed
- `cargo test --release -p uor-r4-training --test stack_d11_oracle`: 13 passed, including the new `a_pointer_model_refuses_a_copy_scale` and `both_engines_share_the_pointer_width_limit`
- `python3 scripts/audit_zero_matmul_serving.py --stack <release uor-r4-stack>`: **FULL PASS** (0 forbidden instructions across 36 symbol ranges and 39 reachable call-graph functions)
- Digest, comparing the release `uor-r4-stack` at origin/main cacb54c4 with this branch on 2 windows of TinyStories-v2 validation (`seconds` and `tokens_per_second` excluded):
  - plain `/1` artifact (`d11-l2`, sha `d5d8acc1…`): records **identical** (`logits_sha256` `bbd45536…`)
  - pointer `/2` artifact (`d11-pointer`, sha `4e5b683b…`): `logits_sha256` changes from `abf928c1…` (logits plus mixture) to `1e82abbb…` (logits only), and `mixture_sha256` = `6f762c82…` is added

Cost: about 5 min of laptop wall time (mostly the cold release build of the oracle test's candle dependencies), 3 build jobs, registered in `locks/jobs` and released afterwards.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5976515072 — 2026-10-04T04:19:04Z — Casey-allard

Exact-head evidence: every check in the PR description ran locally at head `6db3f3f6b0e7eca2413e6b2358d06304ab627fc6` (fmt; integer lib 204/204; lut lib 25/25; release stack_d11_oracle 13/13; audit FULL PASS on this head's release uor-r4-stack; digest comparison against cacb54c4).


---
_Comment/review ids in this file: 5976515072_

