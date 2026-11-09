# ORDER: close workspace-origin and unmount PTY cleanup failures, kimi-7

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `ed8fdbcaecdde3938b3fb445c90b87ec6740e723`, tree `f1c98b9c866398bde7bd32b926342b1eadadc2cb`; merged main `f75725d3071abb9488e058ac158a04d8d093014d`. PR #3890 remains draft and unqueued. Fresh reviewer agent-68 returned CHANGES at that exact head.

## Findings and decision review

- A workspace-change-initiated close captures the already-incremented revision, so its rejection can restore the old session. A stale retry can restore it again. Carry the shell's origin revision in the runtime; only a normal failure in that same workspace may restore visible identity, while stale/cross-workspace failures expose cleanup retry without mounting.
- A rejected close after unmount currently has no cleanup retry and leaves only a dead component ref. Make one bounded automatic `closeTerminal` retry on unmount, with an explicit error log if it also fails; do not imply unlimited retry or detached-process containment.
- Pending read/input/resize rejections from an old session can overwrite a new session's status. `reading` can also block the new session's first read until the old request settles. Use per-session busy ownership and identity checks in all completion handlers.
- Active current-state/typed-shell/desktop-client records still point at da6/ec49 rather than 35117/kimi-6. Update them and name the 18/450 acceptance, or clearly mark them as historical intermediate measurements.
- The integer log says 499 passed and one ignored, not 499/499 without qualification. Current records must say so. Retain the actual red logs beside the final-head green evidence. The policy scope list is historical and broader than the two manifests; the head item includes the complete change surface, while the narrow new manifest exception is documented. Verify protected queue policy at the actual head after a fresh approval rather than treating the local warning as a queue decision.

## Files in scope

- `web/studio/src/components/TerminalSession.tsx` and `TerminalSession.test.tsx` only for lifecycle behavior and regressions.
- `docs/integration/current-state.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/products/desktop-client.md`, `docs/products/harness-parity.md`, `docs/research/notebook/m4-serving.md`, `web/studio/README.md`, and this task folder plus `kimi-6/REPORT.md` only for current claims/pointers.
- `local/kimi/pr-cowork-body.md`, live PR body, and existing evidence under `local/kimi/harness-parity/`, copied without overwriting into the main checkout and uploaded/read back before cleanup.

No model/API/engine/worker/tokenizer implementation, new dependency, public service, unrelated refactor, general scope waiver, test/timeout weakening, or full-parity claim. Do not delete historical artifacts or other labs' work.

## Done when

- Red-first tests prove a workspace change with no pending close followed by two rejected closes never restores the old active session and shows only retry/error before cleanup; unmount after a rejected close invokes one bounded retry; and old read/input/resize rejections cannot overwrite a new session or prevent its first read.
- All existing and new terminal/typed tests pass. Normal same-workspace close failure still retains visible identity without remounting, and one approval per shell with existing bounds remains intact.
- Current records point to c663/kimi-7 and say 499 passed, 1 ignored. Actual red logs are preserved beside the final-head green evidence. Complete merged acceptance, packaged native workflow, all 11 gates and local policy pass at the final candidate.
- Obtain a new fresh exact-head review, required GitHub checks, protected queue acceptance of the head scope, actual merge and fresh-main verification before cleanup. Do not queue on agent-68's CHANGES or any earlier candidate.
