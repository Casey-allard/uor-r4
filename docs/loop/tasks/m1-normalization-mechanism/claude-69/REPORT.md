# REPORT: m1-normalization-mechanism, claude-69 (the driver-source pin checked by `--lib`)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). The item status is unchanged. This is option 1 of the #2028 card, approved by Casey (6379112828).

## What did not work, first

1. **This still does not catch drift at merge time.** No CI job that runs on pull requests executes the discrete `--lib` tests: `cargo test --workspace --lib` is in `fast-gate`, which is `workflow_dispatch` only, and the `nextest` job is release-QA only. The new test catches drift in any unfiltered local `cargo test -p uor-r4-discrete --lib`, which discrete work runs. A filtered run, such as `--lib preflight`, skips it, as any filtered run would. Closing the merge-time gap needs a gate entry, option 2 of the card, which is the PM's call.
2. **The pin cannot go in a driver source.** `driver_source_sha256()` hashes 19 files, and a pin inside one of them would move its own hash on every update. The new `driver_source_pin.rs` and `lib.rs` are not among them. That is checked: the hash is unchanged, `63b276f7…`.

## Cancel probe

At the base, the only assertion of the value is the test in `prime_zeta.rs`, which compiles only into `address-cells-codebook`, `prime-zeta-codebook` and `prime-zeta-qualify` through `#[path]`. No `--lib` test asserts it. The task stands.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/driver_source_pin.rs` (new) | `pub const DRIVER_SOURCE_SHA256` and the test `driver_source_sha256_matches_the_pin`, whose failure message says what to update. The module doc explains why the file must stay outside the hashed set. |
| `crates/uor-r4-discrete/src/lib.rs` | `pub mod driver_source_pin;` (one line). |
| `crates/uor-r4-discrete/src/prime_zeta.rs` | The bins' test reads `uor_r4_discrete::driver_source_pin::DRIVER_SOURCE_SHA256`, so there is one value to update. The bins keep their test counts. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Every discrete target** (`cargo test --locked -p uor-r4-discrete`) exits 0:
  - lib: 241 passed, 1 ignored (240 plus the new test);
  - bins: `address-cells-codebook` 6, `prime-zeta-codebook` 7, `prime-zeta-qualify` 6;
  - tests: 7, 7 and 6.
- **`--lib driver_source_pin`:** 1 passed.
- **Negative gate, run by me.** With one hex digit of the pin changed (`…a2b7…` → `…a2b8…`), the test FAILED (0/1). With the file restored byte-identical (checked with `cmp`), it passed (1/1).
- **The hash is unchanged.** It was recomputed from the working tree in `include_bytes!` order: `63b276f7ece591d3…`, equal to the pin. None of the 19 driver sources changed.
- **Lints.** The changed files are rustfmt-clean, and clippy reports nothing in the changed lines; the one hit in `lib.rs` is the existing `words` function at line 195, now 196. Ratchets at baseline: D23 4451, D25 93, D32 31.

## Checks run at the head

The measurements above, plus `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Skills used

- **`scope-guard`:** three files in `crates/uor-r4-discrete` and this folder; no driver source edited.
- **`solution-integrity`:** the test fails on a perturbed pin, and the limit of the fix (point 1) is stated, not implied away.

## Still open

- The merge-time check (point 1): option 2 of the card, the PM's call.

## Plain summary

The fingerprint check that turned main red on 23 October now runs in the ordinary library test suite, so a lab testing with `--lib` sees the failure. It is still not run automatically on every PR; that needs a separate change.
