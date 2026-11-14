# ORDER: one-model-end-to-end, codex-5

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent contract: [claude-371](../claude-371/PARITY.md). Base: `e978e53ef`.

## Owner direction

> "please make this your next immediate roadmap, plan it thoroughly, and execute it
> completely autonomously"

## Goal

Run the missing CUDA leg of the native-space parity contract on the local RTX 4090. The
same `UORLUT01` fixture bytes feed CUDA training, the D84 CPU oracle and integer serving.
Raw words must agree for:

1. the forward read vector;
2. admitted memory cells;
3. top-8 and emitted token;
4. all sixteen counterfactual loss words;
5. the chooser's candidate, before/after loss, epoch and sensitivity;
6. table, routes, gates and epoch after an exact apply;
7. stale-proposal refusal at the next apply.

This is finite construction parity. It is not a training run, a capability claim or a
throughput claim.

## Native state contract

- State: `i32` codebook words, `u32` route ports/cells, `i32` gate trits and `u64` epoch.
- Lifetime: one artifact epoch, from fixture load through proposal and exact apply.
- Shared operators: `discrete_forward`, `discrete_counterfactual`, `discrete_choose` and
  `discrete_apply` in AOT CUDA; their CPU oracles in `discrete_route.rs`; serving reads in
  `uor-r4-integer::discrete_d11`.
- Updates: exact discrete Table/Route/Gate moves; no gradients and no float optimizer.
- Serialization: the same four `UORLUT01` table sections plus header epoch.

## Cancel probe

Cancel this unit if:

1. the CUDA feature cannot build for `compute_89` without NVRTC;
2. a fixture cannot be loaded without changing its table, route, gate or epoch words;
3. the comparison can pass without executing the CUDA kernels;
4. any output difference cannot be reduced to a named operator and raw word;
5. the only path forward requires a training run or an artifact-format change.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/discrete_route.rs`, `#[cfg(test)]` additions only
- `crates/uor-r4-discrete/src/device.rs`
- `crates/uor-r4-discrete/src/d11_export.rs`
- `crates/uor-r4-discrete/src/driver_source_pin.rs`
- `crates/uor-r4-discrete/src/lib.rs`
- `crates/uor-r4-discrete/src/position_mechanism.rs`
- `docs/loop/tasks/one-model-end-to-end/codex-5/**`
- `docs/research/notebook/cpu-training.md`, one result row

## Size

About 250 to 500 changed lines, including the executable parity test and records.

## Done when

1. `--features cuda` builds the CPU-training test crate against `uor-r4-discrete/cuda`.
2. A finite-state CUDA constructor accepts the two fixture snapshots after device-side
   validation; the ordinary production loader still requires normal provenance.
3. One test on the RTX 4090 compares all seven word groups above for Route and Flock.
4. The test proves a nonzero proposal and one post-apply epoch advance on at least one
   fixture; no-op-only coverage is not a pass.
5. Reusing an applied proposal is refused with stale-epoch status 3 on both CUDA and CPU.
6. Focused tests, full discrete tests, Clippy, `uor-harness check`, formatting and the
   REPORT check pass.

## Stop

- No training corpus run, pod or paid compute.
- No change to the artifact format, D85, D11 or GPU caps.
- No broad identity/attention kernel rewrite in this unit.
- STOP and report after the CUDA leg is green or the cancel probe fires.
