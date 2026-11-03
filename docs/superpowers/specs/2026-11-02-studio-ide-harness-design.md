# Studio native-style IDE harness and connections

Owner direction: simplify the client around one experimental UOR Chat source, keep ordinary custom API providers, support real MCP connections rather than decorative configuration, and make the interface feel like a professional Mac desktop IDE. Issue #3569; references #3500 without closing it.

## Intent and user experience

The Studio should feel immediately useful for coding: sessions and navigation on the left, a calm editor/agent workspace in the center, context and inspection on the right, and compact native-style command/status chrome. Connections are a means to work, not the product's landing-page inventory.

The visual language is Mac-inspired rather than a platform clone: system font stack, translucent-looking but opaque readable panels, 8–14 px corner radii, subtle borders and shadows, compact controls, clear focus rings, generous line height, and restrained accent use. Dark and light themes retain semantic contrast and reduced-motion behavior.

## Connection model

- **UOR Chat — experimental** is the primary source. It calls the existing supported non-discrete stack/WASM path directly; no new UOR HTTP service is invented. Loading, ready, unsupported, and unavailable states come from actual capabilities. A synthetic stand-in is labeled synthetic, and no trained conversational product is claimed.
- **Custom API providers** use the existing validated OpenAI-compatible manager. Selection does not imply reachability; testing and sending are explicit user actions with privacy warnings.
- **Artifact diagnostics** such as M1 next-token prediction, local files, and research artifacts move out of the primary connection flow into a collapsed developer/diagnostic area or a separate diagnostic view. Their code and evidence are preserved.
- **MCP servers** arrive only with a real remote client. Configuration shows discovered tools and errors from the server. A tool result is attached to Agent only after an explicit user action; no autonomous loop or fabricated discovery exists.

## Confirmed technical boundaries

The browser already compiles and loads the integer engine as WASM. Native `uor-chat` is a CLI, not an OpenAI-compatible server, and discrete text chat is unavailable. The desktop has a native Rust engine and a token-protected loopback invoke bridge, not an MCP bridge. There is no current MCP client. M1 is next-token prediction; no useful conversation is established.

Browser MCP support is remote HTTP only. It requires server CORS permission and cannot spawn stdio processes. Desktop stdio needs a separate host-owned bridge with command approval, environment scoping, and lifecycle controls; it is not silently emulated in the browser.

## Approaches

1. **Refine the existing live source page (chosen).** Reuse its working files, sessions, source manager, and WASM worker. Restyle and simplify presentation, then add a small dependency-free remote JSON-RPC client in a later unit. This preserves working behavior and avoids a second state owner.
2. **Mount the typed component library now.** Rejected for these units: migration and parity are already a separate #3500 dependency and would make the visual/MCP work much larger.
3. **Build a UOR HTTP serving API and use it everywhere.** Rejected: the existing direct WASM path already serves the experimental stack locally. A new API would add a backend without improving the requested experience. Native `uor-chat` cannot be presented as that API.

## Delivery units

### Unit 1: Mac-inspired IDE and simplified UOR/API sources

Restyle the existing shell, simplify the Models/Connections presentation around UOR Chat and custom providers, demote research inventory to diagnostics, and preserve truthful readiness and all non-model workflows. Add focused source/page tests and desktop/mobile browser checks. Do not add an MCP placeholder that looks operational.

### Unit 2: real remote MCP connections

Implement dependency-free JSON-RPC initialize, `tools/list`, and `tools/call` using **Streamable HTTP**, with the current protocol revision re-checked at plan time (the builder check verified `2025-11-25`). Use one endpoint and one POST per message; accept and parse both `application/json` and `text/event-stream` responses. Send `notifications/initialized`, echo `MCP-Protocol-Version`, retain `MCP-Session-Id` only in memory, and reinitialize on HTTP 404. Support first-page text tools. Explicitly omit the standalone GET stream, resumability, server-initiated requests, and legacy 2024-11-05 HTTP+SSE transport. Local fixtures prove protocol behavior, not live CORS/SSE interoperability; any broader claim needs a separate no-auth live-server check.

Support optional user-supplied headers, including `Authorization`, using the existing provider-configuration storage pattern without logging secret values. On 401/403, say that the server requires an authorization flow this client does not implement. OAuth, dynamic registration, PKCE, redirects, and refresh are outside this unit.

Use a raw JSON arguments editor prefilled with `{}`, show the tool's `inputSchema` in a details block, validate JSON and required fields, and send exactly the entered arguments. Preview text results and attach them explicitly through the existing `window.__studioUi.addAttachment({ name, content, source })` context-chip hook. Summarize non-text results and refuse to attach them with a clear reason. Do not generate schema forms, add a message role, or run a model/tool loop.

Units 1–2 are browser-page units. Do not change the desktop CSP or verifier. When `window.__TAURI__` is present, show remote MCP as unavailable and identify the later desktop bridge; do not let requests fail opaquely against the existing desktop network policy.

### Later: desktop stdio bridge

Evaluate a host-owned bridge using the existing native desktop security model. It needs process lifecycle, command/environment approval, folder boundaries, and real integration tests; it is not part of Units 1–2.

## Verification and anti-goals

No trained-chat claim, UOR HTTP API claim, decorative MCP success, real external calls during tests, new runtime dependency, model/engine/worker/tokenizer/artifact changes, production publish, real browser-data migration, or typed-library cutover. Keep #3500 and its acceptance boundaries intact.

The owner has directed the native-style/simplified-connection approach and asked the lead to determine the WASM/service details. The split above is the lead's implementation sequencing, not a new model acceptance bar.
