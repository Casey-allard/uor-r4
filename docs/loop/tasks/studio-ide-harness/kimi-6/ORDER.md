# ORDER: close remaining PTY identity races and reconcile delivery evidence, kimi-6

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `6fa2baf377e74080e57cb64330f091b701059e01`, tree `36d5b6701c9d3ee9bfdbecde9a21de011b4c0aca`; merged main `1e5e84a1d497222099a97e22da024b07759750a8`. PR #3890 remains draft and unqueued. Fresh reviewer agent-67 returned CHANGES, recorded on the PR at that exact head.

## Findings and decision review

- `open()` publishes the snapshot to component state before recording its id in the mutable runtime. An unmount/workspace change in that window can close nothing. The stale-close path catches rejection without retaining the returned shell, although the backend reinserts a session whose shutdown failed. A normal close rejection can restore the old workspace's session after a revision change.
- The reviewer also identified two same-lifecycle risks: truncation notice state is not reset for a new shell, and the first read can complete before xterm mounts, advancing the cursor without rendering its events. Cover these in the same state-machine repair rather than leaving another terminal cleanup unit.
- `58d14a29…` and `interactive-terminal-clean.log` are worktree-only despite the preservation claim. Copy them and the new review without overwriting, and read back the copies before any cleanup.
- The roadmap says the PTY is missing and names kimi-4 as current; kimi-5 has a stale ec49 pointer; kimi-4 says 444 tests; the notebook's 445-test row links to kimi-4. `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` are changed but absent from item scope. Historical typed-shell/current-state wording needs an explicit historical scope or the measured native acceptance correction. Distinguish the identical packaged executable from different debug host hashes.
- Scope decision: the item now explicitly authorizes the existing `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` for the already approved `portable-pty` dependency. The policy tool evaluates those earlier changes against their historical item scope and still reports that advisory; the top-session decision review accepts the narrow exception for these two files only, not a general scope waiver or another dependency change.
- This is a lifecycle/authorization repair plus evidence synchronization, not a new runtime, hosted service, full-parity claim, or approval weakening. The one-shell grant, existing backend bounds, original timing limits and prior negative evidence remain in force.

## Files in scope

- `web/studio/src/components/TerminalSession.tsx` and `TerminalSession.test.tsx` only for production lifecycle behavior and its regressions.
- `docs/loop/tasks/studio-ide-harness/kimi-4/REPORT.md`, `kimi-5/REPORT.md`, and this task folder; `docs/products/harness-parity.md`, `docs/roadmap/items/studio-ide-harness.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`, and the first historical typed-shell entry in `docs/integration/current-state.md`.
- `local/kimi/pr-cowork-body.md` and the live PR body. Existing unique artifacts under `local/kimi/harness-parity/`, copied without overwriting to the main checkout and then to the project store.

No model/API/engine/worker/tokenizer implementation, new dependency, public service, broad documentation rewrite, unrelated refactor, or test/timeout weakening. Do not delete historical artifacts or other labs' work.

## Done when

- Red-first tests prove: open-resolution/unmount before the session effect closes the shell; a rejected stale close retains a retryable shell without mounting/polling it; a close rejection after a workspace revision cannot restore the old active session; new shells reset truncation notices; and the first output is rendered only after xterm mounts.
- All existing and new terminal/typed tests pass, with no duplicate close, reopen after failed close before cleanup, stale workspace mount, or silently dropped backend identity. Complete default/GUI suites, browser/loopback and packaged native acceptance remain green at the final candidate, and all 11 repository gates plus local policy pass.
- Item scope covers the changed Tauri manifests; active reports/matrix/notebook point to the current evidence and counts; historical claims are explicitly scoped. Worktree-only negatives and the agent-67 CHANGES review are preserved in the main checkout and project store with byte verification.
- Obtain a new fresh-context exact-head review, required checks, protected merge and fresh-main verification before cleanup. Do not queue on agent-67's CHANGES or any earlier candidate.
