---
id: m4-flock-coupling-prereg
title: Finish the flock-coupling pre-registration
milestone: M4
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The Flock read is the model's attention replacement: it keeps a few nearest earlier positions, rotates them into the query's frame and adds their consensus. No run has yet isolated what the read contributes by itself. The C1 sheet asks whether a learned coupling magnitude carries information that the three-valued gate cannot, and it was the read line that M4 originally selected. Background: [the Flock read](../../mechanisms/flock-read.md).

## What is missing

The M4 issue (18 October) names a draft C1 flock-coupling pre-registration, S1 flock transport mean plus S2 FBS tree (the Murmuration Read), gated on Rung 0 and owner hold release.

Is it still wanted? The mechanism board still lists the Flock read as ACTIVE (`docs/mechanisms/README.md:90`), and `docs/mechanisms/flock-read.md` names the C1 coupling sheet as its next step. So the 19 October renormalization kept it. Confirm that with the PM on the issue.

The draft (`docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md`) is not registered, and its status is `TEST FITNESS: PENDING`. It leaves open:

- whether `k_max > 1` is allowed under D11 (repeated addition);
- whether this is the starling line's decisive run or a new line;
- the exact writer losses for control X7, derived at FIT time;
- owner or PM confirmation that the float gap forward is the D37 item-3 instrument;
- the proxy-agreement threshold and the C0 sample size.

Its base is the Dose 1 decision checkpoint, but `docs/mechanisms/flock-read.md` records Dose 1 (18 October) and the A8 dose (19 October) as diagnostic stops, so no checkpoint is bound. No numeric bar is registered. The draft's rule is two seeds, the 14,287-row development panel, a sealed fresh draw for every keep, and bars no narrower than two-seed noise.

## Done when

A registered C1 flock-coupling sheet with a non-author FIT, or a recorded decision that it is superseded.

## How to start

1. **Settle whether C1 is still wanted.** Read `docs/mechanisms/flock-read.md` ("What comes next") and the board row at `docs/mechanisms/README.md:90`. Post the finding on the issue and ask the PM to confirm. If the PM says superseded, record that decision and stop. Note that `docs/integration/NOW.md` uses "Stage C1" for the BVG-K line, which is a different thing.
2. **Re-check the draft against current main.** Open `crates/uor-r4-discrete/src/discrete.cu` (`snap` at lines 102-110, `flock_vector` at 125-267), `crates/uor-r4-discrete/src/flock_qualification.rs` and `archive/labs/discrete-driver-cost-2026-10-13/README.md`. The draft pins line numbers and the roots at an older commit, so re-derive each literal in its controls (X1 to X12, F1 to F7) from the roots in `crates/uor-r4-discrete/build.rs`.
3. **First deliverable: a design note on the issue.** It must give the construction: a float coupling per block and level over the fixed integer Flock topology, exported as a small integer applied by repeated addition, with a new `FlockCoupled` read beside `Flock` and `k_max = 1` bitwise equal to `Flock`. It must show how the design avoids softmax and matmul: the proxy is a piecewise-linear hinge, the update is SGD with a box projection, and the learned part is 20 scalars multiplied elementwise. List the finite CPU controls with hand-derived literals, and the banned-token scan for the offline float module.
4. **Resolve the open items.** Put the five questions above to the PM or owner on the issue. Record each answer, and fix the base checkpoint, the proxy-agreement threshold and the C0 sample size.
5. **Complete the registration sheet.** Fill the Bar, Base artifact, Data split, Steps, Stop rule, Cost and Minimum test power blocks, which are `NOT_REGISTERED` in the draft. Keep the arms A0, A1, A2, A2-int, A2-prime, Z and R, with zero and random coupling controls, at least two seeds, and the export-gap report. Build the harness with `CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo build -p uor-r4-harness`, then run `uor-harness check`.
6. **Ask a non-author for the TEST FITNESS: FIT review.** A FIT clears the sheet only; it releases no compute, and the owner holds stay in force.
7. **Open a PR** that says `Closes #<this issue>` (or `References` if partial). Record the registration, or the supersession decision, as one row in `docs/research/notebook/m1-discrete-attention.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:38`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:90`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:77`](../../../docs/model/architecture.md)
- issue#2032

