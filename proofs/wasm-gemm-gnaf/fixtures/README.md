# fixtures/

SPEC §5 reserves `wasm-spec-tests/`, `gemm-cases/`, and `mutations/`.

`wasm-spec-tests/` is populated from the pinned vendored spec tree, which is not
yet vendored. `gemm-cases/` and `mutations/` are populated alongside the layers
they exercise. Empty until then, rather than filled with fixtures no checker reads.
