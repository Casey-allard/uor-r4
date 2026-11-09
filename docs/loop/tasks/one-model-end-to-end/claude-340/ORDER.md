# ORDER: one-model-end-to-end, claude-340

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646cdae3e2f8874939680324ed8cd5affd5`.

## Goal

**Parse the served artifact only before the timers in every path of `served-reply-panel`, so no timer in the report holds an artifact parse and the rank modes stop paying one parse per row.**

**What the code holds at this base, which narrows the gap claude-311 named:**
- claude-311 named the per-row parse at `served-reply-panel.rs:204`, the greedy loop.
- That loop no longer parses. [claude-316](../claude-316/REPORT.md) added the decode-only timer (`decode_milliseconds`, `decode_tokens_per_second`), and [claude-338](../claude-338/REPORT.md) built one forward and one model before the loop.
- So under `--engine integer`, the mode the language run uses, the artifact is already parsed once and decode tokens per second is already decode only. This unit expects no change there, and the parity check confirms it.
- **The per-row parses that remain** are the two `CpuFbsForward::from_artifact` calls in `teacher_forced_row` and `paired_first_row`. They run under `--teacher-forced 1` and `--paired-first 1` on the CPU port, up to two parses per row, inside `teacher_forced.wall_milliseconds` and `paired_first.wall_milliseconds`.

**The change, in `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:**
1. Both rank-row functions take the once-built `&CpuFbsForward` and clone it, as the greedy loop already does.
2. A test pins it: in non-test code each artifact parser has exactly one call site, and it sits before the first timer.

**Claim check:**
- **(a) Step:** P3, faster scoring.
- **(b) What it changes:** no model and no scored field. It removes up to 512 artifact parses from a 256-row rank run and makes both rank wall times decode figures for any artifact size.

**Not in scope, named here:**
- claude-339's run, its pinned scorer `served-reply-panel-c6204f66` and the branch `claude/language-carry-run`.
- DeepSeek's draft #3895, which changes the decode step this scorer calls. It edits this file in other hunks and is not touched here.
- The reports that still say "the timer re-parses": they are historical records.

## Cancel probe (run first)

1. No open PR makes this change. **Result: pass.** #3895 (DeepSeek, draft) is the only open PR on this file; its hunks change the decode step and do not touch the rank-row functions or the timers. #3894 does not touch this file.
2. `loop next` offers only DeepSeek's CPU-repair owner direction, its lane. **So this ORDER is written by hand under Casey's harness exception** ("i approve you to make harness exceptions as needed to do what I ask"), as claude-331 to claude-338 were.
3. Disk: at least 20 GiB free. **Result: pass,** 35 GiB free.
4. The instrument crate builds with `--offline --locked`. **Result: fail, then repaired.** 34 of the crate's 115 locked `.crate` tarballs were missing from the shared `~/.cargo/registry/cache`, with their unpacked sources still present, so `--offline` refused. The cause is not known. The session that started this unit relaxed its `--offline` rule for one command: `~/.cargo/bin/cargo fetch --locked --manifest-path docs/labs/one-model-end-to-end/instrument/Cargo.toml`. It restored those 34 tarballs (509 to 543 in the cache) and left `Cargo.lock` unchanged. Every later build and test here is `--offline --locked`.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-340/**`
- `docs/research/notebook/float-training.md`

## Done when

1. **I-c parity holds, as the unchanged-output check.** Under `--engine integer`, claude-322's two sealed artifacts score, with the timing fields removed, to the stripped report hashes in the language-carry sheet: s1 `5a27775c…`, s2 `7eb8dbf5…`. That path already parsed once at this base.
2. **The rank modes' scored output is unchanged.** `--teacher-forced 1 --paired-first 1` on the CPU port gives the same report before and after, with the timing fields removed.
3. **The rank modes hold no per-row parse.** The test above passes, and fails with a parse put back in a row.
4. **The rank modes' wall time before and after is in the REPORT,** with the integer path's tokens per second as the unchanged reference.
5. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). If parity differs, the REPORT names the first differing field and classifies it, instrument versus mechanism, before anything merges.
