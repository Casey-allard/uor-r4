# Reader policy contract: the corrected fit is a clean negative, and the constant boost explains the text gain

> **Principal correction after PR #1330:** [review](policy-objective-review-2026-09-21.md) and [independent audit](../evidence/policy-objective-principal-review-2026-09-21.json). The fit/serve repair and all four negative criteria verify. The old artifact's text gain remains valid; the changed successor does not refute it. Attempt 4 repeats attempt 3's final data/outputs. Objective-only and sufficient-support diagnoses are unestablished; alternative-action emission/stratum statistics needed for joint feasibility were not saved. Loader rejection and neighbor-intervention claims require correction; different-seed admission is not rounding, and construction's paired timing median is positive. The original dated design/report below is preserved as history, not active authority. The [new constructive prompt](deepseek-policy-feasibility-step-2026-09-21.md) owns execution.

Executed September 21, 2026 from base revision `0492926d` (PR #1329 merged). Prospective design and frozen
criteria: [reader-policy-contract-design-2026-09-21.md](reader-policy-contract-design-2026-09-21.md),
recorded before the corrected fit. Receipt:
[../evidence/native_geometric_reader_policy_contract_2026-09-21.txt](../evidence/native_geometric_reader_policy_contract_2026-09-21.txt).
Principal review: [reader-utility-review-2026-09-21.md](reader-utility-review-2026-09-21.md).

Delivered root: `.uor-models/realtext-prior-2026-09-20/reader-utility-4` (sealed, verified, 0 unlisted).
`reader-utility-2` remains sealed and byte-identical; `reader-utility-1` remains unsealed.

## The defect, and the repair

PR #1328 computed tune-selected gap thresholds and then built **both** fit-event streams from the unchanged
legacy parent, whose thresholds are `[0,0,0]`. Because `gap ≤ 0` and the bucket counts strict
`gap > threshold`, every fit event landed in band zero: buckets 0–7 only, while serving addressed all 32.
Twenty-four buckets were never fitted at their served addresses.

The repair is one immutable **`PolicyConfig`** created from the thresholds **before any event exists**. It
yields the fit-time selector, the events, the exported selector and the serving selector, so the two
partitions are the same array by construction. `RelationalSelector::verify_policy_contract(&cfg)` is a
**verified loader contract** the consumer must call before prediction; the artifact carries the digest of its
**fit inputs** in `data_digest`; the delivery manifest carries each artifact's byte hash. The binding is
acyclic and the run exercises **rejection** of a drifted threshold configuration, a swapped opcode table and
a mismatched input identity. A new contract test asserts, on the same observation and logits, that the stored
fit-event bucket and source equal the independently reloaded selector's bucket and source — across
legacy-v3 input, every gap band, exact threshold ties, one and zero candidates and `i32` extremes.

**The corrected fit is materially different.** 16 buckets now earn their own action (was 4) and the opcode
histogram is `[0 NoRead, 0 weak, 25 one-nat, 7 eight-nat]` (was `[0,0,31,1]`): the policy really does use the
strong action where the evidence supports it.

## Result: the corrected policy does not deliver the text benefit; a constant boost does

Held-out reader **documents** (8 new disjoint documents, `sha256`-sorted `[24,32)`, 1,952 positions,
782 candidate-bearing):

| Arm | Δ bits/token vs local | document interval |
| --- | ---: | ---: |
| `relational_ctx_parent` (frozen) | **+0.4532** | — |
| **`h4_policy` (corrected fit)** | **+0.1193** | [+0.0702, +0.1679] |
| `categorical_policy` (corrected fit) | +0.1533 | [+0.0942, +0.2072] |
| **`h4_one_nat_fixed`** (constant 1-nat, same source/contract) | **−0.0148** | — |

`h4_policy − relational_ctx_parent` = **−0.3339 [−0.4872, −0.2005]**. So the corrected policy is much better
than the frozen parent and *still harmful*; **the small held-out text benefit comes from a constant one-nat
boost, not from the fitted table** (`fitted_beats_one_nat_on_text = false`).

The PR #1328 text finding does **not** reproduce. On the previously inspected documents (now a development
panel) the corrected policy gives **+0.1469** where the misindexed policy recorded −0.0195. The earlier
`useful_transfer=true` was an artifact of the indexing defect; the principal review's decision to withdraw the
representation diagnosis is confirmed, and no representation claim is made here either.

## Honest negatives on the declared categories

| Category | Value | Evidence |
| --- | --- | --- |
| `harm_containment` | **false** | +0.1193 > 0.05 bits/token |
| `useful_transfer` | **false** | interval entirely above zero |
| `relational_behaviour_preserved` | **false** | present-query emitted correct **56 vs parent 77**; loss margin also fails |
| `absence_improved` | **false** | absent reads **19 vs parent 5**; absent loss **0.2969 vs 0.1379** bits/query |
| `all_noread_collapse` | false | read rate 1.00 of candidate-bearing |

Construction (new seed `SEED_FINAL2`, 2,418 positions): `local` 12.4821, parent −1.8553 (10.6268),
`h4_policy` −1.7255 (10.7566), `categorical_policy` −1.8464, `h4_one_nat_fixed` −0.3066 (12.1755). The
fitted policy raises correct payload reads (446 → 516) but lowers correct emitted tokens (355 → 319), and the
constant one-nat arm collapses emissions entirely (emitted correct **1**). `fitted_beats_one_nat_on_construction
= true`.

**Reading of the tradeoff.** The expected-cost objective and emitted-argmax correctness are different
functionals. On the equal-weight construction/text mixture the fitted table chooses the strong action on
1,181/1,810 construction positions but also on 303/782 text positions, where it harms; the constant one-nat
arm avoids the text harm by never choosing the strong action, at the cost of destroying construction
emissions. Per-action counts by stratum are serialized (`strength_counts`, `reads`, `no_read`,
`emitted_correct` per stratum), so this is an observable **objective tradeoff**, not a representation
collision and not insufficient support.

## Repaired metrics, all verified

- **Policy-aware decomposition**: `eval_stream` now passes the actual served action. `ranking + gate + dose =
  actual − Lpool` holds per position, and the `actual` term equals the loss change implied by the actually
  emitted logits with **0 mismatches** on every scored arm and panel; `0` reads had a served source differing
  from the ungated top source.
- **Admission regret with correct denominators**: candidate-only **0.1946** nats/candidate position and
  complete-stream **0.8316** nats/position on the final construction (608 empty-pool positions included with
  `Lpool = 0`). These reproduce the principal review's independent reconstruction (0.200636 / 0.831657) to
  within the expected rounding of a different panel seed, which cross-validates the repaired formula.
- **Four-condition intervention**: 140 attempts, disabled-before == disabled-after **140/140**, original
  occurrence **still admitted 140/140**, candidate pool size changed 16, neighbouring payload/features changed
  140, original source reference still served 138, emitted the new payload 93, 0 lost support, 0 NoRead.
- **Binding**: `reload_failures = 0`, including the three rejection probes; the serialized-structure
  **preflight** validated the object actually written before sealing.
- **Cost**: paired but order-fixed and short (18 construction predictions at 1.94 candidates/prediction;
  21 text predictions with zero candidates); the paired medians are negative, i.e. within noise. Reader
  incremental cost and physical energy remain **UNAVAILABLE**. Whole-path D0-b compliance is not claimed.

## Decision

The intended learned finite-influence experiment has now been executed **correctly** and is **negative** on
its declared categories. No representation conclusion is drawn from the earlier misindexed run, and none is
drawn here either: the decisive observable is an objective tradeoff between expected action cost and emitted
accuracy on a mixture whose text half has low copy reliability. Per the design note's decision rule, no
extension is taken — the remaining budget would not fund a defensible constrained-objective fit with a fresh
final draw, and the prompt's own instruction is to report the real distinction rather than sweep.

## Next step

One evidence-supported successor: a **prospectively declared constrained utility objective on the same
features and the same frozen sources**, i.e. choose the per-bucket action that minimises the expected cost
**subject to** emitted-correct preservation on a declared construction stratum (a bounded Pareto check on the
already-serialized per-bucket/per-action costs, which are additive because sources and actions are fixed).
That directly tests whether the observed tradeoff is an objective artifact or an unfavourable scalarisation,
before any new observation or representation is introduced.
