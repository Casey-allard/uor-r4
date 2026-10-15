# #2224 — Starling selector distillation: from step 0 it stops learning; started half-way it lifts the hit rate to 0.61-0.68 (row 3) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T01:41:08Z
- **Updated:** 2026-10-12T01:42:36Z
- **Closed:** 2026-10-12T01:42:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2224
- **Merged:** 2026-10-12T01:42:10Z
- **Merge commit:** dca29a80b152450ce7eb9ab763adc62ae8dae4cf
- **Base / Head:** main / UOR-Foundation:claude/starling-distill
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 4 (owner-directed continuation after 3/3) · headline "softmax at runtime: yes" → yes. References #2032.

Murmuration-read ladder, step 1: **selector distillation**, a training-only KL from the dense content teacher onto the 600-cell addresses. [Record](docs/labs/starling-distill-2026-10-12/README.md).

| Arm | Recall, mean | Hit rate, step 1800 | Row |
|---|---:|---|---|
| SGS (base, #2222) | 0.141 | 0.39–0.45 | — |
| SD, KL from step 0 | 0.006 | 0.00–0.10 | 4a by the letter, but the arm did not train |
| SDh, diagnostic (KL ≈ 0) | 0.045 | 0.00–0.42 | D2: the KL from step 0 was the cause |
| **SDw, KL from step 900** | **0.188** (seed 2: 0.401) | **0.61, 0.68, 0.66** | **3, partial** |

**Code:**
- `set_starling_distill`; only training forwards build the teacher.
- `mqar-bench starling_distill=WEIGHT starling_distill_start=FRACTION`; the KL is logged separately.
- The mqar-bench test target compiles again: its test literals had drifted on main.

**Runtime:** unchanged. The served read is still softmax-free.

**Exact-head checks**, on the pod's CPU at `ba83fa7f2`, whose code tree is byte-identical to this head (the rebase brought only docs):
- starling lib tests: 11/11;
- mqar-bench tests: 16/17. The one failure is `the_transformer_control_is_parameter_matched_to_the_geometric_arm`, a frozen parameter count in a test that never compiled on main; it is filed separately.
- rustfmt: clean on this PR's hunks.

**Self-review:**
- The KL is gated by the training flag.
- Start is at exactly step 900, after the anneal reaches 0 (verified by the adversarial review, items 1–5).
- The values stay detached in the surrogate.
- No new parameters, nothing saved or served.

**Next:** the distillation weight and temperature sweep, with the rank-table readout.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5485912883 — 2026-10-12T01:41:21Z — Casey-allard — COMMENTED

Self-review at exact head 9872ddf8a1b2da7e112139257b26f3e4c517f66e: code tree identical to ba83fa7f2 where the pod ran starling lib 11/11 and mqar-bench 16/17 (pre-existing frozen-count transformer-control test, filed separately); rustfmt clean on this PR's hunks; teacher gated to training forwards; KL from step 900 after the anneal; values detached; no runtime change. APPROVE (self).


---
_Comment/review ids in this file: 5485912883_

