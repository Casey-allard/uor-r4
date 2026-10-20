# #1060 — Preserve learned associative recall through coherent R4 inference (#1059)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T19:20:40Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-09-02T19:23:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1060
- **Merged:** 2026-09-02T19:23:37Z
- **Merge commit:** 83e1e90d9fae8e92457e32f16223996f5389f4ed
- **Base / Head:** main / UOR-Foundation:issue-1059-zoology-r4-inference
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The qualified Zoology recall model previously ran only through its ordinary attention cell. This adds a parameter-free coherent R4 inner-attention adapter and exports the complete 8,192-token frame map through the existing Rust `R4SpinFrameAtlas`, preserving the original learned tensors, positions, normalization, residuals, softmax, and tied head.

The frozen inference comparison preserved **11,900/12,000 correct decisions (99.1667%) with every plain/R4 top-1 prediction identical**. Maximum logit difference was `2.002716064453125e-05`; maximum attention difference was `8.940696716308594e-07`. Deliberately inconsistent source transport reduced recall to **19.225%**, a **79.9417 percentage-point loss**, with identical causal support and work. Exact fresh-process replay passed; run plus replay took **27.49 seconds** on four CPU threads with **1.844 GiB** peak RSS.

The preparation, raw result, replay, run contract, and current programme summaries are committed. This is fixed-weight behavior preservation and transport sensitivity on the retained open Zoology population. It adds no training, geometric-superiority, English-generation, or exact/table-runtime claim. #1057 remains preserved for the next separately scoped inference application.

Validation: native exporter build/export; eight focused synthetic checks including native witnesses, state/tie preservation and causal isolation; source/frame/implementation CID checks; the matched inference and control; exact independent replay; scoped Ruff/rustfmt, claim-wording and diff checks; independent source/evidence review. Broad workspace/product QA remained dormant under the issue contract; required CI jobs are delivery acknowledgements.

Implementation and preparation were frozen and posted before fitted inference at commit `800e7d52`; no code changed after the freeze.

Closes #1059. References #973 and #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

