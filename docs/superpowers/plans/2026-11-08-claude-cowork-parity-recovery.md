# UOR Harness Claude-Cowork Parity Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task in the lead session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the regressed green Studio placeholder with a UOR-branded, web-hosted Claude Desktop Cowork/Code-style harness and deliver the corresponding real agent, project, file, terminal, browser, permission, scheduling, and review workflows through the R4 model API.

**Architecture:** Keep the useful Preact/Vite lifecycle state, workflow adapters, WASM support, and Tauri security boundaries, but replace the placeholder presentation and single-prompt interaction model. One typed frontend talks to one versioned harness-agent contract; the same Rust agent runtime runs embedded for Tauri/local use and as a service for browser/remote use. Claude’s separate R4 model API remains behind a narrow streaming/tool-call adapter and is never edited by this work.

**Tech Stack:** Existing Preact 11 + Vite + TypeScript, existing Rust/Tauri/object-store/reqwest stack, OpenAI-compatible streaming plus versioned harness events, Playwright as a pinned development-only browser acceptance dependency.

**Spec:** This owner-reviewed plan is the design source. At execution, preserve it as `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md` and the executable plan as `docs/superpowers/plans/2026-11-08-claude-cowork-parity-recovery.md`, with the normal ORDER/REPORT records. Reference behavior is pinned from [Claude Desktop](https://docs.anthropic.com/en/docs/claude-code/desktop), [Cowork surfaces](https://support.claude.com/en/articles/15520349-use-claude-cowork-on-web-desktop-and-mobile), [projects](https://support.claude.com/en/articles/14116274-organize-your-tasks-with-projects-in-claude-cowork), and [Dispatch](https://support.claude.com/en/articles/13947068-assign-tasks-from-anywhere-in-claude-cowork).

## Context — what actually went wrong

- The current typed shell is a placeholder, not the requested application. `web/studio/src/components/AppShell.tsx:17-52` renders six generic destination buttons and a status row; `web/studio/src/studio.css:1-311` hard-codes a green theme and generic controls. The retained acceptance screenshots show the resulting sparse green/black interface.
- The published legacy page is visually closer to the intended desktop shell, but it is still primarily a chat interface. Reverting only to that page would not deliver Cowork capability.
- `web/studio/src/adapters/generation.ts:13-82` starts and stops one generation. `AgentComposer.tsx:295-382` displays messages, context, templates, and a prompt. There is no persistent multi-step agent loop, plan/tool timeline, parallel task execution, steering, scheduling, project memory, embedded browser, or agent-accessible computer-use workflow. Existing file/Git/terminal/MCP actions are mostly manual, single-purpose panels.
- Previous Kimi work optimized for lifecycle fixtures and protocol acceptance while allowing visual and product parity to regress. Hundreds of component tests and an exact arithmetic response do not establish a Claude-cowork clone.
- The owner’s target is therefore both a **presentation recovery** and a **capability build**, not another palette change and not a claim that the existing app is already nearly complete.

## Approach

Build a sequence of protected, independently reviewed PRs that each deliver a usable vertical slice. The first PR removes the green placeholder and establishes the reference shell; later PRs add the runtime and workflows without replacing that shell again. A checked parity matrix is the final gate: no row may be inferred from a fixture, a button’s presence, or an old acceptance report.

Work backwards from the finished product:

1. The hosted and desktop apps demonstrably complete the same multi-step tasks through R4.
2. Therefore the same agent runtime, event contract, tools, permissions, and persistence must drive both frontends.
3. Therefore the UI must expose projects, parallel tasks, steering, approvals, tool activity, artifacts, and resumable state rather than one prompt box.
4. Therefore the visual shell must support chat and side-by-side editor/diff/terminal/browser panes from the start.
5. Therefore the first implementation step is a reference/Playwright harness and a shell replacement, not more tests around the placeholder.

**Rejected alternatives:** a CSS-only recolor leaves the capability gap intact; reverting to legacy plain JS restores some appearance but preserves fragmented state and still lacks the agent runtime; waiting for Claude’s API before rebuilding the GUI repeats the current stagnation. Build against explicit contract fixtures now, and require live API acceptance before claiming integration or parity.

## Global Constraints

- Do not edit `crates/uor-r4-api/**`, model/engine/worker/tokenizer implementations, Claude-owned worktrees/files, or the R4 model itself. Integration occurs only through the agreed versioned API adapter.
- Use a new isolated full worktree from fresh `origin/main`; merge forward and preserve relevant unmerged Kimi acceptance repairs by exact commit rather than losing them. Never alter the owner’s checkout or the two pre-existing modified roadmap files.
- Keep UOR branding and the R4 privacy/capability truth. Copy interaction patterns, not Anthropic branding, proprietary source, account/plan behavior, or model claims.
- Neutral application chrome replaces the green theme: light `#faf9f5`, dark `#141413`, subtle `#e8e6dc`/`#b0aea5`, orange accent `#d97757`, blue secondary `#6a9bcc`; green is semantic success only. Desktop controls use system UI fonts; long-form/artifact prose may use Lora/Georgia fallbacks. No marketing hero or oversized page headings.
- Static GitHub Pages must be labeled a limited preview. It must not claim background execution, scheduling, cross-device resume, or desktop control. Full functionality requires the local or hosted agent service, with explicit connection/privacy state.
- Preserve host-owned folder, terminal, MCP, credential, and approval boundaries. No page-supplied boolean creates authority; no arbitrary shell, silent remote fallback, token logging/storage, destructive test action, real credential, or real user profile.
- Reuse `uor-r4-object-store` for durable event/snapshot records and the existing Rust `serde`/`reqwest` boundaries. Do not add a database or second model API. Playwright is the only pre-disclosed new development dependency; any genuinely missing runtime parser/library must be named in the relevant ORDER before installation rather than silently introduced.
- Generated `dist`, `pkg`, `node_modules`, `target`, runtime binaries, screenshots, credentials, and local evidence remain uncommitted. Existing legacy pages and unique prior evidence are preserved.
- Every PR receives fresh exact-head adversarial review, required checks, protected queue/merge, and fresh-main verification. Full parity is not claimed until every matrix row and live hosted/desktop acceptance passes.

## Review Focus

- **Visual substitution:** generic green forms must fail computed-style and screenshot review, on every view and at desktop/mobile sizes.
- **Cross-task races:** stale deltas, approvals, saves, and tool results from one session must never alter another session; test parallel execution and redirection.
- **Authority bypass:** files, terminal, Git writes, browser, computer use, MCP, and remote sync must refuse unapproved or out-of-scope actions audibly.
- **Persistence/contract drift:** malformed/truncated streams, duplicate events, restart during a task, conflicting devices, and changed API capabilities must produce explicit recovery or refusal, never false success.
- **Capability theater:** static preview, unavailable host tools, unconfigured connectors, and fixture model responses must be visibly distinguished from live full-harness behavior.

---

### Task 1: Pin the reference, parity matrix, and browser acceptance harness

**Files:**
- Create: `docs/products/harness-parity.md`, `web/studio/playwright.config.ts`, `web/studio/tests/e2e/{shell,visual,workflows}.spec.ts`, and the approved design/plan records.
- Modify: `web/studio/package.json`, `web/studio/package-lock.json`, `.gitignore`, and the owning Studio roadmap item/ORDER.
- Preserve baseline evidence under ignored `local/kimi/harness-parity/<base>/`.

**Interfaces:** the parity matrix names every required row: shell/themes/responsive layout; projects/files/instructions/memory; parallel sessions/tasks; plan/tool trace; stop/redirect/side questions/subagents; permission modes; editor/diff/comments; terminal; embedded browser; computer use; MCP/connectors/plugins; artifacts; scheduling/Dispatch/notifications; local/hosted environments; GitHub PR/CI review; session export/resume. Each row records fixture evidence, live evidence, host limits, and `NOT_RUN` until executed.

- [ ] Capture the current typed and legacy apps at 1440×900 and 390×852 in both themes; retain the official reference images/observations locally without committing proprietary assets.
- [ ] Install and pin `@playwright/test` exactly; install Playwright Chromium only when no compatible local Chromium exists. Add `test:e2e` and `test:visual` scripts.
- [ ] Write failing Playwright tests for the required shell landmarks, neutral tokens, composer selectors, project/session navigation, pane dock, and every initial workflow. Include assertions rejecting `#111512`, `#1a211b`, and `#8fd3a7` as application theme tokens.
- [ ] Run the new suite against the current app and retain the expected failures and screenshots. Do not convert the old CDP assertions into weaker tests.

### Task 2: Replace the green placeholder with the Claude-style shared workbench

**Files:**
- Replace presentation in `web/studio/src/studio.css` and `src/components/AppShell.tsx`.
- Create focused `ProjectSidebar.tsx`, `SessionToolbar.tsx`, `PaneDock.tsx`, `AgentTranscript.tsx`, and shared `ui/{Icon,Button,Menu,Splitter}.tsx` components.
- Refactor `AgentComposer.tsx`, `WorkspaceEditor.tsx`, `DesktopTools.tsx`, `ModelPicker.tsx`, `SessionList.tsx`, and `StudioApp.tsx` to use that shell without changing their working adapters.
- Test: matching component tests plus `tests/e2e/{shell,visual}.spec.ts`.

**Interfaces:** `AppShell` exposes project navigation, resumable session list, environment/project/permission/model selectors, main transcript, and resizable/collapsible chat/editor/diff/terminal/browser panes. `PaneDock` owns pane layout only; lifecycle and agent state remain authoritative elsewhere.

- [ ] Implement the neutral light/dark token system, compact desktop chrome, project sidebar, session toolbar, message/tool timeline, attachment chips, floating but non-obscuring composer, and pane dock. No generic page-sized textarea or row of bare action buttons remains.
- [ ] Reuse the existing offline editor path for tree/tabs/editing; provide an honest textarea fallback rather than adding a runtime CDN. Turn Changes and Terminal into dockable panels while retaining their standalone accessible routes.
- [ ] Add responsive drawer/mobile panes, keyboard navigation, visible focus, reduced motion, accessible names, and 16px mobile form controls. Preserve all current operations and migration behavior.
- [ ] Run component/typecheck/build plus Playwright. Review every view at both sizes/themes against the reference; establish accepted visual baselines and a `maxDiffPixelRatio` of 0.01 for subsequent unintentional changes. A reviewer must inspect the images, not only the test exit code.

### Task 3: Add the shared harness-agent runtime and versioned client contract

**Files:**
- Create `crates/uor-r4-studio-agent/` with `model.rs`, `runtime.rs`, `events.rs`, `persistence.rs`, `scheduler.rs`, and tests; register it in the workspace.
- Create `src-tauri/src/agent_host.rs` and `src-tauri/src/agent_server.rs`; reuse the existing host implementations for file/Git/terminal/MCP authority.
- Create `web/studio/src/adapters/agentClient.ts` and `src/state/agent.ts`; do not fork the event schema in TypeScript.
- Create loopback model/tool fixtures under `crates/uor-r4-studio-agent/tests/fixtures/` and `web/studio/tests/fixtures/`.

**Interfaces:**
- `ModelApi::stream(ModelRequest) -> Stream<ModelEvent>` supports text, complete tool calls, usage, finish, and error events.
- `AgentRuntime::{create_task, steer, stop, approve, resume, schedule}` emits ordered, versioned `AgentEvent` records keyed by `project_id`, `task_id`, and monotonically increasing sequence.
- Events cover plan updates, assistant deltas, tool request/completion, approval requests, artifact changes, child tasks, completion/error, and usage. `HostApi` exposes only granted file/Git/terminal/browser/computer/MCP operations.
- `AgentClient` exposes the same operations/events over Tauri IPC or authenticated HTTP/SSE; remote mode requires HTTPS and explicit host configuration. Local service remains loopback by default.

- [ ] Test malformed/partial streams, parallel tasks, out-of-order/duplicate events, cancellation, redirection queued at the next safe tool boundary, side questions, child-task fan-out/join, restart recovery, and stale-event rejection before implementing.
- [ ] Implement the event-sourced runtime and object-store snapshots. Model and tool execution are separate from UI state; browser closure does not cancel a service-run task.
- [ ] Implement the OpenAI-compatible adapter against Claude’s documented contract without editing his API work. Exercise contract fixtures now; run the real R4 API contract suite once its tool-call/streaming surface lands. Until that live suite passes, mark integration `NOT_RUN`, never “working.”
- [ ] Run `cargo test -p uor-r4-studio-agent`, desktop Rust tests, TypeScript contract tests, and hostile transport tests.

### Task 4: Deliver Cowork/Code task execution and project workflows in the UI

**Files:**
- Refactor `AgentComposer.tsx`, `AgentTranscript.tsx`, `SessionList.tsx`, `Inspector.tsx`, and `StudioApp.tsx`.
- Create `ProjectHome.tsx`, `TaskTimeline.tsx`, `ApprovalCard.tsx`, `SchedulePanel.tsx`, and `NotificationCenter.tsx`.
- Modify `src/state/agent.ts` and the persistence migration only through versioned readers.

**Interfaces:** one task has its own project folder, transcript, permission mode, model, event cursor, artifacts, and approvals. A project has instructions, files, tasks, and separately viewable/editable/deletable memory. Sessions can run concurrently and be archived/resumed/exported.

- [ ] Implement Chat/Cowork/Code task presentation, project creation from scratch/existing folder, project instructions/files/memory, parallel independent tasks, plan/tool trace, stop, redirect, side question, child-task display, approval handling, and result/artifact review.
- [ ] Implement recurring schedules, persistent Dispatch-style remote assignment to a running desktop service, cross-device resume through the hosted service, and local/web notifications for completion or required approval. Service downtime and unsupported static-preview behavior remain explicit.
- [ ] Verify two tasks can edit disjoint fixtures simultaneously, a redirect changes the next action, stop prevents later tools, approvals cannot be forged by another task, and restart/resume reproduces the exact transcript/event cursor.
- [ ] Run Playwright task/project workflows at desktop and mobile sizes with real runtime fixtures and zero fabricated “agent” text.

### Task 5: Deliver the integrated coding, browser, connector, and review surfaces

**Files:**
- Enhance `WorkspaceEditor.tsx`, `DesktopTools.tsx`, `McpConnections.tsx`, `PaneDock.tsx`, and `src-tauri` host adapters.
- Create `DiffReview.tsx`, `BrowserPane.tsx`, `TerminalPane.tsx`, `ArtifactPreview.tsx`, and `EnvironmentPicker.tsx`.
- Create focused Rust/Playwright fixtures for Git, terminal, browser, computer use, MCP, and CI status.

**Interfaces:** the same agent-accessible and user-accessible tool contract drives chat-initiated actions and visible panes. Every privileged action carries project/task/grant identity and emits an auditable result.

- [ ] Provide side-by-side editor, searchable file tree, dirty/save state, diff comments, terminal output, embedded browser navigation/screenshot/DOM inspection, and artifact preview/download. Browser and computer-use sessions are isolated from the user’s real browser/profile and require per-capability permission with a working stop control.
- [ ] Support explicit user-approved Git branch/commit/push/PR and CI readback through the existing protected host route; no implicit staging, destructive Git, credential exposure, or direct `main` push.
- [ ] Complete MCP connector/plugin discovery and tool invocation through the declared transports, including explicit unsupported states for static web. Computer use and SSH/remote environments are enabled only when their real host adapter and permission checks pass.
- [ ] Run end-to-end coding acceptance: prompt → inspect files → edit → test in terminal → inspect diff → browser preview → revise → explicit PR/CI review. Also run browser/computer/MCP refusal cases and prove no action occurs after cancellation.

### Task 6: Full acceptance, documentation correction, protected delivery, and publication

**Files:**
- Update `web/studio/README.md`, `docs/products/{web-client,desktop-client}.md`, the parity matrix, roadmap item, and ORDER/REPORT evidence.
- Modify runtime staging/publisher only through existing `scripts/studio/**` and `scripts/stage-studio-site.sh`; do not edit `.github/**`.
- Preserve final evidence under ignored `local/kimi/harness-parity/<head>/`.

- [ ] Run `npm test`, `npm run typecheck`, `npm run build`, all plain-JS tests, all existing Studio/worker/page/WASM checks, `cargo test -p uor-r4-studio-agent`, all Tauri default/GUI suites, and `npx playwright test` on the exact deliverable.
- [ ] Repeat the full matrix in the packaged Tauri app and against the hosted service from two isolated browser profiles plus mobile emulation. Verify background completion, cross-device resume, approval notifications, and service restart. A fixture-only row is not a live pass.
- [ ] Run the real Claude-owned R4 API acceptance at its merged exact head and record the API/model artifact identities, request/event counts, errors, and exclusions. Do not claim useful conversation or M7/M8 from harness success.
- [ ] Deliberately reintroduce the green token, a stale-task event, an unapproved write, and a dropped API tool call in separate negative checks; each must fail the appropriate gate.
- [ ] Correct every old “complete,” screenshot, and product claim to match the new evidence, retain the previous reports as history, obtain fresh exact-head review, merge through the protected path, and verify fresh `main`.
- [ ] Publish the reviewed static shell and service package through the existing owner-controlled Studio path. Verify the hosted URL and service package end to end; if no owner-provided remote host exists, mark hosted production deployment `NOT_RUN` rather than calling local-only delivery full web parity.

## Verification — the acceptance bar

- **Visual:** reviewer-inspected screenshots for every view, both themes, 1440×900 and 390×852; approved Playwright baselines catch later drift. The green placeholder cannot return without a red check.
- **Behavioral:** the parity matrix links each row to an executed real workflow and names fixture versus live evidence, denominator, failures, and `NOT_RUN` rows.
- **Security:** out-of-root paths, forged approvals, cross-task events, hostile MCP/browser content, malformed streams, secret redaction, and cancellation all have negative tests.
- **Compatibility:** existing 436 typed/113 plain-JS counts are historical baselines, not targets to game; all current checks must run and pass unless a deliberately replaced assertion has an equivalent or stronger named replacement.
- **Delivery:** exact source/tree identity, protected merge, fresh-main verification, packaged desktop and hosted URL checks. No production publication, signed-release, useful-model, accessibility-certification, or full-parity claim without its corresponding executed evidence.

## Assumptions

- “Clone with equal capability” means the current Claude Desktop Cowork + Code interaction and task workflows with UOR branding and the R4 model, not Anthropic accounts, pricing, proprietary code, or Claude model quality.
- A full hosted version requires a harness agent service in addition to Claude’s model API. That service is harness infrastructure, not a second model API and not a change to Claude’s work. Static Pages remains an explicitly limited shell-only deployment.
- The single-user/local-first security model is retained. Remote access is opt-in HTTPS with host-issued authentication; no public unauthenticated agent endpoint or paid external compute is introduced.
- The installed Claude window was minimized during this read-only investigation, so the reference matrix uses current official Anthropic documentation/screenshots plus the existing recorded local observations. It will be refreshed if the accessible reference changes before visual baseline acceptance.
- Playwright installation is approved as a development dependency by this plan. No installation or product-file edit has occurred during planning.
- The two pre-existing modified roadmap files and generated `target` directories are not ours and will be left untouched. No tests, builds, servers, app launches, or package installations were run during this investigation; current visual findings come from source, retained acceptance screenshots, the live published legacy page, and official reference material.
