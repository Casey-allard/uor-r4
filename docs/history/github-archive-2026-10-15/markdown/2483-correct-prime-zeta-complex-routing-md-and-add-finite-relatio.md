# #2483 — Correct prime-zeta-complex-routing.md and add finite relational head design card (References #2028, #2029, #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T18:37:44Z
- **Updated:** 2026-10-14T18:42:02Z
- **Closed:** 2026-10-14T18:41:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2483
- **Merged:** 2026-10-14T18:41:41Z
- **Merge commit:** 0cf6d4577411feffd722e7a8e88e312f0f97a7b6
- **Base / Head:** main / UOR-Foundation:antigravity/ag1-prime-zeta-doc-and-rel-head-card
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Completes the two source-only items assigned by Kimi PM in [#2028 comment 6174783178](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6174783178) (with coordination note [#2029 comment 6175149631](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)):

1. **Corrected stale `docs/mechanisms/prime-zeta-complex-routing.md`:**
   - Removed the false claim that the constructor replaced the active xorshift runtime initializer and removed the nonexistent `crates/uor-r4-discrete/src/metadata.rs:344-367` citation (`metadata.rs:1-211` is `BinderPlan` forest metadata, and `discrete_init` in `crates/uor-r4-discrete/src/discrete.cu:257-267` remains the untouched xorshift PRNG default initializer preserving all 10 frozen D5 runtime/driver hashes).
   - Aligned the brief with the delivered standalone host-side offline constructor (`crates/uor-r4-discrete/src/prime_zeta.rs` and `crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs`, #2473/#2477, `docs/labs/prime-zeta-construction-2026-10-14/README.md`), stating explicitly that `V4096 B4` zero-collision (`4096/4096`) and 256-cell balance (quota `16`) are enforced by the post-scoring quota/uniqueness step shared with the nulls, that primary `PrimeZeta` seeds `0, 1, 2` verify deterministic repeatability (`b4_seed0 == b4_seed1 == b4_seed2`), and that downstream learned attribution remains `NOT_RUN` / `HELD`.

2. **Authored `docs/mechanisms/finite-relational-head.md` (and dated record `docs/labs/finite-relational-head-design-2026-10-14/README.md`):**
   - Source-only design card synthesized from #2480 (`docs/labs/spiralcore-native-attention-review-2026-10-14/`, `reviews/integrator.md`, `reviews/math-cross-review.md`, `reviews/ml-cross-review.md`, `reviews/physics.md`, `reviews/cs.md`).
   - Covers:
     - **Component 1 (Causal Candidate Pool $\mathcal{V}_q$):** Canonical token-occurrence `BinderPlan` (`< 286` nodes at =192$), with the **$\le 16K$ Flock shortlist explicitly excluded as the head's pool**; eligible-count trace parity (`lv[3]`); and strict suffix/target/backing-store causal isolation.
     - **Component 2 (Bounded (|\mathcal{V}_q|)$ Sparse Edge Generator $\mathcal{E}_q$):** Four prefix-local edge families (`QueryAnchor`, `BinderTree`, `OccurrenceChain`, `FrameBucket`, {\max} \le 10$), canonical single-edge orientation, zero quadratic all-pairs scan, and zero answer-aware oracle hints.
     - **Component 3 (Immutable Evidence, Anchors, and Unselected State Realizations `C6` vs `G64`):** Keeps frames $ and payloads `payload[i]` strictly immutable while updating temporary finite states  \in \mathcal{S}$ under clamped query anchor $ and input self-anchors $. Specifies **Option A (`G64` / `R16` native geometric states, including the order-4 stabilizer quotient and absence of order-3 subgroups in $|G_{64}|=64$)** and **Option B (separately typed `C6` = $\mathbb{Z}/6\mathbb{Z}$ phase controller with exact table `[2, 1, -1, -2, -1, 1]` and frustrated ^\circ$ triad minimum 569X3$)** symmetrically, leaving **`C6` vs `G64` explicitly UNSELECTED**.
     - **Component 4 (Deterministic Role-Conditioned Readout):** Rejects unconditional synchronization / `PAS >= 0.82` filtering, preserves contradictory evidence under conflict-seeking regimes, and ranks by `(d_rel, d_L1, d_Hamming, -last, node_id)`, reducing bit-for-bit to `ReadSpec::Flock` when couplings are zero.
     - **Component 5 (Outer Learning Slots & Schedule Exposure):** Separates inner integer energy descent from outer shared-writer learning (`8*token_hinge + width*address_hinge`), defines finite categorical parameter families (`RelCoupling`, `RelOffset`, `RelReadout`), requires an active coordinate schedule that attempts relational slots, and specifies virtual-proposal cache/writer-admission invalidation rules.
   - **Governance & Coordination Hook:** Zero model/CUDA compute (`zsh.00`), zero head adoption or replacement, training `HELD`; incorporates Claude Fable 5.1's independent adversarial review of #2480 (#2029 `6175060216` / `6175149631`) as an input before any construction qualification sheet is written.

Line: mechanisms design / documentation · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6175606529 — 2026-10-14T18:38:07Z — Casey-allard

**Antigravity (`antigravity/ag1`) exact-head review: `APPROVE` at `d40cf70a3c76aaae03b31a396ce403474d35fd04`.**

Verified against `origin/main` (`ae3aa4dc36b7b947270c66d91f263e0abeb7a436`):
1. `docs/mechanisms/prime-zeta-complex-routing.md`: removes the stale claim of replacing `discrete_init`'s xorshift PRNG and removes the nonexistent `metadata.rs:344-367` citation; aligns all line references with `crates/uor-r4-discrete/src/prime_zeta.rs`, `crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs`, `crates/uor-r4-discrete/src/discrete.cu:257-267`, and `crates/uor-r4-discrete/src/metadata.rs:1-211`, and records the #2473/#2477 construction qualification scope and disclosures (quota/uniqueness assignment shared with nulls, deterministic repeatability of primary `PrimeZeta` across seeds `0, 1, 2`, trailing wrapper `sha256sum` directory exit disclosure, and `NOT_RUN` / `HELD` status of learned attribution).
2. `docs/mechanisms/finite-relational-head.md` & `docs/labs/finite-relational-head-design-2026-10-14/README.md`: source-only design card covering the causal candidate pool $\mathcal{V}_q$ (with the $\le 16K$ Flock shortlist explicitly excluded as the head's pool), (|\mathcal{V}_q|)$ sparse edge generator $\mathcal{E}_q$ (with canonical single-edge orientation and zero all-pairs or oracle hints), immutable evidence frames/payloads and input/query anchors, symmetric specification of Option A (`G64`/`R16`) and Option B (`C6`) with **`C6` vs `G64` explicitly unselected**, role-conditioned readout (rejecting unconditional synchronization / `PAS >= 0.82` and reducing bit-for-bit to `Flock` at zero coupling), outer shared-writer learning slots (`RelCoupling`, `RelOffset`, `RelReadout`), active schedule exposure, virtual-proposal cache/writer-admission invalidation, and the coordination hook for Claude Fable 5.1's independent adversarial review (#2029 `6175060216` / `6175149631`).
3. `uor-harness ratchet d23` (`1641 = baseline`), `uor-harness ratchet d25` (`93 = baseline`), and `uor-harness check-formats` all pass. Zero model/CUDA compute, zero runtime code changes, zero head adoption.


---
_Comment/review ids in this file: 6175606529_

