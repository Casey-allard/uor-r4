# REPORT: one-model-end-to-end, claude-337

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Code head `365bb69346eb734da6548cbaba08e14d8a092e66`, on base `9dab054fc2727703336294ac5cf7c24787648079`.

**In one line:** the language trainer (`run_long`) now trains the found-copy pull at its scored rows under `--hamming-attract λ`. This is the training signal without which the ask heads did not learn the question (claude-330). No model was trained in this unit: the language run with it is claude-338.

## What did not work, first

- **The step-gradient test could not run on the laptop.** The model's forward is CUDA-only ("qualified model execution requires F32 CUDA SpiralCore/FBS; no host or dense fallback"). The ORDER put every test on the host CPU, but the step-gradient test (λ = 0 identity, gradient reach, micro-batch equality) needs that forward. So it is gated on `feature = "cuda"` and ran on one Vast RTX 5090 (`vast-59882489`, $0.563/h, 475 s up to down, about $0.07). This is a finite kernel test, not training or evaluation, so it needs no FIT (CHARTER §4 item 5). The pure-host parts (the pull rows, the tally, the refusals, `config.json`, the cache refusal) run on the laptop.
- **The CUDA gates ran at the first code commit, `e3b1c0369134a77a445700ba328a6435b56cde70`, not at the head.** The review fixes in `6f5265900` and `365bb6934` came after. They add host checks and tests: the early refusals in `run_long` (the placements, then the weight check, before `config.json` is written), `pull_applies` in `train-in-space`, a `config.json` test and a cache-refusal assertion. They do not change `step_gradient`, `PullTally`, `language_pull_rows` or the CUDA test. At the head, the host suites pass and the CUDA build type-checks with a stub `nvcc` (below). The CUDA suite was not run a second time.
- **Deviations from the ORDER, named:**
  - **The weight is the forward's share of the step's copy rows, not of its scored rows.** The pull is a mean over copy rows (`ask_pull`'s `terms = spans × asks` per row). So each forward's pull is weighted `rows_here / rows_in_step`, and the step's pull is the mean over all its copy rows, whatever the micro-batching. The task loss keeps its own scored-row share. The CUDA test checks that micro = 2 gives the micro = 0 gradient within 1e-4 of scale, and the same pull reading within 1e-5.
  - **The reading goes to `pull.jsonl`, one line per checkpoint, not into the progress row.** Each line holds the step, λ, the mean pull distance over the steps that pulled, the mean copy rows per step and the step count. Then the tally starts again. The progress row and the logged task loss are unchanged: at λ = 1 the logged loss is bit-equal to the plain step's.
  - **"No trunk weight" holds by construction, not by a test.** `pull_bits` detaches the embedding table and the key rotation, so the only leaves under the pull are the ask table `a` and the carry gain `c`. The CUDA test checks that both get a nonzero gradient that differs from the plain step's. It does not check that the trunk gradients are unchanged by the pull.
- **A slip that cost an hour, not a result (L-007 class: instrument).** While the review fixes were uncommitted, a host mutation loop restored each file with `git checkout`, which reverted those fixes to the commit. They were recovered byte-for-byte from the session's own edit record, re-applied and committed, and every host gate was then run again on the clean tree. Logged in the lab's FRICTION notes.
- **Instrument faults at the base, unchanged:** `cargo check --all-targets` on `uor-r4-float-train` fails at `packet-parity` without `cuda`, so the targets were checked one by one.

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - `step_gradient` takes `Option<&mut PullTally>`. When λ > 0 and an ask read is set, each forward adds `λ · w · ask_pull` to its loss, over that forward's scored rows that have an earlier copy, where `w` is the forward's share of the step's copy rows.
  - `language_pull_rows`: the scored rows `(b · context + offset, target)` whose target stands at an earlier position `1 ≤ s < offset` of the same window. A target at position 0 or only at the row itself is not a copy.
  - `PullTally` and `pull.jsonl`, as above.
  - `run_long` refuses `--hamming-attract-at question|both` as its first line, before it reads or writes anything: a language window has no question row.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** the flag's doc comment covers the language pull. The placement refusal text names `--recall`. `pull_applies` refuses a nonzero `--hamming-attract` in every mode that trains nothing (grades, scoring, decoding, inspection and the rest), so a pull is never recorded and silently not run.
- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:** a test only. `the_cache_refuses_what_it_cannot_carry` now asserts that a carried read refuses the float decode cache.

## Measurements

No training and no evaluation. The only GPU time was the CUDA gates: 475 s on one RTX 5090, about $0.07.

## What this does and does not show

- **It shows** that a language run can now train the ask table and the carry under the found-copy pull at its scored rows. The pull is off and bit-identical to today's step at λ = 0. It does not change the logged task loss, it reaches `a` and `c`, and it gives the same gradient whether a step is split into micro-batches or not.
- **It shows** that the float scorer's `--score-rows` decode of a carried read runs uncached. The cache refuses a carried question (claude-335's refusal, now pinned by a test that fails with the refusal set aside), and the decode falls back to the full-window forward. The decode itself is not changed by this unit.
- **It does not show** that the pull makes the ask heads learn a question on language. That is claude-338's two-seed run against a no-carry control (L-010).
- **It does not show** how many panel replies have an earlier copy of their target in the window. A row with no copy adds no pull. Language rows that repeat a token are copies too, whether or not they are the asked fact.

## Checks run at the head

- **Host, at `365bb6934`:** `uor-r4-float-train --lib`, 340 passed, 0 failed. `--bin train-in-space`, 11 passed, 0 failed. Stub-`nvcc` `cargo check --features cuda --lib --bin train-in-space --profile test` passes.
- **CUDA, at `e3b1c0369`, on the pod:**
  - G1: `the_language_pull_joins_the_step_gradient_on_cuda` passes.
  - G2: each of the four CUDA mutations makes it fail, then the file is restored: the pull weighted 1 per forward, the pull never added to the loss, the pull run at λ = 0, and the tally's row count off by one.
  - G3: the whole library suite with `cuda`, `--test-threads=1 --no-fail-fast`: 462 passed, 0 failed, 18 ignored.
- **Host mutations, each set aside and restored, with its test failing** (on the clean tree at `6f5265900`, and the last at `365bb6934`):
  - position 0 counted as a copy;
  - the row's own input counted as a copy;
  - the window start dropped;
  - the mean rows divided by the pulled steps;
  - the tally not reset after its line;
  - the `both` placement let through;
  - `run_long`'s early refusal removed;
  - `config.json` writing the pull at 0;
  - `pull_applies` letting a pull into a mode that trains nothing;
  - the cache's carried-question refusal switched off;
  - `run_long`'s weight check moved back after `config.json` is written.
- **The in-unit fresh review**, round 1: one reviewer, given the spec and the diff only, returned CHANGES. Its findings were:
  - the term count and the detached table: checked in the code, held;
  - `config.json`: tested;
  - other modes: `pull_applies`;
  - the refusal placed after the data root was read: moved first;
  - the scorer's carried decode: pinned;
  - `pull.jsonl` against the progress row: named above.

  Round 2, on the delta, returned APPROVE-WITH-MINORS. Fixed: `run_long` now checks the weight (at least 0, the ask heads on) before it writes `config.json`, so a refused `-1` leaves no record; the `run_long` refusal test covers both placements and the weight. Left open: the `other_mode` test checks `pull_applies` alone, not the mode list, which the reviewer checked branch by branch (22 modes, none missed).
- No banned word in the diff.
- `loop report --check` and `loop pr-check --local` run at the head before the queue.

## Skills used

- None beyond the loop itself: the ORDER, the pod's finite CUDA gates and the fresh exact-head review.

## Still open

- **claude-338, the language run.** Ask heads, carry 4 and the pull, on claude-322's d3mwcap6 configuration, two seeds, against a no-carry control and the `−C` cut. It is served through `fbs-snapshot`, `fbs-export --run-config` and `served-reply-panel` built at the pin. It needs its own prep sheet, FIT and pre-run review. Expect about 2.2× the step time (claude-335's carried arms).
- **The float cached decode step for the carry:** still out of scope. Float decodes a carried read uncached.
- **The trunk-exclusion check:** by construction today. A test that compares the trunk gradients with and without the pull would make it explicit.
- **The mode list in `train-in-space`:** no test fails if a mode is dropped from `other_mode`. Moving the dispatch into a function that names each mode would let a test cover it.

Closes nothing: #3407 stays open.

## Plain summary

The model learns to answer a question from a fact earlier in the text only if it is pulled toward finding that fact while it trains. That was shown on a toy recall task. Before this unit, the trainer for ordinary text could not apply that pull. Now it can, at the positions it scores, wherever the right next word already appears earlier in the window.

With the pull switched off, training is exactly what it was. With it on, the reported loss is unchanged, the pull moves only the two new parts it is meant to train, and splitting a step into smaller pieces gives the same result. One rented GPU checked this in eight minutes for about seven cents. The next unit trains a model with it on real text and checks whether replies start to carry the asked value.
