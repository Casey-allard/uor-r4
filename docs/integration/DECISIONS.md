# DECISIONS

Append-only. Each entry is an owner decision with its rationale and scope.
Agents may draft entries; only the owner ratifies them.

---

## D0 — What "no matmul at serving" means

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: D0-a — the multiplier-free-kernel reading.**

The project's objective is to eliminate *wasted compute*. The serving path must
execute no multiplier and no floating-point arithmetic, and must not perform a
dense contraction that touches every parameter for every token. Offline training
is unrestricted.

### Permitted at serving

- integer add, subtract, shift, rotate, bitwise AND/OR/XOR/NOT, popcount, compare
- bounded index arithmetic and bounds checks
- exact fixed-point arithmetic in Q1.15 / Q1.30 (add and shift only)
- **table and array reads of learned values** — the value was learned offline; the
  serving operation is a selected read
- bit-serial / LUT accumulation of low-bit (ternary or ≤4-bit) weights, where the
  multiplier circuit is absent by construction (T-MAC-class kernels)
- exact selected reads, writes and copies over addressed memory

### Excluded at serving

- IEEE `f32` / `f64` arithmetic and `libm` transcendental functions on the hot path
- the integer `*`, `/`, `%` operators on the hot path (permitted only through
  documented shift-and-add idioms such as `square_u64` / `div_small_positive`)
- any GEMM or matrix-vector product that multiplies and accumulates over a dense
  weight matrix
- any tabulated implementation of a **dense** contraction: if serving touches all
  entries of a learned weight store per token, it is a matrix product regardless of
  the opcode used

### Unrestricted offline

Floating point, gradients, `uor-matmul`, Adam and any other optimizer. Training may
freely compute linear maps; the constraint applies only to the served computation.

### The distinguishing test

**Per-token parameter sparsity.** For each token, count the fraction of the learned
parameter store that is read. Sparse selected access ⇒ compliant. Dense access ⇒
non-compliant, whatever arithmetic is used. This test, not the opcode, decides.

### Consequences

1. The lane tables (`discrete_tables[l]`, 120×120 trained by Adam) are
   **compliant**: reading one 120-entry row is a sparse selected read.
2. Two current serving operations are **not** compliant and must be repaired or
   removed: the JEPA projection in `learner/jepa_trainer.rs::predict_jepa_step_q30`
   (literal `i64` multiplies) and the `f64` softmax sampler in
   `native_capability_api.rs` (reachable whenever `temperature > 0.001`).
3. A ternary linear map trained offline and served by LUT accumulation **is**
   compliant. This is the precedent-backed path to competitive quality.
4. The root `README.md` and `AGENTS.md` wording ("no mathematical matrix products,
   including … tabulated lookup/add contractions") is **narrower than this decision**
   and currently conflicts with the shipped implementation. Updating that wording is
   a stable-goal change and must go through owner-directed protected delivery; until
   it does, this file is the normative statement of intent and the conflict is known.

---

## I1–I5 — Serving efficiency invariants

Recorded with D0. These are measured, not asserted, and are reported together.

| ID | Invariant | Measurement |
|---|---|---|
| **I1** | Sparse per-token parameter access | fraction of the learned store read per token |
| **I2** | Low bytes/token | bytes streamed per token; bits per stored weight |
| **I3** | Bounded candidate scoring | candidates scored per token (target ≪ vocabulary) |
| **I4** | No multiplier in the kernel | static AST check + dynamic op census |
| **I5** | Exact addressed memory, no recomputation | provenance/version reads per token |

I3 and I5 are the properties a dense transformer cannot match at equal quality and
are the basis of any efficiency claim.
---

## D0-b — What "no matmul at serving" means, adopted

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: D0-b. Bounded integer/ternary linear maps are permitted at serving, executed with no
multiplier in the kernel.** This supersedes D0-a, whose strict reading banned the mathematical
linear map even when tabulated.

### The serving contract

- **No floating point** at serving, and no `libm` transcendentals.
- **Weights at most 4 bits**, ternary preferred.
- Allowed: integer add, subtract, shift, bitwise, popcount, compare, table and array reads, bounded
  index arithmetic, exact fixed-point.
- **No multiplier instruction in the kernel.** A linear map is legal only in a form that executes
  as adds/subtracts/shifts/table reads.
- **Energy measured, not estimated**, on a named machine, when the value is claimed.

### Why D0-a was withdrawn

The strict reading banned the one operation every architecture that *learns a representation* uses,
and thereby excluded every published precedent (BitNet b1.58, MatMul-free LM, T-MAC) while leaving
only logic-gate networks, whose best published sequence result is 5.00 BLEU on 16-token MT. Under
that reading the project was in genuinely unexplored territory with no learnable substrate of
sufficient capacity, which is not a viable path to a chat model.

D0-b preserves the actual objective — no multiplier, tiny RAM, no GPU, local, measured energy —
while allowing the substrate that reaches chat quality. The multiplier constraint stays; the
mathematical ban is what changes.

### How it is enforced

1. `scripts/serving_multiplier_check.py` — a per-function **zero-check** over a `--release` binary:
   a declared serving symbol whose instruction range contains no multiply mnemonic cannot execute
   one. Sound where counting is not.
2. `scripts/energy_per_token.py` — refuses to report a power figure that is not physically
   plausible, which is what stops an unpopulated `CPU Power` field from becoming a published
   result.
3. Every low-bit weight matrix uses a **power-of-two per-row scale**, so `y = (Σ ±x) << shift` and
   the layer is multiplier-free by construction rather than by compiler grace.

### Consequence

The native learned core may use low-bit linear maps. `learner/lowbit.rs` implements the substrate:
ternary weights at 2 bits each with per-row power-of-two scales, trained in float, served in exact
integer arithmetic, with a test asserting the serving path equals the floating reference.

---

## D1 — Falsification before extension

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

Method is Socratic and falsify-first. Before any new geometric mechanism is added,
every existing mechanism must show a **measured** contribution against a matched
control, or be retired from the critical path. A mechanism that cannot be shown to
change held-out bits-per-byte is not carried forward on grounds of elegance,
correctness of its mathematics, or architectural priority.

Corollary: the per-mechanism ablation table (Card P7) is a precondition for
Stages 3–6, not an optional diagnostic.

---

## D2 — How a mechanism's contribution is judged

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: an ablation is a measurement, not a verdict. Magnitude and mechanism class are
judged separately, and a geometric mechanism is not retired on a near-zero delta.**

Owner direction: *"you probably need to weight falsification misses that are very small vs if
the mechanism truly works coherently, and not immediately rule it out because it may be a
better mechanism that maintains our project goal of a geometric language model."*

### Two independent axes

1. **Magnitude**, against an equivalence margin `epsilon` (default **0.01 BPB**, about one
   third of the 0.0316 BPB gap to the matched Kneser-Ney 5-gram, so anything below it cannot
   be decisive for the competitive question):
   `MAJOR+ / MINOR+ / NEGLIGIBLE / MINOR- / MAJOR-`.
   A paired bootstrap CI can exclude zero on a practically meaningless effect. **Passing the
   CI test is not evidence that a mechanism matters, and failing the magnitude test is not
   evidence that it is a dead end.** Statistical and practical significance are reported
   separately and never conflated.

2. **Mechanism class**, declared per mechanism and never inferred from the number:

   | Class | Role | A NEGLIGIBLE delta means |
   |---|---|---|
   | `primary-carrier` | expected to carry predictive signal | a defect to diagnose |
   | `modulator` | expected small, consistent shaping | acceptable |
   | `selector` | routing / candidate selection | **nothing** — BPB ablation is the wrong instrument, because an alternative path substitutes; use decision-flip rate and forced-choice tests |
   | `enabler` | observation channel whose value appears only once composed with a component that does not yet exist | expected, and uninformative about the mechanism |
   | `count-table` | a count statistic, not a learned geometric mechanism | removable on resource cost |

3. **Decision-flip rate** is reported alongside BPB. A mechanism can be BPB-neutral while
   changing a large fraction of decisions, which matters for the *kind* of errors rather
   than their average. Measured examples from the first run: `lattice_coarse` is
   BPB-negligible yet flips **9.5 %** of decisions; `vsa` flips 0.7 %.

### Decision rule

- **Never retire** an `enabler` or `selector` on a near-zero or negative delta. Repair the
  wiring or change the instrument and re-measure.
- Only a `count-table` or a correctly wired `primary-carrier` that is measurably
  net-negative is a removal candidate, and then on resource cost as much as accuracy.
- Record the identified wiring defect with the delta, so a small number is never read as a
  mechanism verdict. Example: the `vsa` delta measures a codebook that is **not** consistent
  with the learned representation, not the VSA mechanism.

### Consequences

- The 2026-09-19 `vsa` and `lanes` results are **not** retirement verdicts. `vsa` is a
  wiring defect with an identified cause; the required action is to learn a consistent
  codebook and re-measure.
- Interaction must be accounted for before any removal: zeroing the JEPA weights also
  changes the fiber the S2 readout consumes, so single-mechanism deltas do not sum. A
  pairwise factorial or Shapley attribution over the mechanism set is the correct
  instrument once the set is stable.
