# ORDER: m1-normalization-mechanism, claude-63 (check the normalization mode at the five remaining admission gates)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `3df3798e4` (main, after #3071). Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

#3071 (claude-61) added `normalization_mode` to snapshots and `DriverConfig`, and refuses a mismatch at the three training-driver checks. Its REPORT lists five other admission gates that compare `read_initialization` but not the mode, so a control-arm model could pass them:

- `attention_learning.rs:~197` and `~811`;
- `controller.rs:~162`;
- `relation_qualification.rs:~995`;
- `attention_audit_qualification.rs:~1569`.

This unit makes each gate compare the mode exactly where it compares `read_initialization`, with the same refusal style.

## Cancel probe (run first)

At the base, does any of these five gates already compare `normalization_mode`? If all five do, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- This task folder

## Done when

1. **Each gate checks the mode.** Each of the five gates refuses a mode mismatch. For each gate, a test fails if the comparison is removed (negative gate on at least one).
2. **Default identity.** With default modes, every existing test and golden fixture is unchanged.
3. **Checks pass:**
   - `cargo test -p uor-r4-discrete --lib`;
   - the stub-nvcc `cargo check --features cuda --all-targets` (the gates may live in `cuda` code);
   - clippy on touched lines;
   - ratchets at baseline;
   - no `cfg(any())` gate removed;
   - `report --check` and `pr-check --local`.

## Stop and ask if

- A gate's comparison would change any default behaviour.
- A file outside scope must change.
