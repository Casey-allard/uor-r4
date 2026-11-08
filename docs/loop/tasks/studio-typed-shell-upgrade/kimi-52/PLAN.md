# Mounted typed Studio implementation plan

## Context

The live page is a plain-JavaScript application. The typed `src/` tree is only an exported library: `src/main.ts` has no render call and Vite uses library mode. Existing tests pass without proving a mounted product. This unit creates an opt-in real mount while leaving the default page unchanged; subsequent Phase A units finish lifecycle migration, native integration, packaging, desktop acceptance, and publication.

## Approach

Use the existing `index.html` as the Vite application input. Add one typed root and bootstrap it only when `?typed=1` is present. Reuse `ModelSources`, the existing worker, `createLegacyWorkflowApis`, and `createDesktopActions`; do not create a second model/worker/service implementation. Suppress only the legacy MCP auto-mount in this preview so it cannot duplicate `McpConnections` or issue unexpected requests. Add additive browser checks rather than weakening old checks.

## Steps

1. Add red-first bootstrap/mount tests for the opt-in decision, real `render(<StudioApp>)` composition, worker message forwarding, and the no-default-mount boundary. Add the typed browser scenario and verify it fails for the missing root/mount and legacy MCP auto-boot.
2. Convert Vite from library output to the `index.html` application input. Add the typed root and preview-only CSS.
3. Implement `src/bootstrap.ts` to construct the lifecycle store, worker, model-source adapter, workflow APIs, and `StudioApp`; `src/main.ts` invokes it only for `?typed=1` while preserving existing exports. Suppress legacy MCP auto-boot in that preview.
4. Run focused tests, `npm test`, typecheck, `npm run build`, plain-JS tests, and all four legacy checks. Exercise the built preview in a real browser and confirm one visible typed shell, no duplicate MCP, no default mount, and zero external page-load requests.
5. Update actual behavior/scope documentation and the REPORT with exact counts and exclusions. Commit, rerun everything at the exact head, obtain fresh review, and use protected delivery. Do not publish or start native packaging in this unit.

## Verification contract

The old page must still pass its existing static/runtime checks. The new scenario must prove the actual `StudioApp` DOM rather than checking only an export or bundle name. Browser fixtures may use only local files and loopback; no real endpoint or useful-chat claim. The unit is complete only after fresh exact-head approval and protected merge; production remains a later authorized step.
