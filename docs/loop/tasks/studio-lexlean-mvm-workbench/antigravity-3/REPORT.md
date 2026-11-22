# REPORT for `studio-lexlean-mvm-workbench` (`antigravity-3`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939.

## What did not work

- **Clean-`main` pre-publish CDP gate caught missing `/q/pkg/holospaces_web.js` before live push:** After PR #4168 merged into `main` at `d530aa071`, running `scripts/studio/build-and-publish.sh --target gh-pages --trained-manifest scripts/studio/public-m1.json --push --yes` from clean `main` built `468` files in `web/studio/dist/` (one short of `469`) and aborted inside `verify_foundry_studio.mjs` on `HTTP 404` for `/q/pkg/holospaces_web.js`. Nothing was pushed to `gh-pages`.
- **Why committing `holospaces_web.js` into git was rejected:** `web/studio/.gitignore` previously used an unanchored `pkg/` rule that matched `web/studio/foundry/studio/shell/q/pkg/`, so `holospaces_web.js` (127,867 B, companion wasm-bindgen glue loaded by `web/studio/foundry/studio/shell/inference.js`) was omitted from PR #4168 while `holospaces_web_bg.wasm` was already locked in [foundry-assets.lock.json](../../../../../web/studio/foundry-assets.lock.json). Committing `holospaces_web.js` into git would trip Charter §3.2/§3.3 ratchets (`d23`/`d25`) because the upstream `holospaces_web.js` glue documents/exports upstream benchmark symbols (`softmax`, `hologram_matmul_bench`). Pinning `holospaces_web.js` by SHA-256 alongside `holospaces_web_bg.wasm` in `foundry-assets.lock.json` and ignoring it explicitly in `web/studio/.gitignore` supplies the exact upstream glue on build without introducing banned text into git.

## What changed

- **Locked `holospaces_web.js` in [foundry-assets.lock.json](../../../../../web/studio/foundry-assets.lock.json):** Added entry `studio/shell/q/pkg/holospaces_web.js` (`127867` bytes, SHA-256 `b9f8a887de588c5193dc65ca9ce206a70c6df53595b848e0ce0299c2d88a5753`, pinned to `UOR-Foundation/foundry@ea917b8d92974c3d78ebd213eaa1c21fd47666bf`), increasing total locked assets from `50` to `51`.
- **Anchored [web/studio/.gitignore](../../../../../web/studio/.gitignore):** Changed top-level bundle ignore patterns `pkg/`, `pkg-mvm/`, `dist/` to `/pkg/`, `/pkg-mvm/`, `/dist/` and explicitly ignored `foundry/studio/shell/q/pkg/holospaces_web.js` next to `foundry/studio/shell/q/pkg/holospaces_web_bg.wasm`.
- **Added explicit build & publish file guards:** Checked `q/pkg/holospaces_web.js` and `q/pkg/holospaces_web_bg.wasm` at the end of [build_foundry_studio.sh](../../../../../web/studio/build_foundry_studio.sh) and inside the staged-output gate in [build-and-publish.sh](../../../../../scripts/studio/build-and-publish.sh), and updated the asset count in [web/studio/README.md](../../../../../web/studio/README.md) (`51` locked assets).

## Measurements

- `node scripts/studio/fetch-foundry-assets.mjs --seed-dir /tmp/foundry-probe` + `--check`: verified all `51` locked assets (`cached_or_present: 51`, `verified: 51`).
- `git status --short`: confirms `web/studio/foundry/studio/shell/q/pkg/holospaces_web.js` remains untracked/gitignored.
- `bash web/studio/build_foundry_studio.sh`: built `469` files in `web/studio/dist/`, `host.bundle.js` (`98` inputs, `937,689` B, SHA-256 `bc55c586b56df90a163acd61cca606a4489d09f9d011aa24e43e2f17034b36a5`), shell closure `719e0300f5ce133fda55af77f96219689d0c32defdb5d54943a0ffee8da8085e` (`466` files, `468` responses, `52,867,641` B).
- `node web/studio/verify_foundry_studio.mjs web/studio/dist`: `ALL FOUNDRY STUDIO CDP CHECKS PASSED` on both `/` and `/uor-r4/studio/` (`0` failed HTTP responses, `0` uncaught exceptions, `uor-r4:synthetic-seed-5` chat stream `48` chars, RV64IMAC `uname -a` `3145` retired insns, `Terminal` tab + `xtermMounted: true`, CoW reload persistence `#/chat?id=1&appId=1`).

## Checks run at the head

- `node scripts/studio/fetch-foundry-assets.mjs --check`: passed (`verified: 51`).
- `bash web/studio/build_foundry_studio.sh`: passed (`469` dist files).
- `node web/studio/verify_studio_worker.mjs`: passed (`ALL STUDIO WORKER CHECKS PASSED`, `trapSelfTestHits: 6`, `remoteRequests: 0`).
- `node web/studio/verify_foundry_studio.mjs web/studio/dist`: passed (`ALL FOUNDRY STUDIO CDP CHECKS PASSED` on `/` and `/uor-r4/studio/`).
- `cd web/studio/foundry/studio && node tools/check_shell_closure.mjs`: passed (`719e0300f5ce...`, `466` files).
- `python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`: passed (`1..7 ok`, `float=0, unclassified=0`, SHA-256 `6c6dd3e5a67794bbd0b4c0dd7f208139a271ad57005af396f866f0e84bd0dd07`).
- `uor-harness ratchet d23`/`d25`/`d32`: passed (`4451`, `93`, `31` unchanged from baseline).
- `cargo run -q --locked -p uor-r4-claim-wording -- .` & `cargo run -q --locked -p xtask -- links`: passed (`0` broken links in changed files).

## Skills used

- `verification-before-completion`

## Still open

- Merge this follow-up PR into `main`, run `scripts/studio/build-and-publish.sh --target gh-pages --trained-manifest scripts/studio/public-m1.json --push --yes` from clean `main`, and confirm HTTP 200 on all 8 live GitHub Pages URLs.

## Plain summary

After PR #4168 landed on `main`, pre-publish CDP verification blocked deployment because `holospaces_web.js` was excluded by unanchored `pkg/` in `web/studio/.gitignore` and omitted from `web/studio/foundry-assets.lock.json`. Locking `holospaces_web.js` by SHA-256 in `foundry-assets.lock.json` (`51` total locked assets), anchoring `/pkg/`, `/pkg-mvm/`, `/dist/` plus explicitly ignoring `holospaces_web.js` in `web/studio/.gitignore`, and guarding `q/pkg/holospaces_web.js` + `q/pkg/holospaces_web_bg.wasm` in the build and publish scripts fixes clean-`main` builds and Chrome CDP E2E without committing any upstream benchmark symbols or binaries to git.
