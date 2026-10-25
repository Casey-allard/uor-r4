# ORDER: infra-loop-harness, claude-92 (route the Stage T sheet and Stage T behind their prerequisites)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in kimi-27). Base: `4a0acba8c`.

## Goal

Carry out claude-91's decision, recorded in the owner log on #2835 and #2836 (`6393771254`, `6393771432`), whose decision review approved at `46bb1966b`.

1. **`m1-bvg-k-block-apply-c2` (#2835).**
   - Set `depends_on: [m1-a8-full-dose-rerun, m1-w2-code-utilization]`.
   - Set `needs_gpu: false`: its remaining work is a docs-only sheet.
   - In how-to-start step 1, replace "confirm W2's status with the PM before drafting" with W2's result (H_mixed, #3177).
2. **`m1-bvg-k-stage-t-training` (#2836).** Add `m1-a8-full-dose-rerun` and `m1-w2-code-utilization` to its `depends_on`. It already lists W2 and C2.
3. **Re-render** `docs/roadmap/README.md`.

## Cancel probe (run first)

At `4a0acba8c`:
- `UOR_LAB=claude uor-harness loop next` went GO on `m1-bvg-k-block-apply-c2`, and claude-91 cancelled that sheet on its precondition.
- Both item files lack the two prerequisites.
- `loop claim infra-loop-harness` is refused by Codex's idle #3169, which shares no file with this unit. So this ORDER is written by hand, as claude-88 and claude-90 were.
- No cycle: `m1-a8-full-dose-rerun` depends only on `m1-a8-finite-cuda-qualification`, and `m1-w2-code-utilization` only on `m1-w2-capacity-capture`.

The unit stands. Class: instrument (roadmap routing).

## Files in scope

- `docs/roadmap/items/m1-bvg-k-block-apply-c2.md`
- `docs/roadmap/items/m1-bvg-k-stage-t-training.md`
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-92/**`

## Done when

- After the merge, `loop next` no longer offers `m1-bvg-k-block-apply-c2`.
- The render shows the new edges.
- A fresh-context review at the exact head counts.
- The PR says `References #2835` and `References #2836`.

## Stop and ask if

Never stop and wait. A finding that the routing contradicts claude-91's decision becomes a repair here.
