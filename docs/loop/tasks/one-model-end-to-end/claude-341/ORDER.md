# ORDER: one-model-end-to-end, claude-341

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `d4154488fcd41c3541ccf0eaa2b92caaeddd84c8`.

## Goal

**Make `served-reply-panel`'s test suite green on `main` by deciding what the red test should say.**

`only_the_integer_session_reads_the_ask_heads` is red on `main`, 27 of 28 passing. It went red at `2c5d2a407` (#3898, deepseek-261), which taught `CpuFbsForward` to read the ask heads and the carry. The failing line asserts that the CPU port decodes an asking artifact exactly as a plain one. The crate has its own `[workspace]`, so no required check runs this test and nothing stopped the merge.

**The question, in order:** is the test's premise out of date, or did #3898 change the scorer's behaviour in a way that is wrong?

- The specified behaviour changed if #3898 meant the CPU port to read asks, the two engines now agree on the asking fixture, and `--engine cpu` still does not score an asking artifact silently. Then the test is rewritten to say what is now true, and the REPORT says why.
- If `--engine cpu` now silently reads ask heads where it should refuse or differ, `served-reply-panel.rs` is fixed.
- If the fault is in `crates/uor-r4-cpu-train`, it goes to DeepSeek on #2028 and this unit stops. That crate is DeepSeek's and is not edited here.

**Claim check:**
- **(a) Step:** P3, an honest scorer.
- **(b) What it changes:** no model and no scored field. It repairs one test and the text that repeats the same out-of-date premise.

**Not in scope, named here:**
- `crates/uor-r4-cpu-train`, and any change to whether `--engine cpu` accepts an asking artifact. Lifting that refusal is a separate decision.
- claude-339's run, its pinned scorer `served-reply-panel-c6204f66`, `local/claude/claude-339/` and the branch `claude/language-carry-run`.
- claude-338's REPORT, which names the old test. It stays as a record of its time.
- Adding this crate to a required check.

## Cancel probe (run first)

1. No open PR changes this file. **Result: pass.** #3894 and #3889 are the open PRs and neither touches it.
2. `loop next` offers DeepSeek's CPU-repair owner direction, its lane. **So this ORDER is written by hand,** as claude-331 to claude-340 were, under the harness exception recorded in claude-340's ORDER.
3. Disk: at least 20 GiB free. **Result: pass,** 45 GiB free.
4. The test is red at this base. **Result: pass.** With `--offline --locked --features cpu-parity` at `d4154488f`, 27 pass and the one failure is `served-reply-panel.rs:1612`, the line named above.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-341/**`
- `docs/research/notebook/float-training.md`

## Done when

1. All 28 tests of `served-reply-panel` pass on fresh `main` with `--features cpu-parity`, `--offline --locked`.
2. The REPORT classes the failure, instrument versus mechanism, and says why the test changed.
3. The rewritten test fails against the CPU crate from before #3898, so it is not a test that passes whatever the port does. That run uses a throwaway checkout of the older tree; no file in `crates/uor-r4-cpu-train` is edited.
4. The corrected refusal text is pinned by a test that fails with the old text.
5. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). If the two engines disagree on the asking fixture, the REPORT names the first differing row and classifies it before anything merges; a fault in the CPU crate goes to DeepSeek on #2028.
