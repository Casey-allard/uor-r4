# #1576 — Learn temporal query views in the saved grounded-session path

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T14:47:33Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-10-01T14:58:10Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1576
- **Merged:** 2026-10-01T14:58:10Z
- **Merge commit:** 23acc3393ea642b57df5a8e6ac7982f9c5d84372
- **Base / Head:** main / UOR-Foundation:codex/learned-temporal-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The grounded session exposes temporal store views, but its real saved compiler can only request the current value. Add a separately learned query-view adapter over the unchanged relation/action/span artifact, and load the complete adapter through the existing CLI and m-world session.

The new head learns from source words and ordered adjacent pairs using the existing sparse cross-entropy learner. It decorates only base-predicted queries, preserves current queries and all nonquery actions, and binds original base bytes, parameters, feature/label order, training rows/groups and settings. Joint scoring includes wrong base gates/relations; separate head/current-only/source-free diagnostics and input overlap remain visible. Labels describe recorded events, not assumed real-world chronology.

Validation: [exact-head CI36879130765](https://github.com/UOR-Foundation/uor-r4/actions/runs/36879130765) passed compilation of the library, m-world and both CLIs;4temporal tests,1actualCLI integration,9legacy session tests and9compiler tests, plus2explicitfresh-process helpers. Nonauthor review approved the same head. The actual CLI witness fits/saves/reloads then compares9ordinary-text fresh-process turns with in-process generation, including exact historical record IDs and saved generated token IDs/history. Authored training rows and initialized emitter remain construction fixtures, not unseen temporal-language or trained-emitter capability evidence.

The inspected advisory publisher's NotFound failure stays separate. D19manualfallback uses actual review/checks followed by protectedqueue; no statusfabrication oradminbypass.

References #962, #1552. Existing paired-fit/data-review work stays unchanged; no local model/Cargo worker or SSD write is launched under the owner repair pause. The selected trained-emitter temporal complete-answer witness remains NOT_RUN.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5933915541 — 2026-10-01T14:49:13Z — Casey-allard

Nonauthor source review — APPROVE at exact commit `b507167a28eb1354c57b5f53ff19903ca5691e10`, base `35c2c1f6b42d4dc6b1e8e6be7c405b18755f3ffd`, all8changedpaths. Reviewer `/root/adoption_review_evidence` authored none. Module SHA256 `cefd3dd042717cb28ce7b61316a81888694ff728cfced5721e45b7d61a7f373c`; clean inspected tree. No remaining fixes.

Reviewed reuse of sparse learning with ordered source features, actual loss/label-swap/source-free diagnostics, exact frozen-base composition/canonical parameter binding, strict schema dispatch, passthrough writes/corrections, joint scoring of wrong gates/relations, recorded-history semantic fixtures, recordIDs4/1/3/2, controls, full generated-ID/history persistence and actualCLI adapters. Three draft findings repaired beforefreeze: samebuffer parsed/filehash; wrongrelation sentinel cause asserted; freshprocess equality supplemented by independently specified views/recordIDs and fullsavedstate. Sharedbasefile only5visibilityadjustments. Source revision is buildprovided commit orUNAVAILABLE, alongside actualexe digest.

Reviewer executed no Cargo, fit, model or tests. Free exact-head CI36879130765 remains pending. This approval is construction/source evidence; selected trained-emitter temporal complete-answer witness remains NOT_RUN during SSDpause, without language/geometric/D11 promotion.


### comment 5934058514 — 2026-10-01T14:57:13Z — Casey-allard

Actual executed checks PASS at exact head `b507167a28eb1354c57b5f53ff19903ca5691e10`, base `35c2c1f6b42d4dc6b1e8e6be7c405b18755f3ffd`. Free [CI36879130765](https://github.com/UOR-Foundation/uor-r4/actions/runs/36879130765) explicitly checked out that source and bound UOR_BUILD_SOURCE_COMMIT. Operational workflow commit4a2229412711dbe89091b6b290fe7a1aa1809b44 is separate.

Executed: workspaceformat, claimwording/policyJSON/diff; traininglibrary+m-world+grounded-session+temporal-compiler compile (57.34s);4temporal parent tests+explicitfreshchild1passed (testbuild4m42s, tests0.64s); actualCLIintegration1passed (build30.72s,18.25s execution);9legacy sessiontests+explicitfreshchild1passed;9frozencompiler tests. No failed tests. Child helpers are ignored only in parent enumeration and explicitly invoked.

The actualCLI test fits/saves the head, independently loads its composite, executes fit/score binaries and9ordinarytextturns oneprocessperturn, asserts recordIDs4/1/3/2 andexactvalues for all4views afterA→B→C→C, and compares fullsavedTurnOutcome/generatedIDs/loadedhistory with inprocess generation. The deliberate wrongrelation scoring sentinel verifies sourceview correct1/1 but joint0/1. Authored rows/initialized pointericosian emitter are construction evidence, not unseenlanguage/selectedtrainedemitter, geometricadvantage orD11serving qualification. Sourcelearning test verifies lossdecrease, relabelledpredictions, orderfeatures and sourcefree diagnostic.

Nonauthor exacthead review: [comment5933915541](https://github.com/UOR-Foundation/uor-r4/pull/1576#issuecomment-5933915541). Advisory publisher36879140392 was inspected: NotFound, notPASS. D19manualfallback uses recordedreview+actualchecks thenprotectedqueue, with noadminbypass/statusfabrication. No localCargo/modelworker/SSDwrite orresearchfit was launched. Selectedtrained-emitter temporalcompleteanswer witnessNOT_RUN.



---
_Comment/review ids in this file: 5933915541, 5934058514_

