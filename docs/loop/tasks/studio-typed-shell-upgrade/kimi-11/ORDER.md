# ORDER: restore committed desktop integration evidence

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`. Prior reviewed head: `e718a4090b4f28949e8a837e74742290997bc4cb`, tree `ccfd1841fb1f07caf94a5c4ce9df9d25fc9a5f8a`, PR #3819 draft.

## Finished steps

- kimi-10 repaired independent Enable/Run identities, empty-rootPath invalidation, empty-input guidance, unavailable wording, and timeout/unmount/available-host coverage; focused and full suites reached 358 tests.
- Fresh review found the implementation correct but returned CHANGES because `StudioApp.test.tsx` was accidentally left uncommitted. The committed tree retained the old unavailable assertion and omitted the available-host integration test, so the 358 evidence was from a dirty worktree.

## Goal

Commit exactly that omitted test repair and produce new clean-head evidence. Do not change product behavior or reuse the earlier head's logs as proof of a different commit.

## Cancel probe

The working-tree diff contains only the intended unavailable assertion, adapter import, and available-host StudioApp integration test. If any other source change appears, stop and review it separately.

## Files in scope

- `web/studio/src/components/StudioApp.test.tsx`, already reviewed in the working tree.
- This repair folder and the evidence/counts in `docs/products/web-client.md` and `docs/research/notebook/m4-serving.md` only if needed.

No other paths are authorized. No product logic, legacy adapter/HTML, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real repository/command/browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- The committed tree contains the unavailable assertion and available-host integration test; no other source change is included.
- A clean committed head passes the full 358 tests across 43 files, 31 StudioApp tests, 112 plain-JS tests, typecheck/build, legacy checks, claim/report/local-policy checks, and all 11 gates.
- The exact-head diff, logs, PR head/tree, and REPORT refer to that same commit. Fresh review confirms the old mismatch is closed before ready/queue.
- PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop if any additional source change appears or if the clean head cannot reproduce the 358-test result. Stop after this evidence repair and protected delivery; do not start another UI unit in this session.
