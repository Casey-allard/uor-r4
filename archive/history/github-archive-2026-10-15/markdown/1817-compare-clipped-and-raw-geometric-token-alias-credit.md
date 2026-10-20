# #1817 — Compare clipped and raw geometric token-alias credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T01:07:32Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-10-07T01:33:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1817
- **Merged:** 2026-10-07T01:33:24Z
- **Merge commit:** 4757ae79125a007a3489a2b3e50b9af53bfe5d44
- **Base / Head:** main / UOR-Foundation:codex/raw-alias-credit-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Saturated scores currently lose direct score credit through the clipped Jacobian. Add an opt-in raw-identity score adjoint while retaining the default clipped policy, identical native hard forward scores and token-alias probability.

Extend the retained-checkpoint diagnostic to compare clipped, raw and margin gradients by parameter family and jointly, preserving relative coordinate weights and reporting projected unit directions. This is a zero-update diagnostic, not a training-policy adoption or language-quality claim.

Validation at the exact head is in progress: remote release CUDA build, saturated/unsaturated alias tests, numerical metric test, then the fixed eight-row retained-checkpoint comparison. Touched-file formatting and diff whitespace checks passed. References #820.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6028758742 — 2026-10-07T01:15:00Z — Casey-allard

Exact-head review at d2d71a5cb402aa3dae6e2202d6d19c519ff7bf35: independent adversarial source review found no blocking semantic defect. I verified authentic native admission, identical hard forward/loss for clipped/raw, default Clipped policy, whole-group rather than per-tensor normalization, coefficient-bound projection signs and unbounded prototype exception. Numerical ACTIVE is nonzero-vector status, not a quality gate.

Actual Linux x86_64 Rust 1.97.1 checks at this head: touched-file rustfmt --check and git diff --check PASS; cargo build --release -p uor-r4-training --features cuda --example geometric-entry-margin exit 0 (73s); cargo test --release -p uor-r4-training --features cuda --lib raw_alias_credit: 2 passed, exit 0; cargo test --release -p uor-r4-training --features cuda --example geometric-entry-margin scaled_group_comparison: 1 passed, exit 0. GPU masked during compilation/tests; fixtures run CPU. CUDA executable SHA256 1085fa74d4961bf2667c63b6e242762ea1a6568c74f8ab98755aceaf7fd61416. Clean committed source; run outputs excluded from git. Receipts on canonical /workspace/uor-r4/codex/reconciliation/raw-alias-d2d71a5/build-checks.log.

Actual checkpoint CUDA measurement is NOT_RUN: leased pod ywaeptdb5gtvg4 created through main uor-pod, canonical volume, runtime null/no direct SSH yet; provider boot-log requests timed out twice. Existing startup handle is live. Do not interpret this setup state as model evidence. PR not queued until the declared measurement resolves.

### comment 6028956352 — 2026-10-07T01:32:31Z — Casey-allard

Exact-head CUDA measurement now PASS at d2d71a5cb402aa3dae6e2202d6d19c519ff7bf35, executable SHA1085fa74d4961bf2667c63b6e242762ea1a6568c74f8ab98755aceaf7fd61416. Healthy RTX5090 host wt3s72s1qwmlkn, driver570.153.02, CUDA12.8, SM120; GPU lock held. Job ran 01:29:07–01:29:34Z (27s), exit0. Fixed rows0..7, original retained F32 masters and quarter bridge, zero updates. Independent integer full-bank→bridge→Generate→vocabulary parity8/8; old/raw scalar forward losses equal8/8; parameter inventories unchanged. Driver sealed and verified the complete report. Report SHA702ac03749ec6c246dd47a4aff98e9f045a402235a1b2f92d6506aed23e4e587, canonical /workspace/uor-r4/codex/reconciliation/raw-alias-d2d71a5/actual-checkpoint-attempt1/report.json.

On saturated rows0,1,2,4, joint projected margin cosines change from [-.5745,-.6054,-.6321,-.6238] to [-.9558,-.9051,-.9276,-.8894]; old/raw projected alignment [.7809,.8809,.8696,.9077] confirms a direction change rather than norm-only scaling. Generate family is unchanged; restored corrective credit is in potential. Four other rows have identical old/raw directions. These are local surrogate directions, NOT updated native probabilities, Adam behavior, useful generation or attention qualification. The default policy remains unchanged. Retained source-dependent/categorical-transport failures remain separate.

Review conclusion: the declared implementation risks are covered by prior exact-head focused checks plus this real CUDA measurement. No serving/default policy modification or model-quality adoption. Failed first allocation ended at SSH timeout and was deleted; second reachedSSH40s and is deleted after completed job. Queue acknowledgements are not the executed tests.


---
_Comment/review ids in this file: 6028758742, 6028956352_

