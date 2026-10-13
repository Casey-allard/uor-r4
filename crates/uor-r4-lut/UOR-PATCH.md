# D29 metadata-only boundary

The compiled crate retains artifact formats, GROUP32/retained exponent constants
and exact integer grid/arcosh metadata helpers. It contains no execution engine,
sampling, normalized-exponential table operation, SIMD dense projection or
historical numeric runtime kernel. No feature can activate historical modules.
The historical `lut-chat` target is no longer registered (`autobins=false`).

Original source, manifest and README are preserved under `docs/history/d29-excluded-lut-2026-10-13/`, with
SHA256 checksums for original Rust source and manifest. Moving source out of the
active module tree preserves it without placing prohibited numeric bodies behind
runtime guards or replacement stubs. Historical source remains inactive evidence.

`format.rs` retains the existing schemas, container identity, shape checks and
read/write implementation; only its introductory serving link is clarified.
`kernels.rs` now exposes only `grid_encode`, `arcosh_grid`, `arcosh1p_q24` and the
unchanged arcosh grid constants. Their existing integer formulas are retained;
this does not qualify a model execution or a D11 serving path. External callers
of removed engines must be excluded by the owning training integration cut.

The direct SIMD, rayon and tokenizer dependencies are removed from this facade.
Production compilation, source-interface integration and finite CUDA controls
are owned by the controller. No model compute/training or headline change is
claimed by this source-only exclusion. Global and M2 holds remain HELD.
