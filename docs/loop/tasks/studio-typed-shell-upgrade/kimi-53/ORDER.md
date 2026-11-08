# ORDER: make the mounted Studio visible and the evidence truthful, kimi-53

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `d620d5536cc13f0ab44bbda193368e7cf6d1df2a`, tree `e1e4c9021e963b387c48c3da44f709405db0b4ac`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Prior agent-31 review returned CHANGES and blocked push/queue.

## Finished steps

- kimi-52 converted the library into a mounted Vite preview, reusing the model worker, waiting for workflow methods, staging generated artifacts, and suppressing legacy MCP auto-boot.
- Exact-head logs show 382/382 typed, 112/112 plain-JS, all four legacy checks, and 11/11 full gates. Those checks checked DOM state but not computed visibility.

## Goal

Unhide the typed root on successful mount and initialization error, hide it on dispose, prove computed visibility and typed-only text in the real browser, add a direct no-default-mount assertion, and correct exact-head/base and publication evidence. Do not change production or desktop packaging in this repair.

## Cancel probe

Before source changes, assert computed root visibility and typed-only text in the built typed scenario. It must fail because `hidden` and the global `!important` rule leave the root invisible even though DOM and model state pass. Also require a direct no-default-mount test and REPORT/base checks that currently fail or are absent. Classify stale-dist results separately from source defects.

## Files in scope

- `web/studio/src/bootstrap.ts`, `src/bootstrap.test.ts`, `src/main.ts`, `src/main.test.ts`.
- `web/studio/index.html` only for preview visibility/error behavior.
- `web/studio/verify_studio_page_load.mjs` only for computed visibility and typed-only text assertions; no weakened check.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, and `docs/roadmap/items/studio-typed-shell-upgrade.md` only to narrow preview/publication claims.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-52/ORDER.md` and `REPORT.md` only for actual merge-base identity and executed-check evidence.
- This `kimi-53` repair folder.

No publisher, deployment, Tauri, engine, worker, tokenizer, `studio_api.js`, model artifact, `.github/**`, `Cargo.lock`, real endpoint/credential/browser-data migration, owner-controlled file, generated `docs/roadmap/issues.json`, dependency, second model provider, production publish, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Mount and error paths set `root.hidden = false`; dispose restores `root.hidden = true`. The root's computed display is not `none`, and typed-only text is visible while the legacy app is hidden.
- A direct test proves no mount/default mutation without `typed=1`; incomplete initialization still refuses render and leaves an explicit visible error path. Existing worker reuse is preserved.
- The REPORT records the actual merge base and all executed checks. Exact final HEAD/tree are captured in `identity.log` and posted on PR after push, not embedded as a self-referential REPORT hash. Publication wording says only that the default source UI remains legacy and no publish ran; it does not imply current staging can ship the typed app.
- Focused/full Studio checks, plain-JS, all four legacy checks, report/local-PR checks, and full gates pass at the new exact head. Fresh exact-head review approves before ready/queue.
- PR keeps `References #3500`; no production publish, final migration, Tauri acceptance, or model-capability claim.

## Stop and report

Stop before changing an out-of-scope file, weakening a test, or hiding a visibility failure behind DOM presence. Do not publish or start native packaging in this repair. Stop after fresh approval and protected delivery of this unit, then continue Phase A.
