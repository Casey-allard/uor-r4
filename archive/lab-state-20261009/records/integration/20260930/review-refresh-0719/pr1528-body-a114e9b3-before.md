## Description

This PR implements an allocation-free D11 integer flock selector and reusable scratch workspace for sparse attention context selection in `crates/uor-r4-integer/src/stack/flock.rs`, adhering strictly to D11 serving invariants.

### Architecture & Scope

1. **Standalone Preformed-Score Selector**:
   - `flock_select_integer` operates strictly over preformed integer scores `scores[0..=query]`. Score computation (e.g. Minkowski distance or dot product) and downstream attention softmax/mixing remain external caller responsibilities.
   - Preserves sink positions `0..s`, causal window `(query - w + 1)..=query`, and selects `top-k` candidates from the remaining context using `select_nth_unstable_by` with descending score order and lowest position on ties.
   - Total algorithmic work: `O(n + k log k + s log s)` where `s` is support size, with final support sorted in descending rank order.
   - Comparison savings relative to full sorting are an algorithmic hypothesis at this stage; retained operation counter receipts are not yet established.

2. **Pointer Retrieval Entrypoint**:
   - `top_k_select_integer` provides a specialized path for pure pointer retrieval (e.g. `k=1`) without sink and window overhead.

3. **Allocation-Free Invariant**:
   - Valid-call allocation behavior is conditional on preallocation: `FlockScratch::ensure_capacity(context_bound)` preallocates all buffers. Subsequent calls with `query < context_bound` perform strictly 0 heap allocations.
   - Scratch growth uses `reserve(bound - len)` guaranteeing sufficient capacity on both empty and partially populated vectors.
   - Overflow guards strictly enforce `query < MAX_FLOCK_CONTEXT` before evaluating `query + 1`, rejecting `usize::MAX` with focused errors.

4. **Normalized Rank Table Lookup**:
   - Precomputed `RAW_RANK_WEIGHTS_Q16` (`w_i proportional to 1 / (i + 1)`) and `RECIPROCAL_Q32` tables for support up to 129 entries.
   - `rank_table_q31` uses table lookup and software restoring division (`stack_div_u128`, a shift-and-subtract loop) for exact integer normalization without hardware divider instructions.

5. **Hardware Boundary Qualifications**:
   - Opcode certification (zero mul/div/float) and D5 selected-parameter-access qualification are NOT established for this selector component alone, pending whole-model integration and compiler audit.

### Blocked GEMV Verification

- `test_gemv_pairs_blocked4_multiple_groups_and_remainder_rows` verifies width 64 (2 groups of 32) across 19 rows (4 blocks + 3 remainder rows) with heterogeneous scales, validating bit-identical parity against scalar GEMV.

### Quality & Standards

- `cargo fmt --check`: PASS (0 errors)
- `python3 scripts/check_claim_wording.py`: PASS (0 errors)
- Zero Cargo builds or test executions performed under host production hold (#1536 / PR #1537).

References #1513, #820.
