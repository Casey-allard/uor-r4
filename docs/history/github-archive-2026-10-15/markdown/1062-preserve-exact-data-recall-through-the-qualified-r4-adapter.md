# #1062 — Preserve exact-data recall through the qualified R4 adapter (#1061)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T19:46:52Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-02T19:48:45Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1062
- **Merged:** 2026-09-02T19:48:45Z
- **Merge commit:** 62c64ccf2cba7674e27628775e78fb81c44baee0
- **Base / Head:** main / UOR-Foundation:issue-1061-exact-r4-inference
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained #1057 exact-data model had not yet been exercised through the qualified R4 adapter. This adds a sibling inference entry point that binds its final block-40 checkpoint and original development data while reusing #1059’s adapter, frame map, model loader, scorer and execution policy unchanged. The model and data live in separate evidence roots; only development tensor values are loaded.

Plain and coherent R4 both preserved **8,071/8,192 correct (98.5229%) with all 8,192 prediction IDs identical**. Maximum logit difference was `2.5272369384765625e-05`; attention difference was `2.2351741790771484e-06`, within the frozen tolerances. The new inconsistent-transport control scored **1,009/8,192 (12.3169%)**, an **86.2061 percentage-point loss**, with matched causal support and work.

Exact fresh-process replay passed. The complete run and replay took **33.79 seconds** on four CPU/Apple Accelerate threads, with **2.411 GiB** peak RSS. Learned state and tied weights remained unchanged; no training or checkpoint/optimizer/RNG/old physical-binding-control reads occurred. The old #1057 training terminal and unrun control remain historical.

Preparation, raw result, replay, operator instructions, and current programme direction are committed. The next recommendation is a separately frozen English supplied-context binding curriculum using the working architecture and R4 adapter. This delivery adds no English, geometric-superiority, softmax-removal or table-runtime claim.

Validation: six new focused wiring/decision/accounting checks; unchanged source/data/frame/188-file implementation bindings; matched inference/control and exact fresh-process replay; scoped Ruff, claim-wording and diff checks; independent source/evidence review. Historical adapter/native tests and builds were reused, and broad QA remained dormant. Required CI jobs are delivery acknowledgements.

Source froze at `42d5eb34`, and preparation was committed/published at `6d75643f` before fitted inference. No implementation changed afterward.

Closes #1061. References #973 and #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

