# ORDER: mount the real typed Studio application, kimi-52

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0` (original branch base `ab4a5b328`). Owner-approved two-phase Studio goal: Phase A delivers the mounted shared web/desktop harness; Phase B follows only afterward.

## Goal

Convert the Vite library into a real application that mounts `StudioApp` behind `?typed=1`, using the existing model/worker and workflow adapters. Keep the legacy page as the default during this unit, disable legacy MCP auto-boot in the typed preview, and add a real-browser acceptance path. This is development cutover evidence only: no production publication or final lifecycle/migration claim.

## Cancel probe

Before implementation, run the new mount/bootstrap tests and typed page-load scenario. They must fail because `src/main.ts` exports only a library, Vite uses library mode, the typed root is absent, and legacy MCP auto-boot still runs. Classify any failure to execute the browser fixture as instrument, not a product negative.

## Files in scope

- `web/studio/src/main.ts`, `src/bootstrap.ts`, `src/main.test.ts`, `src/bootstrap.test.ts`, and other `web/studio/src/**` files strictly required by the mount.
- `web/studio/index.html`, `web/studio/index.css`, `web/studio/vite.config.ts`, `web/studio/package.json`, and `web/studio/package-lock.json` only if an existing pin needs adjustment; no new dependency.
- `web/studio/assets/js/mcp_client.js` only to suppress legacy auto-boot during the typed preview; its D76 scope advisory is intentional and disclosed.
- `web/studio/verify_studio_page.mjs` and `verify_studio_page_load.mjs` only for an additive typed scenario; no weakened legacy assertion.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, and `docs/roadmap/items/studio-typed-shell-upgrade.md` for actual behavior/scope.
- This task folder, including `PLAN.md`.

No publisher, deployment, Tauri, engine, worker, tokenizer, `studio_api.js`, model artifact, `.github/**`, `Cargo.lock`, real endpoint/credential/browser-data migration, owner-controlled file, or generated `docs/roadmap/issues.json` change. No dependency, second model provider, production publish, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- `src/main.ts` mounts `StudioApp` through a real Vite application build when `?typed=1`; the default `index.html` path retains the legacy UI.
- The preview creates the existing lifecycle store, model-source adapter/worker, and workflow APIs; legacy MCP auto-boot does not create a duplicate panel or request.
- Static/unit tests cover the mount decision and bootstrap, and the real-browser scenario proves one visible typed shell, no duplicate legacy MCP panel, and zero external page-load requests.
- Existing typed/plain-JS suites, typecheck/build, all four legacy checks, report/local-PR checks, and full gates pass at the exact head. Fresh review precedes ready/queue.
- `References #3500`; no production publish, final migration, Tauri acceptance, or model-capability claim is made. Later units complete those gaps.

## Stop and report

Stop before changing an out-of-scope file, weakening a verifier, adding a dependency, or contacting a real endpoint. Stop if the preview would ship accidentally, if legacy/default behavior regresses, or if a failure's root cause is unresolved. Complete only this unit's protected delivery, then continue the approved Phase A queue.
