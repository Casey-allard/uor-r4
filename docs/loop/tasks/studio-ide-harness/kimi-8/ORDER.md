# ORDER: reconcile final evidence labels and complete input identity, kimi-8

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `0f9a4838ae4b96a64e58f8e279d93d3cfff86fdc`, tree `48097619e69918925635f7e68f72a1cd39638c6e`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. PR #3890 remains draft and unqueued. Fresh reviewer agent-69 returned CHANGES on records only; the lifecycle and executed evidence passed.

## Findings and decision review

- The notebook labels the final kimi-7 counts/path/report as kimi-6. Split a correctly attributed kimi-7 row from the historical kimi-6 row; do not erase their different measurements.
- `kimi-6/REPORT.md` still says 499/499 twice although the actual log says 499 passed, 1 ignored. Correct it and identify the historical/active roles. `kimi-7/ORDER.md` still names obsolete 35117/kimi-6 rather than c663/kimi-7. `kimi-7/REPORT.md` lists already completed final gates as open; leave only review/checks/queue/merge/fresh-main verification.
- Scope prose must say the warning is historical across the full change surface, not just two manifests, and name the existing FILEMAP advisory too. Head item scope includes the changed paths; protected queue acceptance is not established by the local warning or the narrow manifest exception.
- An earlier broad identity check included the changed README and returned 1, so it cannot support a no-differences claim. A second narrow list omitted assets/tests/verifiers/launcher paths. Use the complete application/artifact input surface: root Cargo/config, full `src-tauri`, full `web/studio` except only the README, launcher/site scripts, typed-runtime test, and integer/tokenizer/vision crates. Record the actual no-difference result instead of claiming the broad check passed.

## Files in scope

- `docs/research/notebook/m4-serving.md`, `docs/loop/tasks/studio-ide-harness/kimi-6/REPORT.md`, `docs/loop/tasks/studio-ide-harness/kimi-7/{ORDER,REPORT}.md`, and this task folder.
- `local/kimi/pr-cowork-body.md`, live PR body, and `local/kimi/harness-parity/0f9a4838ae4b96a64e58f8e279d93d3cfff86fdc/application-input-identity.txt`, plus the actual red/green evidence needed for fresh review.

No application/model/API/engine/worker/tokenizer change, new dependency, public service, scope waiver, test change, or additional native rebuild solely for these record corrections. Do not delete historical artifacts or other labs' work.

## Done when

- Notebook and ORDER/REPORT roles, counts and evidence paths agree; completed gates are not listed as open; PR body and REPORT disclose the full historical scope and FILEMAP advisories and leave protected queue acceptance open until it actually happens.
- The complete application/artifact input check is run and recorded with its actual result. Only README/records differ after the c663 acceptance; no code or artifact input is claimed to have changed.
- Existing 21/453 tests and final 11/11 gates/local policy remain green, evidence is uploaded/read back, and a fresh exact-head reviewer accepts the corrected delivery. Required GitHub checks, protected queue, actual merge and fresh-main verification precede cleanup. Do not queue on agent-69's CHANGES.
