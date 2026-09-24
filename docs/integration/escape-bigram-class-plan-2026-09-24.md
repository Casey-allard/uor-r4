# Milestone plan — "escape from bigram class" diagnostic gate (M1)

Owner-adopted plan (2026-09-24) for the ordered next work in
[direction-decision-2026-09-24.md](direction-decision-2026-09-24.md) and [project-track.md](project-track.md). It is a
**diagnostic gate that selects the next code change**, not a capability claim. Pre-registration: the arms, panels,
statistics and accept/reject rules below are frozen before execution; any change is recorded with its reason, and a
failed or ambiguous outcome is preserved at its exact scope.

## 0. What decision this makes

The programme has converged its process but not its model: a geometry-free 64-dimensional integer RNN, ~6.27–6.68
bits/target on its own docs, no cross-window state, no input-dependent gate, an effective per-position readout rank
≤ 64, and a soft memory horizon (~75 % at lag 2, ~19 % at lag 3). The open question is **which lever is binding**:
the **readout/features**, the **state/horizon**, or the **objective/data**. M1 answers that with two bounded stages,
in order of cost.

## 1. Preconditions (no compute)

- Record a complete prospective resource projection (context windows, wall time, threads, peak RAM, new/retained
  storage, stop margin) and the ledger increment **before** any run. The shared ledger is ≈ 99 % consumed
  (`452,039,320 / 455,500,000 ms` at `f95b9efb`); any necessary local extension is recorded under the standing
  authorization, not applied silently.
- Re-read the live ledger, free space and the current best artifact identity (`learner/transferable_lexical.rs` and
  its `.tlx`) at run time; do not trust this document's numbers.
- Pre-declare the two panels and their hashes; hold out the acceptance population from the design population.
- Confirm the D0-b boundary for every new serving operation: a **hard compare/select gate** is admissible; a
  **data-dependent multiply** is not (constant-multiplier shift-add chains only cover constants).

## 2. Part A — frozen-state adjudication (cheap, no retraining)

**Purpose.** Bound, on the already-recorded served states, how much of the local deficit is readout/feature-limited
versus state-limited, with a control that cannot be mistaken for language gain.

**Population.** The exact state vectors `h` the served readout receives, recorded by the existing `--state-probe`
path over the 5,376 development targets / 24 documents. **Document-held-out**: fit the readout on fit documents,
evaluate on held-out documents; report the rank-K **curve**, not a single K.

**Arms** (all on the same frozen states, float offline; none served):

| Arm | Description |
|---|---|
| (i) linear float readout | baseline `B0` (the known converged float linear readout) |
| (ii) rank-K bilinear / quadratic | product features of the state, K ∈ pre-declared {2,4,8,16,32,64}, K selected on held-out |
| (iii) 2-layer MLP | hidden width d ∈ {64,128} |
| (iv) transport-decode depth ladder | cur-peeled previous-token decode at lags 1–4, **both** the full 5,376 population and the restricted sub-population |
| **(v) count-augmented linear** | linear readout **plus** the tuned `(prev,cur)` count features — **the decision arm** |

**Statistics.** Paired per-document bits/target differences with a document-cluster bootstrap interval; `prev`
recovery accuracy with a binomial interval; all softmax bases asserted (the base-2 dyadic convention, not base-e).

**Pre-declared decision rule.**

- **If (v) − (i) ≈ 0** (the state adds nothing over the count table on held-out documents): the current state's local
  information is subsumed by a servable count table → **no readout work is justified**; the lever is the
  **state/horizon** (Part B, gate).
- **If (iii) − (i) ≥ 0.35 bits and (v) explains it**: the gain is reconstruction of the count table from the state, a
  *feature-form* result, not a language gain → **do not build a served readout as proxy polish**; proceed to Part B.
- **If (iv) full-population lag-2 recovery ≥ ~60 %**: the state retains second-order information and the blocker is
  the readout's linearity → Part B's gate is *not* the only lever; a bounded nonlinear readout is co-justified.
- Middle outcomes are reported as the rank-K curve, not forced into a binary.

**Confounds to control and report.** A bilinear readout can approximate the count table from `prev` alone, so a
large (iii)−(i) is not language; held-out document selection, not a reject-repair; the frozen state was trained to
feed a *linear* readout, so the test bounds the *current* state's headroom, not the architecture's; the restricted
sub-population is reported separately from the full one.

## 3. Part B — one minimal gate × capacity arm on an induction probe (one training task)

**Purpose.** Test whether the recurrence class can carry a dependency **longer than two tokens at all** — the
precondition for any language capability, and the cheapest experiment whose every outcome routes the roadmap.

**Task (frozen, synthetic).** Associative recall / induction with `I(x_t ; x_{t−1}, x_{t−2}) = 0` by construction:
i.i.d. content tokens over `V = 64`, repeated triggers `ξ`, target = the successor of `ξ`'s previous occurrence.
**64 episodes × 8 queries = 512 held-out query positions**, ≥ 3 repetitions per trigger, seed-fresh sequences,
episodes held out. The tuned order-2 count prior must be **verified at chance** (accuracy ≈ `1/V`) on the panel, and
the panel's learnable-instance count is recorded before training.

**Arms** (identical data, seed(s), step count and, where possible, parameter count):

| Arm | Description |
|---|---|
| (a) current | `h' = clamp(e + (W_h h >> 3) + …)`, 64-dim, no gate |
| (b) gate | (a) + a **hard compare/select gate** (`h' = g_t ? h : update`), D0-b-clean |
| (c) width | 256-dim, no gate |
| (d) width + gate | 256-dim + gate |
| (e) ordinary control | an equal-parameter **non-geometric** gated RNN (or a finite-state table) — for geometric attribution |

Controls: the order-2 count prior (verified at chance), an equal-work stateful n-gram, and **two seeds**.

**Statistics.** Held-out per-query accuracy and NLL; paired differences (b)−C, (d)−C and (e)−(b) with bootstrap
intervals; C must be at chance.

**Pre-declared accept / reject.**

- **Accept:** some arm beats chance (95 % CI excluding `1/V`) **and** beats the count control by ≥ 2 bits on
  held-out episodes → the recurrence *class* can represent the dependency; capacity/context was the binding lever;
  the winner becomes the substrate on which geometric gate/transport parameterisations compete at matched capacity
  (Goal R is finally testable cleanly). Proceed to M2.
- **Reject:** all arms at/near chance → the recurrence class is terminal at this scale; **escalate to the owner**:
  hybrid integer attention (an explicit "no transformer" goal change) or reposition as a memory-augmented
  retrieval front-end.
- **Diagnostic side-outcome:** if the no-gate arm (a) unexpectedly beats chance, the "no-gate short-horizon"
  diagnosis is wrong and the direction decision's §3 ordering must be re-derived before further compute.

**Geometry boundary.** No geometric mechanism is promoted. If a geometric gate/transport parameterisation is
tested, it must beat its **matched ordinary** counterpart (arm (e)) at equal parameters and compute; a tie is the
expected outcome and is reported as such.

## 4. Sequencing, budget and deliverables

- Run **Part A first** (no training; bounded by existing states and the runner). Its outcome selects whether Part B
  prioritises the gate, the width, or a bounded nonlinear readout.
- Part B is a small synthetic task; each arm is short. Record a complete projection first; if the two stages exceed
  the remaining ~3.46M ms, record an owner-authorized extension prospectively. Do not silently raise limits.
- **Deliverables:** a sealed evidence root per attempt (claim/seal/verify); the pre-registered panels and their
  hashes; a result record at the exact scope; updates to `current-state.md`, the owning issue and this plan; and a
  protected PR.

## 5. What M1 does not claim

No capability, language, reasoning, coding, energy or geometric-superiority claim. A Part-A readout win is a
feature-form measurement, not a language result. A Part-B induction pass establishes only that the class *can*
represent a longer dependency on a synthetic panel — not general prose. No served-model arithmetic changes except a
D0-b-clean gate, and no default is replaced.

## 6. Pre-registration checklist (freeze before the first run)

- [ ] Part-A panel hash and document split frozen; arms (i)–(v) and the rank-K grid declared.
- [ ] Part-B induction panel frozen; `C` verified at chance; episode split frozen; two seeds declared.
- [ ] Accept/reject rules above copied into the attempt metadata before execution.
- [ ] Ledger projection and extension (if any) recorded prospectively.
- [ ] Artifact/source/tokenizer identities bound; `transferable_lexical.rs` change (a gate) is a *new* artifact, not
      a mutation of the parent.
