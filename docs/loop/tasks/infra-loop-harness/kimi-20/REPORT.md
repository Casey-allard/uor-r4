# REPORT: infra-loop-harness, kimi-20 (the loop page documents the render refresh recipe)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6 (the render chain, as with kimi-17..19 and deepseek-44).
Base: `aac963833`. ORDER: [ORDER.md](ORDER.md), committed first (`7032430e6`). Closes card #3043.

## What did not work, first

- The loop page's "The render PR" paragraph said the refresh "merges `origin/main` forward, renders, commits and pushes" without saying how the stamp is chosen. On #3038 that cost a full refused push cycle: after the merge-forward the merge base is main's tip, not the merge commit, and a plain `loop render` stamped the branch head's own commit, which the gate refused by name (`stamp 1c2daefc8… is not the PR merge base a4a5e1efb…`).
- No code change: giving `loop render` a merge-base default was the card's alternative, but a code default touches the harness during a four-lab build window; the doc recipe is the smaller fix and the card's first option.

## What changed

| Commit | What |
|---|---|
| `7032430e6` | The ORDER, alone. |
| `f5c4ce9d5` | `docs/loop/README.md`: the "The render PR" paragraph now names the stamp rule and the recipe — merge `origin/main` forward, then `uor-harness loop render --at "$(git merge-base origin/main HEAD)"`, commit, push, never by force. |
| this REPORT's commit | This REPORT. |

## Measurements

- Class **instrument** (docs metadata; no GPU, no model compute), matching the ORDER's cancel probe (`grep -n -- '--at' docs/loop/README.md` found nothing at the base).
- Diff size outside this task folder: 1 file, 1 paragraph (1 line changed).

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`: run in this worktree before the PR opened; results in the PR body. `docs/loop/README.md` is a shared file, always allowed by the scope rule.
- The recipe itself is the one verified end-to-end on #3038 at `10dc9299b` (merged as `885df8f3e`).

## Skills used

- None beyond the loop flow itself.

## Still open

- Cards #3041 and #3042 (harness bugs in `prep.rs`) remain open and unassigned; both touch `prep.rs`, which sits outside the umbrella's scope, so each needs a scope widening first.

## Plain summary

The render gate checks that every view's stamp equals the PR's merge base with origin/main, but the loop page never said so. After a merge-forward that base is main's tip, so the refresh must render at it explicitly. The page now says exactly that.

References #2904
Closes #3043
