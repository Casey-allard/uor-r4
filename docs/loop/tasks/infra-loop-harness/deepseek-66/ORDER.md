# ORDER: infra-loop-harness, deepseek-66 (repair after Claude's counted CHANGES)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `0f6d45460`. Repair for PR #3124. `References #2904`.

## Why this repair

Claude's counted CHANGES at `3d22dd555` (2026-10-24T03:47:56Z) found that `loop drive --collect` sat behind the `--client`/`--model` drive-mode checks, so the supervisor's `GO gpu-collection <id>` could never be closed (reproduced: exit 2, record stays uncollected, so the same collection is re-offered every cycle). It also found no test ran the CLI collect path, and that a watcher error could leave a record `running` forever (reproduced with a non-spawnable renew program).

## Finished steps (past tense, with their commits)

- **Built the GPU collection record** (`0ed21f8ed`): `GpuCollection`, local gitignored `local/<lab>/gpu-collections/<id>.json` with atomic write, reader skipping malformed/foreign files, `collect_collection`, `watch_lease_collect`; `loop next` now offers a finished uncollected record as `GO gpu-collection <id>`; `loop drive --collect <id> [--artifact <path>]` marks it collected; two shared registration docs updated; tests added to the gated suites.
- **Fixed a CI clippy failure** (`3d22dd555`): a needless `&format!()` borrow.

## Files in scope

- `crates/uor-r4-harness/src/loopdrive.rs`
- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_drive.rs`
- `crates/uor-r4-harness/tests/loop_next.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- this task folder (the earlier deepseek-65 folder stays immutable history)

## Done when (repair)

1. **Blocker fixed:** `--collect` is handled next to `--watch` / the lease watcher, before the `--client`, `--model`, adapter and launch-argument checks, so `loop drive --collect <id> --artifact <path>` alone succeeds (exit 0) and marks the record collected.
2. **CLI covered:** a test runs the binary with only `--collect <id> --artifact <path>` against a preserved finished record and then asserts `loop next` no longer offers it and the record is collected with the artifact path.
3. **No forever-running record:** `watch_lease_collect` writes `stopped` with the error as a note when the watcher returns an error, and a test pins that a non-spawnable renew program leaves a stopped record, never a running one.
4. **Low findings:** a second watcher in the same second with the same program gets a distinct id (`-2`, ...); the drive tests clean up their temp roots.
5. `cargo test -p uor-r4-harness` (all targets), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a new counted other-vendor exact-head review records at the repair head.

## Stop and ask if

- A ranked finding cannot be fixed inside the listed scope, or a GPU run/model/bar/hold/release is needed.
