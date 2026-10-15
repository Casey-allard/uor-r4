# #1304 — Learned older-prefix group-state/read pilot: executed, screen FAILED (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T05:41:08Z
- **Updated:** 2026-10-09T13:17:44Z
- **Closed:** 2026-09-20T05:43:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1304
- **Merged:** 2026-09-20T05:43:27Z
- **Merge commit:** 88e7122122717333aed7ee79049854337cf60ead
- **Base / Head:** main / UOR-Foundation:codex/ordered-prefix-state-pilot
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the complete second document of the ordered-prefix pair. The retained step-512 prior is **never retrained**; every sealed root is preserved.

## Component: exact bounded older-prefix group-state/read channel

- Exact `2I` (order 120) product table already in the repo; fixed, data-independent palette `[1,2,56,3,59,84,85,0]` (identity 1, full closure verified, chosen by deterministic search for the first noncommuting generator pair — no targets consulted).
- Served forward is a **hard integer kernel**: `Z_total[v] = Z_parent[v] + ((Σ_j Wcode[v,j]·Rcode[q,j]) << 8)`, implemented as `h[j] = Rcode[q,j] << 5` then a ternary `W` at output shift 3. Fixed-scale packing via `TernaryLinear::from_packed` — the adaptive `quantize` is never used, so training, export/reload and generation execute one identical forward.
- State is `q_i = Π_{j∈[max(0,i-63), i-1)} γ[argmax A[x[j]]]` (half-open; **excludes** the local pair). Empty older prefix → zero residual. Step 0 reproduces the parent's integer scores exactly. Declared parent+residual envelope validated against `i32::MAX/2` before execution.
- Offline credit uses the biased categorical STE surrogate with the derived state/palette adjoints, separate Adam ages (`action` age starts at its first post-warm-up update), reader-only warm-up, master clamp to [-1,1], and a CPXS checkpoint with complete mid-pass resume (distinct from the legacy CPCK).

## Executed three-arm pilot

Dose: the first 512 entries of the recovered 4,096-window consumed permutation (32,046 targets), 256 batch-8 updates per arm after 64 reader-only warm-up updates, one seed, identical initialization and fit order.

| Arm | panel micro bits/target | document-macro | gain vs frozen parent |
|---|---:|---:|---:|
| frozen parent (state-disabled parity exact) | 7.558429 | 7.55617 | — |
| learned older-prefix (**primary**, step 256) | 7.575600 | 7.57275 | **−0.017174** [−0.026095, −0.008483] |
| fixed-action older-prefix | 7.723500 | 7.72079 | −0.165066 [−0.177712, −0.152226] |
| learned local tail | 7.576440 | 7.57338 | −0.018010 [−0.034886, −0.001074] |

Primary vs fixed control **+0.147892** [0.135512, 0.161202]; primary vs local-tail control **+0.000835** [−0.016240, +0.020440].

Panel: all 36 dev documents, 288 evenly spaced windows, 17,342 targets, frozen before any score; 32 of 36 documents overlap the legacy panel (different windows).

## Predeclared screen: FAILED

- ≥0.10 bits/target over the frozen parent with interval excluding zero — **FAILED** (point −0.0172, interval entirely negative).
- Positive paired gain over **both** fitted controls — **FAILED** (beats fixed-action, not the local-tail control).
- Positive conditional-permutation penalty with interval excluding zero — **FAILED** (−0.0015, interval includes zero).
- Disabled parity (exact) and eligibility support (36 docs / 1,177 targets) — passed.

**Actions were genuinely learned**: 1,083 hard transitions of 4,096 tokens, all 8 palette IDs occupied, post-warm-up action-gradient norm 0.596. So the negative is not a no-learning artefact.

## Controls

- State-disabled/zero residual reproduces parent integer logits exactly.
- Conditional permutation within exact `(prev, cur, older-prefix length)`, donors free to cross documents, targets/local inputs fixed: 16,583 strata, 576 no-history excluded, 1,177 eligible records / 36 documents, 554 changed-state recipients. Full penalty −0.000104 [−0.001112, +0.000803]; eligible penalty −0.001532 [−0.015194, +0.012677]. Exact conditioning leaves only ~7% of history records permutable — the intervention is near its support floor.
- Reverse older-prefix order: +0.004350 [−0.002924, +0.011642] (sensitivity only).
- References on the same occurrences: unigram 9.18503; quantized bias 9.25017; order-2 full-fit 5.06116; 4,096-consumed 5.93805; **512-window 7.07926**. A count reference on the *same* 512-window dose beats the frozen parent by **0.479** bits/target.

## Generation

Parent reproduces the three historical legacy outputs exactly. Reporting distinguishes sufficient states: `pair` is a cycle certificate only for the two-token parent, `ring` (ordered last ≤64 tokens) for the new arms. Only the all-32 prompt witnesses a ring cycle (entry 48, period 1); every other arm trajectory shows repeated pairs with **no ring cycle** — observed repetition without a proven cycle. All outputs/bytes kept; no penalty, sampling, blacklist or override added.

## Derived-metadata corrections (separate sealed root; sealed replay untouched)

`--derived-corrections` writes a new sealed root: `token_offset = window_index * 64` corrects **38,152** of 38,987 offsets (835 already correct), with boundary verification across two documents against pinned tokenization; pair labels corrected (`(32,32)→32` preserved as a pair fixed point; `(284,198)→198` is immediate repetition with successor `(198,198)→504`); tune pool 38 vs 32 actually used, seed `0xA5A51234`/effective `0xA5A51235`, historical "eligible" = changed-context records, and the 60-toy-update parity fixture are recorded.

## Validation executed

`cargo fmt -p uor-r4-core -- --check` (clean); offline check + release builds of the changed binaries; 12 focused `prefix_state::` fixtures (palette closure, residual scale identity, four-token older-prefix trace, tail product, finite-differenced state/palette adjoints vs the continuous multilinear extension, step-0 parent parity, warm-up action-credit gating, fixed-action immobility, two-arm warm-up agreement, checkpoint round-trip + split-run resume equality + wrong-parent rejection); 7 `prior-frozen-evaluate` fixtures; probe then two pilot runs (185.3 s / 186.7 s, peak RSS 545 MB); derived corrections 0.3 s; `python3 scripts/check_claim_wording.py` passed.

## What this establishes, and the one next step

Older-token information carried by one 120-state register is **not measurably used** under this palette/surrogate/width/dose; there is no advantage over equally sized added local capacity; learned actions help only against a frozen carrier. The dominant same-input gap is **local**: a count table beats the parent by 0.479 bits at the same 512-window dose while 65,536 added output coefficients make out-of-sample loss worse. The next bounded step is therefore a **local-readout repair** — fit the existing two-token ternary readout toward the interpolated order-2 count distribution on the recovered 4,096-window dose under the same hard integer forward, and require it to approach the same-dose count reference before testing further context state. `2I`-vs-other-algebra superiority remains NOT_TESTED; no general-language, noncommutative-advantage or energy claim.

References #973; #820/#963/#964 remain open. Projection (+5,400,000 ms), extension and charges recorded before use in the shared ledger and JSON: **159,438,565 / 159,800,000 ms**.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

