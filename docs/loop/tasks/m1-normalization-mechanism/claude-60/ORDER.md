# ORDER: m1-normalization-mechanism, claude-60 (wire the `C_unscaled` arm into the GPU read kernel)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `fb6156b46`. Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

Prerequisite 1 of Claude's `NOT_FIT` on the normalization sheet (claude-57, #3034) says the three `NormalizationMode` arms do not run on the GPU. The design note on #2842 (6364724306) found:

- The treatment arm `ScaleMatchedSnapV1` is already what `crates/uor-r4-discrete/src/discrete.cu` computes. `snap(x, n, g)` matches `n·root` at three sites: tree aggregation (`n = 3`), consensus (`n = found`) and the output residual (`n = terms`).
- The `C_unscaled` control, `UnitSnapControlV1`, needs only `n := 1` at those same three sites.

This unit wires `C_unscaled` and nothing else.

1. **Kernel.** One device spec word, the first unused word after `s[11]` (expected `s[12]`; confirm that no kernel reads it).
   - `0` keeps today's `snap_n`.
   - `1` sets `n := 1` at the three snap sites.
   - Any other value, or `1` outside Flock or Relation read mode (Route has no snap site), sets status `10`, alongside the existing spec validation.
2. **Host.** The device binding carries a `NormalizationMode`.
   - `ScaleMatchedSnapV1` is the default and writes `0`.
   - `UnitSnapControlV1` writes `1`.
   - `UnnormalizedControlV1` (`C_raw`, which needs a ℤ⁸ carry and readout) is refused on the GPU path with a clear error. It stays out of scope.
3. **Default identity.** Every existing configuration keeps its serialized form, its digests and its kernel behaviour unchanged.

## Cancel probe (run first)

At the base, is `UnitSnapControlV1` referenced anywhere outside `normalization_mechanism.rs`, or does any kernel already snap with `n = 1` under a mode word? If either is true, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`: the kernel, device plumbing, mode mapping and tests.
- `docs/labs/m1-normalization-mechanism/**`: one README note on the wiring.
- This task folder.

The sheet is not edited. Its Status and prerequisites belong to its author and the next non-author FIT.

## Done when

1. **Host tests pin the mapping.**
   - The default writes `0`, `UnitSnapControlV1` writes `1`, and `UnnormalizedControlV1` is refused.
   - A serialized pre-existing configuration round-trips byte-identical, and so does its digest, if one is pinned.
   - A planted mapping that writes `0` for `UnitSnapControlV1` fails a test (negative gate).
2. **Stub-nvcc check passes.** `cargo check --features cuda --all-targets` passes with a stub nvcc for the crate, and for every detached `docs/labs/*/instrument` that builds the discrete kernel.
3. **Host checks pass.** `cargo test -p uor-r4-discrete --lib` and clippy `-D warnings` on the crate.
4. **A GPU check exists, and is not run here.**
   - A `cuda`-feature test runs one tiny Flock row in both modes against the host reference (`normalize_block_mode`).
   - It runs on one pod only after Casey approves the lease. Until then the REPORT marks it `NOT_RUN`, and the PR stays a draft.
5. **Loop checks pass:** `report --check` and `pr-check --local`.

## Stop and ask if

- A default configuration's serialized form, digest or kernel path would change.
- An embedded kernel source other than `discrete.cu` reads the same spec buffer at the chosen index.
- A file outside scope must change, such as a detached instrument's PTX list.
- Any GPU pod is needed. Ask Casey first.
