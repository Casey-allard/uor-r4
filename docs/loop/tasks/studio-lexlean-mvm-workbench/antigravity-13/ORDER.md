# ORDER for Removing M1 & Weight Upload, Adding Encrypted Custom AI API Endpoint, and Fixing `create-app` & Studio Internal Controls (`antigravity-13`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `a9d8a35aa` (following Stage 3 `antigravity-12` PR #4194). Removes `m1-discrete-trunk-seed-1` and weight-upload controls from the Foundry Web Studio, adds user-configured OpenAI-compatible Custom AI API Endpoint support (encrypted at rest via `core.wasm` ChaCha20-Poly1305 and gated by Prism R4 routed grant with zero unsolicited external requests on boot/reload), and resolves the live `create-app` (`scaffold/src/vite-env.d.ts` 404) failure plus broken internal controls and desktop-only dead UI across Studio routes.

## Cancel probe

Verify `bash web/studio/build_foundry_studio.sh`, `node web/studio/foundry/studio/tools/shell_parse.mjs`, `node --test web/studio/foundry/studio/tools/command_ownership_registry.test.mjs web/studio/foundry/studio/tools/prism_r4_route.test.mjs web/studio/foundry/studio/tools/chat_provider_routing.test.mjs web/studio/foundry/studio/tools/stage3_os_github_stitch.test.mjs web/studio/foundry/studio/tools/custom_api_endpoint.test.mjs`, `(cd web/studio/foundry/studio/vendor/prism && node hologram.typecheck.mjs)`, `node web/studio/verify_studio_worker.mjs`, `node web/studio/verify_foundry_studio.mjs web/studio/dist`, staged site verification via `scripts/stage-studio-site.sh --typed`, `python3 -m unittest scripts/tests/test_typed_runtime.py scripts/tests/test_tauri_runtime.py scripts/tests/test_release_served_forward_gate.py`, `python3 scripts/audit_zero_matmul_wasm.py --serving web/studio/dist/pkg/uor_r4_integer_bg.wasm`, `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32`, `uor-harness claim-wording`, `uor-harness check --staged`, `uor-harness loop pr-check`, zero `crates/**` / `src-tauri/**` / `.github/**` edits, zero binaries (§3.13), and fresh-context exact-head `VERDICT: APPROVE`.

## Scope

Under D90 (`docs/integration/DECISIONS.md` / `docs/rules/changes.md`), `antigravity-13` edits only:
- `scripts/studio/{runtime-files.sh,catalog.mjs,build-and-publish.sh}`
- `scripts/tests/test_typed_runtime.py`
- `web/studio/foundry/studio/shell/{custom-api-client.mjs,r4.mjs,chat-model-route.mjs,uor-r4-runtime.mjs,r4-agent-loop.mjs,host.js,command-registry-data.mjs}`
- `web/studio/foundry/studio/vendor/prism/{hologram.ModelPicker.tsx,hologram.CustomApiEndpointSettings.tsx,hologram.no-remote-model.mjs,hologram.screens.mjs,vite.shell.config.mts,hologram.typecheck.mjs}`
- `web/studio/foundry/studio/tools/{shell_parse.mjs,command_ownership_registry.test.mjs,prism_r4_route.test.mjs,custom_api_endpoint.test.mjs}`
- `web/studio/{verify_studio_worker.mjs,verify_foundry_studio.mjs}`
- `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-13/{ORDER.md,REPORT.md}`

Zero edits to `crates/**`, `src-tauri/**`, `.github/**`, `web/studio/foundry/studio/shell/r4/spec.json`, or owner-paths.

## Done check

- `create-app` succeeds from Home Build App in both `web/studio/dist` and `scripts/stage-studio-site.sh --typed` output because `scripts/studio/runtime-files.sh` preserves `scaffold/src/vite-env.d.ts` (while still excluding `pkg/*.d.ts` and `pkg-mvm/*.d.ts`) and `--verify` validates every entry in `scaffold/files.json`.
- `m1-discrete-trunk-seed-1` and all weight-upload controls (`uor-r4:load-local-file`, `local-upload`, `<input type="file">`) are removed from catalog generation (`models.json`), runtime, `ModelPicker`, and publish defaults, leaving `uor-r4:qwen3-0.6b-foundry` as the built-in UOR-R4 model (`test_release_served_forward_gate.py` preserved via explicit `--trained-manifest`).
- `custom-api-client.mjs` and `hologram.CustomApiEndpointSettings.tsx` let users add, test, edit, select, and delete OpenAI-compatible AI API endpoints with API keys sealed via `core.wasm` ChaCha20-Poly1305 (`refs["vault:custom-api:endpoints"]`), zero external requests on boot/reload, Prism R4 routed-grant enforcement, and full 16-tool + `<prism-write>` integration in both Home Chat and Studio Build chat.
- Broken internal Studio controls (`preview-app-folder-name`, `rename-app`, `app:get-current-commit-hash`, `distributed-machine:subscribe` for `image_generation`/`github_ops`, `get-app-upgrades`, `appCollections:*`, `add-to-favorite`, `copy-app`, `delete-apps`, `get-app-env-vars`/`set-app-env-vars`, `update-app-commands`, `check-problems`, `list-all-media`, `free-agent-quota:get-status`, `get-templates`, `open-external-url`, `reset-all`, `mcp:*`) work cleanly in-browser, and dead desktop/Pro-only UI buttons (TitleBar window controls, Voice to text Pro, Node/Folder/Auto-update/Agent-Tools desktop settings, Upgrade/Capacitor/Security/Tests panels, "Restart Cloud Sandbox") are removed or replaced cleanly with zero console/toast `unsupportedChannel` or `handlerFailed` errors.
- All unit, worker, typecheck, headless Chrome CDP (local + staged), packaging, ratchet, and harness checks pass cleanly at HEAD.

## Stop and decide if

Stop if `command_ownership_registry.test.mjs`, `prism_r4_route.test.mjs`, `hologram.typecheck.mjs`, `test_typed_runtime.py`, or `test_release_served_forward_gate.py` reveals a required contract outside Scope, or if `ratchet d23`/`d25`/`d32` regresses.
