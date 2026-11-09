# ORDER: one-model-end-to-end, claude-330

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `b2caa330e`; claude-329 (PR 3873) is merged in when it lands.

## Goal

**The question-keyed read, from its sheet to its run.** [claude-329](../claude-329/REPORT.md) wrote [the sheet](../../../../harness/prep/one-model-end-to-end-question-keyed-read-2026-11-08.md) and stopped under its two-round rule. Round 2's only blocker, the wording of the I-R reproduction check, and its two minors are applied but not reviewed.

1. **A fresh non-author FIT of the sheet as merged.** Its findings are applied. If it reads NOT FIT, one more round is allowed. If that also reads NOT FIT, the unit stops at the sheet and names what is left.
2. **If FIT: the code**, in `crates/uor-r4-float-train`, behind `--hamming-ask 1`, `--hamming-attract λ` and `--hamming-attract-at question`, all default off.
   - **Ask heads:** a per-token offset table `a`, initialized to 0, on each `HammingRead`. When it is present, the type heads take:
     - the query `rotate(key_rotation, point(x_t) + a[x_t])`;
     - the copy key and the copy payload;
     - the copy-form combine, with the type weight row.
   - `ReadScope::Ask`, for the `R^{−aQ}` row cut.
   - Saving and loading `a`.
   - **The found-copy pull:** computed from `ids` and the detached table alone, with the key and `key_rotation` detached. It is added at the recall loop's `batch_loss` call (`in_space_long.rs` ≈12741).
     - By default its rows are the scored rows with their targets.
     - Under `--hamming-attract-at question`, its rows are each scored offset minus 4, with the same target.
   - **Readings:** the pull distance per checkpoint and depth in `recall.jsonl` (a new optional field, left out when off), and the `−aQ` ablation, only when the ask flag is on.
   - **Instrument checks I-1 and I-A1 to I-A5, plus the I-4 gates, as unit tests**, each with a mutation that makes it fail.
3. **If the code merges inside the unit:** the two-seed run under a D67 pre-run review, read against the bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** the read gains a learned way to look up a key it is not given, which is the reply panel's missing step. A fail closes this configuration, and the sheet's outcome map names the next lever.

## Cancel probe (run first)

1. No open PR adds `hamming-ask` or an ask offset table.
2. claude-329's sheet and REPORT are on `main` at the base.
3. **Compute:** the laptop CPU for the FIT, the build and the tests. A GPU only after FIT and a pre-run review, at about $1.
4. **Disk:** about 8 GB free. Builds use one target dir in this worktree, removed on merge.

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end-*.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet has a FIT line from a fresh non-author reviewer.
2. If FIT:
   - the code is merged;
   - its instrument checks pass, and their mutations fail;
   - the default paths are byte-identical (I-1).
3. If the run happens, both seeds are:
   - sealed to the store and read back;
   - read against the bar;
   - recorded in a notebook row.
4. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: two units. The code and the run may land in separate PRs. Past the bound, the unfinished parts are named in the REPORT.
