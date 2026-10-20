# #1376 — Matched span curriculum and the length-specific copy/stop gap (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T04:37:18Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-09-24T04:41:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1376
- **Merged:** 2026-09-24T04:41:50Z
- **Merge commit:** f95b9efbe87d7533600be41ca8c3a6a80994c094
- **Base / Head:** main / UOR-Foundation:codex/matched-span-curriculum-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

Completes the prior milestone's declared next step on
[`language-transport-result-2026-09-24.md`](docs/integration/language-transport-result-2026-09-24.md):
implement the declared **separate span RNG** so `max_span ∈ {3,8}` arms are exactly data-matched, and isolate the
**multi-token copy/stop gap**. Full executed record:
[`docs/integration/matched-span-copy-length-result-2026-09-24.md`](docs/integration/matched-span-copy-length-result-2026-09-24.md).
Evidence root `/Users/casey.allard/uor-r4-investigations/matched-span-20260924` (19 sealed attempts; not in a clone).

## Code change (runner + cursor format only)

- `TrainingCursor` gains `span_rng_state` with a nonzero `serde(default)`; legacy checkpoints still deserialize.
- The span block draws only from `span_rng_state` and consumes exactly `1 + 2·SPAN_POOL` draws per example, so the
  non-span stream is identical across arms; new `UOR_LANGUAGE_SPAN_DRAW_MAX` separates the declared cap from the
  draw. New `UOR_LANGUAGE_COPY_SET ∈ {base,two,three,four}` (matched 32-value 2/3/4-token additions); frozen
  held-out `EVAL_PANEL`; read-only `UOR_LANGUAGE_MODE=diagnose`.
- Receipts bind `runner_source`, `checkpoint_source`, `causal_runner_source`, `parent_checkpoint_source`,
  `eval_panel_sha256`, `span_pool`, `span_seed`, `copy_set` and `copy_ranges` (closes the prior binding gap).
- `transferable_lexical.rs` (`59801a61…4956e`) and `tl_execution.rs` (`71e7888b…06ed`) are **byte-identical** to
  `origin/main`. No served-model arithmetic, D0-b kernel or compiled path change.

## Measured result

- **Matching witness:** `span8m3` (max_span=8, draw=3) and `span3` (max_span=3, draw=3) produce **byte-identical
  `final.tlx`** (`aab0b63f…`) — the non-span stream, span pool and realised spans are matched and the declared cap
  does not leak into the batch.
- **H1 (matched):** with matched data, `span8` beats `span3` on the long cells (length 5: 10 vs 6; length 8: 13 vs
  7), fresh-span (40/48 vs 12/48) and `repository_bits` (6.464870 vs 6.550110); all four historical gates retained.
- **Mechanism:** a read-only diagnostic finds the prior `moments-8` artifact copies 3–4 tokens correctly but leaves
  the Stop row below the vocabulary maximum at every L≥3 decision (`stop_margin` −20…−336) with `h` saturation
  exactly `0` — a missing terminal Stop, not a copy or state-saturation failure.
- **Length-specific curriculum:** matched 64-value pools at max_span=3 show `add3` (3-token) 34/54 and `add4`
  (4-token) 19/54 against the control 14/54; `add3` fixes 3-token rows and `add4` fixes 4-token rows with no
  cross-transfer, refuting a "mere extra steps/volume" account. `integrate` moves 16/64 → 28/64.
- **Cost:** `repository_bits` regresses `+0.2148` (`add3`) / `+0.2683` (`add4`) beyond the pre-declared `+0.05`
  tolerance; the extended curriculum is a **scoped component with a measured prose cost**, not a promoted artifact.
- Native/compiled parity holds (`cost-add3`: 156 identical rollouts; 11.43×; plan 366,654 B).

## Verification

`cargo fmt --check` clean; offline release build of `ordinary-lexical`; `cargo test --lib
transferable_lexical::checkpoint::continuation_tests` 3 passed; `cargo test --bin ordinary-lexical` 29 passed /
1 ignored; `python3 scripts/check_claim_wording.py` passed. An independent evidence audit confirmed the matching
witness, H1 comparison, mechanism/length-specificity, disjointness and receipt binding from the sealed roots, and
recorded two caveats (the `+0.05` criterion is fixed in the pre-run design rather than machine-sealed; the
`integrate` parity is a runner assertion rather than a receipt counter) — both recorded in the result doc.

## Limits

Bounded authored copy/stop-length and forced-feedback span panels on one 466,711-byte artifact; single seed; no
fresh sealed acceptance draw. **No** general language, copying, dialogue, executed Rust, geometric-superiority,
reasoning or energy claim follows.

## Next

Reduce the prose cost of the length-specific Stop fix with one bounded matched arm that keeps the `add3` copy/stop
endpoint while restoring `repository_bits` within `+0.05`, then broaden-source dialogue/code and complete-session
work.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

