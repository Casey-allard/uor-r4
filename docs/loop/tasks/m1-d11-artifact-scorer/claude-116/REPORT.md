# REPORT: m1-d11-artifact-scorer, claude-116

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`m1-d11-artifact-scorer-2026-10-25.md`](../../../../harness/prep/m1-d11-artifact-scorer-2026-10-25.md). Instrument: [`docs/labs/m1-d11-artifact-scorer/instrument/`](../../../../labs/m1-d11-artifact-scorer/instrument/).
Closes #3248.

## What did not work, first

- **The first route fixture could not catch a wrong composition order.** I swapped the order of `compose` in the route read as a planted bug, and the parity tests still passed. The reason: `BRANCH[r][0] == r` and `BFS` is the identity on its diagonal, so most route steps compose with the identity, and the order never shows. I widened the route fixture to 16 roots, 4 blocks and 256 rows, and that bug now fails the tests (`route: row 2 (length 80): read vector differs`).
- **cudarc's safe module loader exists only with its `nvrtc` feature.** Turning that feature on would compile NVRTC bindings into the binary, which CHARTER §3 bans. So the scorer loads its cubin through the raw driver API instead: `cuModuleLoadData`, `cuModuleGetFunction` and `cuLaunchKernel`, from `cudarc::driver::result`. `cargo tree` shows cudarc built with `std`, `driver`, `cuda-12080` and `dynamic-linking` only.
- **The kernel first built the row plan itself, per row.** The ORDER says the host precomputes it. I moved it to a host table (`kernel::row_plans`, one `D11Plan` per prefix length), which also cut the scratch memory per row from 37,000 bytes to 22,528. The old record held nodes, frontier and counts besides the per-node arrays: 9,250 words. The new one holds 5,632 words.
- **The FIT found the 14,287-row panel test could pass without running:** with `UOR_DEV_CONFIG` unset, it printed "skipped". It now fails instead.
- **The work stayed one PR,** so the stop rule for splitting it was not needed.

## What changed

All of it is in `docs/labs/m1-d11-artifact-scorer/instrument/`, a detached `[workspace]` with path dependencies on `uor-r4-discrete` and `uor-r4-integer`. Neither crate was edited.

- **`export CHECKPOINT OUT_LUT REPORT`** (stage 1, `03dce9d28`). It writes `export_d11`'s bytes unchanged, after checking the checkpoint's schema, its `model_sha256`, and that `epoch` equals `accepted_updates`. The integer reader must then accept the file.
- **`rows CONFIG OUT`** (stage 1). It rebuilds the DEV rows by `eval-corpus`'s own batch walk over `uor_r4_discrete::corpus`.
- **`score ARTIFACT CONFIG REPORT ROWS [--reference]`** (stage 2, `79fd83448`).
  - The kernel is `src/kernel_core.h`, an integer port of `evaluate_prefix`: the Route, Flock and Relation reads, admission and the top-8, one thread per row.
  - `build.rs` copies the geometry tables from `discrete_d11.rs`'s own literals. It builds the row body as host C++ for the parity tests. It compiles `src/scorer.cu` with nvcc ahead of time, admits the PTX by the discrete crate's integer rules (mirrored, with their self-test), and assembles the cubin.
  - The report has integer `top1`, `top8` and `admitted` totals, plus each row's prediction, top-8 and admitted cells.
- **`toks ARTIFACT REPORT [--prompt TOKENS]`** (stage 3, `638a939ad`). It runs the step-6 protocol through `IntegerDiscreteModel`.
- **The prep sheet** (`d2089be9c`), and its FIT and D67 fixes (`932779e32`). The FIT and the D67 review are posted on #3248.
- **Fixes from the fresh code reviews** (`ffc02ee09`), host side only.

**One design call:** the ORDER says "one row per block". The kernel runs one row per thread, with blocks of 64 threads. A row is a sequential tree walk, so there is no in-row work to share across threads. Parity is unaffected, and the fixtures plus the 14,287-row run show it.

## Measurements

**GPU qualification (stage 4): parity on all 14,287 of 14,287 rows.** It ran on one RTX 5090 (`vast-57733465`, compute capability 12.0, nvcc 12.8.93), at `932779e32d1fedee85a6343ca06cf64bc0d2b136`, on 25 Oct. The job took 35 s of wall time, from 23:06:00Z to 23:06:35Z. The pod was up for 288 s (from `up` at 23:03:40Z to `down` at 23:08:31Z) at $0.975/h, about $0.08 (compute ledger). The decision row of [the sheet](../../../../harness/prep/m1-d11-artifact-scorer-2026-10-25.md) reads **qualified**.

| Check | Result |
|---|---|
| Build | nvcc compiled ahead of time and admitted the integer PTX (sha256 `e8ee1c12…`). The cubin targets `sm_120` (`17dd4184…`). ptxas reports 71 registers, a 704-byte stack and 0 spills. The binary `d11-scorer` is `73c3ddf8…`. |
| (a) The finite tests, `--include-ignored` | 17 of 17 pass. `gpu_scores_are_bit_identical_to_the_reference_and_the_host_build` shows the GPU equal to the host build and to `evaluate_prefix` on the Route, Flock and Relation fixtures, on the 16-block 243-context bound, and on 4,100 rows across a launch-chunk boundary. `the_registered_dev_panel_has_14287_rows` passes too. |
| `export` | 542,260 bytes, sha256 `42671d44…`: the same file the laptop wrote |
| `rows` | 14,287 rows, `rows_sha256` `d933c533…`: the same as the laptop |
| `score --reference` | `agreed` 14,287 = `checked` 14,287, `refused` 0, `first_mismatch` null. The GPU pass took 28,542 µs for 14,287 rows, transfers included. |

The totals on this stand-in artifact are top-1 741, top-8 1,345 and admitted 4,251, each out of 14,287. They qualify the instrument only. They are not step 6's measurement, and they are not an export gap: the A8 run sealed no `eval-corpus` evaluation of this checkpoint (`final_dev` is null), so there is nothing to subtract. The evidence is in the store under `results/claude/d11-artifact-scorer-2026-10-25/`: 17 files in `pod/` and 5 in `laptop/`, each folder with its own `SHA256SUMS`.

**After the GPU run,** the fresh code reviews brought host-side fixes in `ffc02ee09`, and `kernel_core.h` was not touched. The fixes are a row-length check in both scorers, a reason match for rows both engines refuse, and refusal of a 0 ns timing sample. `kernel_core.h` is byte-identical between the qualified head and this one. The fixes were tested on the laptop.

**Laptop serving rate, from `toks`.** This qualifies the instrument and is not step 6's measurement. The artifact was A8 FrozenZero `continuation-05248` (vocabulary 4,096, 4 blocks, context 192, Relation with kept 8 and 1 sweep), on an Apple M1 (4 performance and 8 total cores, 16 GiB), macOS 27.0, on AC power, with the default deterministic prompt:

| Rate | median | p95 (nearest rank 19 of 20) |
|---|---|---|
| prefill (192 tokens per run) | 64 µs, 2,967,175 tok/s | 69 µs, 2,765,893 tok/s |
| decode (64 tokens per run) | 4,826 µs, 13,259 tok/s | 4,857 µs, 13,176 tok/s |

Every one of the 21 decode runs wrote the same reply (sha256 `203768f9…`). The timed decode also includes sliding the window: one prompt copy per run and one `Vec::remove(0)` per step. That cost is small next to 64 `evaluate_prefix` calls.

**Host checks.**
- 15 tests pass and 1 is ignored off the pod. On the laptop, the ignored panel test also passes against the store's DEV files: 14,287 rows, `rows_sha256` `d933c533…`. These laptop runs and the planted bugs below ran in this session's terminal; their logs are not sealed. The sealed test log is the pod's `tests.log`.
- Nine planted bugs each fail the tests. Each was set aside, run and restored, with `cmp` confirming the restore:
  - the route's composition order;
  - relation edge 3 and relation edge 4;
  - the tie in the level top-k;
  - the tie in admission;
  - the parent span in the plan;
  - the decode window's slide;
  - the even-count median;
  - the refusal-reason check.
- clippy with `-D warnings` exits 0 with and without `cuda` (using a stub nvcc).
- fmt is clean. The d23, d25 and d32 ratchets are OK.

## Checks run at the head

- **The GPU qualification** at `932779e32` passed every check in the sheet's pass row (see Measurements; evidence in the store, `pod/SHA256SUMS` sha256 `fb6ffc53…`). The fixes since then (`ffc02ee09` and the final commit) touch only the host side. `kernel_core.h`, `scorer.cu` and `build.rs` are byte-identical to the qualified head.
- **Instrument tests on the laptop** (`cargo test --release --target aarch64-apple-darwin --no-fail-fast`) at the final head: 15 pass and 1 is ignored. Two review fixes had their own negative gates: the refusal-reason check, and the host row-length check, each set aside so that `rows_the_reference_refuses_are_refused` fails.
- **Lint and ratchets:** clippy with `-D warnings` exits 0 with and without `cuda` (stub nvcc). `cargo fmt --check` is clean. `uor-harness ratchet d23`, `d25` and `d32` are OK, with no softmax, dense matrix product or discrete-path float site added.
- **Harness:** `uor-harness check` reports 0 errors once the sheet carries the `prep.sheet_shape` blocks. `loop report --check` passes, and `loop pr-check --local` passes with two advisory warnings: a "stop rule" heading in the sheet, and the ORDER's size.
- **Fresh reviews (D66), all from non-authors with fresh context:**
  - The kernel port: APPROVE.
  - The build and driver: APPROVE, with one should-fix, the row-length check, now fixed.
  - The reader and the CLI, in two passes, both cut short by the context budget: no defect found; their notes are fixed (refusal reasons, paired outputs). Their open items were settled from the code: the `training.rs` epoch check, the eval-corpus walk, the plan bounds, and the host check call.
  - The tests and `toks`: APPROVE, with two should-fix: the GPU test must show as `ok`, which the pod log does, and the decode reply must not be one repeated token, now asserted.
  - The claims check of this REPORT, the notebook rows and the #2839 note: CHANGES, now fixed. It found the pod time and price, this section, a verdict word and a sheet line number.

## Skills used

- holding-the-goal: the parity bar stayed bit-identical through every fix, and each planted bug was classed as a gap in the instrument and repaired before any GPU run (§4.1, §4.2).
- gates: a negative gate for every claimed check, and two fresh reviews (FIT, then D67) before the pod.

## Still open

- **Δloss and the planted-fault control B4** on sheet line `:82` have no GPU port. The step-6 unit owns them.
- **The step-6 run itself** (#2839): it exports and scores its own trained checkpoint with these commands, and times it with `toks`.

## Plain summary

Step 6 of the plan has to measure the exported integer model on its own: how often it predicts the next word on the 14,287-row DEV panel, and how fast it serves. The tools it named for that did not exist. I built three commands:
- `export` turns a training checkpoint into the integer file.
- `score` grades that file on a GPU. It uses a new integer CUDA kernel that copies the reference evaluator exactly.
- `toks` times the file on the laptop by the step-6 protocol.

I qualified them on a stand-in checkpoint from the A8 run, using one rented RTX 5090 for under five minutes (about 8 cents). On every one of the 14,287 rows, the GPU kernel gave the same read vector, admitted cells, top-8 list and prediction as the reference evaluator. The laptop served the same file at about 13,000 tokens per second while writing a reply. These numbers check the tools; step 6 will measure its own model with them.
