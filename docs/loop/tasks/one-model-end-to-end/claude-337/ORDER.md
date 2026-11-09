# ORDER: one-model-end-to-end, claude-337

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `9dab054fc`.

## Goal

**Let the found-copy pull train the ask heads on language windows, so a language run can carry the question.**

- **What passed, and what it needed.** [claude-335](../claude-335/REPORT.md) passed B2 at both seeds with the ask heads, the found-copy pull and the question carry. [claude-336](../claude-336/REPORT.md) serves the ask table and the carry in the integer session, with exact parity.
- **Why the language run cannot start yet.** [claude-330](../claude-330/REPORT.md) found that without the pull, the ask heads do not learn the question: B1-a0 answers at most 3 of 1,024. But the language trainer refuses the pull. `run_long` calls `refuse_pull_outside_recall` (`in_space_long.rs:14082`), and `step_gradient` (`:1959`) has no place for it. So a language run with `--hamming-ask 1 --hamming-carry-question 4` today trains the ask table and the carry from the task loss alone, the configuration that failed B1.
- **The panel's reply is a found copy.** A served-reply-panel prompt states two facts, and the reply is the asked value, whose tokens stand earlier in the same window. So, on language windows, the pull at the scored rows is defined exactly as in the recall runner: for each scored row `(row, target)`, the earlier positions of the window that hold `target` are the copies an ask head should find (`HammingReads::ask_pull`, `flock_read_hamming.rs:1096`). Rows with no earlier copy add nothing.

**The change:**
1. **The pull in `run_long`, at the scored rows only.** It is on when `--hamming-attract λ > 0`. `step_gradient` adds `λ · ask_pull` to each forward's loss, over that forward's scored rows and targets. It uses the same share weighting as the forward's task loss, so a micro-batched step's gradient equals a one-forward step's. `--hamming-attract-at question|both` stays refused outside `--recall`: language windows have no question row.
2. **The reading.** Each progress row records the pull's mean distance and the number of scored rows with a copy, as the recall runner logs its pull. The logged task loss is unchanged.
3. **The float scorer on a carried read.** Check that `--score-rows` generation runs on a carried read, uncached, and gives the rows a full-window forward gives. If it refuses, make it fall back to the uncached path for a carried read, in the same files.
4. **Tests, each with a mutation that fails it** (host, CPU):
   - `λ = 0`, or no ask heads, leaves the step's loss and gradient bit-identical to today's;
   - the pull rows that `step_gradient` builds equal a brute-force list over the windows' scored rows, with window starts at `b · context`;
   - the pull's gradient reaches `a` and `c` and no trunk weight;
   - a micro-batched step's gradient equals a one-forward step's with the pull on, within float tolerance;
   - `--hamming-attract-at question` and `both` are refused by `run_long`, by name;
   - `config.json` records `hamming_attract` for a language run and leaves it out at 0.

**Claim check:**
- **(a) Step:** the run-claude order's step 4 (one model trained, served and scored end to end), for the recall gap: no reply carries the asked value (0 complete at every seed so far).
- **(b) What it changes in the model that trains:** the language trainer gets the one training signal under which the ask heads learn a question (claude-330). That signal is required before the next unit, the two-seed language run of ask heads plus carry against a no-carry control (L-010), can test recall. That run is its own unit, under its own prep sheet, FIT and pre-run review.

**Not in scope, named here:** the language run and its sheet (claude-338); the float cached decode step for the carry; DeepSeek's CPU Hamming port (`crates/uor-r4-cpu-train`, its lane; #2028 note 6738682682).

## Cancel probe (run first)

1. No open PR touches `in_space_long.rs`, `flock_read_hamming.rs` or `train-in-space.rs`.
2. claude-336 is merged and verified on `main`.
3. **Compute:** the laptop CPU only. Builds and finite host tests need no FIT (CHARTER §4, item 5). The CUDA library tests run on the next unit's pod, before its training.
4. **Disk:** at least 5 GiB free; one target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass.
- Open PRs: #3889 (DeepSeek, `crates/uor-r4-cpu-train` and its docs) and #3890 (Kimi, workbench). Neither touches these files.
- claude-336 merged as `9dab054fc2727703336294ac5cf7c24787648079` and was verified on fresh `main` (integer lib 499/0/1, float-train lib 336/0).
- 15 GiB free.
- `uor-harness loop next` offers only DeepSeek's CPU-repair owner direction, which is its lane. So the ORDER is written by hand under Casey's harness exception ("i approve you to make harness exceptions as needed to do what I ask"), as claude-331 to claude-336 were.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` (the flag's doc comment and its refusal text)
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`, only if the scorer's carried decode needs it (step 3)
- `docs/loop/tasks/one-model-end-to-end/**`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. `run_long` trains the pull at the scored rows under `--hamming-attract λ`, and records it. Every test above passes, and its mutation fails.
2. The float scorer's handling of a carried read is recorded: it runs uncached, or it is fixed to, with a test.
3. Every existing `uor-r4-float-train` library test passes at the head (host).
4. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit for the code. If the pull's gradient cannot be added without changing the trunk's own step, the REPORT names the conflict, and the next unit decides it.
