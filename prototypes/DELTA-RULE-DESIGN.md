# Delta-rule associative memory for the UOR-R4 geometric stack

> **Preserved research; not an adopted model or qualified mechanism (October 3).**
> This standalone prototype and its original logs are retained to prevent loss of
> negative evidence. The document below is the original investigator narrative,
> including assumptions and claims that were not independently established.
> The full-model embedding gradient is defective/unverified; the isolated core
> check does not qualify it. The best reported hit rate is7.03%, against an exact
> store at100%, on an authored task and one seed. The native lowering, parameter
> accounting, state-capacity arguments and unfetched literature reference remain
> hypotheses. No production crate, model selector or roadmap adopts this code.
> No new fit follows this preservation. See the active [plan](../docs/integration/project-track.md).


**delta-rule-architect** · base `origin/main` @ `38377c87` · worktree `~/uor-r4-worktrees/delta-rule-memory`. Owner checkout untouched. All stated ground truth (rrarra 6×288, mlp 749, 7,153,860 params, lane-wise λ, dead Lorentz reads, D0-b, MQAR 1/109 vs 108/109, 0 distractor hits) taken as given.

## 1. Design

**Why.** `S_t = α_t S_{t-1}(I − β_t k_t k_tᵀ) + β_t v_t k_tᵀ` **projects out** the component along `k_t` before writing. A 288-vector `h_t` cannot: it only superposes, so a later same-key write cannot overwrite an earlier value. A 288×288 `S_t` holds ~288 near-orthogonal associations and overwrites one without disturbing the rest — targeting the measured "binds the relation, emits the distractor's value" failure.

Read: `geometric_stack.rs:6795-6884`, dispatch `:5577-5598`. Layers are `x += mixed; x += mlp(x)` — a **residual stream**, so width-288 memory needs no new interface. The quaternion block is lane-wise (`:6848-6877`) and cannot express cross-lane erasure, so the memory must be a separate matrix state.

**Placement.** New mixer `d` in the **four `r` layers only**; `a` untouched. **One store shared by all four**; sharing costs 1.4% worst-case cross-layer interference and saves 4× state and parameters. `W_k,W_v` are *learned* projections, so `r_kv` must stay **well below 288** — near-orthogonal learned keys degenerate the store to a scalar sum. Band `r_kv ∈ [32,128]`.

Per `d` layer, with `u = RMSNorm(x)`:

```
k=normalize(W_k u); q=W_q u; v=W_v u            # W_*: 288 x r_kv, k unit
read  o = qᵀ S_{t-1};  x += W_o o
write S_t = α_t S_{t-1}(I − β_t k kᵀ) + β_t v kᵀ
α_t = exp(−softplus(a)·sigmoid(W_α u))   (existing form, geometric_stack.rs:6831-6841)
β_t = sigmoid(w_β·u)
```

**`d` replaces the `r` mixer in its layer; it does not compose with it.** Both are linear state transitions, so series composition gives `λu(I − βkkᵀ)` — expressible by neither alone and with no capacity argument behind it.

**Backward.** With `A_t ≜ α_t(I − β_t k kᵀ)`, `S_t = A_t S_{t-1} + β_t v kᵀ`:

- **The erase adjoint is the same erase operator**: `A_t` is symmetric (positive-definite for β∈[0,1]), so `dL/dS_{t-1} = A_tᵀ(dL/dS_t) = A_t(dL/dS_t)` — no transpose, no extra matmul beyond the forward's, same stability. The key structural fact.
- **Adjoints contract columns of `dL/dS`, not rows.** `ΔS[i][j] = β·δ_j·k_j` perturbs column `j` alone, so `dv_j = βk_j Σ_i G[i][j]`, and `k` picks up the normalization adjoint `(dk − k(k·dk))/‖k_raw‖`. I first contracted rows — a plausible but wrong gradient; worth a regression test.
- **Unit keys bound the spectrum** at `{1−β, 1}`, so the 256-step backward product cannot explode; unnormalized keys make it unbounded.

I verified the delta-rule **core** in isolation against central differences and two independent symbolic forms to `<1e-8` relative error on a 4×4 single-write case. **Caveat:** the prototype's *full-model* gradient check still shows 2–3 disagreeing indices per block — the core is exact, the wrapper's embedding-gradient path is not, and that most likely explains §3's weak absolute numbers.

## 2. Cost vs. the 7.15M budget and D0-b

From `StackConfig::shapes` (`geometric_stack.rs:1084-1160`); baseline reproduces 7,153,860 exactly at mlp 749. Memory = `2·288·r_kv + 2·288 + r_kv` shared (×4 per-layer); `mlp_h` re-matched to hold the total constant.

| r_kv | params shared (×4) | mlp_h | total | state scalars | 4-bit KiB shared (×4) |
|---:|---:|---:|---:|---:|---:|
| 16 | 9,808 (39,232) | 747 | 7,153,300 | 4,608 | 2.2 (9.0) |
| 32 | 19,040 (76,160) | 745 | 7,152,164 | 9,216 | 4.5 (18.0) |
| 64 | 37,504 (150,016) | 742 | 7,155,076 | 18,432 | 9.0 (36.0) |
| 128 | 74,432 (297,728) | 735 | 7,155,716 | 36,864 | 18.0 (72.0) |
| 288 | 166,752 (667,008) | 717 | 7,154,724 | 82,944 | 40.5 (162.0) |

The store is **cheap in parameters** — 2.3% of budget at full rank, absorbed by the MLP (749→717); fp32 training state is 2.2 MiB at batch 32. The state increase is the point: 288 scalars (`r` layer) → 18,432 at `r_kv=64` for 0.5% of parameters.

**D0-b.** ✅ low-bit maps (ternary + per-row power-of-two scales, as `learner/lowbit.rs` already does, `DECISIONS.md:124-131`); ✅ sigmoid by table read, scaling by shift; ✅ `‖k_raw‖` in fixed point with a reciprocal table; ✅ read MAC **only if** both operands are power-of-two-quantized. ❌ **The outer product `βvkᵀ` and erase `β(Sk)kᵀ` on arbitrary fixed-point vectors are not in the D0-b set** — definitional, not incidental.

Satisfiable only by cutting precision: **(F1)** bit-sliced shift-add multiplication (`Σ_j (u<<j)·bit_j(v)`, static shifts) — exact, 4–8× cost, `r_kv²` MACs/token (~83k at r=288, one dense 288×288 layer; trivial at 64). **(F2)** power-of-two keys/values — shift-add, capacity collapses. **(F3)** diagonal-only erase — expressible, but it *is* the per-lane decay `r` already has, so not a substitute. **Recommend F1 at `r_kv ≤ 128`; full rank 288 is where the multiplier ban first breaks.**

## 3. Prototype — built, with weak absolute results

`prototypes/delta_rule_memory.rs`: self-contained std-only Rust (no workspace dependency, `#![forbid(unsafe_code)]`), one `rustc` invocation. Same task and budget, three variants: `--arm delta`, `--arm gla` (existing vector state, `h_t = gate·h_{t-1} + βv`), and a **zero-parameter exact nearest-key store** as ceiling. Task: `n` distinct (key,value) pairs, one extra pair repeating an earlier key with a *different* value, then a query key; only the final position is supervised (sparse).

```
rustc -O -C target-cpu=native -o /tmp/drm prototypes/delta_rule_memory.rs
OMP_NUM_THREADS=2 /tmp/drm --arm delta --d 64 --n 24 --steps 1500 --batch 32 --lr 0.01 --seed 1
```

| arm | d | n | loss | hits | distractor |
|---|---:|---:|---:|---:|---:|
| exact store (**0 params**) | 64 | 24 | — | **1.0000** | 0.0000 |
| delta | 64 | 24 | **3.7973** | **0.0703** | 0.0000 |
| gla | 64 | 24 | 3.8700 | 0.0312 | 0.0078 |
| delta | 32 | 12 | 3.8614 | 0.0469 | 0.0078 |
| gla | 32 | 12 | 3.8678 | 0.0312 | 0.0234 |

Logs: `prototypes/results/`, reproduced exactly on rebuild. Delta trains (4.95→3.75) and beats the matched vector state in **both** configs (+0.039 hits at n=24), distractor hits 0.0 vs 0.0078. **But 7% against an exact store at 100%** — one seed, 1500 steps, no tuning. The mechanism runs and is not worse; this is not a capacity win and, given §1, not load-bearing.

## 4. Is it worth the parameters?

**Against the exact store (108/109, 0 params).** For retrieval as such the store wins on every axis: zero parameters, no gradient pathology, exactly D0-b-compliant, 108/109 vs 1/109. The delta rule must not be funded to duplicate it. Its only defensible claim is **graded generalization the store cannot provide** — reading a key never seen, decaying relevance, interference-controlled writes; the measured 14/120 unseen-relation binding is that regime. Fund it **only** for approximate/generalizing recall.

**Against the curriculum (reported 0.021→1.000, unchanged fixed-state model).** The stronger objection: if a curriculum recovers the failure for free, it is an optimization/supervision problem and parameters buy a symptom. My run supports it only weakly (curriculum did not rescue the vector state at 1500 steps) — too short to falsify, and my implementation is imperfect. **I have not refuted the curriculum, so it dominates**: zero new parameters, no new D0-b exposure, no new backward pass, versus 37k–167k parameters and multiplier-ban risk inside the operation that defines the mechanism.

**Recommendation: do not commit parameters now.** Run §3 on the real 7.15M stack with the curriculum as a matched control, and fund the delta rule only if it wins **on unseen keys/relations** at matched budget with that arm present. 7% vs 100% is not that case, and the flattening dose-response (0.003 nats for the last 1,000 steps) warns more state may not move held-out bits-per-byte.

**Strongest counter-argument.** The 7% ceiling is more plausibly my full-model gradient defect (§1) and the read-only-at-the-last-position design than any limit of the delta rule — in the literature the delta rule *is* the known-good solution to this task, so 7% is evidence about my implementation, not the mechanism. Repair the gradient first: if it then reaches 100% at a fraction of the state, the question becomes sample efficiency rather than capability and the spend is justified.

## Uncertainty

The prototype's full-model gradient is unverified (its core is exact), so 7% is not a mechanism ceiling; one seed, 1500 steps, `d∈{32,64}` not 288. arXiv 2609.16183 was not fetched — its numbers are the Lead's input. The 7,153,860 total reproduces only with the non-layer total back-derived (1,178,232 → vocab 4091.08); I could not locate that 1,704-param difference from `shapes()` alone. D0-b analysis is instruction-set reasoning, not a `serving_multiplier_check.py` run; no energy was measured, which D0-b requires before any serving claim.
