# ORDER: m1-normalization-mechanism, claude-66 (a separate normalization run path on the proxy trunk)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `origin/main`. Casey picked option (a) of the #2842 design call (6376686610) in Claude's chat on 2026-10-23: "build option (a), a separate arm-parameterized normalization run path on the proxy trunk that records the mode and leaves the A8 path locked".

## Goal

The normalization sheet's three arms are `N_snap`, `C_unscaled` and `C_raw`. They are in the GPU kernel (#3071, #3088), but the only trunk entry, `attention`, is the locked A8 path and refuses every non-default mode. That is claude-57 `NOT_FIT` prerequisite 1.

This unit adds a separate entry:
- `normalization` and `normalization-resume <config> <admission> <out>`;
- `normalization-build-identity <config>`;
- `normalization-check-plan` and `normalization-owner-release`. These were added before the work landed: the attention ones are scope-bound, so without them no admission could be built.

The entry admits exactly the A8 proxy-trunk config (Relation8 sweep 1, K8, nearest-address objective, active-leaf init, interleaved read-every-8, A8 pinned table, seed 1 or 2) with any of the three modes. The mode is in the scope and in the run state, and a mismatch on resume refuses.

The sheet is still `NOT_FIT` and being edited, so no sheet hash is pinned in code. The runs refuse "not yet registered" before any CUDA work; registration is a later one-line PR. `driver_source_sha256` moves, so this unit re-pins `prime_zeta.rs` and runs every discrete target (#3099 lesson).

## Cancel probe (run first)

At the base, does any trunk entry run `C_unscaled` or `C_raw`? If one does, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- This task folder

## Done when

1. **The three arms are admitted.** `normalization-build-identity` accepts the three modes on the A8 trunk config and prints a scope naming the mode. It refuses Route, Flock, other seeds, other codebooks and other trunk settings.
2. **Runs refuse until registration.** `normalization` and `normalization-resume` refuse "not yet registered" before CUDA.
3. **The mode is checked on resume.** A mode mismatch on resume or restore refuses.
4. **The A8 path is unchanged.** `attention` still refuses non-default modes, and the A8, parent and injective scopes are unchanged.
5. **Each claim is pinned.** CPU tests pin 1–4, and one fails with its fix set aside.
6. **The pin is recomputed.** `prime_zeta.rs` pins the recomputed `driver_source_sha256`.
7. **Checks pass:** every discrete target, clippy on the changed files, `fmt`, `loop report --check`, `loop pr-check --local`, and the D23/D25/D32 ratchets at baseline.

## Stop and ask if

- The A8 entry's behaviour, a scope string or a schema must change.
- Anything outside `crates/uor-r4-discrete` must change.
- DEV scoring needs a new evaluation path. Name it in the REPORT as the next unit.
