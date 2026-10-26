# REPORT: m1-faster-writer, antigravity-90

Lab: antigravity. Order: [ORDER.md](ORDER.md). Decision review: [DECISION.md](DECISION.md) ([owner log on #2840](https://github.com/UOR-Foundation/uor-r4/issues/2840#issuecomment-6413388477), fresh-context `FIT`, `D68`, and `D67` `PASS`). Item: [m1-faster-writer](../../../../roadmap/items/m1-faster-writer.md) (Issue [`#2840`](https://github.com/UOR-Foundation/uor-r4/issues/2840)).

Closes #2840

## What did not work

- **Mechanism / parity negatives (`0`):** None. Across all five registered GPU control groups on both independent `nvcc` builds (`Build A` `64b37774…` and `Build B` `0bc9d5f8…` on `1× NVIDIA GeForce RTX 5090`, `vast-57796936`), the CSR cell-bucketed CUDA writer (`bucket_source.rs`, `writer_bucket_device.rs`, `writer_bucket_qualification.rs`) matched the full-scan reference writer (`discrete.cu` lines `307-345`) with **`0` mismatches** (`12 / 12` `Device` end-to-end parity configs across `forward`, `evaluate`, and `60 / 60` `propose` calls; `72` seeded scale cases / `2,304` rows + `36` cell-changing virtual-move cases across `V ∈ {64, 256, 4096, 16384}`; single-cell collapse, uniform `V = 4,096` and `V = 16,384`, `< K` occupied cells, `10 / 10` `ABSENT` non-root refusals, constructed ties, virtual moves across cells, and a `64`-step GPU incremental update loop; and all `14,287` real-read `ft/dev` rows on both pinned `A8` `6600bdb1…` and pinned `PrimeZeta` `83c1380a…`), while catching **`4 / 4` planted faults**.
- **Pre-run and launch instrument repairs (`classification: instrument`):**
  1. **Pre-flight static audit (`a6fa66ad-16e7-4c7f-9f27-dc43fa7494e2`, committed in `45beb2c16214281db3e999f0f992e38152b0c0e9`):** Before any GPU build, fresh-context review of `bucket_source.rs` and `writer_bucket_qualification.rs` identified five instrument details: (i) an extra `++c_token_dist` increment on the target token's own `td` lookup in `bucket_writer_core` and `fullscan_writer_counted` (which would have reported `token_distance_evals = 129` instead of `128` at uniform `V = 4,096, K = 8`); (ii) `BUCKET_FAULT_STALE_BUCKET` needing to suppress the virtual move inside `bucket_cell_occupied`; (iii) `BUCKET_FAULT_REVERSED_TIE` needing to reverse equal-distance cell admission ties as well as token ties; (iv) `FAULT_OVER_PRUNED` needing exact cell-distance ties (`token 1` in cell `rx`, `token 0` in cell `ry` at equal address and token distance) so `>=` skips cell `ry`; and (v) expanding `run_device_parity_controls` from `6` to all `12` pre-registered configs across both `ObjectivePolicy` modes.
  2. **Pod launch instrument fixes (`b1ab0129fe871a317c4d801c58357bdecbcc2b3f`):** Launch 1 exited in `2 s` (`exit 2`) because `/root/uor-store get` expects a destination directory rather than a file path. Launch 2 compiled both builds cleanly and verified all input SHA-256 pins (`0919e90f…`, `a838bb1a…`, `6600bdb1…`, `83c1380a…`), then exited (`exit 1`) at the start of `run_device_parity_controls` with `invalid discrete model: route parameters are inactive in a flock model` (`device.rs:1942-1944`). Replacing `Parameter::Route(0)` with `Parameter::Gate(1)` in `run_device_parity_controls` and aligning `run_real_read_codebook_panel`'s model constructor to the registered `ReadSpec::Relation { kept: 8, sweeps: 1 }` + `ReadInitialization::ActiveLeafV1` configuration resolved the instrument error on Launch 3 (`exit 0`).

## What changed

- **Ahead-of-time CUDA CSR bucketed writer kernels and engine (`crates/uor-r4-discrete`):**
  - [`crates/uor-r4-discrete/src/bucket_source.rs`](../../../../crates/uor-r4-discrete/src/bucket_source.rs) and [`crates/uor-r4-discrete/build.rs`](../../../../crates/uor-r4-discrete/build.rs): emits `bucket.cu` and compiles `bucket.ptx` (`e53dbb16cdbbf0a3c7d90e4df17db9934141c0b55949fd0a3d15b2f490410a41`, verified `0` float instructions via `admit_ptx_inventory(&bucket_source::ENTRIES)`) and `bucket.cubin` (`22e82fe4eb36e8e6e8f54b8062c40ac6bcea81d55685a7e2ad7e3e427c23c556`) with 7 `.entry` kernels (`discrete_bucket_build`, `discrete_bucket_update`, `discrete_bucket_writer_eval`, `discrete_fullscan_writer_eval`, `discrete_bucket_forward`, `discrete_bucket_counterfactual`, `discrete_bucket_evaluate`). `driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) is untouched.
  - [`crates/uor-r4-discrete/src/writer_bucket_device.rs`](../../../../crates/uor-r4-discrete/src/writer_bucket_device.rs) (wired into [`crates/uor-r4-discrete/src/device.rs`](../../../../crates/uor-r4-discrete/src/device.rs)): implements `BucketWriterEngine`, `BucketIndexDevice`, `BucketIndexSnapshot`, `DirectWriterOutput`, `DirectTripleTimed`, `CounterRow`, `compiled_bucket_image_sha256`, and `bucket_extends_ordinary_source`.
  - [`crates/uor-r4-discrete/src/writer_bucket_qualification.rs`](../../../../crates/uor-r4-discrete/src/writer_bucket_qualification.rs) (wired into [`crates/uor-r4-discrete/src/lib.rs`](../../../../crates/uor-r4-discrete/src/lib.rs)) and [`crates/uor-r4-discrete/src/bin/writer-bucket-qualify.rs`](../../../../crates/uor-r4-discrete/src/bin/writer-bucket-qualify.rs): implements the 5-group GPU qualification suite and CLI binary.
- **Pre-registration, lab report, mechanism page, notebook row, and roadmap item:**
  - [`docs/harness/prep/m1-faster-writer-2026-10-26.md`](../../../harness/prep/m1-faster-writer-2026-10-26.md) and [`docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`](../../../harness/prep/m1-faster-writer-DRAFT-2026-10-22.md): registered the `B1`/`B2` bar and recorded `TEST FITNESS: FIT`, `D68`, and `D67` reviews.
  - [`docs/labs/m1-faster-writer/README.md`](../../../labs/m1-faster-writer/README.md), [`SHA256SUMS`](../../../labs/m1-faster-writer/SHA256SUMS), and [`bucket-resources-build-a.txt`](../../../labs/m1-faster-writer/bucket-resources-build-a.txt): full qualification report and `ptxas` register/stack receipt.
  - [`docs/mechanisms/admission.md`](../../../mechanisms/admission.md), [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md), and [`docs/roadmap/items/m1-faster-writer.md`](../../../roadmap/items/m1-faster-writer.md) (`status: done`).

## Measurements

- **Direct GPU writer throughput (`tokens/s` first) and per-row comparisons (`1× RTX 5090`, `sm_120`, `vast-57796936`):**
  - **Pinned `A8` (`address_structured_v1`, `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, `14,287` `ft/dev` rows at `V = 4,096, B = 4, C = 192, K = 8`, `top1 = 133`, `top8 = 513`):**
    - Full-scan writer (`discrete.cu` lines `307-345`): **`1,790 tokens/s`** (`7,978,314 us`, `39,270` primitive evals/row = `36,865` `token_cell` + `2,277` `cell_distance` + `128` `token_distance`).
    - CSR bucketed writer (`WALK_ID_ORDER`, exact training/loss parity): **`81,533 tokens/s`** (`175,228 us`, **`45.5×` speedup**; **`384` primitive evals/row** = `0` `token_cell` + `256` `cell_distance` + `128` `token_distance`, **`102.26×` comparison reduction**, `0 / 14,287` mismatches).
    - CSR bucketed writer (`WALK_SERVING_PRUNED`, exact serving prediction parity): **`106,648 tokens/s`** (`133,963 us`, **`59.6×` speedup**; **`272` primitive evals/row** = `0` `token_cell` + `256` `cell_distance` + `16` `token_distance`, **`144.37×` comparison reduction**, `0 / 14,287` mismatches).
    - End-to-end `C = 192` `Relation` + writer `forward` / `evaluate`: **`925` / `881 tokens/s`** (full scan) $\to$ **`1,552` / `1,549 tokens/s`** (CSR bucketed, **`1.68×` / `1.76×`** end-to-end speedup, `0 / 14,287` mismatches).
  - **Pinned `PrimeZeta` (`PrimeZeta_v4096_b4_c192_k8`, `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`, `14,287` `ft/dev` rows at `V = 4,096, B = 4, C = 192, K = 8`, `top1 = 38`, `top8 = 95`):**
    - Full-scan writer: **`1,762 tokens/s`** (`8,108,295 us`, `39,270` primitive evals/row).
    - CSR bucketed writer (`WALK_ID_ORDER`): **`78,677 tokens/s`** (`181,590 us`, **`44.7×` speedup**; **`384` primitive evals/row**, **`102.26×` comparison reduction**, `0 / 14,287` mismatches).
    - CSR bucketed writer (`WALK_SERVING_PRUNED`): **`84,832 tokens/s`** (`168,415 us`, **`48.1×` speedup**; **`339` primitive evals/row** = `0` + `256` + `83`, **`115.81×` comparison reduction**, `0 / 14,287` mismatches).
    - End-to-end `C = 192` `Relation` + writer `forward` / `evaluate`: **`924` / `876 tokens/s`** (full scan) $\to$ **`1,563` / `1,559 tokens/s`** (CSR bucketed, **`1.69×` / `1.78×`** end-to-end speedup, `0 / 14,287` mismatches).
  - **Larger vocabulary scale (`V = 16,384, B = 4, K = 8`, `96` seeded rows — `#2887` scale):**
    - Full-scan writer: **`219 tokens/s`** (`150,243` primitive evals/row = `147,457` `token_cell` + `2,277` `cell_distance` + `509` `token_distance`).
    - CSR bucketed writer (`WALK_ID_ORDER`): **`22,977 tokens/s`** (**`104.9×` speedup**; **`765` primitive evals/row** = `0` + `256` + `509`, **`196.15×` comparison reduction**, `0` mismatches).
    - CSR bucketed writer (`WALK_SERVING_PRUNED`): **`26,936 tokens/s`** (**`123.0×` speedup**; **`592` primitive evals/row** = `0` + `256` + `336`, **`253.50×` comparison reduction**, `0` mismatches).
    - Uniform `V = 16,384, B = 4, K = 8` (`64` tokens/cell): `150,246` $\to$ **`768` primitive evals/row** (`0` + `256` + `512`, **`195.63×` comparison reduction**, `0` mismatches).
  - **Counterfactual proposal speedup (`Device::propose` vs `BucketWriterEngine::propose`, `5` parameters × `16` rows at `V = 4,096, B = 4, C = 8`):**
    - `K = 8`: `4,766,257 us` $\to$ `218,513 us` (**`21.8×` speedup**, `0 / 5` mismatches).
    - `K = 16`: `9,014,276 us` $\to$ `422,170 us` (**`21.4×` speedup**, `0 / 5` mismatches).
  - **`ptxas` register and spill footprint (`sm_120`):** `discrete_fullscan_writer_eval` uses `255` registers (`4,912 B` stack, `344 B` spill stores, `180 B` spill loads); `discrete_bucket_writer_eval` uses `64` registers (`2,816 B` stack, **`0 B` spills**).
  - **GPU cost & archival:** `1× RTX 5090` (`vast-57796936`), `362 s` active job runtime (`21.3 min` total pod lease at `$0.908/h` = `~$0.32`), archived in `caseyallard/uor-r4-store:results/antigravity/m1-faster-writer-2026-10-26/` (`qualification-build-a.json` `37419bdc303b79ac4beac0fab0ea292d74dc34ac9623754457e9107d89a40fd0`, `qualification-build-b.json` `df583e45e1ca857cb698faf23c5a266fdc6abd3073be5f897662e432c82262bb`).

## Checks run at the head

- **Host test suites:** `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-discrete` (`0` failures across all library, binary, and integration test targets, including `writer_bucket_controls` `6 / 6` and `bucket_source::tests` `4 / 4`).
- **CUDA stub-`nvcc` target check:** `NVCC=.../local/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check --offline --features cuda --all-targets -p uor-r4-discrete` (`0` errors).
- **Formatting & lints:** `~/.cargo/bin/cargo fmt -p uor-r4-discrete --check` (`0` diffs).
- **Harness & loop gates:** `uor-harness check` (`0` errors), `uor-harness loop report --check --order docs/loop/tasks/m1-faster-writer/antigravity-90/ORDER.md`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full`.

## Skills used

- **uor-research-architect:** Used to design the ahead-of-time `bucket.cu` CUDA module (`bucket_source.rs`) as a prefix-verified extension of `discrete.cu` (`bucket_extends_ordinary_source`) so `driver_source_sha256` remains unchanged while sharing the exact `model_view` / `read_row_context` / `writer` integer semantics.
- **solution-integrity:** Used during pre-flight audit and launch verification to ensure every counter (`token_cell`, `cell_distance`, `token_distance`) and planted fault (`FAULT_STALE_BUCKET`, `FAULT_REVERSED_TIE`, `FAULT_OVER_PRUNED`, `FAULT_CORRUPTED_CSR`) tests the exact invariant declared in the pre-registration sheet.
- **scope-guard:** Verified all touched paths stay strictly inside the ORDER's declared scope (`crates/uor-r4-discrete/**`, `docs/harness/prep/m1-faster-writer*`, `docs/mechanisms/admission.md`, `docs/labs/m1-faster-writer/**`, `docs/loop/tasks/m1-faster-writer/**`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-faster-writer.md`).

## Still open

- **`m1-faster-writer` (`#2840`) is complete (`status: done`).**
- **Downstream consumer unlocked:** [`r-vocab-4096-vs-16384`](../../../../roadmap/items/r-vocab-4096-vs-16384.md) (`#2887`), which depends on `m1-faster-writer`, is now unblocked to compare `V = 4,096` against `V = 16,384` using the qualified CSR bucketed writer (`196.15×`–`253.50×` comparison reduction at `V = 16,384, B = 4, K = 8`).

## Plain summary

Previously, every time the model picked a word, its output step scanned the entire 4,096-word vocabulary to find which words lived in the 8 selected address drawers—doing 39,270 distance and cell lookups per word (and 150,243 lookups per word at a 16,384-word vocabulary). We built and GPU-qualified an exact 256-drawer lookup table (`bucket.cu`) that jumps straight to the words inside the 8 selected drawers without scanning the rest of the vocabulary. Across two independent builds on an RTX 5090, the new writer gives the exact same answers and training losses on every single test (`0` mismatches across all `14,287` development rows on both `A8` and `PrimeZeta` and across vocabularies up to `16,384`), cuts work per word by `102×` to `144×` at 4,096 words (`45.5×` to `59.6×` faster direct writer throughput, `21.8×` faster counterfactual proposals) and by `196×` to `253×` at 16,384 words (`104.9×` to `123.0×` faster), and closes `#2840`.
