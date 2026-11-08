# REPORT: finish agent/session idempotence and evidence fidelity, kimi-59

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `506b45afc79e3f7fb91862dca900f5756eaf2c16`, tree `e934c9cde26c68beeb7473eb30cf7c96576244ff`; original merge base: `f3a3b7869dc7644482b0704465eef24be962d3bc`. Current HEAD/tree and merge base are preserved in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-37 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- The composed rewrite moved five navigation commands out of the base `Navigation` group. The first regression test checked only Agent and therefore missed the drift. Base source comparison confirmed the issue.
- The existing mismatch guard already rejected a repeated identical interruption in the red run. The new explicit current-ID guard and repeated-action characterization test document that invariant; this was not a reproduced repeated-application defect.
- `getMode` could throw outside the generation failure guard. The shared `makeSessions` defaults still returned create/select/delete success without mutating a catalog, and the SessionList active-delete fixture selected an existing row instead of creating a replacement.
- Fresh-main wording in kimi-58 implied nonexistent logs, and the advisory list mentioned an ORDER read-path warning even though the actual check passed. The kimi-57 measurement heading was historical but insufficiently explicit.
- The browser proved exact-response presence but not exact restored content, and could resume before generation completed.

## What changed

- All six navigation commands again use the base `Navigation` group; the search test requires all six IDs.
- Interruption rejects a current active ID before using active or just-stopped snapshots. A second identical dispatch leaves the same state object unchanged.
- `getMode` failures return false without model invocation. Shared fixtures now create/select/delete against a real catalog with replacement-on-active-delete. No-active deletion returns `true` only after verified target removal and no active result; failure still returns null.
- Browser verification waits for `Session: complete`, then requires `restoredTranscript === firstTranscript` in addition to the exact R4 response and active markers. Post-workflow request capture remains in force.
- kimi-57 is explicitly historical. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh-main verification and fresh review remain NOT_RUN, not claimed as preserved evidence.

## Measurements before the final committed repair head

- Red-first focused result: **3 failures / 93 passes** for no-active deletion, `getMode`, and all-six navigation. The separate browser assertion failed because generation completion/equality was not yet measured. After repairs: **408/408 tests across 47 files**, typecheck PASS.
- Typed main JS **88.57 kB** (gzip **28.49 kB**); index JS **151.91 kB** (gzip **44.00 kB**). Plain-JS remains **112/112**. All four legacy checks pass.
- Built browser: two distinct sessions, actual UOR arithmetic `42`, no exact response in the other transcript, and a byte-identical original transcript on return. `generationComplete=true`, `restoredMatchesOriginal=true`; post-workflow requests **12**, all local, no raw TypeScript, zero runtime errors.
- Previous mutation evidence remains valid: old verifier accepted a planted external URL and generic `42`; repaired verifier rejects both. No real endpoint was contacted.

## Checks run at the head

Red/focused/full logs are under `local/kimi/studio-agent-session/`; mutation evidence is under `verifier-mutation/`. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved in a head-specific directory after commit. Fresh-main logs and fresh review are **NOT_RUN**. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes.

Agent-37 returned CHANGES at the base. The follow-up fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with base verification, systematic debugging, red-first TDD and honest-test guidance, solution integrity, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes the bounded agent/session authority and its evidence. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later Phase A units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The controller is now idempotent, navigation grouping matches the base, generation reads fail closed, shared doubles model real sessions, and the browser proves exact transcript restoration. Reports no longer imply fresh-main evidence or an ORDER read-path warning. A fresh exact-head review and protected delivery remain pending.
