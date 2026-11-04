# REPORT: float-kernel-fusion, claude-282 (done-when 1: the generated kernel list)

Lab: claude. [ORDER](./ORDER.md). PR #3684, item #3678. Plan step: THE ORDER of 4 November (#2028 6634218223), step 5, the kernel unit, part (b). Base `a0c0dd73d`. The code is two commits: `3b1201239`, and `d6fc3a2c5` with the exact-head review's fixes. Its GPU check ran at `d6fc3a2c5` on one RTX 5090, Vast `vast-59176411`.

**In one paragraph.** The float crate's kernel list is now generated.
- **What it is:** `build.rs` scans each PTX module's sources for `extern "C" __global__ void <name>` on every build, CPU builds included. It writes one list per module into `OUT_DIR/kernel_lists.rs`, which `lib.rs` includes.
- **What uses it:** every launch through `dispatch_module` passes `admit_launch` first, which refuses a name its module's list does not hold, and a module with no list.
- **What it replaces:** two hand copies of 26 names, and the counts written into five files' docs and messages.
- **What it changes in the model:** nothing. No math, weight or scored token moves. The arms kernels, launched through `launch_prebuilt`, pass the new check on the GPU.
- **Next:** done-when 2 to 5 (the measurement, then the fused quantizers if the cancel rule lets them start). A fused kernel added to `cuda_stack.cu` now changes one generated list, not two copies and four counts.

## Tokens per second first

No tok/s here. This unit changes no forward and no decode, and it times nothing. The kernel unit's next part, the measurement, times a training step.

## What did not work

1. **The first pod never answered SSH.** `vast-59174823` (Vast Alberta, $0.975/h) was created at 23:10:15Z. `uor-pod up` waited 900 s for SSH, then deleted it at 23:25:21Z. That is a host fault, not this unit's. A second pod, `vast-59176411` (South Korea, $0.563/h), answered SSH 36 s after it was created.
2. **GPU run 1 stopped at G1 (EXIT 4), an instrument fault in the job.** At `3b1201239`, the CUDA library suite passed: 342 passed, 0 failed, 8 ignored, exit 0. The three new dispatch tests and the two decode-cache tests passed. But the six `spiralcore_fbs_arms::gpu_tests` are `#[ignore = "needs a CUDA device; run on the pod"]`, so a plain `cargo test` lists them as ignored. The job's named-test gate refused, as it should: the ORDER's done-when 5 names those six. The repair is in the job only: it runs them with `--ignored`, by name.
3. **GPU run 2 stopped before any test (EXIT 3), a second job fault.** It looked for run 1's source folder under the new pin's name, found none, and stopped at its first check. Run 3 points at the right folder. Each run wrote to its own new folder, so runs 1 and 2 are kept as they stopped.
4. **The check guards one load path, not all of them.** `dispatch_module` is the float crate's only `get_or_load_custom_func`, so every float-crate launch is checked. The `uor-r4-training` crate keeps its own `cuda_stack_kernels.rs` and two more load paths (`geometric_selected_potential_cuda.rs:69`, `native_geometric_cuda_kernels.rs:30`) with no list. It is out of this item's scope.
5. **The arms module's name is written twice.** `ARMS_MODULE` in `cuda_stack_kernels.rs` copies `SPIRALCORE_FBS_ARMS_MODULE`, a private constant at `geometric_stack_native_cuda.rs:728`, which is out of scope. A CPU test ties the two: it reads `geometric_stack_native_cuda.rs` and fails if the constant is not that string. Making the constant public would remove the copy; that file is out of scope.
6. **One old count is left, outside the scope.** `spiralcore_fbs_arms.rs:558` still calls the stack's list "the 24-kernel extract". It was already stale (the list held 26), and that file is not in the item's scope. The next unit that edits it should drop the count.
7. **The scan reads text, not what nvcc compiled.** Three things guard the gap:
   - `build.rs` stops the build on any line holding `__global__` that is not a one-line declaration it reads. That covers a declaration split over two lines, a qualifier such as `__launch_bounds__`, and a macro that carries the token;
   - since `d6fc3a2c5`, it also stops the build on a kernel inside an `#if`, `#ifdef` or `#ifndef` block, which one build may compile and another not. The ORDER's stop rule asked for this, and the first commit did not do it (the review's first finding). Today no kernel sits in one: the two `#if` blocks in `cuda_stack.cu` (`:61-85`, `:120-131`) end before its first kernel at `:160`;
   - the GPU job (G3) compares each built PTX's `.entry` names with its list.

   The test's re-scan is separate code but uses the same rule as `build.rs`, so a blind spot in that rule would be shared (the review's third point). G3 is the check that reads what nvcc actually built.

## What changed

- **`build.rs`:** `kernels_of` scans one module's sources in order. Each of these stops the build: a `__global__` line that is not a declaration it reads; one inside an `#if`, `#ifdef` or `#ifndef` block (it counts the depth per file); a name declared twice; and a module with no kernel. `write_kernel_lists` writes `OUT_DIR/kernel_lists.rs` before the CPU early return: `CUDA_STACK_F32`, `CUDA_STACK_BF16` and `SPIRALCORE_FBS_ARMS_F32`, named after their PTX files, and `MODULES` with each module's PTX file, sources and kernels.
- **`lib.rs`:** `KernelModule` and `pub mod kernel_lists` (the included file) replace the literal array. `KERNEL_ENTRY_POINTS` stays, now the stack module's generated list.
- **`cuda_stack_kernels.rs`:**
  - `module_kernels` maps a candle module name to its list;
  - `admit_launch` refuses a name outside its module's list, and a module with no list;
  - `dispatch_module` calls `admit_launch` after the `HISTORICAL_KERNELS` deny check, before the PTX load;
  - the module doc now says only its `cuda` submodule needs the `cuda` feature;
  - four CPU tests: `dispatch_module` calls `admit_launch` before it loads (a check of the source text, since the load path needs a GPU); every module admits its own kernels; a launch outside its list is refused (the arms kernels in the stack modules, three stack kernels in the arms module, three names D29 removed and one no source declares in every module, and an unknown module); the arms module's name is the one its launches use, and its list holds both arm kernels.
- **`tests/no_training_crate_dep.rs`:** the literal 26-name copy is gone. The test scans each module's sources with its own code, compares the result with that module's generated list, checks that every `__global__` in them is a declaration it read and that none sits inside a conditional block (counted its own way), checks that every `src/*.cu` file is in some module, and runs the banned-token scan over every module's sources (it covered only two files before).
- **Doc comments with counts,** rewritten without them: `build.rs:4`, `lib.rs:3-4` and `:33-36`, `tests/no_training_crate_dep.rs:3-4`, `tests/twenty_four_kernel_controls.rs:1-5` (the file keeps its name) and `Cargo.toml`'s description.

## Measurements

The lists, as the laptop build wrote them (`kernel_lists.rs`):

| module | PTX file | sources | kernels |
|---|---|---|---|
| `uor_r4_geometric_stack` | `cuda_stack_f32.ptx` | `cuda_stack.cu` + `spiralcore_fbs.cu` | 26 |
| `uor_r4_geometric_stack_bf16` | `cuda_stack_bf16.ptx` | the same two | 26, the same names |
| `uor_r4_spiralcore_fbs_arms` | `spiralcore_fbs_arms_f32.ptx` | `spiralcore_fbs_arms_prelude.cu` + `spiralcore_fbs.cu` + `spiralcore_fbs_arms.cu` | 6: the 4 FBS kernels, `spiralcore_fbs_arm_fwd`, `spiralcore_fbs_arm_bwd` |

The stack's 26 are the old hand list's 26, in the same order (the test compares them in order).

The GPU check, run 3, at `d6fc3a2c5` on one RTX 5090 (driver 580.178.04, 32,607 MiB, nvcc 12.8, `compute_120`), all in `runs/c282c/`:

| gate | what ran | result |
|---|---|---|
| G1 | the crate's CUDA library suite, `--no-fail-fast`, one thread | 343 passed, 0 failed, 8 ignored, 141 s. The job requires five tests by name: three of the four dispatch tests and two decode-cache tests. All five pass, and so does the fourth dispatch test, the pin test |
| G1b | `spiralcore_fbs_arms::gpu_tests` with `--ignored` | 6 of 6 pass. They launch the arms kernels through `launch_prebuilt`, so each launch passed `admit_launch` |
| G1c | `in_space_long`'s ignored three-step trained-as-served test with `--ignored` | 1 of 1 passes, 3.1 s: three training steps' launches pass the check |
| G2 | `tests/no_training_crate_dep.rs` in the cuda build | 2 of 2 pass |
| G3 | each built PTX's `.entry` names against its generated list | `cuda_stack_f32` 26 = 26, `cuda_stack_bf16` 26 = 26, `spiralcore_fbs_arms_f32` 6 = 6 |

- **G1's 343** is run 1's 342 plus the dispatch pin test that `d6fc3a2c5` added.
- **The 8 ignored:** G1b's six, G1c's one, and `flock_read_hamming`'s timing run at long windows, which this unit does not touch.
- **G3** is the one check that reads what nvcc compiled rather than the source text. It agrees with the scan for all three modules.

## Checks run at the head

- **The laptop gates at `d6fc3a2c5`:**
  - the float crate's CPU suite passes, `--no-fail-fast`: 280 tests (249 in the library, 2 in `no_training_crate_dep`, 12 in `twenty_four_kernel_controls`, 17 in the binaries). At `3b1201239` it was 279, before the dispatch pin test;
  - the stub-nvcc `cargo check --features cuda --all-targets` passes.
- **Five negative gates,** each set aside as a patch against its commit and restored with `git apply -R`, the files then equal to the commit:
  - at `3b1201239`, with `admit_launch`'s `kernels.contains(&name)` made always true, `a_launch_outside_its_module_list_is_refused` fails at `cuda_stack_kernels.rs:497`, the first arms kernel admitted into the stack module;
  - at `3b1201239`, with `build.rs`'s scan made to skip each module's last source, `kernel_entry_points_match_cuda_sources_and_pass_d29_scan` fails at `no_training_crate_dep.rs:89`: the test's own scan of `cuda_stack_f32.ptx`'s sources holds the FBS kernels, and the generated list does not;
  - at `d6fc3a2c5`, with the `admit_launch` line deleted from `dispatch_module`, `dispatch_module_checks_the_list_before_it_loads` fails ("dispatch_module must call admit_launch before it loads the kernel");
  - at `d6fc3a2c5`, with `#if 1` inserted before `cuda_stack.cu`'s first kernel and `#endif` at its end, the build stops at `build.rs:127`: "src/cuda_stack.cu:161: a kernel inside an `#if` block";
  - at `d6fc3a2c5`, with that `#if` kept and `build.rs`'s depth check disabled, the test's own count fails at `no_training_crate_dep.rs:108`, listing all 22 `__global__` lines of `cuda_stack.cu` (which holds 22 of the stack's 26 kernels).

  Restored, each test passes again.

- **The GPU check:**
  - **the pod:** `vast-59176411`, Vast South Korea, $0.563/h, up 23:26:24Z to 23:43:11Z (1,003 s), about $0.16. The first pod, `vast-59174823`, was up 906 s at $0.975/h before `uor-pod` deleted it, at most $0.25. So the unit spent at most about $0.40 on pods;
  - **the job:** `job-282b.sh`, kept with the lab's scripts. It ran as `job-282c.sh` with the pin as its argument, and refuses any other pin. It refuses to write into an existing run folder;
  - **the seal:** run 3's `SHA256SUMS.files`, SHA-256 `bd762511568217b070200226cfe19d56557f6c8e0eb6f113bf38b429ee34c0bf`. It checks 29 of 29 after the copy off the pod, and 29 of 29 again after a full read-back from the store;
  - **the store:** `caseyallard/uor-r4-store:results/claude/float-kernel-fusion-kernel-list-2026-11-04/` holds `c282/` (run 1, 13 files), `c282b/` (run 2, 7 files) and `c282c/` (run 3, 33 files: the 29 sealed, the seal, its empty error file, `DONE` and `EXIT`). The read-back of all 53 files is identical to the copy;
  - the pod was released and deleted after the read-back, and `uor-pod status` then showed 0 running pods.
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews, each in a fresh context:**
  - **the exact-head code review at `3b1201239`:** APPROVE, no blocker. It found two minors and two nits:
    - a kernel inside an `#if` was not refused at build time, as the ORDER's stop rule asked: fixed in `d6fc3a2c5` (What did not work, item 7);
    - no test pinned the `admit_launch` call in `dispatch_module`: fixed in `d6fc3a2c5`, with a test that fails with the call deleted;
    - the re-scan shares `build.rs`'s rule, so a blind spot in it would be shared: kept, and recorded in item 7. G3 is the independent check;
    - the module doc still said the whole module needs the `cuda` feature: fixed in `d6fc3a2c5`.

    It ran no build, by its brief, and named the GPU suite and the set-asides as the author's to run. Both are above.
  - **round 2, at `40ccdde2b`,** on the fix delta (`3b1201239..d6fc3a2c5`) and on every claim in this REPORT against the run files, the job scripts, the patches and the pod ledger: APPROVE WITH CHANGES, no blocker.
    - **The code:** the depth counter, the pin test and the doc fix are correct. Among the cases checked: `# if` with a space, nesting, `#elif` and `#else`, and a directive line that names `__global__`. Its three nits are [Still open](#still-open), item 3.
    - **The claims:** one minor, Still open's cross-references, which pointed at the wrong items; fixed here. The pod total is now the ledger's $0.40, not $0.41.
    - **Not checked by it, checked by hand:** the five tests `job-282b.sh` requires by name (three dispatch tests and two decode-cache tests, at its `NAMED` list); the seal's SHA-256 (`shasum -a 256` of the local copy and the read-back); the 29 / 29 seal checks; and the 53-file read-back (`diff -r`).
  - Both are recorded by `loop review`.

## Skills used

None applied.

## Still open

1. **The kernel unit's next part: the measurement** (done-when 2's precondition, and the item's cancel rule). Time a training step at P2b's recipe with `--train-as-served 1`, chain by chain: the two quantizer chains (`activation_quantize16`, `q16_residual`) against the rest of the step. If they are under 5% of a step, the fusion does not start and the item closes on done-when 1. Time it on one pod type, or report the host: claude-281 saw 137–148 tok/s on one 5090 against claude-279's 98.7 on another, with the same cache.
2. **The out-of-scope points** above: the training crate's own unlisted load paths (What did not work, item 4), the private arms constant (item 5) and the stale "24-kernel" doc (item 6).
3. **Review round 2's three nits,** kept out of this PR so the merged code is exactly the code the GPU run qualified. None changes a list, a launch or a scored token. They land with the next unit that edits these files:
   - the test's `#if` counter can go below zero on a stray `#endif`, where `build.rs` saturates. nvcc rejects an unbalanced `#endif`, so no real build reaches it. Saturate it too, or assert the depth never goes negative and ends at 0;
   - both counters read a `#if` or `#endif` at the start of a line inside a `/* */` comment as a directive. A commented `#if` refuses a kernel (safe); a commented `#endif` inside a real block would hide one. No source does either. This is the shared rule of What did not work, item 7;
   - the pin test matches the call's text, so `let _ = super::admit_launch(module, name);`, which drops the result, would still pass. Put `.map_err(Error::Msg)?` in the needle.
4. **Then, as #2028 orders:** the real-text corpus; P4 (#3471); P5; P6 (#3394).

## Plain summary

The float crate keeps a list of the GPU programs it is allowed to run. Until now that list was typed out by hand twice, and five files said how long it was. Adding one GPU program meant changing all of them, and nothing checked a launch against the list at all.

Now the build reads the GPU source files and writes the list itself, once per compiled module. Every launch is checked against its own module's list and refused if the name is not there. A test reads the same source files its own way and fails if its list and the build's disagree. On a rented RTX 5090, the crate's whole GPU test suite passed with the check in place. That includes the tests that run the newest GPU programs and three steps of training. The programs the GPU compiler actually built matched the lists exactly. Getting there took three tries: the first two failed on mistakes in my test script, not in the code. The pods cost about $0.40 in all.

This changes nothing the model computes. It is the groundwork for the next step, faster fused GPU programs for the served rounding, which first has to be measured to see whether it is worth doing.

References #3678
