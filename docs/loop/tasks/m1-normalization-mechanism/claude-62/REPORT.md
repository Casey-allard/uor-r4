# REPORT: m1-normalization-mechanism, claude-62 (wire the `C_raw` arm into the GPU read kernel)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `3df3798e4` (main, after #3071). `References #2842`; the item stays `ready`.

## What did not work, first

1. **The GPU test has not run.** The new ignored test `raw_control_device_word_runs_unsnapped` needs one pod, and Casey approves pods. It is offered to run with the A8 identity measurement at `fd409fcdf` (#2842, 6371771003).
2. **The `C_raw` model still cannot be exported.** `export_d11` refuses every non-default mode, because the integer engine has no unsnapped read. That is intended: controls are not shipped.
3. **Merging rotates `source_sha256` again** (`discrete.cu`, `device.rs`). The A8 run is unaffected, because its runbook checks out the pinned commit `fd409fcdf`.
4. **Pre-existing clippy debt is unchanged**, and none of it is in touched lines.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/discrete.cu` | Under `s[12]==2`, `flock_vector` writes the raw residual `u` as the block output and skips only the final `snap(u, terms)`. The tree and consensus snaps keep their default `n`, so each `m_l` is still an orbit root, as in the sheet's §3.3 formula. The trace records `rec[1]=ABSENT`. `discrete_validate` sets status 10 for `s[12]>2`, and for a non-zero `s[12]` outside Flock or Relation (`:264`, `:464`). |
| `crates/uor-r4-discrete/src/normalization_mechanism.rs` | `device_word()` maps `UnnormalizedControlV1` to 2. The mapping test is now `unit_snap_word_maps_unnormalized_control_to_raw`. |
| `crates/uor-r4-discrete/src/training.rs`, `device.rs` | The existing refusal of a non-zero word with a Route read now covers `C_raw` too; only its message changed. The training test checks that `C_raw` validates with Relation and is refused with Route. GPU test (c) is updated, and a new ignored GPU test, `raw_control_device_word_runs_unsnapped`, was added. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Crate tests.** `cargo test -p uor-r4-discrete --lib`: 228 passed, 0 failed, 1 ignored (pre-existing).
- **Negative gate.** With the `UnnormalizedControlV1 => Ok(2)` mapping set to `Err`, `unit_snap_word_maps_unnormalized_control_to_raw` fails. After restoring it, the test passes.
- **Stub-nvcc check.** `cargo check --offline -p uor-r4-discrete --features cuda --all-targets` exits 0, so both GPU tests compile.
- **Ratchets.** `d23` 4,451, `d25` 93 and `d32` 31, each equal to its baseline. No `cfg(any())` gate is removed.
- **Downstream audit.** Every consumer of the read vector accepts a raw ℤ⁸ vector:
  - `address_distance` uses `root_cost` (L1 plus Hamming against the roots), and `token_distance` is L1.
  - No `root_id` call takes the read vector. The one `root_id(u)`, at `rec[3]`, reads `u` before any residual is added.
  - The token-cell export reads the table, not the read vector.
  - All four `read_vector` callers pass `q` only to the writer's two distances.
  - The host trace readers (`flock_qualification.rs`, `relation_qualification.rs`) build default-mode devices.
  - `u` is an `int` bounded by `2·terms` with `terms ≤ stages+1`, so it cannot overflow.

## Checks run at the head

`cargo test -p uor-r4-discrete --lib`, the stub-nvcc `cargo check --features cuda --all-targets`, `cargo clippy -p uor-r4-discrete --all-targets --no-deps` (touched lines clean), `uor-harness ratchet d23|d25|d32`, the `cfg(any())` removal grep, `loop report --check` and `loop pr-check --local`.

## Skills used

- **`scope-guard`:** the change stays in `crates/uor-r4-discrete/**`, and the export refusal is kept.
- **`solution-integrity`:** the arm follows the sheet's literal §3.3 definition, so the consensus roots stay snapped and only the final residual is left raw. The GPU test checks invariants only (the residual bound and a difference from the treatment), with no host re-computation of the read.

## Still open

- The GPU run of `raw_control_device_word_runs_unsnapped`, once Casey approves the pod.
- With `C_unscaled` (#3071) and `C_raw` (this PR) both wired, the sheet's three arms run from one config field. The NOT_FIT's other prerequisites are for the sheet's author and the next non-author FIT.

## Plain summary

The third normalization arm, `C_raw`, now runs on the GPU path: the same read with the final snap left out. All three arms of the registered bar are now selectable from one config field.
