# REPORT: neutral cowork workbench recovered in browser and native app, kimi-2

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: fresh `origin/main` `229739cf9` plus preserved acceptance merge `c17a84682`; recovered at `c4d350b8a` after an external cleanup removed the first worktree. Implementation commit: `03efce527` with this evidence report following in the next commit.

## What did not work

- This is not full Claude Cowork/Code parity. There is no persistent multi-step agent runtime, parallel task execution, project memory, scheduling/Dispatch, embedded browser, agent computer-use path, or R4 model API integration. Those rows remain `NOT_RUN` in `docs/products/harness-parity.md`.
- Static GitHub Pages and local static preview cannot provide background execution, scheduling, cross-device resume, or desktop control. The browser pane explicitly requires a harness service. No production publication or hosted production acceptance has run.
- The unsigned development package is not a signed release. `codesign --verify` reports `code has no resources but signature indicates they must be present`, so code-signature verification is not claimed. The packaged resource contract does pass.
- A default `cargo tauri build --bundles app` produced a headless executable with no GUI symbols. `cargo tauri build --bundles app --features gui` is required and is now documented. No default-headless acceptance is claimed.
- The original worktree was removed by an external cleanup before source was committed. The workbench was restored at `c4d350b8a` and committed as `03efce527` before verification continued. No user data was deleted during recovery.
- The filtered roadmap sync built successfully but refused on four unrelated existing defects: `float-kernel-fusion` missing its required section, `infra-desktop-client-tauri` invalid status, and their issue-map mismatches. Those unrelated files were not edited. Existing issue #3569 is used instead of creating a duplicate.
- Initial red acceptance failed seven checks for the missing neutral shell/workflows, then an additional split-pane check. Later visual review exposed a hidden prompt, dark-only light theme, duplicate pane selectors, and a mobile Split no-op. These were fixed in production rather than by weakening old assertions.
- Fresh package attempts first hit `No space left on device`; two screenshot captures also failed with `ENOSPC`. Only regenerable Kimi build caches were cleared (11.3 GiB plus the two residual generated target directories), not source, model artifacts, screenshots, reports, or other labs' outputs. The clean builds and captures then passed.
- No real browser profile, credential, external test endpoint, model training, GPU run, useful-chat result, actual system-quota condition, accessibility certification, or production deployment was used.
- The first local report/policy attempt exposed a missing `Skills used` section and two task ORDERs from the preserved acceptance history. The report section is repaired. The earlier acceptance unit is being delivered separately first; this branch will merge its resulting `main` before its own exact-head review. The full 11 repository gates are green before that merge, not a substitute for the final-head policy check.

## What changed

- Replaced the green placeholder with a neutral Claude-desktop-inspired workbench: project/session navigation, compact transcript and composer, collapsible context/build tasks, and environment/project/permission/model controls.
- Added real chat/editor/diff/terminal/browser pane switching, an accessible splitter, and mobile project drawer/secondary-view transitions. The browser pane explicitly requires a harness service rather than simulating support.
- Added neutral light/dark design tokens, system UI typography, keyboard/focus behavior, reduced motion, responsive layouts, and visible prompt guidance. Existing workflow adapters, model loading, file saving, sessions, MCP, Git/terminal approvals, migration, worker, tokenizer, and `studio_api.js` behavior remain in place.
- Added the separate `org.uor.r4.desktop.cowork` identity so the recovered app cannot inherit the rejected app's restored state. Documented the mandatory `--features gui` build flag.
- Added pinned Playwright 1.65.0 development-only browser acceptance and preserved screenshots, plus focused unit tests for pane splitting, transcript actions, project creation, and explicit theme queries.
- Updated `web/studio/README.md`, both product pages, current state, the serving notebook, the roadmap item, and the parity matrix. Prior reports remain historical rather than being rewritten as parity claims.

## Measurements

- **43/43** Playwright checks pass at 1440×900 and 390×852 in both themes, covering agent/workspace/changes/terminal/models/guide/split, palette/landmarks/selectors, real panes, mobile drawer/split, prompt editing, new project, and independent session resume.
- **441/441** Vitest across 51 files; **113/113** plain-JS Node tests; TypeScript and Vite build pass; **5/5** typed-runtime Python checks pass.
- Static page and runtime page-load pass with zero external requests, raw TypeScript requests, or runtime errors.
- Worker parity passes with `saveRestoreBitIdentical: true`, `midGenCloseResetGuarded: true`, and zero remote requests. Desktop client reports 32 streamed tokens and 38 invoked commands.
- WASM serving audit passes **7/7**, including `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, and stack helpers, with zero floating-point or unclassified instructions and no remote requests.
- The packaged app contract passes with sidecar, catalog, resources, icon, and development-file exclusions. Native executable SHA-256 is `1d7f7900a0546c9e30ad20f5724af81918bf82c05a9f01cc3288180f35e40297`.
- Native GUI symbols are present. Actual native interactions pass: Workspace switches, Split shows chat/editor side by side, and New session creates a real session row. Screenshots: `local/kimi/harness-parity/native-agent.jpg` and `native-split.jpg`.

## Checks run at the head

- `npm test --prefix web/studio`: 441/441.
- `npm run typecheck --prefix web/studio` and `npm run build --prefix web/studio`: pass.
- `node --test web/studio/assets/js/*.test.mjs`: 113/113.
- `npm run test:e2e --prefix web/studio`: 43/43.
- `python3 scripts/tests/test_typed_runtime.py`: 5/5, including the cowork identity regression.
- `node web/studio/verify_studio_page.mjs --strict-router`, `verify_studio_page_load.mjs --strict-router`, `verify_desktop_client.mjs`, and `verify_studio_worker.mjs`: pass.
- `python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`: 7/7.
- `CARGO_TARGET_DIR=<generated-target> cargo tauri build --bundles app --features gui`: pass; packaged contract passes. Code-signature verification is not claimed as noted above.

## Skills used

Owner-reviewed planning, isolated worktrees, red-first component/browser repairs, systematic build/native diagnosis, responsive and accessibility guidance, and verification before completion. Playwright supplies browser acceptance; a fresh exact-head delivery review remains required before merge.

## Modern Web Guidance

- **`css-layout`**: I used grid for the workbench skeleton, flex for toolbars/panes, intrinsic tracks, and logical properties so the layout can reflow instead of relying on fixed panel sizes (“Reach for intrinsic sizing (`min-content`, `max-content`, `fit-content()`) and flexible tracks (`fr`, `minmax()`) before fixed `width`/`height`”).
- **`accessibility`**: I used native buttons/selects/dialogs, named landmarks, visible focus, logical DOM order, and minimum target sizes (“Prefer native HTML elements and attributes to ARIA”).
- **`navigation-drawer`**: I kept a labeled mobile drawer with `aria-controls`/`aria-expanded`, Escape dismissal, and a real secondary-view transition rather than a hidden no-op (“a panel that slides in from the edge of the viewport over the page content”).

## Still open

Fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full agent/runtime parity and the R4 model API integration remain future orders. Hosted production and a signed release are not claimed. The development app and generated target/evidence directories are not committed.

## Plain summary

The green placeholder is gone in the recovered browser and native workbench. The native app now has GUI symbols and a separate identity, and its actual Workspace, Split, and New session interactions work. The result is a measured development shell, not full Cowork parity, a signed product, or a useful-model claim.
