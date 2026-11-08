# ORDER: prove destination-divergent migration reads, kimi-69

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `c41d1a870127022f538c4a0febb760b372ab0147`, tree `7e1ad67a601fc32df525d6dc4db29c8bed46da16`; original merge base: `a3ac3f326fa95ffe38159cc3c589100cea627664`. Agent-48 returned CHANGES at the base and blocked ready/queue.

## Findings to repair

- Theme/current-UI assertions use default values and an uncontrolled viewport, so they can pass without migrated reads.
- Attachments, tabs/workspace, theme, and current-UI are read while source and destination match; later reads after divergence are not proven.
- Deleted recent destination is checked only in storage, not through the rendered recent-files read path.
- Initial marker read failure lacks a direct no-write test.

## Cancel probe

Require destination-divergent evidence: light→dark theme, expanded→collapsed navigation at a controlled 1440×900 viewport, a new attachment, and edited workspace content must differ from their old raws and survive reload. The recent-files view must render its empty state after destination deletion. Fresh UI/file module loads must also re-read divergent destinations, and initial marker read failure must perform no writes. Current assertions are insufficient and must fail if routing returns old sources.

## Files in scope

- `web/studio/verify_studio_migration.mjs`, `verify_studio_page_load.mjs`.
- `web/studio/src/persistence/legacyRouting.test.ts`, `migration.test.ts`.
- `web/studio/README.md`, `docs/products/web-client.md`, the approved design, roadmap/notebook evidence, and this repair folder.

No product behavior change is anticipated; only demonstrated defects may change product source. No dependency, real data/credentials/endpoints, native/package work, production cutover/publication, or Phase B compute.

## Done when

- All named domains have non-default or edited values that differ from old raws and are read back after reload/fresh module initialization.
- Recent-files renders empty after destination deletion; initial marker read failure performs no writes.
- Existing singleton/quota/mutation evidence and complete product/legacy/repository checks remain green.
- Documentation names destination-divergent evidence; fresh exact-head approval precedes protected delivery.

## Stop and report

Stop before relying on source/destination-identical or default-only assertions, weakening a check, using real data/credentials, changing unrelated product behavior, or publishing. Continue only after a fresh reviewer accepts the discriminating evidence.
