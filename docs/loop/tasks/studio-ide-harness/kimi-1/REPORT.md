# REPORT: Mac-inspired Studio IDE and simplified UOR/API connections

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), with follow-up [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569); both issues remain open. Order: [ORDER.md](ORDER.md). Design: [design](../../../../superpowers/specs/2026-11-02-studio-ide-harness-design.md). Plan: [plan](../../../../superpowers/plans/2026-11-02-studio-ide-harness.md). Original base: `85f634af30809c998cc3dbd830366c85be214934`. Merged forward from `origin/main` `7394c344d`, `b6ebf7002745f1c9ec5c7de236bc9f9ae66009e3`, and `e2470ec4d09e873352df128133116f73d3ababda` before final delivery checks; the earlier review base was `de9cfc84a`. Authoritative final-head evidence will be posted on the PR after the clean head is verified; the committed receipts are explicitly historical implementation evidence.

## What did not work

- The frozen session ended after the three implementation/documentation commits and before the REPORT, browser delivery evidence, exact-head review, or PR. Those five commits (including the two planning commits) were intact and clean; no uncommitted source needed reconstruction.
- `origin/main` advanced by DeepSeek's #3581 before verification. The branch merged that commit forward without force-push; no Studio files conflicted.
- The checkout had no generated `web/studio/pkg`, so the page truthfully reported the engine unavailable. This visual/source unit did not build a model bundle or invoke a model, API provider, Test, Send, or real file.
- Browser automation needed fresh element snapshots and explicit responsive orientation/dimensions. More importantly, the first fresh exact-head review returned **CHANGES_REQUIRED** and proved the visible Connections click was genuinely unwired (`navConnections` vs the wired `navModels`); the original test fixture had hidden this by using `navModels`. The actual markup now restores `navModels` and has a direct file-based regression plus desktop/mobile click checks.
- The same review found that stored desktop expansion reopened the mobile drawer on reload, Tab could leave the drawer, close-focus restoration was conditional, regrouping stranded the URL input outside its manager's mount, the palette used a separate focus-only path, and three control classes were below 24px. Mobile hydration now starts collapsed, a drawer-local Tab trap and unconditional close-focus restoration are tested, URL lookup/palette/diagnostic behavior share one helper, and the controls meet the minimum. The URL manager received a declared narrow lookup repair, not a provider/network behavior change.
- Roadmap rendering initially rejected the new item's invalid `issue` key and missing Sources section. Those metadata defects were corrected; a stale `xtask` binary then falsely proposed creating a duplicate of #3569 until a clean rebuild proved the live marker/labels were valid. Sync updated #3569 and the generated 136-item views with zero creates.
- One exact-head review returned **CHANGES_REQUIRED**. Its overlay/focus repairs initially failed three of 19 focused tests: shared cleanup overwrote the return target during replacement, a navigation fixture omitted the actual Settings close button, and dynamic confirmation lacked an initial hidden state. The repair now preserves one return target, uses synchronous dialog focus, and restores the session trigger on Escape/Cancel. Dynamic-node fixtures re-query each new dialog; no assertion was removed.
- The subsequent exact-head review also returned **CHANGES_REQUIRED**: mobile session-menu Tab escaped to the drawer, outside-click dismissal stole focus, F2 collided with the menu, the diagnostics summary and file input lacked visible focus, provider/attachment assertions were incomplete, and gate wording conflated working content with a clean commit tree. Six new regressions failed first, then two remaining root causes (the explicit summary outline and `.session-title` rename selector) were fixed. A later review found Cmd/Ctrl navigation running behind transient menus; a new failing-first test now covers both menu types and blocks those shortcuts. All 34 focused tests and the full 255-check Studio/model-source suite now pass, and the ORDER discloses the local-file keyboard class repair. Final clean-head evidence will be posted on the PR rather than backdated.
- No real model/API trajectory, MCP connectivity, desktop stdio bridge, real disk access, typed-library cutover, production publish, or trained-chat capability was exercised or is claimed. #3569 and #3500 remain open for their broader units.

## What changed

The recovered five-commit unit:

- Applies a Mac-inspired native visual system to the existing plain-JS Studio: system typography, calmer opaque surfaces, softer radii, compact native controls, at least 24px targets, and visible keyboard focus in dark/light themes and responsive layouts.
- Simplifies Connections around **UOR Chat — experimental** through the existing supported non-discrete stack/WASM path, **Custom API providers**, and a closed **Artifact diagnostics** region. Original buttons, listeners, `data-action` values, readiness checks, storage, and truthful capability labels are preserved.
- Renames user-facing Models navigation to Connections while preserving internal `models`/`model` keys and compatibility IDs. No engine, worker, tokenizer, `studio_api.js`, model artifact, MCP behavior, dependency, publisher, deployment, `.github/**`, or typed-library cutover was added.
- Includes the ORDER, roadmap item, design, implementation plan, and focused actual-page tests for computed visual treatment, source regrouping/listener preservation, readiness, and navigation labels.
- Fresh-review scope explicitly includes `studio_files.js`: the tree attachment is a separate native button with visible row hover/focus and coarse-pointer states. Overlay exclusivity, early diagnostics, remaining controls, and live status/aria-current behavior have focused tests in `studioOverlayState.test.ts`, `studioNativeTheme.test.ts`, and `studioUrlDiagnostics.test.ts`. The narrow `model_sources.js` local-file class repair is also declared: the input remains keyboard-focusable and the visible picker label receives the outline. No provider/network behavior changes.

During verification, the modern-web accessibility check exposed that the custom mobile drawer updated ARIA/scrim/`inert` without moving focus. A new regression failed first (`expected '' to be 'navAgent'`). `setChrome` now moves focus to the first destination on mobile open and restores Menu on every mobile close, while a drawer-local Tab trap contains focus and mobile hydration starts collapsed. All 243 tests and the real-browser open/close, Connections, stored-expanded reload, palette, menu, rename, and visible-focus checks pass after these repairs. The recovered commits, merge-forward, ORDER/REPORT, and these scoped repairs form the delivery.

## Measurements and checks

At the latest repaired content before the final evidence commit:

- Focused review regression run: **6 files, 34 tests passed, 0 failures**; the new cases initially produced six failures, then two remaining root-cause failures before the fixes.
- `npm test`: **29 test files, 243 tests passed, 0 failures**.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 123.06 kB (gzip 33.76 kB), `dist/studio-foundation.css` 3.57 kB (gzip 1.14 kB), uncommitted.
- `node --test assets/js/model_sources.test.mjs`: **12 tests passed, 0 failures**.
- `node --check assets/js/studio_ui.js`, `model_sources.js`, and `studio_files.js`: PASS.
- `node verify_studio_page.mjs --strict-router`: PASS for `index.html` and `geometric_prime_router_webapp.html`; `legacyResidual: []`, `failures: []`.
- `node verify_studio_page_load.mjs --strict-router`: PASS. Studio: 12 page-load requests, 0 external; router: 2 requests, 0 external. Planted external detection passed on inline, blob-worker, and late channels.
- Combined executed unit/model-source tests: **255 passed, 0 failures**.
- `cargo xtask roadmap check`: **136 items OK**. The earlier refresh synced #3569 with zero duplicate creates and wrote `issues.json`/the 136-item roadmap README.

Clean-commit repository gates, claim wording, local policy, the fresh exact-head verdict, and GitHub checks will be posted on the PR only after the actual delivery head is verified. Earlier gate logs are historical evidence for their named commits, not represented as final-head evidence; no GitHub result is claimed in this committed REPORT.

## Real browser evidence

The built-in browser exercised the actual local page, with no real credentials/files, external endpoints, or model request:

- **Desktop 1440×900:** Agent, Workspace, Connections, and Guide; dark and light themes; visible keyboard-focus ring; primary UOR Chat source; custom providers; collapsed diagnostics; no-source Send disabled.
- **Demo workflow:** opened the six-file demo tree, opened two editor tabs, edited the CSS buffer without saving to disk, used it as the Agent prompt, and attached `web/studio/index.css`. The inspector/composer showed one file and the explicit local-WASM context limitation.
- **Mobile 393×852:** Agent, Workspace, Connections, and Guide rendered without horizontal page overflow. Navigation started closed; the labeled backdrop appeared when opened; Escape, backdrop click, and destination selection each closed it, with the Menu left collapsed.
- **Latest review repairs:** outside-click dismissal left typed text in the composer; F2 opened one Session title input without a menu; mobile Tab moved Rename → Duplicate inside the session menu; the diagnostics summary measured about 25.5 CSS pixels high; Shift+Tab from the dummy `.invalid` URL field drew a visible outline on Choose model file. No URL load or file selection occurred.
- The page-load trap independently passed its planted-request self-test. Browser presentation and source readiness are visual/non-model evidence, not trained-chat, live interoperability, real API/disk, production-publish, or typed-parity claims.

## Storage and preservation

- Three older clean, merged Kimi worktrees were removed only after their ignored/untracked notes and scratch were moved byte-preserved into `local/kimi/retired-worktrees/`; their PRs were #3476, #3436, and #3468. Other labs' worktrees and scratch were untouched.
- Guarded `uor-hygiene --apply` removed those worktrees and idle build output, freeing about 2 GB. Two additional known idle Cargo target directories (`local/target`, `local/pod-provider-target`) were cleaned for 652.0 MiB of regenerable build output. Nothing unique was deleted and the Trash was not emptied.
- Free space reached 30,131,724 KiB (about 30.1 GB decimal / 29 GiB) before the earlier Studio suite, restoring the project's 30 GB minimum. The resumed final repair measured **35 GiB free**. Generated `node_modules`, `dist`, and the SDD ledger remain uncommitted and will be handled by the normal post-merge cleanup.

## Checks run at the head

- The earlier implementation commit `918aa886f` had **11/11 GREEN** repository gates (log `local/kimi/gates/1793694201-26745`), passing local policy/claim wording, and the disclosed D76 advisories. This is historical evidence for that content, not a claim about the final delivery head.
- The latest repaired content has **34 focused tests and 255 combined Studio/model-source checks**, with typecheck/build/syntax and both page-load layers passing. The real browser rechecked all new menu, outside-click, F2, summary, and file-label cases.
- After the implementation and receipt commits, the same Studio checks plus `loop gates --full`, `loop pr-check --local`, claim wording, roadmap check, and `git diff --check` run again on the clean exact head. Their actual head/tree/results and the fresh review verdict will be posted on the PR only after they exist; pending GitHub checks and merge-queue delivery are not claimed here.

## Skills used

Recovery and working discipline, isolated worktrees, plan execution, test-driven development, systematic debugging, solution-integrity, scope/context guards, built-in-browser verification, modern web guidance, and fresh exact-head delivery review. No GPU, paid resource, real credential, real user data, or model compute was used.

### Modern Web Guidance

- **`navigation-drawer`**: I retained the existing sibling-scrim fallback rather than introducing a new popover/stacking system, and verified that Escape, backdrop, and destination dismissal all converge on the same collapsed/inert state ("replace `::backdrop` with a sibling element styled identically"). Focus is no longer left outside the custom overlay.
- **`accessibility`**: I added failing-first and real-browser checks for entering/restoring focus ("Move focus to the overlay"), retained native buttons and `:focus-visible` outlines, and kept minimum targets at 24×24 CSS pixels ("Enforce the target size of elements using `min-block-size` / `min-inline-size`").

## Exact delivery head

The authoritative delivery head will be the clean commit created after these repairs and its evidence delta. Only after that commit exists will the PR receive its 40-character SHA, tree, clean status, base `e2470ec4d09e873352df128133116f73d3ababda`, actual checks, and fresh exact-head review. The committed [`receipts.jsonl`](receipts.jsonl) is explicitly historical implementation/pre-final-working-tree evidence; it does not assert the delivery head or a final verdict. Earlier gate logs remain labeled lineage, including the earlier verified tree `13f9452cca6611fcebbfbfbe0f218fe2cdf945c2` and the actual `db222246d..dd58c76a` REPORT/receipts-only delta; newer source/test repairs are not mislabeled as docs-only. Fresh exact-head review, protected PR checks, and merge-queue delivery remain pending and are not claimed as complete.

## Still open

- #3569 remains open for real remote MCP behavior and the later desktop bridge. No decorative MCP panel or simulated discovery was added.
- #3500 remains open for typed-library parity/migration/cutover. The Preact library stays unmounted.
- Production publication remains separate. The current trained M1 artifact remains next-token prediction, not a conversational product.
- Generated STATE/HANDOFF/AGENTS lessons remain a separate generated-state render dependency. Their temporary render was reverted rather than including owner-path changes in this PR; the delivered `docs/roadmap/README.md` and `issues.json` carry the current 136-item mapping. Historical implementation receipts are in `receipts.jsonl`; the final clean-head checks and verdict are external PR records, not claims in this committed file.

## Plain summary

The recovered Studio unit is verified: it has a coherent native-style IDE presentation, a simplified Connections workflow, safe demo editing and attachments, working desktop/mobile navigation, and truthful model/API limits. All executed checks passed. No new serving or model capability is claimed.
