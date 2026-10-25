# REPORT: infra-loop-harness, claude-92

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904
References #2835
References #2836

## What did not work, first

- **`loop claim infra-loop-harness` is still refused** by Codex's idle draft #3169, which shares no file with this unit. So the ORDER was written by hand, as claude-88 and claude-90 were.
- **`loop queue` for claude-91 (#3183) was refused at first,** because it ran seconds after the PR went ready, before CI had created the five required policy checks. It refuses a missing check rather than waiting for one to appear. It was re-run after CI started. This is a candidate for a harness fix: wait on missing required checks for a bounded time.

## What changed

- **`docs/roadmap/items/m1-bvg-k-block-apply-c2.md`.**
  - It now has `depends_on: [m1-a8-full-dose-rerun, m1-w2-code-utilization]`.
  - `needs_gpu: false`: the remaining deliverable is a docs-only sheet.
  - How-to-start step 1 now gives W2's result (H_mixed, #3177) in place of "confirm W2's status with the PM before drafting".
- **`docs/roadmap/items/m1-bvg-k-stage-t-training.md`.** `depends_on` adds `m1-a8-full-dose-rerun` and `m1-w2-code-utilization` to W2 and C2.
- **`docs/roadmap/README.md`:** re-rendered, with the new edges.

These carry out claude-91's decision (owner log on #2835 and #2836, decision review APPROVE at `46bb1966b`).

## Measurements

None.

## Checks run at the head

`loop report --check` and `loop pr-check --local` were run before the PR went ready (see the PR).

## Skills used

None.

## Still open

- **The umbrella-item lock** (#3169) still forces hand-written ORDERs for every infra-loop-harness unit. See claude-90's Still open.
- **`loop queue`'s missing-check refusal** right after a PR goes ready.

## Plain summary

The roadmap now says what claude-91 decided: the Stage T sheet and Stage T wait for the A8 full dose (a base that keeps its shortlist) and for the code-utilization diagnostic. So `loop next` stops sending labs to a sheet they cannot finish.
