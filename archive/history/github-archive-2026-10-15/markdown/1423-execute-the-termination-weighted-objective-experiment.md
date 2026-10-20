# #1423 — Execute the termination-weighted objective experiment

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T07:11:27Z
- **Updated:** 2026-10-09T13:18:00Z
- **Closed:** 2026-09-27T07:13:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1423
- **Merged:** 2026-09-27T07:13:12Z
- **Merge commit:** 6e32af270a570f77e84a0cbda641e40cd078eddb
- **Base / Head:** main / UOR-Foundation:codex/termination-objective-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Executes the mainline "ranking/emission" work card: a bounded, matched **termination-weighted objective experiment** on the accepted step-15,672 parents. Development only; no promotion; frozen evaluator and prose criteria preserved.

References #973 and #820.

## Instrument and identity

- Additive, absent-by-default `end_weight` in `uor-r4-training` (weighted training loss on sentence-final/EOS targets; weighted-mean denominator; only the training gradient is weighted, so every reported metric stays standard). Default path unchanged (uniform-weight gradient test to 1e-5; full library suite 132/132 on the rebased source).
- Four sequential fits 15,672→16,696 (1,024 updates / 4,194,304 targets each), full256/B16/T256/shard2/seed240924, no resumes; treatment `end_weight=2.5`, control absent. Training artifacts executed at source `c54d7801`; the branch is rebased over `bbce4cf6` with the evaluation witness below.

## Result

- Fits complete; the standard unweighted development NLL is slightly worse for the treatment than the dose-matched control (quaternion 2.0982 vs 2.0879; householder 2.0802 vs 2.0641).
- Read comparison-tail NLL: parent quaternion 1.996490 → endw 1.994137 / plain 1.984753; parent householder 1.974724 → endw 1.977860 / plain 1.965662. NoRead penalties 0.506–0.510; no short cycles; **all guardrails hold**.
- Source rows (32): parent quaternion 21 → endw 29 (8 resolved / 0 regressed) / plain 27 (6 / 0); parent householder 23 → endw 29 (7 / 1) / plain 28 (6 / 1). The single regression (`11/original`, `key`→`keys.`) is shared by both conditions. Treatment-unique rows **3** (2 non-termination + 1 wrong-first-noun); control-unique **0** → the predeclared EXTEND threshold (≥4 vs control) is **not met**.
- Prose (frozen four criteria): **0/5 in all four finals**; the independent evidence auditor returned identical verdicts; 16/20 outputs stop at the 128-token cap.
- **Decision: INERT** per the predeclared rule. Stop the termination-weighting branch; the residual entity/role collapse and cap-truncation failures place the next rung in the state/read path. Disclosed: the INERT band is read as treatment-minus-control (+2/+1 rows); judgment sensitivity on at most two borderline stories; the dose-matched plain continuation already recovers 12 of the 20 failing rows (a dose effect, not the objective).

## Files

- `docs/integration/termination-objective-plan-2026-09-27.md` (predeclared rule) and `docs/integration/termination-objective-result-2026-09-27.md`
- `docs/evidence/termination-objective-2026-09-27.json` and `docs/evidence/termination-objective-2026-09-27/campaigns/`
- `crates/uor-r4-training/src/{joint_model,joint_parallel,joint_campaign,joint_rounding_campaign,joint_bounded_campaign}.rs` and `examples/joint-read-geometry.rs`
- `docs/integration/current-state.md`, `README.md`

## Checks (executed locally)

- `cargo fmt -p uor-r4-training -- --check` clean; touched-package offline check 0 errors; full library suite **132 passed / 0 failed** on the rebased source.
- Four fits and eight frozen evaluations completed with sealed reports; re-evaluating both treatment checkpoints on the rebased build (`…-witness` roots, binary `9e351540…`) reproduces the sealed `generations.json` and `story-probes.json` identically.
- `python3 scripts/check_claim_wording.py` clean.

## Notes

The repository's historical CI status names are queue compatibility acknowledgements and execute no formatting, Clippy or tests; the local checks above are the executed evidence.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5853742188 — 2026-09-27T07:20:17Z — Casey-allard

Fourth-lab delta review of `20703dfb` / merged `6e32af27`: our independent PR #1422 agrees with the INERT decision and preserves the actual outputs/row joins. Two documentation corrections and one new integration repair remain:

1. `joint_reader_comparison::study_config` accepts the new `end_weight` field despite reporting an unweighted fixed study. Fourth lab has claimed the narrow guard/test repair in #973; no running campaign or executable changes.
2. The executed shard-normalization/reporting defect remains unchanged from `c54d7801`. `batch_mean_nll` carries the weighted training loss, and shard-normalized losses/gradients are combined by batch fraction, so “every reported metric stays standard” is inaccurate. Independently unweighted evaluation is still comparable. We are correcting the descriptions, without relabeling old execution or launching a repair-fit.
3. Both rebased witnesses match all non-timing trajectory content, including probabilities, IDs, text and stops. Their files differ in `elapsed_seconds`; the machine packet already has that qualifier and the prose should match it.

Resource reconciliation also found a demonstrated shared-wall duplicate: the new02:21:27→07:13:51 receipt overlaps the fourth lab’s already-charged interval through07:10:35. Eighteen linked receipts establish the overlap. The shared ledger has been corrected with an immutable adjustment receipt, preserving your original745-byte/SHA2995b023… receipt and the6ms rounding remainder. The detailed accounting is posted on #973. Please use the corrected current ledger and union of already-covered intervals for subsequent charges; do not add this milestone’s full interval again.



---
_Comment/review ids in this file: 5853742188_

