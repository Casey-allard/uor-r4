# #1639 — Matched controls: U(1) recurrence lanes and a flat L2 read

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T00:51:43Z
- **Updated:** 2026-10-09T13:08:43Z
- **Closed:** 2026-10-03T01:46:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1639
- **Merged:** 2026-10-03T01:46:33Z
- **Merge commit:** 38377c87eae2db25ec3d2fa417bdbc9c8ae83649
- **Base / Head:** main / UOR-Foundation:codex/controls-u1-l2
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**Why.** Step 2 of the owner-adopted audit plan (#820) needs matched controls, to test whether quaternion transport and the Lorentz read carry language ability.

**Options added** to both `train` and `dialogue-train`:
- **`rotation_group=quaternion|u1`** (default `quaternion`). U(1) lanes zero the j and k components of each raw transport quaternion before normalization, so all transitions commute.
  - The same gates and parameter count are kept, and the j/k gate gradients are exactly 0.
  - `u1` requires `rotation=true` and is refused for transformer arches.
- **`read=l2`.** A flat read with score `-beta*(||q-k|| - offset) + age`, computed in f64 with a clamp and an exact backward. It uses the same `beta`/`offset` parameters as Lorentz, so parameter counts match.

**Compatibility and scope.**
- `rotation_group` is serde-defaulted and skipped when it is quaternion, so old configs load and re-save byte-identical with bit-identical logits (tested).
- Both controls are refused by QAT, transport snap, export and pointer-L2.
- On Metal they use the existing host fallback.

**Tests:**
- `u1_recurrence_matches_composed_mixer_and_finite_differences`
- `u1_transitions_commute_and_differ_from_quaternion`
- `fused_l2_read_matches_reference_and_finite_differences`
- `l2_read_scores_equal_negative_beta_distance_on_a_hand_example`
- `configs_without_control_fields_load_unchanged`
- `geometric_stack` module: 99 passed. `stack_export`: 13 passed.
- 30-step smokes: all three arms at 7,155,396 parameters.
- GitHub evidence: `codex/ci/v3/controls-u1-l2`.

**Measurement.** Running on free runners: `codex/ci/farm/controls-u1-l2`, 3 arms × 2 seeds × 1,000 updates, identical public data with checked hashes, reports only. Results will be posted on #820.

The code was written by a Claude subagent and reviewed by the Claude lab.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5398342140 — 2026-10-03T01:28:31Z — Casey-allard — COMMENTED

Exact-head review at 121ab87b7b5173b288628ec56786eb82fbbbcd8c (Claude lab; self-review per owner rule on #820). GitHub evidence https://github.com/UOR-Foundation/uor-r4/actions/runs/37083811275 (parent = this head): fmt, check of all targets, lib 591 passed / 0 failed. The diff was written by a Claude subagent. I checked the report and the code paths: U(1) zeroes j/k before normalization with exactly zero j/k gate gradients; L2 shares the Lorentz beta/offset parameters, so parameter counts are equal (7,155,396 in the smokes); old configs are serde-defaulted and tested byte-identical; both controls are refused by QAT, snap and export. Focused tests: the five new ones, geometric_stack 99 passed, stack_export 13 passed.

### comment 5964250942 — 2026-10-03T01:47:32Z — Casey-allard

Merged as 38377c87. The six files that also carry other merged changes on main received exactly this PR's changed lines (sorted changed-line comparison against the merge-base diff: geometric_stack.rs 688 lines, the five others 1 line each). Every other changed file equals the reviewed head.


---
_Comment/review ids in this file: 5964250942, 5398342140_

