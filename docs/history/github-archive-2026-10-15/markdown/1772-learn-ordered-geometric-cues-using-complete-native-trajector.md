# #1772 — Learn ordered geometric cues using complete native trajectories

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T00:52:48Z
- **Updated:** 2026-10-09T13:17:22Z
- **Closed:** 2026-10-07T00:07:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1772
- **Merged:** not merged
- **Merge commit:** 26426df9c4e3196b098e05f060e53416d4bc0cc6
- **Base / Head:** codex/geometric-cue-joint-foundation-20261005 / UOR-Foundation:codex/geometric-cue-joint-learning-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The first ordered joint cue intervention lowered CE while losing a complete reply. This adds a bounded joint-only learner that proposes legal quarter updates from CE gradients and selects independently reloaded native generation by complete answers first, then CE. The source encoder/bank, unary cue, prefix and endpoint payloads remain frozen.

Executed over all512 development rows: six accepted updates from16 proposals improve complete replies59→74 (18gains/3losses), first factual source280→296, first emitted token219→229, and native CE0.9665787503→0.9544226152. Three accepted updates worsen CE locally while improving complete output. The previously harmful first quarter is reproduced and rejected. Trial receipts preserve causal metrics and row gains/losses against every prior artifact, including rejects. No held-out predictions ran; this is one development candidate, not transfer, general chat or serving qualification.

Validation: all7 scoped release commands PASS at numerical source85a9e2e6:2 integercarrier tests,2 exactjoint library tests,45 fitter tests,52 observer tests,formatting and releasebuild. Independently reviewed source and saved16trial decisions. Checks344.572s CPU8; native fit228.176s CPU16, peakRSS5,051,830,272B on Linuxx86 pod3170534ffd7a with CUDAhidden. Executabledd407f18…; config12e2f8f7…. Full sealed report465,953,287B retained under /workspace/uor-r4/codex/cue-joint-learning/fit-1. Initial commandwrapper failed before checks/model, retained NOT_RUN.

Final documentation headd6efa651377bc4acc232247a170b17835ff9b94b changes only current-state/plan;1263 declared numerical-source files equal the tested head. No rerun claimed for documentation-only commit. Complete resource receipt preserves earlier charges/manual admission; global legacy importer reconciliation remains unverified.

Next: frozen baseline/selected/order controls and untouched supported bank-composition evaluation; no refit or unchanged dose extension. DeepSeek softmax/tensor-core seam is separate.

References #820, #1768.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6007084570 — 2026-10-06T01:01:52Z — Casey-allard

Exacthead `85a9e2e6322dda65088bea5af4f9f60d8f4ca897`: scoped pod release checks PASS, all7 declared commands exit0 (formatting,2 integercarrier tests,1 joint roundtrip/frozen-family test,1 orderedouterproduct/missingroot test,fitterexample tests,observerexample tests,releasebuild). Host `3170534ffd7a Linux x86_64`, CPU8/CUDAhidden, 344.572s, peak owned process-group RSS 4148494336B;1263 declared numerical-source files checked against committed snapshot. Source review at this exacthead independently approves the evidencefixes; source and build evidence are separate. Boundedfit stillrunning; no result ortransferclaim. Durable receipt `/workspace/uor-r4/codex/cue-joint-learning/checks-1.json`.


### comment 6027854154 — 2026-10-07T00:07:33Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6007084570, 6027854154_

