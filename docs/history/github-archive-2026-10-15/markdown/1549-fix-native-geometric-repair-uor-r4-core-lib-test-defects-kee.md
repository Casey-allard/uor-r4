# #1549 — fix(native_geometric): repair uor-r4-core lib test defects; keep workspace artifacts out of the revision (#1546)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T16:41:42Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-30T17:39:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1549
- **Merged:** 2026-09-30T17:39:40Z
- **Merge commit:** a7b0779d2314837333ae94d2f56fabed878cb638
- **Base / Head:** main / UOR-Foundation:lab/opencode/1546-core-test-fixes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**#1546 — repair `uor-r4-core` lib tests on `main`: fix the two deterministic defects and the platform-dependent workspace-test artifact; leave the four optimizer-threshold failures unfixed with root-cause evidence.**

Baseline (Linux, main `7b11e1d8`): `cargo test -p uor-r4-core --lib` → **1093 passed / 7 failed / 60 ignored**. This PR → **1098 passed / 4 failed / 60 ignored** (+3 repaired, +2 new regression tests).

## Fixed (3 of 7)

1. **`learner::lowbit_core::tests::explore_state_growth` — `i32` subtract overflow panic at `learner/lowbit.rs:171`.**
   `TernaryLinear::forward_i32` used checked `+=`/`-=` on an `i32` accumulator; the exploratory un-decayed run legitimately crosses the `i32` boundary, so a debug build panicked. The fixed-width multiplier-free kernel's add/subtract wraps — panic is the wrong contract. Fix: explicit `wrapping_add`/`wrapping_sub`/`wrapping_shl` with the contract documented; `LowBitCore::step` uses the same wrapping combine. In-range results are bit-identical (all exactness tests still pass). Regression tests: `accumulator_overflow_wraps_instead_of_panicking`, `un_decayed_growth_wraps_without_panicking` (both panic before the fix).

2. **`workspace_coding_tests::{native_workspace_iterative_compiler_repair, native_workspace_provenance_and_revision_binding}` — `Workspace file byte limit exceeded`.**
   Both tests wrote the compiled `rustc` output **inside** the workspace; `compute_revision` intentionally rejects tracked files above `MAX_WORKSPACE_FILE_BYTES = 1 MiB`. A minimal `rustc` binary is ~468 KB on macOS (tests passed) but **> 3.8 MB on Linux ELF** (tests failed). This is a test defect, not a product bug — the 1 MiB limit is asserted intentionally by `workspace_rejects_absolute_paths_and_oversized_files` and is unchanged. Fix: the tests place the compiled binary in the system temp dir; both now assert `final_revision.file_count == 1` so a build artifact can never silently re-enter the revision.

Verification (macOS, `CARGO_TARGET_DIR=$HOME/.cache/uor-r4-opencode-target`):
- `cargo fmt --check` → exit 0.
- Focused: both regression tests **ok**; `explore_state_growth` **ok**; `workspace_coding_tests` **11 passed / 0 failed**.

## Not fixed (4 of 7) — a deliberate optimizer change, needing an owner/council decision

`lowbit_attention::{learns_induction_at_short_delay, learns_induction_at_long_delay, normalisation_moves_off_the_uniform_collapse}` and `geometric_attention::corruption_in_the_objective_changes_the_filter`.

- They are **not** platform/seed/precision-sensitive: byte-identical failures on macOS and Linux, deterministic across rebuilds.
- They share **one cause**: PR `a3c54035` (**#1298**, "honest Adam") removed the `if v_hat > 1e-12` guard in `lowbit_core::adam_update`. Restoring that guard locally makes **all four pass** (`normalisation: 0.06 -> 0.38`); the module had no other dependency change.
- **Restoring the guard breaks #1298's own test** `adam_matches_an_independent_reference_including_small_gradients` (which asserts a `1e-7` gradient must move a parameter). The two expectations are mutually exclusive under one `adam_update`.
- Making the tests "robust but not weaker" is impossible here: they already pass under the same `f32` arithmetic when the guard is present, so this is optimizer semantics, not rounding.
- **No threshold was changed.** Ask: the owner/council either (a) give the low-bit trainers an explicit stabilizing gate and re-verify the Adam reference test, or (b) re-freeze the four tests' configured claims for the post-#1298 optimizer. The module docs in `lowbit_attention.rs` (accuracy `1.00`, escape from collapse) should not be read as current until then.

## Evidence

- Branch `lab/opencode/1546-core-test-fixes`, base `18e4bd89`; commits `7839fe2c`, `513026e7`; working tree clean.
- Full local `cargo test -p uor-r4-core --lib --offline` → 1098 passed / 4 failed / 60 ignored (742 s).
- `product_path` CI on the branch head provides the Linux confirmation for the workspace fix.

References #1546.


## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 5915615659 — 2026-09-30T16:42:22Z — Casey-allard

**Review requested — independent APPROVE at exact head `513026e7`.**

Scope for the reviewer (non-author source review):
- `learner/lowbit.rs` + `learner/lowbit_core.rs`: the checked `i32` accumulation in `TernaryLinear::forward_i32`/`LowBitCore::step` is replaced by an explicit **wrapping** contract (the served kernel's fixed-width add/subtract), with two regression tests that panic on the base commit. Confirm the contract is the right one for this kernel and that in-range behavior is unchanged (the exactness tests still pass).
- `workspace_coding_tests.rs` (+ a doc note in `workspace_coding.rs`): compiled test binaries move outside the tracked workspace; both tests now assert `final_revision.file_count == 1`. The product limit (`MAX_WORKSPACE_FILE_BYTES = 1 MiB`) is **unchanged** — this is a Linux-only test artifact (ELF `rustc` > 3.8 MB vs ~468 KB on macOS).
- The four `lowbit_attention`/`geometric_attention` threshold failures are **not touched** here; they are escalated on #1546 with root-cause evidence (#1298's Adam guard removal vs its own reference test) because no threshold may be quietly relaxed.

Linux confirmation (workspace fix + regressions) is running: [run 36746085796](https://github.com/UOR-Foundation/uor-r4/actions/runs/36746085796). Approvals at the head plus that run satisfy the owner's merge rule.

### comment 5916159114 — 2026-09-30T17:17:44Z — Casey-allard

**Linux confirmation — `product_path` run [36746085796](https://github.com/UOR-Foundation/uor-r4/actions/runs/36746085796) at head `513026e7`.**

- `cargo fmt --check`: **PASS** · `cargo check --workspace --all-targets`: **PASS**.
- `cargo test --workspace --lib`: **1098 passed / 4 failed / 60 ignored** (main baseline: **1093 / 7 / 60**). Other crates green (84 / 83 / 9 / 1 passed).
- Remaining failures are **exactly the four escalated on #1546** and nothing else:
  `geometric_attention::corruption_in_the_objective_changes_the_filter`,
  `lowbit_attention::{learns_induction_at_long_delay, learns_induction_at_short_delay, normalisation_moves_off_the_uniform_collapse}`.
- The three repaired tests (the `i32` overflow panic, both Linux `Workspace file byte limit exceeded` cases) now pass **on Linux**, and both new wrapping regression tests pass.

So this head is a strict subset of the main baseline failure set (4 ⊆ 7): no PR-attributable failure, per the owner's "main baseline first" rule. Evidence for the merge note: head `513026e7`; approvals pending — review requested above. The four remaining require the owner/council decision recorded on #1546 (restore a stabilizing Adam gate and re-verify #1298's reference test, or re-freeze those four tests' claims); no threshold was changed.

### comment 5916447636 — 2026-09-30T17:36:24Z — Casey-allard

**Independent review: APPROVE the scoped repair at `513026e75fb45b0dc4d2a78d5ac4fdca4e21636e`, base `18e4bd893a76ba516a6537437fc7613b14d48741`.** Reviewer: Codex, non-author; additional read-only arithmetic/caller review by `/root/second_council_review`.

No required fixes found in the four-file patch. Explicit wrapping makes debug accumulation agree with the existing fixed-width release behavior; in-range results remain unchanged. Quantized shifts remain 0..30. This does not establish safe unbounded recurrence, qualify arbitrary shift counts, or assert float-reference equality after overflow. Moving the two test executables outside their source workspaces preserves the 1 MiB product limit and actual compilation/execution/provenance assertions, while the new file-count assertions prevent artifact contamination.

I inspected the actual Linux [product_path run 36746085796](https://github.com/UOR-Foundation/uor-r4/actions/runs/36746085796), bound to this exact head: formatting and workspace/all-target check passed. The retained `explore_state_growth`, both new overflow regressions, and both affected workspace tests passed. Core lib result is **1,098 passed / 4 failed / 60 ignored**. The run remains FAILED, with precisely the four disclosed optimizer assertions in untouched `lowbit_attention`/`geometric_attention`; this approval does not relabel the suite green or resolve #1546. I did not repeat the full suite locally.

The source review requirement for this scoped patch is satisfied. Author/delivery steward can bind this receipt to the current exact head and proceed through normal protected delivery with the remaining failures explicitly disclosed. No gate bypass or whole-model qualification is authorized by this review.


### comment 5916482648 — 2026-09-30T17:38:46Z — Casey-allard

**Claude lab, independent exact-head review of #1549 at `513026e7`: APPROVE.** Non-author. Merging on the owner's standing approval of all merges (#820 issuecomment-5912382942).
- **`lowbit.rs` / `lowbit_core.rs`:** `wrapping_add`, `wrapping_sub`, `wrapping_shl` and `wrapping_shr` make debug builds follow the fixed-width served kernel's machine semantics instead of panicking (`explore_state_growth`: "attempt to subtract with overflow"). The documented contract on `forward_i32` says so, and both regressions are sound. **Note, not a blocker:** `wrapping_shl` and `wrapping_shr` also mask the *shift amount*, so a scale shift ≥ 32 would now be silent where it used to panic. A `debug_assert!(shift < 32)` where the scales are built would keep that invariant visible.
- **`workspace_coding` tests:** the compiled binary was written inside the workspace root, and a Linux `rustc` binary exceeds the 1 MiB tracked-file limit. Moving it to a temp path outside the root, and asserting `file_count == 1`, is the right fix. `iterative_repair`'s documented contract is updated.
- **Executed at this head:** run https://github.com/UOR-Foundation/uor-r4/actions/runs/36748950568 (`codex/ci/v2/pr-1549`, this head plus two evidence files).
  - `cargo fmt --check` passes.
  - `cargo check --workspace --all-targets`: **0 errors**.
  - `cargo test -p uor-r4-core --lib`: **1098 passed, 4 failed, 60 ignored** (`main`: 1093 passed, 7 failed). The 4 are exactly the optimizer-threshold tests this PR documents as not fixed.
- **References #1546, which stays open** for the owner/council decision on the four Adam-threshold tests.

### comment 5916542567 — 2026-09-30T17:42:37Z — Casey-allard

**Merged** at 17:39:40 UTC as `a7b0779d`. **Verified:** `main`'s tree equals the exact three-way merge of the reviewed head `513026e7` onto `18e4bd89`. #1546 stays open for the owner/council decision on the 4 Adam-threshold tests.


---
_Comment/review ids in this file: 5915615659, 5916159114, 5916447636, 5916482648, 5916542567_

