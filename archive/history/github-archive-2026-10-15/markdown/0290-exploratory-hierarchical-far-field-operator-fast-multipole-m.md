# #290 — Exploratory: hierarchical far-field operator (Fast Multipole Method) over the multiresolution cover

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T23:18:51Z
- **Updated:** 2026-08-03T01:17:31Z
- **Closed:** 2026-08-03T01:11:26Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/290
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Type:** exploratory / research spike
**Status:** unproven — opened to be falsified, not to be implemented on sight
**Touches:** `uor-r4-core::transformerless` (cover, score), `uor-r4-graph-format`, `uor-r4-proof-model`
**Related:** #230, #231, #232, #234 (see §6)

---

## 1. Summary

R⁴ already builds a multiresolution, overlapping cover of the semantic space (spherical k-means, calibrated memberships) and stores refinement, neighbor, and forward edges in R4G1. That is structurally most of a Fast Multipole Method (FMM) tree. What is missing is the piece FMM exists to provide: a **far-field operator** that lets a region influence a well-separated region without an explicit edge between them.

This issue proposes evaluating — not adopting — an FMM-style far-field aggregation between `transformerless cover` and `transformerless score`, and sets out in advance what result would kill the idea.

## 2. What FMM actually is

FMM (Greengard & Rokhlin, 1987) computes all-pairs interactions among *n* bodies in O(*n*) instead of O(*n*²), by never computing most pairs:

| Stage | Operation | R⁴ analogue |
|---|---|---|
| P2M | aggregate sources into a leaf's multipole summary | leaf region summary from cover cells |
| M2M | merge child summaries into the parent | **refinement edges (already exist)** |
| M2L | translate a summary into a *local expansion* at a well-separated region | **missing — this is the proposal** |
| L2L | push the parent's local expansion down to children | **refinement edges, reversed (already exist)** |
| L2P | evaluate the local expansion at each target | emission / ScoreQ readout |
| near-field | compute close pairs directly | **neighbor edges (already exist)** |

The claim worth testing is narrow: **three of the six FMM stages are already built in `cover`/`score`. The proposal is one new operator (M2L) plus an admissibility rule for which region pairs count as well separated.**

## 3. Why we would want it

**a. Effective context beyond `WINDOW = 8`.** The deployed runtime reads eight token IDs. Long-range influence is currently unavailable at any price. A far-field operator lets distant context reach the current prediction through *aggregated* region summaries rather than a longer window — cost grows logarithmically in distance, not linearly. If this works at all, it is the largest single capability gain available to the runtime.

**b. It is the right shape for CPU-first.** Dense all-pairs attention is memory-bandwidth-bound, which is the actual reason it demands a GPU. FMM is the opposite: small working set, high arithmetic locality, predictable tree-traversal access patterns. This is the algorithm class that makes "no GPU, no thermal envelope problem" a design consequence rather than a constraint we absorb.

**c. The adjoint is the same shape.** The transpose of a hierarchical operator is a hierarchical operator at the same asymptotic cost. If R⁴ ever needs gradient flow rather than compiled tables, whatever we build forward we get backward at the same price. Not needed now; worth knowing before committing to a structure.

**d. It matches the physics the geometry is meant to encode.** Long-range order without long-range wiring is real and studied — hierarchically coupled systems (Dyson, 1969) exhibit phase transitions that flat nearest-neighbour systems of the same dimension cannot. If the manifold is meant to carry that, the hierarchy has to be load-bearing, not just an index.

## 4. The central risk, stated honestly

**Classical FMM's M2L step is a dense multiply per box pair. Our runtime is zero-multiply, table-native, allocation-free, and fixed-point (ScoreQ, `u32` stores).** Naively bolting on FMM violates the core invariant of the project.

Two candidate routes, both to be validated rather than assumed:

1. **Push M2L to compile time.** The README is explicit that compilation may allocate and multiply; only the deployed prediction hot path is allocation-free. If translation operators for each admissible region pair are precomputed and baked into R4G1 tables, the runtime does lookup + fixed-point accumulation and the invariant survives. This is the intended route.
2. **Use a kernel-independent formulation.** Classical FMM assumes an analytically expandable kernel (1/r and friends). Our "kernel" is a learned semantic score with no such guarantee. Black-box / interpolation-based FMM (Fong & Darve) and kernel-independent FMM (Ying, Biros & Zorin) drop that requirement and build translations numerically from samples.

The property we actually depend on is **not** "the kernel is 1/r." It is: **the region-to-region interaction blocks for well-separated regions are numerically low rank.** That is an empirical question about our data, answerable cheaply before any implementation. See §5.1.

## 5. Falsification plan

Pre-registered. Each test can kill the proposal on its own.

### 5.1 Primary killer — off-diagonal block rank (offline, no code changes, no eval dependency)

Using an existing `cover.r4g1` and its `corpus.records`:

1. Pick an admissibility criterion (e.g. region pairs whose separation exceeds *η* × max diameter).
2. For a sample of admissible pairs, materialise the full interaction block.
3. Compute the singular value spectrum; record numerical rank *r*(ε) at ε ∈ {1e-2, 1e-3, 1e-4}.

**Kill condition:** if *r*(ε) grows proportionally with block dimension rather than staying roughly constant (or growing polylogarithmically), the far-field is not compressible and every downstream FMM / ℋ-matrix / Barnes–Hut variant fails. Close the issue.

**Run this first.** It is cheap, decisive, and the only test in this plan that does not depend on the evaluation infrastructure noted in §6. Nothing else here matters if it fails.

### 5.2 Accuracy

The intended test is held-out top-1 agreement and Witten–Bell bits/token against the current `r4g1` engine, using the `score_report.json` harness and the split recorded in the objective block.

One caveat worth recording: per #234, the Gate C evaluation distribution is currently answered largely by exact-context lookup. That matters more here than for most proposals: the entire motivation in §3a is *long-range context*, and an eval answerable from exact context cannot register a long-range improvement. A passing score would be uninformative.

So this spike should either wait on #234 or define its own held-out set containing genuinely novel contexts, with the construction documented before any numbers are quoted.

**Kill condition (once measurable):** top-1 agreement drops more than a pre-registered ε (propose 0.5 pp absolute), or bits/token worsens against the teacher floor.

### 5.3 Invariant preservation — the operation census is already the instrument

`POST /api/tless/predict` returns an operation census. The FMM path must show **zero multiplies in the hot path** and pass the existing `no_std`, `deterministic-rebuild`, `fuzz`, and `wasm` CI gates unchanged.

**Prior evidence to take seriously:** #230 reports that mul-free context assignment fragments the store 3.71× against its own f32 control. That is a measured precedent for exactly the risk here — replacing a real-valued geometric operation with a mul-free/fixed-point surrogate has already cost us structure once. Fixed-point M2L translation tables are the same class of substitution and should be assumed guilty until measured.

**Additional pre-registered budget:** FMM translation tables must not inflate artifact size or store fragmentation beyond a stated multiple (propose ≤1.25× on both) relative to the current R4G1 artifact for the same cover.

**Kill condition:** any multiply in the deployed path; determinism breaks (floating-point reduction-order sensitivity is the likely culprit — everything stays fixed-point with a fixed reduction order); or the size/fragmentation budget is exceeded.

### 5.4 Speed — the constant factor is the usual assassin

FMM's O(*n*) hides a large constant and routinely loses to direct evaluation below a crossover point *N\**. Measure tokens/sec across realistic region counts and locate *N\**.

Note: timing comparisons should avoid the `certify` path until #232 (≈2.5 of 3.2 hours spent on an R4G1 evaluation returning an all-zero row) and #231 (teacher floor returns `inf` bits/token on the certified legacy path) are settled.

**Kill condition:** *N\** exceeds the region count of workloads we actually run. The idea can be asymptotically correct and still useless here; if so, say so and close it.

### 5.5 Determinism of the compiled artifact

Precomputed translation tables must be reproducible bit-for-bit, and κ re-pin must follow the procedure in `AGENTS.md`.

## 6. Relationship to open issues

No existing issue proposes hierarchical or multipole methods, so this is new ground. It does lean on measurement paths that other open issues touch, noted here so the interaction is on record:

| Issue | Relationship |
|---|---|
| **#234** — Gate C eval answered by exact-context lookup, cannot measure generalization | Affects §5.2. The long-range-context motivation is hard to measure on an eval answerable from exact context. |
| **#231** — teacher floor returns `inf` bits/token on certified legacy path | Affects §5.2. bits/token is one of the two accuracy criteria. |
| **#232** — `certify` spends ~2.5/3.2 h on an R4G1 eval returning an all-zero row | Affects §5.4. Timing baselines come from this path. |
| **#230** — mul-free context assignment fragments the store 3.71× vs f32 control | Useful precedent for §5.3 — motivates the size/fragmentation budget. |
| **#233** — legacy pipeline docs stale | Minor; touches docs referenced in sequencing. |

**Consequence for sequencing:** §5.1 is deliberately independent of all of the above and can run now.

## 7. Alternatives to benchmark against

The proposal is only justified if it beats **all** of these. Increasing implementation cost:

| # | Baseline | Why it's here |
|---|---|---|
| 0 | **Do nothing — raise the neighbor-edge budget in `cover`** | The null hypothesis. If more near-field edges buy the same quality, FMM is unjustified complexity. Must be run. |
| 1 | **Current `r4g1` scorer** | The incumbent. Everything is measured against this. |
| 2 | **Dense / exhaustive region-to-region** | Accuracy ceiling and speed floor. Defines what we're approximating. |
| 3 | **Barnes–Hut (monopole-only, O(*n* log *n*))** | Drastically simpler. If it captures most of the gain, FMM is over-engineering. |
| 4 | **ℋ-matrix / HODLR / HSS** | Same low-rank premise, simpler machinery, mature literature (Hackbusch). Likely a better first implementation than full FMM. |
| 5 | **Nyström / random-feature low-rank approximation** | The ML-native approach to the same problem. Cheap, well understood, strong baseline. |

**Decision rule:** adopt only if the candidate beats baseline 0 on quality *and* baselines 3–5 on the quality/latency Pareto front *and* passes §5.3 unchanged. Otherwise document the negative result and close.

## 8. Prior art worth reading before writing code

- Greengard & Rokhlin (1987) — the original FMM.
- Barnes & Hut (1986) — the cheaper tree approximation.
- Hackbusch — ℋ-matrices; the linear-algebra framing of the same idea.
- Fong & Darve — black-box FMM (interpolation-based, kernel-agnostic).
- Ying, Biros & Zorin — kernel-independent FMM.
- Zhu & Soricut, *H-Transformer-1D* (ACL 2021) — hierarchical-matrix attention for sequences. **Closest existing ML work; read before claiming novelty.**
- Li et al., *Multipole Graph Neural Operator* (NeurIPS 2020) — multipole decomposition in a learned operator setting.
- Dyson (1969) — hierarchical model; why hierarchy buys long-range order.

## 9. Proposed sequencing

1. Run §5.1 on an existing cover. **Gate: stop here if it fails.**
2. In parallel, track #234 — it governs whether §5.2 can produce a meaningful number for this proposal specifically.
3. If low rank holds, implement baseline 4 (ℋ-matrix / HODLR) before FMM — same premise, far less code, and it validates the compile-time-table approach against the #230 fragmentation precedent.
4. Only if ℋ-matrix wins on the Pareto front, evaluate whether full FMM's extra machinery earns its keep.
5. Add proof obligations to `uor-r4-proof-model` for whatever survives.

## 10. What would make this issue succeed

A recorded answer either way. A documented negative result from §5.1 is a good outcome and should be treated as one — it closes off a whole family of approaches cheaply and tells us something real about the geometry of the cover.
**Type:** exploratory / research spike
**Status:** unproven — opened to be falsified, not to be implemented on sight
**Touches:** uor-r4-core::transformerless (cover, score), uor-r4-graph-format, uor-r4-proof-model
**Related:** #230, #231, #232, #234 (see section 6)

## 1. Summary

R⁴ already builds a multiresolution, overlapping cover of the semantic space (spherical k-means, calibrated memberships) and stores refinement, neighbor, and forward edges in R4G1. That is structurally most of a Fast Multipole Method (FMM) tree. What is missing is the piece FMM exists to provide: a **far-field operator** that lets a region influence a well-separated region without an explicit edge between them.

This issue proposes evaluating — not adopting — an FMM-style far-field aggregation between transformerless cover and transformerless score, and sets out in advance what result would kill the idea.

## 2. What FMM actually is

FMM (Greengard and Rokhlin, 1987) computes all-pairs interactions among n bodies in O(n) instead of O(n^2), by never computing most pairs:

| Stage | Operation | R⁴ analogue |
|---|---|---|
| P2M | aggregate sources into a leaf multipole summary | leaf region summary from cover cells |
| M2M | merge child summaries into the parent | **refinement edges (already exist)** |
| M2L | translate a summary into a local expansion at a well-separated region | **missing — this is the proposal** |
| L2L | push the parent local expansion down to children | **refinement edges, reversed (already exist)** |
| L2P | evaluate the local expansion at each target | emission / ScoreQ readout |
| near-field | compute close pairs directly | **neighbor edges (already exist)** |

The claim worth testing is narrow: **three of the six FMM stages are already built in cover/score. The proposal is one new operator (M2L) plus an admissibility rule for which region pairs count as well separated.**

## 3. Why we would want it

**a. Effective context beyond WINDOW = 8.** The deployed runtime reads eight token IDs. Long-range influence is currently unavailable at any price. A far-field operator lets distant context reach the current prediction through aggregated region summaries rather than a longer window — cost grows logarithmically in distance, not linearly. If this works at all, it is the largest single capability gain available to the runtime.

**b. It is the right shape for CPU-first.** Dense all-pairs attention is memory-bandwidth-bound, which is the actual reason it demands a GPU. FMM is the opposite: small working set, high arithmetic locality, predictable tree-traversal access patterns. This is the algorithm class that makes "no GPU, no thermal envelope problem" a design consequence rather than a constraint we absorb.

**c. The adjoint is the same shape.** The transpose of a hierarchical operator is a hierarchical operator at the same asymptotic cost. If R⁴ ever needs gradient flow rather than compiled tables, whatever we build forward we get backward at the same price. Not needed now; worth knowing before committing to a structure.

**d. It matches the physics the geometry is meant to encode.** Long-range order without long-range wiring is real and studied — hierarchically coupled systems (Dyson, 1969) exhibit phase transitions that flat nearest-neighbour systems of the same dimension cannot. If the manifold is meant to carry that, the hierarchy has to be load-bearing, not just an index.

## 4. The central risk, stated honestly

**Classical FMM M2L is a dense multiply per box pair. Our runtime is zero-multiply, table-native, allocation-free, and fixed-point (ScoreQ, u32 stores).** Naively bolting on FMM violates the core invariant of the project.

Two candidate routes, both to be validated rather than assumed:

1. **Push M2L to compile time.** The README is explicit that compilation may allocate and multiply; only the deployed prediction hot path is allocation-free. If translation operators for each admissible region pair are precomputed and baked into R4G1 tables, the runtime does lookup plus fixed-point accumulation and the invariant survives. This is the intended route.
2. 2. **Use a kernel-independent formulation.** Classical FMM assumes an analytically expandable kernel (1/r and friends). Our kernel is a learned semantic score with no such guarantee. Black-box / interpolation-based FMM (Fong and Darve) and kernel-independent FMM (Ying, Biros and Zorin) drop that requirement and build translations numerically from samples.
The property we actually depend on is **not** that the kernel is 1/r. It is: **the region-to-region interaction blocks for well-separated regions are numerically low rank.** That is an empirical question about our data, answerable cheaply before any implementation. See 5.1.

## 5. Falsification plan

Pre-registered. Each test can kill the proposal on its own.

### 5.1 Primary killer — off-diagonal block rank (offline, no code changes, no eval dependency)

Using an existing cover.r4g1 and its corpus.records:

1. Pick an admissibility criterion (e.g. region pairs whose separation exceeds eta times max diameter).
2. 2. For a sample of admissible pairs, materialise the full interaction block.
3. 3. Compute the singular value spectrum; record numerical rank r(eps) at eps in {1e-2, 1e-3, 1e-4}.
**Kill condition:** if r(eps) grows proportionally with block dimension rather than staying roughly constant (or growing polylogarithmically), the far-field is not compressible and every downstream FMM / H-matrix / Barnes-Hut variant fails. Close the issue.

**Run this first.** It is cheap, decisive, and the only test in this plan that does not depend on the evaluation infrastructure noted in section 6. Nothing else here matters if it fails.

### 5.2 Accuracy

The intended test is held-out top-1 agreement and Witten-Bell bits/token against the current r4g1 engine, using the score_report.json harness and the split recorded in the objective block.

One caveat worth recording: per #234, the Gate C evaluation distribution is currently answered largely by exact-context lookup. That matters more here than for most proposals: the entire motivation in 3a is long-range context, and an eval answerable from exact context cannot register a long-range improvement. A passing score would be uninformative.

So this spike should either wait on #234 or define its own held-out set containing genuinely novel contexts, with the construction documented before any numbers are quoted.

**Kill condition (once measurable):** top-1 agreement drops more than a pre-registered epsilon (propose 0.5 pp absolute), or bits/token worsens against the teacher floor.

### 5.3 Invariant preservation — the operation census is already the instrument

POST /api/tless/predict returns an operation census. The FMM path must show **zero multiplies in the hot path** and pass the existing no_std, deterministic-rebuild, fuzz, and wasm CI gates unchanged.

**Prior evidence to take seriously:** #230 reports that mul-free context assignment fragments the store 3.71x against its own f32 control. That is a measured precedent for exactly the risk here — replacing a real-valued geometric operation with a mul-free/fixed-point surrogate has already cost us structure once. Fixed-point M2L translation tables are the same class of substitution and should be assumed guilty until measured.

**Additional pre-registered budget:** FMM translation tables must not inflate artifact size or store fragmentation beyond a stated multiple (propose 1.25x max on both) relative to the current R4G1 artifact for the same cover.

**Kill condition:** any multiply in the deployed path; determinism breaks (floating-point reduction-order sensitivity is the likely culprit — everything stays fixed-point with a fixed reduction order); or the size/fragmentation budget is exceeded.

### 5.4 Speed — the constant factor is the usual assassin

FMM O(n) hides a large constant and routinely loses to direct evaluation below a crossover point N*. Measure tokens/sec across realistic region counts and locate N*.

Note: timing comparisons should avoid the certify path until #232 (about 2.5 of 3.2 hours spent on an R4G1 evaluation returning an all-zero row) and #231 (teacher floor returns inf bits/token on the certified legacy path) are settled.

**Kill condition:** N* exceeds the region count of workloads we actually run. The idea can be asymptotically correct and still useless here; if so, say so and close it.

### 5.5 Determinism of the compiled artifact

Precomputed translation tables must be reproducible bit-for-bit, and kappa re-pin must follow the procedure in AGENTS.md.

## 6. Relationship to open issues

No existing issue proposes hierarchical or multipole methods, so this is new ground. It does lean on measurement paths that other open issues touch, noted here so the interaction is on record:

| Issue | Relationship |
|---|---|
| **#234** — Gate C eval answered by exact-context lookup, cannot measure generalization | Affects 5.2. The long-range-context motivation is hard to measure on an eval answerable from exact context. |
| **#231** — teacher floor returns inf bits/token on certified legacy path | Affects 5.2. bits/token is one of the two accuracy criteria. |
| **#232** — certify spends about 2.5/3.2 h on an R4G1 eval returning an all-zero row | Affects 5.4. Timing baselines come from this path. |
| **#230** — mul-free context assignment fragments the store 3.71x vs f32 control | Useful precedent for 5.3 — motivates the size/fragmentation budget. |
| **#233** — legacy pipeline docs stale | Minor; touches docs referenced in sequencing. |

**Consequence for sequencing:** 5.1 is deliberately independent of all of the above and can run now.

## 7. Alternatives to benchmark against

The proposal is only justified if it beats **all** of these. Increasing implementation cost:

| # | Baseline | Why it is here |
|---|---|---|
| 0 | **Do nothing — raise the neighbor-edge budget in cover** | The null hypothesis. If more near-field edges buy the same quality, FMM is unjustified complexity. Must be run. |
| 1 | **Current r4g1 scorer** | The incumbent. Everything is measured against this. |
| 2 | **Dense / exhaustive region-to-region** | Accuracy ceiling and speed floor. Defines what we are approximating. |
| 3 | **Barnes-Hut (monopole-only, O(n log n))** | Drastically simpler. If it captures most of the gain, FMM is over-engineering. |
| 4 | **H-matrix / HODLR / HSS** | Same low-rank premise, simpler machinery, mature literature (Hackbusch). Likely a better first implementation than full FMM. |
| 5 | **Nystrom / random-feature low-rank approximation** | The ML-native approach to the same problem. Cheap, well understood, strong baseline. |

**Decision rule:** adopt only if the candidate beats baseline 0 on quality AND baselines 3-5 on the quality/latency Pareto front AND passes 5.3 unchanged. Otherwise document the negative result and close.

## 8. Prior art worth reading before writing code

- Greengard and Rokhlin (1987) — the original FMM.
- - Barnes and Hut (1986) — the cheaper tree approximation.
- - - Hackbusch — H-matrices; the linear-algebra framing of the same idea.
- - - - Fong and Darve — black-box FMM (interpolation-based, kernel-agnostic).
- - - - - Ying, Biros and Zorin — kernel-independent FMM.
- - - - - - Zhu and Soricut, H-Transformer-1D (ACL 2021) — hierarchical-matrix attention for sequences. **Closest existing ML work; read before claiming novelty.**
- - - - - - - Li et al., Multipole Graph Neural Operator (NeurIPS 2020) — multipole decomposition in a learned operator setting.
- - - - - - - - Dyson (1969) — hierarchical model; why hierarchy buys long-range order.
## 9. Proposed sequencing

1. Run 5.1 on an existing cover. **Gate: stop here if it fails.**
2. 2. In parallel, track #234 — it governs whether 5.2 can produce a meaningful number for this proposal specifically.
3. 3. If low rank holds, implement baseline 4 (H-matrix / HODLR) before FMM — same premise, far less code, and it validates the compile-time-table approach against the #230 fragmentation precedent.
4. 4. Only if H-matrix wins on the Pareto front, evaluate whether full FMM extra machinery earns its keep.
5. 5. Add proof obligations to uor-r4-proof-model for whatever survives.
## 10. What would make this issue succeed

A recorded answer either way. A documented negative result from 5.1 is a good outcome and should be treated as one — it closes off a whole family of approaches cheaply and tells us something real about the geometry of the cover.

## Comments (5)

### comment 5153705445 — 2026-08-01T22:18:07Z — auser

## §5.1 result — off-diagonal block rank: **proposal survives (proceed to §5.2), with qualifications**

Pre-registered killer test executed 2026-08-01. Full report: `scratch/fmm_rank/fmm_rank_report.md` (+ `analyze.py`, `analyze2.py`, `results/` in the same directory; no repo code modified).

### Material and reproduction

- Bundle: `.uor-models/compiled/SmolLM2-135M-Instruct-7e27bd9f9532/graph-cover/cover.r4g1`, artifact CID `blake3:fa4e5575813984c64ba2f9b8d7be5720a7c41ea04376e319998b209c16d338ed` (362 regions, depths [16, 30, 58, 98, 160], 159,658 train observations). The `smollm2-135m-instruct` bundle was disqualified: its on-disk corpus no longer hashes to its pinned `corpus_kappa`.
- The cover was **re-induced bit-for-bit** from the pinned inputs (worktree at compile-era commit `9d95961`, config pinned in `cover_report.json`); `artifact_kappa`, `corpus_kappa`, and `cover_kappa blake3:67d957c3…` all matched — the cover kappa hashes every region's f32 prototype bytes, so the analyzed geometry is exactly the shipped one.

### Operational definitions

- **Interaction block**: for pair (A, B), the submatrix of the semantic Gram matrix over the cover's top-1 partition (the refinement tree), `K(x,y) = ⟨v(x),v(y)⟩` on the L2-normalized threshold-centered context bundles — the exact vector lane induction and the frozen `ReferenceClassifier` consume, and the matrix a kernel-independent/black-box FMM (the issue's own route 2) would compress from samples. Cross-check with the deployed sign-bit/masked-Hamming kernel. Oversized regions deterministically subsampled to ≤1024 rows.
- **Admissibility**: `arccos(⟨p_A,p_B⟩) > η · max(diam)`, `diam = 2·q95` member angular radius; η = 1.0 primary (0.5/2.0 sensitivity).
- **Numerical rank**: `r(ε) = #{σ_i ≥ ε·σ_1}`, ε ∈ {1e-2, 1e-3, 1e-4}. No RNG in pair/member selection; seed 290501 for the null control only.

### Numbers

Admissible same-depth pairs at η = 1.0: **2 / 5 / 16 / 29 / 87** at depths 1–5 (≤ 1.7% of pairs; η = 2.0: none anywhere; η = 0.5: 63–85%).

Within-pair nested scaling (same pair, blocks n = 32…1024; median r, r/n in parentheses; cosine kernel, η = 1.0):

| n | r(1e-2) | r(1e-3) | r(1e-4) |
|---|---|---|---|
| 32 | 15.0 (0.47) | 29.0 (0.91) | 32.0 (1.00) |
| 128 | 20.0 (0.16) | 80.0 (0.62) | 119.0 (0.93) |
| 512 | 20.0 (0.04) | 123.5 (0.24) | 247.5 (0.48) |
| 1024 | 20.5 (0.02) | 134.0 (0.13) | 274.5 (0.27) |

Median within-pair growth exponents (log-log slope of r vs n, 37 pairs): **n^0.11 at 1e-2, n^0.45 at 1e-3, n^0.64 at 1e-4** (η = 0.5 sensitivity: 0.28 / 0.55 / 0.66). Null control (random subsets, n = 1024): r = 57 / 198 / 287 — admissible blocks sit 2.9× (1e-2) and 1.5× (1e-3) below unstructured blocks; r(1e-4) ≈ 275–287 approaches the ambient ceiling (the cosine Gram kernel is structurally rank ≤ D = 288).

### Verdict against the pre-registered kill condition

The kill condition triggers iff r(ε) grows **proportionally** with block dimension. It does not, at any ε — largest exponent 0.64, and the 1e-2 rank is flat (~20 across a 32× size range, 2.9× below null). **§5.1 does not kill the proposal; per the decision rule it proceeds to §5.2.** Qualifications that shift the burden onto §5.2:

1. **Compressibility is a 1%-tolerance phenomenon.** At 1e-3 the rank grows ~n^0.45–0.55 (a power law — not "roughly constant or polylogarithmic" in the rule's wording, though ≤1.5 decades of n can't fully exclude a high-degree polylog); at 1e-4 blocks saturate the ambient rank. If the scorer needs a better-than-~1% far field, the growth law removes the asymptotic advantage at realistic sizes.
2. **Leaf-sized blocks are full-rank at ε ≤ 1e-3** (r/n = 0.91 at n = 32; compression only meaningful for n ≳ 512) — the §5.4 constant-factor hazard is already visible here, compounded by η = 1 admissibility being scarcest at the shallow depths where blocks are largest.
3. **The deployed sign-bit geometry is ~2× worse** than cosine at 1e-3 (median r 266–279 vs 134 at n = 1024) — binarization destroys low-rank structure, consistent with the #230 precedent the issue itself cites.
4. **Monopole (baseline 3, Barnes-Hut) error is σ₂/σ₁ ≈ 5–7%** on these blocks — the bar any FMM/H-matrix machinery must clear.

So: §5.2 must demonstrate that a ~1%-precise, rank-~20 far field preserves held-out top-1 / bits-token within the pre-registered ε — with the issue's own #234 caveat in force (a Gate C score answerable by exact-context lookup cannot register a long-range improvement, so §5.2 needs its own novel-context held-out set before any number is quoted).

Leaving the issue open for review; per §10, either a recorded pass or a recorded negative was the goal — this records a conditional pass.


### comment 5160924290 — 2026-08-02T23:46:14Z — auser

## §5.2 pre-registration — criteria fixed before any numbers are quoted

§5.2 required that this spike "either wait on #234 or define its own held-out set
containing genuinely novel contexts, with the construction documented before any
numbers are quoted." Recording the setup now, before measuring, per that
requirement and §10.

### The blocker is cleared, and not by us

#234, #231 and #232 are all closed. #234 in particular was resolved by fixing the
instrument rather than by constructing a new eval — its closing note states
"Construction method: none needed — resolved by measurement," with strict
full-code semantics putting the existing D3 story split at 50.3% strict miss
(13.9% under metric-consistent dot probes), above the 5% threshold.

So §5.2 does **not** need a bespoke held-out set. The `score_report.json` harness
on the pinned fixture is a valid basis. #231's closure also makes bits/token
usable, which it was not when this issue was written.

### But the blended number must not be the criterion

Post-#293/#294 the report carries per-status slices. From #234's closing evidence:

| slice | n | top-1 | bits/token |
|---|---:|---:|---:|
| ExactContext | 14,919 | 41.1% | 8.55 |
| Graph | 15,080 | 1.23% | 16.47 |
| Novel | 37 | — | — |
| blended | 30,036 | 21.05% | 12.53 |

The entire §3a motivation is long-range context. A far-field operator cannot
improve rows answerable by exact-context lookup, and those are ~half the corpus
carrying 41.1% top-1. Judging on the blended figure would dilute any real effect
by roughly 2× and let an improvement on the graph path hide inside noise on the
exact path — the same failure mode #234 was opened about, one level down.

**Primary criterion is therefore the Graph slice** (n = 15,080), which is also
what #234's closure names as "the number to improve."

### Pre-registered criteria

1. **Primary — Graph slice.** Top-1 agreement and Witten–Bell bits/token against
   the current `r4g1` engine on the same pinned split. Improvement here is the
   only thing that would support the §3a claim.
2. **Guard — blended and ExactContext slices.** These must not regress: kill at
   >0.5 pp absolute top-1 drop (the ε already proposed in §5.2), or bits/token
   worsening against the teacher floor. A far-field operator that helps the graph
   path by damaging the exact path is not a win.
3. **Reported but not decision-bearing — Novel slice.** n = 37 is far too small
   to carry a verdict; quoting a top-1 delta on 37 samples would be noise. It is
   recorded for completeness only. Anyone tempted to headline it should not.
4. **Precision context from §5.1.** Compressibility measured as a ~1 %-tolerance
   phenomenon (rank ~20 at ε = 1e-2, flat in block size; growing n^0.45 at
   1e-3). §5.2 must therefore demonstrate accuracy is preserved *at that
   precision*, not at an unspecified one — a far field needing better than ~1 %
   has no asymptotic advantage at realistic sizes.

### Order of work

Per §7 item 3, the next implementation step is **baseline 4 (ℋ-matrix / HODLR),
not full FMM** — same premise, far less code, and it tests the compile-time-table
approach against the #230 fragmentation precedent before any M2L machinery is
written. §5.2 measures that baseline. Full FMM is only evaluated if ℋ-matrix wins
on the Pareto front.

### Statistical note, registered in advance

The graph slice sits at 1.23 % top-1. At n = 15,080 the standard error on a
proportion that small is ~0.09 pp, so a 0.5 pp move is comfortably resolvable —
but the floor is low enough that relative improvements will look dramatic while
being absolutely small. Both absolute and relative deltas get reported, and the
kill condition stays absolute.


### comment 5161054540 — 2026-08-03T00:17:18Z — auser

## §5.2 gate — negative. The far-field operator cannot reach usable precision, and there is no asymptotic win to buy.

Per §10 ("a recorded answer either way"), recording a negative. This is the
offline precision check that **gates** §5.2's accuracy measurement, not §5.2
itself: held-out top-1 / bits-token on the Graph slice was never run, because the
precondition it rests on — that a compressed far field reproduces the
interaction operator at a stated tolerance — fails first. Nothing was built into
the deployed path, the artifact format, or the kernel; total cost was one SVD per
admissible pair.

### Method

Measurement-only, on the same bit-for-bit cover as §5.1 (`cover_kappa
blake3:67d957c3…`, 362 regions, 159,658 train observations; all three pinned
kappas re-verified). Admissible pairs recomputed and matching §5.1 exactly:
**2 / 5 / 16 / 29 / 87** at depths 1–5, η = 1.0.

Error is measured against the **truncated SVD**, which by Eckart–Young is the
optimal rank-k approximation. That makes every number here an *upper bound on
achievable quality*: any practical scheme — ACA, cross approximation,
interpolatory bases — does worse. If optimal rank-k misses, nothing cheaper hits.

Reported quantity is operator error, `||Kx − K̃x|| / ||Kx||`, median over 79
admissible pairs at n = 1024 with 32 random right-hand sides. An ℋ-matrix is
*applied*; Frobenius error of an isolated block would flatter the result.

### Numbers

| rank | cosine: rel Frobenius | σ_k/σ₁ | **rel matvec** | sign-bit: rel Frobenius | σ_k/σ₁ | **rel matvec** |
|---:|---:|---:|---:|---:|---:|---:|
| 1 (monopole) | 10.81% | 5.32% | **15.02%** | 6.88% | 3.07% | **10.29%** |
| 8 | 6.28% | 1.84% | **8.97%** | 4.72% | 0.95% | **6.97%** |
| 16 | 4.55% | 1.12% | **6.64%** | 4.13% | 0.67% | **6.05%** |
| 20 | 4.04% | 0.92% | **5.97%** | 3.93% | 0.61% | **5.82%** |
| 32 | 2.98% | 0.63% | **4.53%** | 3.44% | 0.50% | **4.98%** |
| 64 | 1.61% | 0.30% | **2.45%** | 2.53% | 0.34% | **3.68%** |

Rank 1 reproduces §5.1's monopole figure (σ₂/σ₁ = 5.32%, against the recorded
5–7%), and σ₂₁/σ₁ = 0.92% reproduces `r(1e-2) ≈ 20` exactly. The pipeline agrees
with §5.1 wherever the two overlap.

### Finding 1 — `r(ε) ≈ 20` is not a 1% error budget

`r(ε) = #{σᵢ ≥ ε·σ₁}` counts spectrum above a threshold. It was never an error
bound, and the two differ by ~6× here: at rank 20 the *next* singular value is
0.92% of the first, while discarding the tail costs 4.0% Frobenius and **~6%
operator error**.

My own §5.2 pre-registration earlier in this thread inherited that ambiguity
when it wrote "a ~1%-precise, rank-~20 far field." Those are two different
quantities. Correcting it here: there is no rank at which this far field is 1%
precise without approaching the ambient dimension.

Against the bar that matters, rank 20 beats the Barnes-Hut monopole by only
**2.5× (cosine) / 1.8× (sign-bit)** — for the whole M2L apparatus, admissibility
bookkeeping, and compile-time translation tables.

### Finding 2 — the kernel is already globally low-rank, so there is no O(n²) to remove

This is the structural objection and it does not depend on any tolerance.

The interaction block, as §5.1 operationally defines it, is

```
K_AB = V_A V_Bᵀ
```

an inner product of L2-normalized threshold-centered context bundles in
D = 288 dimensions. It is therefore **exactly rank ≤ 288 by construction** —
§5.1 already noted this ceiling in passing, observing r(1e-4) ≈ 275 approaching
it. The exact operator application factors by associativity:

```
K x = V_A (V_Bᵀ x)        cost O((n_A + n_B)·D), exact, no approximation
```

FMM exists to accelerate kernels such as 1/r that are *not* globally low rank;
its win comes from replacing a dense O(n²) interaction that has no exact
factorization. Here the interaction is an inner-product kernel that already has
one. Any far-field operator must therefore beat rank 288 to earn anything at all,
and at rank 64 the error is still 2.5–3.7% — roughly 4.5× compression at a
precision no one pre-registered as acceptable. Driving error toward 1% pushes the
rank back toward the ambient dimension, where the compression vanishes.

The premise in §2 — "three of the six FMM stages are already built" — holds. What
does not hold is the assumption that the missing stage buys an asymptotic
improvement. Against this kernel it buys a constant factor bounded by D, and
§5.4 already flagged constant factors as "the usual assassin."

### Incidental correction to §5.1

§5.1 recorded the deployed sign-bit / masked-Hamming geometry as ~2× worse than
cosine (median rank 266–279 vs 134 at 1e-3) and inferred that binarization
destroys low-rank structure. In **reconstruction-error** terms that does not
hold: 5.82% vs 5.97% at rank 20, essentially identical, and sign-bit is *better*
at rank 1 (10.29% vs 15.02%). The ~2× gap is real in rank counts and absent in
error. The #230-style concern is not supported by this measurement.

### What this does and does not kill

**Killed:** FMM / ℋ-matrix / HODLR far-field aggregation over this interaction
object. Per §7 the sequencing was ℋ-matrix before FMM precisely so this could be
answered cheaply; it has been, and full FMM does not need evaluating.

**Not killed:** the §3a motivation. Long-range context remains a real gap — the
graph-path slice sits at 1.23% top-1 (#234's closure names it "the number to
improve"). This result says a far-field operator over the semantic Gram is not
the mechanism, not that the goal is wrong.

### Caveat on Finding 2

It rests on the interaction object being the Gram block of 288-dim bundles, which
is how §5.1 operationalized it and how a far field would be constructed. If the
intended object is something else — a kernel that is not an inner product of
bounded-dimension vectors — the associativity argument does not transfer and this
should be re-examined. Flagging rather than asserting it as settled.

### Reproduction

Branch `issue-290-hmatrix-prototype`, `research/290-fmm/` — `fmm_dump.rs`
(extractor, self-verifying against the pinned kappas), `validate_51.py` (§5.1
reproduction gate), `hmatrix_proto.py` (this measurement). Tracked deliberately:
the original §5.1 analysis lived in an untracked scratch directory and was
destroyed by a working-tree cleanup, which is why the reproduction status and
both known discrepancies are written into the README rather than left implicit.


### comment 5161309680 — 2026-08-03T01:11:21Z — auser

## DoD evidence

Closed by merged PR #356 at `cc8d3ba03eefc66858d4935de9988b3023ae858c`.

- Added the reproducible §5.2 novel-context protocol in `docs/fmm_290_novel_context_protocol.md`.
- Added a deterministic certifier-side low-rank FMM candidate scorer and fixed-point feasibility scorer.
- S7 measured 96 novel teacher-forced positions: top-1 1.04%, top-8 26.04%, and teacher cross-entropy 10.588 bits/token.
- Rank 20 retained 0.6967 energy; fixed-point selection matched float selection and reduced estimated table storage from 337,888 B to 164,056 B.
- Validation passed for formatting, graph-certify tests/clippy, graph-format no_std checks, and the S7 BDD scenario; the full workspace suite retains the documented pre-existing R4G1 allocation-census failure.

Scope remains honest: this is certifier/research-path evidence, not deployed multiplication-free FMM runtime integration.

### comment 5161337830 — 2026-08-03T01:17:31Z — auser

## Reconciling the §5.2 gate negative with #356's positive result

#356 landed a certifier-side low-rank FMM scorer and reports gains (top-8 +20.83
pp, teacher bits/token −3.367 on 96 novel positions). I posted a §5.2 gate
negative shortly before it. **These are not in conflict, and the difference is
exactly the caveat I attached.**

### They compress different objects

My measurement targeted the interaction block **as §5.1 operationally defined
it**: `K_AB = V_A V_Bᵀ`, an inner product of L2-normalized 288-dim context
bundles. Finding 2 was that this is exactly rank ≤ D = 288 by construction, so
the exact matvec already factors as `V_A(V_Bᵀx)` and a far field buys a constant
factor bounded by D rather than an asymptotic win.

I flagged the limit of that argument explicitly:

> It rests on the interaction object being the Gram block of 288-dim bundles …
> If the intended object is something else — a kernel that is not an inner
> product of bounded-dimension vectors — the associativity argument does not
> transfer and this should be re-examined.

#356 compresses **graph prototypes against emission residuals** — a
regions × vocabulary object, not an inner product of bounded-dimension context
vectors. The associativity argument does not apply to it. So the negative is
scoped to the Gram-block far field, and #356's object was never covered by it.

### What still stands from the gate

- The Gram block is rank ≤ 288 by construction; a far field over *that* object
  cannot deliver an asymptotic improvement.
- `r(ε) ≈ 20` counts spectrum above a threshold and is not an error budget —
  optimal rank-20 truncation of the Gram block costs ~6% operator error, not 1%.
- The sign-bit and cosine geometries are equivalent in reconstruction error
  (5.82% vs 5.97% at rank 20), correcting §5.1's inference that binarization
  destroys low-rank structure.

None of those bear on the emission-residual object.

### One number worth checking

#356 reports **retained energy 0.6967** at rank 20 / tolerance 1e-2. Depending on
whether that is a σ or σ² ratio, it implies a substantially larger relative error
than the ~1% tolerance the §5.2 pre-registration was written around — and yet
top-8 and bits both improved. If that holds up, the interesting conclusion is
that the precision framing inherited from §5.1 was the wrong lens for this
object: the emission matrix apparently tolerates coarse approximation because
what matters is candidate-set quality, not faithful reconstruction. Worth
confirming the definition before leaning on it.

Also worth noting the sample: 96 novel teacher-forced positions is small, and
"novel" is a slice with very few positions in the fixture Gate C run (n = 17
there). The direction is encouraging; the magnitude deserves a larger set before
it is quoted as settled.

### Relation to today's graph-path work

Separately measured on the Gate C fixture corpus (#358, #360): the graph path
retrieves the teacher token 63% of the time and selects it 2% of the time,
because the selected chain's emissions contain that token only 1.35% of the time
while the scorer boosts chain emissions. That is a chain-selection defect
upstream of any scoring change, and it is worth knowing about before attributing
emission-scorer gains — a better scorer over a chain that excludes the answer has
a low ceiling.



---
_Comment/review ids in this file: 5153705445, 5160924290, 5161054540, 5161309680, 5161337830_

