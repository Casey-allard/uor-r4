# #106 — Lie-Jordan splitting and allocation-free universal product kernel

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-23T15:48:39Z
- **Updated:** 2026-07-23T18:46:51Z
- **Closed:** 2026-07-23T18:46:51Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/106
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #103
Depends on: #105
Spec: `docs/r4_furey_quantum_geometric_plan.md` §Phase 3

Implement Lie-Jordan splitting over centralizers $\Delta_{\text{SM}}$ and $\Delta_{\text{LE}}$ in `crates/uor-r4-core/src/transformerless/lie_jordan.rs`.
Split endomorphisms into anti-Hermitian Lie symmetries $\mathcal{L}_\Delta$ (context operations 0$) and Hermitian Jordan observables $\mathcal{H}_\Delta$ (state probabilities $\{h,s\}$).
Execute hot paths using the universal product (a,b) = ab + ba^\dagger$ over bitwise integer tables.

---

## Definition of Done
- Lie-Jordan state split implemented with zero-allocation, zero-float hot-path kernel.
- Machine-checked static source scan asserts 0 floats, 0 multiplies, 0 divides in deployed path.

## Acceptance criteria
- [ ] `universal_product` integer kernel implemented.
- [ ] `allocation_census.rs` asserts 0 heap allocations in steady state.
- [ ] `cargo test` passes 100% cleanly.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

