# Existing geometric readout: native direction evidence, October 3

The existing Rust learner supplies usable local credit for the retained geometric
readout. This diagnostic reuses the final64 checkpoint and performs zero optimizer
updates. All four selected geometric coordinates have legal native quarter-steps
that reduce joint answer CE and conditional Copy CE. None of those descents improves
Copy-only top1, and some worsen joint Copy/Period decisions. No candidate is adopted.

The affected `context_unary` coefficients weight the Q25 observation of a signed-H4
relative code. The native reader uses the compiled q4 tables and integer additions.
This measures the existing geometric attention/readout component on selected records;
complete native chat, unseen-source generalization and whole-path serving remain open.

## Method and identity

Executed source: `7a517dffe5021352c58ee8046118c88b9b532d25`, clean committed git archive.
The Rust `direction` mode uses the existing `SourceRealizerWeights.prepare/loss`
and parameter APIs. It reproduces all114 canonical traces/ranks from the
[prior audit](geometric-final-prefix-audit-2026-10-03.md), and the baseline own-prefix
rows reproduce the saved M1 generation. Inputs are the same20 exposed development
cases, five distinct source-view/query groups, bound4096 BPE/protocol2, full
source+query+prefix under128 positions and one selected record. Period/EOS targets
remain in the ordinary mean20episodes(mean token CE) objective.

One gradient pass selects four nonzero-credit coordinates whose fixed features
distinguish a target Copy occurrence from a winning Copy token. Selection orders
absolute gradient, parameter name, then index. Both legal q±1 directions are tested
independently from the original checkpoint. One common-Copy presence coefficient
provides two calibration controls: ten candidates total. Context, Period and Stop
payloads stay fixed. The changed shadow becomes newq/4; the actual off-grid shadow
displacement and the native ±0.25-nat step prediction are reported separately.
Each candidate is sealed, original shadow bits are restored and inputs reverified.

The selected declared adjoints match independent analytic reconstruction: maximum
absolute autodiff difference1.2796126603e−8 across five checked coordinates. This
validates those surrogate calculations at this checkpoint. It does not establish
the derivative of a discrete map, every model gradient or a globally sound learner.
Finite-step prediction error can include curvature and native rounding.

## Native results and the calibration tradeoff

Baseline joint CE is1.3407132581 and joint correct choices46/114. Copy targets have
18/74 joint-action correct choices versus20/74 Copy-only correct choices; Period
is18/20 and Stop10/20. These are canonical-prefix diagnostics, not generated replies.

| Coordinate | Δq | Joint CE change | Joint correct | Gains | Losses |
|---|---:|---:|---:|---:|---:|
| context_unary[7] | −1 | −0.024265 | 42/114 | 0 | 4 Copy |
| context_unary[7] | +1 | +0.028768 | 40/114 | 0 | 6 Period |
| context_unary[24] | −1 | −0.020221 | 46/114 | 0 | 0 |
| context_unary[24] | +1 | +0.027372 | 42/114 | 0 | 4 Copy |
| context_unary[29] | −1 | +0.026748 | 48/114 | 2 Copy | 0 |
| context_unary[29] | +1 | −0.019920 | 36/114 | 0 | 4 Copy +6 Period |
| context_unary[17] | −1 | −0.019775 | 40/114 | 0 | 6 Period |
| context_unary[17] | +1 | +0.026302 | 44/114 | 2 Copy | 4 Copy |
| content_presence[0], control | −1 | −0.002610 | 42/114 | 0 | 4 Copy |
| content_presence[0], control | +1 | +0.010682 | 40/114 | 0 | 6 Period |

The four geometric CE descents also lower equal-episode conditional Copy CE by
approximately0.01975–0.03598. All retain Copy-only top1 at20/74 with no changes.
Their joint Copy losses result from terminal competition. The opposite29−1 step
trades two gains and two losses in Copy-only ranking while gaining two joint Copy
choices; both joint gains are the same late Klotdradburg numerical input. Its first
token error persists. Stop choices remain unchanged for every candidate.

The common-Copy control preserves pairwise raw Copy Q24 differences. Its−1 step
worsens all74 joint Copy target probabilities while improving all40 terminal
probabilities, enough to reduce joint CE. Conditional Copy CE changes only by
about9e−9 of native rounding. Average CE alone can therefore hide capability losses.

Candidate own-prefix generation is **NOT_RUN**. Only the baseline's complete4/20
replies and20/20 EOS replay are established. No fresh held-out draw, fit, better
complete answers, family-capacity conclusion or geometry-superiority claim follows.

## Next existing-learning continuation

Freeze the learned context and let the existing Copy potential, Period and Stop
readouts coadapt under the unchanged joint answer CE. Reuse the saved checkpoint,
prepared loss, optimizer and native export path. Stable geometric features are the
causal change from the prior context/readout joint fit; there is no new learner,
state, loss rewrite or estimator replacement. Fixing terminals was appropriate for
this diagnostic and does not constrain the subsequent learning task.

Before that continuation, declare its bounded cost and source/configuration. Retain
the parent and all candidates. Assess checkpoints with actual complete own-prefix
replies/EOS, canonical joint and conditional Copy CE, and every stage gain/loss.
Do not promote the lowest average CE by itself. At this diagnostic's completion,
the fit was **NOT_RUN**; the [executed continuation](geometric-readout-coadapt-2026-10-03.md)
now supplies its outcome and next action. Shared session changes stay coordinated
on#1552.

## Checks, retained evidence and cost

Linuxx86_64 CPU-only Runpod, no GPU or Metal. Six focused Rust tests pass at the
executed source: alias aggregation, absent target, native ties, legal one-quantum
packing, deterministic selection and shadow restoration. Test worker102.125s,
sampled process-tree peak9,456,652,288B; optimized build274.946s, peak5,605,388,288B.
The successful diagnostic worker30.399s (report29.063s), maximum child
RSS1,274,437,632B, is within900s/4GiB. Python supervises processes/resources only;
model loading, loss, gradients, candidate construction and inference are Rust.

Independent reviews check1,140 candidate rows, all five analytic gradients,
119 returned file SHAs, eleven manifest inventories/sizes, ten one-q mutations,
frozen other binary payloads, unchanged context traces/terminal scores and228
calibration rows. Rust seal verification ran on the pod; local transfer SHA checks
and inventory reads do not claim an independent local BLAKE3 seal rehash.
The in-memory restoration assertion is executed evidence, not a saved-file proof.

Report SHA256: `b9b0deaf8c54a5457a0e57be331b17480d758c8e26c806ab3c5f54ad64b8de55`.
Executable SHA256: `31ddfc7a1645046ba03f691cac34f719c680cbc38530a8f7c911fcaa23d9f5c5`.
[Bound evidence](../evidence/geometric-q4-readout-direction-2026-10-03.json).
Local retained root: `~/uor-r4-local/workspace/research/geometric-q4-readout-direction-20261003`.

The first cleanup guard refused resolved Git LFS payloads without deleting source.
After validating complete Git inventories and Git/LFS identities, two reproducible
old source copies were removed. Checkpoints, results, logs and caches are preserved.
The projected new pod storage is explicitly revised2→3GiB before more compilation;
the9GiB hard stop/10GiB area ceiling stay fixed. The Codex job card is released;
final ownedarea5,465,024KiB (about5.21GiB), with all results brought back locally.

Complete preparation/build/model/review/cleanup/transfer/documentation/delivery is
charged as a conservative45-minute estimate, separate from measured workers:
cumulative1,055,235,028→1,057,935,028ms under the unchanged1,130,000,000ms limit.
Fresh-main reconciliation changes documentation/CI only; numerical source and
dependencies remain equal to the executed head. No additional model run is counted.
