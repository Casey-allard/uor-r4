# Architecture review — D37 and the post-dose plan

**Route:** DeepSeek/deepseek-flash (owner cost direction, 25 Sep). No Kimi/Moonshot call. Read-only: no edits, no posts, no compute. Repo at `21224b446`.
**Reviewed:** `AGENTS.md`; `DECISIONS.md` D0-b/D5/D8/D11/D19–D37; `project-track.md` § Post-dose optimization path and § Rung 1; `current-state.md` (newest entries); `optimization-path-2026-10-18/README.md`; all requested `docs/mechanisms/` pages; the active K16 sheet; the C1, BVG-K and decode-t/s drafts; the Q1 diagnosis record; and the relevant source (`discrete.cu`, `training.rs`, `build.rs`, `check_native_cuda_vendor.rs`).

---

## 1. Verdict

**The governance is coherent; the sequencing is not.** D37 is written narrowly and correctly — float is a training instrument, the integer export is the boundary, and the bans that exclude *expensive operations* survive. But the post-dose plan was authored before D37 and has not absorbed it, and the lab is not "circling one thing" out of stubbornness: the K16 line is the only line with a FIT'd sheet, a base artifact and a live pod. Single-line focus is defensible for the next 48 hours and indefensible beyond that — not because dormant mechanisms are being wasted, but because **three already-FIT'd diagnostics costing $0–$1 can decide whether the M1 bottleneck is representation, objective or optimizer, and none of them has run.** Worse, the first-principles derivation below finds the most likely cause is a fourth thing neither the K16 redirect nor any dormant float mechanism addresses: **the codebook is not injective at initialization (≤16 distinct full vectors for 4,096 tokens), and the discrete learning law accepts only ~1,600–1,800 vector moves in a full pass, which puts the sheet's own `Bar (i)` at or just above the aliasing ceiling.** My recommendation: keep the K16 qualification and A/R run (≈$2 on the fast path) but add a pre-run reachability gate; run the $0 stored-checkpoint census today; run the FIT'd read-code capture on the same lease; register BVG-K as the *matched discrete competitor* to float; register exactly one float sheet (a differentiable writer objective over the table) only if both fail. Of the dormant set, exactly one item is promoted — the prime-zeta **initializer**, on injectivity, not on zeta semantics. D37 needs one plan-level answer now: a single export-gap contract with a declared decision rule, before any float sheet is registered.

---

## 2. What D37 actually changed — strict reading

| Question | Answer under D37 | Evidence |
|---|---|---|
| Float forward/gradients/optimizer in offline training | Legal, GPU-only, Rust, in a **separate scanned module** | D37 §1, §2, §6 |
| Float *evaluation* used as a decision metric | **Not** the decision metric. Every float candidate reports its export gap on the same evaluator; a float score "qualifies nothing" | D37 §3 |
| Softmax/softplus/sigmoid/tanh/GELU, incl. renamed/table forms | Still absolute, in objectives, gradients, sampling, teachers, comparators | D37 §2, D23, D25, D30 |
| Dense matmul/GEMM | Still absolute, including "wrapper laundering" | D37 §2, D25, D30 |
| CPU fallback, Python, transformer comparisons, neural graders | Still banned | D24, D27, D28, D30 |
| The direct-K16 path and its integer scans | Unchanged; Dose 1 stays integer-only as FIT'd | D37 §6 |
| Holds | Training, M2, Stage B HELD; Q4 NOT_EXECUTABLE. D37 releases nothing | D37 §6 |
| Archived float sheets | Re-registration only, fresh FIT + owner release where a hold applies | D37 §5 |
| Zeta-zero complex-plane exception | Unchanged and separate | D37 §4, D32 addendum |

**The reading that matters, and that the plan has not internalized:** "archived float sheets return by re-registration" does **not** mean the frozen sheets are now runnable. Four of the five were built on the dense float trunk (`crates/uor-r4-training`, hundreds of `Tensor`/matmul lines, 133 softmax mentions), and matmul is still banned in training. Their *sheets* cannot return; only their mathematical content can, re-expressed on a matmul-free substrate. `crates/uor-r4-training` cannot be the float module — D29 counts presence, and D37 §6 requires a new allowlist entry, never a widened discrete scan. This is the single most likely violation in the first month of D37.

A second reading: **most of the "dormant" mechanisms are not dormant in substance.** Rank-margin is active as the integer writer hinge `8·T + width·A`; Flock/Relation are active as `ReadSpec::Flock`/`Relation`; SpiralCore/FBS transport is active; prime-zeta construction is active. The mechanism index labels the *float implementation* dormant while the discrete implementation is active under the same name (`docs/mechanisms/README.md`, status column). The owner is reading a status table where "dormant" mostly means "the float version is preserved", not "the idea is unused". That is a documentation artifact, and it inflates the apparent value on the table.

---

## 3. First-principles derivation: information content, symmetry, failure cases, serving cost

### 3.1 The carrier and the code

Per block, the carrier is one of 16 roots — signed-permutation images of `[2,2,0,0,0,0,0,0]`, all L1 norm 4 (`crates/uor-r4-discrete/build.rs:227-248`). So 4 bits/block, `B=4` → 16 bits/vector for a 4,096-token vocabulary (12 bits). The code alphabet is 4 bits wider than the vocabulary: an injective embedding is *possible*, not forced. The realized code is whatever the table holds.

**The initializer is not injective.** `discrete_init` sets `state = seed ^ block ^ 0x9e3779b9`, mixes, and takes `state & 15` as the root index (`discrete.cu:350-356`). The block index is `t·4+b`, so the low nibble depends on `t mod 4` and `b`; the project's own measurement is ≤16 distinct full-vector patterns across 4,096 tokens at epoch 0 (`docs/mechanisms/prime-zeta-complex-routing.md` §2; `docs/labs/a2-pilot-diagnosis-2026-10-14/README.md`), and the K16 sheet states the consequence: "the xorshift low-4-bit initializer populates ≤16 occupied cells at checkpoint 0" (`m1-discrete-rung0-admission-redirect-2026-10-15.md`, Fact 5).

**Consequence 1 — the writer cannot emit an aliased token except its representative.** `token_cell` and `token_distance` read only the table vector; ties break to the smallest token id (`discrete.cu:307-321`). Two tokens sharing a vector are indistinguishable to the read and the writer for every query. The number of tokens the model can ever predict is therefore bounded by the number of **distinct full vectors**, not by the vocabulary, the read, or the objective.

**Consequence 2 — the aliasing ceiling is close to the registered bar.** Each accepted `VectorBlock` move changes one block of one token, so it can create at most one new distinct vector. With the observed acceptance rate (396 accepted of 3,584 vector proposals at Q1 ≈ 11.0%, `docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md`), one canonical pass of 16,384 vector coordinates yields ≈1,800 accepted moves, hence **≤≈1,820 distinct full vectors** — and ≤≈1,820 predictable tokens. At K=16 Q4, admission is 2,400, so the top-1 ceiling is `min(2,400, ≈1,820) ≈ 1,820`, **below `Bar (i)` = 1,941**. The bar is unreachable at the registered dose unless acceptance rises by ≳7% *and* admission holds — a margin inside run-to-run variation that has not been measured. This is the weakest link in the argument (the acceptance rate is from the interleaved Relation/Q1 lineage, not the canonical K16 lineage), which is exactly why it should become a **pre-run gate**, not a verdict.

### 3.2 The address key has ≤5 values for a 256-way order

`address_distance` uses only block 0 and block `B−1` of the read output, and compares each to a fixed root (`discrete.cu:274-276`). The read output is always an exact root after `snap`, so each endpoint distance ∈ {0,4,8} and the sum ∈ **{0,4,8,12,16}** — five values, log₂5 ≈ 2.32 bits, used to order up to **256** occupied cells and admit the first K. Ties are broken by cell id (`cell_precedes`, `discrete.cu:296-298`), and cell id is `first_root·16 + last_root` — an arbitrary label, not a geometric invariant.

**Consequence 3 — the K-boundary is largely decided by labelling, not geometry.** On a row with ~189 occupied cells (Q4), ~38 cells share each key value; K=16 selects from the minimum bucket by id. This is why `A` is K-invariant (proved in the sheet: `rival_address` is fixed within the first two occupied cells) and why `R` is the only term that can see rank K vs K+1. It is also why a *better objective* cannot fully repair admission: the key cannot distinguish 256 cells. The sheet already names "Lever 1 — address metric: full-vector / interior-block / cell-envelope" in its `ADMISSION_STILL_CAPPED_AT_K16` branch. That branch should be understood as the *likely* branch, not the fallback.

### 3.3 Symmetry and invariance

- **Gauge symmetry.** The 64-state signed-axis group acts on each block; L1 distances and every hinge are invariant under a common action applied to the query and the table. Therefore (i) seeds that differ by a group element are not independent draws — the D22 two-seed noise floor may be *underestimated*; (ii) checkpoint averaging across seeds must quotient by the group; (iii) a "geometry-preserving token relabelling" control is necessary (one exists in the qualification set).
- **Broken symmetry in the writer.** Cell id is not invariant under the gauge group, yet it decides admission ties and vocabulary ties (`token < best_token`). The served output is therefore **not** gauge-invariant when ties exist, and ties are the rule at 5 values / 189 cells. This sits awkwardly with the project's own discipline that "exact identity stays separate from geometry" (AGENTS.md; `unified-geometric-carrier-credit.md`, Boundaries).
- **Scale is discrete.** Table entries are exact roots; `snap_n` divides by the count; the writer's margins are fixed (`width=32`, 8). There is no continuous scale to exploit — which is why a float master must be quantized at export and why the export gap is structurally non-zero (C1's F5 witness is a clean example: `K=1.4` float → B, export 1 → A).

### 3.4 Failure cases, in order of likelihood

1. **Aliasing** caps top-1 before any objective or optimizer question is reached.
2. **Address-key quantization** (5 values) makes the K-boundary tie-decided.
3. **Update starvation**: one coordinate per batch, 16,429 coordinates, one pass = 16,404 batches → ≈1 visit per coordinate per pass, ~64 rows of evidence each.
4. **Objective scale confound**: `R ∈ 0..240` vs `A ≈ 18/row` at the same coefficient (the sheet discloses this, Amendment 6; the mitigation is per-checkpoint Σ8T/Σ32A/Σ32R reporting — adequate, but a win can still be a scale effect).
5. **Denominator**: the model predicts only inside admitted cells (`writer` loops over admitted cells), while `Bar (i)/(ii)` compare to an unconstrained count predictor (1,226 / 4,060). The admission-matched comparator (`table_prediction_admitted`) is designed in the read-code sheet but unrun.
6. **Read cannot exceed table identity**: no read mechanism repairs an aliased table.

### 3.5 Serving cost

- Read: ≤286 nodes × ≤8 edges × sweeps ≈ 10⁴ integer ops, ~0.2% of the writer (`finite-relational-head.md` §4.2).
- Writer: `O(K·256)` cell scans + `O(K·V)` membership + distances; the project's own figure is ≈5×10⁶ vocabulary-cell comparisons per row (`finite-relational-head.md` §4.2).
- **Every float-offline candidate exports to these same kernels, so its serving-cost delta is zero** (C1's `|K|` repeated additions are ≤60 vector adds, ~10² ops — noise). D37 mechanisms must be judged on (a) quality per training GPU-hour and (b) export gap, **never** on serving cost.
- The real serving-cost defect is the writer's `O(V)` scan, which violates D5's end-state; no D37 mechanism touches it. Do not let a float result be presented as a cost result.

---

## 4. Matched ordinary comparators (same inputs, same budget)

| Candidate | Matched ordinary mechanism | Why it is the honest control |
|---|---|---|
| Float credit on the table | **BVG-K**: discrete batched-coordinate scoring, same coordinates, same objective, same rows, matched wall clock (`m1-batched-verified-coordinates-DRAFT-2026-10-18.md`) | Same parameter set, same objective, same GPU. Isolates "continuous credit" from "more search per second". It is the only comparator that makes D37's value falsifiable without an export gap. |
| Float credit on the table, second control | **Injective init** (PrimeZeta or a random injective null) with the objective and law unchanged | Tests whether the bottleneck is capacity/init rather than optimizer. If this alone moves Q4 admission/top-1, float is not needed for M1. |
| Relation read | **T2** context-free successor table on the same 192-token window (`d34-successor-table-control-2026-10-17.md`) | Exact, O(1) memory, no geometry; bounds what the geometric read adds on its own headline panel. T1 is tautological (3,847/3,847) and is only a panel check. |
| Prime-zeta codebook | **NormalNull** (permuted primes) + **quota-only injective random** | Separates *injectivity* from *zeta geometry*. The brief already specifies exactly these controls; the primary contrast should be injectivity, not semantics. |
| C1 flock coupling | **A1** discrete widened-ordinal learner at matched proposal budget | Already in the C1 draft and correctly designed; the problem is the base artifact, not the control. |

The pattern is that for every float candidate the project already has, or can build in a day, a discrete comparator that runs on the same population. That is the correct response to D37: **do not compare float to yesterday's weak discrete law; compare it to the best discrete law at the same budget.**

---

## 5. Per-mechanism reactivation assessment

Cost classes: **XS** <1 GPU-h/<$2; **S** 1–5 GPU-h/$1–8; **M** 5–30 GPU-h/$5–40; **L** 30–150 GPU-h; **XL** new trunk.

| Mechanism / sheet | Status today | Genuinely re-registrable under D37? | Concrete re-registration form | Decision informed | GPU cost | Disposition |
|---|---|---|---|---|---|---|
| **Prime-rank + zeta-phase codebook** (`prime-zeta-complex-routing.md`) | Construction active & qualified (17+6 controls, 4,096/4,096 unique); learned attribution dormant | **Not a D37 item** — the constructor is already legal (Oct-14 exception + integer boundary). But it is the highest-value dormant *test*. | 4-arm init comparison at V4096/B4/C192/K16, objective A fixed, 2 seeds, full pass: PrimeZeta / TangentNull / NormalNull / xorshift default. Trained in from step 0. Bar: Q4 target admission and top-1 with the alias census as the mechanism check; fresh transfer on KEEP. | Is admission erosion an **injectivity/curriculum** artifact? Does a collision-free init remove the 5,680→1,920 collapse? | **S** ≈$4–6 | **PROMOTE** — register after the $0 census |
| **Rank-margin objective + writer-adjoint** (two frozen sheets, one idea) | The *idea* is active as the integer writer hinge; the F32 implementations are frozen | Yes, but only as a **new differentiable writer objective** on a matmul-free module. The archived sheets cannot return (dense trunk). | One sheet: float hinge over the exact writer cost (`8·T + width·A/R`) on the table coordinates only; arms = float-trained → exported, K=1 discrete, BVG-K, random-refit; export gap mandatory; ≥2 seeds, same 14,287-row panel. | Is the **discrete search**, not the objective or representation, the quality bottleneck? | **M** (build + 2–6 GPU-h) | **RETAIN as the single float-credit sheet**; run only after the $0 diagnostics and BVG-K |
| **Secant** and **murmuration-qualification** | Frozen F64 numerical instruments | Yes, as instruments; they answer no live question | Only inside a float sheet that actually uses a secant or F64 qualification; otherwise leave frozen | None | **XS** | **RETIRE at tested scope** — do not schedule |
| **F32 operator** (G2 Hamilton gate, recurrence gates) | Dormant float trunk operators; no separate brief | Only inside a new float module; the trunk is matmul | A float gate over the discrete carrier would duplicate C1's question at higher cost | None distinct | **M–L** | **RETIRE at tested scope** for now |
| **Flock / starling reads**, incl. C1 draft | Discrete Flock/Relation active; C1 drafted | Yes in principle; **blocked in practice** — Dose 1 stopped at Q1, no decision checkpoint, and conditional top-1 on 225 admitted rows is underpowered | Re-base C1 on a post-K16 checkpoint; keep A0/A1/A2/A2-int/A2′/Z/R; the A1-vs-A2 contrast is the valuable part | Can continuous credit beat discrete search at matched budget on 20 scalars? | **S** | **RETAIN; do not register now** (base artifact absent) |
| **Spin-native weights** (quaternion/icosian) | Historical KEEP/parity on the dense float trunk | **No** on the active substrate: the discrete model has no dense projections to replace (the writer is a scan, not a matrix) | Would require a new float Hamilton-block trunk — a new architecture, not a re-registration | Whether a dense float trunk can be matmul-free and competitive | **L/XL** | **RETIRE at tested scope**; reopen only with an explicit architecture decision |
| **E8 orbit memory** | Dormant; +0.033 BPB replacing half the MLPs, behind a learned product-key control | No useful site: there is no MLP on the discrete path | — | — | **L** | **RETAIN dormant**; subsumed by addressed memory |
| **Exact addressed memory** | Dormant (D22 §5(c)); `DialogueSettings` still lacks fields | Not a float question; blocked on a **base artifact** with a memory panel | Needs a trained model + memory fields + trained-in read; ≥20M params, 2–3 seeds, +8/40 bar | D19's lead goal | **M–L** | **RETAIN dormant**; blocked |
| **Pointer identity key** | Archived soft-pointer configs; true entity-address key untested | Not a float question; blocked on a base artifact and the memory panel | Oracle-address ablation first, then a parsed/learned key, ≥3 seeds | Exact copy vs learned attention | **M** | **RETAIN dormant** |
| **Invariant-addressed read** | Dormant spanning-tree line; router campaign cancelled under D23/D24 | Not a float question; needs a working base model and the tangent/normal nulls | Train from step 0, ≥3 seeds, wrong-value primary, compute beside quality | O(n) vs O(n²) read | **M** | **RETAIN dormant** |
| **Unified geometric carrier credit** (M2) | Dormant; whole-path law blocked on softmax | **No** — D37 does not lift the softmax ban, which is what blocks it | Needs a softmax-free episode credit; M2 is held | M2 grounded reply | **XL** | **RETAIN dormant** |
| **Finite relational head** | Active design; Stage M1 NOT_RUN; Stage M2 gated on seed-1 Route at 16,404 | Not a D37 item | **K16-A is a Route run that reaches 16,404** — it can satisfy the Stage M2 prerequisite. Reuse, do not duplicate. | Copy-successor read vs Route | **S** | **KEEP on the K16 line** |
| **Ternary / XOR arithmetic** | Assessed, not adopted; dense contraction | **No** — D37 keeps the dense-matmul ban, and the brief itself says it "does not answer D25" | None as a contraction; the FBS/Gate/Hamming components are already active | — | — | **RETIRE at tested scope** as a contraction proposal |
| **SpiralCore/FBS float projection** | Discrete transport active; F32 projection preserved | Only as the transport inside the float-credit module | Fold into the float-credit sheet if needed | Surrogate vs exact credit | **S** | **RETAIN as a component** |
| **Hopf fiber / S7 / harmonics** | No brief, no operator, no control | **Not re-registrable** — there is nothing to register | Would need a mechanism brief naming the *witnessed limitation* it answers | — | — | **DO NOT OPEN**; no current limitation is a fiber/observation problem |

**Summary of the table:** of 14 rows, one is promoted (prime-zeta initializer), one is retained as the single float-credit sheet, one is retained as a component, four are blocked on a base artifact, and seven are retired at tested scope or not openable. That is what "re-register" actually amounts to — not a plan, a short list.

---

## 6. Prioritized parallel workstreams (after the K16 qualification concludes)

### W0 — Stored-checkpoint alias census + Stage-A re-score (today, $0, host-only, no hold release)

Load the preserved K8 seed-1 Q4 checkpoint and the Dose 1 Q1 checkpoints. Count, per checkpoint: distinct 32-int full vectors across 4,096 tokens; alias-group size histogram; tokens in groups ≥2; within-cell alias mass; occupied cells. Join to the sealed Stage-A 36-point archive (host-only re-score, already FIT'd).

- **Decision:** is `Bar (i)` arithmetically reachable at the tested dose? Is erosion alias-driven?
- **Kill/keep:** if distinct vectors ≥ ~2,000 at Q4 and alias groups are small, **drop the injectivity hypothesis** and restore the objective/optimizer explanations to first place. If distinct vectors ≈1,600–1,800, the ceiling argument is confirmed and the K16 KEEP branch is dead at its dose.
- **Why now:** it is free, it needs no GPU, no pod, no release, and it decides whether the next $2–$12 is worth spending. D24 explicitly allows grading stored outputs on the host.

### W1 — K16 qualification + A/R run, with two pre-run amendments (≈$2, ~1.2 h)

Keep the staged sequence. Before release, amend the FIT'd sheet with:

1. **Replace `G-Pre` with a non-vacuous capacity gate.** As written, `G-Pre ≥ 4,894` at checkpoint 0 passes trivially: with ≤16 occupied cells and K=16, *every* target is admitted (the sheet says so itself: "K=16 admits all occupied cells on every row at initialization"). It cannot distinguish capacity from degeneracy. Replace or supplement it with the census quantities (distinct vectors, tokens/vector, within-cell tie mass).
2. **Add `G-Cap` — a reachability gate.** Require `distinct_vectors ≥ Bar (i)` (net of admission) before a KEEP can be interpreted; otherwise the run is `INSTRUMENT_UNREACHABLE` (D31), not a mechanism negative.

- **Decision:** does the K-boundary objective `8T+32R` slow admission erosion at matched K?
- **Kill/keep:** both arms fail `G-Adm` → close the `(endpoint-only address, K=16, A/R)` configuration pair at scope (the sheet's `ADMISSION_STILL_CAPPED_AT_K16` branch) and route to the address metric/init. `K16-R` beats `K16-A` by >Δ_noise and passes `G-Adm`/`G-Occ` → KEEP and iterate.
- **Parallel:** the $0.91 read-code capture rides the same lease after the qualification; do not rent a second pod.

### W2 — Read-code capacity capture (≈$0.91, 1 h, already FIT'd, same lease)

The FIT'd sheet (`m1-read-code-capacity-2026-10-17.md`) decides **H_rep vs H_law**: does the 16-bit read code carry information the trained output map fails to extract? It includes the admission-matched count-table arms the headline bar lacks.

- **Decision:** representation wall or learning-law wall.
- **Kill/keep:** H_rep → **do not fund float credit**; fix the read/address metric. H_law → continuous credit (or BVG-K) is justified; register it against the discrete comparator, not against K=1.

### W3 — BVG-K, the matched discrete competitor to float (≈$12)

Register the drafted batched-coordinate law (step 3 of the post-dose path) *before* any float sheet. Arms K=1/4/16, ≥2 seeds, primary axis equal wall clock, quality non-inferiority at the registered dispersion.

- **Decision:** is update/search rate the binding training constraint, and can the discrete law capture it without an export gap?
- **Kill/keep:** no wall-clock gain at equal quality → the discrete route to speed is closed and float becomes the only lever. Gain with quality non-inferior → **float is not needed for M1**; keep BVG-K and drop the float-credit sheet from the critical path.

### W4 — Float-credit sheet, conditional (≈$5–10 GPU + host build)

Only if W2 = H_law **and** W3 fails. One sheet, not three: differentiable writer objective over the table coordinates, on a **new scanned module** (never `crates/uor-r4-training`), with a **Stage-1 export-gap probe** (bounded float steps → export → paired integer/float metrics and per-block disagreement) before any full training arm. Arms: float-trained → exported; K=1 discrete; BVG-K; random refit; `K≡0` ablation. Decisions use integer metrics only.

- **Decision:** does continuous credit beat the best discrete law at matched budget, and is the gap acceptable?
- **Kill/keep:** gap beyond the two-seed band or no quality gain → close float at this scope (D22) and keep the instrument. One successor change allowed on the gap; a second flag closes the configuration.

**Parallel vs competing:** W0 is host-only and parallel with everything. W1 and W2 share one discrete CUDA binary and one lease — sequence them on the same pod. W3 needs the same binary; it can follow W1 on a new lease. W4 needs a new module and a different scan; it is a separate lane but must not start before W2/W3. The D19 object-store line (`uor-r4-object-store`) and the documentation corrections are genuinely independent and cost no GPU.

**What I would *not* do:** register C1 as drafted (no base artifact; 225 admitted rows); re-open the F32 operator or spin-native weights (no site on the active substrate); run the prime-zeta *attribution* claim before the injectivity contrast (the attribution would be swamped by the collapse); or start any float work before the $0 census.

---

## 7. Strongest opposing evidence, and what would change my mind

1. **The K16 line is cheap and already staged.** At 0.130 s/batch, four runs are ~1.2 h (~$2). Waiting for diagnostics costs more in calendar time than the run costs in money. *Response:* true, which is why I keep W1 — but the run's KEEP branch is unreachable if the ceiling argument holds, so the $2 buys a diagnosis, not a KEEP. That is still worth it; it is not worth calling it the plan.
2. **The accepted-move extrapolation is from a different lineage.** 396/3,584 is the interleaved Relation/Q1 rate; the canonical K16 lineage may accept more. *Response:* correct, and it is why `G-Cap` is a gate, not a verdict. The $0 census removes the uncertainty.
3. **The model may not need injectivity because context disambiguates.** *Response:* the read and writer see only table vectors; aliased tokens have identical cells and identical distances for every query. Context cannot distinguish identities the table has merged. The argument holds.
4. **The address-key bound may not bind because q varies per row.** *Response:* the pigeonhole is per row, which is where admission is decided. It binds per row.
5. **Maybe the real wall is the read's selection, not the table.** *Response:* possible — and that is exactly H_rep. If H_rep fires, my injectivity emphasis is incomplete and the read/selection work leads. W2 is designed to tell us.
6. **D21/D9 risk:** another diagnostic-first delay can become another loop. *Response:* W0 is $0 and already implied by FIT'd sheets; W2 is $0.91 and already FIT'd. This is not a new campaign; it is executing registered work. I am not asking for a new test campaign.

Measured negative evidence I will not overrule: the Q1 collapse in **both** arms (2,034→225/205) is a configured admission collapse, not a mechanism negative, and I treat it as such. It is, however, strong evidence that the *shared vector/combine pathway* — not the relation trits — moves admission, which is consistent with the aliasing account and inconsistent with "the relation read is the lever".

---

## 8. D37 misreading risks — where semantic drift is most likely

1. **"Float is legal offline" → reusing `crates/uor-r4-training`.** It is a workspace member with hundreds of matmul lines and softmax. D29 counts presence; D37 §6 requires a separate allowlist entry. Most likely violation; guard by making the new module a distinct crate and adding a `uor-harness` check that the float sheet's build graph excludes `uor-r4-training`.
2. **"Archived sheets return by re-registration" → treating the five frozen sheets as runnable.** Four were built on the dense trunk. Only the objective and the numerical instruments can return, on a matmul-free module.
3. **"Export boundary" → scoring float and reporting it as quality.** D37 §3 says a float score qualifies nothing and the gap must be on the same evaluator. A sheet could still put a float number in the headline. Guard: integer metrics are the only decision metrics; float metrics are labelled *instrument*.
4. **"No softmax" → `exp`/`pow` in a loss "because it is offline training, not the model path".** D23/D25 cover objectives and gradients. The inverse drift is also live: C1 treats `pow(β,t)` as banned and uses a running product. Neither the ban nor the over-restriction is written down. **A float sheet must declare its allowed primitive list explicitly** (e.g., `+ − × ÷ sqrt abs min max round floor`, no `exp/log/pow/tanh/...`) and scan for it.
5. **"D32 item 2 still applies: the path must be geometric" → a generic embedding-table SGD with an L1 head.** Matmul-free is necessary, not sufficient. The float module must carry the actual carrier (E8 roots, BFS transport, FBS binders), and the sheet must say which.
6. **Zeta-exception creep.** The Oct-14 f64 exception is scoped to the complex-plane map and its integer boundary; it is not a general float license and cannot justify f64 in a loss.
7. **Holds.** A FIT, a re-registration, or a "D37 makes it legal" argument releases nothing. Only the owner's personal release does (D33 is scoped and expressly non-precedential). Q4 stays NOT_EXECUTABLE.
8. **Live documentation hazard.** `project-track.md` § "Rung 1 computation graph", step 3, still specifies `a = softmax(−energy / temperature)` and an "ordinary recurrent/Householder arm" as the active graph. That text is now doubly wrong (D23 and D37 scope) and is a plausible template for a lab building a float path. It should be annotated or superseded by the PM before any float sheet.
9. **"Export gap" as a number without a decision rule.** C1 invents an "export-limited" flag with one successor change; a second sheet will invent another. Standardize.
10. **Plan item "spinor/quaternion adapters — float offline, discretized serving."** As written it does not name a site or an algebra; a low-rank adapter is a dense projection (matmul). Either specify a per-block Hamilton action (which is a discrete gate and needs no float) or drop the item.

---

## 9. Open questions only the owner can decide

1. **Training-speed lever:** does the owner want the speed problem solved by the **discrete** batched-coordinate law first (no export gap, ~$12), or by **float SGD** (faster move rate, gap-bearing, new module)? This is the core priority call behind D37 and it is not an agent decision.
2. **Interim M1 headline:** is the headline allowed to stay SUSPENDED while the $0–$1 diagnostics run, or does the owner want an interim capability target (e.g., "injective codebook + Q4 admission ≥ 4,894") so progress is legible without a language claim?
3. **Initializer:** may new lineages use an injective codebook (PrimeZeta or a random injective null) while the frozen D5 completion lineage keeps its xorshift default? This is a config/init change, not a kernel change, but it changes the lineage of record.
4. **Pre-run reachability gate:** does the owner accept a `G-Cap` amendment to the FIT'd K16 sheet (a gate that can declare the run unreachable before it spends money)? It is a D31 instrument correction, but it changes a FIT'd bar.
5. **Export-gap decision rule:** what gap disqualifies a float candidate — the two-seed band, a fixed BPB-equivalent, or "reported only"? Without an answer, "reports its export gap" is unenforceable.
6. **Serving-float future:** `optimization-path-2026-10-18/README.md` says serving floats are "a possible future D11 revision only with measured evidence and an owner decision". Does the owner want that path kept open (which would justify building gap infrastructure now), or closed for this planning horizon (which would make the gap a reporting requirement only)?

---

## 10. Required next measurement (one line) and limits

**Run the $0 stored-checkpoint alias census today** (distinct full vectors, alias-group sizes, within-cell alias mass at checkpoint 0 / Q1 / Q4) — it is free, needs no GPU or hold release, and it decides whether the K16 KEEP branch and the float-credit sheet are reachable at all. If it confirms ≤~1,820 distinct vectors at Q4, the next measurement is the FIT'd injective-init comparison, not another objective.

**What I did not test.** No compute, no new artifact. The aliasing ceiling is analytic plus project-cited numbers, with the acceptance rate extrapolated from a different lineage — it is a gate hypothesis, not a measured result. The five-value address key is derived from source (`discrete.cu:274-276`, `build.rs:227-248`) but the empirical tie mass at the K boundary is unmeasured. The export-gap contract is a proposal, not an implementation. And I have not inspected the archived Stage-A per-row records, which may already contain enough to answer the tie-mass question without a GPU; that should be checked before any capture is booked.