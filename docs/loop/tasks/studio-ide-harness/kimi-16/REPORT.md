# REPORT: make the neutral Studio the default web UI, kimi-16

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Original base: `402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`, tree `5af32980ffc2f95cc52de8ead8e80493184f1f73`; independent main `67972456d9ded94737a0a7bb1d0b36798560024b` was merged forward without model edits. Implementation and full browser/artifact verification head: `c23679a2125113c48b4c3a12c192d3a2b3bcbaab`, tree `5a0576c160ef8507771aad8d4ba6c67a56baf98c`. PR #3903 is draft. Plan: [2026-11-09-studio-default-cutover](../../../superpowers/plans/2026-11-09-studio-default-cutover.md). The following REPORT commit changes only this record; final repository gates bind the actual delivery head.

## What did not work

- Plain `/studio/` opened the legacy conversation UI. The neutral workbench mounted only for explicit `typed=1`; treating that preview as a completed default cutover was incorrect.
- The first redirect made the source page request `src/main.ts`, violating the source/runtime boundary. A built-entry-only guard then exposed a bare `get` receiver error. `runtime-page-load-red-source.log` and `runtime-page-load-red-receiver.log` preserve the exact tool records.
- The built bootstrap initially left migration inactive. Making activation unconditional then changed an already migrated store from `migrated` to `current`, breaking first-mount identity. Activation now occurs only when inactive.
- Four legacy visual fixtures and the foundation no-mount test used the pre-cutover default route. They now explicitly select `?legacy=1`, preserving their original assertions.
- Agent-79 rejected `a753667d33f0d04903debbc60da11a515904f601`: its full Playwright log predated final implementation, the mount assertion could accept `showMountError`, the plain route auto-booted legacy MCP beside typed MCP, the Vite plugin was not literally build-only, and current records implied a live default that did not exist. The index-delta wording was also wrong: `index.html` is net-unchanged, not a final-diff removal.
- The new plain-route runtime trap reproduced `legacyMcp=1` beside `typedMcp=1` (`red-default-mcp.log`). The build-transform test reproduced `apply=undefined` (`red-build-only.log`). The first post-fix runtime rerun still used the pre-fix `dist`; rebuilding was required, rather than another source change.
- The first browser run lacked generated WASM artifacts; that was setup failure, not a routing regression. Automatic `loop claim` refused the item's pre-existing shared roadmap-file scope. This ORDER uses an isolated worktree/protected PR without changing owner roadmap metadata.

## What changed

- `studio-default-typed-entry` replaces only the built `typed_entry.js` guard, retains the full `URLSearchParams` receiver, rejects a missing preview guard, and now declares `apply: 'build'`. Source `typed_entry.js` remains byte-identical and makes no raw-TypeScript request by default.
- `typedPreviewRequested` defaults to typed except for explicit `legacy=1` or `typed=0`. `mountTypedStudio` activates migration only when inactive and applies theme queries/hashes. No source redirect remains, so the original URL and hash survive.
- Legacy MCP auto-boot now uses the same default selection: suppress it unless explicit `legacy=1` or `typed=0` selects the old application. The source index is net-unchanged.
- The Playwright default-route case waits for visible `.harness-shell` and `.agent-workspace` before asserting the original URL/theme/root state. The runtime trap executes both plain and `?typed=1` built routes, requiring one shell, hidden legacy UI, `legacyMcp=0`, `typedMcp=1`, no typed MCP after returning to Agent, artifact readiness, exact arithmetic and isolated session restoration.
- Current-state, product, parity, notebook and roadmap records now describe gh-pages `6540e271…` as an explicit typed-preview publication. They say plain-root cutover is pending; `kimi-17` will own the actual post-deployment record.
- No model, engine, worker, tokenizer, API implementation, lifecycle-persistence implementation, Tauri, native-consent, dependency, `.github/**`, or public terminal service changed. This is not the shared agent runtime or a useful-model result.

## Measurements at the implementation head

- **455/455** typed tests across **52 files** pass; TypeScript check passes. **47/47** Playwright checks pass in a fresh full run. **15/15** typed/Tauri runtime contracts pass. Static page and worker checks pass, including bit-identical save/restore and zero remote requests in that check.
- The runtime page-load trap passes on both default and explicit-preview routes with zero runtime errors and no raw-TypeScript request. Both report `legacyMcp=0`, `typedMcp=1`, `typedMcpAfterWorkflow=0`, `artifactReady=true`, exact `17 + 25 = 42`, and isolated/restored transcripts. Migration success, malformed source, destination conflict and one-shot quota cases pass, including first-mount `migrated`, marked-reload `current`, restored workspace/settings/attachments/theme/UI and provider-secret isolation.
- Typed staging has **17 files, 3,063,808 bytes**. The direct **7/7** WASM audit is clean. Artifact SHA-256 is `a9e73341213880c107c7a7cc91bf644de926be9655b699eef9e7d11a70a2fea7`, **807,378 bytes**; staged and dist `index.html` are byte-identical.
- Built-in-browser acceptance of the plain staged URL shows the neutral workbench at desktop and native-scale mobile sizes. Explicit `?legacy=1` shows the old interface. A fitted mobile capture initially produced a duplicated mosaic; reload at native scale produced the clean retained capture and agrees with the independent Playwright mobile screenshot. This capture-state issue is not relabeled as an application fix.
- Exact-head logs and retained screenshots: `local/kimi/default-cutover/c23679a2125113c48b4c3a12c192d3a2b3bcbaab/`, including `typed-tests.log`, `typecheck.log`, `playwright.log`, `runtime-contracts.log`, `runtime-page-load.log`, `static-page.log`, `worker.log`, `stage.log`, `stage-verify.log`, `wasm-audit.log`, `artifact-identity.txt`, `staged-default-desktop.png`, and `staged-default-mobile.png`.
- No GPU, paid compute, new dependency, new model execution or real browser profile/credential was used. Measured command durations include typed tests 10.36 s, runtime contracts 18.146 s, Playwright 2.2 min, and the two-route runtime trap 66.144 s; the complete correction/review/delivery window is not presented as a precise measured cost.

## Checks run at the head

- Default routing, migration, build-only and duplicate-MCP regressions failed for their intended reasons before their corresponding fixes. The focused default/legacy Playwright cases then passed (`green-focused-review.log`), followed by the complete checks above at `c23679a2125113c48b4c3a12c192d3a2b3bcbaab` / `5a0576c160ef8507771aad8d4ba6c67a56baf98c`.
- `git diff --check`, staged allowlist/catalog validation and direct staged WASM **7/7** pass. Earlier raw-TypeScript, bare-receiver, stale-dist, setup and fixture failures are not relabeled as passes.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local` and `uor-harness loop gates --full` passed at clean HEAD `0ceddb0f8009ad6c55eb7afedf0c37b56474298c`, with all 11 gate commands GREEN and retained logs. Agent-80 then required this REPORT-only tense correction; the same checks are rerun at its new clean delivery head before fresh review. Fresh exact-head approval, required GitHub checks, protected queue acceptance and actual merge remain outstanding at this writing.
- Fresh-main publication and hard-reloaded public default-route verification are also outstanding. `kimi-17` will own live publication/current records and rollback identity after actual deployment; none is inferred from this staged evidence.

## Decisions and rejected alternatives

- Rejected a source-level redirect because it requested raw TypeScript. The build-only entry transform defaults the workbench while the source preview guard remains explicit.
- Kept activation limited to inactive migration state, preserving first-mount `migrated` and marked-reload `current` behavior.
- Preserved legacy screenshot/no-mount coverage with explicit legacy queries rather than deleting those tests or accepting typed output as expected legacy output.
- Used the same default predicate for legacy MCP suppression; explicit legacy routes still boot the existing panel. Both built routes are tested instead of assuming the explicit-preview result covers the new default.
- Corrected current records before publication rather than leaving a false live-default claim. Added only `mcp_client.js` and the existing runtime trap to the review-repair scope; no owner scope/roadmap metadata was changed.
- Merged independent `origin/main` at `67972456d…` forward without editing Claude's model work. The branch's actual fork-point diff remains limited to the 19 scoped Studio/record files.

## Skills used

Systematic debugging, receiving code review, red/green test-driven development, inline plan execution with a durable ledger, worktree/scope controls, real built-in-browser and Playwright acceptance, and verification-before-completion. Modern web guidance was searched earlier for startup/navigation patterns; no directly applicable guide was used for this source/build-specific cutover.

## Still open

Fresh exact-head approval, required GitHub checks, actual protected merge, fresh-main repository gates and typed build/publication, public default/legacy/terminal-limit verification, and `kimi-17` publication records. Shared agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, connectors, embedded browser/computer use, live R4 API, signed release and useful-model behavior remain unbuilt.

## Plain summary

The plain built route now mounts the neutral workbench without starting a second legacy MCP panel, and explicit legacy routes still work. The previous preview-only publication is no longer mislabeled as a completed default cutover. Local implementation evidence is green; protected merge and actual public deployment remain future facts to verify, not claims in this report.
