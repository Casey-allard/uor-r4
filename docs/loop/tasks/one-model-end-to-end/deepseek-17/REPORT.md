# REPORT: one-model-end-to-end, deepseek-17 (registered cache inventory)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `a9a0f75ac`.

## What did not work, first

- **The prepared GPU capture could not reach the merged evaluator.** `score_rows` cached
  the built-in DEV split before the requested panels and then added TRAIN. That produced
  six no-memory splits, while the registered evaluator requires exactly TRAIN plus the
  four registered panels. The failure would have occurred only after an expensive GPU
  capture.
- **A first full-library invocation failed one unrelated save/load test.** The test
  requires `CARGO_TARGET_DIR` for its owned temporary directory. Rerunning the unchanged
  suite with `CARGO_TARGET_DIR=target` passed all 294 tests.

## What changed

`train-in-space --query-cache-dev 0` omits only the DEV cache split. DEV is still scored
and written normally, all forward decisions are unchanged, and the option defaults to
the prior inclusive behavior. The option is refused unless `--query-cache-out` is set.

A focused test pins the split-selection rule, including that TRAIN and panels remain in
the cache when DEV is omitted. The deepseek-16 ORDER and REPORT now record this repair
and the installed credential state.

## Measurements

No model-level measurement changed. The repaired no-memory cache inventory is TRAIN plus
`oasst2`, `wiki-00000`, `wiki-00020` and `wiki-00040`.

## Checks run at the head

- `cargo check -p uor-r4-float-train --bin train-in-space --locked --offline`:
  passed.
- `CARGO_TARGET_DIR=target cargo test -p uor-r4-float-train --lib
  --no-fail-fast --locked --offline`: **294 passed, 0 failed**.
- `cargo clippy -p uor-r4-float-train --all-targets --no-deps --locked
  --offline`: passed with the crate's pre-existing warnings.
- D23, D25 and D32 ratchets: passed.
- `uor-harness loop report --check`, `git diff --check`: passed.

Not run: CUDA tests, GPU capture, cache evaluation or model scoring.

## Skills used

No system-provided Codex skill applied. The repository's loop, D22 and delivery rules
were used directly.

## Still open

1. Merge this inventory repair and use its verified main commit as the capture pin.
2. Correct the capture job's staging, headroom, whole-run timeout, immutable output path,
   complete readback and CPU restore commands.
3. Obtain a fresh `TEST FITNESS: FIT` and D67 pre-run review for the corrected package.
4. Capture both n3S seeds without touching either existing busy pod.
5. Evaluate both caches and aggregate their registered decisions.

## Plain summary

The capture was going to save one extra DEV section that the strict evaluator refuses,
so a multi-hour GPU run would have been discarded at the CPU step. The scorer can now
leave that one section out while still scoring DEV exactly as before. The source repair
passes its full host suite; the execution package still needs its separate safety and
readback corrections before it is fit to run.
