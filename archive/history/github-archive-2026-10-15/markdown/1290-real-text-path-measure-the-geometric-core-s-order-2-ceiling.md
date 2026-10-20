# #1290 — Real-text path: measure the geometric core's order-2 ceiling before spending the training block

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T20:58:43Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-19T20:59:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1290
- **Merged:** 2026-09-19T20:59:45Z
- **Merge commit:** 127c0eafc8c33cfb8476e3b9514d67fd9b550066
- **Base / Head:** main / UOR-Foundation:codex/realtext-mechanism-ceiling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

The funded next block is "train the geometric core at `V=4096`, `dv=128` for ~2,000 steps" on real text (~65 min projected, drawn from the already-recorded 2-hour real-text projection). Project trap #2 requires testing the **reduced form** of an idea before building it. This measures what that training could *possibly* achieve, before the budget is spent.

## Why the reduced form is decisive here

`GeometricAttention`'s addressing is fixed and not learned:

- `element_table(vocab)` is `(0..vocab).map(|t| (t % RADIX) as u16)` — `learner/geometric_attention.rs`
- `from_f32` and `GeometricAttentionTrainer::new` both set `elements` from it
- `from_f32` rejects `order` outside `{1, 2}`

So at `order = 2` the entire prediction context is `(t-2 mod 120, t-1 mod 120)` — a `120^2 = 14,400`-class coalescing. The best any predictor can do from that context is an upper **ceiling** on the trained model, which uses ternary weights, a normalised read, and `dv=128` value vectors over a 4096-token readout.

## Change

New `crates/uor-r4-core/src/bin/geometric-realtext-ceiling.rs`:

- derives a 4096-vocabulary tokenizer from the local `tokenizer.json` through the existing verified `HfBpeTokenizer` parser (dense prefix + surviving merges), rejecting non-dense or over-large requests;
- tokenizes real text and measures unigram / residue / true-token context models with **Jelinek-Mercer interpolation tuned on a validation split**;
- controls: monotonicity, a **shuffled corpus refit end to end**, printed observation counts, and a sha256-pinned corpus digest.

## Measured

| corpus | unigram | res1 | **res2 (mechanism)** | true1 | true2 |
|---|---|---|---|---|---|
| repo source | 8.7520 | 6.5785 | **5.0019** | 5.1736 | 4.1573 |
| repo docs | 9.0957 | 7.3446 | **5.9546** | 5.7979 | 4.8955 |

(bits/token; bits/byte `res2` = **1.9847** / **2.3759**)

The order-2 residue context recovers **81.6 % / 74.8 %** of the true order-2 gain over unigram. Shuffled-corpus control gains are **exactly 0.0000** in every case.

**A ceiling, not a model result.** The trained model can only be worse; the gap is unmeasured.

## Two estimator defects caught by the controls

A first add-1 estimator was **invalid**: it made the *true* bigram score worse than order-1 (7.5891 vs 6.2749 bits/token) and reported the residue context as recovering **"102.28 %"** of the true gain — a false positive *for* the mechanism. Its shuffle control also compared a real-trained model against shuffled text instead of refitting. Both are fixed. This is the third session in a row where a measurement defect initially looked like a mechanism result.

## Consequence

The result **contradicts the structural prior** that a `% 120` coalescing destroys the context, so the block is **not** killed — it is given a target band. Cause is hypothesised (frequency skew), not measured.

The receipt also records a **cost re-projection from the project's own op counts**: 524,288 ops/token readout at `dv=128` x 4.096e6 tokens = 2.15e12 forward-readout operations alone, so the recorded 1,800 s implicitly assumes ~1.2e9 scalar ops/s sustained with no backward pass, Adam, write path or state clear. The run is plausibly **3-10x the projection**; a **measured step-time probe should precede spending the block**. That is arithmetic, not a timing measurement.

## Limits

Ceiling only, mechanism **as configured** (fixed `element = token % 120`, `order <= 2`) — learned packaging is untested and this is not a verdict on the architecture. No instruction corpus exists locally, so repository source and Markdown prose were used; the direction is the same on both and the corpora are pinned by digest. Bits/byte is **not** comparable to the shipped artifact's 1.7989 BPB (different corpus/tokenizer/model). No chat, generation or language ability claimed; energy/token UNAVAILABLE; no serving-multiplier claim for the new module.

## Validation

- `cargo fmt --all --check` — clean
- `cargo test --release -p uor-r4-core --bin geometric-realtext-ceiling` — **3 passed** (truncation keeps exactly the dense prefix and its merges; over-large request rejected; non-dense prefix rejected)
- `python3 scripts/check_claim_wording.py` — passed

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

