---
id: past-window-scorer-refusals
title: Refuse unreadable inputs to the past-window scorer
milestone: research
area: evaluation
status: held
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, docs/roadmap/items/past-window-scorer-refusals.md, docs/loop/tasks/past-window-scorer-refusals/**]
---

**November 17, 2026 · retired from the schedule under D68.** Contributor PR #4039 and its merged adoption #4062 addressed the four malformed-input cases and validated the carry-scoring context. For finding 5, the default scorer now checks windows before parsing panel queries, so a panel failing both checks returns the windowing error first. The matched default-output compatibility comparison remains outstanding. The written-state scorers under #4063 use a separate path. Reopen if a named legacy scorer consumer requires the default-path comparison on matched inputs. [November queue decision](../../loop/tasks/infra-loop-harness/codex-10/DECISION.md) · [Written-state experiment](https://github.com/UOR-Foundation/uor-r4/issues/4063) · [Merged contributor adoption](https://github.com/UOR-Foundation/uor-r4/pull/4062)

### Historical registration and evidence

The original rationale, acceptance criteria and work instructions follow.

## Why it matters

A missing or silently zeroed loss can reverse the interpretation of a long-context comparison.

## What is missing

Four malformed-input cases and the default invocation’s compatibility were reported after the original past-window run. PR #4039 supplied the repairs for findings 1–4. Finding 5 remains open for default-output compatibility. Merged adoption #4062 also validates the context actually used for carry scoring. [Adoption and remaining checks](https://github.com/UOR-Foundation/uor-r4/pull/4062) [PR 4039](https://github.com/UOR-Foundation/uor-r4/pull/4039)

## Done when

The scorer refuses the four malformed cases in the original report, with regression evidence. Account for the default-path compatibility observation separately.

## How to start

Compare the implementation PR with all five numbered findings in the original report.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3714)

<details>
<summary>Original report from 2026-11-05</summary>

[claude] Found by claude-287's exact-head review of PR #3696 (#3471). None of these touched claude-287's run: its laptop readout checks that every band with targets has a numeric, non-zero `loss_sum` (156 of 156), that every file's objective is `cell`, and that each pass ran one panel. They are filed here, not fixed in that PR, under the two-round review rule.

**Refusals `--score-rows --past-window` should make, in `crates/uor-r4-float-train/src/in_space_long.rs`:**
1. **No run config:** `run_objective` returning `Ok(None)` prints "no objective" and writes bands with no loss (around `:7583-7594`, and `:5409`). R1, the deciding loss, cannot be read without it, so the pass should refuse.
2. **A losses vector shorter than the bands:** `losses.get(band).copied().unwrap_or(0.0)` (around `:5495`) writes `loss_sum: 0.0`, which would read as the best rule. Refuse when `losses.len() != bands.len()`.
3. **Carry with two panels of different C:** `model_context` is the largest C over the splits (around `:7554`), so the shorter panel decodes at the longer C while its file says its own. Refuse carry when the panels' C differ.

**Also from the review:**
4. **`build_document_blocks` has no guard for W = 0** (`hamilton_sheet.rs`, around `:1993`). Its loop would never advance. The base `build_document_windows` has the same hazard, and W comes from the checkpoint. Refuse W = 0, as the sliding windower already does.
5. **No CPU test pins `score_rows`' default path.** "Without the option, every output is as at the base" rests on reading the code, since `score_rows` runs on the GPU. One refusal order did change: `windows_of` now runs before `load_panel_queries`, so a panel failing both checks reports the windowing error first.

The next unit that runs this scorer (claude-288's re-read, or the next long checkpoint) should land 1 to 4 with a test and a negative gate each.

</details>
