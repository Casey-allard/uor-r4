# #1306 — Prefix recovery + paired fixed-feature output-head diagnostic (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T07:47:22Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-20T07:49:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1306
- **Merged:** 2026-09-20T07:49:43Z
- **Merge commit:** d15360527f7c69ac8b83eef0bbd5839b87c26f02
- **Base / Head:** main / UOR-Foundation:codex/readout-diagnostic
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the complete second document of the #1305 pair. The frozen parent is never retrained; every sealed root is preserved.

## Part A — recovery of the three negative prefix candidates (no refit)

- **Bound exact `2I` table.** `learner/prefix_artifact.rs` rebuilds the table from the exact twice-scaled `Z[phi]` construction and matches it to the historical `learner::embedding` enumeration through an explicit bijection. All **14,400 products are verified twice** (independent exact `Z[phi]` quaternion multiplication AND `validate_h4_binary_icosahedral_closure`) with **0 mismatches** against the historical floating table, so the learned function is preserved; the table is stored in historical state order so reader rows and palette/action IDs keep their meaning.
- **Executable artifacts.** `bin/prefix-recover.rs` reconstructs each pinned `CPXS` into a `CPX2` (51,507 bytes) carrying packed reader/output coefficients with shifts, nibble-packed actions, the exact table/inverse/root-order mapping, palette, arm, dimensions, `F`/bias scale and the **raw parent-file digest**. The legacy `parent_sha256` = SHA256(hex digest string) convention was validated. The recovered packed reader/output hashes and the reconstructed **RAW** action-array hash equal the old descriptors before any evaluation.
- **Exact reloaded parity:** 0 state and 0 logit mismatches on all 17,342 panel records for all three arms; recorded losses reproduced exactly (parent 7.558429; primary 7.575604; fixed-action 7.723496; local-tail 7.576439).
- **Comparator fixes.** Generation now uses one arm-aware state selection in the library and the runner (the local-tail arm previously generated with the older-prefix state); the count-reference argmax scans **all 4,096** entries with lowest-ID ties, and corrected outputs no longer collapse for 5 of 6 prompts. The local-tail arm is now **exactly invariant** under the older-prefix conditional permutation (penalty 0.0), and reverse-order is reported NOT_APPLICABLE for it.
- **Control vectors.** Per-occurrence vectors for parent/own/permuted/reversed/disabled of every arm plus the frozen donor map are saved, so every interval is reconstructible.

## Part B — paired output-head diagnostic

Frozen: `e_old`, `e_new`, row shifts, bounded ReLU/normalization, bias codes/scale, `F=10`. Only `w_o` trains. Both arms start from the **verified** legacy CPCK output masters (verified to re-quantize to the parent head) with fresh moments/age/schedule; step-zero parity asserted exact. Data: the recovered first 4,096 consumed windows / 257,113 targets, one saved seed-13 pass, 512 batch-eight updates, Adam (.9,.999), eps 1e-8, wd 0, lr 0.05, global clip 1.0.

| Arm (36-doc / 288-window / 17,342-target panel) | dev micro bits/target | gain vs parent | fit objective | fit teacher KL |
|---|---:|---:|---:|---:|
| frozen parent | 7.558429 | — | — | — |
| **empirical** conditional targets | **7.170816** | **+0.387614** [0.360308, 0.415469] | 6.802581 | 4.502363 |
| **smoothed** count-family targets | 7.188585 | +0.369845 [0.337491, 0.400210] | 7.196477 | 3.186145 |
| conditional floating head (offline only) | 6.874144 | +0.684285 [0.650749, 0.716343] | 6.877289 | — |

**Predeclared screen:** the practical ≥0.10 bits/target threshold **passes** (best arm +0.3876, paired lower bound 0.3603 > 0), but **smoothed-target attribution is FALSE** — Arm S does not beat Arm E on true-label development CE (0.0178 bits worse) while fitting its own teacher much better (KL 3.19 vs 4.50). Better teacher imitation is not language improvement.

**Conditional floating branch: RUN** (Arm S failed the combined screen and the teacher-fit gap 4.5024 > 0.10). On the same frozen features and soft targets it reaches soft objective 6.877289 / dev 6.874144 versus the smoothed hard arm's 7.196477 / 7.188585 — ~0.32 bits attributable to the ternary/dyadic output constraint and/or STE. Compared on the same objective, true-label population and dose. Offline comparator only, never a serving path. A finite optimizer run is not a certified optimum, so this does not separate ternary representability from STE difficulty.

## What the experiment establishes

- Refitting the frozen features' output head on the same dose recovers a large part of the local gap: +0.388 bits/target over the parent, against the sealed same-panel 512-window smoothed reference at 7.079256 (the refit head is 7.170816, i.e. ~0.09 above that reference), with full-fit 5.061160, 4,096-consumed 5.938052, unigram 9.185030, quantized bias 9.250170.
- **Smoothed distributional supervision does not help** beyond equally dosed empirical-CE refitting.
- The dominant measured limitation is the **frozen local feature map**: even a convex floating head on the same features sits ~1.8 bits above the full-fit reference with a teacher KL of 3.2–4.5 bits.

## Delivery, limits and next step

Fixtures: exact-table agreement/bijection, `CPX2` round-trip and malformed rejection, arm-aware selection, full-vocabulary reference argmax, analytical soft-target gradient vs central finite difference, one-hot/histogram/occurrence-weight identities, missing teacher mass detection, step-zero hard and floating parity, and a mid-pass continuation test (split run equals uninterrupted, next batch equal, wrong identity rejected). `cargo fmt --check`, offline check/release builds, `python3 scripts/check_claim_wording.py` all pass.

The superseded `readout-diagnostic-1` is preserved as a retained negative: byte-comparing the two archived `CPL2` exports caught a literal teacher flag in the arm loop before any result was reported; the corrected run is `readout-diagnostic-2`.

**One next step, from the observed bottleneck:** a local **representation** increase (wider hidden feature or a learned local interaction) under the same hard integer serving contract and dose, measured against the same panel and references. Count matching is not a prerequisite; no geometric-superiority, general-language or energy claim follows.

References #973; #820/#963/#964 remain open. Second local allowance extension recorded before the charge; ledger/JSON now **166,038,565 / 167,600,000 ms**.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

