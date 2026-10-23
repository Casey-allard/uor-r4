# ORDER: infra-loop-harness, claude-65 (`loop close --verify` keeps a red gate's log)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `origin/main`. Casey asked for this unit in Claude's chat on 2026-10-23: "make loop close --verify copy a red gate's log out before it removes the worktree, as a loop unit with ORDER, test and cross-vendor review."

## Goal

`loop close --verify` runs `loop gates --full` in a detached worktree at `<main>/local/<lab>/verify-<n>`, then removes that worktree with `--force` whatever the result. The gate logs live inside it, at `local/<lab>/gates/<run>/<gate>.log`, so a red verify deletes the only evidence of why it was red. The `logs:` line it prints then points at a folder that no longer exists.

On 2026-10-23 the #3090 verify went red twice on `loop-drive` (exit 101). Every other run at the same commit was green: run directly, run through `loop gates --full`, and 10 isolated runs. The failing test could not be named, because its log had gone with the worktree (#2028, 6373547607). DeepSeek reports the same flake on #3083.

This unit makes `done_when` copy each red gate's `<gate>.log` and `<gate>.exit` to `<main>/local/<lab>/verify-logs/<n>/<run>/` before it removes the worktree, and print where they went. A green run copies nothing. The verdict, the worktree removal and every other proof are unchanged. A failed copy is reported and never turns a red into a green.

## Cancel probe (run first)

At the base, does a red `loop close --verify` leave no gate log anywhere under the main checkout once it finishes? If a log survives, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopclose.rs`
- `crates/uor-r4-harness/tests/loop_close.rs`
- This task folder

## Done when

1. **Logs kept.** A red verify leaves the red gate's `.log` and `.exit` under `local/<lab>/verify-logs/<n>/`, prints that path, and still removes the verify worktree and exits 1.
2. **Nothing kept on green.** A green verify creates no `verify-logs` folder.
3. **The test pins it.** A test in `tests/loop_close.rs` fails with the copy set aside and passes with it.
4. **Checks pass:** `cargo test -p uor-r4-harness --test loop_close`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `cargo fmt --check`, `loop report --check` and `loop pr-check --local`.

## Stop and ask if

- A file outside scope must change.
- Keeping the logs would need the verify worktree itself to survive.
