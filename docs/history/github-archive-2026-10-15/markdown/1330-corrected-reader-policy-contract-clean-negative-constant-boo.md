# #1330 — Corrected reader-policy contract: clean negative, constant boost explains the text gain (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T06:32:14Z
- **Updated:** 2026-10-09T13:17:50Z
- **Closed:** 2026-09-21T06:34:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1330
- **Merged:** 2026-09-21T06:34:06Z
- **Merge commit:** 4be53d88e7346d32d9a6e0a4584bb3cbc6f813a9
- **Base / Head:** main / UOR-Foundation:codex/reader-policy-contract
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

> **Principal correction, PR #1331:** the fit/serve repair and four negative categories verify. The [review](https://github.com/UOR-Foundation/uor-r4/blob/c3611de1/docs/integration/policy-objective-review-2026-09-21.md) preserves PR #1328's old artifact gain and qualifies objective/support attribution, loader rejection, neighbor accounting, replay freshness and timing. Joint constrained feasibility needs missing counterfactual development statistics; [next prompt](https://github.com/UOR-Foundation/uor-r4/blob/c3611de1/docs/integration/deepseek-policy-feasibility-step-2026-09-21.md). Original report below is preserved.

## What this delivers

The intended learned finite-influence experiment, **executed correctly**, with one immutable feature contract from event extraction through fitting, export, reload and serving — plus the attribution, endpoint and admission repairs the review required. References #973, #820, #963, #964. **No issue is closed.**

### The defect and the repair

PR #1328 computed tune-selected gap thresholds and then built **both** fit-event streams from the unchanged legacy parent, whose thresholds are `[0,0,0]`. Because `gap <= 0` and the bucket counts strict `gap > threshold`, every fit event landed in gap band zero — buckets 0–7 only — while serving addressed all 32. Twenty-four entries were never fitted at their served addresses.

Now one immutable **`PolicyConfig`** is created *before any event exists* and supplies the fit-time selector, the events, the exported selector and the serving selector, so the partitions cannot drift. The artifact carries the digest of its **fit inputs**; the delivery manifest carries each artifact's byte hash — an **acyclic** binding — and `verify_policy_contract` is a verified loader contract the consumer calls before prediction. The run exercises rejection of a drifted threshold configuration, a swapped opcode table and a mismatched input identity.

A new contract test asserts, on the same observation and logits, that the stored fit-event bucket and source equal the independently reloaded selector's bucket and source — across legacy-v3 input, every gap band, exact threshold ties, one and zero candidates, and `i32` extremes.

### Result: a clean negative, and the earlier text gain does not reproduce

Held-out reader documents (8 **new** disjoint documents, 1,952 positions):

| Arm | Δ bits/token | interval |
| --- | ---: | ---: |
| `relational_ctx_parent` (frozen) | +0.4532 | — |
| **`h4_policy` (corrected fit)** | **+0.1193** | [+0.0702, +0.1679] |
| `categorical_policy` | +0.1533 | [+0.0942, +0.2072] |
| **`h4_one_nat_fixed`** (constant 1-nat) | **−0.0148** | — |

`h4_policy − parent` = **−0.3339 [−0.4872, −0.2005]**: much better than the parent and still harmful. **The small held-out benefit comes from a constant one-nat boost, not the fitted table.** On the previously inspected documents the corrected policy gives **+0.1469** where the misindexed policy recorded −0.0195 — the earlier `useful_transfer=true` does not reproduce.

The corrected fit is materially different: **16 buckets now earn their own action** (was 4) with opcode histogram **[0, 0, 25, 7]** (was `[0,0,31,1]`), so the repaired experiment really did test the gap-conditioned policy.

**All four declared categories are false:** harm containment (+0.1193 > 0.05), useful transfer, relational preservation (**present-query emitted correct 56 vs the parent's 77**; loss margin also fails) and absence (**reads 19 vs 5**, loss **0.2969 vs 0.1379** bits/query). No all-NoRead collapse. The constant one-nat arm is better on text and destroys construction emissions (emitted correct **1**), so the observable is an **objective tradeoff** between expected action cost and emitted-argmax accuracy on a mixture whose text half has low copy reliability — not a representation limit, and not insufficient support.

### Repaired metrics, verified

- `eval_stream` passes the **actual served action**; `ranking + gate + dose = actual − Lpool` holds per position with **0** actual-vs-emitted-logit mismatches and **0** served-source disagreements on every arm and panel.
- Admission regret now includes empty pools (`Lpool = 0`) and reports **both** denominators: **0.1946** nats/candidate position and **0.8316** nats/position complete-stream on the final construction — reproducing the principal review's independent reconstruction (0.200636 / 0.831657).
- Four-condition intervention: 140 attempts, disabled-before == disabled-after **140/140**, original occurrence **still admitted 140/140**, pool size changed 16, neighbours changed 140, original source reference still served 138, emitted the new payload 93, 0 lost support, 0 NoRead.
- Serialized-structure **preflight** validated the object actually written before sealing; `reload_failures = 0`.
- Cost: paired but order-fixed and short; reader incremental cost remains **UNAVAILABLE**, energy `UNAVAILABLE`, no whole-path D0-b claim.

### Tests run

- `cargo test -p uor-r4-core --lib native_geometric::learner::relational` → **27 passed** (1 new contract test)
- `cargo test -p uor-r4-core --bin competitive-reader` → **6 passed**, 1 ignored
- `cargo fmt --check`, `cargo check --offline`, `python3 scripts/check_claim_wording.py` clean

Delivered sealed root `.uor-models/realtext-prior-2026-09-20/reader-utility-4` (0 unlisted). `reader-utility-2` stays sealed and byte-identical, `reader-utility-1` stays unsealed, and superseded attempt `reader-utility-3` is retained and never resealed (identical measurements; one bookkeeping defect in the new rejection probe). All parent roots untouched.

**Next (one):** a prospectively declared **constrained/Pareto utility objective on the same features** — choose the per-bucket action minimising expected cost **subject to** emitted-correct preservation on a declared construction stratum. Per-bucket and per-action costs are additive with fixed sources, so this is a bounded check on already-serialized evidence, not a sweep and not a representation change.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

