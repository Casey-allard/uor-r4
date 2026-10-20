# #1794 — mqar-bench: query/key accessor + checkpoint load path — the norm is not the obstruction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:59:02Z
- **Updated:** 2026-10-09T13:21:19Z
- **Closed:** 2026-10-06T05:00:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1794
- **Merged:** 2026-10-06T05:00:41Z
- **Merge commit:** 6007da39d7c52ee29d1ac90f18a15b8041a0aeb1
- **Base / Head:** main / UOR-Foundation:deepseek/norm-split
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds the query/key accessor that was missing, plus a checkpoint load path — and records a result that **reframes the Stage-0 headline**. `+313/-3`, add-only; production serving behaviour unchanged (the new surface is a read-only diagnostic, `read_qk_rows`).

## Why this test existed

A Stage-0 measurement held every learned vector fixed and swapped only the score function. A plain **dot** product on the same frozen q/k was as bad as the fused read, and the verdict was "the representation is the missing component" — with a recorded qualification that *"identity EXISTS but is **norm-entangled**"*, a third possibility the H_repr/H_head dichotomy does not name. **The cosine control had never been run**, because no key-vector accessor existed. This adds it.

## Pre-registered, before compiling (2026-10-06T04:39:57Z)

- **Gate**: B1 (dot) must reproduce the sealed A2b numbers.
- **NORM-ENTANGLED**: cosine value top-1 **≥ 0.05**, CI excluding 0.0093.
- **NOT-LINEARLY-RECOVERABLE**: **≤ 0.02**, CI overlapping the baseline.
- **else PARTIAL.**

## Gate 1a: PASS, stronger than required

Loading the frozen checkpoint with `steps=0` reproduced the sealed `stage0_readout.json` **bit-identically** (overall, by_bucket, row_outcomes, a3_sieve_detail, panel) and the documented reference exactly (0.2554; d16 .807 / d64 .176 / d200 .010 / d400 .029). B1 = sealed A2b: key .0048, value .0093, r@16 .1664. Checkpoint **reused, not retrained** — both hashes verified before any measurement (`29b6b97a…83b9`, `52fe9d4e…de2e`), zero training steps.

## Gate 1b: PASS, and it found a new fact about the Stage-0 instrument

- offline L2 vs the library's own dumped weights: **2.17e-07 at all six layers**.
- offline **dot** vs library dot: 1.85e-07 at **layer 0**, but **0.197…0.919 at layers 1–5**.

Cause, measured not assumed: swapping `config.read` **re-runs the forward**, so the swapped read's output feeds every later layer — **q/k at layers ≥1 are not the vectors A1 scored.** So **Stage-0's A2a/A2b were a truly frozen-vector swap only at layer 0.** That is exactly the gap this accessor closes (one forward, all six layers bitwise fixed). Inertness measured, not asserted: the capture's forward returns logits bit-identical to the weight-dump forward.

## Verdict: **PARTIAL** — because the pre-registered clauses disagree

Rule 2 (≥0.05) **does not fire** — B2 = .0143, 3.5× below the bar. Rule 3's two clauses **disagree**: value .0143 ≤ .02 ✓, *but* B2's CI excludes the baseline point .0093 while overlapping the baseline's own CI. Per the pre-registration's own discipline, **PARTIAL fires, with both readings reported** — the flattering reading was not taken.

## The decisive like-for-like reading

The pre-registered baseline .0093 is the **library's re-run forward**; the correct same-vector baseline is **B1b (offline dot) = .0137**. On **identical frozen q/k**:

> **cosine .0143 vs dot .0137 = 1.04×.** The bar was 5×.

**The norm-entanglement escape CLOSES**, three independent ways: (1) normalising recovers nothing (1.04×); (2) **the norm is not the signal either** — magnitude-only .0041 is *below* the dot baseline and cos+mag .0020 is worse than either, so the norm neither drowns nor carries; (3) it is not a sharpness artifact — temp-matched returns **identical** rank metrics (rank metrics are monotone in score, hence provably temperature-invariant).

## The finding that reframes Stage-0's headline

**A1's .0595 is dominated by the learned per-distance AGE BIAS — a positional prior.**

- Age **zeroed** → *every* family (dot / cosine / direction-only) collapses to a **flat 1.2–1.7% across d16..d400**.
- Age **restored** → *every* family jumps **7.5–13.5× AT d16 ONLY** (dot .0095→.1285, cosine .0148→.1111).
- **At long range, A1 is WORSE than both dot and cosine.**

> *"So the surviving signal was a recency effect — not content identity, and not a norm-vs-direction effect."*

Layer-0-only (768 head-rows; the strictly frozen comparison): A1 .2214, dot .0534, cosine .0508, cosine+age .1185, magnitude-only .0000.

## Named missing component (no mechanism retired; D20 §3 not invoked)

**Content identity in the read's query/key projections.** The candidate set **contains** the answer — A3 sieve = **1.0000** on the same rows, positions and candidate sets — and **no score function over these vectors finds it**: the purest linear content readout (cosine) reaches .0143 pooled / .0508 at layer 0, **flat across distance**, against the sieve's 1.0000.

**The read has a usable positional selector (age, worth 7.5–13.5× at d16) and NO measurable content selector at any distance.** Two instrument gaps are named alongside: the layer-0-only validity of the old swap, and `ReadScore`'s inability to express a normalised score (closed measurement-side, deliberately left open serving-side).

**What would change the answer:** a representation change lifting direction-only long-range value top-1 above 0.05 excluding .0143; **a second draw on a fresh seed/panel** (cheapest — needed to tell a 1.04× effect from noise, since the two baselines disagree); a controlled-N panel (all arms sit at/below chance on key top-1); and the **parameter-matched ordinary attention comparator on the same vectors — never run** (D20 §2 cond. 1), which is the one outcome that would move the defect from representation to read geometry.

Cost: CPU only, **zero training steps**, cold build 490 s, run 24.8 s, ~10.2 min total; the author reclaimed its own 665 MB cache (net retained ~74 MB). All four roots newly and exclusively claimed; none reused. The rebuilt-binary rerun gave 23/25 files byte-identical with **zero numerical difference**.

References #820, #1747, #1785.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

