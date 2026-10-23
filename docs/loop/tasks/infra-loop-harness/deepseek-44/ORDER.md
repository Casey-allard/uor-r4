# ORDER: infra-loop-harness, deepseek-44 (close the render-review item; card its hardening)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `885df8f3e`.

## Goal

The render-review path is on `main` (#3035, `6e9d6db41`) and its first real subject merged (#3038, `885df8f3e`). Close the item it was carded from (`infra-loop-render-review` -> `done`) and card the review follow-ups as one new ready item so they are not lost: the `context()` fake-git fixture and negative gates, the xtask-rendered roadmap README tamper comparison, and the loop-page wording (owner-only, so a note for the PM).

## Cancel probe

Class **instrument** (docs metadata; no GPU). At the base: `docs/roadmap/items/infra-loop-render-review.md` still says `status: ready` although #3035 and #3038 are merged, and no follow-up item exists. Task stands.

## Files in scope

- `docs/roadmap/items/*.md` (the status flip and the new follow-up item)
- `docs/roadmap/README.md` (re-rendered)
- this task folder

## Done when

- `infra-loop-render-review` is `status: done` with the merged evidence in its own file.
- A new ready item carries the three follow-ups with a concrete done-when and a planted fault for the fixture.
- `roadmap check`, `roadmap render --check`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; a fresh-context record covers the docs-only PR.

## Stop and ask if

- The loop page itself must change (owner-only; card it for the PM instead).
