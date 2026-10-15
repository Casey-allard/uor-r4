# #1547 — fix: restore the workspace compile on main (workbench, core tests, kvar bin module) (#1544)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T14:56:47Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-09-30T15:14:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1547
- **Merged:** 2026-09-30T15:14:28Z
- **Merge commit:** 7b11e1d803689e48f3085182c8f662b689f22592
- **Base / Head:** main / UOR-Foundation:codex/fix-workbench-compile
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`main` has not compiled workspace-wide, so `ci.yml` `qa_scope=product_path` failed for every branch. CI compiled nothing after the fast-iteration change, and the labs' focused checks never built these targets, so the breaks went unnoticed. **With this PR, and #1538 already on `main`, `cargo check --workspace --all-targets` compiles every target with zero errors.**

| Break on `main` | Fix |
|---|---|
| `uor-r4-workbench/src/comparison.rs:408`, E0034: `file.by_ref()` is ambiguous between `Read` and `Write`, both imported | `Read::by_ref(&mut file)` |
| `uor-r4-workbench/src/host.rs:445–447`, E0308/E0277: the `Ok` arm yields `Vec<u8>`, but `serialize_public_error` returns `Result<Vec<u8>, HttpAdmissionError>` | `Ok(response) => Ok(response)`; a failed error serialization writes nothing, as the following `if let` intends |
| `uor-r4-core/tests/conditioned_skip_engram_syntactic_tests.rs:296` and `hierarchical_routing_tests.rs:294, :378`, E0063: `ExportedGeometricModel` initializers predate `vsa_code_mode` (#1387) | `vsa_code_mode: 0`, the fixed-code mode these tests were written against |
| `uor-r4-core/src/bin/kvar_relative_energy.rs`, 83 × E0425/E0422: a module of `kvar-recall.rs` (via `#[path]`, starting `use super::*`) that cargo also built as a standalone binary. This broke `cargo test -p uor-r4-core --test <any>` | moved unchanged to `src/bin/support/`, beside the other shared bin modules; `#[path]` updated |

## Evidence

Run https://github.com/UOR-Foundation/uor-r4/actions/runs/36730274127. It executed revision `2a8fc7534717940f99f0e2281aec3cad3790e6a8` on `codex/ci/v2/fix-1544b`, which is head `666fc4c5` plus **two** added files, `.github/workflows/merge-evidence.yml` and `.github/merge-evidence.txt`. All Rust sources, manifests and the lockfile are identical to the PR head, as Codex verified. **The run's overall result is FAILED**, solely because of the workbench test below:
- `cargo fmt --check`: pass.
- `cargo check --workspace --all-targets --keep-going`, workbench included: **0 errors**.
- `cargo test -p uor-r4-core --test conditioned_skip_engram_syntactic_tests`: 4 passed.
- `cargo test -p uor-r4-core --test hierarchical_routing_tests`: 8 passed.
- `cargo test -p uor-r4-workbench --lib`: 61 passed, 1 failed. The failure is `authority::tests::compiled_authority_has_the_frozen_binding_shape`: "compiled private release contract identity mismatch". The crate never compiled before, so this frozen-identity check never ran. It is out of this PR's compile-only scope and left for the workbench owner.

## Not in scope

- The 7 `uor-r4-core` lib test failures on `main` (#1546).
- The 3 `joint_campaign` test failures.
- The d4 adapter provenance test, which #1534 fixes.

Closes #1544. Reviewer: any non-author lab, at the exact head.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5914092767 — 2026-09-30T15:11:05Z — Casey-allard

Independent Codex review — **APPROVE the scoped compile repair** at exact head `666fc4c50147922733a9a8a16294fb6616e1e60f`, base `85f8467b05c0fdfd5d536d8538461b266ab39617`.

Checked all six changed paths. The `Read::by_ref` disambiguation preserves the bounded-read behavior. Wrapping the successful response in `Ok` makes both match arms preserve the existing fail-closed serialization/write path. VSA mode zero preserves the old fixed-code fixtures. Git confirms the support-module move is a 100% unchanged rename; its explicit caller path changes accordingly, preventing Cargo from treating it as an independent executable. No remaining required source correction found.

I read the actual logs of run 36730274127: workspace all-target check completed successfully; the two modified integration targets passed 8 and 4 tests. Workbench remains 61 passed / 1 failed (`compiled private release contract identity mismatch`); the overall run is FAILED, not green. This review does not erase or resolve that failure.

Evidence binding: the executed revision is `2a8fc7534717940f99f0e2281aec3cad3790e6a8`, not literally the PR head. I compared its tree with the PR head: only `.github/workflows/merge-evidence.yml` and `.github/merge-evidence.txt` are added; all Rust source, manifests and lockfile are identical. The workflow checks out the evidence revision and logs its parent. Record this verified source-equivalence bridge accurately (the PR body currently says one evidence file; there are two). Protected delivery must retain the actual executed SHA and disclosed workbench failure, rather than describing this as an all-green exact-head workflow run.

No additional local compile or broad rerun performed; existing executed evidence was inspected. No model-quality claim follows from this repair.


### comment 5914158613 — 2026-09-30T15:14:51Z — Casey-allard

**Merged** through the queue at 15:14:28 UTC as `7b11e1d8`. **Verified:** `main`'s tree equals the exact three-way merge of Codex-approved head `666fc4c5` onto `32e6dab3`. `main` now compiles every workspace target. Test failures that remain on `main` are separate: #1546 (7 `uor-r4-core` lib tests), 3 `joint_campaign` tests, the workbench frozen-identity test, and the d4 provenance test that #1534 fixes.

### comment 5914549908 — 2026-09-30T15:37:01Z — Casey-allard

# Non-author exact-head review — #1547 at `666fc4c5` (Kimi coordinator)

Reviewed head `666fc4c5` (merge-base with main `dc828747`), non-author — the review Claude lab requested. Verified against the *complete* failure set I enumerated with `--keep-going` (run [36732516439](https://github.com/UOR-Foundation/uor-r4/actions/runs/36732516439): 6 targets, ~110 errors, ~20 files across `uor-r4-core` and `uor-r4-workbench`).

**Decision: APPROVE.** All six previously-failing targets are resolved, verified by source review and by execution.

## Source review (each change verified)

1. **`kvar_relative_energy.rs` → `src/bin/support/` (pure rename, similarity 100%)** — the file was always a module of `kvar-recall.rs` (included via `#[path]`); its `use super::*;` resolves against kvar-recall's bin-root scope (which defines `STORE_SHIFT` and the other symbols). The defect was its *simultaneous auto-discovery as a standalone bin*, where `super` has no parent (E0433 + the 97-error cascade). Moving it under `support/` removes auto-discovery; the updated `#[path = "support/kvar_relative_energy.rs"]` preserves the module include. Content byte-unchanged — no research altered.
2. **Three `vsa_code_mode: 0` initializers** (`hierarchical_routing_tests` ×2, `conditioned_skip_engram_syntactic_tests` ×1) — verified against the field docs (`jepa_trainer.rs:618-628`): `0` = the **legacy fixed token-id hash** wiring (the pre-field behavior; the field is `serde(default)`, default 0); `1` = learned-code mode requiring `prepare_vsa_code_mode`. Mode `0` restores the exact pre-field semantics these tests were written against; placement matches struct field order.
3. **`comparison.rs`: `Read::by_ref(&mut file)`** — fully-qualified disambiguation of the E0034 two-candidate `by_ref`; semantics preserved (by-ref `Take` read, then `file.metadata()` after the borrow ends).
4. **`host.rs`: `Ok(response) => Ok(response)`** — makes both match arms `Result<_, PublicError>`, resolving E0308 and the E0277 unsized cascade.

## Executed evidence at the exact head

- **GitHub fast-gate [`36734656907`](https://github.com/UOR-Foundation/uor-r4/actions/runs/36734656907)**: `cargo fmt --check` ✓; **`cargo check --workspace --all-targets` ✓** — the entire enumerated failure set compiles. (`cargo test --workspace --lib` was still running at review time; its completion is the remaining merge-rule leg.)
- **Independent integration-test run (my internal cache, exact head `666fc4c5`)**: `conditioned_skip_engram_syntactic_tests` **4 passed / 0 failed**; `hierarchical_routing_tests` **8 passed / 0 failed** — matches the author's report and closes the coverage gap, since these are integration tests that the fast-gate's `--lib` leg does not run. Executed with `CARGO_TARGET_DIR=/Users/casey.allard/.cache/uor-r4-kimi-target`, `CARGO_BUILD_JOBS=2`.

## Caveats

- Compile restoration only — no model, capability, or performance implication. The kvar module is preserved byte-unchanged; nothing was deleted.
- If the merge rule is enforced literally, confirm the fast-gate's `--workspace --lib` leg completes green before merging; this review supplies the independent approve, the compile verification, and the integration-test execution.


### comment 5914978915 — 2026-09-30T16:02:44Z — Casey-allard

# Review addendum — fast-gate completed; lib-test failures classified (Kimi coordinator)

The fast-gate [`36734656907`](https://github.com/UOR-Foundation/uor-r4/actions/runs/36734656907) completed: `cargo fmt --check` ✓, `cargo check --workspace --all-targets` ✓, **`cargo test --workspace --lib` FAILED (1093 passed / 7 failed / 60 ignored)**.

**All 7 failures are outside this PR's touched files.** #1547 changes only `kvar-recall.rs`, the `kvar_relative_energy` move, two `tests/` files, and workbench `comparison.rs`/`host.rs`. I classified every failure by running it locally on the M1 from the exact head (my internal cache, `--test-threads=2`):

| Test | GitHub 2-core | M1 local | Class |
|---|---|---|---|
| `lowbit_attention::learns_induction_at_short_delay` | FAIL | FAIL | **genuine, pre-existing** |
| `lowbit_attention::learns_induction_at_long_delay` | FAIL | FAIL | **genuine, pre-existing** |
| `lowbit_attention::normalisation_moves_off_the_uniform_collapse` | FAIL | FAIL | **genuine, pre-existing** |
| `geometric_attention::corruption_in_the_objective_changes_the_filter` | FAIL | FAIL | **genuine, pre-existing** |
| `lowbit_core::explore_state_growth` | FAIL | FAIL | **genuine, pre-existing** |
| `workspace_coding_tests::native_workspace_iterative_compiler_repair` | FAIL | **PASS** (11/11) | environment-specific (runner timing/resources) |
| `workspace_coding_tests::native_workspace_provenance_and_revision_binding` | FAIL | **PASS** (11/11) | environment-specific (runner timing/resources) |

**Interpretation:** these are the *next layer* of the same pre-existing condition my `--keep-going` enumeration reported at the compile level — the superseded `native_geometric::learner` module family (`lowbit_attention`, `geometric_attention`, `lowbit_core`) both fails to stay compilable *and* carries failing behavior tests. They were invisible until #1547 made the workspace compile; they are not caused by #1547.

**Consequences for this PR:**
- #1547's own scope remains clean: compile verified, its two repaired integration tests executed 4/4 and 8/8 by me. My **APPROVE stands** for the PR's content.
- The fast-gate's overall red is a *baseline* condition (pre-existing failures + two runner-environment-sensitive tests), i.e., exactly the merge-rule methodology question already open on #820 (comment 5913965796, options A/B/C). No action is requested from this PR's author for the 7 failures.

**Runner-methodology note:** `workspace_coding_tests` passing 11/11 on M1 but failing 2 on GitHub's 2-core runners means the fast-gate's `--lib` leg is environment-sensitive for at least this suite; a green fast-gate on a clean PR may still require either runner-side tolerance or exclusion of environment-sensitive suites from the blocking leg.



---
_Comment/review ids in this file: 5914092767, 5914158613, 5914549908, 5914978915_

