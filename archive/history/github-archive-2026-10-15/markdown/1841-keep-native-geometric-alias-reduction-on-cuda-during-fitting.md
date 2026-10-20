# #1841 — Keep native geometric alias reduction on CUDA during fitting

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T14:23:03Z
- **Updated:** 2026-10-09T13:17:41Z
- **Closed:** 2026-10-09T13:16:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1841
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/native-device-alias-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Offline geometric fitting currently downloads all 4,096 hard Generate scores at every token, builds the complete alias pool and target mask on the host, then uploads action data for the loss. This opt-in CUDA path keeps the authenticated Q24 Generate scores and exact integer Generate/physical-Copy alias reduction on the GPU.

The portable native runtime and default `native_pool_backend=host` path retain their existing behavior. The device path uses the same canonical exponential table, legal vocabulary, physical Copy multiplicity, common clipping, alias sums, tie rules and native probability anchor. Both existing score-credit policies remain available. Private device-bank construction binds the numerical pool to its authentic Context/Copy/Generate graph. Full trace downloads are explicit diagnostics; ordinary updates still download compact status/mass summaries and still perform Context/Copy preparation on the host.

Validation at head `336ff6c0e807bebd40c29247b0c6c202bdc328ab`:

- Independent source review accepted the reducer and repaired bank/fitter integration; execution validation remains separate.
- Touched-file rustfmt and `git diff --check`: PASS.
- Full-workspace `cargo fmt --check`: recorded separately; inherited formatting differences are not represented as a pass.
- Touched-package CUDA/non-CUDA checks and focused CPU tests: pending on two spare pod CPU threads, with GPUs masked.
- Exact CUDA reducer/Generate tests and zero-update retained-bank comparisons: pending after the already-running six fixed fits. Admissions compare authentic host/device loss and active-variable gradients for both score-credit policies; separate legacy and categorical read-state configurations are required. Gradient checks cover the first position of each nonempty phase per admitted episode.

No speedup, full GPU residency, new training outcome or general language capability is claimed. This change does not alter the executable or conditions of the current six-fit comparison.

References #820. Work card: https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6039386314


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081656120 — 2026-10-09T13:16:54Z — Casey-allard

Owner-directed consolidation9October: this exact draft336ff6c0e807bebd40c29247b0c6c202bdc328ab is preserved in the restorable source archive on main delivered by PR2016, with pending compile/CUDA/gradient validation explicitly recorded. It is not adopted or promoted into the active runtime. Closing the delivery draft and retiring its branch avoids stranded work; a justified integration must recover this exact source from main and satisfy its declared checks before delivery. References #820 and docs/labs/consolidation-2026-10-09/runtime-integration-pending-summary.json.


---
_Comment/review ids in this file: 6081656120_

