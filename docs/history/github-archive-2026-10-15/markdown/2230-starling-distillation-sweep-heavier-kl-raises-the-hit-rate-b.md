# #2230 — Starling distillation sweep: heavier KL raises the hit rate but breaks the readout; light KL gives the best starling recall, 0.380 (row 3) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:20:26Z
- **Updated:** 2026-10-12T02:22:03Z
- **Closed:** 2026-10-12T02:21:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2230
- **Merged:** 2026-10-12T02:21:34Z
- **Merge commit:** cf30950b23cd260853f23e86cb938675c2b8263d
- **Base / Head:** main / UOR-Foundation:claude/starling-sweep
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 5 (owner-directed continuation) · headline "softmax at runtime: yes" → yes. References #2032.

Murmuration-read ladder, step 1b: **a sweep of the distillation weight and start point** ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116308030) with its amendment; FIT after two review rounds). [Record](docs/labs/starling-distill-sweep-2026-10-12/README.md).

| Arm | Hit rate, step 1800 (mean) | Recall per seed | Mean recall |
|---|---:|---|---:|
| SDw (base) | 0.65 | 0.079, 0.401, 0.085 | 0.188 |
| W3, KL weight 3.0 | **0.765** | 0.022, 0.350, 0.012 | 0.128 |
| **W03, KL weight 0.3** | 0.678 | **0.141, 0.561, 0.439** | **0.380** |
| S33, KL from step 594 | 0.316 | 0.010, 0.608, 0.005 | 0.208 |

**Decision:** row 3 by the bars, best arm W3.

**Beyond the bars:**
- A heavy KL raises the hit rate but breaks the readout: on W3 seed 1, recall is 0.02 even with the source forced into the kept set.
- The light KL gives the best starling recall so far, 62% of the L2 softmax read's 0.617, with no softmax at runtime.
- The teacher-read helper collapses 2 of 3 seeds.

**Also in this PR:**
- a correction to the SDw record (SDw never ran the teacher-read helper);
- STATUS and ROADMAP rows updated.

**Checks:** docs only, with no code change against main; the claim-wording gate passes. The runs used the binary built at `dca29a80b` on pod aoq49jic2vhg1a, with `device=cuda` for all 9; outputs are in the cloud store (md5 fd686ee2…).

**Would the next step differ if this came out the other way?** Yes. It moves the scoring to recall and the base to W03. That step, 1c, is pre-registered and FIT, and is running.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486042729 — 2026-10-12T02:20:32Z — Casey-allard — COMMENTED

Self-review at exact head c11241e5cbea3a69eb34950bf98cd0d151e89f3b: docs only (no crates diff vs main); the claim-wording gate passes; the numbers match the archived logs (claude-sweep-20261012); row 3 is scored per the amended precedence; the SDw correction matches the code (helper alpha is exactly 0 at step 900). APPROVE (self).


---
_Comment/review ids in this file: 5486042729_

