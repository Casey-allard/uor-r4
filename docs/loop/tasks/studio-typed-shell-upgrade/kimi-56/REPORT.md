# REPORT: exercise the real readiness failure and close preview evidence gaps, kimi-56

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `a59e5916ba4683134ac7114226a05fd10b128c58`, tree `56f855629f67b2e4d90512cd10c6e04aff466a30`; original reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. The current merge base after required forward merges is captured in `merge-base.log`. Agent-34 returned CHANGES at the base. The exact final HEAD/tree are captured in `identity.log` and will be posted on PR #3835 after push, not embedded as a self-referential REPORT hash. Before push they are local evidence.

## What did not work

- Agent-34 proved that `main.test.ts` mocked `mountTypedStudio` and rejected immediately. Its fake-timer advancement therefore did not exercise the real bounded readiness wait or prove that incomplete legacy methods reach the actual `main.ts` catch.
- Readiness checked only workspace `tabs` and UI `getSettings`, while the initial composed render also reads the active file, attachments, session list/active ID, and conversation messages. A partial session API could pass the gate and fail later with a different error.
- The active report omitted the `typed_entry.js` scope advisory, and the notebook linked later catch/entry work to the kimi-52 report. Earlier build sizes and seven-request browser readings were not labeled as historical measurements.
- The browser collected only a boolean shell presence and did not assert default raw-TypeScript request absence. A Preact render mock was described as mounting the real component without checking its component identity, and jsdom did not exercise the built error-page CSS.

## What changed

- `main.test.ts` no longer mocks the bootstrap. It supplies real partial legacy APIs, advances the real pending readiness timer, awaits the real entry, and asserts the workflow-timeout message, `hidden=false`, `data-active=true`, and computed display. Duplicate-root cleanup and the pre-existing-root no-default test are retained.
- `waitForWorkflowApis` now checks the methods synchronously consumed by initial rendering. The partial-session regression rejects before Preact render; complete fixtures still verify worker creation/reuse and disposal semantics. The Preact unit test checks the actual `StudioApp` component reference and no longer calls the mocked render an actual DOM mount.
- The runtime verifier requires exactly one typed shell and explicitly rejects raw TypeScript or `src/main` requests on default and typed pages. Both retain zero-external-request and zero-runtime-error assertions and expose those lists/counts.
- A controlled built-page variant suppresses the legacy application entry, supplies partial workflow APIs, and loads the real compiled typed chunk. It proves the real five-second timeout and visible catch DOM with zero mounted shells, zero external/raw-TypeScript requests, and zero runtime errors. No test-only behavior was added to the product.
- The notebook now cites the kimi-52–56 chain, earlier build/request measurements are explicitly historical, the README names the initial-render readiness contract, and both intentional D76 file advisories are disclosed.

## Measurements on the pre-commit repair tree

- Red-first focused result against the old predicate: **2 failures / 5 passes**. The partial-session bootstrap resolved instead of rejecting, and the real entry showed `Worker is not defined` instead of the workflow timeout. After the readiness fix: **7/7 focused tests pass**.
- Complete typed suite: **384/384 tests across 46 files**. Plain-JS suite: **112/112 tests pass**. TypeScript and application build: PASS.
- Final build dimensions: index JS **151.91 kB** (gzip **44.00 kB**), typed main JS **84.14 kB** (gzip **27.51 kB**), MCP chunk **52.80 kB** (gzip **17.56 kB**), index CSS **113.65 kB** (gzip **19.37 kB**), typed CSS **5.19 kB** (gzip **1.46 kB**), worker **27.70 kB**, generated WASM **794.67 kB**, and `index.html` **55.61 kB** (gzip **11.49 kB**). All build outputs are uncommitted.
- Built-browser default: **23 requests**, no external/raw-TypeScript requests, zero runtime errors. Typed preview: **10 requests**, `shellCount=1`, `active=true`, `legacyHidden=true`, `legacyMcp=0`, `typedMcp=1`, `artifactReady=true`, `visible=true`, `typedText=true`, no external/raw-TypeScript requests, zero runtime errors.
- Built controlled failure: **4 requests**, `timeout=true`, `hidden=false`, `active=true`, `visible=true`, `shellCount=0`, actual workflow-timeout text, no external/raw-TypeScript requests, zero runtime errors.
- Default static page, runtime page-load, desktop-client, and worker checks: PASS; desktop streamed 32 tokens and invoked 38 commands, worker emitted the expected 32 tokens with bit-identical save/restore, and their remote-request counts are zero.
- Built-in-browser visual inspection confirms actual mounted DOM. It also shows that the session/actions layout and visual hierarchy are not yet a Claude Code-style production interface. No visual-parity or production-readiness claim is made; that remains later Phase A coherent-shell and acceptance work.

## Checks run at the head

Red and pre-commit green logs are under `local/kimi/studio-mcp-connections/superpowers/checks/kimi-56-red/`; these are not represented as exact-head results. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate and built-browser logs are preserved in the head-specific sibling directory named by the reviewed commit. The intentional `mcp_client.js` and `typed_entry.js` D76 scope advisories are disclosed; shared notebook/FILEMAP advisories and the 4,785-byte ORDER read-path advisory remain disclosed. The roadmap item's canonical scope was not changed.

Agent-34 returned CHANGES for the mocked initialization failure, incomplete readiness, advisory/notebook inconsistencies, and weaker browser/test evidence. The follow-up fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open.

## Skills used

Receiving review, systematic root-cause diagnosis, red-first TDD and honest-test guidance, modern web guidance, project research/delivery rules, and verification before completion. The modern-web search covered asynchronous initialization errors; no returned guide applied to this test/readiness repair. No model compute, paid resource, credential, real user data, real MCP endpoint, dependency, or production publish was used.

## Still open

This is a mounted development preview, not final lifecycle migration, shared/native packaging, desktop acceptance, production publication, or an autonomous/model capability claim. Legacy workflow services remain temporarily bridged. The inspected visual layout is not final Claude Code-style parity. Later Phase A units establish the shared agent/session owner, migrate browser state, complete coherent UI and native/shared workflows, pass acceptance, and publish only after review. The current runtime allowlist must stage `dist` before any typed publication.

## Plain summary

The real readiness failure now reaches the real entry catch in both the unit test and built browser. Partial workflow APIs cannot reach Preact rendering, and the browser rejects duplicate shells and raw-TypeScript requests. The evidence links, advisories, and historical measurements are corrected. The local preview mounts, but its visual layout and the remaining Phase A product/delivery units are still open; no production or useful-chat claim is made.
