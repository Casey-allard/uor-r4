# Hyperbolic geometry, hierarchy and where the compute actually goes — 2026-09-19

Owner question: we already had `artanh`/`arcosh` from earlier Poincaré-ball work; can hyperbolic
geometry save massive compute, and can an operation act on the entire manifold at once? Owner's
original idea: phase shifts (nulls) are avoided by a tangent operation. Owner allowance: a mechanism that
is not the runtime bottleneck may be carried as an **offline test substrate** even if it violates the
serving contract.

Reviewed against the live state, the project's own measurements, and the mathematics.

---

## 1. Correction first: H4 is spherical, so "Poincaré ball on H4" is a category error

The project's **H4 is a finite spherical Coxeter group** — order 14,400, the symmetry group of the
600-cell (120 vertices) and the 120-cell, acting on **S³**. Its 120 roots are the `2I`/H4 roots the
core already composes. Spherical geometry has **positive** curvature.

The **Poincaré ball is a model of hyperbolic space** — constant **negative** curvature. It is not a
different coordinate chart for S³; it is a different geometry. So a Poincaré ball "on H4" would not be a
re-parameterisation of the existing structure, it would be a different structure.

Worth noting for its own sake: `|H4| = 14400`, which is exactly `120²`, the address space the ordered-
pair word already uses. That is a suggestive coincidence — the ordered pair indexes something of the
size of the full H4 group — and it is the kind of thing worth checking rather than assuming.

The Poincaré ball's legitimate home in a project like this is a **hierarchy** (a tree), not the spherical
Coxeter group. That reframing is what makes the rest of this document useful.

## 2. Can hyperbolic geometry save massive compute? Not through its arithmetic

Per-operation cost is **higher**, not lower: Poincaré-ball arithmetic is Möbius/gyrovector addition plus
`exp`/`log`/`artanh`/`arcosh`, i.e. a dozen flops and two or three transcendentals per operation, against
an add or a table read in the current serving path. Hyperbolic embeddings win on **representation
efficiency** — fewer dimensions for the same distortion on tree-like data, because volume grows as
`e^{cR}` — not on arithmetic cost.

**But the compute win the owner is reaching for is real, and it is structural rather than arithmetic.**
Hierarchical access is `O(log N)` where linear scan is `O(N)`. Hyperbolic space is the geometry *matched
to* a hierarchy; the log-depth win, however, comes from **traversing the hierarchy**, and a tree can be
traversed with integer comparisons and table reads — **no hyperbolic arithmetic at all**. So the honest
conclusion is the useful one:

> You can capture the logarithmic-access win without paying for hyperbolic arithmetic, by deepening the
> hierarchy the project already has.

The project already has a **two-level** hierarchy: six active H4 sectors with `MAX_PER_SECTOR = 8`,
i.e. ~48 candidates scored out of a 4,096 vocabulary — already a ~85× reduction over full-vocabulary
scoring, and it is built from integer comparisons and table reads. Deepening to a tree with fanout 8 and
depth 4 spans `8⁴ = 4096` leaves with **8 comparisons per level × 4 levels = 32 comparisons**, a ~128×
reduction over scoring the vocabulary, with no floats and no multipliers. That is the concrete
"massive compute" answer, and it is an extension of an existing, measured mechanism rather than a new
geometry.

## 3. "An operation on the entire manifold simultaneously" — real, but in the right model

Two standard facts, both true:

* **The hyperboloid (Lorentz) model** puts `Hⁿ` inside Minkowski space `R^{n,1}`, where hyperbolic
  **isometries become linear maps** preserving the Minkowski form. A whole-manifold isometry is then a
  single linear action — this is the precise sense in which "one operation acts on the entire manifold".
* **The Klein model** maps hyperbolic **geodesics to Euclidean straight lines**, which linearises
  geodesic transport.

Both are genuine. Neither is free: converting to and from these models costs transcendentals, so the
linearity buys mathematical convenience rather than serving-path savings.

## 4. The tangent operation and "nulls avoided"

The owner's idea — avoid phase nulls by operating in the tangent space — is the standard Lie-algebra
move: the tangent space at the identity is the Lie algebra `𝔤`, and the exponential map turns
composition into algebra arithmetic, `log(exp X · exp Y) = X + Y + ½[X,Y] + …`, i.e. **composition is
addition to first order**. Near the identity this is exactly "linearise the manifold so no point is a
null".

For **our state** this adds nothing: `2I` is a *finite* group, so composition is already an `O(1)`
table read — strictly cheaper than any tangent-space linearisation, and exact rather than first-order.
The tangent trick becomes relevant only if the state is made **continuous**, which is precisely the
offline substrate below.

So the idea is sound mathematics, correctly aimed, and already superseded for the serving path.

## 5. Reassessment under the owner's allowance

My previous answer was "no" flatly. That was right about the **serving** path and **too absolute as a
research decision**. Under the owner's allowance — a mechanism that is not the runtime bottleneck may be
carried as an **offline test substrate** — the correct answer changes to:

> **Yes, build the hyperbolic substrate offline, as a *decision instrument*, but do not expect it to be
> the compute win, and do not serve it.**

That fits the project's own separation: offline preparation and training may use floats, gradients and
matrix multiplication; only the served computation is constrained. The `artanh`/`arcosh` work the owner
already has is exactly the right raw material for that instrument. The falsifiable question it answers
is **not** "is hyperbolic faster" (it is not) but:

> Does a hyperbolic representation of the **hierarchy** achieve equal fidelity with materially fewer
> parameters or dimensions than the Euclidean/ternary path? If yes, the discrete tree metric is worth
> building for serving. If no, retire it with a measurement rather than an opinion.

That is a cheap, decisive experiment and it is the right way to spend the allowance.

## 6. Roadmap items, with falsifiable predictions

Ranked by leverage against the project goal (multiplier-free, float-free serving; bounded candidate
scoring; exact addressed memory).

| # | Item | Kind | Falsifiable prediction |
|---|---|---|---|
| 1 | **Deepen the hierarchy**: extend the existing 6-sector × 8-candidate routing to a fanout-8, depth-4 tree over the vocabulary | **serving** — integer compares + table reads | candidate scoring falls from ~48 to ~32 per token while held-out BPB is unchanged or better; the knee is where BPB degrades |
| 2 | **Offline hyperbolic substrate** using the existing `artanh`/`arcosh`: represent the address hierarchy in the Poincaré ball and compare parameters/dimensions needed for equal fidelity | **offline instrument only** | hyperbolic reaches equal fidelity with materially fewer dimensions; if not, retire |
| 3 | **Discrete curvature typing** (Ollivier–Ricci on the 2I Cayley graph) as a per-address property, computed exactly from the group table | serving-side, cheap | curvature predicts which buckets are ambiguous, sharpening the abstention threshold |
| 4 | **Tangent-space composition offline** — check whether Lie-algebra addition reproduces the exact 2I table to first order on the used subset | offline instrument | agreement degrades predictably with distance from the identity; if it holds on the used subset, it is a cheap way to train a continuous substrate |
| 5 | **Address-change ("velocity") as a routing feature** — exact, integer | serving-side, cheap | recently-moved routes are the ones worth re-reading |
| 6 | **Sup-norm growth as a standard gate** for every new mechanism (transfer from the Navier–Stokes formalisation review) | process | every mechanism gets a bounded-state measurement, not just a loss |
| 7 | **Machine-checked serving invariant** (extend the group-axiom habit beyond tests) | process, later | the integer kernel's semantics is certified, not merely zero-checked |

Explicitly **not** on the roadmap: Poincaré arithmetic as a serving path; R8 expansion *for accuracy*
(precision is measured as not the accuracy constraint); Hamming distance as a semantic similarity metric
(falsified in-project: the VSA ablation is inert, ΔBPB ≈ 0).

## 7. What this changes and what it does not

* **Does not change:** the serving contract. No floats, no multipliers, no dense contractions, no
  transcendentals. Nothing above proposes otherwise.
* **Does change:** the status of the hyperbolic work from "rejected" to "**offline decision instrument
  with a stated falsification**", and the framing of the compute question from *geometry* to
  **hierarchy depth** — which is where the logarithmic saving actually lives.
* **Corrects:** H4 is spherical, so the Poincaré ball belongs with a hierarchy, not with the spherical
  Coxeter group. The two should not be conflated even though both are called "geometric".

## 8. Honest limits

Items 1–6 are **unmeasured proposals**; only the existing two-level routing is measured. This document
performs no run and creates no artifact. The hierarchy-depth projection (fanout 8, depth 4 → 32
comparisons) is arithmetic, not a measurement — the real cost depends on memory layout and cache
behaviour, which only a run can settle. The hyperbolic substrate requires the owner's existing
`artanh`/`arcosh` work to be located; it is not in the current tree as inspected.
