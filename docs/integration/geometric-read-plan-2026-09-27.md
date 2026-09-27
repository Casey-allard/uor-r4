# Finite geometric read kernel: predeclared plan

September 27, 2026. References #973 under #820. OpenCode/DeepSeek lab.
**Status: predeclared design; no compute executed by this document.** The
implementation and its first matched screen follow this plan, which fixes the
arms, conditions, endpoints, decision rule, guardrails and budget before any fit.

Authority: the owner's native multi-lab direction, `AGENTS.md`, the progress-control
rules, and the fourth lab's reviewed design
[`fourth-lab-geometric-attention-2026-09-26.md`](fourth-lab-geometric-attention-2026-09-26.md)
(“First operator: learned signed relative-group score, full admission”). This plan
implements that designed operator; it does not re-derive or extend it.

## Objective

Add an optional, absent-by-default finite geometric read kernel to the mainline
joint learner — a learned nonlinear score over the signed relative element of the
120-element binary icosahedral group `2I` (the canonical H4 roots) — and run its
first matched screen against a same-binary baseline continuation from the retained
quaternion parent.

## Why this mechanism, now

- The read score is the only mechanism change tested at the retained dose so far.
  The radial alternatives (Lorentz, LorentzAffine) were **worse than Dot** on both
  likelihood and source answers (full development NLL 1.999367 / 2.045766 / 2.012670;
  complete source answers 21/32 / 14/32 / 19/32) and are parked at that fixed dose.
- The signed relative-group score is fully designed and reviewed but **never
  implemented or trained** anywhere in the repository (no `geometric_read` source;
  the `geometric-attention-research`/`synthesis` branches measured nothing).
- Prior learned relative-group readers exist **outside this model** and were matched
  or beaten by competent ordinary controls, with named failure modes the new screen
  must not repeat: `relational_attention` (Sept 13) `DiagonalOnly` and
  `PhasesDisabled` controls also pass, so unique benefit was unproven; `relational.rs`
  (Sept 20–21) had categorical+contextual loss *lower* than H4+contextual and harmed
  natural text; the KVAR Q8 relative-energy residual (Sept 24) was rejected because an
  exact key-addressed store makes the queried key equal the stored key, so the
  relative action is always identity; A4 (Sept 24) found 2I versus C120 both at 0/12
  complete answers. Historical negatives retain their exact scope.

## Hypothesis and single-variable intervention

**H:** at the same parent, dose, seed and optimizer protocol, replacing the dense
Q/K dot score with a learned nonlinear score of the finite relative element
(16 unit-coded lanes composed through exact `2I`) changes complete generated
behaviour and/or likelihood beyond the same-binary baseline continuation.

**Single variable:** the read kernel (enabled/disabled), with the trained parameters
that the enabled kernel adds. Everything else — events, value/copy path, NoRead,
masks, objective, age, interface, softmax, full 256-token admission, data order —
is unchanged. The score must be **nonlinear** over the relative element; a linear
score is a bilinear form absorbable into the Q/K projections and therefore not a
different operation class (confirmed algebra).

## Fixed conditions

| Item | Fixed value |
|---|---|
| Parent | `fit-quaternion-6/checkpoint-final`, step 15,672, model sha256 `6defec21fc2be395a9505b9f10301c03aeb3c23529c6e1be4e2e1639c7ce6d79`, checkpoint sha256 `490922decefee3e6036331dd0bc112746f24662d4bddfbe540b59c1d00820571`, 64,192,512 target visits |
| Arm A | kernel `signed_2i` enabled; resume parent to `total_steps=16696` |
| Arm B | same binary, kernel disabled (default); resume parent to `total_steps=16696` |
| Arm C | conditional norm-controlled dot (below); not run unless triggered |
| Dose | 1,024 updates × 4,096 targets = 4,194,304 targets per arm |
| Batch/context/access | B16 / T256 / full 256-event admission, `cpu_gradient_shards=2` |
| Seeds | model seed 240924; data seed 240927 |
| Limits | `max_process_seconds=7200`, `checkpoint_steps=[]` |
| Kernel | 16 lanes × 4 coordinates; per-lane L2 normalization (eps 1e-6, clamp gradient); nearest signed root by signed dot (never fold antipodes), lowest canonical index on ties, zero lane → identity; exact composition via `uor-r4-core` `group_table()` inverse+product rows; straight-through surrogate (forward = exact hard relation, backward = smooth Hamilton composition and score derivative); per-lane MLP 4→8→1 tanh = 49 parameters/lane, 784 total, both layers initialized `U(−0.55, 0.55)`, biases zero (per-lane score std ≈ 0.29, total ≈ 1.15, comparable to the Dot initial scale); score = Σ_l E_l(r_ST,l) + existing learned age, then the unchanged interface/NoRead/softmax/value path |
| Quantity discipline | Training length (1,024 updates), evaluation length (233,472 targets), memory access (full 256) and vector width (read 64, state 256) are reported separately |

Protocol resolution: both arms use the same resume mechanism. B is a pure resume (no
new parameters). A resumes shared parameters and moments and initializes its kernel
parameters freshly with zero Adam moments for those parameters only. The added kernel
parameters are the intervention, not a protocol difference; the conditional Arm C
controls for normalization and added capacity.

## Endpoints (frozen; all exposed development data — no fresh holdout)

1. **Comparison-tail Read and NoRead NLL** (nats/target) over 233,472 targets
   (912 comparison blocks × 256), frozen evaluator v2
   (`d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62`); report
   tune (first 64 blocks) and comparison partitions separately.
2. **Source panel**: complete/32, first-noun/32, pairs/16; per-row transitions versus
   the parent and versus B, with unique loss accounting (a row B completes and A
   does not is a new unique loss).
3. **Prose**: the five frozen prompts under the sampled policy (seeds 2014–2018,
   cap 128) and the greedy policy, against the frozen complete-prose criteria;
   record cap-truncation and short-cycle stops.
4. **Hard-path usage**: per-lane distinct codes, occupancy of the 120-entry score
   table, and non-identity relation share.
5. **Guardrails** (below).

## Predeclared decision rule

`dNLL = ReadNLL(A) − ReadNLL(B)` (negative = A better); `dComplete = complete(A) − complete(B)`.

| Outcome | Condition | Route |
|---|---|---|
| **EXTEND** | (`dNLL ≤ −0.02` AND `dComplete ≥ +4` AND zero new unique losses) OR (prose(A) ≥ 3/5 on either policy while prose(B) < 3/5, with no new malformed termination) | propose the integer-compile step and the ordinary-transport (householder) arm as separate later milestones |
| **MIXED → named repair** | material movement not meeting EXTEND or HARM (`\|dNLL\| ≥ 0.015` or `\|dComplete\| ≥ 4` or a prose-bar change), including net row gains with ≥1 new unique loss | name the failing rows/mode and repair only that, with no dose or seed extension |
| **INERT** | `\|dNLL\| < 0.02` AND `\|dComplete\| ≤ 3` AND no prose-bar change | park the kernel at this scope; the next candidate is the next state/read alternative |
| **HARM** | `dNLL ≥ +0.02` OR `dComplete ≤ −4` OR new unique losses ≥ 4 OR (prose(B) ≥ 3/5 while prose(A) < 3/5) OR any guardrail failure | park with exact negative scope; no repair-fit follows automatically |

**Arm C trigger (predeclared):** run the norm-controlled dot arm only if
`|dNLL| ≥ 0.015` OR `|dComplete| ≥ 4` OR a prose-bar change occurs. C uses the same
lane count, unit coding, MLP functional form and **identical trainable parameter
count** as A, and replaces only the relation operand (per-lane group composition)
with a per-lane dot; it therefore separates “finite relation coding” from
“normalization plus added score capacity”. If A ≈ C, the group composition adds
nothing and any effect is normalization/capacity; if A > C, composition contributes.
A and C with unequal parameter counts make C invalid (instrument failure).

## Guardrails and instrument checks (before and around the screen)

- **Numerical**: NoRead penalty ≥ 0.02 nats in both arms; `ReadNLL(A) ≤ parent ReadNLL + 0.03`;
  no increase in short-cycle stops versus B.
- **Hard-path collapse**: ≥ 8 distinct codes per lane on the zero-update window and at
  least one non-identity relation per lane; abort if any lane collapses to ≤ 2 codes.
- **Gradients**: every new parameter family receives a finite nonzero gradient on the
  smoke window; abort on zero, NaN or Inf, or on a systematic hard/soft mismatch
  beyond the declared boundary-tie rate.
- **Baseline invariance**: with the kernel disabled, B reproduces the parent's
  fixed-window predictions exactly at zero updates, and B's fit reproduces
  `plain-16696` comparison Read NLL `1.984753` within `1e-3` nats; otherwise diagnose
  code-path drift before interpreting A. (Historical anchor, not a comparator.)
- **Cost gate**: a 4-update smoke records per-update wall for B and A. If the kernel
  adds more than 1.5× per-update wall, stop and optimize (cache per-event detached
  key codes at write time) before the full screen.
- **Causality**: the evaluator's no-future-read invariant is retained; the kernel reads
  only causally available events.

## Budget and stop conditions

Live shared ledger at freeze: `698,847,339 / 744,000,000` ms (headroom ≈ 45.15M ms ≈
12.5 h). Prospective projection, charged once for the milestone: implementation and
focused checks ≤ 1.0 h; smoke (B zero-update, A zero-update, 4-update fits) ≤ 0.5 h;
A+B fits ≈ 2.0–2.6 h (kernel-on may be higher; the cost gate decides); evaluations
≤ 0.25 h; analysis, evidence, review and protected delivery ≤ 1.5 h. Total ≤ 5.5 h;
Arm C adds ≤ 1.5 h → ≤ 7 h. **Stop margin:** keep the cumulative ledger below
735,000,000 ms and preserve the 128 MiB physical storage stop margin. New storage
≤ ~400 MiB internal (fit/evaluation roots); the SSD build cache is reused. One model
process at a time; check for other labs' running fits before each launch.

## Scope boundaries

- No admission pruning, recurrence change or output-head change; Full admission only.
- Kernel with quantization/rounding or packed paths is rejected by validation in this
  milestone (integer compile is a separate later step, only after EXTEND).
- No edits to `crates/uor-r4-integer` or other labs' dialogue/native-576 files; the
  kernel config lives on the training crate's campaign/checkpoint config and a new
  module, with a minimal adapter in `read_scores`.
- Development scope only. No promotion, no fresh holdout during design selection, no
  dose/seed/decoder extension from a negative.
