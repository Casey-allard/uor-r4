# ORDER: r-vsa-hamming-binding, antigravity-69

Lab: antigravity. Item: [r-vsa-hamming-binding](../../../../roadmap/items/r-vsa-hamming-binding.md). Base: `79b2df270`.

## Goal

The item's done-when reads:

> A registered GPU run of Hamming binding on the current writer, with a result compared against the count tables.

Under the item's `docs/**` scope, this unit runs the cancel probe first, authors the one-page pre-registration sheet (`docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md`) and lab specification (`docs/labs/r-vsa-hamming-binding/README.md`) defining `DiscreteVsaHammingWriterV1` (multiplier-free 64-bit XOR role-filler binding, `SpiralCore` signed-axis cyclic shift, bitwise majority bundling, and `__popcll` Hamming distance in `discrete.cu`'s `writer`, comparing `VSA-Off`, `VSA-Fixed-64`, and `VSA-Root-Residual-Frozen-64` against full-panel and admission-matched count tables), obtains a fresh-context non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and `D68` decision review (`status: ready -> status: held` until `#2836` `m1-bvg-k-stage-t-training` completes), and posts the review on `#2888`.

## Cancel probe (run first)

- **Class:** `instrument` (missing pre-registration sheet + `discrete.cu`'s `writer` has no XOR/Hamming context-binding term yet, while `crates/uor-r4-discrete/**` is outside this item's `docs/**` scope and gated on `#2836` `m1-bvg-k-stage-t-training`).
- **Command & Evidence:** Inspected `crates/uor-r4-discrete/src/discrete.cu` (`root_cost` line 37, `address_distance` line 328, `token_distance` line 331, `writer` line 361), `crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs`, `archive/labs/vsa-native-test-2026-10-09/README.md`, `docs/mechanisms/vsa.md`, and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/vsa.json`.
- **Findings:**
  1. No pre-registration sheet `docs/harness/prep/r-vsa-hamming-binding*` or lab record `docs/labs/r-vsa-hamming-binding/**` exists yet (`instrument` missing).
  2. In the 9 Oct 2026 VSA native test (`archive/labs/vsa-native-test-2026-10-09/README.md`), fixed-code VSA (Mode 0, 4,096 distinct codes) beat VSA-off on all 4 seed-slice pairs (`-0.0144` to `-0.0226` BPB), whereas root-only codes (Mode 1, 120 codes) collapsed token identity (`+0.0003` to `-0.0054`) and Mode 2 (root XOR readout residual rebuilt every 1,000 steps) lost (`+0.013` to `+0.028`) due to moving codes, and the run used a CPU learner barred by `D24` (`CHARTER §3.5`).
  3. In `discrete.cu`, `root_cost` computes coordinate Hamming (`*hamming += (x[d]!=y)`) to break ties in `route_vector` and `flock_vector`, but `writer(...)` (lines 361–399) scores candidates inside admitted cells solely by static L1 distance $d_{\text{L1}}(q, v(t))$ with zero context-bound XOR/Hamming term, leaving `3,421 / 4,292` admitted rows (`79.71%`) unranked at `top-1` in `#2831` `Q1` (`871 / 4,292`).
  4. Because `r-vsa-hamming-binding.md` scopes only `docs/**`, this unit delivers the pre-registration sheet, `DiscreteVsaHammingWriterV1` specification, `TEST FITNESS: FIT`, and `D68` decision review (`status: ready -> status: held` until `#2836` completes and `crates/uor-r4-discrete/**` is free).

## Files in scope

- `docs/harness/prep/r-vsa-hamming-binding*`
- `docs/labs/r-vsa-hamming-binding/**`
- `docs/loop/tasks/r-vsa-hamming-binding/**`
- Always-allowed paths: `docs/roadmap/items/r-vsa-hamming-binding.md`, `docs/roadmap/README.md`

## Done when

- Cancel probe committed first in `ORDER.md`.
- `docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md` and `docs/labs/r-vsa-hamming-binding/README.md` authored and reviewed (`TEST FITNESS: FIT` + `D68` decision review posted on `#2888`).
- `r-vsa-hamming-binding` set to `status: held`, `docs/roadmap/README.md` rendered, and `REPORT.md` passes all gates.

## Stop and ask if

- Never stop to wait; decide via `D68` decision review and log `[antigravity] owner log:` on `#2888`.
