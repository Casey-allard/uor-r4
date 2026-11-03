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
scope: [web/studio/index.html, web/studio/index.css, web/studio/README.md, web/studio/assets/js/studio_ui.js, web/studio/assets/js/studio_files.js, web/studio/assets/js/model_sources.js, web/studio/assets/js/mcp_client.js, web/studio/assets/js/mcp_client.test.mjs, web/studio/src/adapters/**, docs/products/web-client.md, docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md, docs/superpowers/plans/2026-11-02-studio-ide-harness.md, docs/loop/tasks/studio-ide-harness/**, docs/roadmap/items/studio-ide-harness.md, docs/roadmap/issues.json]
---

## Why it matters

The owner wants the Studio to feel like a professional desktop IDE harness, not a model catalog or a marketing page. The primary workflow should be one clearly labeled experimental UOR Chat connection, ordinary custom API providers, workspace/files, sessions, and real MCP connections. See #3569.

## What is missing

The live page already has a working WASM stack engine, API-provider manager, files, sessions, and an IDE-style shell. Its model-source presentation exposes too much research inventory, and its squared visual treatment does not feel like a native Mac application. There is no OpenAI-compatible `uor-chat` HTTP service and no MCP client or desktop stdio bridge. The current trained M1 artifact is next-token prediction, not a conversational product.

## Done when

- The Studio presents a restrained Mac-inspired interface in dark/light themes and desktop/mobile layouts, with accessible controls, truthful readiness, and preserved non-model workflows.
- One **UOR Chat — experimental** source uses the existing supported stack/WASM path without claiming a trained conversational product. Custom OpenAI-compatible providers remain available. Research artifacts and next-token diagnostics do not dominate the connection experience.
- A later unit adds real remote MCP initialize/tool-list/tool-call behavior with explicit protocol, CORS, authentication, privacy, and failure handling. No simulated tools, successful connections, or autonomous tool loops are claimed. Desktop stdio remains a separate host-bridge unit unless an existing verified path supports it.
- Tests cover actual behavior, including unavailable connections and failures. Built-in-browser checks exercise the source page without real credentials, user files, paid services, or a model request.
- Repository gates, claim wording, fresh exact-head reviews, green GitHub checks, and protected queue delivery pass. Production publication remains a separate owner decision.

## How to start

Use `docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md` and `docs/loop/tasks/studio-ide-harness/kimi-1/ORDER.md`. First deliver the Mac-inspired source experience and simplified UOR/API connections. Do not add a decorative MCP panel before its real client unit, change the engine/worker/tokenizer/model artifacts, or touch Claude's active model work.

## Rules that apply

- Preserve existing engine, worker, tokenizer, model artifacts, provider credentials, user data, and non-model workflows.
- No new runtime dependency is needed for the visual/source unit or the small remote JSON-RPC client. Do not invent a UOR HTTP API or claim the native CLI is one.
- Keep remote HTTP MCP separate from desktop stdio. Do not execute arbitrary commands or weaken host-owned approval and folder boundaries.
- #3500 stays open for typed-library parity/migration/cutover. This item references it without changing its acceptance scope.

## Sources

- #3500, #3569, and #3440
- `web/studio/README.md` and `docs/products/web-client.md`
- `docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md` and `docs/loop/tasks/studio-ide-harness/kimi-1/ORDER.md`
