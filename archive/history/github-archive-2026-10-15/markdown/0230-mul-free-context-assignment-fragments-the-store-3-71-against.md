# #230 — Mul-free context assignment fragments the store 3.71× against its own f32 control

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @maurathat
- **Created:** 2026-07-27T03:26:35Z
- **Updated:** 2026-07-30T01:57:28Z
- **Closed:** 2026-07-30T01:57:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/230
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

A single `certify` run reports both the shipped mul-free path (A-binary) and an f32 ablation differing only in whether multiplies are used at context-to-class assignment. Compared like-for-like against its own certified row, the f32 path's store grew modestly; the shipped mul-free path's store grew 3.86× and is 3.9 pp less accurate than its control.

| Path | top-1 | teacher agreement | WB bits/token | store keys |
| :---- | :---- | :---- | :---- | :---- |
| A-binary (shipped, mul-free) — this run | 28.2% | 31.9% | 9.7868 | **344,604** |
| A-binary — certified (PROOF.md P2) | 28.9% | 31.7% | 6.5427 | 89,200 |
| A-f32 (ablation) — this run | 32.1% | 36.4% | 9.6982 | 92,903 |
| A-f32 — certified (PROOF.md P2) | 31.5% | 34.7% | 6.3214 | 86,574 |

Like-for-like: A-f32 keys are +7.3% vs their certified value; A-binary keys are +286% vs theirs. At certification time the two paths had nearly equal stores (89,200 vs 86,574). The fragmentation is a regression in the mul-free assignment since the P2 pin, not an inherent cost of going mul-free. Same corpus, same run, same harness, one variable.

## Why this is a defect rather than a known tradeoff

The mul-free path is the shipped runtime and the basis of the throughput claim. Its assignment step is currently costing, relative to its own f32 control in the same run:

- **3.9 pp top-1** and **4.5 pp teacher agreement**
- **3.71× store keys** — 344,604 vs 92,903, roughly 11.3 MB vs an estimated 3 MB

That is a large, quantified headroom in the deployed path. If a mul-free assignment could match f32 assignment quality, the artifact would be substantially smaller *and* more accurate at unchanged throughput.

## Scope note

This issue is about assignment only. The codebook itself is doing its job — the B bit-prefix path (no codebook classes) is worse on every metric: 22.1% top-1, 25.0% agreement, 11.8762 bits/token, 405,548 keys. Design A beats Design B decisively; the problem is inside A.

## Not addressed here

Bits/token is +50% against the certified figure in **both** A configurations (9.7868 and 9.6982 vs 6.5427 and 6.3214), roughly independent of key count. That is a separate matter — a uniform entropy shift that leaves argmax intact is more consistent with a scoring-representation difference than with fragmentation. AGENTS.md notes two compatible ScoreQ definitions in flight pending pre-Phase-5 consolidation; that is the first place to look. Filing separately if it does not fall out of consolidation.

## Environment

Commit `0ffb71b94d78ff722af089f1337a08fce42dccff` (HEAD of main, 2026-07-26). rustc 1.95.0, `--release`. macOS 26.5.1, arm64 — the platform the κ baseline is pinned to. Fresh clone; no pre-migration store reused. Fixture `/tmp/ref/out/model.bin` present and size-verified (60,816,028 bytes), source κ confirmed by `certify` as `blake3:0ae73395…`. Corpus built with `gen 300 150000`: 150,000 tokens, 754 stories, 30,036 held out.


## Comments (3)

### comment 5099736194 — 2026-07-28T03:53:19Z — Casey-allard

Roadmap note tying the new issue set together (filed 2026-07-28, from the two-lineage reintegration plan):

**Sequencing / dependencies**

```
#241 (CI, merged) → #242 (tokenizer) → #231 (inf clamp) → #237 (context fix) → #244 + #243 (#230 splits)
#245 (content→store) → #246 (spectral projection) → #247 (session-signature lane) → #234 + #249
#248 (cascade honesty) — independent, anytime
```

- **#242** is the highest-leverage single fix: until it lands, every HF-path measurement (and artifact) is suspect.
- **#243** and **#247** are the two points where the geometric/R4 lineage becomes load-bearing in the primary engine — addressing and long-range binding respectively — each with a quantified benchmark that already exists.
- **#245** is small (two edits) and independent; good immediate start.
- **#244/#243** split #230 by verified mechanism: store size is the multi-membership beam; accuracy is sign-only quantization. The +50% WB bits/token anomaly is neither (WB path is pure f64; ScoreQ untouched) and still needs its own investigation.

This comment closes the loop on #230's "not addressed here" items and the geometric-reintegration discussion.


### comment 5105905062 — 2026-07-28T15:06:34Z — Casey-allard

Roadmap addendum (2026-07-28, afternoon): four UOR-integration issues filed from the gap analysis of the product goal ("inference = content-addressed lookup") against what exists — the seam where UOR currently wraps *around* inference rather than participating in it:

```
#243 (address space) ──▶ #263 (region objects + resolver) ──▶ #266 (proof-carrying witness)
        │                        ▲
        └── #244 decision        │
#264 (R4G1 realization for uor-addr, + vendored sync) ──┘   (also feeds #259)
#265 (D2 canonical deterministic compile) — independent, load-bearing for all distribution claims
```

Also recorded (from today's investigation of the ordered/bijective-digest question, verified against the Foundation's latest HEADs — which are exactly the revs this repo pins): **no change to uor-addr itself is needed or wanted** — κ-labels' uniformity/avalanche are CI-enforced design goals, and everything ordered/truncatable/distance-bearing belongs to the relational layer per docs/addr_prism_correspondence.md (#258). If work ever appears blocked on an invertible digest, that is the layer conflation the correspondence doc exists to catch.


### comment 5123946277 — 2026-07-29T22:19:21Z — Casey-allard

Update from the #243 decision rows (2026-07-29): the fragmentation mechanism is now attributed. The shipped mul-free path's 3.71× key blow-up and accuracy gap trace to **sign-Hamming assignment**, not to residual handling — A-dot-only (dot-product assignment against the same per-stage codebooks, no residuals) measures 30.4/34.2 with **47,603 keys** (7.2× fewer than the shipped 344,604 in the same-run comparison, and below even the f32 ablation's 92,903), while adding true residual subtraction to the sign metric changes nothing (27.0/30.5 either way). A store keyed by dot-assigned codes is simultaneously smaller and more accurate at unchanged runtime ops **if** the shift-add dot buildability rows (power-of-two centroid value sets, #243 round 3) hold the criterion. This issue's DoD then rides on #243 Phase B adoption.


---
_Comment/review ids in this file: 5099736194, 5105905062, 5123946277_

