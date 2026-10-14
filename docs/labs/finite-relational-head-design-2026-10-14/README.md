# Prime-Zeta Mechanism Doc Correction and Finite Relational Head Design Card — 2026-10-14

- **Lab / Session:** `antigravity/ag1`
- **Issues:** [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028), [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032)
- **PM Assignment:** [#2028 comment 6174783178](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6174783178) and coordination note [#2029 comment 6175149631](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)
- **Basis:**
  - Prime-Zeta construction qualification ([#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473), [#2477](https://github.com/UOR-Foundation/uor-r4/pull/2477), [`docs/labs/prime-zeta-construction-2026-10-14/README.md`](../prime-zeta-construction-2026-10-14/README.md))
  - SpiralCore research assessment ([#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), [`docs/labs/spiralcore-attention-assessment-2026-10-14/README.md`](../spiralcore-attention-assessment-2026-10-14/README.md))
  - SpiralCore native reconstruction/attention review ([#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480), [`docs/labs/spiralcore-native-attention-review-2026-10-14/README.md`](../spiralcore-native-attention-review-2026-10-14/README.md) and [`reviews/integrator.md`](../spiralcore-native-attention-review-2026-10-14/reviews/integrator.md))
- **Scope:** Source-only documentation correction and finite relational head design card. Zero model/CUDA compute (`$0.00` spend), zero runtime code changes, zero head adoption or replacement, `C6` vs `G64` explicitly unselected, training `HELD`.

---

## 1. Item 1 — Correction of Stale [`docs/mechanisms/prime-zeta-complex-routing.md`](../../mechanisms/prime-zeta-complex-routing.md)

The initial draft of [`docs/mechanisms/prime-zeta-complex-routing.md`](../../mechanisms/prime-zeta-complex-routing.md) contained stale pre-implementation text (noted in [#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), line 79, and PM assignment [`6174783178`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6174783178)):
1. **Removed nonexistent `metadata.rs:344-367` citation and false runtime-replacement claim:** `crates/uor-r4-discrete/src/metadata.rs` is 211 lines of `BinderPlan` ternary forest metadata (`1..=211`), and `discrete_init` in `crates/uor-r4-discrete/src/discrete.cu:257-267` **remains the untouched xorshift PRNG default initializer** (`state = seed ^ block ^ 0x9e3779b9u`) so that all 10 files hashed by `device::source_sha256()` and `training::driver_source_sha256()` remain frozen for the active D5 completion lineage.
2. **Aligned with the delivered standalone constructor (#2473 / #2477):**
   - Documented the exact line ranges in `crates/uor-r4-discrete/src/prime_zeta.rs` (`PhaseQ29` at `48-66`, `UnitS3Q30` at `68-99`, `first_n_primes` at `107-130`, `CodebookMode` at `135-153`, `embed_s3_pair_into_e8_planes` and `exact_root_alignment` at `252-267`, `generate_codebook` at `275-508`, `compare_codebooks` and `verify_zero_float_ptx` at `532-666`) and `crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs` (`63-284`, `306-690`).
   - Recorded the honest scope disclosures from `#2473`/`#2477` and `#2478`:
     - Collision freedom (`4096/4096` unique `Snapshot.table` vectors at `V4096 B4`) and uniform 256-cell writer occupancy (quota `16`) are enforced by the post-scoring quota and uniqueness pass (`prime_zeta.rs:361-470`), which is also shared with `TangentNull` and `NormalNull` — not established as an unconstrained effect of raw prime phases alone.
     - In `CodebookMode::PrimeZeta`, seeds `0, 1, 2` produce bit-identical tables (`b4_seed0.table == b4_seed1.table == b4_seed2.table`), demonstrating deterministic repeatability of the primary map rather than three distinct random initializations (whereas `NormalNull { seed }` permutes primes per seed).
     - The finite CUDA construction qualification (`17/17` base + `6/6` Prime-Zeta controls `C18..C23` PASS on `vast-55945260`, `applied_updates: 0`, `corpus_rows_consumed: 0`) does not establish a trained language or attribution advantage, which remains `NOT_RUN` / `HELD` pending a separate downstream pre-registration sheet, `TEST FITNESS: FIT`, and owner release.

---

## 2. Item 2 — Finite Relational Head Design Card ([`docs/mechanisms/finite-relational-head.md`](../../mechanisms/finite-relational-head.md))

Authored [`docs/mechanisms/finite-relational-head.md`](../../mechanisms/finite-relational-head.md) and indexed it in [`docs/mechanisms/README.md`](../../mechanisms/README.md) per `#2480` ([`reviews/integrator.md`](../spiralcore-native-attention-review-2026-10-14/reviews/integrator.md), [`reviews/math-cross-review.md`](../spiralcore-native-attention-review-2026-10-14/reviews/math-cross-review.md), [`reviews/ml-cross-review.md`](../spiralcore-native-attention-review-2026-10-14/reviews/ml-cross-review.md), [`reviews/physics.md`](../spiralcore-native-attention-review-2026-10-14/reviews/physics.md), and [`reviews/cs.md`](../spiralcore-native-attention-review-2026-10-14/reviews/cs.md)):

1. **Five-Object Separation & Tree Boundary:** Separates `ExactContent` (`ObjectRef`), `Occurrence` (`OccurrenceRef`), `ModelSymbol`, `GeometricCarrier`, and `DerivedReadState`, keeping the canonical token-occurrence `BinderPlan` (`metadata.rs:14-178`) distinct from the 32-byte Showcase byte-storage DAG.
2. **Component 1 — Causal Candidate Pool ($\mathcal{V}_q$):**
   - Explicitly **excludes the $\le 16K$ Flock shortlist** as a valid pool for the relational head (its dominance proof holds only for `Flock`'s independent comparator `discrete.cu:165-182`).
   - Uses all causally eligible `BinderPlan` nodes (`last[node] + 1 < L`, $< 286$ nodes at $C=192$ and $< 364$ at $C=243$), preserves exact `eligible` counts in the trace (`lv[3]`), and enforces strict suffix/target/backing-store causal isolation.
3. **Component 2 — Bounded $O(|\mathcal{V}_q|)$ Sparse Edge Generator ($\mathcal{E}_q$):**
   - Defines four prefix-local, deterministic $O(|\mathcal{V}_q|)$ edge families (`QueryAnchor`, `BinderTree`, `OccurrenceChain`, `FrameBucket`) with max out-degree $D_{\max} \le 10$ ($M \le 2,860$ edges at $C=192$), zero quadratic all-pairs scan, zero answer-aware oracle hints, and canonical single-edge orientation (`math-cross-review.md:63`).
4. **Component 3 — Immutable Evidence, Anchors, and Unselected State Realizations (`C6` vs `G64`):**
   - Keeps candidate frames $f_i$ and payloads `payload[i]` strictly immutable while updating temporary finite controller states $s_i \in \mathcal{S}$ under clamped query anchor $s_q$ and input self-anchors $a_i$.
   - Specifies **Option A (`G64` / `R16` native geometric states, including the order-4 anchor-stabilizer quotient and absence of order-3 subgroups in $|G_{64}|=64$)** and **Option B (separately typed `C6` = $\mathbb{Z}/6\mathbb{Z}$ phase controller with exact doubled-cosine table `C = [2, 1, -1, -2, -1, 1]`, minimum $-3$ on equal-negative $120^\circ$ triads `(0, 2, 4)`, and no $90^\circ$ I/Q claim)** symmetrically, leaving **`C6` vs `G64` explicitly unselected**.
   - Defines a deterministic sequential strict-descent sweep with explicit cap $S_{\max}$ and `cap_hit` trace reporting.
5. **Component 4 — Deterministic Role-Conditioned Readout:**
   - Rejects unconditional synchronization filtering and `PAS >= 0.82` thresholds so contradictory/opposed evidence is preserved and retrievable under conflict-seeking regimes ($\rho_{b,l} = \text{Conflict}$).
   - Ranks level candidates by `(d_rel, d_L1, d_Hamming, -last, node_id)`, which reduces **bit-for-bit to `ReadSpec::Flock`** when couplings are zero ($J=0$ or $\rho=0$), followed by unchanged SpiralCore BFS payload transport, `snap`, ternary gates, and the shared integer vector writer.
6. **Component 5 — Outer Learning Slots, Schedule Exposure, and Cache Invalidation:**
   - Separates inner integer energy descent from outer shared-writer learning (`8*token_hinge + width*address_hinge`), defines finite categorical parameter families (`RelCoupling`, `RelOffset`, `RelReadout`), requires an active schedule that attempts relational slots rather than deferring them behind `16,384` `VectorBlock` coordinates, and specifies virtual-proposal cache/writer-admission invalidation rules.
7. **Governance & Coordination Hook:**
   - Records that Claude Fable 5.1's independent adversarial review of #2480 ([#2029 comment 6175060216](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175060216) / [`6175149631`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)) lands as an input to this card before any construction qualification sheet is written.
