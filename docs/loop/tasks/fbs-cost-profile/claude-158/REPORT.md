# REPORT: fbs-cost-profile, claude-158

Lab: claude. Order: [ORDER.md](ORDER.md). Code: `d1078d5cb02838e3c9b738cdbac902fda02639ec`. Device: the M1 laptop (Apple M1, 4 performance cores, 16 GiB, macOS 27.0, AC power).

One served token is now split by stage. Within each platform, the costs fall in opposite places from training's (claude-157). On the M1 the six-layer trunk is **98.6%** of a token's step, while the writer is 0.6% and the cost outside the step 1%. A whole-table nearest readout would cost 21 times the writer.

## What did not work, first

- **The first test run executed zero tests.** `cargo test -p uor-r4-integer --lib -- fbs_stage_tests` reported `0 passed; 56 filtered out`, because the stack runtime compiles only under the `serving-wasm` feature, which `fbs-toks` itself requires. Rerun with `--features serving-wasm`: `2 passed`.
- **A heredoc append was refused** by the loop guard, which read `-> u32` in the Rust text as a shell write to a file named `u32`. The test module went in with the Edit tool instead.
- **Nothing here could split each layer's two blocks further.** The recurrence block holds three route gathers and the recurrence core. The feed-forward holds three gathers, the gated product and the requantization. Which part dominates each is not measured.

## What changed

- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - **`IntegerStackSession::measure_fbs_step_stages`**, the stage-timed twin of `stack_step_fbs`, as `measure_step_phases` is of the dense step.
    - It runs the same kernels in the same order and returns the same `NativeStep`.
    - `step` and `stack_step_fbs` are untouched, and the zero-multiply audit of `step` still passes.
    - It times the embedding, each layer's recurrence and feed-forward block, the final norm and query, the writer, the bookkeeping and the whole step.
    - Beside the step, it times a whole-table nearest search over the query (`stack_whole_table_nearest`), which reads only.
  - **Two host tests:** the twin is the served step, and the nearest search breaks ties to the smaller id.
- **`crates/uor-r4-integer/src/bin/fbs-toks.rs`:** `--stages`. After the plain protocol, whose numbers are unchanged, it runs the same 1 + 20 runs through the twin and adds `stages`:
  - each stage's median over the 1,280 timed decode steps;
  - their sum against the step;
  - the nearest's median;
  - the step against the plain per-token time.

  It also gains a host test of that arithmetic.
- **The item** is set `done`, with a note: its done-when 1 to 4 are met by claude-157 and this unit.

## Measurements

**The artifact:** claude-126's seed-1 export, sha256 `314c3909…fdb305`, which matches its seal. Width 256, 6 layers, vocabulary 4,096: the shape of claude-152's (a) trunk.
- **Binary:** `fbs-toks`, release, sha256 `5008b0142e4ba356a7466a7a2f862e50bc9e0fb2a4439b3a2ffd01d6d27bc85d`.
- **Protocol:** 1 warm-up and 20 timed runs, each a 128-token prefill and 64 decode steps.

**Instrument checks:**

| Check | Result |
|---|---|
| I-1 the twin is the served step (host test, 48 positions of a synthetic trunk) | the same tokens, kept cells and queries at every step; `save_state` and tokens equal at the end |
| I-2 the nearest search against brute force, ties included | equal at every position; ties go to the smaller id |
| I-3 replies with `--stages` against the plain protocol | `replies_match: true` |
| I-3 the stages against the step | `3,709,746 / 3,733,896` ns = `993` per mille |
| I-4 plain decode rate against claude-126's on this artifact | `265` tok/s against `231` (+15%, inside the 20% band) |

**One decode step, median ns over 1,280 steps:**

| Stage | ns | Share of the step |
|---|---|---|
| embedding | 83 | 0.00% |
| each layer's recurrence block (×6) | 351,729–363,625; sum 2,146,270 | **57.5%** |
| each layer's feed-forward block (×6) | 252,583–258,937; sum 1,534,019 | **41.1%** |
| final norm and query | 6,292 | 0.17% |
| writer | 23,041 | 0.62% |
| bookkeeping | 41 | 0.00% |
| **stages summed** | **3,709,746** | **99.35%** |
| **the step** | **3,733,896** | 100% |
| outside the step: the loop, and `step_native`'s copies of the kept cells and query (plain per token 3,769,877 less the step) | 35,981 | 0.95% of the plain token |
| *beside the step:* a whole-table nearest over 4,096 rows | 486,291 | 13.0%, 21.1× the writer |

So a token costs 3.73 ms, of which 3.68 ms is the six layers: 613 µs a layer. The trunk holds `64,000` learnable values (claude-155), so a token spends about 57 ns per trunk value.

**The top three:**
1. **The recurrence blocks:** 2.15 ms, 57%.
2. **The feed-forward blocks:** 1.53 ms, 41%.
3. **The writer:** 23 µs, 0.6%. The whole-table nearest, if it were the served readout, would be 486 µs and third by a wide margin.

**The cheapest change for each** (item done-when 3):
1. **The recurrence blocks.**
   - The block is three route gathers around the recurrence core, in scalar integer code with no NEON (the [pipeline map](../../../../model/pipeline-map.md), stage 11).
   - **The change:** vectorize the gathers and the core with NEON, after splitting the block to see which part dominates.
   - **Effect on prediction:** none, if it is bit-exact. The I-1 test is the check: the vector step must equal `step_native`. How much it saves is a hypothesis until it is measured.
2. **The feed-forward blocks.**
   - The block is three route gathers, the gated product and the requantization, also scalar.
   - **The change:** the same NEON work, and the same check.
   - **The hypothesis:** the gathers dominate both blocks. A layer's routed projections hold `8,448` values (claude-155). At today's 613 µs a layer, that is about 73 ns per routed value: an upper bound by arithmetic, since the layer's time includes the core. Either way it is far above a load and an add.
3. **The readout.**
   - The writer already costs 0.6% of a token, so the cheap serving choice is the writer, not whole-table nearest, which costs 21 times as much. But on one checkpoint the writer's top-1 is `345` against nearest's `7,813` ([pipeline map](../../../../model/pipeline-map.md), stage 7).
   - **The change:** fix the writer's admission, so the cheap readout predicts as the trained one does (the `one-decoder` line).
   - **Effect on prediction:** that is what the change is for. It is measured there, not here.

**Against training (claude-157):** only shares within each platform are compared.
- **Training:** the trunk's forward is 5% of a training step on the GPU. Its backward sits inside the backward, which is not yet split, so 5% is a lower bound.
- **Serving:** the six layers are 98.6% of a served token on the M1.

The two also differ in hardware, in number type (float against integer) and in what a step holds. Even so, the costs fall in different places: training's in the objective and the backward, serving's in the trunk's scalar kernels. Each needs its own fix.

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-cost-profile-serving-2026-10-28/`:
- `plain.json` and `stages.json`, the two `fbs-toks` outputs;
- `binary.sha256`;
- `SHA256SUMS-claude-158`, read back with `shasum -c` OK.

## Checks run at the head

- **`cargo test -p uor-r4-integer --features serving-wasm --lib -- fbs_stage_tests`:** `2 passed; 0 failed`.
- **`cargo test -p uor-r4-integer --features serving-wasm --lib -- stack::`:** `73 passed; 0 failed`.
- **`cargo test -p uor-r4-integer --features serving-wasm --test m5_full_path_cost_controls`** (the zero-multiply audit of `step`, and `measure_step_phases`' equality): `3 passed; 0 failed`.
- **`cargo test -p uor-r4-integer --features serving-wasm --bin fbs-toks`:** `3 passed; 0 failed`.
- **`cargo clippy -p uor-r4-integer --no-deps --features serving-wasm --all-targets -- -D warnings`:** no finding in either file.
- **`cargo fmt`:** clean.
- **`loop report --check`:** PASS. **`loop gates --full`:** 11 of 11 green.
- **The fresh in-unit review, in two parts at `40abae097`:**
  - **The code:** APPROVE, four low findings. Two are fixed in code:
    - `ns` is cleared before the non-FBS refusal;
    - `loop_overhead_ns` is documented as the cost outside the step, which includes `step_native`'s copies.

    The test's `>=` check is now marked a sanity check: the stages tile the step, so it cannot fail. The reviewer confirmed by reading that the step time excludes the nearest search. `--stages` matching any argument is left as is.
  - **The numbers against the sealed outputs:** CHANGES. Every measured number matches; one medium and five low wording findings, all fixed:
    - **medium:** claude-157's 5% is the trunk's forward only, a lower bound, so the comparison now stays within each platform;
    - the trunk's share is 98.6%;
    - the cost outside the step is 0.95% of the plain token;
    - about 73 ns per routed value, as an upper bound;
    - "per learnable value", not "per weight";
    - the item's old "no split" line is dated.

## Skills used

- **holding-the-goal.** The owner's criterion is cost per token. The unit measures where a served token's time goes, and stops at naming the changes. The NEON kernels and the writer's fix are their own units.

## Still open

- **Each layer's split:** the gathers against the recurrence core, and against the gated product. This decides where NEON pays.
- **The NEON kernels**, bit-exact against `step_native`.
- **The writer's admission** (`one-decoder`), so that the 21-times-cheaper readout predicts.
- **Training's open items** from claude-157: the backward's split, and a cheaper rival search.

## Done when

1. **Met.** I-1 to I-4 pass, with their outputs above.
2. **Met.** One served token, by stage, on the M1, summed within 10% (`993` per mille), with the writer and the whole-table nearest each timed. The cost outside the step is 1%.
3. **Met.** One line for each of serving's top three.
4. **Met.** A notebook row and this REPORT. The item's done-when 1 to 4 are all met (claude-157 and this unit), so the item is `done`.
5. **Met on merge.**

## Plain summary

On the laptop, a served token takes 3.7 ms, which is 265 tokens a second.
- Almost all of it, 98.6%, is the model's six layers, in plain integer code that uses no vector instructions.
- Choosing the token afterwards takes well under 1% with the model's own writer. Checking every word in the vocabulary instead would take 21 times as long.

So, to serve cheaply:
- speed up the six layers' integer code, which is about 57 ns per learnable value today;
- make the cheap writer pick as well as the expensive search does.

In training (claude-157) it was the other way round: the layers were cheap, and the backward pass and the vocabulary search were not.
