# REPORT: float-kernel-fusion, claude-284 (the cancel probe and the kernels' scope)

Lab: claude. [ORDER](./ORDER.md). PR #3689, item #3678. Plan step: THE ORDER of 4 November (#2028 6634218223), step 5, the kernel unit. Base `bff50c993`.

**In one paragraph.** The fused kernels can match both served quantizer chains bit for bit, so the work goes ahead; it was not cancelled.
- **Why it can work:** every op in the two chains has one IEEE binary32 rounding on CUDA.
- **Where the kernels go:** not in `cuda_stack.cu`. Its SHA-256 is pinned as earlier sheets' source identity, so the kernels get their own source and PTX module.
- **What this unit lands:** the probe, and the scope the kernels need. The kernels themselves are claude-285's, under the plan in the ORDER.
- **What it changes in the model:** nothing.

## Tokens per second first

Nothing was measured; no training or serving speed changes. The ceiling for what fusion can save is still claude-283's: a training step is 7.5% faster at (S) and 2.6% at (L) with both chains skipped entirely.

## What did not work

1. **The kernels could not start in this unit.** The loop's scope rule reads an item's scope at the merge base (`loopprcheck.rs:431-446`). So a file the item does not list yet, such as the kernels' own `.cu` source, fails the check until a scope change reaches `main`.
2. **The scope could not go through its usual item.** Earlier scope changes went through `infra-loop-harness` units (claude-223, #3453). `loop claim infra-loop-harness` was refused: its scope `docs/roadmap/items/*.md` overlaps this item's own file, which this item's open claim holds. So this unit lands the scope itself; the item file is in its own scope.
3. **Appending to `cuda_stack.cu` was ruled out** (the ORDER's probe, item 5). `git grep` at the base finds its hash in 12 files: two of code (`spiralcore_fbs_sheet.rs:54`, whose test at `:4156` asserts that the pin matches, and one lab runner), seven lab records and notes across three sheets, and three docs. One appended kernel would turn that test red and re-identify those sheets' source.

## What changed

- **The ORDER** records:
  - the cancel probe: every op in both chains, its CUDA rounding, the gradient, the callers, the overlap and the pin;
  - the scope decision, with its reasons and reversal cost;
  - the plan for claude-285: one module of four kernels, two custom ops with CPU twins, the flag and its option, CPU tests, and three GPU gates (the ops, one training step at P2b's recipe with an A/A first, and the PTX's explicit rounding), plus claude-283's carry-overs.
- **The item** `docs/roadmap/items/float-kernel-fusion.md`:
  - its scope adds `crates/uor-r4-float-train/src/fused_quantize*.cu` and `crates/uor-r4-float-train/src/in_space_long.rs`;
  - *How to start* says where the kernels go and why. `in_space_long.rs` is for the one-step gate's test only: only its `tests` module reaches the served model parts and the step's loss.

## Measurements

None. The probe is a reading of the code:

| Op in the chains | Its CUDA form (vendored candle-kernels 0.9.2, `-ftz=false -prec-div=true -prec-sqrt=true`, no fast math) | One rounding? |
|---|---|---|
| `abs`, the row max | `fabsf`; `fmaxf` from `-INFINITY` | yes; the max is exact in any order |
| `clamp` | `maximum` then `minimum`: `fmaxf`, `fminf` | yes |
| compares with a scalar | the f64 bound cast to f32 to nearest (`cmp`, `tensor.rs:1125-1131`); each bound is exact in f32, except `i32::MAX`, which rounds to 2^31 | yes |
| `affine` `x * mul + add` | may be contracted to an fma (no `--fmad` flag, so its default); for the chains' constants both forms give the same bits, −0 included | yes |
| `floor`, broadcast multiply and divide, `where_cond` | `floorf`; IEEE multiply; precise division | yes |
| the gradient | `floor`, compares and `detach` carry none (`backprop.rs:582-587`). With the bridge on it is `grad · inside`; `q16_residual` without its bridge gives `x` no gradient at all | exact |

## Checks run at the head

First run at `a1db79136`, then again at the final head, after round 2's fixes, with the same results. The commits after `a1db79136` change this unit's ORDER and REPORT only.
- **`uor-harness loop report --check --order <ORDER>`:** PASS.
- **`uor-harness loop pr-check --local`:** PASS, with two advisory warnings: the ORDER's size on the read path, and four rows missing from `docs/start/FILEMAP.md`, none of them this unit's.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- No code changed, so no cargo suite ran.
- **Fresh exact-head reviews:**
  - round 1 at `a1db79136`: CHANGES, three should-fixes and five nits, all fixed:
    - the gradient claim was wrong for `q16_residual` with its bridge off, which has no gradient. The plan's backward and G1 now say so;
    - the pin's records were undercounted; the ORDER now lists all 12 files;
    - this section was unfilled;
    - the nits: the compare's line reference, the bounds sentence, the `in_space_long.rs` wording, the early returns the flag must keep first, and G3, now supporting evidence that also rejects `.ftz`;
  - round 2, a fresh reviewer on the delta from `a1db79136` to `6ba36c1ff`: CHANGES, one should-fix and two nits, all fixed. The REPORT's breakdown of the pin's 12 files summed to 13; the checks named the head they ran at, not the final one; and G3 said subnormal rows would catch a contraction, which random rows catch. The final verdict is in the PR's review record.

## Skills used

None beyond the loop's own steps.

## Still open

- **claude-285: done-whens 2 and 3,** under the ORDER's plan, once this unit is on `main`.
- **Done-whens 4 and 5** after that: the cost on a 5090, and the flag's default.
- **The scout's findings for P4** (#3471) are in the claude lab's place record; P4 stays blocked by kimi's #3675 scope lock on `crates/uor-r4-integer/**`.

## Plain summary

Before writing custom GPU code for the two rounding steps that make training match serving, we checked whether one kernel each can give exactly the same numbers as the many small steps it replaces. It can: every step rounds in one fixed, standard way.

We also found that the main kernel file's fingerprint is recorded by earlier experiments, so the new kernels go in a file of their own. The project's rules needed that new file listed for this item first, so this unit adds it. The next unit writes and tests the kernels.
