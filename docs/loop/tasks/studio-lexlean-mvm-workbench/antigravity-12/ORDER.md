# ORDER for Foundry Web Studio Linux 6.18.7 VM, Stitch Mesh & Browser-Native GitHub (`studio-os-github-stitch`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `7b74734fc` (following D90 `antigravity-11` PR #4192). Completes Stage 3 of the Foundry Web Studio overhaul: real Linux 6.18.7 VM (`afflom/stitch` v86 + bidirectional `/workspace` 9p <-> IndexedDB sync + Linux/RV64IMAC MVM toggle), Stitch `stitch.signal/1` Ed25519 opt-in WebRTC + `stitch://` fetch + NE2000 LAN + `afflom/me` `app.json` & `?embed=me` `uor-workspace-bridge/1` postMessage bridge, real browser-native GitHub REST v3 / Git Data API + `core.wasm` ChaCha20-Poly1305 encrypted-at-rest vault + intuitive Settings/Connector UI + all 5 `github_*` AI tools, and full 8-check headless Chrome CDP verification.

## Cancel probe

Verify `bash web/studio/build_foundry_studio.sh`, `node web/studio/verify_foundry_studio.mjs web/studio/dist`, `node web/studio/verify_studio_worker.mjs`, `node web/studio/foundry/studio/tools/shell_parse.mjs`, `node --test web/studio/foundry/studio/tools/command_ownership_registry.test.mjs web/studio/foundry/studio/tools/prism_r4_route.test.mjs web/studio/foundry/studio/tools/stage3_os_github_stitch.test.mjs`, `(cd web/studio/foundry/studio/vendor/prism && node hologram.typecheck.mjs)`, `python3 -m unittest scripts/tests/test_typed_runtime.py scripts/tests/test_tauri_runtime.py scripts/tests/test_release_served_forward_gate.py`, `python3 scripts/audit_zero_matmul_wasm.py --serving web/studio/dist/pkg/uor_r4_integer_bg.wasm`, `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32`, `uor-harness claim-wording`, `uor-harness check --staged`, `uor-harness loop pr-check`, zero `crates/**` / `src-tauri/**` / `.github/**` edits, zero binaries (§3.13), and fresh-context exact-head `VERDICT: APPROVE`.

## Scope

Under D90 (`docs/integration/DECISIONS.md` / `docs/rules/changes.md`), Stage 3 edits only:
- `web/studio/foundry/studio/shell/{linux-vm-host.mjs,stitch-network.mjs,github-vault.mjs,github-client.mjs,app.json,host.js,command-registry-data.mjs}`
- `web/studio/foundry/studio/vendor/prism/{hologram.GitHubIntegration.tsx,hologram.GitHubConnector.tsx,hologram.screens.mjs,vite.shell.config.mts,hologram.typecheck.mjs}`
- `web/studio/foundry/studio/tools/{shell_parse.mjs,command_ownership_registry.test.mjs,stage3_os_github_stitch.test.mjs}`
- `web/studio/{build_foundry_studio.sh,verify_foundry_studio.mjs}`
- `scripts/studio/runtime-files.sh`
- `scripts/tests/test_typed_runtime.py`
- `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-12/{ORDER.md,REPORT.md}`

Zero edits to `crates/**`, `src-tauri/**`, `.github/**`, or owner-paths.

## Done check

- Terminal & `linux_exec` boot real Linux 6.18.7 (`v86` + `buildroot-bzimage.bin`) over `ttyS0` with bidirectional `/workspace` 9p <-> IndexedDB sync, while preserving `RV64IMAC MVM` via a toggle in `TerminalEscapeBanner`.
- `stitch-network.mjs` provides opt-in Ed25519 `stitch.signal/1` peer transport (`stitch://` fetch, NE2000 bridge) with zero unprompted external network traffic on boot/reload, alongside `app.json` and `?embed=me` (`uor-workspace-bridge/1`) postMessage integration.
- `github-vault.mjs` seals GitHub PATs via `core.wasm` ChaCha20-Poly1305 in IndexedDB (`refs["vault:github:credentials"]`) with zero plaintext leakage; `github-client.mjs`, `hologram.GitHubIntegration.tsx`, `hologram.GitHubConnector.tsx`, IPC handlers, and all 5 `github_*` AI tools support real connect, clone, diff, pull, and CAS push (`force: false`).
- All 8 headless Chrome CDP checks and unit/typecheck/packaging/ratchet/harness checks pass cleanly at HEAD.

## Stop and decide if

Stop if `buildroot-bzimage.bin` fails to mount `host9p` or start a shell in headless Chrome, `manifest.json` total closure approaches 128 MiB, any `crates/**` modification appears necessary, or any ratchet/harness gate regresses.
