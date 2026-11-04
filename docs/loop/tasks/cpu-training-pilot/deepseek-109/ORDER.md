# ORDER: cpu-training-pilot, deepseek-109 (expanded memory-weight panel)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `9bd3c5773`.

## Goal

Run the next materially larger CPU training comparison on the exact
training-through-serving memory-weight surrogate. Expand deepseek-95's
repeated-key panel from eight keys / 60 training / 56 evaluation examples to
the largest bounded 16-key selection available: 102 training and 114
evaluation examples. Train 32 exact Q24 movement-coordinate updates and
compare the trained artifact with frozen, equal-move random and memory-off
controls on both public model seeds.

This is a bounded repeated-key internal-development comparison. It is not
unseen-key, complete-reply, fresh-phrasing or GPU qualification evidence.

## Files in scope

- `crates/uor-r4-cpu-train/**` only if a defect blocks the registered run
- `docs/harness/prep/cpu-training-pilot-memory-expanded-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-109/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Registered configuration

- two public dual-c131 artifacts, seeds 1 and 2;
- fixed DEV token/mask/manifest inputs used by deepseek-95;
- `--keys 16 --per-key 16 --key-tokens 8 --reply-turn 8`;
- 102 training and 114 evaluation examples selected by the existing strict
  document/group splitter;
- `--steps 32 --seed 1`;
- cached exact trajectory;
- 32 Rayon workers for seed 1 first, then the same for seed 2 if the
  registered 30-minute total-cost stop allows.

## Done when

1. The FIT sheet and selection counts are frozen before either seed runs.
2. Both runs complete without overriding inputs, selection, steps, seed or
   controls.
3. The trained, frozen, equal-move random and memory-off readings are reported
   with CPU/serving parity and cost.
4. The REPORT records the paired three-sigma result first, including a negative
   or inconclusive outcome.
5. Tests reached by the change, strict CPU Clippy, formatting, evidence,
   report, exact-head review, protected merge and fresh-main verification pass.

## Cancel probe

- `main` is clean at `9bd3c5773`.
- Claude's #3629 changes `train-in-space` and its docs, not the CPU memory
  driver.
- `cpu-memory-panel-driver --keys 16 --per-key 16 --select-only` returns 16
  keys, 102 training and 114 evaluation examples.
- No new code is required unless the registered run exposes a defect.

## Stop and report

Stop on selection drift, missing exact parity, non-deterministic checkpoints,
or any post-result change to keys, examples, steps, seed or controls. Do not
describe repeated-key transfer as unseen-key generalization.
