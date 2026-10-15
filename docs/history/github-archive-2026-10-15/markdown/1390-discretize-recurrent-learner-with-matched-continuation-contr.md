# #1390 — Discretize recurrent learner with matched continuation controls

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T07:00:46Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-09-25T09:27:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1390
- **Merged:** 2026-09-25T09:27:00Z
- **Merge commit:** 93689d81dda7685b576066c5af02c67beefd7e3b
- **Base / Head:** main / UOR-Foundation:codex/quantized-recurrent-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Programme purpose and decision

References #973 and #820. Implements and completes D8 rung 2's first quantized continuation of the retained joint recurrent-memory learner.

**Reject the paired recipe at its frozen retention scope.** Both final packed models fail the +0.05-nat matched-continuous loss allowance; ordinary packed generation also fails the short-cycle criterion. Preserve implementation, every negative artifact and the continuous controls. No model promotion or integer-serving claim.

## Implementation

- Frozen parent-calibrated dyadic scales, packed signed four-bit multiplicative parameters, sixteen-bit additive offsets and explicit recurrent interface grids.
- Clipped straight-through surrogate, bound ramp clocks and preserved AdamW moments/sampler at B16/T256.
- Actual packed loader with hard export/reload and separate same-shadow evaluation. F32 emulation still executes dense maps and floating arithmetic.
- Shared Rust state-drift diagnostic, complete report sealing, exact source/data/parameter bindings and full row-level source-retention evidence.

## Executed result

All four branches complete the fixed step 8,348: 1,024 new updates / 4,194,304 target visits each, including the retained QAT profiles. QAT uses 256 ramp + 768 full-hard updates. One paired seed on previously exposed development; no checkpoint reselection.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Matched continuous | 2.090518499 | 2.064403221 |
| Packed QAT | 2.149631373 | 2.144230654 |
| Packed minus continuous | **+0.059112874** | **+0.079827433** |
| Packed zero-update parent | 2.222929353 | 2.211900518 |
| Packed NoRead | 2.634758168 | 2.605525329 |

QAT improves direct parent quantization but does not retain the continuous control within the frozen tolerance. Gate results are quaternion **FAIL/PASS/PASS/PASS/PASS**, ordinary **FAIL/PASS/FAIL/PASS/PASS**. Ordinary seed 2014 ends in a period-two cycle. Quaternion old/continuous/packed exact completions are **28/23/19 out of 32**; ordinary **23/26/25**. All responses, losses and gains are preserved in the result receipt.

Both packed exports have zero measured full-context reload probability difference and matching seeded IDs/distribution hashes. Finite normalized evaluation covers all 249,856 positions; comparison tail is 233,472. State diagnostics report finite packed-versus-shadow coordinates on original tune64, with no isolated-transport/stability claim.

## Review and validation

- Sixteen focused release Rust checks at 38983600; later primary-source changes only add shadow reporting and clarify selection text.
- Actual fit/evaluation binary bound to b5b5fe75; state example built at cb81e7c7 with model library unchanged.
- `cargo fmt --all -- --check` passed at cb81e7c7; no Rust edits afterward. Final claim-wording and diff checks passed.
- Independent review reconciles all 12 sealed evaluator roots, all 36 partition means, both packed exports/state reports, 20 packed/continuous generations and all 192 old/continuous/packed source responses.
- RDC ran DeepSeek math/systems and Kimi architecture sessions concurrently with local work; two final Rust diagnostics also ran concurrently through RDC. Advisory errors and unsupported causal claims were corrected by principal review.
- GitHub compatibility acknowledgements execute no tests and are not validation evidence.

## Next and resources

Recommend one separate projected-shadow continuation from retained full-hard midpoint 7,836 to 8,348, preserving fixed scales/data/moments/clocks and all five gates. Require entry packed-code and hard-output parity. **Proposal only, NOT_RUN.** Retain current unprojected branches as direct controls; bounded admission and final integer execution remain downstream.

Complete-cycle wall is charged once through the committed cutoff; delivery tail will be charged after merge and recorded on the owning issues. Sampled model RSS and physical free space remained within the configured guards. Current retained allocation plus full temporary/metadata allowance fits the campaign ceiling; compiler temporary peaks are unmeasured and historical gross allocation remains UNRESOLVED. No external training hardware used. All unique models, source, reports, executed binaries and worktrees preserved.

README, current state, canonical plan, project map, evidence index and training README point to the completed result and scoped next action. #973 and #820 remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

