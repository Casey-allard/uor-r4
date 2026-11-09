---
id: studio-ide-harness
title: Deliver the native-style Studio IDE harness and real connection workflows
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [studio-typed-shell-upgrade]
done_class: host
scope: [Cargo.toml, Cargo.lock, src-tauri/Cargo.toml, src-tauri/Cargo.lock, .gitignore, crates/uor-r4-studio-agent/**, src-tauri/src/**, src-tauri/tauri.conf.json, src-tauri/tests/**, scripts/tests/test_typed_runtime.py, web/studio/src/**, web/studio/tests/**, web/studio/playwright.config.ts, web/studio/package.json, web/studio/package-lock.json, web/studio/index.html, web/studio/index.css, web/studio/README.md, web/studio/verify_*.mjs, web/studio/assets/js/studio_ui.js, web/studio/assets/js/studio_files.js, web/studio/assets/js/model_sources.js, web/studio/assets/js/mcp_client.js, web/studio/assets/js/mcp_client.test.mjs, web/studio/assets/js/desktop_actions.js, web/studio/src/adapters/**, scripts/studio/**, scripts/stage-studio-site.sh, docs/products/harness-parity.md, docs/products/web-client.md, docs/products/desktop-client.md, docs/integration/current-state.md, docs/research/notebook/m4-serving.md, docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md, docs/superpowers/plans/2026-11-02-studio-ide-harness.md, docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md, docs/superpowers/plans/2026-11-08-claude-cowork-parity-recovery.md, docs/superpowers/specs/2026-11-08-interactive-terminal-design.md, docs/superpowers/plans/2026-11-08-interactive-terminal.md, docs/superpowers/specs/2026-11-09-typed-studio-live-publish-design.md, docs/superpowers/plans/2026-11-09-typed-studio-live-publish.md, docs/loop/tasks/studio-ide-harness/**, docs/roadmap/items/studio-ide-harness.md, docs/roadmap/items/studio-typed-shell-upgrade.md, docs/roadmap/issues.json, docs/roadmap/README.md]
---

## Why it matters

The owner rejected the green standalone Studio placeholder and reaffirmed the actual target: a UOR-branded, web-hosted Claude Desktop Cowork/Code-style harness with equivalent task capability, using the separately owned R4 model API. The current typed shell has useful lifecycle and native-host work, but its presentation and single-prompt interaction model do not meet that target. This item's earlier Mac-inspired source and real-connection work remains the foundation; the approved 8 November recovery is the next phase, not a rejection of that history.

## What is missing

Persistent multi-step agent runtime, parallel projects/tasks, approvals and steering, project memory, scheduling/Dispatch, complete side-by-side editor/diff/browser workflows, connector and computer-use workflows, and honest local/hosted capability boundaries remain missing. Claude owns the model API; this item does not edit or reimplement it.

The neutral workbench, Playwright visual acceptance, and persistent PTY are implemented at unsigned development scope. Host-approved `/bin/zsh` sessions accept direct keyboard input and `read` replies, preserve cwd, resize, and close/reap, with bounded input/output/session counts and no Run command button. Workbench/PTY #3890 is merged. The current publication-status ORDER and REPORT are `docs/loop/tasks/studio-ide-harness/kimi-14/`; they follow live-identity correction in `kimi-13`, evidence reconciliation in `kimi-12`, typed publication in `kimi-11`, approval-flow wording in `kimi-10`, record corrections in `kimi-9`, evidence reconciliation in `kimi-8`, and the workspace-origin implementation in `kimi-7`. Static `/studio/` is now published as gh-pages `6540e2717d3a606330dbd141e3642f3f13b87ecf` with Pages run `42668187497` successful and live desktop/mobile verification; rollback head is `ab752d595`. Static web remains explicitly unsupported, and the shared agent runtime, task orchestration, and R4 API adapter are still open.

## Done when

- The green placeholder is replaced by a reviewed neutral Claude-style shell in both themes and at desktop/mobile sizes, with Playwright evidence for every view.
- One versioned agent contract drives the same Rust runtime in Tauri/local and hosted-service modes, with durable event/snapshot persistence and explicit static-preview limits.
- Projects, parallel tasks, plans/tool traces, stop/redirect/side questions/child tasks, approvals, project memory, schedules, cross-device resume, and notifications pass real workflow checks.
- Editor, diff review, guarded terminal, embedded browser, artifacts, MCP/connectors, computer use, environments, and explicit GitHub PR/CI review pass their live host checks and refusal cases.
- The real R4 API contract passes at its merged exact head; no fixture, arithmetic response, or static page is presented as live model or full-parity evidence.
- Existing Studio/Rust checks, fresh exact-head review, protected merge, fresh-main verification, packaged desktop and hosted acceptance, and corrected product records pass. Hosted production is `NOT_RUN` until an owner-provided remote host is verified.

## How to start

Read the earlier `docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md`, then the approved `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md` and `docs/superpowers/plans/2026-11-08-claude-cowork-parity-recovery.md`. Begin with the baseline parity matrix and Playwright shell recovery, then the shared agent runtime and workflows. Use `docs/loop/tasks/studio-ide-harness/` for ORDER/REPORT records.

## Rules that apply

- Do not edit `crates/uor-r4-api/**`, model/engine/worker/tokenizer implementations, Claude-owned files, or the owner's checkout.
- Preserve host-owned folder, terminal, MCP, credential, browser, computer-use, and approval boundaries. No unauthenticated public agent endpoint, destructive test action, real credential, or real browser profile.
- Static GitHub Pages is a limited preview and cannot claim background or desktop capabilities. No second model API or hidden runtime dependency is introduced.
- Playwright is a pinned development-only dependency. Generated assets, build outputs, screenshots, credentials, and local evidence are not committed.
- Every claim links to executed evidence and every unrun row stays `NOT_RUN`. Existing legacy pages and unique prior evidence remain preserved.

## Sources

- #3440, #3500, #3569, #2826, and the owner's 8 November 2026 rejection of the green standalone app and approved parity-recovery plan.
- `web/studio/README.md`, `docs/products/web-client.md`, and the current Claude Desktop/Cowork official documentation linked from the approved design.
