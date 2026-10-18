# Row-parallel exact counterfactual — GPU parity and timing, 18 October 2026

**Status:** finite GPU parity **PASS**, negative gate **FAIL as required**, about **56–57× faster** proposals at the Dose 1 shape with identical losses. The path is opt-in; the default `propose` is unchanged. Making it the default and wiring it into the training driver are separate PRs. No training ran and no hold changed.

## Why

`discrete_counterfactual` launches one block of 16 threads, and Gate proposals use only 3 of them. Each thread walks every row serially and recomputes the baseline. That kernel is 99.99% of `Device::propose`, and so of every learning dose's wall time: Dose 1 projected about 37–67 h.

The owner chose "Build now, Dose 1 runs on". The faster kernel had to give exactly the same losses, and the running Dose 1 stays untouched.

## What

The new code is a separate generated CUDA module, `crates/uor-r4-discrete/src/rows_source.rs`. It contains the ordinary helpers and writer byte for byte, plus three integer entries:
1. **Baseline:** once per row.
2. **Virtual move:** once per (row, lane).
3. **Reducer:** per lane, replaying the ordinary loop over those records in row order — refusals, eligibility, break, overflow and status — so it publishes exactly what the ordinary lane would.

It is reached through an explicit opt-in handle, `Device::rows_counterfactual` plus `propose_rows`, and the D29 checker admits exactly the three entries. The suite is `rows_qualification.rs`, run by the `rows-qualify` binary. The suite is a separate binary, so the training driver's source identity is unchanged.

`device.rs` `source_sha256` changes, as with the earlier reuse and decomposition modules. That is declared in the sheet. Dose 1 runs its own pinned binary and is unaffected.

## Run

- **Sheet:** [m1-counterfactual-rows-parity-2026-10-18](../../harness/prep/m1-counterfactual-rows-parity-2026-10-18.md), TEST FITNESS FIT from a non-author reviewer subagent (owner-authorized peer review).
- **Pod:** one RTX 5090, `vast-56495093`, driver 580.95.05.
- **Build:** release, CUDA 12.8, `compute_120`, cloned at the exact branch SHA, with source file hashes checked before building.

| Item | Attempt 1 (`697d9d96c`) | Attempt 2 (`9291976115`) |
|---|---|---|
| Release build / D29 self-test / D29 repo | 0 / 0 / 0 | 0 / 0 / 0 |
| Route parity (72 cases) | PASS | PASS |
| Flock Gate parity (108) | PASS | PASS |
| Relation Gate parity (288) | PASS | PASS |
| Refusal parity | **FAIL (instrument)** | PASS (3/3) |
| Forced terminals: status 8 + lane break | PASS | PASS |
| Negative gate: reducer `tr` clause deleted | FAIL as required | FAIL as required |
| Restored source hashes | clean | clean |

- **Attempt 1's refusal FAIL** was in the suite, not the kernel. Gate-on-Route is refused on the host with `UnimplementedParameter`, which fell into a catch-all arm. Attempt 2 compares host refusals of any kind by their text. Both attempts are kept.
- **Negative gate:** with the virtual-`tr` break clause deleted, the forced fixture diverges, ordinary `[1344, MAX, MAX]` against rows `[1344, 0, 0]`. The suite therefore detects a reducer that does not break.
- **Earlier operational blocks:** two launch attempts stopped before building, one on a source-path check and one on a module-path compile error. Neither ran any model work. The compile error is why the CUDA-feature Rust is now type-checked locally with a stub `nvcc` before any pod run.

## Timing (attempt 2; non-gating, not a qualified rate)

The shape is V4096/B4/C192, `kept_cells` 8, 64 full rows. Each sample is one synchronized `propose`; one warmup is excluded, and the median of 5 is kept.

| Read / parameter | Ordinary | Row-parallel | Ratio | Losses |
|---|---|---|---|---|
| Route / VectorBlock(0) | 3.843 s | 0.0691 s | 55.6× | equal |
| Relation{kept 8, sweeps 1} / Gate(0) | 4.144 s | 0.0728 s | 56.9× | equal |

**Derived (not measured):** Dose 1's 16,404 batches per arm come to about 0.073 s × 16,404 ≈ 20 min per arm on the row path, versus about 19 h at the ordinary 4.1 s.

## Evidence and cost

- **In the repo:** `evidence/attempt{1,2}/`, holding the logs, JSON reports, exit codes, source and binary hashes, GPU identity and negative diff. The release `rows-qualify` SHA-256 is `809d7912…a9010`.
- **Full pod archive:** `icloud:UOR-R4/results/claude/vast-56495093.tar` (10,882,048 bytes, MD5 `d7d35d2da896647a81f7ef03b2d3063b`).
- **Cost:** one pod at about $0.91/h for about 20 min, roughly **$0.30**. The pod was released and deleted after archiving.
