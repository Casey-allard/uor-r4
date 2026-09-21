# Direct read-influence policy: useful transfer on held-out prose, relational behaviour traded away

Executed September 21, 2026 from base revision `31972e34` (PR #1327 merged). Prospective design and frozen
criteria: [reader-utility-design-2026-09-21.md](reader-utility-design-2026-09-21.md), recorded before
implementation. Receipt: [../evidence/native_geometric_reader_utility_2026-09-21.txt](../evidence/native_geometric_reader_utility_2026-09-21.txt).
Principal review: [relational-learning-review-2026-09-21.md](relational-learning-review-2026-09-21.md).

Delivered root: `.uor-models/realtext-prior-2026-09-20/reader-utility-2` (sealed, verified, 0 unlisted files).
Superseded attempt `reader-utility-1` is retained and never resealed: every measurement completed, and it
failed only because the binding manifest was written as `manifest.json`, which collides with the seal
filename. Attempt 2 changes `manifest.json` → `binding.json` plus two reporting fixes (per-*position*
document attribution for document intervals, and the read-rate denominator). The measurement is otherwise
identical.

**The parent's stronger relation result is untouched.** No source map, ranker or descriptor was refitted;
the five frozen parent artifacts are loaded by SHA256 and only their *influence* is learned.

## What was built

A **direct hard-action utility policy** over the frozen ungated top source: 32 buckets
(`gap_bin*8 + ctx_class*2 + margin_bit`) selecting `{NoRead, 0.0625, 1, 8}` nats, fitted by the **mean
individual** action cost `Δ(a) = log(1 + p·(exp(a)−1)) − a·I[payload == target]` (NoRead = 0) with a declared
minimum support of 20 weighted events, a global-action fallback for sparse/unseen buckets, and a
safe-abstention tie rule (a read only if strictly better than NoRead). Serving uses only integer comparisons
and a 32-byte opcode table; the gap observation is a causal integer, not a probability estimate. Artifact
`RLR2` moves to **v4** (policy + gap thresholds), and v2/v3 keep loading.

Fit mixture, declared before scoring: the parent's construction fit split plus natural-text positions from
the **fit** documents, scaled so construction and text carry **equal total weight** (realised `w_text = 3.145`,
2,969 + 944 weighted events).

## Headline: the reader now *helps* held-out prose

Held-out reader **documents** (8 disjoint documents, path and content-hash disjoint from fit and tune;
1,952 positions, 849 candidate-bearing, one document per window):

| Arm | Δ bits/token vs local | document-cluster interval |
| --- | ---: | ---: |
| `relational_ctx_parent` (PR #1325) | **+0.5453** | [+0.4038, +0.6847] |
| **`h4_policy`** | **−0.0195** | **[−0.0329, −0.0055]** |
| **`categorical_policy`** | **−0.0243** | [−0.0369, −0.0126] |

`h4_policy − relational_ctx_parent` = **−0.5648 [−0.7020, −0.4269]** bits/token. This is the first time the
reader has produced a **net benefit** on held-out prose rather than harm; the parent's +0.2743 on the
previously inspected documents is retained as its own scoped measurement. Strata show where it comes from:
on copy-available text positions the H4 policy is **−0.5649 bits/token**, on copy-unavailable positions
**+0.0204** — a small, real transfer benefit concentrated exactly where causal copying is possible.

Categories, all five declared prospectively: **harm containment `true`**, **useful transfer `true`**,
**absence improved `true`**, matched categorical reported, no all-NoRead collapse (read rate **1.00** of
candidate-bearing positions).

## The honest negative: relational behaviour is not preserved

| Construction, final draw (2,442 positions) | hard bits/position | reads | strengths (1/8 nats) | correct payload reads | emitted correct |
| --- | ---: | ---: | ---: | ---: | ---: |
| local | 12.5334 | 0 | — | 0 | 0 |
| `relational_ctx_parent` | **10.5981** | 819 | 0 / 819 | 465 | **362** |
| `h4_policy` | 11.8945 | **1822** | 1665 / 157 | **517** | **29** |
| `categorical_policy` | 11.7364 | 1822 | 1522 / 300 | 547 | 46 |

`relational_behaviour_preserved = false`: present-query hard CE is **+7.59 bits/query worse** than the parent
(11.94 vs 4.35), and correct emitted tokens collapse from 362 to 29 even though *correct payload reads
increase* (465 → 517). The parent's population reproduces the same pattern (`construction_regression`).

**Diagnosed cause, and it is a representation limitation, not a tuning failure.** The fitted policy reads
*every* candidate-bearing position at essentially the weakest action (1 nat, 1,665 of 1,822; only 4 of 32
buckets earn their own action, 28 take the global fallback). Under the declared equal-weight mixture the
expected-cost minimiser is conservative: the marginal expected-cost optimum for a low-correctness population
is a small boost applied everywhere, which is *harmless-to-slightly-helpful* on prose and *useless* for
flipping an argmax on the synthetic queries. The compact observations (`gap`, `ctx_class`, `margin_bit`)
therefore **cannot separate the high-copy-reliability construction buckets from the low-reliability text
buckets**. The prompt anticipated exactly this outcome: the missing distinction is a **contextual
source/query representation** question, not training support, quantization or the local predictor — the
policy has full support (2,969 + 944 weighted events), the artifact round-trips exactly, and the local
predictor is unchanged.

Note the two functionals are genuinely different: expected action cost (what was fit) and **emitted argmax
correctness** (what the endpoints measure). A CE-optimal strength is not an argmax-optimal strength, and this
run makes that gap concrete rather than hypothetical. Both are reported.

## Attribution, controls and artifact boundary

- Every position records the ungated top source, its exact `(seq, abs)` reference, payload, bucket, action
  and emitted token; **any** correct payload counts as correct. The repaired identity
  `ranking + gate + dose = actual − Lpool` is verified per position for both new arms, with **0** reads where
  the served source differs from the ungated top source.
- **Four-condition selected-source intervention** (reloaded policy): 140 attempts, **disabled-before ==
  disabled-after on 140/140** (the repaired invariance the parent got wrong), 138 selected the new payload,
  138 kept the original source reference, 9 emissions changed, 8 emitted the new payload, 0 lost support,
  0 NoRead. The replacement payload is absent from the whole sequence, the query is fixed, and the mutated
  occurrence is outside the four-token query context.
- **Ring diagnostics** on the same full stream (`Lpool − Lring ≥ 0`): mean admission regret **0.150**
  nats/position on the final construction and **0.283** on the final text window. The declared **0.25 trigger
  fires on text but not on construction**, so admission is named as a successor candidate at its declared
  scope, with no admission change made in this run. Ring-eligible targets at the ring level: 929/2,421 on the
  parent regression stream, matching the review's independent reconstruction.
- **Group identity** is now a real SHA256 over the group table bytes (product, inverse, identity), not a
  builder-name string. **`binding.json`** carries parent artifact digests, the fit/tune/final document and
  window identities, the full observation digest, the group digest, feature/bin definitions, action
  semantics, parameters and format versions, with a **`manifest_digest` the runner recomputes and verifies**
  on load. Artifact reload parity: **0 failures**.
- The expected result field set is validated against `schema.json` **before** the expensive pass.

## Cost

Paired, interleaved same-stream measurement (7 alternating repeats, medians, no token-pair cache):

| Stream | predictions | admitted/prediction | reads | local µs/prediction | reader µs/prediction | paired reader − local |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| construction stream | 15 | 1.47 | 10 | 1352.2 | 1352.3 | **+1.04e-5 s** |
| text window | 21 | 0.00 | 0 | 1343.4 | 1345.3 | −1.30e-5 s |

The reader increment is **+0.7 µs/prediction against a ~1,352 µs/prediction uncached local path** — about
0.05%, and within repeat noise on the text window. So this probe *does* resolve the question at this
precision: the incremental reader cost is real but negligible relative to the unoptimised local path, and
the local path is what a cost campaign would have to attack next. Resident: 32 policy opcode bytes and 12
gap-threshold bytes per arm; serialized 4,392 (h4) and 8,488 (categorical) bytes. Physical energy
`UNAVAILABLE`; no whole-path D0-b compliance is claimed.

## Generation

Finite greedy continuations from the reloaded artifacts on the final construction queries and the final
documents produce no fluent prose; all five compared arms remain degenerate, and `h4_policy` output is
indistinguishable from local on the sampled prompts (its 1-nat influence rarely flips an argmax). A
teacher-forced CE gain is not language capability.

## Decision

Under the design note's frozen rule: **`useful_transfer = true`** and **`harm_containment = true`** on
document-held-out prose, **`absence_improved = true`**, **`relational_behaviour_preserved = false`**, no
all-NoRead collapse, and the matched categorical arm is reported (−0.0243, marginally better than H4). The
primary question — *does the new policy reduce document-held-out harm or produce net benefit against local
while preserving useful source-dependent relational behavior?* — is answered **half in the affirmative and
half in the negative**: transfer yes, preservation no. That tradeoff is the result, and it is not relabelled.

## Next step

One justified successor, following the prompt's own stopping interpretation: the missing distinction is a
**contextual source/query representation**, so the next test is a *learned contextual observation* that can
separate high-copy-reliability from low-reliability reads — the concrete candidate is a persisted
**role/scope or query-relation state** (stage 2) rather than another bucket on the same three observations.
An admission complement is the second-ranked candidate and applies at the 0.25-trigger scope on text only.
