# uor-r4-discrete

The active M1 geometric learning crate: integer 8D carrier blocks, 64 signed
SpiralCore transport actions, a 16-root carrier orbit, ordered ternary FBS context,
Route/Flock/Relation reads and one shared input/carry/output vector table and writer.
Four blocks give width32. See the [architecture source map](../../docs/architecture.md)
and [current status](../../STATUS.md) for evidence and release scope.

## Read the source

| Source | Responsibility |
|---|---|
| [build.rs](build.rs), [metadata.rs](src/metadata.rs) | Generate and expose exact finite actions, carrier roots and transport tables. |
| [metadata.rs](src/metadata.rs), [discrete.cu](src/discrete.cu) | Ordered context plans; causal reads, shared writer and exact hinge objectives. |
| [device.rs](src/device.rs) | CUDA snapshot, proposals, apply and state boundaries. |
| [rows_source.rs](src/rows_source.rs), [rows_device.rs](src/rows_device.rs) | Opt-in row-parallel exact counterfactual proposals. |
| [training.rs](src/training.rs), [controller.rs](src/controller.rs), [attention_learning.rs](src/attention_learning.rs) | Pinned corpus/schedule/checkpoint lineage and scoped admission controllers. |
| [prime_zeta.rs](src/prime_zeta.rs) | Offline optional codebook construction: C⁴ per block, realification into R⁴ + R⁴, normalization/Q30, permutation and carrier allocation. |
| [main.rs](src/main.rs) | `discrete-stack` command dispatch; CUDA execution requires the `cuda` feature. |

## Build and validation boundaries

The repository pins Rust1.97.1. From its root, source/host checks are:

```sh
cargo check --locked -p uor-r4-discrete --lib
cargo test --locked -p uor-r4-discrete --lib
```

These are intended for metadata and construction tests; the default feature set
has no CUDA model execution. The `cuda` feature requires CUDA12.8-compatible build
and driver tooling. Device qualification, evaluation and learning are GPU-only,
under the admitted runner, exact sheet and applicable scoped release. Commands
above are not permission to launch a dose. The generic training entry remains held;
controller-specific releases are separately bound.

The selected core dependency uses `native_support.rs`, not historical `lib.rs`;
integer selects `native_geometry.rs`. See their [manifests](../uor-r4-core/Cargo.toml)
and [integer manifest](../uor-r4-integer/Cargo.toml). Do not substitute old model CLI examples.

## Evidence to start with

- [D33 pilot](../../docs/labs/a2-pilot-2026-10-14/README.md): 1,640 batches, 236 accepted updates; mixed DEV measurements.
- [Original paired attention dose](../../docs/labs/attention-dose1-strategic-stop-2026-10-18/README.md): owner-stopped before Q1, partials preserved.
- [Rows parity attempt3](../../docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json): 1,085 finite cases, zero updates; proposal timing is non-gating (`qualified=false`).
- [Live M1 issue](https://github.com/UOR-Foundation/uor-r4/issues/2029): successor dispatch and actual outcomes.

D37 permits offline float training without matmul/GEMM or softmax/squashing, while
this discrete implementation stays integer. D11 multiplier-free serving requires
its own export and complete-path qualification. Useful conversation is unqualified.
