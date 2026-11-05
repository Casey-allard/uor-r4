# ORDER: finish harness cleanup and guard edge cases

Lab: kimi. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `fe67d1e505979ef5099ecad3ef3ba36d3a13e8a6`.

## Prior units

The unified harness landed through `32578efaf`; security/state repairs through `7e6d523e8`; lifecycle/provenance repairs through `fe67d1e50`. Fresh reviewers returned CHANGES again. They confirmed many repairs but found exited-parent/descendant hangs, terminal execution-flag bypasses, bundled-pkg/root-symlink verification gaps, and unit-attribution drift. No protected merge has occurred.

## Goal

Return bounded results when descendants retain process pipes, make terminal execution claims true for every allowlisted command, verify all catalog-pinned local artifacts and their containing directories, and keep current evidence exclusively under kimi-4 with accurate prior-unit history.

## Files in scope

- `src-tauri/**` and focused process/MCP/terminal tests.
- `scripts/studio/**`, `scripts/tests/**`, and `scripts/stage-studio-site.sh` for complete catalog verification.
- `docs/products/**`, `docs/labs/infra-desktop-client-tauri/**`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, and this task folder.
- `docs/loop/tasks/infra-desktop-client-tauri/kimi-3/ORDER.md` only to correct its evidence-location statement; `docs/loop/tasks/infra-desktop-client-tauri/kimi-3/DESIGN.md` and `REPORT.md` remain prior-unit records unless moved to `kimi-4` with exact replacement.

No other paths are authorized. Do not edit retired current-state or old out-of-scope SVGs.

## Required repairs

1. A direct child that exits while descendants hold stdout/stderr must return a bounded timeout/error, never hang waiting for channel EOF or a reader join. Apply the same abandon-on-exit rule to bounded terminal processes and MCP stdio requests; add direct fixtures for both.
2. Refuse `rg --pre`/attached forms and Git config/worktree/exec-path overrides including `-c`, attached forms and long forms. Add real execution-refusal regressions and preserve exact-command approval.
3. Verify bundled `pkg/synthetic-stack.lut` and `pkg/tokenizer.json` against catalog hashes, verify every declared local artifact, and reject a symlinked `models/` root as well as symlinked children. Add tamper and root-symlink negatives.
4. Attribute the current notebook row and current evidence to kimi-4. Correct the kimi-3 ORDER's evidence-location statement, explicitly record the `7e6d523` and `fe67d1e` review findings, and name the final head after committing. Keep prior-unit reports historical and correct.

## Done when

All new regressions fail first for their intended reason; focused and full browser/Rust/runtime checks pass at the final head; rebuilt app stream and normal exit pass; both fresh reviewers approve the exact head; required GitHub checks, protected merge, fresh-main verification and hosted publication are observed. Regenerable build outputs are removed after verification.

## Stop and report conditions

No useful-conversation, visual-understanding, production-readiness, M3/M7/M8, exact hidden-state resume, signed-distribution, forced-kill-cleanup or GitHub-auth claims. Do not detach a live process and call its output complete, signal a reused PID, weaken provenance/origin/folder/approval checks, add dependencies/secrets/committed binaries, or use paid compute. Stop and re-scope before touching another path.
