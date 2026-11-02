# REPORT: integer-nearest-readout, claude-255 (the serving audit's `fmov`)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Q5 chain step 5, under claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

The served session is unchanged byte for byte (below), and on the laptop it decodes faster at the head.

- **Setup:** R's artifact `a7r-s1.uorlut`, teacher-forced with `fbs-toks --forced` on claude-249's DEV s1 smoke prompts. That is 308 rows, and each row steps over its prompt and its reference less one token: 17,491 steps.
- **Machine and runs:** the laptop (Apple M1, AC power). No trace was written. The base's and the head's release binaries were alternated three times each, head first.

| Binary | ms per row (runs 1, 2, 3) | tok/s (17,491 steps / wall) |
|---|---|---|
| head (`fbs-toks` `f6d0b777…`) | 42.749, 43.210, 42.807 | 1,328, 1,314, 1,327 |
| base `05021840e` (`fbs-toks` `c8db382a…`) | 56.115, 54.584, 56.270 | 1,012, 1,040, 1,009 |

- **This is a reading, with no bar, and no cause is claimed.** The release codegen changed: at the base, `TypeHeads::mean_by` is inlined into its callers; at the head, it is its own function.
- **Not comparable to claude-243's numbers:** its free-decode median (1,204 tok/s on R s1) is a different protocol.
- **Source:** `per_row_us` from `local/claude/claude-255/speed.log`, run by `speed.sh`. Every run's rows file equals the base's, byte for byte.

## What did not work, first

- **The Python serving audit did not see the fault.** `scripts/audit_zero_matmul_serving.py --stack` passes the base's release `uor-r4-stack` too ("FULL PASS", 0 violations over 127 reachable functions).
  - The `fmov` was only in the test-profile build that `m5_full_path_cost_controls` disassembles: opt-level 3, through the workspace's `[profile.test.package.uor-r4-integer]`.
  - So done-when 2's Python audit passes at both commits and pins nothing about this fix. The negative gate is `m5`'s.
- **The first cancel probe ran 2 tests, not 4.** `m5`'s `serving_controls` module is `#[cfg(feature = "serving-wasm")]`, and the first run had no features. Re-run with `--features serving-wasm`, it failed as main does. `probe-base.log` is the second run.
- **The ORDER's first done-when 3 check could not see this change, and was replaced during the work.**
  - It was claude-254's `--decision-check` and self-grade on F's trace. Both read stored traces and tables and never run the served session.
  - The replacement is a base-against-head decode of R's artifact, plain and forced. The ORDER records the amendment, and that it was written after the decodes ran (their script was written before).
- **The forced-against-unforced comparison in `decodes.sh` is weak.** The forced run decodes the 3 windows `force-own-3.json` names, and the plain run decodes 4. So "DIFFER" there shows only that the outputs are not one file. What carries the forced branch is the base-equals-head comparison of the forced run itself.
- **The review's findings, all fixed.** Both reviews were APPROVE WITH NITS.
  - **The claims check:** the first notebook row and plain summary said the forced decode covered DEV's 308 rows, but it covered `prompts-4.json`'s 3 forced windows. The head binary's `mean_by` claim had no saved disassembly, and the base binary's pin cited no file. All three are fixed above.
  - **The code review:** the release binaries were not yet tied to the head, because the set-aside and restore had touched `hamming.rs` after the fix commit. The release rebuild at the final head ties them. Its doc-comment nit is applied, with the comment's line count kept, so no code moved.
- **The 36 failing tests of `uor-r4-integer --features serving-wasm` are still red.** They are the base's own: claude-254's run at `4edc6e841`, whose crate tree is identical to `05021840e`, failed the same 36 plus the two this unit repairs. They are the M1/M2/M3 challenger and timing tests and the frozen-panel CLI test. Of their 36 panics, 32 are a quaternion or Householder bundle not found, 3 a missing `tokenizer.json`, and 1 is `verify_m7_parity.mjs` at its WASM target check. None is touched here.

## What changed

`TypeHeads::mean_by` (`crates/uor-r4-integer/src/stack/hamming.rs`) takes no `choose` closure.
- **The fault:** in the test-profile build, LLVM copied the closure's captured environment with `ldr q0` / `str q0` and took its low word with `fmov x8, d0`. That is a SIMD-to-general transfer, which the serving audit refuses. It happened in both instances: `TypeHeads::mean`'s closure and `ServedHamming::type_mean`'s forced one.
- **The fix:** `mean_by` now takes the forced heads as `Option<&[HeadForce]>` and calls `choose` or `choose_forced` itself, with the same arguments the closures passed. `mean` passes `None`, and `type_mean`'s forced branch passes `Some(heads)`.
- **Size:** code 11 lines added and 5 removed, in one file; tests none; docs this REPORT, the ORDER, one item bullet and one notebook row. The ORDER gave no size estimate.

## Measurements

- **The audit at the base `05021840e`:** `m5_full_path_cost_controls --features serving-wasm` gives 2 passed and 2 failed (`probe-base.log`). Both failures name `fmov x8, d0` in `TypeHeads::mean_by::<ServedHamming::type_mean::{closure#0}>` and in `TypeHeads::mean_by::<TypeHeads::mean::{closure#0}>`.
- **The disassembly,** with the instructions before the `fmov`, is in `local/claude/claude-254/typeheads-fmov-excerpt.txt`, from claude-254's test build.
- **At the head:** 4 passed (`m5-head.log`, and again at the final head in `m5-final.log`).
  - The head's test-profile `uor-r4-stack` (sha256 in `head-test-uor-r4-stack.sha256`) has one, non-generic `TypeHeads::mean_by` symbol, saved as `head-mean_by-disassembly.txt` (`objdump --demangle -d`). Its 442 lines hold no `fmov` and no `q`, `d` or `v` register.
  - The review found no `fmov` in any other `stack::hamming` symbol. That includes all three `CopyHeads::mean_by` instances.
- **Negative gate:** `fix.patch` is `git diff 05021840e -- hamming.rs`. With it set aside, `uor-r4-integer` recompiled and `m5` failed 2 of 4, naming `TypeHeads::mean_by`'s `fmov` twice (`m5-negative.log`). Restored and recompiled, it passed 4 of 4 (`m5-restored.log`), and the tree had no diff.
- **Served bytes:** `decodes.sh` ran the base's and the head's release `fbs-toks` on `a7r-s1.uorlut`, recording rows and decision traces. The base binary is the set-aside build, and its sha256 `c8db382a…` equals claude-254's pinned grader at `768c39ff3` (`local/claude/claude-254/reading-r1/README.txt`).
  - The head binaries are tied to the head. A fresh release build at the final head, which recompiled `uor-r4-integer` after the doc-comment fix, gave the same sha256s as `bin/head-fbs-toks` and `bin/head-uor-r4-stack` (`build-final.log`).
  - So the decodes, the speed runs and the Python audit all ran the head's binaries, and the comment fix changed no byte.
  - DEV s1 plain: 308 rows, 4,382 positions.
  - Forced: `prompts-4.json` under claude-249's `force-own-3.json`, 3 rows and 56 positions.
  - Plain: `prompts-4.json`, 4 rows and 66 positions.
  - **All six files are equal, base against head.** R's DEV trace is `11e6b7d9347fed0cbb390ba2e7b04b1bfe7031e946db87c05fd842d2421cca7d` both times. The head's three traces also equal claude-249's stored `trace-dev-s1.json`, `trace-4.json` and `trace-force-own-3.json`.
- **Why these decodes reach the change:** R's artifact has type heads. Both reads in F's snapshot have 256 nonzero type weights of 512, and `ServedHamming::new` builds type heads when any type weight is nonzero. So every step runs `TypeHeads::mean`, and with `--force` every forced read runs `mean_by(.., Some(heads))`.

## Checks run at the head

- `m5_full_path_cost_controls --features serving-wasm`: 4 passed.
- `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast`: 631 passed, 36 failed, 4 ignored (`suite-head.log`). The 36 are a subset of the base's 38; the other two are `m5`'s, now passing.
  - Passing among them: `type_heads_match_the_f64_reference` (the type mean against an f64 reference, to 2⁻¹⁶) and `a_forced_bucket_or_code_takes_the_forced_value_and_changes_what_follows` (forced queries and keys in the copy and type heads, at both depths).
- `python3 scripts/audit_zero_matmul_serving.py <head release uor-r4-stack> --stack`: FULL PASS, 0 forbidden instructions over 44 symbol ranges and 127 reachable functions (`py-audit-head.log`). The base passes too (`py-audit-base.log`).
- The base-against-head decodes: all six files equal (`decodes.log`).
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: 106 warnings in the crate, 0 in `hamming.rs` (`clippy.log`).
- `cargo fmt -p uor-r4-integer -- --check`: exit 0.
- `uor-harness loop gates --full` and `loop pr-check --local`: listed in the PR's review record.

The logs are in `local/claude/claude-255/`, and the binaries with their SHA256SUMS in `local/claude/claude-255/bin/`.

## Skills used

None.

## Still open

- **The replay driver** (Q5 chain step 6): the first differing code or bucket forced in forward order, with a single-window float mode. It also carries claude-254's six round-2 points (its REPORT's "Still open"). Then the sheet with its FIT and pre-run review, and the GPU run. Tracked in the item's done-when 3 (#3462).
- **The WASM build** in done-when 2 was not run here. This change touches no WASM-only code.
- **`CopyHeads::mean_by` still takes a closure.** It passes the audit today; a codegen change could move the same copy into it. It is not changed here because nothing refuses it.

## Plain summary

The project serves its language model with integer arithmetic only, and an audit reads the compiled machine code to make sure no floating-point or vector instruction is used. On main, the audit failed. One function that averages the "type heads'" picks (a part of the model's memory read) got a vector-register move, `fmov`. It came from how the compiler copied a small bundle of captured pointers that the function was handed as a closure, not from any arithmetic.

The function now receives the forced choices directly instead of a closure, so nothing is left to copy, and the audit passes. To show nothing else changed, the old and new programs decoded the same 308 test windows, and 7 short ones: 4 plain and 3 with forced choices. Every output file is identical, and identical to files stored by an earlier unit. The new build also decoded about 30% faster in three alternating runs. That is reported as a reading only; no bar was set for speed.
