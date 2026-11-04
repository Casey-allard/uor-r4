# Design: conversation-first Studio and native connectivity

## Intent

Replace the panel-heavy IDE first impression with a calm, modern chat window that works immediately. The owner explicitly rejects the old boxy interface and unreachable API. The work has two independent parts: the conversation experience and a way to reach the native engine without pretending GitHub Pages is a server.

## Approaches considered

1. **Offline-first conversation plus runner-served local native page (chosen).** Default to the bundled synthetic demo, with model choices and diagnostics secondary. The native runner serves the same UI and authenticated API on loopback only. It keeps execution on the user's device and requires no paid service or public tunnel.
2. **Custom API first.** Keep the current provider cards and let users configure remote APIs. This preserves the confusing first impression and does not solve unavailable local endpoints, so it is rejected as the default, not removed as an advanced option.
3. **Direct HTTP loopback with CORS.** This cannot make an HTTPS page's HTTP loopback request safe or reliable. It is rejected as a fix. Runner-served HTTP local UI remains valid because it is same-origin.
4. **Remote hosted model API.** GitHub Pages cannot execute a native API. A separate remote host would add cost, trust and model-serving obligations not requested here, so it is not adopted.

## Conversation experience

- One centered conversation column; no default navigation or inspector sidebars. Model/workspace/settings remain accessible without dominating the chat.
- A short greeting and two or three real actions, not a setup wizard. First-run loads the bundled demo and labels it synthetic evidence, never a useful conversational model.
- System type, black/graphite surfaces, blue accents, rounded message bubbles and a floating rounded composer. Dense telemetry and build/workspace setup move to secondary views.
- Desktop and mobile keep the same simple structure, keyboard behavior, session history, attachments and exact metrics.

## Connectivity

- **Offline default:** bundled demo uses the existing integer WASM path and needs no provider or token.
- **Native local page:** the runner downloads and verifies public synthetic demo/tokenizer and binary artifacts, starts the existing authenticated loopback, and serves the Studio on `127.0.0.1`. No CORS or plugin is needed on that page.
- **Static Pages:** only the bundled offline demo and user-configured external APIs are offered. No public native tunnel is provided: the current bootstrap would disclose its bearer token, and the tunnel hostname would fail admission. A future private-overlay mode needs a separate admission review.
- Provider errors distinguish missing runner, insecure loopback, authentication failure and unsupported endpoint. Unsupported families retain explicit errors.

## Evidence and limits

- Failing-first UI/runner regressions, existing Studio/native checks, desktop/mobile visual inspection, real offline generation and real native OpenAI streaming over runner-served loopback with exact usage.
- No claim of useful chat, model quality, visual understanding, production readiness, M7/M8 completion, or GitHub Pages API execution. The public model's current responses are not presented as capable conversation.
- Protected source PR, independent review, fresh-main verification, runtime-only Pages publish, and cleanup of temporary binaries/worktrees afterward.
