# Studio Native-Style IDE and Simplified Connections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the existing Studio source page feel like a professional Mac desktop IDE and simplify its primary connection experience around one experimental UOR Chat source and custom API providers.

**Architecture:** Refine the live plain-JS page in place. Restyle the existing design system and regroup the existing model-source DOM without changing its event listeners or engine/worker paths. Keep MCP, typed-library migration, and desktop stdio outside this unit.

**Tech Stack:** HTML, CSS, vanilla JavaScript, Vitest/jsdom, Vite, existing UOR WASM worker and provider manager.

**Spec:** `docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md`

## Global Constraints

- Work under `docs/loop/tasks/studio-ide-harness/kimi-1/ORDER.md` and issue #3569. Use `References #3569` and `References #3500`; both issues stay open.
- Merge forward from fresh `main` after the prerequisite Studio backend and visual deliveries land; never force-push.
- Do not change the engine, worker, tokenizer, `studio_api.js`, model artifacts, publisher, deployment, `.github/**`, real browser data, or Claude's active work.
- Do not add MCP UI/behavior, mount the typed library, add a runtime dependency, or claim trained conversation, a UOR HTTP service, MCP connectivity, production publication, or full parity.
- Preserve existing source button elements, `data-action` values, listeners, localStorage keys, and non-model workflows.
- Use native controls and visible `:focus-visible` indicators, with interactive targets at least 24×24 CSS pixels.
- `web/studio/node_modules` and build output are never committed.

## Review Focus

- **Unavailable UOR source:** readiness must come from actual capabilities, not the new UOR Chat label. Task 2 retains `liveAgentExperience.test.ts` readiness cases.
- **Regrouped source controls:** moving a card must not remove its original click behavior. Task 2 clicks the existing `data-action="load-bundled"` button after regrouping.
- **Small/coarse-pointer controls:** native styling must not shrink targets below 24 pixels. Task 1 checks computed minimum size; Task 4 checks mobile controls in the browser.
- **Custom API reachability:** configuration and a new visual card must not imply connectivity or send a request. Task 2 retains the existing HTTP(S)/empty-state tests; Task 4 performs no Test or Send.
- **Responsive/theme accessibility:** soft surfaces must preserve focus, contrast, mobile navigation, and reduced-motion behavior. Task 1 checks computed style; Task 4 exercises both themes, keyboard focus, and mobile drawer behavior.

---

### Task 1: Mac-inspired computed visual system

**Files:**
- Modify: `web/studio/index.css`
- Create: `web/studio/src/adapters/studioNativeTheme.test.ts`

**Interfaces:**
- Consumes: existing `.primary-btn`, `.ghost-btn`, `.icon-btn`, `.source-card`, `.view-head`, `.nav-item`, and theme attributes.
- Produces: computed system-font, radius, minimum-target, surface, and focus styles reused by all views; no JavaScript API.

- [ ] **Step 1: Write the failing computed-style tests**

Import `index.css?inline`, install it into a test document, and create native controls/cards. Assert:

```ts
expect(getComputedStyle(document.body).fontFamily).toContain('-apple-system');
expect(parseFloat(getComputedStyle(primaryButton).borderTopLeftRadius)).toBeGreaterThanOrEqual(8);
expect(parseFloat(getComputedStyle(sourceCard).borderTopLeftRadius)).toBeGreaterThanOrEqual(8);
expect(parseFloat(getComputedStyle(primaryButton).minBlockSize)).toBeGreaterThanOrEqual(24);
```

Add dark/light checks that the existing `data-theme` attribute still changes computed surfaces and base `.icon-btn` checks for `min-block-size` and `min-inline-size` of at least `24px`. Do not test raw CSS source text or jsdom layout rectangles; coarse-pointer layout is checked in the real browser in Task 4.

- [ ] **Step 2: Run the tests and verify the old visual system fails**

Run: `npm test -- --run src/adapters/studioNativeTheme.test.ts`
Expected: the primary button/source-card radius or target-size assertions fail (`4px`/small controls rather than the required native treatment).

- [ ] **Step 3: Implement the native visual refinement**

In `index.css`, put `-apple-system`, `BlinkMacSystemFont`, and `SF Pro Text` before other fonts. Refine existing selectors rather than replacing the stylesheet: 8–12 px control/card radii, subtle panel borders/shadows, compact opaque surfaces, readable line height, and at least 24 px minimum targets. Remove the dashboard-like grid background in favor of a calm application surface. Preserve theme variables, reduced-motion rules, forced-colors outlines, and mobile layouts.

- [ ] **Step 4: Run the computed-style and existing navigation tests**

Run: `npm test -- --run src/adapters/studioNativeTheme.test.ts src/adapters/studioNavigation.test.ts`
Expected: all pass with no changed element IDs or behavior contracts.

- [ ] **Step 5: Commit**

```bash
git add web/studio/index.css web/studio/src/adapters/studioNativeTheme.test.ts
git commit -m "[kimi] Add Mac-inspired Studio visual system"
```

### Task 2: Simplified UOR/API connection presentation

**Files:**
- Modify: `web/studio/assets/js/studio_ui.js` (`MODELS_SECTIONS` and `regroupModels`)
- Modify: `web/studio/index.html` (Connections heading/subtitle and quick-action label)
- Create: `web/studio/src/adapters/studioConnections.test.ts`
- Modify: `web/studio/src/adapters/liveAgentExperience.test.ts`

**Interfaces:**
- Consumes: existing source groups from `model_sources.js`, especially `data-action="load-bundled"`, and `regroupModels()` invoked by `watchModels()`.
- Produces: a primary **UOR Chat — experimental** local section, **Custom API providers**, and collapsed **Artifact diagnostics**. No MCP UI, new source API, or replacement buttons.

- [ ] **Step 1: Write failing actual-module connection tests**

Build the actual source-group DOM with local, trained, research, and API cards, then run the real UI boot/regroup path. Assert:

```ts
expect(localSection.textContent).toContain('UOR Chat — experimental');
expect(apiSection.textContent).toContain('Custom API providers');
expect(diagnostics.open).toBe(false);
expect(diagnostics).toContain(trainedGroup);
expect(diagnostics).toContain(researchGroup);
expect(document.getElementById('mcpServers')).toBeNull();
expect(bundledButton.dataset.action).toBe('load-bundled');
expect(bundledButton.textContent).toBe('Connect UOR Chat');
```

Attach a spy to the original bundled button before regrouping, click the moved button, and assert the spy fires once. Keep the existing unavailable/API/local-ready tests, changing only the quick-action label expectation to `Open Connections`.

- [ ] **Step 2: Run the tests and verify they fail for the intended reason**

Run: `npm test -- --run src/adapters/studioConnections.test.ts src/adapters/liveAgentExperience.test.ts`
Expected: the old `Local engine (offline)`/`API providers (remote)` presentation and `Open Models` label fail; no fixture should replace the real regroup function.

- [ ] **Step 3: Implement connection regrouping without changing source behavior**

Update `regroupModels()` to move the existing local/bundled card into the primary UOR Chat section, retain its node/listener, and label the existing bundled action `Connect UOR Chat` while preserving `data-action="load-bundled"`. Present the existing API section as `Custom API providers`. Move trained/research and secondary local artifact tools into a closed native `<details id="artifactDiagnostics">` region. Do not add MCP markup or call any new endpoint. Update the page heading to Connections and the quick action to `Open Connections`.

- [ ] **Step 4: Run connection, readiness, demo, and navigation tests**

Run: `npm test -- --run src/adapters/studioConnections.test.ts src/adapters/liveAgentExperience.test.ts src/adapters/studioDemoWorkspace.test.ts src/adapters/studioNavigation.test.ts`
Expected: all pass; unavailable states remain disabled, custom API configuration remains valid only for HTTP(S), and the original bundled listener runs once.

- [ ] **Step 5: Commit**

```bash
git add web/studio/assets/js/studio_ui.js web/studio/index.html web/studio/src/adapters/studioConnections.test.ts web/studio/src/adapters/liveAgentExperience.test.ts
git commit -m "[kimi] Simplify Studio UOR and API connections"
```

### Task 3: Navigation and source-documentation alignment

**Files:**
- Modify: `web/studio/index.html` (navigation/model-view accessible labels)
- Modify: `web/studio/README.md`
- Modify: `docs/products/web-client.md`
- Modify: `web/studio/src/adapters/studioNavigation.test.ts`

**Interfaces:**
- Consumes: existing `data-section="models"` and `data-view="model"` contracts.
- Produces: visible/accessibility label **Connections** without renaming legacy IDs, views, or section keys.

- [ ] **Step 1: Add failing navigation-label assertions**

Assert that `navModels` exposes `Connections`, the model view has `aria-label="Connections"`, and section activation still uses `models`/`model` internally. Preserve existing mobile dismissal and breakpoint assertions.

- [ ] **Step 2: Run navigation tests and verify the old label fails**

Run: `npm test -- --run src/adapters/studioNavigation.test.ts`
Expected: `Models` does not satisfy the new visible/accessibility label.

- [ ] **Step 3: Update labels and documentation**

Rename only user-facing model/navigation text to Connections. Document the primary experimental UOR Chat stack source, custom API providers, collapsed diagnostics, and Mac-inspired source design. Explicitly distinguish source changes from the current publish and leave real MCP/browser/desktop bridge work to Unit 2/later.

- [ ] **Step 4: Run the complete Studio static and unit checks**

Run:

```bash
npm test
npm run typecheck
npm run build
node --test assets/js/model_sources.test.mjs
node --check assets/js/studio_files.js
node --check assets/js/studio_ui.js
node verify_studio_page.mjs --strict-router
```

Expected: all pass with no new warnings/errors introduced by this unit.

- [ ] **Step 5: Commit**

```bash
git add web/studio/index.html web/studio/README.md docs/products/web-client.md web/studio/src/adapters/studioNavigation.test.ts
git commit -m "[kimi] Align Studio connections and documentation"
```

### Task 4: Live browser evidence, REPORT, and protected delivery

**Files:**
- Create: `docs/loop/tasks/studio-ide-harness/kimi-1/REPORT.md`

**Interfaces:**
- Consumes: all preceding tasks and the ORDER's verification/delivery contract.
- Produces: exact-head evidence, `uor-review v1`, green GitHub checks, and protected merge-queue delivery.

- [ ] **Step 1: Exercise the real source page in the built-in browser**

At desktop 1440×900 and mobile 393×852, verify Agent, Workspace, Connections, and Guide; the UOR Chat primary card; collapsed diagnostics; custom provider presentation; both themes; keyboard `:focus-visible`; mobile scrim/Escape/destination dismissal; and the existing demo/edit/attachment workflow. Do not invoke a model request, provider Test, Send, external endpoint, real file, or real credential. Confirm no MCP placeholder or reachability claim appears.

- [ ] **Step 2: Record negatives, exact checks, and remaining scope**

Write the REPORT with what did not work first, failing-first evidence, exact test/build totals, browser coverage, and explicit exclusions: no trained-chat, live interoperability, MCP, real API/disk, production publish, desktop bridge, typed cutover, or full parity claim.

- [ ] **Step 3: Run exact-head repository checks**

Run:

```bash
UOR_LAB=kimi CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/.worktrees/kimi-studio-mac-connections/target/kimi cargo run -q --locked -p uor-r4-harness --bin uor-harness -- loop gates --full
UOR_LAB=kimi CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/.worktrees/kimi-studio-mac-connections/target/kimi cargo run -q --locked -p uor-r4-harness --bin uor-harness -- loop pr-check --local
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/.worktrees/kimi-studio-mac-connections/target/kimi cargo run -q --locked -p uor-r4-claim-wording -- .
git diff --check
```

Expected: 11/11 gates green, policy/claim wording PASS, only disclosed D76 scope/ORDER/FILEMAP advisories. Commit the REPORT and rerun affected checks if it changes the head.

- [ ] **Step 4: Obtain fresh exact-head review and deliver**

Generate the base-to-head diff containing only this unit, obtain a fresh `uor-delivery-reviewer` verdict, and repair/re-review any findings. After APPROVE, post `uor-review v1`, push the exact head, create the draft PR with `References #3569` and `References #3500`, mark ready, require green GitHub checks, and use the protected merge queue. Verify fresh `main` source/tree identity and issue states before deleting only this task branch/worktree.
