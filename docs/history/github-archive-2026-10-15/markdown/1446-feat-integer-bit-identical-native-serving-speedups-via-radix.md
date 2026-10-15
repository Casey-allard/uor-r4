# #1446 — feat(integer): bit-identical native serving speedups via radix-16 mixture and grouped LUT projection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T01:51:56Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-09-28T05:33:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1446
- **Merged:** not merged
- **Merge commit:** 23cacb3e91412afa37e28eb8c141ea76a601cb88
- **Base / Head:** main / UOR-Foundation:lab/anti-gravity/phase-b-speedups
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary of Changes

Track T3 Phase B: bit-identical native integer serving speedups on Apple Silicon M1 Firestorm.

1. **Hotspot Analysis under `/usr/bin/sample` on 32,768 steps**:
   - Initial profile showed probability mixture loop (`checked_mul` and `div_round`) taking 38.7% of step time, and `project_vocab` taking 19.9% of step time.
   - Hotspot 1 Rewrite: Replaced generic 128-bit `checked_mul` and `div_round` in `IntegerModel::step` mixture loop with precomputed radix-16 table lookup (`build_fraction_table`, `mul_fraction_radix16`, `div_round_100m_raw`, and `mul_shift_add_i64`).
   - Hotspot 2 Rewrite: Grouped LUT / T-MAC style acceleration in `project_vocab_with_products_into` (`build_pair_tables_256`, `low_bit_dot_8_pair_tables`, `build_pair_tables_576`, and `low_bit_dot_4_contiguous_wide_pair_tables`), pre-fetching the contiguous packed range once and halving table lookups and additions.

2. **Gates & Invariant Verification**:
   - **Bit-Identity Gate**: 32,768-step trace SHA-256 is `a88db0edad0b20c227cf06201bc381009d7382237bbee6bceaae25dd62055179` — 100% bit-for-bit identical to baseline.
   - **Dialogue Witnesses**: Both `witness-final` (`dialogue-token-witness`) and `witness-continuity-final` (`dialogue-continuity-witness`) pass cleanly with `all_comparisons_equal: true`.
   - **Disassembly Audit (AArch64 D0-b)**: `scripts/audit_zero_matmul_serving.py` achieves **FULL PASS** (0 Class I hardware multipliers, 0 Class II hardware dividers, 0 Class III floats) across all 49 matched symbol ranges.
   - **Single-Thread Latency & Throughput** (Apple Silicon M1 Firestorm, 32,768 steps):
     - Baseline: 68.74 s (476.7 tok/s, 2.098 ms/tok)
     - Rewrite 1: 41.85 s (782.9 tok/s, 1.277 ms/tok, +64.2% speedup)
     - Final (Rewrite 1 + 2): 36.14 s (906.7 tok/s, 1.103 ms/tok, **+90.2% speedup**, 47.4% latency reduction).
   - **Tests**: All 80+ tests in `uor-r4-integer` pass. `cargo fmt --check` passes. `check_claim_wording.py` passes.
   - **Clean Isolation**: Executed 100% in isolated worktree `.worktrees/phase-b-speedups`. Primary checkout remains completely untouched.

References #973, #820

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5864088870 — 2026-09-28T05:33:10Z — Casey-allard

**Closed as integrated via #1450** (squash-merged to main as `000b41c6`), under the owner's instruction to clear the PR backlog. This PR's head is an ancestor of #1450's merged head, so its commits are on main in their reviewed and revised form. The radix-16 and grouped-LUT speedups are on main, and they pass #1450's upgraded call-graph audit. References #820.


---
_Comment/review ids in this file: 5864088870_

