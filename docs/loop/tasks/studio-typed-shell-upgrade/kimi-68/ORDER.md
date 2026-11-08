# ORDER: repair migration activation and evidence, kimi-68

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `dd8957d0a0a9e5a1b13330d681d9a474c393b448`, tree `e8983d839ba59928a19362c298788e234320b83b`; original merge base: `859621f3373708fdd0b8f78421a494f5f0ee88c4`. Agent-47 returned CHANGES at the base and blocked ready/queue.

## Findings to repair

- Classic and bundled runtime initialization both activate, allowing a first migration failure to be retried and hidden. Make global initialization singleton/idempotent; first success reports `migrated`, and one-shot failure cannot mount.
- `migration-disabled.patch` is empty and mutation build/browser logs lack identity. Preserve a real patch and identity-tagged red/restored logs.
- Marker read-back/read failure and rollback-failure paths lack tests; the marker-write test does not assert source preservation.
- The design names browser quota evidence but only unit injection existed. Add a one-shot real-browser quota failure that also proves no second activation succeeds.
- README/notebook overstate restoration as “every seeded domain”; assert theme/current-UI and narrow wording to measured domains.

## Cancel probe

Require red-first singleton failure evidence before implementation: the second global runtime load must not activate or replace the existing instance, a one-shot browser quota failure must remain visible with no shell/marker, and new marker/rollback tests must fail on the missing cases. Preserve the actual activation-disable patch with exact identity and byte-identical restoration.

## Files in scope

- `web/studio/assets/js/lifecycle_persistence.js`, `web/studio/src/persistence/*`, and focused tests.
- `web/studio/verify_studio_migration.mjs`, `verify_studio_page_load.mjs`, and the migration build/browser evidence path.
- `web/studio/README.md`, `docs/products/web-client.md`, the approved design, roadmap/notebook evidence, and this repair folder.

No dependency, real data/credentials/endpoints, cross-tab atomicity, native/package work, production cutover/publication, or Phase B compute.

## Done when

- A second runtime evaluation cannot activate or replace the singleton. First success is `migrated`; reload is `current`; one-shot quota error remains `quota-exceeded` with no shell or marker and old raws preserved.
- Marker mismatch/read failure rolls back new writes and preserves sources; rollback failure reports `rollback-failed`; marker-write assertions include all sources.
- Real patch/build/browser red and restored-green logs carry exact identity; restoration is byte-identical.
- Documentation matches measured browser domains and injected browser quota scope. Complete product/legacy/repository checks, fresh exact-head approval, and protected delivery pass.

## Stop and report

Stop before weakening a check, hiding a first-attempt failure, claiming a real system quota or cross-tab guarantee, using real data/credentials, or publishing. Continue only after all five findings are resolved and freshly reviewed.
