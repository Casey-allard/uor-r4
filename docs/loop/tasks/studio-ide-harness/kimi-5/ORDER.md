# ORDER: close PTY lifecycle and delivery-evidence findings, kimi-5

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `ec49e32b4d5af2f12cd6b84328584336c6b182ba`, tree `fa2e0af9a80c5994616423ce7b02400c6331ca27`; merge base `ad2f12edd8e3c2396b2fe678dbf16d02d128bb9f`. PR #3890 remains draft and unqueued. Fresh reviewer agent-66 returned CHANGES.

## Findings and decision review

- `TerminalPanel` can mount an `openTerminal` result after the panel unmounts, without closing that returned shell. A rejected close also marks the UI disposed before the error branch, drops the session id, and cannot display the failure while the backend reinserts the live session. Preserve the consumed one-shell approval and all current bounds, but make unmount close any returned stale shell and make close failure retain retryable session/error state.
- The active report has pre-merge suite counts and a stale final-evidence pointer. Current native/loopback evidence is at ec49/fa2e0af, but that head will become intermediate after this repair. The new candidate's exact HEAD/tree belongs in the PR body and durable identity, not a guessed self-hash.
- The notebook says merged before protected merge; #3569 remains ready and the PR says References, not Closes. The product pages/README still contain earlier counts and contract-only package wording, and the parity matrix calls 975eb the final native head. Reconcile them to measured development scope and pending protected delivery.
- Earlier unique screenshots/JSON are not all copied into the main checkout. Preserve every existing artifact before cleanup. The cited canonical-launch screenshot is absent; disclose that its file is not retained rather than claiming a durable screenshot. The approval-consumption RED run is observed in the session but has no separate raw log; disclose that limitation.

## Files in scope

- `web/studio/src/components/{TerminalSession.tsx,TerminalSession.test.tsx}` and `web/studio/README.md`.
- `docs/loop/tasks/studio-ide-harness/kimi-3/REPORT.md` only to disclose the missing screenshot, `kimi-4/REPORT.md` for accurate historical/final roles, and this task folder.
- `docs/products/{desktop-client,harness-parity}.md`, `docs/research/notebook/m4-serving.md`, and `docs/integration/current-state.md` only if a claim needs the same correction.
- Existing unique artifacts under `local/kimi/harness-parity/`, copied without overwriting into the main checkout. Local `pr-body.md` is historical only; `pr-cowork-body.md` and the live PR are authoritative.

No model/API/engine/worker/tokenizer, new dependency, public service, security-assertion weakening, or full-parity claim. Do not delete historical artifacts or other labs' work.

## Done when

- Unmount during `enableTerminal` or `openTerminal` closes any returned stale shell and never mounts/polls it; close failure retains session id and shows a retryable error. Both regressions fail before implementation, then pass with all existing component tests.
- Neutral workbench, PTY, source guards, host approvals, all previous tests, and the actual packaged native workflow remain green. Report actual suite counts, exact-head evidence, limitations, and pending protected delivery consistently.
- Preserve earlier unique artifacts or explicitly withdraw unsupported durable-file claims. Obtain fresh exact-head review, required checks, protected merge, and fresh-main verification before cleanup.
