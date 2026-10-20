# #2132 — Localize legal-constructor failure to branch-basis small pivots

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T02:38:08Z
- **Updated:** 2026-10-10T02:46:17Z
- **Closed:** 2026-10-10T02:39:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2132
- **Merged:** 2026-10-10T02:39:24Z
- **Merge commit:** d4628eff9362ce3c8cef7de92245b66e80ab2c7a
- **Base / Head:** main / UOR-Foundation:codex/legal-basis-diagnosis-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why
The protected legal constructor aborted with a generic singular-matrix error. This isolates the unchanged constructor in a Rust diagnostic harness and observes the pinned backend without changing its arithmetic, constraints, thresholds or search policy. It preserves the exact failing bases needed for a numerical repair. References #2030.

## Result
On the authenticated original development input245 saved credit (1,920 coordinates, 380 guards; 1,960 solver variables/440 constraints), root relaxation completes and branch feasibility refactorization rejects nonzero pivots 9.133998119987652e-11 and 6.8006852399509905e-12 against 1e-10, after 322 completed branch-node LP visits. The second failure is directly observed during the backend’s existing all-slack recovery. 100 factorization calls, 2 captured failures; 0 assignments/proposals/new derivatives/native scoring. KEEP diagnosis; constructor NOT YET PROMOTED. No exact-rank/global-feasibility or model-quality result; accepted 8/512 and separate 9/15 unchanged.

First observer labels incompletely identified recovery. The label-only repair and fresh short trace retain identical basis bytes, pivot bits and iterations; initial source is preserved as an indexed patch. Both sealed traces, bit input, final runtime and build receipts are verified in iCloud (13,264,384 bytes, MD5 32dcdd9a0f2aa1092430490d754f49e2).

## Validation
Exact head `472639b61`: 14 focused Rust fixtures PASS, standalone Rust format PASS, claim wording PASS, non-patch diff whitespace PASS (literal archived patches retain blank context spaces). Independent source/saved-result mathematical and adversarial reviews PASS; parent reviewed final documentation/storage additions. Solver backend failure is the measured result, not a failing fixture. No production numerical code changed. Next: generic factorization stability repair with residuals checked against the original captured basis, preserving all native gates.

## Checklist
- [x] Checks run at this exact head (commands and outcomes posted on this PR)
- [x] Dated research record
- [x] Newest current-state entry ending in Next
- [x] Stale-doc search and follow-through: prior record, training README; geometry role/status unchanged
- [x] STATUS/ROADMAP/#2028 state unchanged
- [x] README capability/headline/command/mechanism unchanged
- [x] Claim wording passes
- [x] Fresh-main verification: merge `d4628eff9362ce3c8cef7de92245b66e80ab2c7a`, all 22 delivered blobs match; remote/local branch and owned worktree removed
- [x] Hygiene completed; final milestone receipt posted to M2 #2030

References #2030


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6092862002 — 2026-10-10T02:38:32Z — Casey-allard

Exact-head review PASS at `472639b619afa4596f1a5043ff0f34f176e3ebab`.

The isolated observer includes the production constructor unchanged, authenticates saved f32 bits, and adds capture calls without solver arithmetic/control changes. Independent source and saved-result mathematical/adversarial reviews PASS. Parent final review confirms the label-only correction, source preservation, complete output seals and iCloud index. Two traces preserve identical basis/pivot/iteration records; recovery identity is direct only in the corrected trace. No claim of rank, conditioning, global feasibility or model improvement.

Executed at this head:
- `cargo test --manifest-path docs/labs/legal-basis-diagnosis-2026-10-09/harness/Cargo.toml --release --locked --offline -- --test-threads=1`: 14 passed, 0 failed. Internal `CARGO_TARGET_DIR`, two compile jobs; temporary/test paths inside the worktree.
- `rustfmt --check --config skip_children=true` on harness main and diagnostics: PASS.
- `python3 scripts/check_claim_wording.py`: PASS.
- `git diff origin/main --check -- . ':!*.patch'`: PASS; literal preserved unified-diff patches intentionally retain blank context spaces.

Actual diagnostic source freeze `7363273b4` and binary `1bba40a7...` are bound in the record. Corrected trace: 3.600 seconds, peak child RSS 94,076,928 bytes; two numeric pivot rejections at 322 previously completed branch LP visits. Backend error is the observed execution boundary; it is not a failed fixture, an accepted update or a model-quality result. All model/native scoring remains NOT_RUN for this diagnosis. Required transport names are queue acknowledgements, not these tests.



---
_Comment/review ids in this file: 6092862002_

