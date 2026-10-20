# #2049 — Add bounded ordered-pair credit to native episode learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:54:52Z
- **Updated:** 2026-10-09T15:59:26Z
- **Closed:** 2026-10-09T15:56:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2049
- **Merged:** 2026-10-09T15:56:25Z
- **Merge commit:** 6f8bf90ff31926b8e065c7e1930e923c60c293f1
- **Base / Head:** main / UOR-Foundation:codex/bounded-pair-credit-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The complete-episode learner currently extracts and proposes only Generate unary coefficients, leaving the existing ordered-pair family frozen. Add explicit pair-family selection with a required 1–960 coordinate limit, ranking the full actual pair family once from its original gradient and testing all fourteen legal alternatives per selected coordinate. Keep the complete native objective, fifteen episode positions including EOS, seventeen references, 380 guards and all other parameter families unchanged.

Pair artifacts now require their own floating-master hash and pinned proposal bound. Authentication reconstructs the saved full-family ranking and committed masters and checks the frozen fractional U payload, including changes invisible in packed Q4 codes. Omitted family retains unary behavior. No serving rules or token-specific exceptions are added.

Validation: optimized locked/offline training-library build; 15 Generate and 16 Prefix/artifact tests pass, rerun after current-main training changes; named formatting, diff and claim-wording checks pass. Mathematical and artifact-authority source reviews are retained. Two initial test tokenizer setup failures were repaired; one review finding about fractional U preservation was repaired and tested. Evidence and test executables are verified in iCloud.

Scope: implementation and fixtures only. Fresh actual-parent fitting, CUDA execution, whole-answer qualification and a new 512-row evaluation are NOT_RUN. Accepted 8/512 and unselected coupled 9/15 conditional results are unchanged. Next, after merge verification and workspace/branch cleanup, recover the original selected parent and admit a fully costed bounded pair run. A positive construction must immediately receive the existing actual whole-answer/EOS check.

References #2030.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6084421281 — 2026-10-09T15:55:33Z — Casey-allard

Exact-head validation and review: `f69d84e1461c8f3e4842e170aadcb8ae57e7e572`.

Optimized locked/offline example tests were executed again at this head: Generate **15/15**, Prefix/artifact **16/16**. Changed source SHA-256 values match the committed validation receipt; no source edits followed testing. Training-library compile, named-file formatting, diff check and claim-wording check pass. Independent mathematical review found no scoped arithmetic blocker; artifact review's fractional-U omission was repaired, source-reviewed and tested with same-Q4 fractional tampering. Final author diff review found no remaining blocker in this scope.

No fresh actual-parent fit/CUDA run or autoregressive model evaluation was performed. Fixture success does not change accepted8/512 or conditional9/15. Source, logs, first test-setup failures and review scopes are committed; test evidence/executables are MD5-roundtrip verified in iCloud. CI compatibility acknowledgements are transport only.


### comment 6084463552 — 2026-10-09T15:57:52Z — Casey-allard

Protected delivery verified: merge `6f8bf90ff31926b8e065c7e1930e923c60c293f1` is on fresh origin/main. All17 changed files have identical blobs at reviewed head, actual merge and main. GitHub delivery branch removed; Casey checkout fast-forwarded and clean main. Local worktree/build removal and mandated hygiene follow immediately. M2 result updated; accepted8/512 and conditional9/15 unchanged.


### comment 6084491643 — 2026-10-09T15:59:26Z — Casey-allard

Cleanup completed after verified merge: GitHub and local `codex/bounded-pair-credit-20261009` branches removed; owned managed worktree and its build cache removed. `/private/tmp/identity-land` was verified against merged #2047 and removed after its DeepSeek process finished; its redundant6.8GiB build folder is gone. Casey checkout is clean `main` at `6f8bf90ff31926b8e065c7e1930e923c60c293f1`. Both test evidence/executable archives are MD5-roundtrip verified in iCloud.

Ran current `scripts/storage/uor-hygiene --apply`, scoping its local-result scan to the now-removed owned workspace. Global worktree/cache checks preserved DeepSeek’s new active `/private/tmp/m4-land` build and the dirty entry-isolation tree; git GC completed, busy uv cache pruning skipped. Final observed free space40GiB. No pod was provisioned. A remaining retired lab-state heartbeat-writer delta is recorded on #2028, comment6084433048, rather than discarding another participant’s unarchived commit.



---
_Comment/review ids in this file: 6084421281, 6084463552, 6084491643_

