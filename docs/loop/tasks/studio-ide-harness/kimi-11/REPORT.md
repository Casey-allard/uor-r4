# REPORT: publish the neutral workbench through the typed Studio path, kimi-11

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `59494646cdae3e2f8874939680324ed8cd5affd5`, tree `907f341c57da00fb03b839da54d2fb2985bcd3a7`. ORDER `c2c403d86213e7cdcd549aadef6faddb6df46009`; typed staging `26b31fb47bd4cb8fa4ebf73742f638cc80181589`; publisher wiring `25c63d42d58275c8fb4836f8c6defe6fd6a7c5f2`. PR #3890 is merged. This unit's protected review/merge and actual typed publication remain open.

## What did not work

- Public `/studio/` remained a 7 November legacy build while later Studio work merged to main. Live `gh-pages` is `ab752d595`, not the older `eb832431a`; its current artifact is 794,677 bytes with SHA-256 `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`.
- The manual publisher staged only the legacy source. Running it would have preserved the old conversation page rather than the neutral workbench; automatic deployment is intentionally paused.
- The first typed-staging regression failed with `stage-studio-site: --typed/index.html is missing`; the publisher-help regression had no `--typed`. The first dry-run assertion then expected single-space mode formatting although the publisher aligns labels with four spaces. Only that assertion was corrected; production behavior was not changed to fit it.
- Fresh-main preparation initially invoked Vite before generated WASM artifacts existed and desktop tests before the pinned sidecar existed. Canonical preparation generated both and reproduced WASM SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`; the interactive verifier also required its declared target paths. These were invocation/setup errors, not source regressions.
- The inline browser trap first had launcher/path mistakes and a Chrome cleanup race; its corrected run passed with the normal 500 ms exit wait. Local policy at 25c63 passed scope and ORDER-first but failed only because this REPORT was not yet present.
- During native acceptance, the first app window disappeared before `exit` completed; cause is unknown and the app log was empty. Exact test app/sidecar PIDs were terminated, then a clean instance passed the second-open dialogs/cancellation and quit normally. Fresh native exit code 0 is therefore **not claimed** here; c663 retains the earlier exit-0 evidence.

## What changed

- `scripts/stage-studio-site.sh --typed` selects `web/studio/dist`, uses typed copy/verify and retains the existing catalog, trained/public-model checks and Pages limits.
- `scripts/studio/build-and-publish.sh --typed` generates artifacts before Vite, runs `npm ci` and `npm run build`, reports the mode, and forwards typed staging. Legacy mode, dry-run default, main-only live guard and rollback behavior remain unchanged.
- Red-first regressions prove typed staging and a real publisher dry run. Product/current-state/roadmap records distinguish implemented dry-run capability from actual live publication and correct the current gh-pages/artifact identities.
- Static web still explicitly says `Interactive PTY is unavailable in this browser build.` No hosted terminal, background agent service, full Cowork parity or live R4 API contract is claimed.

## Measurements

- **15/15** typed/legacy runtime contracts pass, including real typed staging and publisher dry-run behavior. Full default desktop **11/3/21/4/7/13/8** and GUI **13/3/25/4/8/13/9** pass. Typed **453/453 across 52 files**, plain-JS **113/113**, staged Playwright **45/45**, and repository gates **11/11 GREEN** pass.
- Full dry-run source `25c63d42d58275c8fb4836f8c6defe6fd6a7c5f2` produced local publish commit `1684785face01e02d4f79a34e49c0d1b954d0f40`. `git ls-remote` before/after is identical: `ab752d59586c595f9409d8c9ba59026a51d4a613`; nothing was pushed.
- The retained staged root passed typed allowlist/catalog checks and a direct staged-copy 7/7 WASM audit (`staged-wasm-audit.log`). `stage-playwright-explicit.config.ts` explicitly starts `ruby -run -ehttpd <retained-stage> -p 4174`, and `stage-playwright-explicit.log` records **45/45** against that root. The authoritative clean browser-load is `stage-browser-load.log`, with 14 requests and zero external/raw-TypeScript/runtime errors. Exact arithmetic `42`, neutral navigation, both themes and desktop/mobile layouts passed in the exact staged output, not merely the development dist.
- The earlier `stage-browser-load-cleanup-failed.json` is explicitly retained as a failed Chrome-cleanup attempt with `ENOTEMPTY`; it is not the authoritative browser result. The clean corrected log above is separate.
- The static terminal-unavailable screenshot and all staged Playwright outputs are preserved under `local/kimi/typed-publish/25c63d42d58275c8fb4836f8c6defe6fd6a7c5f2/`.

## Checks run at the head

- Task 1 red: missing `--typed` source path → green; Task 2 red: missing help flag → real dry-run green. Final task-completion runs each passed all 15 runtime contracts.
- `scripts/studio/build-and-publish.sh --typed --target gh-pages --keep-worktree --stage-dir <evidence>/stage`: full verification PASS; retained commit `1684785face01e02d4f79a34e49c0d1b954d0f40`, previous/rollback head `ab752d59586c595f9409d8c9ba59026a51d4a613`.
- `python3 -m unittest scripts.tests.test_typed_runtime scripts.tests.test_tauri_runtime`: **15/15 PASS**. Typed/plain-JS, desktop default/GUI, staged Playwright and 11/11 gates PASS as above. One separate typed run had all 453 assertions pass but Vitest flagged one unhandled `document is not defined` Inspector teardown error; its red log is retained, and the complete rerun passed 453/453 plus 113/113 plain-JS without the error. No unrelated Inspector source was changed.
- `local/kimi/typed-publish/live-models-identity.txt` records the actual live/local commands and outputs: both `models.json` files are 4,365 bytes with SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- `uor-harness loop pr-check --local` at 25c63: scope/ORDER-first PASS, **FAIL only missing REPORT**. It is rerun at the report head; no earlier failure is hidden.
- Final report/scope heads pass local policy and **11/11** gates. The scope list now includes both new design/plan files, while the historical base-scope advisory still names them outside the base item; protected queue acceptance remains a separate required check. `cargo xtask roadmap check` independently fails four unrelated existing problems: `float-kernel-fusion.md` missing `## What is missing`, `infra-desktop-client-tauri.md` using unsupported status `in_progress`, and their two stale `issues.json` ids. Those files were not changed or represented as passing.
- Modern web guidance was searched for static publication; no directly applicable guide was retrieved. Existing typed allowlist and browser checks were used instead.

## Skills used

Systematic debugging, brainstorming and plan writing, inline execution with a durable ledger, test-driven development, native macOS accessibility acceptance, built-in browser and CDP acceptance, artifact preservation, and verification before completion.

## Still open

Evidence reconciliation in kimi-12, then fresh exact-head review, required checks, protected merge, fresh-main typed dry run and publication, Pages deployment and hard-reloaded public verification, evidence preservation, then branch/worktree/build-output cleanup. Full agent runtime, parallel tasks, memory, steering/orchestration, Dispatch, browser/computer use, connectors, live R4 API and signed production release remain unbuilt.

## Plain summary

The repository can now stage and dry-run the neutral workbench through the existing guarded publisher instead of the legacy page. The public site has not changed yet: actual protected merge and publication are the remaining rollout steps, not facts to infer from a local commit or green fixture.
