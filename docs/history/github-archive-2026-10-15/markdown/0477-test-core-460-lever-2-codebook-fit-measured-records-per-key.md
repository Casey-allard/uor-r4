# #477 — test(core): #460 lever 2 — codebook fit measured; records-per-key is a symptom, not the binding quantity

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T11:06:18Z
- **Updated:** 2026-08-07T11:36:03Z
- **Closed:** 2026-08-07T11:30:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/477
- **Merged:** 2026-08-07T11:30:24Z
- **Merge commit:** 2bccf2fb26efd65cce48444c351132409595821d
- **Base / Head:** main / UOR-Foundation:measure/460-codebook-fit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #460 (lever 2 of the rescoped issue; lever 1, `cover_scaling.rs`, is untouched and still dark).

I took lever 2 ahead of your recommended lever 1 — reasoning is in the run contract on the issue. Short version: the graph path saw 283 of 10,000 positions in the STAGES=5 run, so the cover lever is capped in the low single digits by construction, whereas the codebook produces the graded code every position is keyed under.

## Result

Five teacher-free `compile_recorded` runs on the pinned 500k fixture corpus, cover held fixed, held-out store top-1. The k-means training set is `min(CTX_SAMPLE, RVQ_SAMPLE_CAP)`, so both knobs are swept.

| arm | pool | cap | train | occ. keys | rec/key | top-1 | vs shipped |
|---|---:|---:|---:|---:|---:|---:|---:|
| A | 10k | 10k | 10,000 | 77,799 | 5.14 | 35.53% ± 0.15 | −0.16pp |
| **A — SHIPPED** | **50k** | **10k** | **10,000** | **79,244** | **5.04** | **35.69% ± 0.15** | — |
| A | 200k | 10k | 10,000 | 74,952 | 5.33 | 35.74% ± 0.15 | +0.05pp |
| B | 50k | 50k | 50,000 | 85,496 | 4.68 | **36.14% ± 0.15** | **+0.44pp** |
| B | 200k | 200k | 200,000 | 79,933 | 5.00 | 36.06% ± 0.15 | +0.37pp |

**NEGATIVE against the pre-declared +1.0pp exit rule, but not null.** +0.44pp at 2.1 SE on the *difference* (conservative — the arms are paired on the held-out set), and it **saturates**: 200k does not beat 50k.

## The result with consequences beyond this issue

**Records-per-key is a symptom, not the binding quantity.**

The STAGES=5 negative was read as: records/key fell 36.02 → 18.80 and top-1 fell with it, "the signature of thinner per-key evidence." Here records/key falls **5.04 → 4.68** and top-1 **rises +0.44pp**.

Both thin the evidence per key; they differ in *why*. Added key **resolution** splits evidence that belonged together. Better **fit** moves evidence onto the key that represents it. So a falling records-per-key is not itself a warning sign, and #460's causal reading narrows to resolution specifically. The subdivision negative itself stands untouched.

This forces a correction to the README pattern paragraph, which listed *"a better-fitted codebook"* among the failed key-resolution levers. That was a misclassification on both counts: fit is not resolution (fixed `K`, fixed `STAGES`, fixed nominal key space — it changes *which* key evidence lands on), and measured in isolation it is **positive**. The distinction to carry forward is **which key evidence lands on** (fit — helps) versus **how many keys there are** (resolution — has never helped). This is the first confirmation of the evidence-quality pattern on something that touches the graded code itself rather than storage or calibration.

## Two corrections to `capacity_scaling_432.md`

**§3.1 is one step too strong.** It states #407's `CTX_SAMPLE` 6,000 → 50,000 raise "did not change the codebook's training-set size at all." The training set is `min(CTX_SAMPLE, RVQ_SAMPLE_CAP)`, and at 6,000 the 10,000 cap **does not bind** — so that raise moved it 6k → 10k and #407's attributed starvation share was real. What sweep A kills is raising `CTX_SAMPLE` *further* while the cap stands (0.21pp over a 20× pool range), which is every configuration since #407. The three points form one diminishing-returns curve: 6k→10k ≈ +0.5pp, 10k→50k +0.44pp, 50k→200k nothing.

**§5.3 is right in shape and over-valued.** `N/10` = 39,969 here, essentially the measured saturation point — the formula's shape is confirmed. The 500,000 ceiling measurably buys nothing. Adoption is κ-affecting and worth 0.44pp, so it belongs in the next re-pin that happens for another reason rather than triggering one. Also: the proposed instrument reading ("occupancy rises and records-per-key falls monotonically") is **not usable as a gate** — occupancy went 77,799 → 85,496 → 79,933.

And §6's standing worry that the calibrated 500k rows were themselves taken on an under-fit codebook is true and now **bounded at 0.44pp**, which closes it rather than leaving it as open-ended doubt about every pinned row.

## Scope limits

Teacher-free recorded compile at 500k, so absolute numbers are not comparable to `BASELINE.md` — only arm-to-arm deltas are. Saturation was located between 50k and 200k on a 500k corpus; whether it tracks `N/10` at 2.11M is untested and is the one thing that would change the recommendation. Full Gate C was not run: at a 0.44pp effect it could not have separated the arms better and it costs hours.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` all green. The harness is `--ignored` (~30 min, five compiles, no teacher or checkpoint required).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

