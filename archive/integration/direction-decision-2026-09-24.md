# Direction decision — 2026-09-24 (owner-directed)

**Status.** Owner-adopted decision record. On 2026-09-24 the owner instructed the run lead to apply the
recommendations of an independent three-reviewer reassessment — architecture (K3), mathematics and computer
science — of [repo-review-direction-2026-09-23.md](repo-review-direction-2026-09-23.md). This record adopts those
recommendations, corrects three stale claims, and fixes the ordered next work. It changes no artifact, runs no model
and asserts no new capability; it is a change of emphasis, **not** of goal.

**Authority.** Owner instruction, 2026-09-24. Supersedes the "owner decisions requested; not yet adopted" status of
`repo-review-direction-2026-09-23.md` §11. The owner's 2026-09-23 terminal objective — a fully transformerless model
in which geometry replaces floating-point matrix multiplication at serving — stands unchanged; it remains an
objective, not a measured result.

**Evidence base (read-only).** The three reviews verified against `origin/main` `f95b9efb`. Key verified facts: the
best served language artifact (`learner/transferable_lexical.rs`) contains **no geometry at all** (zero
prime/zeta/R4/S3/H4/`Z[phi]` references) and is a 64-dimensional integer RNN; no geometric mechanism has beaten an
information- and compute-matched ordinary control on any task; the served readout is **dense per-token**
(611,814 packed-coefficient inspections/step vs 56,457 nonzero); the merged `depth_probe`
(`crates/uor-r4-core/src/bin/support/principal_continuation.rs`) reports previous-token recovery of
**97.84 % / 74.86 % / 18.69 % / 0.60 %** at lags 1–4, so the earlier "second-order context is absent (2.1 %)" claim
is a raw-decode artifact and the memory horizon is soft (~2–3 tokens of decaying retention).

## 1. Goal decomposition — adopted

- **Goal S (serving arithmetic) — KEPT.** "No floating-point matmul and no dense transformer at serving" is
  theorem-backed for the permitted operator class. Two open defects: the served path is D0-b-compliant but **dense**
  per token, and no quality-matched whole-path energy measurement exists.
- **Goal R (geometric predictive advantage) — RE-SCOPED to a gated hypothesis.** Geometry is retained by default
  only where already load-bearing (identity, addressing, version authority, serialization) and as **candidate**
  sparse-access structure / parameterisation (quaternion/H4 4× only under equivariance; exact finite-table closure).
  It earns a causal serving role **only** by beating an information- and compute-matched ordinary control on the same
  probe at matched capacity and cost. This is a positive re-scoping, not a demotion of the objective.

## 2. Decisions recorded in DECISIONS.md

| ID | Decision |
|---|---|
| **D4** | Goal decomposition and the geometry gate (§1 above). |
| **D5** | **Per-token parameter sparsity is the terminal serving invariant.** D0-a's distinguishing test survives D0-b as the *end-state* contract; dense low-bit maps remain permitted as an interim experimental substrate but must be reported as non-compliant with the end state and cannot be claimed as the target. This converts Goal R into a measurable sparse-access/routing contest. |
| **D6** | **The target objective is a long-range information probe**, not the own-docs order-2 count prior. The count prior is retained as a standing local control. |

## 3. The staged plan — adopted, with amendments

Adopted from `repo-review-direction-2026-09-23.md` §8, with the reviewers' amendments:

1. **Stage 0 — record correction (no compute).** Applied by this record; the stale claims in §5 are fixed.
2. **Stage 1 — frozen-state adjudication (cheap, no retraining).** Run the transport-decode arm first (it is already
   answered on main; report it as estimator scope, not a live contradiction), then the readout ladder — (i) linear
   float, (ii) rank-K bilinear, (iii) 2-layer MLP, **(v) count-augmented linear as the decision arm** — on
   document-held-out frozen states. Strengthened guard: the integer marginal-relative composite already demonstrates
   closability-to-the-count-table with second-order table features, so a large readout gain measures *learned-smooth
   vs count* feature form, not language. **Commit in advance that Stage 2 and Stage 3 run on any Stage-1 branch.**
3. **Stage 2 — one-variable gate + capacity ablation, crossed with corpus.** A D0-b-honest **hard compare/select
   gate** (a data-dependent multiply is a forbidden multiplier) and `h_dim` growth are **separate axes**: a gate
   changes the memory horizon; `h_dim` changes simultaneous capacity only. Cross {64, 256, 256+gate} × {repo-only,
   broadened corpus}. Endpoint on the Stage-3 probe, with `repository_bits` demoted to a `+0.05` guard.
4. **Stage 3 — cross-window state + the information-theoretic probe** (induction / copy-at-distance where the tuned
   order-2 count control is at chance by construction), plus a natural-text long-range agreement panel, an equal-work
   stateful n-gram control, and the count prior as a standing local control. **This is the deciding metric.** Avoid the
   authored-fixture trap: seed-fresh episodes, source-disjoint held-out panel, and a verified at-chance control.
5. **Stage 4 — bounded-integer/LUT attention-like access**, only on a witnessed long-range need. Any attention-like
   layer must execute inside the declared bounded-integer/LUT kernel; float softmax requires `exp` (transcendental)
   and would be an explicit owner goal change, not a default.
6. **Stage 5 — LUT kernel + scale + whole-path M1 measurement** (tokens/s, J/token, RSS) against
   llama.cpp / bitnet.cpp. Only after a measured capability over the count control.

**New track added by the reviewers (largest gap in the prior plan):** the programme has one genuinely distinctive,
working asset — **exact addressed memory with version authority** (16/16, survives replacement). No stage integrates
it with the sequence model. The coherent target is a **memory-augmented multiplier-free LM**: an ordinary low-bit
recurrent/hybrid backbone with exact addressed memory as a first-class retrieval/tool subsystem, and geometric
addressing/identity/serialization as the memory's native infrastructure. This is also the only form in which Goal R
survives honestly — **geometry as per-token sparse access structure**, tested against ordinary LSH/learned-kNN at a
matched access budget.

## 4. Ordered next work

The next milestone is the **"escape from bigram class" diagnostic gate** — the Stage-1 frozen-state adjudication
(fused with one minimal gate × capacity arm on a synthetic induction probe). Plan:
[escape-bigram-class-plan-2026-09-24.md](escape-bigram-class-plan-2026-09-24.md). It is deliberately a **diagnostic
gate that selects the next code change**, not a capability claim.

## 5. Stale claims corrected by this decision

1. `model-direction-2026-09.md` asserted "first-order local context is present and second-order context is not
   (prev at 2.1 %)". **Corrected**: the merged record recovers `prev` at ~75 % under a transport-aware
   (cur-peeled) decode; the 2.1 % figure is the training-free raw-embedding rule. Lag profile: 97.84 / 74.86 /
   18.69 / 0.60 % at lags 1–4.
2. The rank description is split by scope: the readout input is **147-wide** (type-level rank ≤ 147), but for
   ordinary prose (m = 0, typed block and event constant within a window) the **effective per-position rank is
   ≤ 64** — there is no "rank-64 versus rank-147" disagreement, only a scope distinction.
3. `README.md` now states, in the model-paths table, that the best language artifact contains **no geometry**, and
   formalises the disclaimer that UOR-R4 is **not** a "world model" in either industry sense.

## 6. What this decision does not change

The terminal objective; D0-b's arithmetic allowance; D1 (falsify before extension); D2 (ablation as measurement, not
verdict); the frozen TLA/R4G1 kernel contract; and every prior result at its recorded scope. No capability, energy,
frontier or geometric-superiority claim follows. The next milestone must be started under a recorded prospective
resource projection (the shared ledger is ≈ 99 % consumed).
