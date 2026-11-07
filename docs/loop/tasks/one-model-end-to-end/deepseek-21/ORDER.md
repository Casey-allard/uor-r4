# ORDER: one-model-end-to-end, deepseek-21 (EOS-rank and rendering diagnostics)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `b80933c4b`.

## Goal

Extend `served-reply-panel` so the next training decision can distinguish:

1. the current `one-user-turn/v1` rendering from an explicit per-turn rendering;
2. EOS exposure failure from ranking-geometry failure, using teacher-forced
   reference-token and EOS ranks; and
3. the served integer scorer from the trained float scorer on the same replies where a
   compatible checkpoint/scorer interface exists.

The default invocation and its current JSON/metric behavior must remain unchanged.

## Cancel probe

Run first. Cancel a subpart rather than guessing when its required artifact or interface
is absent:

- confirm `served-reply-panel` and `CpuFbsForward` are present;
- inventory the panel packet turn layout and available saved checkpoints;
- inspect whether a float whole-reply scorer reads the same panel/reply schema;
- locate `m2-reply512`; if its content is unavailable, record NOT_RUN with the checked
  store paths rather than fabricating a panel.

## Required behavior

- Add an explicit rendering selector. Preserve `one-user-turn/v1` as the default and keep
  its existing rows/metrics stable. Add a documented per-turn mode that represents each
  packet turn without back-to-back role ambiguity and refuses malformed turn sequences.
- Add a teacher-forced mode that reports, per reference reply token and in aggregate:
  reference-token rank, EOS rank, target count, rank percentiles and the count where EOS is
  top-1/top-k. Ranks must use the same nearest/readout arithmetic as serving.
- Add same-scorer float/integer parity only if the float reply scorer consumes the same
  reply schema. Report exact agreement/disagreement counts and ranks. Otherwise document
  the missing interface and leave that measure NOT_RUN.
- Add negative tests for malformed turn layout, incorrect acceptance, off-by-one target
  alignment, wrong EOS extraction/rank ordering and parity mismatch.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-21/**`
- `docs/research/notebook/float-training.md`

## Done when

1. Default rendering output is pinned by a regression test and remains unchanged.
2. Per-turn rendering and teacher-forced EOS-rank reports are implemented and tested.
3. Float/integer parity is either measured on the same scorer semantics or precisely
   documented as unavailable with the blocker named.
4. `m2-reply512` is located or recorded NOT_RUN after a bounded store search.
5. Builds/tests, `git diff --check`, loop report and required local checks pass.
6. REVIEW receives a fresh-context adversarial exact-head review before delivery.

## Stop and ask if

Ask means decide, record and continue. Laptop only; no GPU, pod or spend. Do not touch
Claude's draft branch or redefine a milestone bar.
