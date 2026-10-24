# ORDER: m1-normalization-mechanism, claude-72 (score the non-inferiority term)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `dc5082f03`.

Casey registered the margin in Claude's chat at 2026-10-24T03:41:41Z (record: #2028 6379430093). His selected answer, verbatim, to "Normalization bar, non-inferior to C_unscaled": "b − c > −F_I (Recommended)".

## Goal

`normalization-dev-paired` (claude-67, #3119) reports `"non_inferiority":"UNSCORED: no margin registered (claude-57 NOT_FIT prerequisite 3)"`. The margin is now registered. This unit scores it next to `beats`, on both populations (`all` and `P_succ`): `non_inferior` iff `b − c > −F_I`, with `F_I = max(24, ⌈2√(b+c)⌉)` as in the DEV contrast.

## Cancel probe (run first)

At the base, does `attention_preflight.rs` still print that `UNSCORED` string, with no margin scored? If not, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/src/attention_preflight.rs`
- `crates/uor-r4-discrete/src/driver_source_pin.rs` (the re-pin)
- this folder

## Done when

1. Each `DevPairedCount` carries `non_inferior`, computed in `dev_paired_close`. The report's `non_inferiority` field states the rule and its source instead of `UNSCORED`.
2. Unit tests pin both sides of the margin, at the 24 floor and above it: `b − c = −F_I` is not non-inferior, and `b − c = −F_I + 1` is. A negative gate shows the new assertions fail with the rule set aside.
3. `driver_source_pin.rs` is re-pinned. Every `uor-r4-discrete` target passes (lib, bins, tests), and a stub-nvcc `cargo check --features cuda --all-targets` is clean.
4. `loop report --check`, `loop pr-check --local` and `git diff --check` pass.

The sheet (its authors quote the margin in §3), the fresh-draw term, every bar, `NOT_FIT` and every hold are unchanged.

## Stop and ask if

- the margin needs a reading other than `b − c > −F_I` on the DEV contrast;
- a file outside scope must change.
