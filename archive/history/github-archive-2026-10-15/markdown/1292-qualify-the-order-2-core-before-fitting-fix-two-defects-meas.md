# #1292 — Qualify the order-2 core before fitting: fix two defects, measure route coverage and step cost (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T23:23:31Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-19T23:24:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1292
- **Merged:** 2026-09-19T23:24:37Z
- **Merge commit:** 5b5bc8f5aa842f73f635cdc7f3b4f5531d7c3622
- **Base / Head:** main / UOR-Foundation:codex/realtext-qualification
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Bounded **#973** tranche: qualify the order-2 geometric core before any substantial fit, then decide.

## Result: do not train this configuration

The task's own frozen failure branch was *"if the cold-route loss floor already exceeds the frozen development target, or useful reads are too rare to train the intended function, stop scaling this configuration."* Both conditions hold on both corpora.

| | docs (prose) | crates (source) |
|---|---:|---:|
| `p_empty`, 64-token windows | **0.9381** | **0.8549** |
| floor `p_empty · log2(4096)` bits/token | **11.2575** | **10.2586** |
| unigram baseline bits/token | 9.2223 | 8.8373 |
| best static backoff baseline bits/token | 6.3310 | 5.0402 |
| live routes | 6.2 % | 14.5 % |

An empty route is the zero vector, the default readout has no bias, so its loss is exactly `log2(4096) = 12` bits/token. Widening to 128/256-token windows only reaches 10.7664/10.1510. The first eight positions of every window are cold with probability ≈0.994, and the routes that *are* written are near-unique (distinct exact contexts per address ≈1.004). Vector cancellation was 0/388 under a **named, seeded** initialisation — explicitly not a trained result. Independent corroboration: the untrained trainer's held-out loss is 12.27 bits/token.

## Two source defects confirmed and fixed

1. **Short-prefix underflow.** `tail_word` / `word_at` / `sequence_loss` computed `len + k - order` and `i + k + 1 - order` on `usize`: a panic under overflow checks, a silent wrap in release. One checked padding contract (`PAD_TOKEN`, `word_ending_at`, `tail_word_of`) now serves the served path, the loss and the trainer, with empty / 1-token / order-2 fixtures that run under overflow checks.
2. **Double length normalization in the gradients.** `g = p * inv` at the readout already converts the summed loss to the mean; `gkernel += acc * inv` and `gwv += dscratch * inv` applied `inv` again — value and kernel gradients were `1/(n-1)` too small (63× at n=64). Removed, and verified against **two independent analytic references** that never call the trainer's backward pass and never finite-difference the quantised step function: the order-2 exact read (value gradient as an explicit suffix sum) and the order-1 kernel convolution. Both required non-vacuous.

## Measured behavioural effect of the fix — recorded, not smoothed

The gradient fix also breaks two **pre-existing** synthetic filter fixtures at their recorded seeds: `corruption_in_the_objective_changes_the_filter` 0.13 → 0.10 and `a_general_filter_is_at_least_as_good_as_a_class_filter` 0.27 → 0.09. They pass again only after re-tuning their learning rate (0.05 → 0.15 and 0.05 → 0.5, the fix having changed the per-group gradient scale), and at those rates the general-filter margin is **0.11 vs 0.09 against a 0.02 tolerance — a tie, not the previously recorded "does not lose"**. The earlier filter accuracies are superseded at their recorded scope and the general-filter capacity claim is **not** validated at that budget.

## Timing measured, replacing both unmeasured figures

One `train_batch` at `V=4096`, `dv=128`, `order=2`, batch 8 × window 64 covers both full-table quantizations, the touched-bucket reset, forward, backward and Adam: **cold 0.5602 s, warm 0.5649 s (range 0.5608–0.5729), 512 tokens/step, 906 tokens/s, peak RSS 56.3 MiB**. The recorded 1,800 s projection assumed ~2,278 tokens/s and is ~2.5× optimistic; the later "3–10× slower" arithmetic was pessimistic.

## Count instrument repaired without erasing prior output

Relabelled a **static backoff comparison**, not a ceiling: the residues select the bucket, but the bucket holds prefix-accumulated values, so `[0,1,2,0,1]` and `[0,1,3,0,1]` share an address yet differ — the earlier "the entire context is the residue pair" claim is withdrawn. CE and top-1 now come from the **same** interpolated distribution, with a `lambda = 0` fixture reproducing the unigram probability *and* argmax; the argmax is computed once at the final weights rather than inside the tuning sweep; and the protocol is explicit — **counts fit on the fit split only, weights tuned on the disjoint validation split, no full-training refit**.

## What is missing, named

A **learned cold-context prior**: the reset state holds no corpus statistics, so a cold route cannot fall back to anything, while the count baselines carry global priors the empty bucket does not. The proposed smallest intervention is a learned cold-context prediction trained through the same discrete path with an explicit fallback marker and matched ablations — **not** more `dv` or steps, because an injective map from 4,096 tokens to 120 single roots is impossible. The `order`-2 radix read still exercises no group multiplication, zeta phase or chirality, and is not evidence about the distinctive geometry.

## Validation

- `cargo fmt --all --check` — clean
- `python3 scripts/check_claim_wording.py` — passed
- `cargo test -p uor-r4-core --lib -- geometric_attention` — **40 passed, 0 failed**
- New fixtures for padding, causality, prefix sensitivity, empty buckets, document reset, `blame`-free batch-vs-incremental traces, and the two independent gradient references

**No pilot was run and no model artifact was created.** Receipt: `docs/evidence/native_geometric_causal_qualification_2026-09-19.txt`. Ledger: 1,200,000 ms charged (measured) → `147,638,565 / 154,400,000`, ~112.7 min remaining; 128 MiB storage margin untouched.

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

