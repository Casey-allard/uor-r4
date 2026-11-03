# ORDER: Mac-inspired Studio IDE and simplified UOR/API connections

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), with follow-up [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569), following the owner's explicit native-style IDE and simplified-connection direction. Base: `85f634af30809c998cc3dbd830366c85be214934`. Both issues remain open.

## Goal

Make the existing Studio source page feel like a professional Mac desktop IDE and simplify its primary connection experience around one **UOR Chat — experimental** source plus ordinary custom API providers. Preserve actual capabilities and all working non-model workflows.

## Design

[Studio native-style IDE harness and connections](../../../../superpowers/specs/2026-11-02-studio-ide-harness-design.md). The owner directed the approach and asked the lead to resolve the WASM/service details. Unit 2 will implement real remote MCP behavior separately; this order does not add a decorative MCP panel.

## Files in scope

- `web/studio/index.html`, `web/studio/index.css`, and `web/studio/README.md`.
- Narrow source/navigation presentation in `web/studio/assets/js/studio_ui.js` and focused actual-page tests under `web/studio/src/adapters/**`.
- Fresh-review repairs only: the moved URL input lookup and the local-file input's keyboard-focus class in `web/studio/assets/js/model_sources.js`, plus the native tree attachment in `web/studio/assets/js/studio_files.js`, with focused actual-module tests. The local-file change keeps the control keyboard-focusable while its visible label receives the focus outline; overlay/accessibility state otherwise remains in the already scoped HTML/CSS/`studio_ui.js`. No provider behavior or network call may change.
- This task folder, the design, an implementation plan if needed, `docs/products/web-client.md`, and the owning roadmap item/generated issue data if the normal generator permits it.

## Done when

- The existing page has a coherent Mac-inspired visual system with softer radii, compact native-style controls, readable system typography, subtle panel treatment, and visible keyboard focus in both themes and desktop/mobile layouts.
- One primary UOR Chat experimental source uses the existing supported non-discrete stack/WASM path, with truthful loading/ready/unavailable states and no trained-chat claim. Custom API providers remain usable and do not imply reachability.
- M1/research/local artifact tools no longer dominate the primary connection flow; they remain available only as clearly labeled diagnostics. No engine, worker, tokenizer, API-client, or model-artifact change is made.
- Focused failing-first tests and complete Studio checks pass. Built-in-browser checks exercise all four views, source readiness, custom-provider presentation, both themes, and desktop/mobile without real credentials/files, external endpoints, or a model request.
- Repository gates, claim wording, local policy, fresh exact-head review, green GitHub checks, and protected queue delivery pass. The PR says `References #3569` and `References #3500`; both issues remain open for their broader units.

## Stop and report conditions

Stop before changing files outside this scope, adding MCP behavior or a nonfunctional MCP placeholder, mounting the typed library, adding dependencies, editing the engine/worker/tokenizer/`studio_api.js`/model artifacts, changing publisher/deployment/`.github/**`, touching Claude's work or real browser data, or claiming trained conversation, a UOR HTTP service, MCP connectivity, production publication, or full typed parity. Merge forward from fresh `main` before implementation if the prerequisite Studio deliveries have landed; never force-push.
