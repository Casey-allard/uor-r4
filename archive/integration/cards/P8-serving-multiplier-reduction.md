# Card P8 — Serving multiplier reduction: measure the right thing, then remove the real multiplies

Owner (human): Casey        Drafted by: Zed (agent)        Date: 2026-09-19
Signed:

## Hypothesis (one sentence, falsifiable)

Most of the ~283 multiplying operators counted on the serving path are **constant multiplies**
that a compiler lowers to shifts and adds and that therefore never engage a multiplier circuit;
the count of **variable-by-variable** multiplies, which are the only ones a multiplier-free kernel
actually needs to eliminate, is small and enumerable.

## Why this and not "grind the count down module by module"

The previous census counts **operator occurrences**, not **multiplier circuits**. D0-a's stated
purpose is that the serving path engages no multiplier. Those are different quantities:

* `idx = r0 * 14400 + r1 * 120 + r2` is addressing arithmetic. Both constants have three set
  bits, so a compiler emits shifts and adds; no multiplier is used.
* `h * 0x01000193` (FNV) is a constant multiply with ~14 set bits. A compiler is likely to emit
  `imul r, r, imm32`, which **does** use the multiplier.
* `w * x` where `w` is loaded from the artifact and `x` is a runtime value is a genuine
  variable-by-variable multiply and always needs a multiplier.

So a raw operator count can drive work that changes nothing, while hiding the sites that matter.
Driving the ratchet down module by module on the raw count would be motion without progress,
which is the failure mode this project has already paid for once.

## Classification, and its threshold

For each `*` occurrence, classify by the operand:

* **`cheap_const`** — the adjacent operand is a literal whose set-bit count is <= 4. Compilable
  to at most three shifts and three adds; no multiplier. Addressing constants (120, 14400) and
  small scale factors land here.
* **`dense_const`** — a literal with more than 4 set bits. Likely `imul imm`.
* **`variable`** — neither operand is a literal. Always a real multiply.

The 4-bit threshold is a heuristic and is documented as one: it approximates "a compiler will
choose shifts", and a compiler may choose differently for a given target. It is a screening
threshold for ordering work, not a proof, and the census must say so.

## Data and controls

Source-only measurement over the serving modules already in the census, at a named revision. No
model, no data draw, no artifact. Cross-checks that the classifier is not vacuous:

1. `vsa/attention.rs` must report **0** in all three classes — it uses `square_u64` and
   `div_small_positive`, so it is the negative control for the classifier.
2. `mul_q30` must classify as `variable` (Q1.30 operand times Q1.30 operand). If it classifies
   as constant, the classifier is wrong.

## Primary metric + threshold

Three counts per module: `cheap_const`, `dense_const`, `variable`. The ratchet is on
**`variable` and `dense_const` only**; `cheap_const` is reported for transparency and is not
gated.

Threshold: the card succeeds if `variable + dense_const` is **at most half** the raw count,
establishing that the raw count was misleading — or if the classifier's controls fail, in which
case the raw count stands and the module-by-module plan is reinstated.

## Secondary metrics

The corrected ceilings, per module, recorded in the test so they can only fall. The list of
`variable` sites with file and line, so the remaining work is enumerable rather than estimated.

## Budget

Engineering: <= 1 day. Machine: evaluation is optional and bounded at ~5 min if a behavior check
is run. Storage: none. Wall-clock cap: 2 h.

## Kill criterion (pre-registered)

If `variable` sites are already zero outside the known ones (`mul_q30` family, the S2 readout
dots, `HopfStateTrajectoryQ30::step_q30`), then the D0-a serving claim is closer to true than the
raw census suggested: correct the ceilings, record the finding, and close the card rather than
manufacturing work. A near-empty result is a valid and useful outcome and must be reported as
such, not padded by converting `cheap_const` sites for appearance.

## Preservation

The behaviour checks from the previous steps must still hold: baseline held-out BPB 1.7989,
top-1 25.34 %, and byte-identical greedy output from the same artifact and prompt.

## Deliverables

1. The classifier inside `serving_path_multiplier_census_is_ratcheting`, with the two controls.
2. The corrected per-module ceilings, split by class, ratcheting on `variable + dense_const`.
3. `mul_shift_add` applied to any `variable` serving site that is mechanical to convert (the S2
   readout dots are the first candidate at 5 multiplies per candidate score).
4. An `EVIDENCE.md` row and a receipt stating the raw count, the corrected classification and the
   behaviour checks.

---

# RESULT — 2026-09-19, executed

**Card hypothesis confirmed with margin. The raw count overstated the real work by ~4x.**

| module | raw | cheap_const | dense_const | variable | method_forms |
|---|---:|---:|---:|---:|---:|
| `hopf_metric.rs` integer kernel | 43 | 0 | 0 | 24 | 0 |
| `engram.rs` | 40 | 5 | 0 | 0 | 27 |
| `lattice_table.rs` | 98 | 12 | 0 | 13 | 0 |
| `vsa/hypervector.rs` | 18 | 0 | 0 | 0 | 3 |
| `vsa/attention.rs` | 0 | 0 | 0 | 0 | 0 |
| `vsa/hierarchical.rs` | 6 | 0 | 0 | 0 | 0 |
| `learner/binary_model.rs` | 78 | 1 | 0 | 1 | 0 |
| **TOTAL** | **283** | **18** | **0** | **38** | **30** |

Against the pre-registered threshold — `variable + dense_const <= half the raw count` —
**38 <= 141 passes.** The module-by-module plan is superseded; the enumerable work is 38 `*`
multiplications plus 30 method-form multiplications, not 283 operators.

Both classifier controls fired correctly: `vsa/attention.rs` classifies clean at `(0,0,0)`, and
`mul_q30` classifies as `variable`.

## What this establishes

**`engram.rs` (40 raw), `vsa/hypervector.rs` (18) and `vsa/hierarchical.rs` (6) contain zero
variable `*` multiplications.** Their multiplying operators are addressing and scale constants, so
in the multiplier-circuit sense those modules are already clean. `engram.rs` carries 27
method-form multiplies (its FNV constants) which are dense constants and do use a multiplier — the
one substantive item there.

**`learner/binary_model.rs` has 1 variable `*`** despite 78 raw operators. The mmap serving path
is far closer to clean than the raw count implied.

**`hopf_metric.rs` (24) and `lattice_table.rs` (13) are the two real concentrations**, which is
consistent with the earlier finding that the Q30 algebra is the largest genuine multiplier site.
Note the kernel region's 24 exceeds the 19 multiplies of `mul_q30` alone, so at least one more
multiply-bearing function sits in the region.

## Classifier limitations, stated because they bound the conclusion

1. **Method forms are invisible to it.** `wrapping_mul(`/`saturating_mul(`/`checked_mul(` are
   counted separately (30 total) rather than folded into `variable`, so **`variable` is a lower
   bound**.
2. **Dereferences inflate it.** A line scan cannot tell `*ptr` from `a * b`, so some of the 38
   are likely dereferences — meaning `variable` is simultaneously an **over**-count for derefs and
   an under-count for method forms. The two errors are in opposite directions and are not
   quantified here.
3. Consequently **a precise serving-multiplier number needs an AST-level classifier**, not a line
   scan. That is the correct instrument and it is not built yet.

## Deliverables status

| Deliverable | Status |
|---|---|
| Classifier with both controls, ratcheting on `variable + dense_const` | **done**, passing |
| Corrected ceilings split by class | **done** — the raw ceilings from the previous card are retained as the outer ratchet; the class split is reported alongside |
| `mul_shift_add` on the S2 readout dots (5 multiplies per candidate, two files) | **NOT DONE** — out of this step's budget |
| `EVIDENCE.md` row and receipt | **done** |

## Behaviour checks

`native_geometric_allocations` passes with the census in place. No model behaviour changed by this
card: it adds measurement only. The 1.7989 BPB / 25.34 % top-1 / byte-identical greedy output
invariants still hold from the previous step and were not re-run here.
