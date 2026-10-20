# #1364 — Local prose deficit is output-side: a state probe of the served path (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T13:42:16Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-09-23T17:17:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1364
- **Merged:** 2026-09-23T17:17:54Z
- **Merge commit:** 91339b47d69e5b7a9ec0621e50cf9cad8574641b
- **Base / Head:** main / UOR-Foundation:codex/local-capacity-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Answers, by measurement, the question the previous milestone left open: is the shared model's ~1.5-bit deficit against the tuned two-token count reference a limit of the **served recurrent state** or of its **trained output layer**?

`References #820` and `References #973`. Nothing is closed.

**Stacked PR.** This branch's base is the head of open, unmerged PR #1363 (`67f2e350`), so its diff includes that PR's commits. It should merge **after** #1363. The new work here is one commit, `072e24fb`.

## What changed

A read-only diagnostic mode `--state-probe` in `crates/uor-r4-core/src/bin/ordinary-lexical.rs`. It walks every fit and dev window exactly as `TlModel::score_example` walks it and records the **exact state vector the served readout receives** at every scored prose position, then fits a float linear readout of that state, decodes `cur`/`prev` from it, and re-prices the fitted readout into the servable alphabets. The model source `learner/transferable_lexical.rs` is **unchanged** (`sha256 03f83eb8…b250`). The probe is a float instrument over recorded states; it is not a serving or product dependency and it modifies no artifact.

An independent **K3 architecture review** of the instrument corrected two errors that are load-bearing for reading the numbers: a readout linear in `h` cannot in general reach a full interpolated count table (its score matrix has rank ≤ `h_dim = 64`), so the decision rule is **`P` versus `A`**, not "does `P` approach the count reference"; and the probe's class-restricted softmax must be compared under the **same normalisation** as `A`.

## Result (K=1024, 4,954 of 5,376 development targets = 92.15% coverage; bits/target)

`A`, `E1`, `C` and `U` share the full-set normalisation and are directly comparable.

| quantity | bits/target |
| --- | ---: |
| `A` artifact's own served ternary readout | **6.055956** |
| `P_softmax` converged float linear readout of the **same input** | **5.5136** (top-1 0.2618) |
| `E1` tuned unigram + bigram(`cur`) count | **5.565123** |
| `C` tuned `(prev, cur)` count | **4.784593** |
| `U` fit-only unigram | 8.669572 |
| `P − A` | **−0.5423** |
| `P − E1` | −0.0515 |
| `P − C` | +0.7290 |

The refinement is at its floor: the last epoch moved the fit loss by **0.0119 bits/target**. Allowing the probe's class-restriction convention at most the ~0.14 bits it can buy, the served output layer is losing **≥0.40 bits** that a converged readout of the very same input recovers.

**State content**, by a *training-free* nearest-embedding rule over the artifact's own 4,096 rows (4,228 decodable dev positions): `cur = x_(t-1)` **0.9674** top-1 (majority 0.0381, chance 0.0087); `prev = x_(t-2)` **0.0206**; the generated target `x_(t+1)` 0.0201. A ridge decode agrees on the ordering (`cur` 4.0495 bits/0.5333 vs `prev` 6.4473/0.2838). **The state carries the last token and not the one before it.**

**Decomposition of the restricted gap to `C`:** about **0.54 bits (≥0.40 after the allowance) is output-side and recoverable on the same state**; about **0.73 bits is second-order structure** the state does not supply to a linear readout. Consequently the previous milestone's two-regime plateau at ≈6.69 (`olx-form-2` 6.6841, `olx-form-5` 6.6903) is a shared **output-layer** ceiling at this state, **not** two independent confirmations of a recurrence limit.

## Recorded negatives

- **Post-hoc re-pricing of a fitted readout into the servable alphabets is destructive**: 8.6058 (ternary, fitted per-row scales), 7.6181 (ternary + offsets), 8.9683 (4-bit) — all far worse than the artifact's own in-class-trained 6.055956. Naive quantisation is not a valid class ceiling; only in-class training can judge what the served class expresses.
- **An unresolved initialisation discrepancy**: the artifact-initialised probe's epoch-0 dev loss is 6.5954, which exceeds `A` = 6.055956 — impossible under the intended mapping, since renormalising from the full legal set onto the class set must lower the loss, and the initialisation's logits were verified to match the artifact's readout (`worst absolute deviation 0.0`). Nothing in this PR is built on the epoch-0 value or on the epoch-0-to-refined drop.

## Controls

Recorded per-position bits equal `score_example`'s own bits, and the recorded state count equals its scored Generate targets (334,796 fit, 5,376 dev). Every recorded state carries its window's `(cur, prev)` — checked on all 340,172 positions. The permuted-label null probe sits at the restricted unigram (`olx-probe-3`: 8.5662 vs 8.5264, gap +0.040; K=64 smoke +0.008), `leaking: false`. 6 runner unit tests and 12 `transferable_lexical` tests pass; `cargo fmt --check` and the offline release build pass; `scripts/check_claim_wording.py` passes; all links in the edited documents resolve.

## Limits

Scoped to `olx-form-2`, the served window conditioning, teacher-forced states, one corpus/split/seed, and the top-1024 target population (92.15% coverage). The probe is a *linear* readout, so its loss upper-bounds what a linear readout extracts and bounds nothing for nonlinear readouts. No per-document cluster interval is computed for probe quantities; the comparison is paired by position and the measured matched-arm noise scale on this population is 0.03–0.08 bits, so a ≥0.40-bit effect is several times that. The state-side negative applies to **first-order** local retention only. Greedy generation still collapses; sampling NOT_RUN; energy UNAVAILABLE. No general-language, reasoning or geometric-advantage claim.

## Resources

Machine time 5,864,000 ms (two full probe runs, four sealed smokes, an interrupted attempt, builds/tests); complete milestone charged **19,500,000 ms**, which exceeded two prospective projections by 12,000,000 ms because two measured instrument defects forced extra rounds and the independent review changed the instrument twice after implementation had begun — the lesson (sequence the review before implementation; design for probe convergence) is recorded in the ledger. Owner ledger: cumulative 414,715,640 / limit 416,000,000 ms, leaving 1,284,360 ms. Physical free 53,817,400 KiB, above the 24 GiB reserve plus the 128 MiB stop margin. No paid or external compute; no unique artifact or preserved root modified.

## New and preserved

New sealed roots: `olx-probe-3`, `olx-probe-4` (plus four `/tmp/olx-sp-*` smokes and the interrupted `olx-probe-1/2`, all preserved). Untouched: `ordinary-lexical-3`, `olx-channel-1`, `olx-audit-1..5`, `olx-form-1..5`, and the owner checkout.

## One evidence-supported next milestone

Freeze the recurrence and the embedding table, and train the served readout **in class** on the frozen-state data — the level the converged float readout already demonstrates is available — keeping the grounded Copy/Stop responsibility and the served window conditioning. Pre-declare the acceptance before the fit: served loss below `olx-form-2`'s **6.6841** by ≥**0.30 bits** with preflight A 32/32, held-out 3/3 and class 4/4 retained, and the epoch-0 mapping reconciled. In-class training, not post-hoc re-pricing, is the only valid route. If it cannot recover the headroom, the next branch is explicit second-order local access, since the state does not retain `prev`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

