# #1391 — Project fixed-grid shadows and record failed paired retention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T13:03:27Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-25T14:12:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1391
- **Merged:** 2026-09-25T14:12:53Z
- **Merge commit:** ada3c77bbebcde37c36e3cd5a68b242857dc2be9
- **Base / Head:** main / UOR-Foundation:codex/projected-recurrent-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

References #973. References #820.

Implements detached projection of stored QAT shadows into their existing dyadic ranges, binds the policy to exact full-hard parents, and completes the owner-authorized paired continuation. Both arms finish step 7,836 → 8,348 with 512 new updates / 2,097,152 target visits each at B16/T256, preserving moments, clocks, scales, data and all five original gates.

**Decision: reject the paired projected recipe for retention.** No model is promoted, no gate is changed, and every original negative/control remains preserved.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Continuous control | 2.090518499 | 2.064403221 |
| Prior unprojected packed | 2.149631373 | 2.144230654 |
| New projected packed | 2.150319798 | 2.147928712 |
| Packed minus continuous | **+0.059801299** | **+0.083525491** |
| New shadow, quantizers disabled | 2.113220567 | 2.098025647 |
| New whole-prefix NoRead | 2.637288928 | 2.618333390 |

Both exceed the unchanged +0.05 allowance. Five-gate vectors are **FAIL / PASS / PASS / PASS / PASS** for both. The ordinary previous short cycle is absent in this five-prompt panel, but text remains semantically unreliable. Source first-noun losses are two/one; exact completions are 23/32 and 22/32, with individual losses/gains against every prior artifact retained.

## Implementation and evidence

- Detached parameter projection preserves interior shadows and Var identities. Exact parent/spec/policy bindings persist through resume, checkpoint, export, evaluation and matched comparison.
- Entry eliminates 41,184/43,770 out-of-range coordinates while preserving all 1,678,466 hard parameter values/codes, all 1,048,576 actual full-context probabilities, moments/clocks and next sampled inputs/targets.
- All 512 updates per arm record post-AdamW projection. Final clipping is zero. Shadow NLL and state RMS improve, while packed NLL does not. No single-cause clipping/scale/interface diagnosis follows.
- Both packed reloads have zero measured probability difference and equal seeded generated IDs/distribution hashes. Full evaluations retain finite normalization and exact data identities.
- Independent arithmetic review verifies 31 sealed roots, 34 input-binding records and 54 partition means across 18 populations, plus all response losses/gains and state inputs/arithmetic.
- RDC runs concurrent DeepSeek mathematics/systems reviews, paired Rust fitting/evaluation and a Kimi result review on the same M1. Principal corrections to advisory overclaims are recorded.

## Validation and resources

- **22 focused optimized Rust tests passed**, zero failed/ignored; source `66349cdb69883dd7d438c76394c22f4e19000c73`.
- Optimized training binary and state example built successfully; actual fitting, exports, population evaluations, source edits, generations and state diagnostics completed.
- `cargo fmt --check`, `python3 scripts/check_claim_wording.py`, and `git diff --check` passed. Executed Rust and dependency inputs are unchanged by final documentation.
- An initial wrong Cargo target name and an analysis-only packed-parent metadata expectation were corrected. Original error logs are retained; neither correction altered model criteria or required retraining.
- Paired fits took 2,167.629 seconds. Peak sampled aggregate model RSS 7,530,168,320 B; minimum free disk 22,765,936,640 B, above the 21,676,163,072 B stop. Retained allocation plus full temporary/metadata allowances is 2,044,174,336 B, below 2.5 GiB. Compiler temporary peaks and historical gross allocation remain unmeasured/unresolved.
- Complete-cycle wall through the committed cutoff is 5,309,506 ms; cumulative ledger 525,236,590/532,800,000 ms. Delivery tail will be charged once after protected merge. No cleanup, unique-artifact deletion or paid external training hardware.
- **GitHub compatibility statuses execute no tests.** Local executed validation and artifact evidence carry the claims.

## Direction and limits

Next recommendation, **NOT_RUN**: one fixed-checkpoint 2×2 parameter/interface precision comparison, reproducing retained FF/QQ endpoints and evaluating QF/FQ on the same population and actual response panels. No new training/scale search/holdout in that diagnosis. Its result selects one coherent integer-realizable bridge change under the original retention gates.

One paired seed and exposed development remain the scope. Packed artifacts expand into F32 computation; this does not qualify integer serving, D5 parameter sparsity, geometric advantage, Hamiltonian dynamics, useful general language/code or energy.

README, current state, canonical ladder, source map, evidence index and training README are synchronized. Complete result/evidence/resource/review records are in `docs/integration/projected-recurrent-*-2026-09-25.md` and `docs/evidence/projected-recurrent-*-2026-09-25.json`. #973 and #820 remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

