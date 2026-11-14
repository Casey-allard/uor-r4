# REPORT: one-model-end-to-end, codex-5

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Order: [ORDER.md](ORDER.md). Result: [PARITY-RESULT.md](PARITY-RESULT.md).

## What did not work, first

- The production snapshot and container loaders correctly refused Claude-371's synthetic
  fixture provenance. Finite tests now use explicit qualification-only loaders while production
  loading still requires normal hashes.
- The serving loader still accepts a Route artifact with non-zero gates while CPU training
  refuses it.
- Serving still exposes neither the chosen route port nor the Flock kept list as direct words.
- Neither path has generation stop state.

## What changed

- Forwarded a `cuda` feature from `uor-r4-cpu-train` to `uor-r4-discrete`.
- Added explicit finite qualification constructors for the discrete artifact and device state.
  They validate the container, tables, roots, cells and CUDA device inventory; they bypass only
  the synthetic provenance hash.
- Exposed the existing exact apply path to the finite contract test without making it a public
  training authority.
- Added one CUDA/CPU/serving parity test covering both Claude-371 fixtures, all 39 prefixes,
  forward carry, addresses, top-8, token, 16 counterfactual losses, chooser words, apply and
  stale reuse.

Size: about 330 added/deleted lines across the implementation, test and task records.

## Measurements

Device: RTX 4090, compute capability 8.9, local CUDA 13.1, AOT `compute_89`.

Route fixture:

- 39 rows and 624 carry words were exact across CUDA, CPU and serving.
- all 16 loss words and chooser words matched.
- candidate 7 improved and applied; epoch advanced `0 -> 1`.
- post-apply table, routes, gates and epoch matched.

Flock fixture:

- 39 rows and 624 carry words were exact across CUDA, CPU and serving.
- all 16 loss words and chooser words matched.
- candidate 1 improved and applied; epoch advanced `0 -> 1`.
- post-apply table, routes, gates and epoch matched.

Both fixtures detected a one-word planted carry difference. Reusing an applied proposal was
refused by CUDA and by the CPU applier (status 3). Exact loss rows are in
[PARITY-RESULT.md](PARITY-RESULT.md).

No training run, model capability, throughput or language result is claimed.

## Checks run at the head

- `cargo fmt --package uor-r4-cpu-train --package uor-r4-discrete -- --check` - passed.
- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1
  CARGO_TARGET_DIR=target/cuda-parity cargo test --locked -p uor-r4-cpu-train
  --features cuda,qualification
  native_space_cuda_parity_contract_runs_the_same_words --lib -- --nocapture` - passed on the
  RTX 4090.
- Existing Claude-371 CPU/serving parity tests - to be rerun after rebasing/merge if its PR moves.
- Clippy, full crate tests, formatting, `uor-harness check` and REPORT check - final results are
  listed in the exact-head review record.

## Skills used

- `uor-research-architect`: kept the unit finite and artifact-bound.
- `working-discipline`: used the same bytes on all backends and retained printed rows.
- `solution-integrity`: compared the shipped CUDA paths, not copied arithmetic, and added a
  planted word fault plus stale-reuse control.
- `native-space-cycle`: tied carry, addresses, emitted ids, update state and serialization to
  raw words.

## Still open

- Direct route-port and kept-list traces (step 1 of [ROADMAP.md](ROADMAP.md)).
- Serving loader symmetry for malformed Route gates.
- One-source typed operators over CUDA, CPU and serving.
- Exact Table/Route/Gate status parity beyond the current apply/stale check.
- Generation stop state, copy and turn memory.
- This branch is stacked on Claude-371's open PR #3962; it cannot be a standalone PR until that
  parent is merged or the review base is changed explicitly.

## Plain summary

The actual integer model now runs the same two small artifacts through the RTX 4090 training
kernels, the CPU training oracle and the integer serving engine. All 1,248 carry words, every
address, every top-8 and token, all 32 counterfactual losses, both chooser decisions and both
post-update states agree. That closes the missing CUDA leg of Claude's parity contract. It does
not make the model conversational: direct route/read traces, a generation stop and native
memory are still missing.
