# #1300 — First corrected real-text prior curve: executed; the step-512 gate passes on the corrected sign (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T03:44:54Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-20T03:45:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1300
- **Merged:** 2026-09-20T03:45:55Z
- **Merge commit:** 74fef0886ca3b14ff90943c8677d6b7815064198
- **Base / Head:** main / UOR-Foundation:codex/realtext-prior
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Thin real-text runner plus the section-2 primitive repairs, **executed end to end in 609.6 s**. The corrected small gate from #1298 is preserved unchanged; no synthetic campaign was restarted and no floating comparator was needed.

## Result: the exported low-bit prior learns context beyond the marginal

| step | dev bits/target (micro = macro) | bias-only | exact unigram | permuted |
|---:|---:|---:|---:|---:|
| 0 | 9.1316 | 9.1316 | 9.0599 | 9.1364 |
| 256 | 8.2182 | 9.1316 | 9.0599 | 9.3122 |
| 512 | **7.1425** | 9.1316 | 9.0599 | 10.0168 |

Fit 355 documents / 38,987 windows / **2,446,208 scored targets**; development 36 documents / 96 windows / **6,048 scored targets**.

Step 0 is **exactly** the frozen quantized constant — the zero-exported-residual initialization verified on the real-text path — and sits 0.0717 bits *worse* than the exact unigram, which is precisely the quantized-vs-exact marginal gap.

## Step-512 gate (corrected sign)

`paired_interval(a, b)` returns `a − b`; the runner passed `(ctx, reference)`, so the printed "gain" carried the model's sign backwards and the verdict compared the wrong quantity. **The retained numbers are unaffected**; corrected gains are the negation:

| comparison | printed | **corrected gain** | paired 95% interval | excludes zero |
|---|---:|---:|---|---|
| exact fit-only unigram | −1.9174 | **+1.9174** | [+1.8073, +2.0321] | yes, improving |
| identical quantized constant | −1.9892 | **+1.9892** | [+1.8822, +2.1030] | yes, improving |
| permutation penalty | +2.8744 | +2.8744 | [+2.7273, +3.0290] | yes, positive |

All three frozen criteria are met at 512 steps, at roughly nineteen times the required 0.10-bit margin. The permutation is a genuine negative control: **5,805** context-target associations changed with the context marginal and target list preserved (the PAD stratum is non-permutable and is reported, not dropped). Position knockouts cost 0.4347 and 1.3572 bits, so both inputs matter, the current token more.

## Generation collapses — reported, not filtered

Every prefix at every checkpoint produces a short locally plausible continuation followed by a repeated token (`,,,,,`, `000000`, `\n\n\n`). Full prompt/output IDs and decoded bytes are in `generation.json`. Loss improvement is not generation quality; no fluent-prose claim is made.

## Section-2 primitive repairs, all tested

- **Integer-only trace**: `Trace` stores the ReLU mask as `u8` and the shift as `u32`; float conversion and inverse-shift credit happen only in the offline backward, so the greedy path carries no floating auxiliary value.
- **Checked embedding sum** in i64 before the clamp; `norm_bits = 0` bounded by `PRIOR_CLAMP` instead of an assumed unit bound.
- **Shared validation**: `PriorCore::validate` applies the loader's checks to constructed cores too.
- **Exact continuation**: checkpoint v3 persists and validates the Fisher–Yates permutation (`perm` was previously omitted and then cleared); mid-pass, partial-batch and cross-pass continuation are tested against the permutation, cursor, masters, both moments, consumed IDs, the next batch, the next update and the exported cores, with nondefault betas and weight decay. Rejected loads leave the trainer unchanged.
- **Hand-computed STE fixture** for both embedding tables and the output, asserting active *and* clamped ReLU coordinates, a nonunit normalization shift, a nonflat bias and a nonzero residual.

`prior_learning` **12 tests pass**; `cargo fmt --all --check` clean; claim-wording gate passes.

## Deviations and NOT RUN

The prompt's **4 MiB selection bound was not applied** — the pinned tree is ~86 MB and all eligible whole documents were used, so this is not a bounded 4 MiB pilot. **NOT RUN**: corrected interpolated backoff references, the unseen-fit-pair stratum, an on-disk mid-run continuation, peak RSS, and any binary-level instruction inspection.

## Retained identities

Report root `.uor-models/realtext-prior-2026-09-20/attempt-1` (claimed exclusively, sealed, verified 0 unlisted). `prior_realtext.cpl2` 454,788 B `sha256:cd5a3aa1…`; `prior_realtext.ckpt` 19,192,104 B `sha256:1c3dfedb…`; tokenizer derived `sha256:a7ac75b6…`; source revision `e9c04e80`.

Ledger 1,200,000 ms measured → `152,238,565 / 154,400,000`. Development documents are correlated repository prose, so the intervals are nominal screening intervals, not final held-out qualification; the modulo-120 table remains metadata and no geometric, prose, coding or energy claim follows. Energy UNAVAILABLE.

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

