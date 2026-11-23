# ORDER for Foundry Web Studio UOR-R4 V13, 16-Tool Webharness & Home Chat (`studio-v13`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `f922fc149` (following D90 `antigravity-10` PR #4189 weight publication and external `Casey-allard/lexlean-r4@wasm-v13-foundry` `127e954f113b1cac398fb494d9002332138afc70`). Replaces dummy Studio models with the freshly LoRA-tuned pure-integer `UORR4V13` Qwen3-0.6B Foundry engine, wires all 16 trained webharness tools and `<prism-write>` application, fixes `create-app` / `create-chat`, and adds on-page Home Chat mode with Studio promotion.

## Cancel probe

Verify `bash web/studio/build_foundry_studio.sh`, `node web/studio/verify_studio_worker.mjs`, `node web/studio/verify_foundry_studio.mjs web/studio/dist`, `python3 -m unittest scripts/tests/test_typed_runtime.py scripts/tests/test_tauri_runtime.py scripts/tests/test_release_served_forward_gate.py`, `python3 scripts/audit_zero_matmul_wasm.py`, `uor-harness ratchet check --all`, `uor-harness check`, `uor-r4-claim-wording`, `uor-harness loop pr-check`, zero `crates/**` / `src-tauri/**` / `.github/**` edits, zero binaries (§3.13), and fresh-context exact-head `VERDICT: APPROVE`.

## Scope

Under D90 (`docs/integration/DECISIONS.md` / `docs/rules/changes.md`), Stage 2 edits only:
- `web/studio/.gitignore`, `web/studio/foundry-assets.lock.json`, `web/studio/build_foundry_studio.sh`, `web/studio/verify_foundry_studio.mjs`
- `web/studio/foundry/studio/shell/{host.js,uor-r4-runtime.mjs,uor-v13-worker.mjs,r4-agent-loop.mjs,chat-model-route.mjs,command-registry-data.mjs,sw.template.js}`
- `web/studio/foundry/studio/vendor/prism/{vite.shell.config.mts,hologram.screens.mjs,hologram.ModelPicker.tsx,hologram.screens/HomePage.tsx}`
- `scripts/studio/{catalog.mjs,build-standards-index.mjs,runtime-files.sh}`, `scripts/stage-studio-site.sh`
- `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-11/{ORDER.md,REPORT.md}`

Zero edits to `crates/**`, `src-tauri/**`, `.github/**`, or owner-paths.

## Done check

- `uor-r4:qwen3-0.6b-foundry` is the primary `models.json` card (`UORR4V13` Stitch 2-of-3 packs with OPFS cache) alongside real M1 `m1-discrete-trunk-seed-1` and local `.uorlut`/`.d11` upload; `synthetic-seed-5` and `foundry:reader` are absent from `models.json` and the UI picker.
- Home Chat mode stays on Home streaming real V13 replies and promotes to Studio via `home-chat:promote-to-studio`; Home Build App mode cleanly executes `create-app` / `first-prompt:commit-creation` and runs all 16 Foundry tools + `<prism-write>` preview updates.
- All build, unit, CDP E2E (using local V13 pack mirror), zero-matmul/ratchet, claim-wording, and `uor-harness loop pr-check` checks pass.

## Stop and decide if

Stop if `shell-cache.js` closure bounds are exceeded, any `crates/**` modification appears necessary, or any ratchet/harness gate regresses.
