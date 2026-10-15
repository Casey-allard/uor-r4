# #1302 — Corrected evaluation-only replay of the frozen real-text prior (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T04:39:15Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-20T04:40:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1302
- **Merged:** 2026-09-20T04:40:45Z
- **Merge commit:** 3095c1d48e213deb234d31d21c94da7adcca7e6e
- **Base / Head:** main / UOR-Foundation:codex/frozen-prior-evaluation-replay
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the complete evaluation-only task in `docs/integration/deepseek-frozen-prior-step-2026-09-20.md` (delivered by #1301). The retained step-512 `prior_realtext.cpl2` (SHA256 `cd5a3aa1...`) is loaded **unchanged**. No weights, seeds, decoder, corpus or stored result changed; no `PriorTrainer` was constructed and no optimizer step ran.

## What was defective and what is repaired

1. **Occurrence-collapsing permutation.** The legacy map shuffled target values into `HashMap<(doc,prev,cur),target>`, overwriting repeated occurrences and changing target frequencies. Replaced by a seeded Fisher-Yates bijection per `(document, PAD/non-PAD)` stratum over occurrence **indices**; recipients keep their target and draw a donor context. Invariants asserted: bijection over occurrences, context multiset preserved, target vector untouched, observation identity `(content_id, window_token_offset, target_offset)`.
2. **Window-level aggregation labelled as document aggregation.** Every scorer is now summed per true document cluster before macro/interval, and the 2,000-draw paired bootstrap resamples documents and recomputes ratio-of-resampled-sums.

## Executed results

| | legacy panel (32 docs / 6,048 targets) | spread-position panel (36 docs / 72 windows / 3,734 targets) |
|---|---:|---:|
| contextual model | 7.142472 | 7.47284 |
| frozen quantized bias | 9.131630 | 9.30911 |
| exact full-fit unigram | 9.059915 | 9.24383 |
| interpolated order-2 reference (full-fit / consumed counts) | 3.969071 / 5.196450 | 4.33330 / 5.45742 |
| unigram gain | **+1.9174** [1.7744, 2.0641] | **+1.7710** [1.6445, 1.9033] |
| bias gain | **+1.9892** [1.8439, 2.1271] | **+1.8363** [1.7011, 1.9712] |
| permutation penalty (full / eligible) | +3.0572 / **+3.0858** [2.8761, 3.2834] | +2.8367 / **+2.9012** [2.7355, 3.0808] |
| decision | **PASS** | **PASS** |

Reproduction is asserted first: the corrected replay reproduces the recorded step-512 micro losses to ~1e-15 bits/target and the reconstructed full-fit unigram reproduces the artifact's frozen bias codes exactly.

**Falsifying fixtures all pass**, including the constant-predictor invariance the legacy instrument failed (bias-only CE exactly 9.131630 original vs permuted, max per-occurrence delta 0.0) and the unequal-document-weight aggregation fixture.

**Exposure recovered:** CPCK v3 step 512, cursor 4096, pass 0, permutation length 38,987 (all distinct); the consumed 4,096 windows sum to **257,113** `n-1` targets against 2,446,208 full-fit targets.

**Loops:** all six trajectories (the three legacy prefixes — verified to be one document — plus three genuinely distinct spread documents) enter period-1 cycles within <=11 steps. `argmax Z(32,32)=32` is a real fixed point (margin 1,024, no ties) *supported by local counts* (full-fit 1,947/6,099; consumed 246/725) and selected by the reference too; `(284,198)` is a model-specific divergence from the reference (198 vs 80). No repetition penalty, blacklist, sampler change or output override was added.

## Primitive corrections with parity

- `prior_learning.rs`: the asserted embedding sum uses signed shifts instead of integer multiplication by a power of two (value-preserving for ternary weights).
- `PriorCore::core_view` now calls `PriorCore::validate` on constructed cores as well as loaded ones.
- A 12-context x 2-mode integer-score fixture (full `z`-vector digests) plus 7 greedy prompts including the recorded loop states is **byte-identical** before and after the edits, on the unchanged artifact (`parity-before` fixture `9bafc440...`, `parity-after` `identical=true`).

## Claims preserved / withdrawn

Withdrawn (unchanged from the #1301 review): complete-gate PASS, the 2.8744-bit penalty and its interval, the 5,805 applied-association count, "36 evaluated documents", "three distinct generation documents". Preserved: the positive loss improvement at step 512 and every negative generation. Steps 0/256 were never persisted as artifacts and are not replayed.

## Retained roots

`/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/eval-replay-2` (14 sealed members, zero unlisted; `result.json` SHA256 `6db96a1c...`), plus `parity-before` and `parity-after`. The original sealed `attempt-1` is untouched.

## Validation actually executed

`cargo fmt -p uor-r4-core -- --check` (clean); offline `cargo check` and release build of the changed binary; `cargo test --bin prior-frozen-evaluate` (7 passed); five individually named non-training lib tests (`uniform_integer_scores_are_exactly_twelve_bits_at_v4096`, `a_common_offset_changes_nothing_and_extreme_errors_are_uncapped`, `the_frozen_bias_reproduces_the_fit_unigram_up_to_quantization`, `artifact_roundtrip_is_exact_and_malformed_inputs_are_rejected`, `training_and_serving_scores_are_the_same_computation`) — all ok; the broad `prior_learning` filter was deliberately **not** used because it contains Adam update loops; `python3 scripts/check_claim_wording.py` passed. The replay ran in 52.5 s at 283 MB peak RSS.

## Next mechanism (one, evidence-supported)

The corrected gains survive across positions, so retain this artifact as the local frozen baseline and specify the smallest **one learned ordered prefix-state/read channel** (equal-tail/different-prefix, state-disabled, order and matched capacity/cost controls), using the order-2 interpolated count reference as the comparator — precisely because the current two-token learned readout is dominated by that reference on the *same* input (7.1425 vs 3.9691). No new state/decoder is implemented here; exact occurrence/version memory stays distinct from coalescing modulo buckets.

References #973; #820/#963/#964 remain open. Projection/extension/charges recorded in the resource ledger before use.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

