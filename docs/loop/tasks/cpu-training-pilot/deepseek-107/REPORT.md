# REPORT: cpu-training-pilot, deepseek-107 (remove the CPU build race)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `6d88bf103`.

## What did not work, first

- **The first minimal-core check asked Cargo for every package target.** Many
  `uor-r4-core` command-line binaries require the full default modules, so
  `cargo check -p uor-r4-core --no-default-features` produced a large,
  unrelated binary failure. The supported minimal configuration is the
  library used by the build dependency: `--no-default-features --lib` passes.
  The ORDER wording was corrected before the head was frozen.
- **The default package-wide all-target surface is also not green at this
  head.** `cargo check -p uor-r4-core --all-targets` reaches many stale core
  binaries, examples and integration tests that refer to removed modules such
  as `semantic`, old `native_geometric::learner` paths, `hex`, `ciborium` and
  `uor_r4_model_source`. These failures predate this unit and are outside its
  build-graph target. The default core **library** compiles and its 75 tests
  pass; the minimal library review is `--no-default-features --lib`.
- **The package-wide formatting command reports pre-existing differences.**
  Current `main` has rustfmt changes in three tokenizer integration tests
  outside this unit. The changed core source passes
  `rustfmt --check` directly. No unrelated test formatting was rewritten.
- **This does not remove Cargo's general duplicate-`cdylib` limitation.** It
  removes the project path that forced the integer crate to build twice in
  one CPU invocation. A future build that deliberately requests two
  feature-distinct integer builds can still encounter upstream Cargo #6313.

## What changed

`uor-r4-core` now has a default-enabled `integer-runtime` feature. Its
`uor-r4-integer` dependency is optional and is activated by that feature.
The full `native_geometric`, `report_output` and `transformerless` modules
remain behind it, while `spiralcore_operator`, `prime_route_attention`,
`zeta_zeros` and `answer_oracle` remain available in the metadata-only build.

`uor-r4-discrete`'s build dependency sets `default-features = false`. Its build
script still executes the exact SpiralCore/prime-route metadata path but no
longer compiles `uor-r4-integer` under a second feature set. Normal core
consumers are unchanged because the feature is on by default.

The code diff is 3 files, `+10/-2`; retained evidence is 2 files, `+47/-0`.
Documentation adds the ORDER, this REPORT, the item entry and the program
entry. `loop pr-check` reports their exact total at the final head. This
remains within the ORDER's finite build-repair scope.

## Measurements

On the Ryzen 9 9950X:

- metadata-only core library: 10 tests passed;
- default full core library: 75 tests passed;
- CPU training library: 163 passed, 4 release benchmarks ignored;
- ordinary all-target `uor-r4-cpu-train` check in one shared target:
  passed, including a second incremental check after the feature rename;
- no duplicate `libuor_r4_integer.rlib`, `.so` or Cargo output-collision
  warning appeared;
- `cargo tree` shows the discrete build-dependency `uor-r4-core` node with no
  `uor-r4-integer` edge;
- strict all-target CPU Clippy with `-D warnings`: passed.

The retained graph and command readings are in
[build-graph.txt](evidence/build-graph.txt).

## Checks run at the head

- `cargo check --locked -p uor-r4-core --no-default-features --lib`: passed.
- `cargo check --locked -p uor-r4-core --lib`: passed.
- The three library test suites and strict CPU Clippy listed above: passed.
- `rustfmt --check crates/uor-r4-core/src/native_support.rs`: passed.
- `git diff --check`: passed.
- Report, evidence, policy, exact-head review, protected merge and fresh-main
  verification run after the report head is frozen.

## Skills used

No available Codex skill applied.

## Still open

- The complete default `uor-r4-core` all-target set remains broken and
  unrepaired because those binaries intentionally require the full API; the
  failure is the pre-existing stale-target condition recorded above.
- Three unrelated core tokenizer tests differ under this machine's rustfmt;
  they remain outside this build-graph unit.
- No model training, recognition, generation or throughput result changed.

## Plain summary

CPU tests sometimes compiled two differently configured copies of the integer
library into one folder, so one could overwrite the other and cause misleading
compile failures. The cause was the discrete crate's build tool loading the
entire core library even though it only needed exact geometry tables.

This unit separates that metadata-only core build from the full default core.
The build tool no longer loads the integer runtime, while ordinary users of the
core still get the same full API. A normal CPU build now succeeds repeatedly
in one target directory without a fresh-target workaround.
