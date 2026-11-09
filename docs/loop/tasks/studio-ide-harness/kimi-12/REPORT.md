# REPORT: reconcile typed-publication evidence and live identities, kimi-12

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base after merging current main: `970847d41951b523984575369a83995970394e1b`, tree `6a5890c4f8512aa6315c5e545db34c6de6396b15`; current main `5643a45c5c82bc9b72be3a5ac9df89183e4f2c3f`. Agent-73 returned CHANGES on PR #3897 at d4cb and the verdict is posted. The PR remains draft and unqueued.

## What did not work

- Two README paragraphs still called eb832431a/700756-byte artifacts current although live gh-pages is `ab752d59586c595f9409d8c9ba59026a51d4a613` with 794677-byte artifact `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`.
- Agent-74 subsequently found a third current-identity statement in the README publishing section calling `454bb30fc` live. `kimi-13` replaces it with `ab752d595`, its 794677-byte artifact and `e03b6cd5…`, and labels 454bb30fc/d8e67c66 as history.
- The previous Playwright config inherited a development-dist server command. A stage-root server was actually used, but the retained configuration did not prove it. The prior 45/45 result therefore needed replacement with explicit root evidence rather than a stronger claim.
- `stage-browser-load.json` held a nonzero Chrome-cleanup `ENOTEMPTY` result beside the clean `stage-browser-load.log`, leaving the authoritative result ambiguous.
- The `models.json` byte-identity sentence had no live/local hash in the packet. The earlier REPORT also listed completed final policy/gates as open.
- One separate typed run had all 453 assertions pass but Vitest flagged one unhandled `document is not defined` Inspector teardown error. The complete rerun passed without it; no unrelated Inspector code was changed, and the red log is retained.

## What changed

- README now identifies `ab752d595`, its 794677-byte artifact and earlier eb832431a/233b97fb9 only as history. Product/current-state evidence names the same live identity.
- `stage-playwright-explicit.config.ts` explicitly starts `ruby -run -ehttpd <retained-stage> -p 4174`, and `stage-playwright-explicit.log` records **45/45** against that root. The staged WASM copy passed direct 7/7 audit in `staged-wasm-audit.log`.
- The failed cleanup artifact is now explicitly `stage-browser-load-cleanup-failed.json`; the clean corrected run is `stage-browser-load.log`, with 14 requests and zero external/raw-TypeScript/runtime errors.
- `live-models-identity.txt` contains the executed live/local commands and outputs: both `models.json` files are 4,365 bytes with SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- The earlier REPORT's open list now contains only outstanding review/merge/publication/cleanup work. Current main merged cleanly and changed no publisher/frontend/Tauri/integer/tokenizer/vision input. `application-input-identity.txt` records quiet diff 0 from accepted source 25c63 to this base.

## Measurements

- Explicit staged Playwright **45/45**, direct staged WASM **7/7**, browser-load 14 requests with zero external/raw-TypeScript/runtime errors, and explicit static-terminal unavailability.
- Typed/legacy runtime contracts **15/15**, typed **453/453 across 52 files**, plain-JS **113/113**, default desktop **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**.
- Live `models.json` and local pinned file match byte-for-byte at **4,365 bytes**, SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- Agent-73's CHANGES and all failed evidence are preserved. The public gh-pages ref remains `ab752d595`; no typed publication or remote push is claimed.

## Checks run at the head

- `npm run test:e2e --prefix web/studio -- --config=<stage-playwright-explicit.config.ts>`: **45/45 PASS**, server command `ruby -run -ehttpd <retained-stage> -p 4174` with `reuseExistingServer: false`.
- `python3 scripts/audit_zero_matmul_wasm.py <retained-stage>/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap --build-commit 25c63d42d…`: **7/7 PASS**, artifact SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`.
- `python3 -m unittest scripts.tests.test_typed_runtime scripts.tests.test_tauri_runtime`: **15/15 PASS**. `npm test --prefix web/studio`: **453/453 PASS** on the clean rerun; plain-JS **113/113**. Desktop default/GUI suites pass as above.
- Final report-head local policy and **11/11** gates are rerun after this REPORT is committed. Fresh exact-head review, required checks and protected queue acceptance remain outstanding.
- `cargo xtask roadmap check` previously reported four unrelated existing problems in `float-kernel-fusion.md` and `infra-desktop-client-tauri.md` plus two stale `issues.json` ids. Those files remain untouched and are not represented as passing.

## Skills used

Adversarial review reception, source/evidence identity checking, explicit-root browser verification, native/macOS acceptance, documentation synchronization, artifact preservation, and verification before completion.

## Still open

Final live-identity correction in kimi-13, then fresh exact-head review, required GitHub checks, protected queue acceptance and actual merge, fresh-main typed dry run and explicit staged-browser verification, guarded gh-pages publish, Pages deployment and hard-reloaded public verification, evidence preservation, then branch/worktree/build-output cleanup. Full agent runtime, hosted native workflows, live R4 API and useful-model behavior remain unestablished.

## Plain summary

The evidence now proves the exact staged root, current live identities and catalog identity instead of relying on ambiguous configuration or prose. No application behavior changed during reconciliation. The remaining steps are independent review and protected delivery, not claims that the public site has already updated.
