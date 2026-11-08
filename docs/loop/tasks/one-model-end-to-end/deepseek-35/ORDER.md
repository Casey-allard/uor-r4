# ORDER: one-model-end-to-end, deepseek-35 (opening-weighted SFT)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `859621f33`.

## Goal

Add an opt-in opening-row weight to the geometric nearest objective and run one
1,024-step, seed-1 controlled SFT from `(b64L)` on the original OASST2-heavy
mixed store. The target is the measured first-answer-token failure: the selected
content-ratio checkpoint had 0/256 first targets in the top 32.

## Cancel probe

- The merged opening diagnostic establishes the failure and pins the source
  checkpoint.
- The original 1,024-step mix is immutable and already sealed.
- The new weight changes only how often opening targets enter the existing
  nearest loss; serving, architecture, selector and outcome code stay fixed.
- No other live DeepSeek pod or opening-weight PR exists.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/harness/prep/one-model-end-to-end-opening-weight-2026-11-08.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-35/**`
- `docs/research/notebook/float-training.md`

## Done when

1. `--opening-weight` is opt-in, range checked, serialized only when non-default,
   and duplicates only scored rows whose target opens a reply.
2. Unit tests prove default training is byte-for-byte unchanged, opening rows
   are weighted exactly, and empty/non-opening rows are unaffected.
3. A closed runner selects the original mix, 1,024 steps, the new weight, the
   opening diagnostics, CPU evidence, unchanged selector and unchanged seals.
4. Fresh FIT/D67 reviews approve the exact runtime before launch.
5. One run records whether opening top-1/top-8 and complete replies move under
   the weight; a non-positive result closes only this opening-weight
   configuration.
6. Evidence is read back, the owned pod is deleted, and one result PR is merged.

## Stop and ask

Do not change the architecture, serving graph, closed content-ratio recipe,
selector or outcome bars. Never touch Claude's pod or files. Any instrument or
input mismatch stops the run without a result.
