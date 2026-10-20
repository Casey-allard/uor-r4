# Finite geometric read kernel: first screen result

September 27, 2026. References #973 under #820. OpenCode/DeepSeek lab.
**Status: executed; outcome HARM at the predeclared screen scope.** The kernel
source and the screen record are delivered; nothing is promoted and no serving
path changed.

The [predeclared plan](geometric-read-plan-2026-09-27.md) fixed the arms, dose,
endpoints, EXTEND/MIXED/INERT/HARM rule, guardrails and budget before any fit.
It implements the fourth lab's reviewed first operator
([`fourth-lab-geometric-attention-2026-09-26.md`](fourth-lab-geometric-attention-2026-09-26.md)):
an optional, absent-by-default learned score over the signed relative element of
the 120-element binary icosahedral group `2I` (16 unit-coded lanes, exact
composition table, straight-through hard-forward/smooth-backward surrogate,
per-lane 4→8→1 nonlinear score).

## What was executed

| Item | Value |
|---|---|
| Source / binary | `b7fd254ad4d9376c51dc6ce0eeac7b336087b786`; release binary sha256 `1dfea84ede55f719ddbe515e83f2f917da72effb6c5a3454187f9c09a2abf8fe` |
| Parent | `fit-quaternion-6/checkpoint-final`, step 15,672, model sha256 `6defec21…` |
| Arm A (kernel enabled) | `fit-quaternion-2i-screen-on-8a6d`, 1,024 updates / 4,194,304 targets, B16/T256/full256/shard2, seed 240924 |
| Arm B (same binary, kernel disabled) | `fit-quaternion-2i-screen-off-8a6d`, identical dose/seed/protocol |
| Evaluations | `evaluate-quaternion-2i-{off,on}-{read,no-read}-8a6d` (frozen evaluator v2, 976 blocks, 233,472 comparison targets) |
| Evidence | [`docs/evidence/geometric-read-2026-09-27.json`](../evidence/geometric-read-2026-09-27.json) (+ campaigns copy) |

## Result

| Endpoint | Parent | B (kernel off) | A (kernel on) | A − B |
|---|---|---|---|---|
| Comparison-tail Read NLL (nats/target) | 1.996490474 | **1.984752805** | 2.004471395 | **+0.0197190** |
| Comparison-tail NoRead NLL | 2.492064889 | 2.491002382 | 2.480454294 | −0.0105481 |
| NoRead penalty | 0.495574415 | 0.506249577 | 0.475982898 | −0.0302667 |
| Complete source answers (32 rows) | 21 | **27** | **17** | **−10** |
| First-noun answers (32 rows) | 26 | 29 | 24 | −5 |
| Five-prompt sampled prose | 0/5 | 0/5 | 0/5 | unchanged |

Row transitions: A has **0 unique completions** and **10 unique losses** versus B
(`story-source-edit-{02|edited, 04|original, 06|original, 07|original, 07|edited,
08|original, 09|edited, 11|edited, 14|edited, 15|original}`). Training also
degrades: retained batch NLL 2.09457 (B) versus 2.10501 (A), with the kernel's
initial disturbance (+~0.4 nats at step 15,673) decaying but never closing.

Numerical and instrument guardrails hold: NoRead penalty ≥ 0.02 nats in both arms;
`ReadNLL(A) ≤ parent + 0.03`; no short-cycle stops in either arm; hard-path usage fully
non-collapsed (120 distinct key codes and 120 distinct relation codes in every lane,
non-identity relation share 0.9925, 784 kernel parameters — measured cumulatively over
the fit, not the plan's zero-update window). The **predeclared cost gate failed**
(1.84× paired same-session versus the 1.5× threshold); it is carried to the full screen
as a recorded budgeted exception, and under the plan's own heading a guardrail failure
is itself a HARM condition, so the decision holds independently of the row criteria.

## Predeclared decision: HARM

`dComplete = −10 ≤ −4` and `new unique losses = 10 ≥ 4` meet the predeclared HARM
conditions (`dNLL = +0.019719` falls just inside the +0.02 bound, so it is not an
independent trigger). Per the plan: **park the mechanism at this exact scope with no
automatic repair-fit**. Narrowed interpretation: this rejects the finite relation score
as parameterized here — including its 784 added per-lane parameters and the
normalization that comes with unit coding — at this dose and representation. The
attribution between finite-relation coding and added capacity/optimizer interaction is
**unresolved** because arm C was deferred; the supported statement is that this
parameterization does not improve and measurably degrades the same-dose baseline.

**Arm C (norm-controlled dot) deferred, with the budget arithmetic recorded.** Its
predeclared trigger fired (`|dNLL| = 0.019719 ≥ 0.015`, `|dComplete| = 10 ≥ 4`). At the
decision point the shared ledger read **712,153,247 / 744,000,000 ms**, up 13,299,908 ms
from the 698,847,339 ms freeze balance through concurrent cross-lab consumption. This
milestone's uncharged accrual is ~5.0 h (18.1M ms), projecting ~730M ms after charging
against the predeclared 735,000,000 ms stop margin; arm C would add ~2.5 h including its
implementation, build, fit, evaluations and extra delivery (~9.0M ms), projecting ~739M ms
and leaving under 5M ms below the hard 744,000,000 ms ceiling while another lab charges
concurrently. C is therefore deferred to a fresh budget decision rather than dropped; the
plan's freeze-time projection would have fitted it, and the departure is caused by
unforecast concurrent consumption, not by a silent budget change. The attribution
question remains open and is the main limitation of this evidence.

## Instrument checks (all pass)

- **Kernel-off path unchanged, evaluation:** the new binary with the kernel disabled
  reproduces the retained parent evaluation token-for-token (witness
  `evaluate-quaternion-baseline-invariance-8a6d2-read`; `blocks.jsonl` bit-identical,
  `generations.json`/`story-probes.json` identical). The first attempt
  (`…-8a6d-read`) was rejected as an unbound-source build and is recorded as a failed
  attempt, not the witness.
- **Kernel-off path unchanged, training:** an accidental 341-update run of the same
  configuration replays the historical `plain-16696` learning curve with **0 NLL
  mismatches** across all 341 overlapping steps.
- **Baseline anchor:** arm B reproduces the historical `plain-16696` evaluation
  exactly, including its comparison Read NLL `1.984752805` ≈ the predeclared
  `1.984753` within 1e-3.
- **Cost gate:** the predeclared optimization (per-event detached key-code caching,
  vectorized per-lane score) reduced the kernel's per-step cost from 15.3/8.7/12.9/10.2 s
  to 6.1/5.4/6.6/9.8 s. The paired same-session ratio is 1.84× over the kernel-off arm
  (kernel-off 3.796 s/step under concurrent cross-lab load; the historical idle baseline
  2.32 s does not reproduce this session). The residual cost is the frozen
  straight-through Hamilton composition over the full candidate tensor; the implementer's
  profiler sampling attributes the residual there, but that attribution is not an
  independent measurement. The 1.5× gate is therefore recorded as missed with an
  explicit, budgeted exception required to complete the predeclared comparison.

## Limitations

All endpoints are exposed development data with a single seed and a single parent;
there is no fresh holdout and no promotion. Reported NLLs are unweighted standard
metrics. Two model processes (this screen and another lab's dialogue fit) ran
concurrently, so absolute wall times are inflated; numerics are unaffected. The 32-row
authored source panel is small and dose-sensitive (the same-parent plain dose itself
moved 6 rows), so the row-based HARM could overstate a per-seed effect; the strongest
caveat is that the harm may arise from the added 784-parameter capacity and its
optimizer interaction rather than from finite-relation coding, and arm C (the
capacity/normalization control) is unrun. This is a first screen for one mechanism, not
a rejection of geometric readers as a family: it rejects this parameterization at this
dose and representation, and retains the exact negative scope at the
data/operator/dose/decision boundary.

## Cost

Charged once at delivery from the milestone start, including the failed/aborted
attempts listed in the evidence: two full 1,024-update fits (3,423.6 s + 5,923.3 s),
the cost smokes (including one 341-update partial run and one shell-aborted run), the
baseline-invariance and four frozen evaluations, and three release builds. No paid
external compute; the 128 MiB physical stop margin was preserved.

## Next decision

Stopped as required. The recommended next milestone is the **state/read path**
targeting the two measured failure modes (entity/role collapse after the correct
noun; cap-truncation), not a re-parameterization of this score — or another
owner-chosen branch.
