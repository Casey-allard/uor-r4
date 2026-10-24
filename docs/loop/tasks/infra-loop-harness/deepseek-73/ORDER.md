# ORDER: infra-loop-harness, deepseek-73 (refresh two stale roadmap item sections)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `5844d911c`. Umbrella scope: `docs/roadmap/items/*.md`. No item status, bar, hold or plan change.

## Goal

Two `What is missing` sections are stale after deliveries that are already on main:

- **`m2-fresh-heldout-panel`** still says a genuinely fresh test is `NOT_RUN` and that no noise is stated, while `m2-fresh256` is fully staged and sealed (256 rows/128 histories/16 blocks, four pinned artifacts, 0 overlaps against 74 historical literals, all four question-blind/role controls at 0/128, measured 6.24-row cluster SE / 8.64 at the 40% bar, owner bar `pairs_both_complete_distinct >= 52/128` with single-row `>= 104/256`, deepseek-43 FIT prerequisite 1 passed, `frozen: false` only pending the maintainer upload).
- **`m3-nonrelation-prose-panel`** still says no pre-registered set exists, while the 50-turn panel is frozen (`data/panels/nonrelation-prose-v1.txt`, `7c29c665…`, 10 per category, 25 `heldout_v2_disjoint` with zero 4-word runs against the training sources), the deterministic Rust grader + four finite controls are merged, and the pre-registration is `NOT_FIT` with the saved `world=v2` compiler/tokenizer pinned (deepseek-58).

The m1-address item was refreshed separately by Kimi's #3144, so it is out of scope here.

## Cancel probe (run first)

At `5844d911c`: `grep -n 'genuinely fresh test is NOT_RUN' docs/roadmap/items/m2-fresh-heldout-panel.md` and `grep -n 'No pre-registered set exists' docs/roadmap/items/m3-nonrelation-prose-panel.md` both match, while the deliveries above are already merged. The task stands.

## Files in scope

- `docs/roadmap/items/m2-fresh-heldout-panel.md`
- `docs/roadmap/items/m3-nonrelation-prose-panel.md`
- this task folder

## Done when

- Both sections state the delivered instrument, the current FIT/registration state and what remains, with evidence; no item frontmatter, status, bar, hold or scope line changes.
- `xtask roadmap check`/render/sync (dry run), `loop report --check`, `loop pr-check --local`, `loop gates --quick` and `git diff --check` pass; a review records at the head (independent subagent allowed under the owner direction of 2026-10-24T05:29Z when labs are slow).

## Stop and ask if

- Any item status, bar, hold, frontmatter or file outside the list would change.
