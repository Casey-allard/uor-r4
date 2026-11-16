# ORDER: one-model-end-to-end, claude-404 (one native training step of the whole discrete model on the D87 student stream, CUDA against the CPU oracle)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `a454a250a` (D87's corpus sealed, PR #4028; the ingest instrument, PR #4037).

## Goal

**Take two committed training steps of the whole discrete model (the 4,096-id M1 model family) on a real batch of the D87 student stream on `wick`'s RTX 4090. At each step, compare the CUDA kernels with the CPU training oracle word for word:**
- read vectors;
- the sixteen counterfactual loss words per candidate;
- the choice;
- the artifact's data section after each commit.

This is a finite kernel test. It makes no training run, quality claim or speed claim.

**Why now.** Casey asked to resume training readiness (16 Nov). The reasons:
- claude-403 made the student stream: UORT tokens, a binary response mask and BOS at 0, the format `uor-r4-discrete`'s `PreparedCorpus` already admits, with the same tokenizer (`d36d3e87…0f89`).
- claude-372 showed CUDA and CPU agree on one step, but only at vocabulary 3, context 3, 39 rows. Its REPORT says "Agreement here does not show agreement at corpus scale" and names a multi-step sequence as open.
- Until the two agree at the real model's size on the real stream, a training run on that stream is not known to make the updates the CPU (and through claude-371's leg, serving) compute.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a CUDA-to-CPU-oracle comparison of a discrete step at vocabulary 4,096 on corpus windows, or any multi-step comparison.
- **Result (at base):** none. `cpu_oracle_parity.rs` has the vocabulary-3 fixtures only. The driver's own receipts (`StepReceipt`) compare CUDA with nothing.

## Native-space items (`native-space-cycle`)

1. **State schema:** the discrete `Snapshot`:
   - spec;
   - the i32 table (vocabulary × blocks × 8 words);
   - routes;
   - gates;
   - epoch;
   - seed.

   It is carried across steps on the device and saved as `UORLUT01` by `export_d11`.
2. **Shared operators:** CUDA `discrete_forward`, `discrete_counterfactual`, `discrete_choose` and `discrete_apply` (`discrete.cu`), against `uor-r4-cpu-train`'s scorer, chooser and applier (D84, test-only here). Serving reads the same `UORLUT01` (claude-371's leg).
3. **Replay oracle:** raw `i32`/`u64` words, both sides loading the same bytes; no tolerance.
4. **Forward, gradient and update:** the "gradient" is the exact integer counterfactual loss of each legal move, and the update is the chosen move, committed only on a strict gain. Nothing leaves the integer state.
5. **Serialization:** `UORLUT01`, lossless; the data section is compared byte for byte after each commit.

## Design

- **Stream:** `student-ingest` (claude-403) over two sealed D87 API shards, run on `wick` inside Claude's container. Every input file's sha256 is recorded in the REPORT.
- **Model:** the M1 driver's spec, vocabulary 4,096, 4 blocks, context 192, 8 kept cells, `NearestAddressMarginV1`, seed 1. Two reads:
  - Route, from `zero_gates_v1`;
  - Flock K = 8, from `active_leaf_v1` (claude-372 note 3: the CPU Flock oracle has no `zero_gates_v1`).
- **Batch:** `PreparedCorpus::batch(first, 192, 64)`: 64 response-masked targets with their within-document windows.
- **Parameters scored:** every route and gate, and the vector blocks of the batch's distinct target tokens (all 16,384 blocks would make the CPU side hours long). The list is fixed before scoring; it is not chosen by loss.
- **Two steps:** at each step:
  - forward;
  - proposals and choices for the listed parameters;
  - apply the first committing parameter;
  - compare the bytes.

  Step 2 starts from step 1's CUDA export on both sides, so it is the multi-step case claude-372 left open.
- **Place:** a new `#[ignore]` test in `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`, reusing its `compare_*` functions. It reads the stream folder from `UOR_STUDENT_STREAM` and fails loudly if that is unset or the files are absent. The two existing tests are unchanged.
- **Not a vacuous check:** at real scale, a target moved on the CPU side only must be caught at the losses step, as in claude-372's planted faults.
- **Times:** the CUDA forward, propose and apply times are recorded as diagnostics, not as a speed claim.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-404/**`
- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`: one new test and the small helper changes it needs.
- The discrete line's notebook, one row.

## Done when

1. On `wick` (container `uor-r4-claude`, `UOR_NVCC_ARCH=compute_89`), the test runs at vocabulary 4,096 on the student batch for both reads and two steps. Either every compared word is equal, or each difference is reported with its step, parameter and row.
2. The planted target shift is caught at the losses step and nowhere else.
3. A REPORT, negatives first, with the backend identities (GPU, driver, nvcc, kernel sha256), the stream's hashes, the word counts per step, and the next unit: a pre-registered timed run.

## Stop list

- No training run, pod or rented GPU. `wick` only, inside Claude's own container. No other lab's process or file is touched; the corpus pool is read only.
- No change to any kernel, serving code, export, artifact format, driver capability or public training API.
- No harness, loop-kit or bookkeeping change.
