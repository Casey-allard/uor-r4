# #1670 — Learn geometric context and isolate its native reply regression

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T04:18:07Z
- **Updated:** 2026-10-09T13:17:19Z
- **Closed:** 2026-10-04T04:25:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1670
- **Merged:** 2026-10-04T04:25:48Z
- **Merge commit:** 9aa249c0a2d85c584ca090f60db71ba986cdd9fe
- **Base / Head:** main / UOR-Foundation:codex/geometric-context-adaptation-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native source realizer already computes answer credit for geometric context, but its readout-only continuation discarded that credit. Add opt-in `context-fit` using the existing Rust learner and strict saved checkpoint32 lineage, followed by `context-transplant` to isolate context/readout changes through exact zero-update family replacement, normal export/reload and actual generated replies. No model/runtime/session interface or inference mechanism is added.

On the same 20 preservation + 8 construction cases, joint64 context/readout learning changes the packed context but regresses original replies20→6; construction remains0/8. Development transfer3/16 includes3gains and2losses, with two gains sharing a source. Preserve all stages and new repeated-source capability. The zero-update matrix gives old/old20, old/new20, new/old4 and new/new6 originals, all0/8 construction. Context changes alone are sufficient for broad preservation loss; identical actual-prefix traces also expose a later EOS interaction. This does not prove a gradient defect, geometry incapacity or general chat. The roadmap now points to a bounded legal-quarter native context direction diagnostic before another fit.

Validation: committed-source CPU-only Runpod Linux x86_64; context sourceaca9173c passes11 focused Rust tests/release build and64 updates; final diagnostic source562a8013 passes12 tests/release build and zero-update execution. Rustfmt/claim-wording/diff checks pass. Independent reviews verify3,268 training positions,220 fit replies,112 diagnostic replies, all family mosaics/parent inventories, row gains/losses and8 full native traces. All134 fitfiles/277,204,456B and58 diagnosticfiles/99,025,583B are returned with exact file-set/SHA/size checks plus both executables. Fit305.331s, diagnostic12.207s; resource receipts and conservative complete90min ledger charge are bound in evidence. NoGPU/GitHubresearchrunner/laptopmodelCPU; Metal unavailableLinuxnotcounted. Pod card released, ownedarea~6.55GiB. CI compatibility acknowledgements are not executed tests.

References #1552 and #1512. All candidates remain unadopted; shared session code is untouched.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

