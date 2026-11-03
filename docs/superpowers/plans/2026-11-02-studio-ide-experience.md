# Studio IDE Experience Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the live UOR Studio feel like a polished IDE harness while keeping non-model workflows useful and unavailable chat honest.

**Architecture:** Refine the existing live page and public browser APIs; do not cut over the typed library or introduce a framework. Add narrow readiness, empty-state, demo-workspace, and mobile-navigation behavior, then apply a consistent visual system to the existing panels.

**Tech Stack:** Existing HTML/CSS/JavaScript browser modules, TypeScript/Vitest/jsdom tests, built-in real browser verification.

**Spec:** `docs/superpowers/specs/2026-11-02-studio-ide-experience-design.md`

## Global Constraints

- No engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, real browser-data migration, `.github/**`, external asset, or runtime dependency change.
- No serving-conversation claim. Only the positively synthetic, supported non-discrete stack uses local synthetic continuation; research-only and discrete M1 artifacts do not enable chat.
- Preserve existing element IDs, public browser APIs, storage keys, and non-model workflows.
- Demo mode never discards dirty work and is visibly labeled.

## Review Focus

- No source/API/discrete/local source states: Send and every entry path must follow the same readiness rule (Task 1 tests).
- Dirty real/local tabs when Explore demo is invoked: preserve buffers and attachments (Task 2 tests).
- Reloaded demo state: restore and label it without implying a real folder (Task 2 tests).
- API-ready and local-ready empty sessions: conditional copy must not repeat the unavailable claim (Task 1 tests).
- Narrow viewport with drawer open: scrim, destination, and Escape must close it without horizontal page overflow (Tasks 3–4 checks).

---

### Task 1: Truthful source readiness and Agent empty states

**Files:**
- Modify: `web/studio/index.html`
- Test: `web/studio/src/adapters/pageGenerationHooks.test.ts` or a focused new actual-page test in the same directory

**Interfaces:**
- Consumes: existing `app.mode`, `app.caps`, `app.apiConfig`, `isDiscreteCapabilities`, `sendFromComposer`, and `updateComposerState`.
- Produces: one internal readiness predicate used by both Send UI and the shared send function; conditional unavailable/API-ready/local-ready empty-state content.

- [ ] **Step 1: Write failing tests**
  - With no source, `#btnSend` is disabled and `sendFromComposer`/`__studioUi.sendPrompt` returns false.
  - Discrete M1 disables chat but leaves prediction available.
  - A positively synthetic, supported non-discrete stack enables local synthetic continuation; a non-synthetic research FBS stack remains disabled.
  - An HTTP(S) API config enables API chat without a persisted reachability result.
  - Empty-state text differs correctly for unavailable, API-ready, and local-ready sessions.

- [ ] **Step 2: Run focused tests and verify intended failures**
  Run: `cd web/studio && npm test -- src/adapters/pageGenerationHooks.test.ts`
  Expected: FAIL for missing shared readiness and conditional states.

- [ ] **Step 3: Implement readiness and empty-state composition**
  Reuse one predicate for the button and `sendFromComposer`. Blocked sends return false and direct to Models. Render the three source-state variants with no fabricated conversation and quick actions for Models, Workspace, Explore demo, and New session.

- [ ] **Step 4: Run focused tests and verify passes**
  Run: `cd web/studio && npm test -- src/adapters/pageGenerationHooks.test.ts`
  Expected: all focused tests PASS.

- [ ] **Step 5: Commit**
  Commit only the readiness/empty-state implementation and tests.

### Task 2: Discoverable, safe, restorable demo workspace

**Files:**
- Modify: `web/studio/index.html`
- Modify: `web/studio/assets/js/studio_files.js`
- Modify: `web/studio/assets/js/studio_ui.js`
- Test: focused actual file/UI module tests under `web/studio/src/adapters/`

**Interfaces:**
- Consumes: existing `window.__studioFiles.openDemo`, tab persistence, workspace summary, attachments, and Task 1 quick action.
- Produces: guarded demo entry, consistent demo labels, restored demo mode; no automatic dirty fixture.

- [ ] **Step 1: Write failing tests**
  - Dirty tabs block demo replacement and preserve buffers/attachments.
  - Explore demo loads the synthetic tree and enters Workspace without real files/network.
  - Restored demo tabs set demo mode and visible labels; manual edits, not a timer, control dirty state.

- [ ] **Step 2: Run focused tests and verify intended failures**
  Expected: FAIL for missing dirty guard, inconsistent restore/labels, and automatic dirty timer.

- [ ] **Step 3: Implement the demo guard, entry action, and restore behavior**
  Guard at the shared demo entry point before clearing tabs/storage. Preserve attachments. Remove the automatic dirty timer. Restore demo mode and surface it in topbar/status/workspace metadata without changing storage keys.

- [ ] **Step 4: Run focused tests and verify passes**
  Expected: all focused file/UI tests PASS.

- [ ] **Step 5: Commit**
  Commit only demo behavior and tests.

### Task 3: IDE visual system and mobile navigation

**Files:**
- Modify: `web/studio/index.css`
- Modify: `web/studio/index.html`
- Modify: `web/studio/assets/js/studio_ui.js`
- Test: focused UI state tests under `web/studio/src/adapters/` or `src/components/`

**Interfaces:**
- Consumes: existing shell regions, `setChrome`, `activateSection`, and Task 1 empty states.
- Produces: `btnNavScrim` dismissal path and consistent dark/light IDE presentation.

- [ ] **Step 1: Write failing tests**
  - Opening navigation shows the scrim and `aria-expanded=true`.
  - Scrim click, section selection, and Escape collapse navigation and hide the scrim; destination dismissal persists collapsed.
  - An open Settings modal or command palette consumes Escape before the drawer; repeated test-module imports leave no duplicate keydown listeners.
  - Main content remains available after dismissal.

- [ ] **Step 2: Run focused tests and verify intended failures**
  Expected: FAIL for missing scrim/Escape behavior.

- [ ] **Step 3: Implement scrim and responsive behavior**
  Add one labeled `btnNavScrim`, synchronize it in `setChrome`, close on click and Escape, and preserve the existing section-selection close. Use dynamic viewport-safe dimensions and no new overlay framework.

- [ ] **Step 4: Apply the visual system**
  Refine existing final design tokens/rules: panel hierarchy, readable type, spacing, active/hover states, quick actions, editor tabs, model cards, dialogs, and status bar. Preserve offline fonts, reduced motion, light/dark contrast, and existing IDs.

- [ ] **Step 5: Run tests and real browser checks**
  Run focused tests, then inspect desktop 1440×900 and phone dimensions in the built-in browser. Check all four views, dark/light themes, drawer dismissal, and no horizontal page overflow.

- [ ] **Step 6: Commit**
  Commit only visual/navigation changes and tests.

### Task 4: End-to-end verification and protected delivery

**Files:**
- Modify: `web/studio/README.md`
- Modify: this unit's ORDER, REPORT, and implementation records

**Interfaces:**
- Consumes: Tasks 1–3 behavior and checks.

- [ ] **Step 1: Run complete automated verification**
  Run Studio tests, TypeScript, production build, Node model-source tests, modified-JS syntax checks, and `node verify_studio_page.mjs --strict-router`. Run repository gates, claim wording, local policy, and `git diff --check`.

- [ ] **Step 2: Exercise the real non-model workflow**
  In the built-in browser: Explore demo → open a file/tab → edit buffer → attach it → prepare prompt/context. Confirm no model request and no source Send disabled. Check unavailable and API-ready empty states with synthetic fixtures only.

- [ ] **Step 3: Record exact evidence and limits**
  Report counts, build size, browser viewports/themes, exact negative scope, and the fact that no real model/API/disk trajectory or typed-library cutover is claimed.

- [ ] **Step 4: Review and merge**
  Obtain fresh exact-head review, require green GitHub checks, queue through protected delivery, verify fresh `main`, and delete only this branch/worktree. Keep #3500 open for later parity/migration/cutover.
