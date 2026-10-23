# ORDER: m1-normalization-mechanism, claude-61 (carry the normalization mode through snapshots, driver configs and export)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `61706a39e`, on PR #3071 after claude-60. Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

claude-60 wired `NormalizationMode::UnitSnapControlV1` (`C_unscaled`) into the GPU read kernel as spec word `s[12]`. Its REPORT says no command can run the arm yet, for two reasons:

- `Device::snapshot()` refuses a non-default mode, because the snapshot schema cannot record it.
- No driver config selects the mode.

This unit closes both gaps. It follows the `read_initialization` pattern exactly, because that is the most recent binding field threaded the same way.

1. **Snapshot.** `Snapshot` gains `#[serde(default, skip_serializing_if = "NormalizationMode::is_scale_matched_snap")] normalization_mode`. `Device::snapshot()` records the mode instead of refusing, and restore rebuilds the device with it (`new_with_normalization`).
2. **Driver config.** The training driver config gains the same default-omitted field. `PreparedDriver` builds its devices with it, and a checkpoint whose mode differs from the config is refused, as `read_initialization` mismatches are (`training.rs:~831-838`, `~2584`, `~2929`).
3. **Export.** `d11_export` refuses a non-default mode with a clear error. The integer serving engine has no unit snap, and controls are not shipped.

## Cancel probe (run first)

At the base, does `Snapshot` or the driver config already carry `normalization_mode`, or does `snapshot()` accept a non-default mode? If any is true, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-normalization-mechanism/**`
- This task folder

## Done when

1. **Default identity.** A default snapshot's JSON and hash domains are byte-identical to the base: the golden test in `snapshot.rs` stays green, and it also asserts that the text has no `normalization_mode`. A default driver config also serializes without the field.
2. **Round trip.** A `UnitSnapControlV1` snapshot serializes the field, and restores to a device whose `normalization_mode()` is `UnitSnapControlV1`.
3. **Driver.** A config with `normalization_mode: unit_snap_control_v1` builds unit-mode devices. A checkpoint whose mode differs from the config is refused. A negative gate: if the mismatch check is set aside, its test fails.
4. **Export.** Exporting a non-default snapshot returns `Err`; default export is unchanged.
5. **Checks pass:**
   - `cargo test -p uor-r4-discrete --lib`;
   - the stub-nvcc `cargo check --features cuda --all-targets`;
   - clippy `-D warnings` on touched lines;
   - ratchets `d23` / `d25` / `d32` at baseline;
   - `report --check` and `pr-check --local`.
6. **GPU work is not done here.** The pod run that verifies both claude-60 and claude-61 (the unit-snap device test and the default-path A8 checkpoint-0 identity check) waits for Casey's approval.

## Stop and ask if

- Any default serialized form or digest would change.
- A file outside `crates/uor-r4-discrete/**` must change.
- Any GPU pod is needed. Ask Casey first.
