# R4-Native Architecture: Restoring Geometry as the Load-Bearing Substrate

- **Status:** Historical research/design record (issue #393; programme issues
  #394 and #395); not current architecture or sequencing authority.
- **Date:** 2026-08-04
- **Claim language:** per `docs/formal_vocabulary.md`. Measured statements
  cite their issue records; everything else is design intent, not evidence.
- **Current authority:** [Geometric Intelligence Programme](geometric_intelligence_programme.md).
  Its route-native spherical-harmonic/spin hierarchy carries forward relevant
  evidence without adopting this document's anchor-infill or hybrid serving
  sequence. No source-free chat capability follows from this record.

## 1. Purpose

The 2026-08 measurement campaign (#374) closed a chapter: every geometric
*scoring channel* added on top of the serving stack measured zero or negative
(#379/#386 distinctiveness, #385 penalty-as-cost, #276/#392 phase clocks),
and held-out generalization sits below the unigram null (#391). This document
states why that outcome was structural rather than accidental, what the
system's actual product role is, and the program for making R⁴ geometry
load-bearing again — with every piece attached to a falsifiable measurement.

## 2. The two stacks

The repository carries two architectures that barely touch:

**The R4 stack** (`uor-r4-router`): 512-dim state grounded on the zeta-zero
grid (`zeta_zeros.rs`), 4D block-norm projection, Hopf sector transport on S³
(`hopf_sector_occupancy` harness, #303), VSA/spectral geometry, evolving
session state. This is the original geometric mechanism. No active board
track measured it during the campaign.

**The serving stack** (`uor-r4-graph-*`): R4G1 bundles whose semantic space
is the sign-bit orthant structure of teacher representations — a
locality-sensitive hash of the *teacher's* learned geometry — plus
exact-context NGRAM rows (memorization). Every campaign result lands here.

The consequence (#391): the serving stack's semantic manifold is inherited
from the transformer it observes, not constructed from R⁴ first principles.
Memorization carries the blended metrics; the geometry that exists is either
inherited (Hamming balls over teacher sign bits), retired by measurement
(phase, #392), or ungrounded (see §7).

## 3. The objective is misaligned with the product role

Certify's headline rows grade open, teacher-forced next-token top-1 —
transformer mimicry. Under that objective, memorized exact-context channels
are the best strategy, and geometric mechanisms structurally cannot win;
the campaign measured precisely this, repeatedly.

The system's historical role in the hybrid deployment was different: the R4
router supplied **content anchors** (roughly every 4th token) and a
downstream LM completed the **syntax between them**. The serving stack
exists to take over the completion job. It should be graded on that job.

## 4. The anchor-infill criterion (#394)

Every 4th reference-stream token is pinned as an anchor; free positions are
graded. Day-0 fixture numbers (harness merged in #396; 500k records, 2,507
stories, held-out free targets 75,385):

| arm | top1 (free) | off1 | off2 | off3 |
|---|---|---|---|---|
| shipped causal store | 34.7% | 34.5 | 34.8 | 35.0 |
| null: unigram | 6.3% | — | — | — |
| null: bigram | 28.7% | — | — | — |
| null: fwd-anchor table | 21.9% | 14.2 | 19.0 | **32.7** |

Two measured facts anchor the program:

1. **Forward context is a large, unconsumed signal.** A table conditioned
   only on (next anchor token, distance) — zero past context — ties the
   entire causal store at offset 3. The offset gradient (14→19→33) is
   syntactic anticipation. No current mechanism consumes any forward
   context.
2. **The store's margin over a bigram table is ~6pp** on this slice —
   the geometric store's current added value at the infill task, consistent
   with #391's memorization finding.

## 5. Syntax as routing (design direction)

The transformer's answer to §4.1 is attention: content-based lookback plus
(at training time) bidirectional credit. The R4-native answer is that each
region carries its routing: forward transition edges (E_f) and reverse
indexes (E_b) already exist in the format as the successor/predecessor
structure. The design obligation is an inference step that, given the pinned
anchors, routes *backward from the next anchor* through E_b while routing
forward through E_f, and scores free positions from the intersection —
grammar as precompiled routing structure rather than dot products. The
fwd-anchor null is the floor such a mechanism must clear at off3; the
criterion (#394) is how it gets graded. Design work happens against the
criterion, not against open next-token rows.

## 6. Construction-level geometry: the E8/icosian store (#395)

The campaign killed scoring channels, not substrates. #276's negative ruled
out phase as an *additive term*; it says nothing about what the store is
made of. Measurement 1 (PR #397; matched 288-bit budget, real store
vectors): E8 block quantization beats the shipped orthant code on every
metric — recall@10 0.606/0.634 (cos/L2) vs 0.553/0.541, relative
reconstruction MSE 0.213 vs 0.446, with the codebook truncated to 256 of
~22k observed lattice points per block.

The icosian reading makes this R⁴-native in a precise sense: the icosian
ring (quaternions over Z[φ], Turyn norm) is the E8 lattice as a Z-module,
so each 8-dim block codeword is canonically a golden-coupled pair of R⁴
points. A context vector becomes 36 icosian pairs — 72 quaternionic
orientations — rather than 288 uncoupled sign bits. The 600-cell / H4 ⊕ φH4
folding gives the same object its polytope description. Decoder cost is
round/compare/coset-select per block (Conway–Sloane), plausibly conformant
with `INFERENCE_OPERATION_CONTRACT.md` under fixed-point scales
(conformance unverified; required before any serving use).

Follow-on (decision gate met): a compiled bundle whose assignment/membership
geometry consumes icosian block codes, graded by certify and #394 at equal
store budget.

## 7. The ungrounded term

`syntactic_morphism_score` (graph-runtime `engine.rs`) computes a
Cayley–Dickson centralizer score over `CayleyDicksonVector::from_u32(token)`
— a multiplicative hash of the raw token id. It carries no corpus, teacher,
or geometric content and predates the measurement discipline. It must be
either re-grounded (operate on H(x)/region geometry, where a centralizer
score has meaning) or measured and removed. Its hard-coded token-id lists
(determiners/prepositions) are the only explicit syntax mechanism in the
system and should be superseded by §5.

*Addendum 2026-08-05: resolved by measurement — the #400 A/B recorded the
term as dead (0/1,998 executions on the surface that calls it) and it was
removed in PR #410; `cayley_dickson.rs` remains in the crate without serving
callers.*

## 8. Program and gates

| track | issue | gate |
|---|---|---|
| Anchor-infill criterion | #394 | stride sweep + natural-corpus rerun (from #381 obs); bits ladder |
| E8/icosian store | #395 | construction experiment ≥ shipped assignment on certify AND #394 at equal budget |
| Syntax-as-routing (E_f/E_b infill step) | new, after #394 stabilizes | clear fwd-anchor null at off3; then clear shipped store overall |
| CD term hygiene | new | measured contribution or removal |
| Structural stratification | private track | construction-partition gains on held-out, #391 nulls |
| Router-stack reconnection | after the above | routed anchors from the R4 stack feeding #394's pinned positions end-to-end |

Scoring-term additions to the serving stack are a measured dead end and are
out of scope for this program.

*Addendum 2026-08-05 (issue records are authoritative): the CD-term-hygiene
gate closed as removal (#400/#410). The forward-anchor track landed as #399
(PRs #414–#418): the optional FWDA section plus `score_candidates_infill`,
with anchors-as-inputs (A-mode) measured positive on the live slice and the
standalone two-pass B-mode refuted by two falsifiers (Gate C arms for
true/self/gated/draft/strict anchors, report schema 21).*

## 9. Governance

Measurement pins and results are data and live on their issues. Whether any
of this changes certify's *gating* policy remains with Alex & Ari; this
document proposes criteria and mechanisms, not gates.

## Addendum 2026-08-05 — closure verdicts (issue #425 hygiene pass)

The program issues this document opened are now closed with records; the
issue records are authoritative and this addendum only pastes the verdicts.

Closures: the #374 measurement campaign, the #394 anchor-infill criterion,
and the #395 E8/icosian store track are closed with recorded results. #394's
harness, stride sweep, and bits-ladder rows landed (#396, #402, #405, and
the rows-inclusive baseline); the criterion itself is superseded in serving
by the #399 A-mode infill surface recorded in the section-8 addendum above.

The #395 record fixes a design law. The v0 construction experiment — an E8
spatial-group store that flattened lattice codes into the store's *keys* —
recorded NEGATIVE for that keying, with the mechanism diagnosed (commit
`88a9b9d`, PR #403). The v1 residual-E8 experiment moved the lattice to the
comparison side — snapping residuals during scoring, retraining-controlled —
and recorded that lattice snapping is FREE at 0.5 rms (commit `558d8cd`,
PR #404). The law, stated once so it is not relearned: the E8 lattice
belongs on the COMPARISON side of the store, never flattened into keys.
Keying by lattice code destroys the neighborhood structure the store routes
by; snapping on the comparison side keeps it and costs nothing measurable.

Related dispositions from the same pass (FMM removal per the #290 negative,
the Spin(4) attention deferral, and the bott_fock re-pointing to #424) are
recorded in `docs/deferral_record_2026_08_05.md`.
