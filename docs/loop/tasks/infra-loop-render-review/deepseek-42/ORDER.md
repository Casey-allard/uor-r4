# ORDER: infra-loop-render-review, deepseek-42 (a counted review path for the kimi/render PR)

Lab: deepseek. Item: [infra-loop-render-review](../../../../roadmap/items/infra-loop-render-review.md) (#3036; carded from #2904). Base: `8720c5d6d`. `References #3036`; the item stays `ready` until a render PR is reviewed through the path.

## Goal

Give the steady-state render PR (branch `kimi/render`) a counted-review path: no ORDER, gates `loop render --check` + views-only, and a hand-edited view fails review. The queue's `review-record`/after-approval rules must work for it too.

## Cancel probe

Class instrument (harness code; no GPU). At the base: `loop review` on #3031 fails `adds no ORDER`; `loop queue` refuses it; no render-branch case exists. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs`, `crates/uor-r4-harness/src/loopprcheck.rs`
- `crates/uor-r4-harness/tests/loop_review*.rs`
- `docs/roadmap/items/*.md` (the item itself)
- this task folder

## Design

1. `ReviewPr` gains `head_ref_name` and `is_cross_repository`; parser accepts the sentinel `order: kimi/render`.
2. `render_pr_without_order(head_ref, files)`: branch exactly `kimi/render`, not a fork, files non-empty and all rendered views (STATE, HANDOFF, LESSONS, AGENTS block; roadmap README excluded until its own comparison).
3. `context()` fallback: fetch the explicit head sha; for each view require stamp == merge base(origin/main, head), render at that stamp and compare (`git show <sha>:<path>`); AGENTS changes must lie strictly inside the `loop:lessons-top-ten` markers and the block must equal `render_top_ten` at its stamp. Any mismatch refuses.
4. `loopprcheck::ReviewContext::of` mirrors the fallback (sentinel order, `order_lab: kimi`, empty item, doc-only) so pr-check/queue see the render context; the tamper gate stays in `loop review`.
5. Tests: predicate positives/negatives (branch, views, empty, code path, roadmap), hand-edit caught, sentinel record counts on the render context and not on an ordinary PR.

## Repair addendum (both reviews)

- AGENTS.md block-gated as in Design 3; roadmap README out of the no-ORDER path (follow-up item needed).
- Stamp must equal the PR merge base with `origin/main` (kills branch-only/stale/rollback stamps); explicit-sha fetch replaces `FETCH_HEAD`.
- Forks rejected via `isCrossRepository`; queue fallback added. **The item id/task-folder rename is deferred, not done here:** the scope rule reads the item at the PR's base, so renaming `infra-loop-render-review` in this PR fails `scope`; the gate fix (`is_setup_item`, so only the umbrella and `infra-loop-prNN…` rows are setup items) is what makes the current id reviewable. Kimi's rename should land after this merges, when the scope overlap no longer blocks the claim.
- Still open: a full `context()` fake-git negative-call fixture; roadmap README gating. Both named in the REPORT.

## Done when

- A `uor-review v1` record with `order: kimi/render` counts on a render PR whose covered views match their renders (and whose AGENTS block is inside the markers), with no ORDER; a non-render PR without an ORDER still refuses; a hand-edited view or an AGENTS edit outside the block refuses.
- `cargo test -p uor-r4-harness`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass at the head; a non-author exact-head review is requested.

## Stop and ask if

- Tests would need network or a dependency; the render comparison must stay offline.
- A change outside the scope above is needed.