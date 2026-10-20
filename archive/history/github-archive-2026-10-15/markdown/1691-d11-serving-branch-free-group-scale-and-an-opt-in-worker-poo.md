# #1691 — D11 serving: branch-free group scale and an opt-in worker pool, 3.6x faster chat at 4 threads with identical tokens (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T16:51:58Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-04T17:15:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1691
- **Merged:** 2026-10-04T17:15:48Z
- **Merge commit:** 7e27dfb0b7dbdf49c75bbf41a16eb7b8139a74b9
- **Base / Head:** main / UOR-Foundation:codex/d11-serve-speed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This makes D11 multiplier-free chat serving faster on the M1 laptop. No served token changes. The 29M artifact goes from 26.7 to 97.0 generated ids/s and the 96M artifact from 8.7 to 32.5, both at 4 threads. The served integers are unchanged: on both real artifacts, all 32 everyday-32 rows match the existing `chat.json` token for token, at 1 and at 4 threads.

There are two causal changes:

1. **Branch-free group scale.** `scale_16_plus` applies the 4-bit group scale `(16 + m) a`. It looped over the bits of `m` with a data-dependent branch (`tbz`), about two mispredictions per 32-weight group. The weight maps were limited by those mispredictions, not by cache or bandwidth: a microbenchmark with constant weight bytes ran just as slowly. Each bit now selects `a` through a mask, `((((2a + m3·a)·2 + m2·a)·2 + m1·a)·2 + m0·a`. The dependent shift chain keeps LLVM from vectorizing the four terms. A first version that did vectorize emitted an `fmov`, which the audit refused. Single-thread map cost fell from about 1.1 to 0.5 ns per weight byte.
2. **Optional worker pool** (`IntegerStackModel::set_threads`, `threads()`). **The default is 1 thread**, the caller's. Laptop jobs register their thread counts in a shared 8-thread budget, so a loaded model never starts threads by itself; a caller opts in with `set_threads(n)`. `step` installs the rayon pool when there is one. The pool splits three kinds of work across threads:
   - the rows of the large weight maps, in tasks of 64 rows, with the two maps of one input run together;
   - the recurrence lanes;
   - the read heads.

   One thread computes each output row, lane or head whole, with the unchanged kernel, so the integers do not depend on the thread count. Each read head now has its own query tables, scores, weights and mixture. A row range starts at a digit-table product (`stack_mul_u64`), not a multiply. No step allocates: `rayon::join` uses stack jobs.

`lut-chat` and `d11-evaluate` accept `threads=N` for the D11 engine, **default 1**, and record the actual count. The reported backend is still `d11 multiplier-free scalar`. The unused `stack_gemv_pairs_blocked4` and its two tests are removed.

## Profile (sampling, `sample`, `lut-chat engine=d11`, everyday-32)

Self time per kernel, before (main at 14fc7e5a):

| kernel | 29M | 96M |
|---|---|---|
| `stack_gemv_pairs` (pair-table maps) | 62.5% | 64.8% |
| `stack_gemv` (nibble-table maps: down, rec_out, read out) | 25.4% | 26.5% |
| `stack_mul_u128` (mostly Hamilton products of the recurrence lanes) | 2.5% | 1.9% |
| recurrence, `stack_square`, `stack_div_u128`, `stack_isqrt`, `stack_mul_u64` | 6.5% | 5.2% |
| everything else | about 3% | about 1.6% |

After, at 1 thread on 96M, the maps are still 84.7% of the time (62.6% + 22.1%). The remainder is recurrence lanes and the read's L2 distances and mixtures (`mul_u128` 3.7%, `isqrt` 2.2%, `square` 2.0%, `div_u128` 1.8%). The pool parallelizes all of these.

## Speed: ids/s, generated ids over the wall time of all replies

These runs were back to back on the same M1. The machine was shared with other labs, with a load average of 5.5 to 8 during the series.

| artifact | before, threads=1 | before, threads=4 | after, threads=1 | after, threads=4 |
|---|---|---|---|---|
| 29M (export/model.lut) | 26.7 | 27.0 | 55.0 | 97.0 |
| 96M (c100-export/model.lut, sha256 abdadaf7…) | 8.71 | 8.67 | 16.3 | 32.5 |

- The "before" engine is single-threaded. Its `threads=` setting is ignored and its report records `threads: null`.
- The new reports record `"threads": 1` or `"threads": 4`.
- At e2309fae, a default run without `threads=` recorded `"threads": 1` (29M, 32/32 rows identical).
- At the head commit, the 96M artifact at `threads=4` was **32/32 rows identical** (28.3 ids/s under load). That commit changes only how pool closures capture their arguments.

The stated baseline, 8.64 and 23.2 ids/s, matches the "before" column within machine noise.

## Bit-identity evidence

- **Real artifacts:** every `lut-chat` run above compared `history_ids` and each turn's `reply_ids` and `stop` against the existing `chat.json` (`c100-everyday-32` for 96M, `int-everyday-32` for 29M). The result was **32/32 rows identical** for both artifacts, at both thread counts, before and after.
- **New unit tests:**
  - `every_thread_count_serves_the_same_integers` covers 2, 3 and 4 threads against 1 thread on the Lorentz, L2 and Dot reads, the free and the snapped transport, and with and without the copy scale. It checks the logits, the copy weights and the snap trace. Under `cfg(test)` the maps split into 2-row tasks and the recurrence into 1-lane tasks, so the small test stacks reach every split path and every uneven remainder.
  - `gemv_pairs_row_blocks_and_ranges_equal_the_nibble_kernel` checks every two-way row split of random maps.
  - `the_group_scale_is_the_wrapping_product` checks every `m` against edge and random values of `a`.
- **Commands:**
  - `cargo test -p uor-r4-integer --lib`: 224 passed.
  - `cargo test --release -p uor-r4-training --test stack_d11_oracle --test stack_d11_snap`: 13 and 2 passed. They passed both under the earlier 4-thread default and at the 1-thread default of the head commit.
  - `cargo test --release -p uor-r4-training --example geometric-stack`: 10 passed.
  - `cargo test -p uor-r4-integer --no-fail-fast`: every stack test passes.

**Pre-existing integration-test failures.** Seven `uor-r4-integer` integration-test files fail both before and after this PR:
- `adversarial_challenger_m1_token_boundary`
- `adversarial_challenger_m2_projection_latency`
- `challenger_m2_stress_test`
- `challenger_m3_hardware_invariants`
- `diagnostic_slot_scaling`
- `real_bundle_alignment_m1`
- `zero_matmul_4k_projection_m2`

Their fixture `/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/` (bundles `bundle-quaternion-1` and `bundle-householder_pair-1`) is absent from this machine. They test the older `IntegerModel`, not the D11 stack, so they are UNAVAILABLE rather than failing because of this change.

## Audit

At the head commit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` reports:

```
Scheduler instantiations over uor_r4_integer::stack closures: 38 in the binary (21 reached by the walk), all audited; 0 contain forbidden instructions
Total matched symbol ranges checked: 44
Class I (Multipliers) Violations: 0
Class II (Dividers) Violations:    0
Class III (Floats) Violations:    0
Missing Mandatory Symbols:        0
Call-Graph Reachability Checked: 83 functions
Call-Graph Violations:           0
FULL PASS: 0 forbidden instructions across 44 symbol ranges AND 83 reachable call-graph functions; all hardware invariants satisfied.
```

The same audit of the release `geometric-stack` binary that served the head's 96M check reports 43 instantiations (26 reached), 0 with forbidden instructions, and a FULL PASS across 44 symbol ranges and 81 call-graph functions.

**New mandatory symbols** (`session`):
- `stack_step`, `stack_map_pairs` and `stack_map_nibbles`;
- the pool task functions `stack_step_call`, `stack_map_task` and `stack_head_task`;
- the per-lane and per-head kernels `stack_lanes` and `stack_heads`.

The generic `stack_row_task` has three instantiations. The mandatory-symbol patterns do not match generic (`__RINv`) names, so it is not on that list. The walk reaches all three, and they contain no forbidden instruction.

## Audit scope

**What `--stack` mode skips (`STACK_SCHEDULER_ALLOW`).** The call-graph walk skips a function only when it is pure scheduler code. Its name must match one of:
- `rayon_core`: the pool registry, `WorkerThread::find_work`/`steal`/`wait_until_cold`, `Registry::new`/`in_worker_cold`/`in_worker_cross`, `join_context`, latches, sleep and wake;
- `crossbeam_deque`, `crossbeam_epoch`, `crossbeam_utils`: the job deques (`Injector::push`/`steal`), epoch reclamation and backoff that rayon-core uses.

The name must also carry no item of our engine. These crates are thread scheduling and compute no model value. They choose a victim thread, index a job queue, wait on a latch and wake threads, and they contain `mul`, `udiv`, `msub`, `madd` and `umaddl` for those purposes.

**What is not skipped.** A scheduler symbol whose name carries a `uor_r4_integer::stack` item (v0 `uor_r4_integer5stack…`) is a generic instantiated over one of our closures or task records. LLVM could inline a closure's arithmetic into it, so the audit checks every such symbol in the binary, reached or not, and walks into it like any other function. Instantiations created in our crate but not over our closures, such as `Registry::new` or the pool's drop glue, carry the crate name only as the instantiating-crate suffix. They are pool construction and teardown, not served computation, and stay skipped.

**What the stricter rule found and how it was fixed.** At e2309fae the stricter rule found 12 such instantiations with forbidden instructions, all `fmov xN, d0`. These were not arithmetic. rayon's `StackJob::execute` and `in_worker_cold`/`in_worker_cross` moved a two-word closure capture, or a job's result or panic payload, through a SIMD register. Every closure handed to the pool now captures exactly one pointer, to a task record (`StepCall`, `RowTask`, `MapTask`, `HeadTask`), and only calls a named `#[inline(never)]` function: `stack_step_call`, `stack_row_task`, `stack_map_task` or `stack_head_task`. The pool's wrappers now move one word, and all arithmetic stays in audited functions. After the change, 0 of the 38 instantiations contain a forbidden instruction.

**Other audit modes are unchanged.**

## Checks

- `cargo fmt --check`: clean.
- `cargo clippy -p uor-r4-integer`: 21 lib warnings, none in `stack/kernels.rs`, `stack/session.rs` or `stack/mod.rs`. They are in `codec.rs`, `model.rs`, `geometric_*`, `stack/flock.rs` and the bins.
- `python3 scripts/check_claim_wording.py`: passed.

## Risks and decisions for review

- **Threads are opt-in.** Existing `IntegerStackModel` users keep one thread unless they call `set_threads`, so their CPU use is unchanged.
- **Audit scope.** Leaving pure rayon/crossbeam scheduling out of the call-graph walk is a policy decision for the D11 audit's scope (see "Audit scope"). Instantiations over our closures and all kernels stay audited.
- **Timing noise.** The laptop was shared, so absolute ids/s vary by about ±10% between runs. Scaling from 1 to 4 threads is about 1.8 to 2× rather than 4×: maps of 576 to 1024 rows dispatch roughly 70 times per token, and idle rayon workers sleep between dispatches.
- **Memory.** Per-head read scratch adds `heads × context × 16` bytes per session, 98 KB for 96M.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5982464666 — 2026-10-04T17:14:55Z — Casey-allard

Exact-head review at 6ab25091d00593da73c750523c2c2fe9f324151a: **approved**.

**Audit scope (owner-accepted)**
- Pure scheduler symbols are skipped.
- All 38 rayon/crossbeam instantiations over our `uor_r4_integer::stack` closures are audited, and none contains a forbidden instruction.
- Each closure now holds one pointer and calls a named `#[inline(never)]` task function. Those task functions are mandatory audit symbols, so no model arithmetic can be inlined into scheduler code.
- The --stack audit fully passes on `uor-r4-stack` and on the `geometric-stack` binary that served the identity check.

**Engine and checks**
- The engine defaults to 1 thread; `threads=` is opt-in and recorded in reports.
- Tokens are identical: 32/32 rows on 29M and on 96M at threads=4.
- `cargo test -p uor-r4-integer --lib`: 224 passed. The stack oracle tests pass and `cargo fmt` is clean.
- The 7 `uor-r4-integer` integration-test files that fail are pre-existing: their `IntegerModel` fixture is absent.

Queuing.


---
_Comment/review ids in this file: 5982464666_

