# REPORT: m4-lawful-likelihood-bpb, deepseek-17 (implementation unit)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `3120b01c5`. References #2861; item stays `ready` (registration unit open).

## What did not work, first

- **The crate's historical integration-test targets already fail to compile on `main`**, because the D29 selected root excludes the historical model/stack/sampler executors while the old test targets still import them. This unit therefore proves itself with the lib unit tests; it does not claim the historical targets were ever green, and the ORDER says so.
- **A stale build artifact briefly hid the new tests.** After adding the module, one `cargo test --lib` run listed 0 of the nine new tests and ten pre-existing tests failed with fixture `NotFound`, while the same baseline worktree passed. Touching the sources forced a rebuild and the suite came back as 42 lib tests, all passing; the fixture file was present the whole time. Reported here because a shared target dir plus a partial earlier build can produce exactly that confusion, and the fix was a deterministic rebuild, not a code change.
- **The crate root is `src/native_geometry.rs`, not `lib.rs`** (Cargo.toml selects it under D29), so the module is wired there under the same `#[cfg(not(feature = "serving-wasm"))]` gate as its neighbours. A first edit that only named the module in `lib.rs` would have been invisible.
- **Two error paths are deliberate refusals, not gaps.** A zero-count MLE target has mass zero and its bits are infinite, so `bits_q12(0)` returns `MassZero` rather than printing infinity; and a vocabulary too large for base-1 masses inside `2^48` is refused. Both are tested.
- **This is the host-only core, not a registered instrument.** No model is run, no artifact is read, no held-out stream is named, no BPB number is claimed, and the item stays `ready` for its registration unit. The base-1 floor is also a chosen distribution, not a calibrated one.
- **Clippy on the lib still reports pre-existing warnings** (`manual_is_multiple_of` and others in old modules); the new module adds none, and the lib compiles clean.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-integer/src/lawful_likelihood.rs` | code | +322 | the host-only PRLL core: base-1 Q48 masses with u128 ratio and largest-remainder ties, MLE baseline with zero-count refusal, integer fixed-point log2 with the stated bound, exact BPB fractions, and nine finite tests |
| `crates/uor-r4-integer/src/native_geometry.rs` | code | +2 | wire the module into the selected crate root under the existing feature gate |
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +14 | implementation section: module path, three APIs, the nine controls, and the host-only boundary |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-17/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** about 340 changed lines, against the ORDER's estimate of about 300 (1.1); the surplus is the tests.

## Measurements

- **`cargo test -p uor-r4-integer --lib`:** 42 passed, 0 failed (33 pre-existing + 9 new).
- **Worst case:** V=4096, one score at `i64::MIN`, the rest at `i64::MAX`; the minimum's mass is exactly 1 and its bits exactly `48 * 4096` (48 bits), asserted in the test.
- **Uniform:** V=4096 gives exactly `2^48 / 4096` per candidate; V=256 counts give exactly 8 bits per token.
- **Ratchets unchanged:** d23 `4451 remaining / 165 files`, d25 `93 / 26 files` — no softmax form and no dense product added.
- **`cargo fmt -p uor-r4-integer --check`:** clean; clippy lib: no new warning for the module.

## Checks run at the head

- `cargo test -q -p uor-r4-integer --lib`: 42 passed.
- `cargo fmt -p uor-r4-integer --check`: clean.
- `cargo clippy -q -p uor-r4-integer --lib`: only pre-existing warnings, none in the new module.
- `uor-harness ratchet d23` / `d25`: OK, baseline totals unchanged.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates; `report --check` and `pr-check --local` PASS.

## Skills used

- **solution-integrity**: the stale-build confusion and the pre-existing broken targets were treated as measurement problems first (rebuild, compare with baseline) before any code story.
- **scope-guard**: one module, one wiring line, one docs section, all inside the item's existing scope.

## Still open

- **Registration unit:** name and hash the held-out chat stream, write the one-page pre-registration, get a non-author FIT review, then the registered GPU scoring under an owner release. The item stays `ready`.
- **Calibration and the family choice** remain the PM/owner's (the owner's Card 1 answer makes PRLL lawful; the 0.90 bar reading is separate).
- **Integration with `sampling`/`eval-corpus`** is deliberately not done here; the module is the pure `(scores -> masses -> bits)` core the registration unit can call.

## Plain summary

- **The design is now code.** A pure integer function turns the model's scores into whole-number probabilities that sum exactly to `2^48`, every candidate at least 1, with no exponential, no squashing and no matrix product; a second function scores a count-table baseline; a third turns masses into integer bits and exact bits-per-byte fractions.
- **The seven finite checks the design promised all run and pass**, including the case that would have been fatal in the first draft (a token with zero probability and infinite bits): the worst-case minimum now has mass exactly 1 and 48 finite bits.
- **Nothing was run on a model and no score is claimed.** The item remains `ready`; the next unit names the held-out stream, registers the instrument and runs it under an owner release.