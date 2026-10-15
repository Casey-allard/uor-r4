# #1328 — Direct read-influence policy: useful transfer on held-out prose, relational behaviour traded away (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T05:31:32Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-21T05:37:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1328
- **Merged:** 2026-09-21T05:37:13Z
- **Merge commit:** a57524767390f0491e7fa2af65fa335d715a9b30
- **Base / Head:** main / UOR-Foundation:codex/reader-utility-transfer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

> **Principal correction, PR #1329:** the held-out text CE gain reproduces, but fitting and serving use different gap thresholds. The [review](https://github.com/UOR-Foundation/uor-r4/blob/5943789aa37279d63f1a253897ab1609f579d68c/docs/integration/reader-utility-review-2026-09-21.md) withdraws the contextual-representation diagnosis and qualifies actual-action regret, absence criteria, admission denominators, source/binding and cost claims. The [successor](https://github.com/UOR-Foundation/uor-r4/blob/5943789aa37279d63f1a253897ab1609f579d68c/docs/integration/deepseek-reader-policy-contract-step-2026-09-21.md) completes the intended correctly indexed policy experiment. Original report below is preserved.

## What this delivers

One constructive utility-transfer step: **learn whether, and how strongly, an already selected source should influence the local predictor**, using a direct hard finite-action objective and compact causal integer observations — with the improved relation learners **frozen** and the specified causal-control and artifact repairs integrated.

References #973, #820, #963, #964. **No issue is closed.**

### Mechanism

The five parent artifacts are loaded **by SHA256** and never refitted; only *influence* is learned. A 32-bucket direct opcode table over the **ungated top source** selects `{NoRead, 0.0625, 1, 8}` nats, keyed by three causal integer observations:

```text
bucket = gap_bin*8 + ctx_class*2 + margin_bit
  gap_bin    : selected payload's local logit gap from the local argmax, quantized by three
               artifact-stored integer thresholds
  ctx_class  : payload == current token, and ordered two-neighbour agreement of the top source
  margin_bit : the ungated top source score strictly exceeds the runner-up's
```

Fitted by the **mean individual** action cost `Δ(a) = log(1 + p·(exp(a)−1)) − a·I[payload == target]` (NoRead = 0), with a declared minimum support of 20 weighted events, an explicit global fallback for sparse/unseen buckets, and a **safe-abstention tie rule** (a read only if strictly better than NoRead). Serving uses integer comparisons and a 32-byte table; `RLR2` v4. Fit mixture: construction fit + natural-text fit positions at **equal declared total weight** (2,969 + 944 events).

### Headline: the reader now *helps* held-out prose

8 disjoint held-out documents (path + content hash disjoint from fit and tune), 1,952 positions:

| Arm | Δ bits/token vs local | document interval |
| --- | ---: | ---: |
| `relational_ctx_parent` (PR #1325) | **+0.5453** | [+0.4038, +0.6847] |
| **`h4_policy`** | **−0.0195** | **[−0.0329, −0.0055]** |
| **`categorical_policy`** | **−0.0243** | [−0.0369, −0.0126] |

`h4_policy − parent` = **−0.5648 [−0.7020, −0.4269]** bits/token. First **net benefit** on held-out prose rather than harm, concentrated where causal copying is possible (−0.5649 on copy-available vs +0.0204 on copy-unavailable). Categories: **harm containment true**, **useful transfer true**, **absence improved true**, no all-NoRead collapse.

### The honest negative

**`relational_behaviour_preserved = false`.** On the new final construction draw the policy reads **every** candidate-bearing position at ≈1 nat (1,665 of 1,822; only 4 of 32 buckets earn their own action):

| final draw (2,442 positions) | hard bits/pos | reads | correct payload | emitted correct |
| --- | ---: | ---: | ---: | ---: |
| `relational_ctx_parent` | 10.5981 | 819 | 465 | **362** |
| `h4_policy` | 11.8945 | 1822 | **517** | **29** |

Present-query hard CE is **+7.59 bits/query worse**, and correct payload reads *improve* (465 → 517) while emitted tokens collapse (362 → 29). Diagnosed cause: under the equal-weight mixture the expected-cost minimiser is conservative, and `gap`/`ctx_class`/`margin_bit` **cannot separate high-copy-reliability construction buckets from low-reliability text buckets** — a **contextual-representation limitation**, not support (full 2,969 + 944 events), quantization (exact round-trip) or the local predictor (unchanged). Expected action cost (fitted) and emitted-argmax correctness (measured) are different functionals, and this run makes that concrete.

### Repairs integrated

- **Four-condition selected-source intervention**: enabled/disabled × original/changed on the same prefix — **disabled-before == disabled-after on 140/140**, 138 selected the new payload, 138 kept the original source reference, 9 emissions changed, **0 lost support, 0 NoRead**. The parent's mis-matched comparison is fixed.
- **Real group digest** (SHA256 over product/inverse/identity) instead of a builder-name string; **`binding.json`** with parent digests, fit/tune/final document+window identities, full observation digest, feature/bin definitions, actions, seeds and format versions, plus a **`manifest_digest` the runner recomputes and verifies** on load.
- **Schema pre-validation** before the expensive pass; artifact reload parity **0 failures**; the reloaded arms drive every panel, intervention, generation and timing.
- **Ring/admission diagnostics** on the same full stream (Lpool − Lring ≥ 0): mean admission regret **0.150** nats (final construction) and **0.283** (final text) — the declared 0.25 trigger fires **on text only**, named and not acted on.
- **Cost**: paired interleaved, same stream, medians — reader increment **+0.7 µs/prediction** against a ~1,352 µs/prediction uncached local path; 1.47 admitted/prediction on the construction stream. Energy `UNAVAILABLE`; no whole-path D0-b claim.

### Tests run

- `cargo test -p uor-r4-core --lib native_geometric::learner::relational` → **26 passed** (3 new: policy round-trip and opcode/threshold rejection; fitter support/global fallback and the abstention tie; gap-observation bucket indexing)
- `cargo test -p uor-r4-core --bin competitive-reader` → **6 passed**, 1 ignored (1 new: the shared `read_step` uses the policy and its gap observation)
- `cargo fmt --check`, `cargo check --offline`, `python3 scripts/check_claim_wording.py` clean

Delivered sealed root `.uor-models/realtext-prior-2026-09-20/reader-utility-2`. Attempt `reader-utility-1` is retained and unsealed (all measurements completed; it failed only because the binding manifest was named `manifest.json`, colliding with the seal filename); attempt 2 also fixes per-position document attribution and the read-rate denominator. All parent and historical roots untouched.

**Next (one):** a **learned contextual observation that separates high-copy-reliability from low-reliability reads** — concretely persisted role/scope or query-relation state (stage 2) — rather than another bucket on the same three observations. The admission complement is second-ranked and applies only at the triggered text scope.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

