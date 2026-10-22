# ORDER: infra-loop-pr12-git-hooks, deepseek-3 (loop PR 12: git hooks, pr-check before push, Lab and Order trailers)

Lab: deepseek. Item: [infra-loop-pr12-git-hooks](../../../../roadmap/items/infra-loop-pr12-git-hooks.md) (#2917). Setup row: 12. Base: `7864b05`.

## Goal

The item's done-when: **A commit hook checks that the binary is current and that `Lab:` and `Order:` trailers are present; a pre-push hook runs `pr-check --local`. Trailers are tags, not proof (D55).** The planted fault: **a commit without trailers is refused.**

## Cancel probe (run first)

At base: `crates/uor-r4-harness/rules/` holds only `precommit-hook.sh`; no source or rule file mentions `commit-msg`, `pre-push`, trailers, `check-message` or `push-check`; `uor-harness git check-message` and `uor-harness git push-check` exit 2 with "unknown action"; the installed pre-commit hook runs only `check --staged`. Class: instrument; the task stands.

## Files in scope

- `crates/uor-r4-harness/src/githelp.rs`, `version.rs` (readers only if needed), `looparrive.rs`, `main.rs`, `lib.rs` (registration only).
- `crates/uor-r4-harness/rules/{precommit-hook.sh,commit-msg-hook.sh,prepush-hook.sh}`.
- `crates/uor-r4-harness/tests/**`.
- `docs/harness/gates.json`, `docs/harness/README.md`.
- `docs/harness/claims.json`, the item's `status:` line, `docs/loop/tasks/infra-loop-harness/deepseek-3/**`.

## Plan

1. Two new managed hooks: `commit-msg` (via a new `uor-harness git check-message <file>`: binary current plus `Lab:`/`Order:` trailers) and `pre-push` (via a new `uor-harness git push-check`: `pr-check --local` for each pushed lab branch). `pre-commit` keeps `check --staged`.
2. `install-hooks` writes and refreshes all three with the existing refusals for foreign, symlinked and unreadable files; `loop arrive`'s hooks check verifies all three.
3. The `git-hooks` gate and its planted break; the docs.

## Done when

- `uor-harness git install-hooks` installs executable `pre-commit`, `commit-msg` and `pre-push` hooks, is idempotent, and refuses foreign, symlinked or unreadable files.
- A commit message with no `Lab:` and `Order:` trailers is refused with a reason; merge messages are allowed; an older binary is refused with the rebuild command. `--no-verify` stays the documented bypass.
- The pre-push hook runs `pr-check --local` for a pushed `<lab>/...` branch and skips other refs with a printed reason.
- `loop arrive`'s hooks check FAILs when any of the three hooks is missing, different or not executable.
- `loop gates --break all` is CAUGHT for `git-hooks` with its own text; the tests, `cargo fmt -p uor-r4-harness --check`, claim-wording and links on changed files pass.

## Stop and ask if

- A hook would block the owner's or another lab's branch type; a new dependency is needed; a file outside scope must change; an owner-only path would change.

Size estimate: about 900 lines.
