# REPORT: m4-lawful-likelihood-bpb, deepseek-8 (design unit)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `a8ff5690e`. References #2861 (not Closes: the item stays ready; this unit lands the design note only).

## What did not work, first

- **The item's "sources name no design" is true only in part.** The sources do not name a design, but they do constrain it: the historical integer read already normalizes through a Q31 exp table (`geometric_read.rs`), which is exactly the banned normalized-exponential shape; the Q48 sampling contract requires masses summing to `1<<48` exactly; and the current model emits no probabilities at all. So the first design constraint is not a blank page — it is "do not repeat the exp table, and land inside the Q48 contract's exactness".
- **The obvious construction is also the trap.** Re-using the historical exp-table read would score BPB today, but it is the banned form renamed; a design that only wraps it in a new name would fail review and the charter. The note states the distinction explicitly instead of leaning on it.
- **A shifted-linear normalizer is not calibrated.** The potential-ratio construction is lawful and order-preserving, but it is not a temperature-calibrated likelihood; whether the 0.90 BPB bar may be read through an uncalibrated lawful likelihood is a PM/owner question, so the note raises it rather than deciding it.
- **The held-out stream is not named anywhere.** The item itself says the sources do not name it; the pre-registration must name the file, byte count and hash. That is deliberately left out of this unit.
- **No code, tests or model numbers are produced.** That is the item's order (design note before code) and this unit's boundary, not an omission hidden in the REPORT.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +66 | the design note: problem, existing Q24 potentials and Q48 contract, the potential-ratio construction, the six lawfulness properties, the six finite host controls, the five open PM/FIT choices, and the unit boundary |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-8/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** about 90 changed lines, against the ORDER's 90-line estimate (1.0). No source, test, generated file or item-status change.

## Measurements

- **No numeric result** is produced or claimed; this is a design unit.
- The design rests on named constraints: the Q24 score scale and its `MAX_ABS_SCORE` bound (`geometric_potential.rs`), the `1<<48` mass and its exact-sum requirement (`sampling.rs`), and `EXP_TABLE_LEN = 8194` in the historical read (the banned path).
- The next unit's finite controls are enumerated in the note (tiny-alphabet enumeration, uniform case, monotonicity/argmax property test, permutation equivariance, BPB arithmetic against an analytic value, and a count-table baseline through the same scorer).

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on the five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.
- `cargo fmt -p uor-r4-harness --check`, the full test suite and clippy: no source changed, so they are unchanged from the base.

## Skills used

- **solution-integrity**, read from its private local install: check the premise against the sources, name the falsifier, and keep a design unit's claims inside what it actually did.

## Still open

- **The implementation unit**: the chosen family's Rust implementation and the six finite host controls; it can start once the PM/FIT approves a family, or with the simplest family as a declared arm.
- **The registration unit**: name and hash the held-out chat stream, write the one-page pre-registration, get a non-author FIT review, and then run the registered panel-scoring on the GPU under an owner release.
- **The five open choices** in the note: normalization family, calibration, the held-out stream, division in the scoring path, and the crate boundary.
- **The item stays `ready`** after this PR merges; `loop next` may offer it again, and the next unit continues from this note.

## Plain summary

- **Why this exists:** the model gives scores, not probabilities, so the M4 bits-per-byte bar cannot be scored. The historical integer path turned scores into probabilities with an exp table, which the charter bans, so a lawful replacement is needed before BPB returns.
- **The proposed law:** keep the model's existing integer scores on its H4 geometry, shift them so they are all non-negative (an order-preserving identity-or-shift, no curve and no exponential map), divide each by their total to get exact whole-number probabilities out of `2^48`, and compute bits from those probabilities.
- **Why it is lawful:** every probability is a non-negative integer, they sum to exactly `2^48`, the highest score keeps the highest probability, shuffling the candidates only shuffles the probabilities, and no banned function or dense matmul appears. The note checks all of this, and the finite host controls that will test it, before any GPU work.
- **What this unit did not do:** it wrote no code and ran no model; it wrote the design the item asks for *before* code, posted it on the issue, and left the implementation and the registered GPU scoring to their own units.