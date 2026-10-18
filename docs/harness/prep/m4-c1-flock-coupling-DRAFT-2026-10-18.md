# C1 — float-trained flock coupling, exported as integer Gate weights (DRAFT)

- **Status:** DRAFT design and finite-control specification. It is **not registered**.
  - TEST FITNESS is **not requested**.
  - No compute, no source change, $0.
  - The pre-registration follows the Dose 1 decision (`m1-learned-attention-link-2026-10-17.md`), because C1 starts from that decision's checkpoint.
- **Line:** M4 starling/murmuration read (#2032); conditional future rung. **Lab:** claude.
- **Authority and rules:**
  - [D37](../../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised): float is legal in offline training only; the export is the integer boundary.
  - D11: served integer, no float or multiplier instruction.
  - D23/D25: no softmax, squashing or normalized exponential.
  - D25/D30: no matmul/GEMM.
  - D24: GPU only.
  - D29: scanned modules.
  - D22: a negative closes a configuration only.
  - Brief: [starling-read.md](../../mechanisms/starling-read.md) § Next configuration.

## 1. Mechanism

Today's discrete Flock read (`crates/uor-r4-discrete/src/discrete.cu:125-267`) computes, per row and payload block `b`:

```
u_b      = query payload root (8 coords)
m_{b,l}  = snap_{found}( Σ_k action(BFS[frame_k][f_q], payload_k) )   per level l
out_b    = snap_{terms}( u_b + Σ_{l: found_l>0} g_{b,l} · m_{b,l} ),   g ∈ {−1,0,+1},  terms = 1 + Σ_{l: found_l>0} |g_{b,l}|
snap_n(x) = argmin_r ( L1(x − n·r), Hamming(x, n·r), r )   over the 16 orbit roots
```

**C1 changes only the coupling.** `g_{b,l}` becomes a learned coupling `K_{b,l}`:
- **Training (offline, float):** `K ∈ ℝ`.
- **Serving (exported):** `K ∈ {−K_max … +K_max}`, applied by `|K|` repeated additions of `±m_{b,l}`, with `terms = 1 + Σ_{l: found_l>0} |K_{b,l}|`. A level with no eligible candidate (`found_l = 0`) adds nothing and does not count (`discrete.cu:250`, which runs `continue` before `++terms` at `:260`).

Everything else stays fixed and integer. That includes the selection, the rank order, the transport, the consensus `m`, the writer and every other learned table/route/gate, all frozen at the Dose 1 decision checkpoint.

**Inventory and learning signal:**
- Dose 1's configuration (`Spec{V4096,B4,C192}`) has `blocks·stages = 20` combine couplings, read from the Dose 1 ledger: `GateCombine (0..19)`. The Relation variant adds 25 relation trits (5 edge types × 5 stages).
- **What the test asks:** does a learned coupling *magnitude*, found by float training, carry information that the trit learner cannot reach?
- **Why it is cheap:**
  - In Flock mode, selection and `m_{b,l}` do not depend on the combine gates, because the rank uses `rel`, frame L1, Hamming and recency (`discrete.cu:222-255`; consensus `m` at `:256` is computed before the gate is applied).
  - The read output `q` feeds only the writer (`discrete_forward`, `discrete.cu:375-380`).
  - So `(u_b, m_{b,l}, target)` tuples can be produced once on the GPU, and C1 optimizes over them. The traced forward exports most of what is needed: the query payload root is head word 3, each level's consensus root is level word 2 (traced even for gate 0), and `found_l` is level word 1 (`discrete.cu:218-256`). The untraced path skips consensus at gate 0 (`:227`), so the cache must use the traced forward. It is **not** complete: `root_id` returns ABSENT when an input block is not exactly one of the 16 roots (`:282-285`), and the consensus is ABSENT when `found_l = 0`. Control C0 (§4.0) therefore gates the cache.
  - Under Relation, the same holds while the relation trits stay frozen.

**Why this is the right next question:**
- The value/readout step is where the historical starling runs failed: the source was kept, but the readout picked wrong, as recorded in rungs 2 and 3 of the brief.
- A trit can only add, subtract or ignore a level's consensus. It cannot weight one level twice as heavily as the query.
- `snap` already builds `n·r` by repeated addition (`discrete.cu:102-110`). An integer coupling therefore adds no new arithmetic class at serving.

## 2. Where float lives, and the integer boundary

**Float lives only in the offline float module**, on the GPU (D24). It is a separate crate or module with its own nvcc-built PTX, with no NVRTC, and its own D29 allowlist entry in `scripts/check_native_cuda_vendor.rs`. The discrete crate's integer scan is not widened. The module contains:
- `y_b(K) = u_b + Σ_{l: found_l>0} K_{b,l} m_{b,l}` (an elementwise scalar multiply-add, not a matrix product);
- `n_b(K) = 1 + Σ_{l: found_l>0} |K_{b,l}|`, matching the integer `terms` exactly;
- the relaxed objective (§3) and its subgradient;
- a first-order update followed by a **box projection** `K ← clamp(K, −k_max, +k_max)`. The projection is required because the writer's address margin (8) exceeds the minimum inter-root L1 distance (4), so the hinge can keep favouring larger `|K|` (F2). SGD is the default. Adam is admissible only with bias correction by a running product (`β_t ← β_t·β`), never `pow(β, t)`. Its step uses `sqrt`, which is not a squashing function.

**Banned in the module (scanned tokens):** `exp`, `expf`, `__expf`, `log`, `logf`, `softmax`, `tanh`, `sigmoid`, `erf`, `pow`, `wmma`, `mma`, `cublas`, `cutlass` and `cudnn`.

**Export:** `K_int = clamp(round_half_away_from_zero(K), −K_max, +K_max)`. The export is the only crossing point:
- The served read is the integer kernel: a new `ReadSpec::FlockCoupled { kept, k_max }` beside `Flock`, never a change to `Flock`.
- Its rollback identity is `k_max = 1 ≡ Flock`, bitwise.
- Gate storage is already `i32`. Validation widens from `[−1, 1]` to `[−k_max, k_max]` for this ReadSpec only.

**`k_max` bound:** `k_max ∈ {1, 2, 3}`, giving `terms ≤ 1 + stages·k_max ≤ 16` at C192. Whether `k_max > 1` is admissible under D11 is the open owner/PM decision recorded in the brief. Repeated addition is multiplier-free; arm A1/A2 at `k_max = 1` is the fallback if it is refused.

## 3. Float objective (declared; piecewise-linear; no squashing)

**The proxy follows the writer's own cost structure** (`writer`, `discrete.cu:307-345`), relaxed only where `snap` would be applied. The writer compares the snapped read `q` in two ways:
- **token term:** the target token against its rival tokens, by L1 over **all** blocks (`token_distance`, `:277-281`);
- **address term:** the target's address cell against occupied rival cells. The address cell is built from **block 0 and the last block only** (`token_cell`, `:286-290`).

C1 replaces each block's snapped read `q_b` by its float mean `ȳ_b(K) = y_b(K) / n_b(K)`, and each comparison `L1(q_b − v)` by `L1(ȳ_b − v)`.
- Dividing every candidate's distance by the same `n_b` leaves `snap`'s argmin unchanged.
- It keeps the distances in root units, so the writer's margins apply unchanged.
- It weights every block equally, as the integer `token_distance` and `address_distance` do.

```
T(t; K)   = Σ_b L1( ȳ_b(K) − table_b(t) )                  token comparison, all blocks
A(c; K)   = Σ_{b ∈ {0, B−1}} L1( ȳ_b(K) − cellroot_b(c) )   address comparison, frame blocks
H_μ(d*, d) = max(0, μ + d* − min_rival d)
L(K)       = Σ_rows [ 8 · H_width(T(target), T(·)) + width · H_8(A(target_cell), A(·)) ]      width = 8·blocks
```

- These are the margins and weights of `writer` under `NearestAddressMarginV1` (`discrete.cu:326-340`): token margin `width`, address margin 8, and `loss = 8·tm + width·am`.
- **Rival sets** are recomputed at every step from `ȳ`, by the writer's own procedure with `q` replaced by `ȳ`: occupied cells for the address term, and the tokens in the `kept_cells` admitted cells for the token term. With no rival, the hinge is 0, as in the writer.
- **Unsupported:** `OccupiedCellRankHingeV1` (the rank term has no relaxation, so its gradient is zero almost everywhere). C1 requires a `NearestAddressMarginV1` checkpoint, which Dose 1 is (m1 prep §7.12.4).
- **Gradient convention:** the gradient is zero when a hinge argument is ≤ 0. At `|K| = 0` the derivative of `|K|` is 0, and L1 ties take the lowest index, as in `snap`.
- **Labels:** training labels enter the objective only, never the forward. No DEV labels are used, and no oracle source index.
- **Proxy-agreement instrument (pre-run, TRAIN only):**
  - At the frozen checkpoint's own trits, the proxy's argmin token must agree with the integer writer's `prediction` on at least a pre-registered fraction of TRAIN rows; the threshold is set at registration from a sampled measurement.
  - The rate is reported.
  - A rate below the threshold is an **instrument stop** under D31, never a negative.
- **Building block:** both terms are sums of the mean-L1 hinge primitive `H_μ(L1(ȳ − v*), L1(ȳ − v_rival))`. §4.2 tests that primitive with literals on one block; the composed `L` is tested by C0 and the agreement instrument.

## 4. Finite controls (hand-derived literals; generated anchor orbit at the pinned source)

**Roots** (`build.rs` `ROOTS`, at `origin/main` `d1f0a9a99`): signed ±2 pairs on the coordinate pairs (0,1), (2,4), (3,7) and (5,6). In particular:
- `r0 = a = [2,2,0,0,0,0,0,0]` and `r1 = −a`;
- `r3 = −b = [0,0,−2,0,−2,0,0,0]`, `r4 = [0,0,−2,0,2,…]`, `r13 = [0,0,2,0,−2,…]` and `r14 = b = [0,0,2,0,2,0,0,0]`.

Every root has L1 norm 4 and two nonzero coordinates.

**Fixture T′:** the C8 harness of `flock_qualification.rs:129` with its tokens swapped to `[[A,A],[A,B]]`. That is context 2, kept 1, one stage, and `Gate(1)` = block 1, level 0, with input `identity(1,0), identity(0,1)`.
- The query is token 0, `(A, A)`.
- In block 1, `u = a`. The context node's frame (block 0 = `a`) equals the query frame, so the identity action carries its payload `b`, and `m = b`.
- Block 0 has `Gate(0) = 0`, so its output is the query payload `A`.

### 4.0 Cache fidelity (gates the tuple cache)

| ID | Control | Expectation |
|---|---|---|
| C0 | Cached replay equals the live forward | On a pre-registered TRAIN sample, the cached-tuple integer replay at `K ∈ {−1, 0, +1}` (the checkpoint's own trits) equals the live `discrete_forward` bitwise, in both read output and writer loss. Rows with an ABSENT `u_b` or an ABSENT consensus at a level with `found_l > 0` are counted and reported. Any such row refuses the run with a typed status; it is never silently dropped. |

### 4.1 Integer export kernel (`FlockCoupled`)

| ID | Control | Literal expectation |
|---|---|---|
| X1 | Zero coupling is inert | `K = 0` → out `[A, A]`, `terms = 1`, bitwise equal to `Flock` with all gates 0 |
| X2 | Trit embedding: `k_max = 1 ≡ Flock` | For `K ∈ {−1, 0, +1}`, the vectors, losses and full trace equal the `Flock` kernel bitwise on T′ and on all six existing Flock control fixtures (`flock_qualification.rs:349-356`). On T′: `+1` → `snap(a+b, 2)`, an L1 tie of 8 between `a` and `b` and a Hamming tie of 4, broken by index, so the output is `A`. `−1` → `snap(a−b, 2)`, a tie between `a` and `r3` broken by index, so `A`. Out `[A, A]` with `terms = 2` in both cases. |
| X3 | Magnitude flips the read | `K = +2`: residual `[2,2,4,0,4,0,0,0]`, `n = 3`. `D(b) = 8` is the unique minimum (`D(a) = 16`, `D(r13) = D(r4) = 16`, everything else ≥ 20). Out `[A, B]`, `terms = 3`. |
| X4 | Signed magnitude | `K = −2`: residual `[2,2,−4,0,−4,0,0,0]`, `n = 3`. `D(r3) = 8` is the unique minimum. Out `[A, −B]` with `−B = r3`, `terms = 3`. |
| X5 | Saturation | `K = +3`: residual `[2,2,6,0,6,0,0,0]`, `n = 4`, `D(b) = 8`, `D(a) = 24`, `D(r4) = D(r13) = 20`. Out `[A, B]`, `terms = 4`. |
| X8 | Empty level does not count | A level with `found_l = 0` and `K = +2` leaves `terms` and the residual unchanged, and the float `n_b` excludes it. The fixture is a two-stage context whose upper level has no eligible node; the literal is derived at FIT. |
| X9 | Two levels at once | `K = (+2, −1)` on a two-stage fixture gives `terms = 4` and residual `u + 2m_0 − m_1`. The literal is derived at FIT from the C7 two-stage harness (`flock_qualification.rs:109`). |
| X10 | `n = 16` bound | With `k_max = 3` and 5 stages, `terms = 16`. `snap` builds `16·r` by repeated addition with no overflow (`\|16·2\| = 32`). Its existing "n ≤ 9" comment is updated, and a fixture at `n = 16` returns the hand-derived root. |
| X11 | Relation trits stay trits | Under a Relation variant, the combine gates widen to `[−k_max, k_max]`, while the 25 relation trits are still validated in `[−1, 1]` (status 10 otherwise). |
| X12 | Trace encoding | Level word 0 is `gate + k_max` under `FlockCoupled`. The current `gate + 1` encoding would make `−2` equal ABSENT. X2's full-trace bitwise identity applies at `k_max = 1` only. |
| X6 | Refusals | `\|K\| > k_max` in a snapshot, `k_max ∉ {1,2,3}`, or a `FlockCoupled` gate count that differs from `blocks·stages`: each refused with a typed status and no state change. Neither `Route` nor `Flock` accepts `\|g\| > 1` (the existing status 10). |
| X7 | Counterfactual lanes | The widened discrete learner proposes `2·k_max + 1 ≤ 7` ordinals in the existing 16 lanes, in the fixed order CURRENT, then `0, −1, +1, −2, +2, −3, +3` skipping CURRENT, with lanes `2·k_max+1 … 15` left at MAX. On T′ with `Gate(1) = 0` and target `B`, the lanes rank so that `K = +2` and `K = +3` improve and `K = +1` does not. The exact writer losses are derived at FIT time from the writer's literal costs, as in C8 (`[448, 256, 64]`); they are not computed here. |

### 4.2 Float training module

These rows test the mean-L1 hinge primitive `H_8` (the address-term margin) on T′ block 1 (`u = a`, `m = b`), with target `r* = B` and rivals = all other roots.
- **Closed forms for `K ≥ 0`, with `n = 1+K`:** `D(B) = 8/n`, `D(a) = 8K/n`, `D(r4) = D(r13) = (8+4K)/n`, and every other root ≥ `(8+8K)/n`. These are the X-table distances divided by `n`, so the argmin is unchanged.
- **Minimum rival:** `8K/n` for `K ≤ 2`, and `(8+4K)/n` for `K ≥ 2`.

| ID | Control | Literal expectation |
|---|---|---|
| F1 | Loss and subgradient at `K = 1` | `n = 2`: `L = 8 + 4 − 4 = 8`, `∂L/∂K = −16/n² = −4`; one SGD step with `η = 1/8` gives `K = 1.5` |
| F2 | No interior fixed point; projection at the box | For `K ≥ 2`, `L = 8 − 4K/(1+K)` and `∂L/∂K = −4/(1+K)² < 0`. `L` tends to 4 as `K` grows: the writer's residual address hinge at the exact root `B`, because `r4` and `r13` sit at L1 4. At `K = 3` (`k_max = 3`): `L = 5`, `∂L/∂K = −1/4`, and the step followed by projection returns `K = 3`. At `k_max = 1`, starting from `K = 1`, it returns `K = 1` |
| F3 | Sign symmetry | Target `r* = −B` (`r3`), start `K = −1`, `n = 1 − K = 2`. `D(r3) = 8/n = 4`, `D(a) = 8\|K\|/n = 4`, and the others are ≥ 6. `L = 8`, `∂L/∂K = +16/n² = +4`; the step with `η = 1/8` gives `K = −1.5` |
| F4 | Float/integer agreement | At `K = 2.0`: `D(B) = 8/3` against `16/3`, so the float output is `B`. The export gives `K_int = 2`, so the integer output is `B`. Disagreement 0 |
| F5 | **Export-gap witness** | At `K = 1.4`, `n = 2.4`: `D(B) = 3.333…`, `D(a) = 4.666…`, `D(r4) = D(r13) = 5.666…`, so the float output is `B`. The export gives `K_int = 1`, so the integer output is `A` (X2 tie). The export-gap instrument must report this block as **1 disagreement** and block 0 as 0 |
| F5b | Rounding at a half | `K = 1.5` (F1's step) exports to `2`, and `K = −1.5` exports to `−2` (half away from zero) |
| F5c | Determinism | Two runs at a fixed seed give a bitwise-identical `K` vector. No atomics or unordered reductions are used in the update, or the order is fixed |
| F6 | Banned-operation scan | The D29 scan of the module's sources finds zero of the §2 tokens. It fails on a seeded `expf(` fixture line in a scanner unit test, not in the module |
| F7 | No float in the export path | The exported snapshot passes the discrete `validate` with `ReadSpec::FlockCoupled` and loads into the integer binary unchanged. The integer kernel source passes the existing D32 integer token scan |

**Source of truth for expectations.** Every expectation is a literal derived from the generated ROOTS above, with no CPU reference forward. A GPU result that differs is a control FAIL that stops the line for diagnosis (D31), never a mechanism negative.

## 5. Export-gap contract (D37 item 3)

1. **Same evaluator.** The float and the exported integer candidates are scored by the same DEV evaluation as Dose 1, on the same rows and in the same order:
   - `eval-corpus`: `B3(P_succ / P_comp / full)` and conditional top-1;
   - `attention-cell-reference-scorer` (#2604): `T1 / T1b / T2 / U_succ / U_train`;
   - the same 14,287 DEV rows, and later the same sealed fresh draw.
2. **Float forward.** This is the D37 item-3 export-gap instrument: GPU-only, confined to the D29-scanned float module, and never served. The owner/PM is asked to confirm this reading at registration (§7). The float forward used only for the gap is the §1 read with real `K` and `snap` over real vectors, run on the GPU in the float module. Its downstream writer is the unchanged integer writer, fed the float read's chosen root.
3. **Reported for every candidate:**
   - float metrics, integer metrics, and their per-metric differences, all reported;
   - the per-block read-output disagreement count and rate (F5 is the instrument's test);
   - the exported snapshot's hash and `k_max`.
4. **Decision use.**
   - Decisions use only the **integer** metrics. The gap is never subtracted from a result or used to promote one.
   - **Export-limited flag:** the metric is integer `P_succ` conditional top-1, and the band is arm A0's two-seed range on that metric. A float-minus-integer gap larger than that band flags the candidate as *export-limited*.
     - At most **one** successor change (`k_max` or the export rule) may follow. If the flag fires again, the configuration closes (D22).
     - If the integer metrics win while the gap is large, the KEEP is decided on the integer metrics, and the gap is reported beside them.
5. **Identity.** Every artifact binds the source, the Dose 1 checkpoint hash, the cached-tuple file hash, the seed, `k_max` and the float-module PTX hash.

## 6. Arms and fair-test outline (registered after Dose 1)

| Arm | Coupling learner | `k_max` | Isolates |
|---|---|---|---|
| A0 | Existing discrete trit proposal learner, as trained in Dose 1 | 1 | reference |
| A1 | Discrete widened-ordinal counterfactual learner (X7) at matched proposal budget | 3 | the effect of magnitude |
| A2 | Float-trained (§3), exported | 3 | the effect of float training (vs A1) |
| A2-int | A2's optimum, rounded and then polished by A1's counterfactual search under the same remaining budget | 3 | proxy error vs export error |
| A2′ | Float-trained, exported | 1 | the export ceiling at the trit (vs A0) |
| Z | `K ≡ 0` ablation | — | that the read is used |
| R | Random `K_int` refit control at A2's `\|K\|` distribution | 3 | magnitude without learning |

**Conditions:**
- **Budget matching:** budget is counted in full-writer TRAIN evaluations plus accepted moves. A1, A2 and A2-int get equal caps; float proxy steps count as one writer-equivalent evaluation per row scored.
- **Scope (trunk frozen):** the trunk was co-adapted to trits at the Dose 1 checkpoint. A C1 negative therefore says nothing about coupling magnitude trained jointly with the trunk, and closes only this frozen-trunk configuration (D22).
- ≥ 2 seeds (D22). The Dose 1 DEV panel (14,287 rows; `P_succ` 3,847).
- Sealed fresh transfer reported for every KEEP. The bars are fixed before the run and are never narrower than two-seed noise.
- **Cost:** the cached-tuple training covers 20 scalars. The dominant cost is one GPU pass to cache the tuples, plus the integer evaluations. A projection is made in the sheet from Dose 1's measured per-batch cost.

**Dose 1 branches:**
- **Relation KEEP:** C1 learns the 20 combine couplings on the Relation checkpoint, with the 25 relation trits **frozen**. `rel` drives selection (`discrete.cu:236-248`), so learning those trits would invalidate the tuple cache and X11. Learning them as couplings is out of scope and needs its own non-cached sheet.
- **Relation negative** (that configuration only): C1 runs on the Flock arm checkpoint and asks whether magnitude rescues the read.
- **Instrument stop:** C1 waits.

## 7. Open items for the pre-registration

1. `k_max > 1` admissibility under D11 (repeated addition) — PM/owner.
2. Whether this is the starling line's decisive run under D21 or a new line — PM.
3. X7's exact writer losses on T′ — derived at FIT time from the writer's literal costs.
4. Owner/PM confirmation that the §5.2 float gap forward is the D37 item-3 instrument, offline, GPU-only and never served.
5. The proxy-agreement threshold (§3) and the C0 sample size, both set at registration from a sampled TRAIN measurement.

## 8. D22 registration blocks (NOT_REGISTERED, NOT_RUN, $0)

This section supplies the blocks the harness requires. It completes the sheet's shape only:
it registers nothing, sets no bar or budget, changes no mechanism or control, and authorises
no compute.

### Bar (NOT_REGISTERED)

No bar is registered. This DRAFT sets no numeric bar; the proposed comparison and the
two-seed-noise rule remain those in §5.4 and §6, and are fixed at registration before any
result is visible.

### Base artifact (NOT_REGISTERED)

No base artifact is bound or loaded. At registration the base will be the Dose 1 decision
checkpoint named in `m1-learned-attention-link-2026-10-17.md`; its exact artifact hash is
recorded then. Dose 1 has not run, so no checkpoint is available to this DRAFT.

### Data split (NOT_REGISTERED)

No data is read. At registration C1 inherits the Dose 1 TRAIN/DEV split (14,287 DEV rows;
`P_succ` 3,847), with sealed fresh transfer reported for every KEEP; the C0 and
proxy-agreement samples are drawn from TRAIN at registration (§3, §7.5). This DRAFT adds no
split, draws no sample and opens no data.

### Steps (NOT_RUN)

No step has run. After registration and FIT, the proposed order is: C0 cache fidelity →
finite controls (C0, X1–X12, F1–F7, F5b, F5c) → tuple cache → arms A0, A1, A2, A2-int, A2′, Z, R →
export-gap report. Every step remains gated by registration and the owner holds; nothing
here authorises compute.

### Stop rule (NOT_REGISTERED)

No run exists to stop. At registration the stop rule stays: any finite-control FAIL stops
the line for diagnosis (D31) and is never a mechanism negative; an instrument stop waits;
an export-limited flag permits at most one successor change before the configuration closes
(D22). All owner holds remain in force.

### Cost (NOT_REGISTERED)

$0. Zero GPU-hours, zero pods, zero model compute; this repair is a host-only documentation
edit. Any future cost is projected at registration from the Dose 1 measured per-batch cost
(§6), and none of it is authorised here.

### Minimum test power (NOT_REGISTERED)

No decisive run is registered. The proposed design states the D22 §3 minimum: ≥ 2 seeds, the
14,287-row Dose 1 DEV panel (≥ 200 rows), sealed fresh transfer for every KEEP, and bars not
narrower than two-seed noise (§6). No power claim is admitted by this DRAFT.

**Fitness status.** DRAFT, not registered; no FIT is requested and no compute is cleared.

TEST FITNESS: PENDING
