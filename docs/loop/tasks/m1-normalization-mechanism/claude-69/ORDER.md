# ORDER: m1-normalization-mechanism, claude-69 (the driver-source pin checked by `--lib`)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `origin/main`.

Casey approved option 1 of the #2028 card (6375843900) in Claude's chat on 2026-10-24, recorded at 6379112828: "Yes: move the driver_source_sha256 pin check into a --lib test (option 1 of your #2028 card) as a Claude unit, so any lab's lib run catches drift".

This item holds it because the drift came from this line's PRs (#3071, #3088, #3089) and will recur at the sheet registration.

## Goal

`training::driver_source_sha256()` hashes 19 driver sources. Its only pin is a test in `prime_zeta.rs`, which compiles only into three binaries (`#[path]`), so `cargo test -p uor-r4-discrete --lib` never checks it. On 2026-10-23 main stayed red for about 4 h because PRs tested with `--lib` (#3099).

This unit:
- moves the pin into a new library module, `src/driver_source_pin.rs`. It is not one of the 19 sources, so updating the pin never moves the hash it pins;
- adds a `#[cfg(test)]` test in that module that compares the hash with the pin, and wires the module from `lib.rs`, which is also not a driver source;
- points the `prime_zeta.rs` bin test at the library constant, so there is one value to update and the bins keep their test counts.

`driver_source_sha256()` must not move in this PR, since none of the 19 files changes.

## Cancel probe (run first)

At the base, does `cargo test -p uor-r4-discrete --lib` run any test that asserts the `driver_source_sha256()` value? If one does, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- This task folder

## Done when

1. **The lib run checks the pin.** `--lib` runs `driver_source_pin::tests::driver_source_sha256_matches_the_pin`, and it passes.
2. **Drift fails it.** A one-character change to the pin makes that test fail, and restoring it passes.
3. **One value.** The `prime_zeta.rs` test reads the library constant, and no 64-hex literal of the hash remains in `prime_zeta.rs`.
4. **The hash is unchanged**, by recomputation at the head.
5. **Checks pass:** every discrete target, clippy, `fmt` on the changed files, D23/D25/D32 at baseline, `loop report --check` and `loop pr-check --local`.

## Stop and ask if

- Placing the pin requires editing any of the 19 driver sources.
- Anything outside `crates/uor-r4-discrete` must change.
