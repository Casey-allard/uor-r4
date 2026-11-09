# ORDER: establish Studio cowork parity baseline and recover the shell, kimi-2

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: fresh `origin/main` `229739cf9` plus preserved acceptance merge `c17a84682`; recovery reattached at `c4d350b8a`. Owner-approved design: `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md`.

## Decision review and goal

The owner's approved plan authorizes the M7 parity recovery. Use the existing open IDE item rather than creating a duplicate. Replace the green placeholder with the approved neutral Claude-style workbench and pin the parity/browser acceptance foundation. Later orders implement the agent runtime and full workflows; this order does not claim them. The new `org.uor.r4.desktop.cowork` product identity separates the recovered native app from the rejected app's restored state.

## Files in scope

- `docs/roadmap/items/studio-ide-harness.md`, generated `docs/roadmap/issues.json` and `docs/roadmap/README.md` only if the normal generator later succeeds, and this task folder.
- The approved design/plan and `docs/products/harness-parity.md`.
- `.gitignore` only if an unignored evidence path is found, `web/studio/package.json`, `web/studio/package-lock.json`, `web/studio/playwright.config.ts`, and `web/studio/tests/**`.
- `web/studio/src/**` limited to the shell, presentation, focused component tests, and preserving adapter behavior.
- `src-tauri/tauri.conf.json` only for the new product identity, and `scripts/tests/test_typed_runtime.py` for its regression.

No `crates/**`, model API/engine/worker/tokenizer, Claude-owned, production-publish, real-credential, real-profile, or unrelated file changes. Any additional required path needs a new top-session scope review before editing.

## Size budget

Up to 4,500 added/changed lines for the approved design/plan/parity matrix, Playwright foundation, shell components, styles, and tests. A larger diff requires splitting the design records from the implementation before review, not hiding generated files or deleting assertions.

## Done when

- The existing #3569 item and this ORDER are present. The unrelated roadmap sync refusal is recorded without editing unrelated files.
- Playwright is pinned development-only; baseline failures are retained; neutral shell landmarks, responsive themes, project/session navigation, composer selectors, and pane dock pass in installed Chromium at 1440×900 and 390×852.
- Existing typed/plain-JS/typecheck/build checks pass, screenshots receive direct inspection, and no green placeholder token remains as application chrome.
- The parity matrix names all later `NOT_RUN` capabilities honestly. Report negatives first, run local policy/full applicable gates, obtain fresh exact-head review, and deliver through the protected path.

## Stop and report conditions

Stop before weakening/removing an existing assertion to hide a regression, adding another dependency without a scope review, touching Claude/model API paths, using real data/credentials, or claiming agent/background/hosted/model capability from this shell unit. Use installed Chromium, do not download a browser or delete prior evidence/other labs' outputs, and stop before any build that would cross the 128 MiB physical stop margin.
