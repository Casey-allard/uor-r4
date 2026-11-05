# ORDER: repair the unified harness exact-head review findings

Lab: kimi. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `32578efaf5930ea3d942f3360ad01e8ab357ecde`.

## Prior unit

`kimi-1` built the shared browser/Tauri interface, native sidecar, scoped file/Git/terminal workflows, MCP transports, packaged macOS app and real GUI evidence through `a07e6e6a1`; `32578efaf` recorded its merged-head checks. Two fresh-context reviewers returned CHANGES. This repair supersedes the active ORDER only; their implementation history remains unchanged.

## Goal

Resolve every Critical/Important finding and the practical Minor regressions without weakening the existing host, origin, folder, process or claim boundaries. Preserve the synthetic-engine limitation: useful chat and visual understanding are not established.

## Files in scope

- `web/studio/**`, `src-tauri/**`, and their focused host/browser tests.
- `scripts/stage-studio-site.sh`, `scripts/studio/**`, and `scripts/tests/**` for the trained-manifest staging and packaged runtime contracts.
- `docs/products/desktop-client.md`, `docs/products/web-client.md`, an in-scope replacement product diagram under `docs/products/`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, and this task folder.

No other paths are authorized. The retired `docs/integration/current-state.md` and out-of-scope legacy SVG asset must not be edited.

## Required repairs

1. Permit only the publisher-generated `models/` subtree during final runtime verification, retain all other unexpected-file refusals, and stage a real trained manifest in a test.
2. Inspect Git local and worktree filter configuration with includes and close the check/use race with repository mutation serialization or an equivalent lower-layer boundary. Cover include, worktree and mutation-race fixtures.
3. Refuse static-file candidates whose canonical path escapes the canonical Studio root; cover a symlinked parent.
4. Release a native MCP session and stdio child when tool listing fails.
5. Allow only the real `tauri://localhost` navigation origin on macOS; retain platform-specific aliases only where Tauri uses them and test the macOS refusal.
6. Serialize browser/native connection replacement, disable both connect controls, retain the old session on failure/cancel, and disconnect stale replacements.
7. Persist recent files as `(path, rootPath)` records and make concurrent `openFile` calls idempotent per identity.
8. Correct trained-model, native-engine, local-sidecar/MCP and origin-family wording without expanding capability claims.
9. Cover Changes/Terminal deep links as initial history entries and add a non-ASCII MCP argument-size regression.

## Done when

Every new regression is observed failing for the intended reason before its production repair; focused and complete JavaScript, Rust, runtime and package checks pass at the final head; real native A/B file, both MCP transports, cancellation and normal-exit checks still pass; both fresh reviewers approve the exact head; protected merge, fresh-main verification and hosted publication are actually observed. Cleanup removes only this task's regenerable outputs and preserves evidence/other labs' files.

## Stop and report conditions

Stop before useful-conversation, visual-understanding, production-readiness, M3/M7/M8-completion, exact hidden-state resume, signed-distribution, forced-kill-cleanup or GitHub-auth claims. Do not add dependencies, arbitrary execution/file access, public tunnels, secrets, committed binaries or paid compute. If a repair requires a path outside this ORDER, stop and re-scope rather than editing it first.
