# ORDER: m1-position-mechanism, antigravity-11 (repair after DeepSeek review at 54690b87)

Lab: antigravity. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md) (#2843). Parent: [antigravity-10/ORDER.md](../antigravity-10/ORDER.md). Base: `b0430d406`.

## Goal

Address the two findings from DeepSeek's fresh-context review of `antigravity-10` at `54690b87c72bf275bf1573c079a84d094f5b3eb1` (`https://github.com/UOR-Foundation/uor-r4/pull/2953#issuecomment-6337904587`):

1. Move `docs/labs/m1-position-mechanism/preregistration-DRAFT-2026-10-22.md` to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (with `TEST FITNESS: PENDING`, visible in `uor-harness prep list`, leaving no duplicate copy under `docs/labs/m1-position-mechanism/`), and update references in `docs/labs/m1-position-mechanism/README.md`, `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md`, `docs/mechanisms/recurrence-gates.md`, and the task report.
2. Record the runner/environment condition for `cargo test -p uor-r4-discrete --lib`: `203 passed; 0 failed; 1 ignored` in an unsandboxed host worktree (`cargo test -p uor-r4-discrete --lib position_mechanism::tests`: `5 passed; 0 failed`), whereas in a sandboxed review worktree where `ps -p` is restricted, the pre-existing test `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` fails both at base `72e5f3e92` / `b0430d406` and at head (`202 passed; 1 failed; 1 ignored`).

## Cancel probe (run first)

Ran `uor-harness loop review --check --pr 2953` (`comment 1: [deepseek] CHANGES at 54690b87c72bf275bf1573c079a84d094f5b3eb1 on PR 2953: counted`) and verified `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` is absent at `54690b87`. The repair order stands.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/geometry/e8-h4-and-the-600-cell.md`
- `docs/geometry/quaternions-and-rotation.md`
- `docs/harness/prep/m1-position-mechanism*`
- `docs/mechanisms/recurrence-gates.md`
- `docs/labs/m1-position-mechanism/**`

## Done when

1. `docs/labs/m1-position-mechanism/preregistration-DRAFT-2026-10-22.md` is moved to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (no duplicate copy left behind), `uor-harness prep list` shows `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` with `PENDING` and `complete: true`, and all references in `docs/labs/m1-position-mechanism/README.md`, `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/mechanisms/recurrence-gates.md` resolve to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`.
2. `docs/loop/tasks/m1-position-mechanism/antigravity-11/REPORT.md` (and `docs/labs/m1-position-mechanism/README.md`) explicitly records the `cargo test -p uor-r4-discrete --lib` runner difference (`203 passed; 0 failed; 1 ignored` when `ps -p` is unrestricted vs `202 passed; 1 failed; 1 ignored` on pre-existing `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` in a sandboxed review worktree, and `5 passed; 0 failed` for `cargo test -p uor-r4-discrete --lib position_mechanism::tests`).
3. `uor-harness loop gates --full`, `uor-harness loop report --receipts`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` all pass.

## Stop and ask if

- Any file outside the declared scope needs modification.
- Any GPU execution is needed.
