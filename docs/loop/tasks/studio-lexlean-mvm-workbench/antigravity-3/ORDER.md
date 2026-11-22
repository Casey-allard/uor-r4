# ORDER for locking `holospaces_web.js` in `foundry-assets.lock.json` and anchoring Studio `.gitignore`

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `d530aa071`. Clean-`main` pre-publish verification (`verify_foundry_studio.mjs`) after PR #4168 caught a 404 on `/q/pkg/holospaces_web.js`: unanchored `pkg/` in `web/studio/.gitignore` ignored `web/studio/foundry/studio/shell/q/pkg/` while `holospaces_web.js` (which contains upstream `softmax`/`hologram_matmul_bench` benchmark strings and therefore must NOT be committed into git) was absent from `web/studio/foundry-assets.lock.json`.

## Cancel probe

Verify zero committed binaries (§3.13), `holospaces_web.js` remains untracked/gitignored in git, D23/D25/D32 ratchets (`0` added), `fetch-foundry-assets.mjs --check` (`51`), `build_foundry_studio.sh`, `verify_foundry_studio.mjs`, `check_shell_closure.mjs`, `audit_zero_matmul_wasm.py`, `uor-r4-claim-wording`, `xtask links`, and `uor-harness loop pr-check`. Cancel immediately if any forbidden op or unguarded `holospaces_web.js` source enters git.

## Scope

Inside `web/studio/foundry-assets.lock.json`, `web/studio/.gitignore`, `web/studio/build_foundry_studio.sh`, `web/studio/README.md`, `scripts/studio/build-and-publish.sh`, and `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-3/**`:
- Pin `studio/shell/q/pkg/holospaces_web.js` (127,867 B, SHA-256 `b9f8a887de588c5193dc65ca9ce206a70c6df53595b848e0ce0299c2d88a5753`) in `web/studio/foundry-assets.lock.json` alongside `holospaces_web_bg.wasm` (51 locked assets total).
- Anchor `/pkg/`, `/pkg-mvm/`, `/dist/` in `web/studio/.gitignore` and explicitly ignore `foundry/studio/shell/q/pkg/holospaces_web.js`.
- Guard `q/pkg/holospaces_web.js` and `q/pkg/holospaces_web_bg.wasm` in `web/studio/build_foundry_studio.sh` and `scripts/studio/build-and-publish.sh`, and update the locked-asset count note in `web/studio/README.md`.

## Done check

- `fetch-foundry-assets.mjs --check` (`51` verified), `build_foundry_studio.sh` (`469` files in `web/studio/dist`), `verify_studio_worker.mjs`, Chrome CDP `verify_foundry_studio.mjs` (`/` and `/uor-r4/studio/`), `check_shell_closure.mjs`, `audit_zero_matmul_wasm.py`, ratchets `d23`/`d25`/`d32`, `uor-r4-claim-wording`, `xtask links`, `uor-harness check --staged`, and `uor-harness loop pr-check` all pass.

## Stop and decide if

Stop if `fetch-foundry-assets.mjs` fails SHA verification, `verify_foundry_studio.mjs` hits any 404/network/exception on `/` or `/uor-r4/studio/`, or any ratchet (`d23`/`d25`/`d32`) increases.
