# #1296 — Prior-learning recovery Stage 1-2: one correct numerical contract; the cheap contextual gate fails (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T01:46:24Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-20T01:47:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1296
- **Merged:** 2026-09-20T01:47:26Z
- **Merge commit:** f4fc1c6089f4b047ec8feb92ba184c2f81bb077d
- **Base / Head:** main / UOR-Foundation:codex/prior-recovery-r2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Stage 1 of the recovery prompt is implemented and tested. **The Stage 2 cheap balanced-contextual gate fails at this configuration, so the Stage 3 real-text curve was NOT RUN** — the prompt's own stop condition ("Do not spend a full-corpus dose while this cheap gate fails"). No real-text model was trained. The deliverable is a correct instrument plus a measured gate failure with a bounded diagnosis.

## The unit error is fixed

#1294 reported clipped **nats** labelled bits (`-ln(max(p,1e-9))`, capped at 29.8974 bits). Replaced by `z = Z·2^-F` with one artifact-bound exponent `F` and `bits = (logsumexp(z) − z[target])/ln 2`, uncapped and unfloored. Tested: uniform V=4096 is **exactly 12.000000 bits**; a common offset is invariant; a confidently wrong prediction is finite and **above** the old ceiling.

## One target iterator

`prior_learning::targets` is shared by training, integer evaluation, coverage and references: position `i` over `0..n-1`, context ends at `tokens[i]`, target `tokens[i+1]`, absent-prefix row at `i == 0`. A length-64 window scores **63** targets, not 62; tested on first/last targets.

## Train and serve are the same computation

One integer forward `Z[r] = (Σ_j W[r][j]·h[j]) + B[r]` with a **frozen fit-only ≤4-bit bias** (`bias_code = round(ln p_fit + 7)`, clamped to [−7,7], declared dyadic scale). Measured **fit-unigram quantization gap 0.10 bits** (3.7823 vs 3.6809). The old train/export bias mismatch is structurally impossible now: with integer codes a master bias of `(0.49,−0.49,0)` cannot be visible — asserted as a fixture. `2^-F` is applied **once**, at credit to `Z`; greedy serving is `argmax(Z)`. Step-0 exported residual is **exactly zero** (asymmetric below-threshold output masters, so the model starts *as* the constant control), the output gradient is live, and prior credit appears only after the codes cross the threshold.

## Bounded loading and a real checkpoint

`CPL2` v2 binds vocab/dv/norm bits/`F`/bias scale/seed/tokenizer digest and validates shapes, checked size arithmetic, shift range and the bias code range — truncation, a trailing byte, bad magic and an out-of-range code are all rejected. `checkpoint_bytes`/`resume_from` carry masters, both Adam moments, step, optimizer config, schedule cursor/pass/rng and a data identity: **a resumed split run equals the uninterrupted run exactly**. Legacy `CPR1` stays readable under its original semantics and is not relabelled compliant.

## The gate, measured

Balanced fixture `(x,y) → (x+y) mod 4` over all 16 ordered pairs, vocab 4, dv 32, 2,000 Adam steps, prior-only with the frozen bias:

| | |
|---|---:|
| exported (integer) accuracy | **0.375** |
| constant-marginal accuracy | 0.250 (the task is genuinely contextual) |
| frozen success bar | 0.900 |
| | **GATE FAILS** |

**Diagnosis (bounded, not a proof of cause).** The model is expressible for this task, so the failure sits in the **optimization or quantization of the learning path**, not capacity: the prior tables receive no credit at all until the output codes cross the quantizer threshold (training a readout against frozen random ternary features first), and the bounded ReLU masks half the feature entries to zero. The prompt's prescribed next step for this branch is one bounded **offline floating-forward comparator**; it is not run here and must never become a served path.

## Not run — stated as omissions

Stage 3 (real-text manifest, references, learning curve, real-text checkpoints), the gates, permutation control and position knockouts, any generation, and the `report_output::claim`/`seal`/`verify` preservation path — all because the gate failed before them. Also not run: the **binary/whole-path multiplier audit** of the new integer path (source inspection only), a `crates` replicate, a second seed, and any learning-rate/width sweep. The retained `cold_prior_joint.cpr` was **not** rescored: its training bias was the unquantized float master while export rounded to i32, so a replay would be a new-population, new-convention diagnostic rather than a faithful continuation.

## Validation

`cargo fmt --all --check` clean · `python3 scripts/check_claim_wording.py` passed · `cargo test --release -p uor-r4-core --lib -- prior_learning` → **9 passed, 0 failed**.

No chat, reasoning, coding or energy claim follows. Ledger: 800,000 ms charged (measured) → `149,838,565 / 154,400,000`, ~76.0 min remaining; 128 MiB storage margin untouched; no deletion, no paid compute.

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

