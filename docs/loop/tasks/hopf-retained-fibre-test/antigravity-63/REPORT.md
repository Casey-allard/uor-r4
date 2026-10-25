# REPORT: hopf-retained-fibre-test, antigravity-63

Lab: antigravity. Item: [hopf-retained-fibre-test](../../../../roadmap/items/hopf-retained-fibre-test.md). Order: [ORDER.md](ORDER.md).

References #2825.

## What did not work

- **No pre-registration sheet, no discrete Hopf consumer, and docs-only item scope (`instrument`)**: Running the cancel probe on `hopf-retained-fibre-test` (`#2825`) at base `e6b34e332` revealed three pre-execution gaps:
  1. No pre-registration sheet existed under `docs/harness/prep/hopf-retained-fibre-test*`, and no lab directory existed under `docs/labs/hopf-retained-fibre-test/`.
  2. `crates/uor-r4-discrete/` has `0` occurrences of `hopf`, `fiber`, or `fibre` (`docs/geometry/hopf-observation.md:28`: *"Hopf fibre tracking is not wired into the current discrete M1 read"*), and `crates/uor-r4-integer/src/model.rs:4023-4041` marks `session.hopf_state = next_s3.hopf_fiber_project()` and `session.cumulative_holonomy_q30` as `TELEMETRY-ONLY STATE (does NOT affect model outputs, logits, or probabilities)` inside the parked pre-D32 `IntegerModel` runtime bridge.
  3. `docs/roadmap/items/hopf-retained-fibre-test.md` line 10 restricts `scope:` to `[docs/harness/prep/hopf-retained-fibre-test*, docs/labs/hopf-retained-fibre-test/**, docs/loop/tasks/hopf-retained-fibre-test/**]`, excluding `crates/uor-r4-discrete/**` and `crates/uor-r4-integer/**`.
  - Per `CHARTER.md` §3.5, §4.2, §4.5, and §5.1 (D68), renting a GPU pod before the discrete/integer consumer is wired and while the item scope is docs-only would waste fleet GPU budget without changing a decision. All in-scope preparation (the FIT-reviewed pre-registration sheet, the exact discrete/integer Hopf specification, and the D68 transition to `status: held`) was therefore completed in this unit.

## What changed

- **Size**: 6 files changed (0 code, 0 test, 5 docs +236/-1, 1 generated +5/-5; total +241/-6 lines against an item without a line estimate; ratio 1.0x against the ORDER plan).
  - [`docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md`](../../../../harness/prep/hopf-retained-fibre-test-2026-10-25.md) (+92 lines): One-page pre-registration sheet (`CHARTER.md` §4.5, `docs/mechanisms/hopf-observation.md:73-74`) specifying the equal-byte 5-arm retained-fibre language test (`H_base_fibre` with `6b` base + `2b` $S^1$ fibre, `C_base_only` with `8b` base + `0b` fibre, `C_bank_tag` with `6b` base + `2b` explicit `pos & 3` tag, `C_fibre_perm` with cyclic fibre derangement, and `C_no_hopf` baseline) across two seeds (`seed = 1, 2`) on the 14,287-row DEV panel (`3,847`-row `P_succ` slice + `10,440`-row complement) and `>= 2,000`-row `fresh_v2` transfer slice, the six finite algebraic/instrument controls (`I-1..I-6`), SHA-256 data pins, cost/stop rule, and the non-author `TEST FITNESS: FIT` stamp (`0fdc6a0a-4c05-43f9-9ee3-c0b210867fb6`, [`#2825` comment `6396126182`](https://github.com/UOR-Foundation/uor-r4/issues/2825#issuecomment-6396126182)).
  - [`docs/labs/hopf-retained-fibre-test/README.md`](../../../../labs/hopf-retained-fibre-test/README.md) (+47 lines): Codebase audit of Hopf implementations across `crates/uor-r4-discrete`, `crates/uor-r4-integer`, and `crates/uor-r4-core`, the exact multiplier-free discrete/integer Hopf decomposition (`HOPF_TABLE` over the 30-base $\times$ 4-fibre icosian / 16-root $E_8$ orbit structure and `UnitS3Q30::hopf_fiber_project`), the equal-byte retained summary contract (`1` byte/block with exact occurrence identity kept separate per `CHARTER.md` §3.12), and the prerequisites to unhold and execute on GPU.
  - [`docs/roadmap/items/hopf-retained-fibre-test.md`](../../../../roadmap/items/hopf-retained-fibre-test.md) (+1/-1 lines): Set `status: held` under the fresh-context D68 decision review posted on [`#2825` comment `6396126182`](https://github.com/UOR-Foundation/uor-r4/issues/2825#issuecomment-6396126182) until `crates/uor-r4-discrete/**` and `crates/uor-r4-integer/**` are added to `scope:` and `#2836` (`m1-bvg-k-stage-t-training`) lands.
  - [`docs/roadmap/README.md`](../../../../roadmap/README.md) (+5/-5 lines): Re-rendered via `cargo run -p xtask -- roadmap render`.
  - [`docs/loop/tasks/hopf-retained-fibre-test/antigravity-63/ORDER.md`](ORDER.md) (+43 lines) and [`docs/loop/tasks/hopf-retained-fibre-test/antigravity-63/REPORT.md`](REPORT.md) (+53 lines): Task order and report.

## Measurements

- **Cancel probe and codebase verification** (macOS host, commit `7a710d85a`, `command grep -rn -i -e "hopf" -e "fiber" -e "fibre" crates/uor-r4-discrete/` and `uor-harness check`):
  - `crates/uor-r4-discrete/` Hopf references: `0` matches.
  - `crates/uor-r4-integer/src/model.rs:4023-4041`: `hopf_state` and `cumulative_holonomy_q30` confirmed `TELEMETRY-ONLY STATE`.
  - `docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md`: `0` `prep.sheet_shape` errors and `0` `prep.fit_before_run` warnings under `uor-harness check`.
  - `cargo run -p xtask -- links --json local/antigravity/links.json`: `0` broken links in `hopf-retained-fibre-test` files.

## Checks run at the head

- `uor-harness loop gates --full` (`PASS`: all 11 gates green)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` (`92 items OK: ready 44, held 6, decision 1, done 41`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- docs-catalog --check` (`1175 records, 20 lines, 0 paths not found`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- links --json local/antigravity/links.json` (`0` broken links in touched files)
- `uor-harness loop report --check --order docs/loop/tasks/hopf-retained-fibre-test/antigravity-63/ORDER.md` (`PASS`)
- `uor-harness loop pr-check --local` (`PASS` on all hard rules)

## Skills used

- `working-discipline`: Verified repository state (`crates/uor-r4-discrete/src/`, `crates/uor-r4-integer/src/math.rs`, `crates/uor-r4-integer/src/model.rs`, `docs/mechanisms/hopf-observation.md`, and `docs/roadmap/items/hopf-retained-fibre-test.md`) before writing the pre-registration sheet and protocol.
- `scope-guard`: Kept all edits strictly inside `hopf-retained-fibre-test`'s declared scope (`docs/harness/prep/hopf-retained-fibre-test*`, `docs/labs/hopf-retained-fibre-test/**`, `docs/loop/tasks/hopf-retained-fibre-test/**`), the item's `status:` line, and the shared `docs/roadmap/README.md` registration file.
- `solution-integrity`: Designed the equal-byte (`1` byte/block) 5-arm comparison (`H_base_fibre` vs `C_base_only`, `C_bank_tag`, `C_fibre_perm`, `C_no_hopf`) and six finite controls (`I-1..I-6`) using exact integer lookup tables (`HOPF_TABLE` / `UnitS3Q30::hopf_fiber_project`), zero softmax/squashing, zero dense matmul, and separate exact occurrence identity (`CHARTER.md` §3.1–§3.6, §3.12).

## Still open

- Adding `crates/uor-r4-discrete/**` and `crates/uor-r4-integer/**` to `hopf-retained-fibre-test`'s `scope:` (once the active M1 path items release the `crates/uor-r4-discrete/**` lock) and running the registered 5-arm GPU test on the Stage T (`#2836`) base checkpoint under `docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md`.

## Plain summary

The Hopf map takes a four-dimensional unit rotation (a point on the 3-sphere $S^3$) and splits it into two parts: a two-dimensional direction on an ordinary sphere ($S^2$, called the "base shadow") and a one-dimensional twist angle around a circle ($S^1$, called the "fibre"). In the 120-element icosian group used by UOR-R4, the 120 states collapse onto 30 base points on the sphere with 4 fibre twists per base point. Currently, the discrete GPU model (`crates/uor-r4-discrete`) does not compute or read this Hopf split at all, and the integer runtime (`crates/uor-r4-integer`) only logs it as diagnostic telemetry without letting it influence token predictions. Because this roadmap item's file scope only covers documentation and pre-registration files, this unit authored and FIT-reviewed the one-page experimental pre-registration sheet (`docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md`) and lab specification (`docs/labs/hopf-retained-fibre-test/README.md`) defining an equal-byte (`1` byte per block) 5-arm test comparing base+fibre (`6` bits base + `2` bits fibre) against base-only (`8` bits base), an explicit non-geometric position bank tag (`6` bits base + `2` bits position tag), and a fibre-permuted null control, and placed `#2825` on `held` under D68 until the discrete crate scope and Stage T base checkpoint (`#2836`) are ready.
