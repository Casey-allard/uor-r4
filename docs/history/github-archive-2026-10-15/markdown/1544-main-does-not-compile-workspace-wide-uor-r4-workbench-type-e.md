# #1544 — main does not compile workspace-wide: uor-r4-workbench type errors break ci.yml product_path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-30T13:45:42Z
- **Updated:** 2026-09-30T15:15:28Z
- **Closed:** 2026-09-30T15:15:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1544
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**`main` does not compile workspace-wide. `uor-r4-workbench` has three type errors, so `ci.yml` `qa_scope=product_path` fails for every branch.**

Found by the product_path baseline run on `main` `d563b02f` (run 36722815271, 2026-09-30), dispatched under the owner's merge rule (#820 issuecomment-5912382942). `cargo check --workspace --all-targets` stops at:
- `crates/uor-r4-workbench/src/comparison.rs:408`, E0034: `file.by_ref()` is ambiguous, because the file imports both `Read` and `Write`.
- `crates/uor-r4-workbench/src/host.rs:445`, E0308: in `handle_connection`, the `Ok` arm yields `Vec<u8>` from `dispatch`, but `serialize_public_error` returns `Result<Vec<u8>, HttpAdmissionError>`.
- `crates/uor-r4-workbench/src/host.rs:447`, E0277: follows from the E0308.

The crate is unchanged since #1110 (2026-09-04). CI has compiled nothing since the fast-iteration change, and the labs' focused checks never build this crate, so the break went unnoticed.

**Fix:**
- `comparison.rs` calls `Read::by_ref(&mut file)`.
- `host.rs` wraps the `Ok` arm: `Ok(response) => Ok(response)`. Both arms are then the `Result` that the following `if let Ok(response)` expects: a failed error serialization writes nothing, as that line intends.

**Validation:** `product_path` on the fix branch, which compiles the whole workspace and runs every lib test.


## Comments (4)

### comment 5912574420 — 2026-09-30T13:45:53Z — Casey-allard

**Work card v1: #1544, the workspace compile fix.** Claude lab, session `claude-lab-20260930`. Base: `main` `d563b02f`. Policy: `ba266ad4`. This card is immutable.

- **Deliverable:** one PR from `codex/fix-workbench-compile` that makes `cargo check --workspace --all-targets` compile `uor-r4-workbench` again. It has two one-line edits:
  - `comparison.rs:408`: `Read::by_ref(&mut file)`;
  - `host.rs:444`: `Ok(response) => Ok(response)`.
- **Blocker it removes:** `ci.yml` `qa_scope=product_path` fails on every branch, including `main`, at this crate. This breaks the owner's GitHub-runner test path (#820 issuecomment-5912382942).
- **Outcomes:**
  - (a) product_path on the fix branch passes: open the PR and request an independent review.
  - (b) product_path fails elsewhere: fix what belongs to this crate here, and file other crates' failures separately.
  - (c) Behaviour would change beyond the two type errors: stop and publish.
- **Checks:** `ci.yml` `qa_scope=product_path` on the branch head: `cargo fmt --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --lib`.
- **Owned paths:**
  - `crates/uor-r4-workbench/src/comparison.rs`;
  - `crates/uor-r4-workbench/src/host.rs`.
- **Reviewer:** any non-author lab.
- **Resources:** GitHub runners only; no local cargo, model, SSD or GPU.
- **Next checkpoint:** when the run completes.


### comment 5913086119 — 2026-09-30T14:15:41Z — Casey-allard

**Work card v2: #1544, restoring the workspace compile.** Successor to v1 (issuecomment-5912574420, claim epoch 1). Claude lab, session `claude-lab-20260930`. Base: `main` `d563b02f`. Policy: `ba266ad4`. This card is immutable.

- **What changed since v1.** The v1 fix works: on `codex/fix-workbench-compile` `2fbfed35`, `cargo check --workspace --all-targets` compiles `uor-r4-workbench` (run 36724152365). That same run and the evidence runs show two more compile breaks on `main`:
  - `crates/uor-r4-training/tests/d4_map_codec_adapter.rs`: 7 × E0061. The fix is already in #1538, so it is not repeated here.
  - `crates/uor-r4-core/tests/conditioned_skip_engram_syntactic_tests.rs:296`: E0063, missing `vsa_code_mode` in the `ExportedGeometricModel` initializer. The field is `u8`, with 0 meaning fixed codes. It was added after this test was written (#1387).
- **Deliverable:** one PR with the two workbench edits plus `vsa_code_mode: 0,` in that initializer.
  - With #1538, `cargo check --workspace --all-targets` then compiles every target on `main`.
  - `uor-r4-core`'s 7 failing lib tests on `main` are test failures, not compile errors, and are reported separately.
- **Outcomes:**
  - (a) Evidence shows every target compiling, apart from the d4 adapter test that #1538 fixes: open the PR for independent review.
  - (b) Other compile errors appear: fix them here only if they are the same kind of stale initializer or call; file anything else.
- **Checks:** a `--keep-going` all-targets check on the fix branch, plus `cargo test -p uor-r4-core --test conditioned_skip_engram_syntactic_tests`.
- **Owned paths:**
  - `crates/uor-r4-workbench/src/comparison.rs`;
  - `crates/uor-r4-workbench/src/host.rs`;
  - `crates/uor-r4-core/tests/conditioned_skip_engram_syntactic_tests.rs`.
- **Reviewer:** any non-author lab.
- **Resources:** GitHub runners only.


### comment 5913425023 — 2026-09-30T14:33:51Z — Casey-allard

**Work card v3: #1544, restoring the workspace compile.** Successor to v2 (issuecomment-5913086119, claim epoch 2). Claude lab, session `claude-lab-20260930`. Base: `main` `dc828747`, which now includes #1538's `d4_map_codec_adapter` fix. Policy: `ba266ad4`. This card is immutable.

- **Evidence behind v3:** run 36727908024 on the v2 fix at `db4671a4`.
  - `uor-r4-workbench` compiles, and its tests run: 61 of 62 pass.
  - The `conditioned_skip…` E0063 is gone.
- **Two compile breaks remain on `main`:**
  - `crates/uor-r4-core/tests/hierarchical_routing_tests.rs:294, :378`: the same stale `ExportedGeometricModel` initializer (E0063, `vsa_code_mode`). Fix: `vsa_code_mode: 0`.
  - `crates/uor-r4-core/src/bin/kvar_relative_energy.rs` (83 × E0425/E0422). It is a module of `kvar-recall.rs`, included by `#[path]` and starting with `use super::*`, but it sits directly in `src/bin/`, so cargo also builds it as a standalone binary. That build fails, and it breaks `cargo test -p uor-r4-core --test <any>`. Fix: move it to `src/bin/support/`, as the `integrated_attention_data` modules already are, and update the `#[path]` in `kvar-recall.rs`. The file's content does not change.
- **Deliverable:** the v2 edits plus these. `cargo check --workspace --all-targets` then compiles every target on `main`.
- **Not in scope:**
  - test failures: #1546 (`uor-r4-core` lib), the workbench frozen-identity test, the `joint_campaign` tests, and the d4 provenance test that #1534 fixes.
- **Checks:** the evidence run with the workbench included and `--keep-going`; `cargo test -p uor-r4-core --test hierarchical_routing_tests --test conditioned_skip_engram_syntactic_tests`; the `kvar-recall` bin build.
- **Owned paths:**
  - the three v2 paths;
  - `crates/uor-r4-core/tests/hierarchical_routing_tests.rs`;
  - `crates/uor-r4-core/src/bin/kvar_relative_energy.rs` → `crates/uor-r4-core/src/bin/support/kvar_relative_energy.rs`;
  - `crates/uor-r4-core/src/bin/kvar-recall.rs`.
- **Reviewer:** any non-author lab.
- **Resources:** GitHub runners only.


### comment 5914169267 — 2026-09-30T15:15:27Z — Casey-allard

Done by #1547, merged as `7b11e1d8` (Codex exact-head APPROVE; evidence run 36730274127). `cargo check --workspace --all-targets` compiles every target on `main`. Test failures that remain are tracked separately (#1546 and others; see #1547).


---
_Comment/review ids in this file: 5912574420, 5913086119, 5913425023, 5914169267_

