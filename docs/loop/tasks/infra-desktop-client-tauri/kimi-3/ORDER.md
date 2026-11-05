# ORDER: close harness lifecycle and evidence review findings

Lab: kimi. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `7e6d523e8ad37e39c8b8fe785102f36edcf85dab`.

## Prior units

`kimi-1` built the unified harness through `32578efaf`. `kimi-2` ordered repairs at `446aed0b4`, implemented them at `9f6f50339`, merged current main at `964a447a`, and recorded verification at `7e6d523e8`. Two fresh reviewers returned CHANGES at that head. Their source findings require one bounded repair; no protected merge has occurred.

## Goal

Make native process/cancellation cleanup idempotent and immediate, bind staged model files to the generated catalog, and align every user-facing verification/claim record with the actual implementation and measured matrices. Preserve all previous negative claims and 28 GiB of reclaimed space.

## Files in scope

- `src-tauri/**`, `web/studio/**` and focused tests.
- `scripts/studio/**`, `scripts/tests/**`, and `scripts/stage-studio-site.sh` for runtime provenance verification.
- `docs/products/**`, `docs/labs/infra-desktop-client-tauri/**`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, and this task folder.
- `docs/loop/tasks/infra-desktop-client-tauri/kimi-1/DESIGN.md` only to restore the prior-unit history; `docs/loop/tasks/infra-desktop-client-tauri/kimi-4/DESIGN.md` for current-unit design/evidence.

No other paths are authorized. The retired current-state file and old out-of-scope SVG stay unchanged.

## Required repairs

1. Make process-group termination a no-op after a child has already exited/reaped, so repeated Drop/shutdown/timeout cleanup cannot signal a reused PID. Add a regression that observes the second call doing nothing.
2. Register sidecar cancellation under one lock so an immediate cancel cannot be overwritten by request setup. Add a concurrent/immediate regression.
3. Verify every generated local model/tokenizer/audit path and SHA-256 against `models.json`; reject extra files even with allowed suffixes, undeclared paths and hash mismatches. Add those negatives before implementation.
4. Correct default/GUI matrices and working-directory-qualified commands in the lab/product/REPORT records.
5. Replace stale API, networking/consent and trained-artifact wording. Call the staging fixture a trained-manifest-format fixture using synthetic bytes, not a trained model.
6. Restore kimi-1 design history and carry current repair evidence only in the kimi-5 record. Update the stale notebook-scope note.

## Done when

New regressions fail first for their intended reason; focused and complete browser/Rust/runtime checks pass at the final head; the rebuilt macOS app and real acceptance paths still pass; both fresh reviewers approve the new exact head; GitHub checks, protected merge, fresh-main verification and hosted publication are actually observed. Cleanup removes only regenerable task outputs.

## Stop and report conditions

No useful-conversation, visual-understanding, production-readiness, M3/M7/M8, exact hidden-state resume, signed-distribution, forced-kill-cleanup or GitHub-auth claims. No new dependencies, arbitrary execution/file access, public tunnels, secrets, committed binaries or paid compute. Do not weaken process, origin, folder or provenance guards. If a change requires another path, stop and re-scope first.
