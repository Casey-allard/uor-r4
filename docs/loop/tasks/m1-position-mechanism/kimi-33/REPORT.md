# REPORT: m1-position-mechanism, kimi-33

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2843

**Finished by claude, 24 Oct, under the owner's close-out direction.** I merged in main (`af6109613`) and ran `cargo fmt`, which moved the driver pin (`7fa9ec499`). I then fixed one CUDA-only compile error (`418eeecc7`) and ran the GPU checks on a pod after a D67 pre-run review ([#2843 log](https://github.com/UOR-Foundation/uor-r4/issues/2843#issuecomment-6390503839)). Kimi's text below is kept; I updated the lines on CUDA, the pin and the checks.

## What did not work, first

- **Kimi's Mac had no `nvcc`.** An earlier progress note incorrectly said PTX admission had run; only default host builds and the build-script self-test had. On 24 Oct I ran the CUDA checks: first a stub-nvcc type-check on the Mac, then a real-`nvcc` build on an RTX 5090. Results are under Measurements.
- **The stub-nvcc `cargo check --features cuda --all-targets` found a CUDA-only compile error.** `bvgk_device.rs` `scratch_clone` built a `Device` without the new `position_steps` field (E0063). It is fixed in `418eeecc7` by copying the table like the other device buffers.
- **One host test fails on any fresh checkout.** `completion_baseline::…::authentication_publication_is_immutable_and_resume_fails_closed` (from #2484, untouched here) calls a non-recursive `fs::create_dir` under the gitignored `local/` folder. A fresh pod checkout has no `local/`, so under `--features cuda` it failed with "No such file or directory". After `mkdir local` the whole suite passes. This is a test-environment problem, not part of this change, and it is tracked for a separate fix.
- Merging `c27a5ced8` conflicted in the non-CUDA owner-release API and the driver identity assertion. The resolution retains the generic owner wrapper with main's shared registration function and centralized pin. The first compile caught a redundant wrapper; the merged source identity is repinned below.
- Independent DeepSeek architecture review found real defects in the first implementation: an invalid `self` argument in `from_snapshot`, a duplicate position argument that made validation fail open, a wrong `root_cost` API call, a missing CUDA-only `DriverConfig` field, accidental formatting outside scope, ambiguous payload/span anchors, and a gate-mutated query alias. These were repaired with focused controls; a later review found only the driver pin, which was updated after the final source/test text change.
- Existing internal BFS aggregation is unchanged. The new action is only the final block-`b` transport anchored at `last[node]`, ranked against an immutable copy of the block-`b` query. The pre-registration must pin that definition before any experiment; this PR does not run one.
- The position table is frozen configuration, not a learned parameter. `Parameter` has no position variant and choose/apply never writes `position_steps`, so the registered learned `H_pos`/`C_match` experiment is not runnable yet. Identity initialization collapses all three position modes to identity.
- Old binary snapshots are invalidated by the kernel/source identity changes even with `position: None`. Legacy JSON still omits `position`; D11 export explicitly refuses positioned snapshots because serving has no position read.
- Accidental formatting in five out-of-scope files and four unrelated in-crate files was reverted. The shared model-ledger path `.uor-models/native-joint-learning-2026-09-04/model-time.json` was unavailable; local free space was about 32 GiB before the final checks.

## What changed

Optional `PositionConfig { mode, steps }` now flows through `DriverConfig`, `Device::Binding` and `Snapshot`, with constructor and deserialization validation, checkpoint mismatch refusal, and default-omitted JSON. `s[14]` selects legacy BFS (`0`), prefix holonomy (`1`), matched endpoint control (`2`) or identity (`3`). The table reaches ordinary and generated CUDA entries and launchers. Nonzero modes use payload-block roots at `last[node]` and an immutable query; legacy ranking and BFS payload behavior remain on the original frame path. Validation covers every position word. D11 export refuses positioned snapshots. Driver and compiled source identities include the position module.

Harness size: **code 15 files +556/−162; docs 2 files +80/−0; generated 0 files +0/−0; other 1 file +68/−18; tests 0 files +0/−0** (the Rust unit controls live inside source files). There are no dependency, corpus, roadmap-status, hold, bar or generated-document changes. The ORDER gave no numeric size estimate, so a ratio is not applicable.

## Measurements

- `cargo test -p uor-r4-discrete` after merging `5844d911c`: **250 lib passed, 1 ignored; 13 binary passed; 33 integration passed; 0 doc-test failures**.
- Position-focused controls: **22 passed**. These include constructor/deserialization validation, optional snapshot and driver round trips, mismatch refusal, raw-root legacy ranking, immutable payload query, last-position span anchors, one-position-argument snapshot ABI, and D11 export refusal.
- Red-green checks: missing config/snapshot types failed before implementation; positioned D11 export succeeded before the refusal was added; the raw-ranking, query-alias and launch-ABI controls failed on the reviewed defects; removing table revalidation allowed a full 64-entry table with candidate `16` before the guard was restored.
- `driver_source_sha256`: `1a3de704089076196fcdf73c3705d68f13903b373b66b2ba981bf85d3f128455` after merging `5844d911c`. It became `47886ad85662149a24d1ab161ff7023a0427b7221f7e6c46f61be9b9557da320` after the `cargo fmt` commit `7fa9ec499`, which reformatted four driver sources. It is checked by the shared library pin.
- Host suite at `418eeecc7` on the Mac: `cargo test --locked -p uor-r4-discrete --tests`, **289 passed, 0 failed, 1 ignored**.
- **CUDA on an RTX 5090** (Vast `vast-57575147`, compute capability 12.0, nvcc 12.8, `UOR_NVCC_ARCH=compute_120`), at the exact head `418eeecc77a6f356ac46fb3b508670f78f3b5a9a`. The run took about 23 minutes and cost about $0.85; the pod had 2 GPUs by the tool's default, and the run used one.
  - **Build and PTX admission:** `cargo build --release --locked -p uor-r4-discrete --features cuda --bins` passed in 136 s. `build.rs` ran `ptx_admission_self_test`, admitted every PTX inventory, and wrote **6 non-empty ptxas resource reports** (10,068 bytes in all).
  - **Full CUDA test suite:** `cargo test --release --locked -p uor-r4-discrete --features cuda` gives **14 targets, 296 passed, 0 failed, 4 ignored**, with `local/` present. The first run, without `local/`, failed the one test named above.
  - **GPU unit tests:** `--lib unit_snap_device_tests -- --include-ignored` passed all 3 device tests: `unit_snap_device_word_runs_and_refuses`, `raw_control_device_word_runs_unsnapped` and `route_read_off_and_permuted_controls_run_and_refuse`.
  - **Finite device controls:** all seven exit 0. They are `discrete-stack qualify` (17 s), `qualify-flock` (11 s), `qualify-relation` (61 s), `qualify-driver` (11 s), `rows-qualify` (271 s), `rowsk-qualify` (74 s) and `bvgk-qualify` (119 s). They run the legacy, no-position device path through the new `position_steps` ABI on every kernel and on the rows, rowsk and BVG-K launchers.
  - **Where the logs are:** transcripts, resource reports and qualifier reports are saved off the pod in `local/claude/pod-3111-20261024/q3111.tar.gz`, then archived to the iCloud store.
- GPU training or evaluation, position learning, order-effect measurements, mechanism verdicts and owner releases: **NOT_RUN**. No device test runs position modes 1–3 or the device-side refusal; those are covered on the host only.

## Checks run at the head

- `git diff --check`: pass.
- `uor-harness loop gates --quick`: 3/3 GREEN.
- `uor-harness loop gates --full`: 11/11 GREEN.
- `cargo test -p uor-r4-discrete`: all targets above pass.
- Independent DeepSeek architecture reviews: earlier blockers addressed; final review found no remaining compile/ABI/semantic defect beyond the then-stale driver pin, now corrected.
- At `418eeecc7`, run by claude:
  - `cargo fmt -p uor-r4-discrete -- --check`: pass.
  - Stub-nvcc `cargo check --features cuda --all-targets`: pass.
  - `git diff --check origin/main...HEAD`: pass.
  - `loop pr-check --local`: PASS.
  - The real `nvcc` build, PTX admission, ptxas reports, CUDA suite, GPU unit tests and seven device controls: all pass, as listed under Measurements.
  - Positioned device equivalence and device-side malformed-table refusal: **NOT_RUN on the device**. They are covered on the host only.
- Clippy: `--all-features -D warnings` fails on this crate on main as well (#3168), and this PR adds two `too_many_arguments` (`make_binding`, `Device::new_with_position`). The ORDER asks for no clippy pass, and CI does not run native clippy on PRs. Both are recorded in #3168.

## Skills used

`uor-research-architect`, `superpowers:using-superpowers`, `superpowers:brainstorming`, `superpowers:test-driven-development`, `superpowers:systematic-debugging`, and `superpowers:verification-before-completion`.

## Still open

The ORDER's done-when is met at `418eeecc7`: focused host tests, host PTX admission, full checks, this REPORT, `pr-check --local`, and no GPU model work. The other-vendor review is recorded on the PR. Running position modes 1–3 and the device-side refusal on the GPU is next-unit work on #2843. The pre-registration needs the explicit final-transport definition before an experiment; position learning/proposal application and D11 position export are separate later work. #2843 remains `ready`; no GPU run, item completion, hold release or capability claim is made here.

## Plain summary

This change teaches the existing discrete GPU read how to accept an optional table of geometric position steps and use it for prefix, endpoint-only, or identity transport. If no position configuration is supplied, the old frame ranking and BFS transport remain selected. The change also preserves table data in checkpoints and rejects malformed tables and unsupported D11 export. Host tests and loop gates pass, and independent review caught several integration mistakes that were fixed. On 24 Oct the kernel was compiled with NVIDIA's compiler and run on an RTX 5090: the full CUDA suite and seven device controls pass on the unpositioned path. The positioned modes have not yet run on the GPU, and the table cannot learn yet, so this is instrument wiring—not evidence that position helps language or that the registered experiment can run.
