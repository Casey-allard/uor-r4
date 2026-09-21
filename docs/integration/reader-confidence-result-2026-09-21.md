# Read-confidence influence interface: fitted feasibility, fresh preservation failures

> **Principal correction completing PR #1333:** [review](reader-confidence-review-2026-09-21.md) and [audit](../evidence/reader-confidence-principal-review-2026-09-21.json). Fit expressivity is retained, but both confidence arms fail fresh own-parent present CE and absent-read criteria; categorical parent is 81 present/six absent reads, not H4's 73/ten. Confidence rollout/interventions/timing are NOT_RUN. Three attempts repeat one exposed population; current source hashes verify. The exclusive stream-mixing diagnosis is unproven and the [next prompt](deepseek-read-conditioned-state-step-2026-09-21.md) tests a read-conditioned update/shared emission operation. Original dated design/measurements retain their historical scope.

September 21, 2026. Executed from reviewed parent `e0d6296c` (PR #1332). Prospective
[design](reader-confidence-design-2026-09-21.md); [principal review](policy-obstruction-review-2026-09-21.md);
[constructive prompt](deepseek-reader-confidence-step-2026-09-21.md). Delivered root
`.uor-models/realtext-prior-2026-09-20/reader-confidence-3` (sealed, verified, 0 unlisted, manifest
`d81bda9e3dbff98a69ba5020631527698fbad83f3359611c1a15c5f3d6b0255f`); superseded `reader-confidence-1`
and `-2` are retained and never reused as the delivered root.

## Decision, independently corrected

Retain the confidence interface as a useful fitted component. Both selected tables meet development-fit constraints and have reported exhaustive solver status at their support/fallback scope. Both fail fresh own-parent present CE and absent-read preservation, despite meeting the selected-query count tolerance and point text-harm screen. No useful negative text gain or new confidence generated-behavior result exists. Preserve the artifacts; the principal review selects a read-conditioned state/emission operation instead of automatically adding another stream-type feature.

## Witness: selected-index and strength parity on nonempty fit positions

`D = max_strength strength_score(candidate, relation, strength, parent_bucket) - noread_score(parent_bucket)`
in widened `i64`; `parent_bucket` is the selector's own `bucket_of`. The rule
`empty pool -> NoRead; otherwise the ungated top source at eight nats iff D > 0` reproduces the
frozen parent's own served action on **2,969 development positions with 0 mismatches** for both the
H4 (`relational_ctx`) and categorical scored parents. This is the cheap implementation prerequisite,
not a model-quality claim. Empty pools are skipped, and independently reloaded integer-logit/predictor parity and explicit tie coverage remain unmeasured.

## The confidence address restores feasibility

Address `= utility_bucket * 2 + 1[D > 0]`, at most 64 opcodes, same four actions. Fit populations:
197 final-present, 23 final-absent, 1,952 text tokens (944 candidate-bearing).

| Arm | Solve | Supported | Text b/t | Present correct | Present Δ bits | Absent reads | Absent Δ bits |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| H4 parent reference (same fit) | — | — | +0.5724 | 149 | −1931.32 | 7 | +3.497 |
| **H4 confidence (selected)** | `OPTIMAL` exhaustive | 28/64 | **+0.0228** | 148 | −1926.64 | 7 | +3.497 |
| **categorical confidence (selected)** | `OPTIMAL` exhaustive | 27/64 | **+0.0036** | 158 | −2032.89 | 5 | +1.416 |
| witnessed parent rule (H4) | — | — | +0.5724 | 149 | — | 7 | — |

All rows refer to the development-fit population. The learned tables satisfy every fitted behavioral constraint **and** the +0.05 bits/token text
screen, while the raw witnessed parent rule would fail the screen (+0.5724). The 64-address search
finishes exhaustively (about 1.7–2.6M nodes), unlike the coarse class, which was proven infeasible.

## Final evaluation (one draw, exposed in attempt 1 and replayed in attempts 2/3)

Fresh construction seed `0x5C0F_C0DE` (140 sequences, 117 final-present / 23 final-absent) and the
**four remaining eligible `Dev` reader documents** (976 positions, disclosed as a small honest final
population; the other 32 are development history).

| Arm | Fresh present emitted | Fresh absent reads | Fresh text Δ bits/token |
| --- | ---: | ---: | ---: |
| local / NoRead | 0/117 | 0/23 | 0.0000 |
| `relational_ctx_parent` | 73/117 | 10/23 | +0.4346 |
| old coarse `h4_policy` | 57/117 | 23/23 | +0.0885 |
| `categorical_policy` | 80/117 | 23/23 | +0.1251 |
| fixed one-nat | 0/117 | 23/23 | −0.0115 |
| **`h4_confidence`** | **71/117** | **11/23** | **+0.0379** |
| **`categorical_confidence`** | **79/117** | **7/23** | **+0.0201** |

Required present preservation is parent − 2 (margin translated to 117): H4 confidence keeps 71 versus
the parent's 73, so **the emitted-count margin is met**; its absent reads rise 10 → 11, which **fails the
no-increase absence criterion by one read**. The categorical confidence arm preserves the emitted-count tolerance (79 against its own parent’s 81,
7 absent reads against its own parent’s 6) and keeps text inside the screen, but also fails absence. Both arms fail present CE preservation (+0.159790/+0.152981 bits/query against a +.05 limit). Both confidence arms lift text from the parent's
+0.4346 to within +0.05, but neither reaches a *useful* negative gain; the constant one-nat reader is
the only negative text arm and it emits **zero** correct present answers. Copying an answer is not
reasoning and this is not a prose result.

## Diagnosis, corrected by principal review

Restoring the sign fixes a demonstrated loss of information and yields feasible fitted tables. The proposed exclusive stream-mixing/forced-eight-nat explanation is not established: 64-address fit/tune matrices and per-position causal outcomes were not saved. Target absence from admitted payloads explains the scope of possible copy gains, and the current positive payload-only boost cannot improve any nonpayload target. Retain the interface; characterize its missing actual rollout and test one learned read-conditioned state/emission update. This is a new operation, not a claimed solution to every text regression.

## Interface repairs delivered in this run

- **Source-hash completeness**: the receipt now hashes `relational.rs`, `policy_feasibility.rs` and
  `competitive-reader.rs`, written after the final source edit.
- **Actual revision recorded**: `base_revision` and `running_source.git_rev` are both `e0d6296c`
  (dirty snapshot recorded) rather than a hard-coded stale revision.
- **Verified predictor used**: the one-nat comparator is now the predictor returned by the
  expected-manifest loader, not a separately loaded equal selector.
- **Expected-manifest loader**: 0 failures across `h4_policy`, `categorical_policy`, `h4_one_nat` and
  the two confidence artifacts; four rejections exercised (drifted thresholds, wrong byte hash, wrong
  fit-input identity, truncated bytes).
- **Neighbour accounting (old coarse H4 policy only)**: the deliberately edited occurrence is excluded from neighbour-change
  counts (140 → 20 changed), with neighbours added (0), removed (16), lost admission (0), still-admitted
  (140/140) and NoRead (0) reported separately; the disabled-condition invariant holds 140/140.
- **Identity**: the counterfactual score-space identity holds to `1.42e-13` bits over 5,442 positions.

## Retained and next

Retained: the confidence interface with a feasible fitted policy, the witnessed parent rule as fallback,
the two compact tables, and all superseded sealed roots. The [principal successor](deepseek-read-conditioned-state-step-2026-09-21.md) is frozen-confidence rollout plus one learned read-conditioned geometric update feeding shared emission. Structural persistence, dependent composition, broader language/executed Rust and
qualified scale remain the ordered responsibilities in [project-track](project-track.md).

Development selection plus one acceptance population and two replays; not generalization. Physical energy is
UNAVAILABLE and whole-path D0-b is not claimed. Resources and charges are in the
[resource ledger](resource-ledger-2026-09-19.md).
