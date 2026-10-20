# #1183 — feat(native_geometric): four-operator joint admission & broad language learning (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T03:17:48Z
- **Updated:** 2026-09-08T03:19:10Z
- **Closed:** 2026-09-08T03:18:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1183
- **Merged:** 2026-09-08T03:18:51Z
- **Merge commit:** 620ea5961319ec506c7258c3bd46381cc72266b9
- **Base / Head:** main / UOR-Foundation:issue-973-four-operator-joint-admission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
This PR implements four-operator joint admission and linguistic arbitration for the native geometric language model, addressing [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### Key Deliverables
1. **Four-Operator Exact Support Legality**:
   - `joint_admission::legal` enforces boundary checking across all 4 operators: `Copy`, `Add`, `Sub`, and `Mul`.
   - Subtraction bounds check rewritten using explicit bounds (`!((b > 0 && a < i64::MIN + b) || (b < 0 && a > i64::MAX + b))`).
   - Multiplication checked via integer-kernel-compliant `shift_add_product`.
   - Verified against checked arithmetic (`checked_add`, `checked_sub`, `checked_mul`) across extreme `i64` boundaries.
2. **Integer Kernel Compliance**:
   - Strictly preserves `#![no_std]` zero-multiply integer serving kernel invariants without forbidden arithmetic (`*`, `/`, `%`) or float opcodes in `joint_admission.rs` lines 14–98.
   - Verified by in-tree source scanner `native_kernel_source_has_no_forbidden_arithmetic_or_float_types`.
3. **Feature Encoding & Arbitration**:
   - `joint_admission::features` encodes action `kind: 6` with backward compatibility: `Copy: 0`, `Add: 1`, `Sub: 2`, `Mul: 3`.
   - Clamps margin features to `0..=127` (`kind: 7`).
   - `joint_admission::permits` arbitrates lexical vs numeric domain proposals based on geometric router scores, updating `work.admission_decisions` and `work.admission_rejections`.
4. **Comprehensive Test Suite & Allocation Census**:
   - Authored `crates/uor-r4-core/src/native_geometric/joint_admission_tests.rs` with 5 focused tests covering overflow rejection, action encoding, lexical arbitration, control bypass, and input validation.
   - Zero runtime heap allocations verified across all 9 allocation census tests in `native_geometric_allocations.rs`.
5. **Accounting & Verification**:
   - Added milestone to `docs/integration/current-state.md`.
   - Charged 4.500 model seconds (cumulative 5,309.418 / 5,550s; 240.582s remaining).
   - Preserved 128 MiB storage margin.
   - Passed `cargo fmt --check` and `scripts/check_claim_wording.py`.

References #973, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

