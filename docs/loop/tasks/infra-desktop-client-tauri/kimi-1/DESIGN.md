# Design: unified Apple-like web and Tauri harness IDE

## Intent and why

The owner likes the dark conversation interface and wants its simplicity applied to a complete desktop harness, not a prettier chat panel hiding the old IDE elsewhere. The product should support ordinary coding-desktop workflows: chat, sessions, local files, Git changes, guarded terminal, models, MCP and settings. Web and Tauri share one interface and behavior wherever the platform permits it.

The current Studio has no active Tauri calls. The older Tauri model worker uses an obsolete byte-mod tokenizer and is not a parity path. The existing desktop backend supplies granted-folder file I/O, Git status, guarded terminal execution, model management and durable memory; expose those capabilities rather than replacing them with mocks.

## Architecture review ruling

The first draft proposed a sidecar-served page plus a second desktop RPC server. Independent architecture review rejected that composition: `frontendDist` and sidecar origins conflict, two backends would split grants/approvals, sidecar chat has no durable engine snapshot, fixed ports add lifecycle risk, and the current CSP/verifiers contradict remote browser calls. It also found that the documented terminal allowlist is absent and that Git/terminal output needs stronger bounds.

The implementation therefore uses **Comparator B: one Tauri asset-origin family, one backend state, Tauri IPC for every privileged host action, and one `uor-chat` loopback sidecar reached only through a host-mediated streaming proxy.** The declared platform aliases are `tauri://localhost`, `http://tauri.localhost` and `https://tauri.localhost`, each accepted only with its exact host, default port and no userinfo; they are not one literal URL scheme. The Tauri GUI adds no second HTTP server or second RPC token. The older opt-in `--serve-studio` CLI mode remains a separately documented legacy host, not part of the packaged GUI path. This is an implementation decision from the review, not an owner-approved two-server design.

## Goals

- One canonical Apple-like design across every view, modal, drawer, menu and responsive breakpoint.
- Obvious workflows: Chat, Files, Changes, Terminal, Connections and Settings; sessions and command palette remain easy to reach.
- One frontend source shared by Pages, other modern browsers and Tauri.
- Exact native chat behavior through the verified `uor-chat` OpenAI-compatible path, mediated by Tauri rather than called from the page origin.
- Real local folder, Git, guarded terminal, local conversation history and MCP workflows with explicit errors and tests.
- A buildable macOS arm64 Tauri app bundle containing the native sidecar and runtime assets, with no committed binaries or secrets.

## Non-goals and anti-goals

- No useful-conversation, visual-understanding, production-readiness or M3/M7/M8 completion claim.
- No second desktop-specific interface, legacy panel skin, fake terminal/MCP button, silent network fallback, paid provider or public API tunnel.
- No unrestricted folder picker, page-enabled terminal, arbitrary page-launched MCP executable, token in logs/storage, or weakening of admission, context, command or artifact checks.
- No claim that the app bundle is one native binary: it is one Tauri app package with a managed sidecar and backend.
- No claim that sidecar chat resumes exact hidden engine state. Conversation text/history persists locally; `UR4SESS1` continuation remains limited to the older desktop-engine path and is not presented as sidecar resume or M3 evidence.

## Approaches considered

1. **Tauri asset origin, Tauri IPC, and host-mediated sidecar proxy (chosen).** One page origin and one state authority; native dialogs are Tauri commands. Only the loopback chat server remains, and the page never calls it directly.
2. **Two loopback servers with separate RPC token.** Rejected by architecture review for origin conflict, split state, CORS/token complexity and lifecycle risk.
3. **Separate desktop-only interface or decorative native buttons.** Rejected because it creates drift and cannot prove native workflows.
4. **Revive the old native model worker.** Rejected because its byte-mod tokenizer is not the current stack-bound BPE/OpenAI path.

## Canonical interface and design system

- Consolidate the current token-based Apple-like styling into one canonical component vocabulary rather than preserving competing legacy skins: system type, black/graphite surfaces, blue accents, generous spacing, rounded cards/bubbles/controls, visible focus, forced-colors support and 16px mobile form text. Preserve element IDs and engine protocols. Update existing theme tests deliberately rather than claiming they are untouched.
- Chat stays a centered conversation with rounded messages and floating composer. Sessions open in a simple drawer rather than a permanent inspector.
- Files has a readable tree, clean tabs and editor, with Open folder as the primary empty action. Changes shows branch/status and a bounded diff. Terminal shows exact command, working directory, consent state and bounded output; disabled/unapproved states are explicit.
- Connections separates local models, custom OpenAI providers and MCP, with the native runner guide visible and duplicate-guide prevention. Settings groups appearance, data/storage and About. The command palette exposes only real actions.
- Every modal, session row menu, build-template menu, diagnostics disclosure, toast and mobile drawer follows the same tokens and focus rules. Desktop and 390px mobile views are inspected per view, not only chat.

## One-origin native/chat architecture

- Tauri serves a staged runtime-only Studio from `frontendDist` under its asset protocol. No sidecar `--studio-root` or second frontend copy is used. `withGlobalTauri` exposes only the declared host commands, and an `on_navigation` guard keeps the main WebView on the asset origin while external links open in the system browser.
- The host reuses the published pinned `uor-chat` sidecar, starts it with `--stack`, `--tokenizer`, `--serve 127.0.0.1:0` and `--token-stdin`, parses its actual bound port while draining stdout/stderr, and waits for authenticated `/v1/models` readiness. Startup failure is explicit; normal app exit kills the child and a random-port/stale-process policy prevents reuse of an orphaned sidecar. No unauthenticated `/healthz` readiness or fixed-port success is assumed.
- The host proxy surface includes `/v1/models` for list/test, `/v1/images/decompose` for bounded RGBA and exact `X-UOR-*` fields, and `/v1/chat/completions` for JSON and SSE. The page never contacts the sidecar directly. Native images and model tests therefore have the same working path as chat rather than controls that fail in Tauri.
- The JS transport seam is a `ReadableStream`-backed `Response` shim: host events carry status, headers and body chunks plus a terminal done/error. Existing SSE/JSON parsing, usage, `uor` metadata and HTTP error bodies remain unchanged. Each request has an ID; AbortController maps to `host_cancel_chat(requestId)`, which closes the sidecar request. Queue bytes are capped and overflow aborts rather than growing without bound. Tests cover SSE, non-streaming JSON, 413/501, cancellation and unchanged direct browser `fetch`.
- The sidecar token remains in host memory and is not written to logs, URLs, localStorage or exported sessions. Any bootstrap provided to the shared model-source layer is an in-memory transport descriptor, not proof of direct browser reachability.
- Conversation text/history persists locally and can be exported. Exact sidecar `UR4SESS1` state export/resume is not claimed or faked.
- Shared grants/approvals/model state use one authority. The mutex is not held across sidecar streaming, Git, terminal or MCP execution; approval/state is checked or removed under lock first. Model, file and durable-store I/O can serialize on that shared state, so every blocking Tauri command runs on a worker rather than the main UI thread. Narrowing those remaining I/O critical sections is follow-up work, not a claim of parallel storage access.

## Native host operations and trusted consent

- Add `tauri-plugin-dialog` under the existing optional `gui` feature for actual host-owned folder selection and terminal confirmation, parented to the main window without exposing page-invokable dialog permissions. This dependency is required because a web-page boolean is not trusted consent; headless/default builds do not inherit GUI dependencies, and unavailable dialogs fail closed.
- A no-argument host command opens the native folder dialog, canonicalizes the selected path, and calls `native_grant_folder` on the same `TauriDesktopState` used by subsequent commands. The page never supplies a grant path or approval boolean. File I/O and durable-store paths retain granted-root and symlink checks.
- Model directories get the same authority: a no-argument native model-folder picker grants the directory, or model listing/loading calls `verify_granted_existing_path` on both the directory and file. Page-supplied paths outside a grant are refused before scan/read, with a focused regression.
- Changes adds bounded `git diff` beside the existing safe status invocation: no pager, no external diff/textconv, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, disabled hooks/fsmonitor, `status -z`, bounded streaming reads, and child termination at cap. No implicit stage, commit, push or destructive Git action.
- Terminal enablement is a no-argument host command that opens native confirmation and only then calls `native_set_terminal_enabled(true)`; the old page-RPC enable command remains refused. Every exact command also requires native confirmation on the same backend. Add the documented exact-program allowlist before exposing the view, refusing interpreters such as `bash`, `sh`, `python3`, `osascript` and `env`, plus relative script execution. Direct execution, metacharacter refusal and granted-folder confinement remain. Add timeout and streaming output caps with explicit truncation, not unbounded `Command::output`.
- Custom OpenAI providers and remote model URLs are browser-only in this unit. Tauri shows an honest “available in the browser build” state instead of a broken direct-fetch button; native chat, local models, local files, Git, terminal and host-mediated MCP still work. No arbitrary-provider proxy or broad CSP exception is added.
- Existing object-store tests and the older desktop-engine durable-memory contract remain intact, but are not represented as sidecar chat resume or M3 qualification.

## MCP architecture

- **Remote Streamable HTTP in the browser:** user-entered URL and optional credential, explicit Connect action, protocol initialize/initialized, `tools/list` and bounded `tools/call`. Support JSON and SSE responses, session/protocol headers, timeout/size limits and actionable CORS/auth/mixed-content errors. Credentials are memory-only and never logged or stored beside other settings.
- **Remote HTTP in Tauri:** the host, not the asset-origin page, performs the request using a direct Rust `reqwest` client with rustls, bounded headers/body/timeouts/redirects and streaming JSON/SSE through the declared channel. This is a disclosed new dependency; no `curl` stdin/argv ambiguity, subprocess, broad CSP or wildcard remote origin is introduced. If the client or transport cannot run, it fails closed.
- **Native stdio in Tauri:** exact executable path, args and granted working directory require native approval before direct spawn. This is host-approved exact-path execution, not a misleading interpreter allowlist. Bound frames/timeouts/process count, drain stderr, handle partial/coalesced frames, serialize JSON-RPC, and clean up children. Implement initialize/initialized, `tools/list` and `tools/call` with explicit errors. Browser stdio is reported unavailable.
- A real loopback HTTP fixture covers CORS/preflight, auth failure, JSON and SSE, `Mcp-Session-Id`, `MCP-Protocol-Version`, timeout and size cap. A compiled stdio fixture covers newline-delimited JSON, partial/coalesced frames, stderr, process exit, timeout and kill/orphan behavior. `tools/list` pagination follows `nextCursor` only up to a declared cap or returns an explicit unsupported error. Credentials disappear on reload.
- MCP names/descriptions/results render as text only, with an executable inert-markup regression. No autonomous tool invocation occurs in this unit.

## Packaging and resource integrity

- One shared allowlist under `scripts/studio/**` stages the runtime-only Studio into `src-tauri/runtime/studio` and is also the Pages publish source. No `src`, tests, package configuration, declarations, verifier, README or `node_modules` enters the app or Pages publish; the older copy-all helper is not used for this delivery.
- `src-tauri/binaries/` is generated and ignored. Use `externalBin: ["binaries/uor-chat"]` with the generated file `uor-chat-aarch64-apple-darwin`, not a double-triple name. Package the published pinned sidecar or pin a newly built artifact in a reviewed manifest; never call a locally computed hash an upstream provenance pin.
- Generate or fetch the verified stack/tokenizer/catalog and include them as resources. `bundle.active` becomes true only after sidecar/resource hashes and the actual package path pass. A real icon is generated from a reviewed vector source rather than shipping a 1×1 placeholder. Nested-binary signing identity/entitlements are decided explicitly; an unsigned local `.app` may be used only for development verification and is not a signed distribution claim.
- The asset-origin CSP remains narrow. Host-mediated native/network actions do not require wildcard remote origins. Measure the actual macOS WebView WASM/module-worker behavior and add only the narrowest necessary `'wasm-unsafe-eval'` and `worker-src 'self'` if required, with a written rationale and Tauri page-load test. Verifiers that previously asserted no native hooks or the old CSP are updated deliberately, not weakened silently.
- `tauri-plugin-dialog` and `reqwest`/rustls are the only disclosed new dependencies in this design; both are optional to the appropriate GUI/native feature and receive focused build/tests. `src-tauri/runtime/` and `src-tauri/binaries/` ignore entries and clean-checkout build ordering are required.

## Verification and delivery

- Failing-first tests cover the canonical view map, one-state consent, exact program allowlist, bounded Git/terminal output, host sidecar proxy/lifecycle, remote JSON/SSE MCP, stdio MCP lifecycle, inert markup and package resource/hash validation.
- Existing Studio, Node, worker/page/WASM, native loopback, vision, xtask and desktop controls remain green except deliberately updated native-hook/CSP/theme contracts, which receive focused replacements.
- Real browser checks cover every view/submenu at desktop and mobile sizes. Real native GUI checks cover streaming, folder choice/file save, Git diff, terminal consent, local conversation restore and both MCP transports using fixtures. The packaged `.app` is launched, not merely compiled.
- Independent architecture and exact-head reviews precede protected merge. Fresh main, Pages runtime identity, public sidecar hashes and final free space are verified. No binary, corpus, secret or development-only file enters source or Pages commits.

## Explicit limits and follow-up

The first package target is macOS arm64; other platforms are not claimed. MCP support is bounded to initialize, tools list/call and declared transports, not every MCP feature or arbitrary server. Advanced agent autonomy, automatic Git writes, signed updater distribution, exact sidecar hidden-state resume and production model quality remain follow-up work.

## Observed desktop reference and GitHub authentication

The owner asked for the installed Claude Code desktop interaction model rather than another invented panel. Inspection of that app showed a compact session toolbar, expandable navigation, Back/Forward, session switching, direct Changes/Terminal/Files controls and a simple composer. The shared Studio now uses those observable patterns with the existing UOR theme and APIs: one `session-toolbar`, a native session selector, section history initialized from the current view, and direct working actions. The old duplicate New chat header action was removed. This copies interaction patterns, not proprietary implementation.

GitHub authentication remains separate and unimplemented. [GitHub's OAuth documentation](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps) supports a device flow with a public OAuth client ID and no client secret, but the ordinary web application flow requires a client secret even when PKCE is used. A configured OAuth app with device flow enabled is therefore the native candidate. Static Pages must not embed a secret. A live unauthenticated CORS probe against `https://github.com/login/device/code` returned HTTP 404 without `Access-Control-Allow-Origin`, so direct browser device authorization is **not established** and no sign-in control is claimed. Static Pages needs a trusted token broker or a changed, measured GitHub CORS contract; the desktop helper may instead use native HTTPS. No credentials, client secrets or real account authorization were used in the probe.
