# ORDER: reconcile typed-publication evidence and live identities, kimi-12

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `d4cb497060542839e812d089e5be61440f7e0552`, tree `b57ebf5a6e889d28ac470824b11f41164da4fe59`; current `origin/main` `5643a45c5c82bc9b72be3a5ac9df89183e4f2c3f`. PR #3897 is draft and unqueued. Fresh reviewer agent-73 accepted the publisher behavior but returned CHANGES on record/evidence identity. Do not queue or publish on that verdict.

## Findings and decision review

- `web/studio/README.md` still calls eb832431a/700756-byte bundles current in two places, contradicting actual `gh-pages` `ab752d59586c595f9409d8c9ba59026a51d4a613` with 794677-byte artifact `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`. Correct the live identities and preserve prior publications as history.
- The 45-test config inherited a server command that builds/serves development dist. Although a separate stage-root server was actually used, the retained configuration did not prove that root. Replace the evidence with an explicit independent server command/root/port and rerun all 45 tests. Audit the staged WASM copy directly too.
- Rename the nonzero browser-cleanup artifact explicitly as a failed attempt, preserving its bytes, and identify the clean corrected log as authoritative. Do not hide the cleanup failure or leave two unnamed candidates.
- Cite actual live/local `models.json` verification: both are 4365 bytes and SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`. If those values change, narrow rather than invent the claim.
- Reconcile the REPORT's completed policy/gates with its open list. Merge current main before this repair's final review so the PR is not evaluated behind.

## Files in scope

- `web/studio/README.md`, `docs/products/web-client.md`, `docs/integration/current-state.md`, `docs/products/harness-parity.md`, `docs/roadmap/items/studio-{ide-harness,typed-shell-upgrade}.md`, `docs/products/desktop-client.md`, and this task folder plus the earlier REPORT's open list.
- Explicit staged Playwright config/log, staged WASM audit, browser-load evidence labels, live-model identity record, updated PR body, and final evidence under `local/kimi/typed-publish/`.

No publisher/application/model/API/engine/worker/tokenizer behavior change, dependency change, `.github/**` edit, ad-hoc gh-pages commit, public terminal service, fresh native exit-0 claim, or deletion of prior evidence/other labs' work.

## Done when

- All current live-identity references agree with actual gh-pages and its artifact; the models.json claim is bound to an executed live/local identity check.
- A retained Playwright config explicitly starts its own server rooted at the retained `stage` directory on a distinct port, and all 45 tests pass against it. The exact command/root/port and staged 7/7 WASM audit are logged.
- The failed browser cleanup is explicitly labeled, the clean corrected run is named, and the REPORT's open list contains only outstanding review/merge/publication/cleanup work.
- Current main is merged, affected and complete checks pass, local policy and 11/11 gates are green, and a fresh exact-head reviewer approves before ready/queue. Actual protected merge, fresh-main dry run/browser verification, guarded gh-pages publish, Pages verification and evidence preservation precede cleanup.
