# #61 — router: eliminate usize at wasm/JSON serialization boundaries (issue #12 remainder)

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-22T18:55:38Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-07-22T19:53:49Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/61
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:copilot/fix-width-newtypes-serialization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Router result structs were carrying `usize` across the wasm/JSON boundary — a platform-width type that is semantically wrong at serialization boundaries. This completes the fixed-width newtype work from issue #12 for the router crate (the core/graph-format side was already done).

## Changes

### `crates/uor-r4-router/src/lib.rs` — result struct field types
All serialized result structs get `usize` → `u32`:

| Struct | Fields changed |
|---|---|
| `RoutedResult` | `window_index`, `active_range` |
| `QimcResult` | `prime`, `index` |
| `UorControlPlanInfo` | `hopf_chi_bins` |
| `HopfResult` | `hopf_chi_bins`, `sector_id` |
| `RouteInfo` | `window_index`, `active_range` |
| `ResonanceResult` | `window_index` |
| `TrajectoryStep` | `step`, `window_index` |
| `QuantumMetrics` | `stratum`, `cascade_length` |
| `DihedralInfo` | `s`, `k` |
| `UorR4Router.expert_active_counts` | `Vec<usize>` → `Vec<u32>` |

`get_expert_counts()` return type updated to `Vec<u32>` accordingly.

**Construction sites** get `as u32` casts where internal `usize` values (routing indices, QIMC primes, Hopf sector) are stored into the new fields. **Consumption sites** (slice indexing, `HashMap<usize,_>` keys) keep `usize` via `as usize` casts — internal computation types are untouched.

The `catastrophe` comparison simplified from `(stratum as i32 - prev_stratum as i32).abs()` to `(stratum - prev_stratum).abs()` after `prev_stratum` is explicitly typed `i32` (was a clippy `unnecessary_cast` with the new inference).

### `src/server.rs` — call-site updates
- `get_window_theme(win_idx: usize)` → `u32` (used exclusively with `window_index`)
- `CliAnswer.window_index: usize` → `u32`
- Two `get_sentence_projection_native` call sites get `as usize` cast (function signature stays `usize`-typed for array indexing)

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #12

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5050782922 — 2026-07-22T19:53:48Z — auser

Superseded by e3ddd7d (already in main), which completed the full fixed-width conversion with u64 across all serde-serialized router structs and state maps, with exact-JSON regression tests and proof that the pre-change manifold cache still imports. The one unique improvement here — get_expert_counts going fixed-width u32 instead of keeping the usize ABI — was adopted and pushed as 9d91473 (u32 keeps the JS dashboard on Uint32Array; u64 would have flipped it to BigInt). Everything else in this PR duplicates the merged work, so closing; thanks for the nudge on the export type.

### comment 5050786800 — 2026-07-22T19:54:10Z — auser

Correction to the previous comment: the get_expert_counts u32 refinement was pushed as 98b1cbe (not 9d91473 as written).


---
_Comment/review ids in this file: 5050782922, 5050786800_

