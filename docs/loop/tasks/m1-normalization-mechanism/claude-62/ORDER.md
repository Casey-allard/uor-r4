# ORDER: m1-normalization-mechanism, claude-62 (wire the `C_raw` arm into the GPU read kernel)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `3df3798e4` (main, after #3071). Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

The registered bar on the normalization sheet compares `N_snap` against `C_raw` (`NormalizationMode::UnnormalizedControlV1`). #3071 wired `C_unscaled`, so `C_raw` is the last arm with no GPU path; `device_word()` still refuses it.

The sheet (§3.3) defines `C_raw` as passing "the raw integer residual sum `u + Σ_l g_l m_l ∈ ℤ⁸` directly to `address_distance` and `token_distance` without snapping back to 𝒪₁₆".

1. **Kernel.** Spec word `s[12]` gains value `2`.
   - In `flock_vector`'s residual stage, the final `out = snap(u, terms, g)` is skipped, and the block's output is the raw integer `u` itself.
   - The tree and consensus snaps stay as they are, so the `m_l` remain orbit roots, which is the sheet's formula.
   - The trace records `ABSENT` for the output root under `C_raw`.
   - Validation sets status 10 for `s[12] > 2`, and for a non-zero `s[12]` outside Flock or Relation.
2. **Host.**
   - `NormalizationMode::UnnormalizedControlV1.device_word()` returns `2`.
   - `DriverConfig::validate` admits it with a Flock or Relation read.
   - Snapshot and config carry it through the existing field.
   - `export_d11` keeps refusing every non-default mode.
3. **Downstream.** Every kernel path that consumes the read vector must accept a non-root ℤ⁸ vector under `C_raw`, or refuse it with a clear status. Audit them: `address_distance`, `token_distance`, any `root_id(q)` use, and token-cell export.

## Cancel probe (run first)

At the base, does `device_word()` already return a value for `UnnormalizedControlV1`, or does any kernel skip the output snap under a mode word? If either is true, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-normalization-mechanism/**`
- This task folder

## Done when

1. **Host tests.** `device_word()` maps `ScaleMatchedSnapV1` to 0, `UnitSnapControlV1` to 1 and `UnnormalizedControlV1` to 2. A `C_raw` config validates with Flock and is refused with Route. A planted mapping that returns `Err` for `C_raw` fails a test (negative gate).
2. **Default identity.** The default kernel path is byte-for-byte unchanged in behaviour (`s[12] = 0`), and default snapshots and configs are unchanged; the existing golden tests stay green.
3. **A GPU test, compiled but not run here.** A `cuda`-feature `#[ignore]` test runs one Flock row under `C_raw`. Each output block must equal the integer residual bound the sheet states (`‖x_b‖_∞ ≤ 2·terms`), and on a constructed multi-term row it must differ from the `ScaleMatchedSnapV1` output. It checks invariants only, with no host re-computation of the read.
4. **Checks pass:**
   - `cargo test -p uor-r4-discrete --lib`;
   - the stub-nvcc `cargo check --features cuda --all-targets`;
   - clippy on touched lines;
   - ratchets `d23` / `d25` / `d32` at baseline;
   - no `cfg(any())` gate removed;
   - `report --check` and `pr-check --local`.
5. **GPU work is not done here.** The pod run waits for Casey's approval.

## Stop and ask if

- A downstream kernel path cannot accept a non-root read vector without a design change beyond refusing it.
- Any default behaviour, serialized form or digest other than `source_sha256` would change.
- A file outside scope must change.
- Any GPU pod is needed.
