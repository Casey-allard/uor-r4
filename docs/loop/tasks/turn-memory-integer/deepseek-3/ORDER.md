# ORDER: turn-memory-integer, deepseek-3 (served-contract readout)

Lab: deepseek. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392).
Base: `c58b80a7a72b391bdbaf42cfdc2b4dbdab95cdda`.

## Goal

Give `fbs-snapshot` a served-contract readout mode for checkpoints trained by
the real `with_served_surrogate_training` path. The mode must enable the same
Q12 query, Q16 residual, dynamic-i16 activation and projection roundings, use
`SERVED_SNAPS`, and mark the output so a snapped D11 readout cannot be
mistaken for it.

This is the prerequisite for Claude's remaining I-3 layer-by-layer diagnosis.
It does not change training, serving, an artifact byte or a capability bar.

## Cancel probe

At the base, `fbs_float_readout_on` selects D11 through `with_d11_mode` outside
the served bridge. For a checkpoint trained unsnapped, that mode uses
`D11Snaps::ALL`, so the readout would snap gates and transition the model was
not trained with. `fbs-snapshot` has no mode that enters
`with_served_surrogate_training`.

Claude's claude-266 branch is a drafts-only run order at
`57ba5095ce0872bb8818878983b975671e38f1e3`; it touches no code in this unit.

## Files in scope

- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `docs/loop/tasks/turn-memory-integer/deepseek-3/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/turn-memory-integer.md`

## Done when

1. `fbs-snapshot --served-readout MODEL_DIR OUT_JSON READOUT_JSON` writes one
   D11 readout under the complete served surrogate contract.
2. The same flag works with `--dev` and `--hamming-trace`.
3. The readout JSON carries `served_readout: true`; old readouts parse as
   `false` and serialize byte-identically.
4. Host tests pin options parsing, duplicate/conflicting flags, the served
   flags during the readout, and restoration after success and error.
5. Float and integer focused tests, strict Clippy and formatting pass.
6. A CUDA run may remain `NOT_RUN` only if no GPU is available; its exact
   command and POD status are recorded.
7. Exact-head review, PR checks, merge and fresh-main verification pass.

## Stop if

- The mode changes plain, Export or snapped D11 readouts.
- The readout can be confused with a snapped D11 readout.
- The change edits Claude's open branch or modifies a model artifact.
