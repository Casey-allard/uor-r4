# Studio Lifecycle Migration Verification and Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify the existing one-time, non-destructive Studio browser-data migration at an exact `main` head and close only a reproduced gap, without creating a second migrator.

**Architecture:** Reuse the merged early `lifecycle_persistence.js` transaction and key router. Its TypeScript wrapper, existing legacy writers, typed bootstrap guard, and isolated-browser verifier already form one implementation; this plan adds no parallel state owner or persistence stack.

**Tech Stack:** Existing browser JavaScript, TypeScript, Preact, Vite, Vitest/jsdom, Node built-in tests, Rust/WASM build tools, and the repository's isolated Chrome/CDP verifier. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`, especially the mounted migration addendum at lines 163–186.

## Global Constraints

- Preserve every old migratable key byte-identically; unknown JSON fields survive through exact raw copying.
- `modelSources.v1` remains the only live provider-credential store. Never log credentials, use real credentials, inspect an actual browser profile, or contact an external endpoint.
- Fail closed before writes on malformed/conflicting data or read failure. Write and verify raw destinations, re-read sources, and write the marker last; roll back only destinations created by that attempt.
- A valid marker short-cuits later source preflight. Destinations then own edits and deletions; never resurrect a deleted destination.
- First activation is single-tab only. `ui.v1` is archival only. Injected quota is not actual system-quota evidence.
- No production cutover/publication, native packaging, default-page change, Phase B work, model compute, or useful-chat/M7/M8 claim.
- Do not weaken a verifier, add a dependency, or change product code without a reproduced in-scope defect and a newly reviewed ORDER.

## Review Focus

No uncovered in-scope input class was found. Existing tests exercise every inventoried key, unknown fields, malformed input, invalid markers, destination conflicts, marker/read/write failures, quota rollback and rollback failure, source-edit races, read-only failure behavior, singleton initialization, destination-divergent edits/reloads, and deleted-destination non-resurrection. Cross-tab atomicity, actual system quota, real profiles/credentials, `ui.v1` restoration, packaged acceptance, and production cutover remain explicit non-claims rather than reasons to widen this unit.

---

### Task 1: Confirm the premise and activation surface

**Files:**
- Read: `web/studio/assets/js/lifecycle_persistence.js`
- Read: `web/studio/src/persistence/migration.ts`
- Read: `web/studio/src/bootstrap.ts`, `src/main.ts`, and `index.html`
- Read: `web/studio/assets/js/studio_ui.js` and `studio_files.js`
- Read: `docs/loop/tasks/studio-typed-shell-upgrade/kimi-67/PLAN.md` through `kimi-69/REPORT.md`
- Create: `docs/superpowers/plans/2026-11-08-studio-lifecycle-migration.md`

**Interfaces:**
- Consume the existing `migrateLifecycleStorage`, `createLifecyclePersistence`, `activateStudioMigration`, and `studioMigrationStatus` exports.
- Consume the existing `window.UORStudioPersistence.getItem`, `setItem`, and `removeItem` route used by legacy conversation, theme, settings, UI, attachment, tab, and recent-file code.
- Produce no new runtime interface.

- [x] **Step 1: Establish exact identity and the cancel probe**

Run `git status --short --branch`, `git rev-parse HEAD`, and `git rev-parse 'HEAD^{tree}'` in the isolated worktree. The verified head is `bfa5de6ce73dbd459f1d2dca1e4e677ccc2bae54`, tree `b5dea93d0dde75f3134cc8576f279d57f08232df`, with no tracked edits before this plan.

Cancel the implementation premise if the writer, marker, routed persistence, mount refusal, and real-browser verifier already satisfy the design. They are all present.

- [x] **Step 2: Trace activation before every storage reader**

Confirm `lifecycle_persistence.js` loads in `<head>` before the inline theme reader. Confirm conversation/editor helpers in `index.html`, settings/UI/attachments in `studio_ui.js`, and tabs/recent files in `studio_files.js` use `UORStudioPersistence`; provider modules continue using `modelSources.v1` directly. Confirm typed bootstrap throws before Preact rendering on migration error.

- [x] **Step 3: Confirm prior protected delivery**

`gh pr view 3859 --json state,mergedAt,mergeCommit,headRefOid,title,url` reports MERGED at `2026-11-08T10:09:42Z`, merge commit `000f4b457ee020013654ffbc65f9da5061c2317a`. `git merge-base --is-ancestor 000f4b457ee020013654ffbc65f9da5061c2317a HEAD` succeeds. Do not reopen that delivered implementation.

### Task 2: Reproduce unit, build, and real-browser acceptance

**Files:**
- Test: `web/studio/src/persistence/migration.test.ts`
- Test: `web/studio/src/persistence/legacyRouting.test.ts`
- Test: `web/studio/src/persistence/runtimeSingleton.test.ts`
- Test: `web/studio/src/bootstrap.test.ts`
- Test: `web/studio/verify_studio_migration.mjs` through `verify_native_runner.mjs`
- Modify no product or verifier file.

**Interfaces:**
- Consume the existing Vitest, Node, Vite, local native-runner, and CDP checks unchanged.
- Produce measured pass/fail evidence, not a new test harness.

- [x] **Step 1: Run focused and complete static/unit checks**

From `web/studio`, run `npm ci`, the four focused test files, `npm run typecheck`, `npm test`, and `node --test assets/js/*.test.mjs`.

Measured: focused **32/32**, complete typed **435/435 across 49 files**, plain JavaScript **113/113**, and typecheck PASS.

- [x] **Step 2: Classify the fresh-worktree build prerequisite**

An initial `npm run build` before artifact generation failed because `pkg/uor_r4_integer.js` did not exist. The clean worktree had no `pkg/`; this was a missing generated input, not a migration failure. Do not patch imports or commit generated binaries.

- [x] **Step 3: Generate through the standard local path and verify in isolated Chrome**

Run `bash scripts/studio/native-runner.sh --local` with `$HOME/.cargo/bin` first on PATH and `CARGO_TARGET_DIR=$PWD/target/kimi-lifecycle-migration`. This generated the WASM/pkg inputs, completed `npm run build`, built `uor-chat`, and served only `127.0.0.1:8000`.

While it was active, `node web/studio/verify_native_runner.mjs` returned **NATIVE TYPED RUNTIME PASSED**. Success, malformed, conflict, and quota modes each made zero external/raw-TypeScript requests and emitted zero runtime errors. Success restored every named domain, preserved all old raws after edits, isolated the provider sentinel, became `current` after reload, and did not recreate the deleted recent-files destination. Every failure mode showed the migration error with no typed shell, no marker, and preserved sources.

- [x] **Step 4: Stop the owned runner**

Stop the background runner after the verifier passes and confirm no unit-owned listener remains. No actual profile, real credential, external endpoint, or actual system-quota condition was used.

### Task 3: Close the no-gap branch or stop for a scoped repair plan

**Files:**
- Review: this plan and the design addendum.
- Modify no product file on the measured no-gap branch.

**Interfaces:**
- Consume the Task 2 evidence.
- Produce either a no-gap conclusion or a new defect-specific ORDER/plan addendum; never a duplicate migrator.

- [x] **Step 1: Select the evidence-driven branch**

All acceptance checks passed, so select **no product gap**. No product code change, new PR, exact-head review, or protected merge is needed for this already merged implementation.

- [x] **Step 2: Preserve the stop rule for any later failure**

If a later exact-head run reproduces an in-scope migration failure, stop before editing. Record the failing input, expected/actual storage behavior, owning test/file, and whether the instrument or implementation is at fault. Obtain review of a new bounded ORDER and plan addendum before a product change; the next unused loop number at this head is `kimi-73`, not a reason to create work by itself.

- [x] **Step 3: Present the no-gap plan for review**

Review confirms that the requested migration is already delivered by PR #3859 and verified at the head above. Keep this unit documentation-only; do not start Phase B, final cutover, packaging, publication, or another Studio unit.

## Modern-web guidance check

`npx -y modern-web-guidance@latest search "migrate local storage data" --skill-version 2026_10_16-6654652b` returned `migrate-web-app-origin` plus unrelated topics and reported a newer skill version. No guide was applied: origin migration guidance does not determine this application-specific, same-origin raw-storage transaction, and the repository's reviewed design/tests are the authority.
