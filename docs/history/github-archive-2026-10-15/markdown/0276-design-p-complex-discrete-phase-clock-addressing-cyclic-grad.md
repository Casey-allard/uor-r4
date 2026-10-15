# #276 — Design P: complex discrete phase-clock addressing (cyclic graded codes on the zeta torus)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T03:31:34Z
- **Updated:** 2026-08-03T21:05:21Z
- **Closed:** 2026-08-03T21:05:21Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/276
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Near-neighbor prediction failed in the 4D continuous embedding (capacity: 32k tokens × contexts cannot be discriminated in R⁴; neighborhoods collapse into collision zones dominated by session trajectory rather than content). But NN already works where the space is large enough — top-M membership assignment over 288-bit signatures IS k-NN under Hamming. This issue tracks the phase-space variant of that lesson.

The lineage's zeta-seeded word vectors `sin(ln(p)·γᵢ)` are the REAL parts of phases — each component is a clock reading at frequency γᵢ with the imaginary half discarded. Completing the analytic signal (store cos + i·sin per frequency, i.e. complex discrete L2) makes phase transport an exact rotation, and "walking around the clock" a well-defined discrete step per frequency. A product of clocks is a torus with real capacity, unlike a single clock (capacity N — insufficient alone).

Prime-router evidence consistent with this framing: the mod12 signal-preservation tables (research report §6; recorded in docs/prime_router_geometric_context_evidence.md, PR #275) show linear decodability resolving to 1.0 on a discrete phase coordinate at trajectory end.

## Design sketch

- Per frequency γᵢ, quantize phase to b buckets → a cyclic graded digit (a b-hour clock).
- Context address = vector of clock digits across F frequencies (torus coordinate).
- Distance = cyclic L1 per digit, summed — realizable as table-reads only (mul-free runtime contract §4 holds; quantization at compile time is unrestricted).
- This is the cyclic sibling of Design G's thermometer coding (docs/graded_signature_address_design.md, PR #270): G grades magnitude on a line; P grades phase on a circle.
- Next-token prediction candidate mechanism: apply the per-token discrete phase step (transport), then nearest stored configuration under cyclic distance.

## Measurement

Joins the SAME decomposition harness as Designs R/G (PR #270): a P row against the A-f32 ceiling and A-binary baseline of the #244 matrix, at unchanged op census. Same empirical criterion (≥30.1% top1 / ≥34.1% agreement) and exit rule. No separate harness.

## Sequencing

After #243 Phase A completes — Phase A's single-loss rows tell us where the signal lives before we add a third design. Related: #243, #244, #245, #246. External design reference: sanity/renegade (KNN mechanics; see companion VP-tree issue).

## Comments (3)

### comment 5124270616 — 2026-07-29T23:03:47Z — Casey-allard

Design doc landed in batch-2 (PR #289, merge c187ef0): `docs/phase_clock_address_design.md`, status **DESIGN FOR REVIEW** — ⚑ flags open for Casey / Ari / Alex: encoding (cyclic thermometer vs digit LUT), b ∈ {8, 12, 16}, frequency set, phase-source lane (transport-composed primary vs #246 design-matrix), joint-vs-separate membership scans. Positioning is synced to the #243 round-2/3 verdict: P is a channel addition measured against the post-decision baseline (shift-add dot assignment), joins the existing certify decomposition harness, standing criterion, recorded-negative exit rule. Sequenced strictly after the #243 kernel PR.

### comment 5169028379 — 2026-08-03T16:27:40Z — Casey-allard

Re-evaluation trigger per the #374 queued-next plan: the sweep results are in, and they strengthen this design's position. Under probability/contrast the cover-granularity knob converts (36.98→37.70% top-1 at 38→102 regions) but the gap to the store baseline (−1.4pp) remains, and per-region contrast tops out at 0.91 with mean 0.59 — the addressing itself is now plausibly the binding constraint, which is exactly the lever Design P proposes (cyclic phase-clock addressing as a distinctiveness mechanism). The review flags in the design doc (encoding, b, frequency set, phase-source lane, joint-vs-separate scans) are worth resolving against these numbers; sequenced after the #364 follow-up (a) distinctiveness term establishes the induction-side baseline.

### comment 5170097140 — 2026-08-03T18:14:14Z — Casey-allard

## Maintainer decisions: all five ⚑ flags resolved (review brief 2026-08-03; Ari/Alex review still welcome on the implementation PR)

1. **Encoding: cyclic thermometer** — reuses Design G's popcount path and every downstream consumer unchanged (#339 VP-tree, ROUT prototype/mask, #247 secondary probe). Digit LUT reconsidered only if measured F·b width becomes the binding artifact cost.
2. **b = 12 primary, b = 8 control** in the same row batch (mod12 is the only empirically privileged modulus; 8 is the power-of-two control). 16 dropped initially — width without evidence.
3. **F = the 16 design-matrix frequencies** (#246/#313, now on main); stride-8 subset as the cheap control arm.
4. **Both phase-source lanes in one batch**: lane (b) transport-composed primary (the mechanism claim), lane (a) design-matrix projection now unlocked by #313 — one run answers the composition question for free.
5. **Separate membership scans first**; the P∘G composition row runs with concatenated distance but separate scans so the op census stays attributable per channel. Joint scan is a later optimization question.

Plus: the Empirical Criterion re-anchors to the TLA7-era A-row measured in the same run the P rows land in (#294 convention) — the 30.1/34.1 anchor is TLA5-era.

**Updated motivation, and sequencing:** the #364 entropy-collapse analysis (2026-08-03) shows the binding constraint on region distinctiveness is **what splits geometry proposes**, and #386 measures that the veto-side distinctiveness weight leaves the frontier unmoved at w=1.0. Design P is a proposal-side change — a phase channel alters what the clustering sees. P work starts after #386 lands, measured against the post-#386 machinery.


---
_Comment/review ids in this file: 5124270616, 5169028379, 5170097140_

