# ORDER: hopf-retained-fibre-test, antigravity-63

Lab: antigravity. Item: [hopf-retained-fibre-test](../../../../roadmap/items/hopf-retained-fibre-test.md). Base: `e6b34e332`.

## Goal

The item's done-when reads:

> A retained-fibre test is run on the integer DEV metrics against a control and recorded.

Because `hopf-retained-fibre-test` (`#2825`) has no pre-registration sheet (`docs/harness/prep/hopf-retained-fibre-test*`), no discrete/integer consumer for `HopfFiberPointQ30` (`crates/uor-r4-discrete` has `0` Hopf references and `crates/uor-r4-integer/src/model.rs:4023-4041` marks `hopf_state` `TELEMETRY-ONLY`), and its item `scope:` (`[docs/harness/prep/hopf-retained-fibre-test*, docs/labs/hopf-retained-fibre-test/**, docs/loop/tasks/hopf-retained-fibre-test/**]`) excludes `crates/uor-r4-discrete/**` and `crates/uor-r4-integer/**`, this unit:
1. Authors and FIT-reviews the one-page pre-registration sheet `docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md` (`CHARTER.md` §4.5, `docs/mechanisms/hopf-observation.md:73-74`) specifying the equal-byte 5-arm test (`H_base_fibre`, `C_base_only`, `C_bank_tag`, `C_fibre_perm`, `C_no_hopf`), the six finite controls (`I-1..I-6`), and the multiplier-free discrete/integer Hopf decomposition.
2. Documents the codebase audit and protocol in `docs/labs/hopf-retained-fibre-test/README.md`.
3. Sets `hopf-retained-fibre-test` (`#2825`) to `status: held` under a fresh-context D68 decision review until consumer crate scope and `#2836` Stage T land.

## Cancel probe (run first)

Ran the repository probe on `hopf-retained-fibre-test` at base `e6b34e332`:
1. `ls docs/harness/prep/hopf-retained-fibre-test* docs/labs/hopf-retained-fibre-test 2>/dev/null` -> no pre-registration sheet or lab directory exists (`instrument`).
2. `command grep -rn -i -e "hopf" -e "fiber" -e "fibre" crates/uor-r4-discrete/` -> `0` matches (`instrument`: no discrete Hopf map or kept-fibre consumer is wired into `crates/uor-r4-discrete`, matching `docs/geometry/hopf-observation.md:28`).
3. `command grep -n -C 3 "TELEMETRY-ONLY" crates/uor-r4-integer/src/model.rs` -> lines `4023-4041` confirm `hopf_state` and `cumulative_holonomy_q30` are `TELEMETRY-ONLY STATE` inside the parked pre-D32 `IntegerModel` bridge (`instrument`).
4. `docs/roadmap/items/hopf-retained-fibre-test.md` line 10 restricts `scope:` to docs/prep paths, excluding all Rust crates (`instrument`).

Classification: **`instrument`**. Staging the FIT-reviewed pre-registration sheet, protocol, and D68 `status: held` transition proceeds inside scope.

## Files in scope

- `docs/harness/prep/hopf-retained-fibre-test*`
- `docs/labs/hopf-retained-fibre-test/**`
- `docs/loop/tasks/hopf-retained-fibre-test/**`
- `docs/roadmap/items/hopf-retained-fibre-test.md` (status line, always allowed)
- `docs/roadmap/README.md` (shared registration file)

## Done when

- `docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md` is written and passes a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2825`.
- `docs/labs/hopf-retained-fibre-test/README.md` documents the exact discrete/integer Hopf decomposition, the 30-base $\times$ 4-fibre icosian / 16-root E8 structure, the six finite controls `I-1..I-6`, the equal-byte 5-arm comparison (`H_base_fibre`, `C_base_only`, `C_bank_tag`, `C_fibre_perm`, `C_no_hopf`), and the execution dependencies.
- `docs/roadmap/items/hopf-retained-fibre-test.md` is set to `status: held` after a fresh-context D68 decision review posted on `#2825`, with `docs/roadmap/README.md` re-rendered and all `uor-harness check` and `uor-harness loop gates --full` checks green.

## Stop and ask if

- Any proposed design requires softmax, squashing, dense matmul, or floating-point operations in the discrete crate (`CHARTER.md` §3.1–§3.6).
- Any change outside the declared scope is needed (`stop and decide` via D68 decision review, never wait).
