# #2134 — Qualify saved LU bases and retain later constructor rejections

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T03:10:17Z
- **Updated:** 2026-10-10T03:18:03Z
- **Closed:** 2026-10-10T03:11:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2134
- **Merged:** 2026-10-10T03:11:32Z
- **Merge commit:** 9763ec89789d86a64a4ab388db2c54492dedd207
- **Base / Head:** main / UOR-Foundation:codex/basis-residual-repair-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The saved legal constructor rejected nonzero LU pivots using its backend's absolute EPS. This unit tests a generic scale-aware rejection rule and verifies every fresh factor against the original basis through all unit RHS, both orientations and both dense/sparse solve paths. It retains the unchanged legal problem, global simplex EPS and native gates. The patch is an isolated offline experiment, archived rather than activated as a production dependency.

## Result

Both prior 440-dimensional bases pass 1,760 unit solves each, with maximum normalized backward error below9e-17 against the fixed1.250555e-11 threshold. The one unchanged saved-constructor replay completes148 fresh-factor verifications out of150, then rejects two later bases during the same branch visit/recovery and returns no assignment. KEEP numerical evidence; backend integration NOT YET PROMOTED. No native proposal, new derivative, artifact reload or language result. Accepted8/512 and separate conditional9/15 remain unchanged.

The next causal question concerns eta-updated solves and the pivot transition against the original old basis before membership changes. Eta drift is not established. Full sealed outputs, original/later bases, executable, source and setup failures are verified in iCloud; the unpromoted numerical patch and INDEX row are included here.

## Checklist

- [x] Exact-head checks:13 harness tests and5 focused numerical fixtures; changed Rust source formatting; claim wording; whitespace excluding literal archived patch data
- [x] Dated research record with exact artifact, source, configuration, numbers, decision and limits
- [x] Newest current-state entry ending in Next
- [x] Stale-doc search performed; training README and prior diagnosis linked to follow-through
- [x] STATUS/ROADMAP/#2028 unchanged: no model/milestone change
- [x] Top-level README and geometry figures unchanged: no capability or geometric role/fact changed
- [x] Peer #2133 merged into the delivery branch; all peer material preserved
- [x] After merge: verify fresh main and source/tree; delete remote/local branch and worktree; run storage hygiene; report the merged result on M2

References #2030



## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093169378 — 2026-10-10T03:10:40Z — Casey-allard

Exact-head review and checks: 2ea9b53c247601d9090da8e4f6fd7455dc55f179.

Independent adversarial reviewer PASS at this exact delivery head. Parent source/result review PASS. Mathematics review covers fixed scale-aware pivot policy and original-basis normal/transpose residuals; final diagnostic-only installer changes independently reviewed. Diff contains18 intended paths and no foreign deletions after merging fresh main bf70a555c/#2133.

Executed at this head:
- cargo test --offline --release --manifest-path docs/labs/basis-residual-repair-2026-10-09/harness/Cargo.toml -- --test-threads=1:13/13 PASS (9 constructor,2 mapping,2 report-output).
- cargo test --offline --release --manifest-path docs/labs/basis-residual-repair-2026-10-09/tests/Cargo.toml -- --test-threads=1:5/5 PASS. The dependency's internal cfg(test) module was not executed.
- rustfmt --check --edition2021 --config skip_children=true on the three new Rust source/test files:PASS.
- python3 scripts/check_claim_wording.py:PASS.
- git diff --check origin/main excluding literal *.patch data:PASS; archived dependency patch git apply --check on exact corrected observer base:PASS.

Measured execution remains pinned to source1fe9602d2 and binary0e929bcf61b9107d0f1d904b035dbcc1f77b2b96cd3e5c6621fd9be71d97c7c3. Relevant implementation bytes are unchanged; no experiment was repeated for documentation/main merge. Both old bases qualify; unchanged constructor still returns noassignment after148/150 factors and2 later rejections. No native/model result or global infeasibility claim. No production backend activation. Backend patch+INDEX and complete verified private results preserve the unpromoted work.

The five required GitHub statuses are transport acknowledgements, not these tests.



---
_Comment/review ids in this file: 6093169378_

