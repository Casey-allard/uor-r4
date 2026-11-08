# REPORT: align typed Studio item scope with native acceptance, kimi-72

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `973bc075050aef59e753159c9ba30a3086b6c7f1`.

## What did not work

Agent-54 correctly blocked the acceptance head because its host/product/current-state/adapter/runtime-test/final-plan paths exceeded the item's base scope. The loop requires this separate item PR rather than treating an ORDER or a local advisory as permission to widen scope. No product source, model behavior, milestone bar, release default, or production deployment is changed here. The item remains `ready`; typed publication, full Phase A, useful-chat/M7/M8, and Phase B remain unclaimed.

## What changed

`docs/roadmap/items/studio-typed-shell-upgrade.md` adds the narrow paths needed by the already reviewed native acceptance: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/tests/**`, `web/studio/assets/js/desktop_actions.js`, `scripts/tests/test_typed_runtime.py`, `docs/products/desktop-client.md`, `docs/integration/current-state.md`, and the two dated final-acceptance plan/spec files. No broad root wildcard or unrelated path was added.

Size: the gate reports **3 documentation files, +58/-1 lines**. The item scope line itself is +1/-1; the remaining additions are this ORDER and REPORT. The ORDER's documentation-only estimate is met; there is no code or generated diff.

## Measurements

The scope list now covers every non-always-allowed path identified by agent-54's acceptance review. Notebook rows and lab task folders retain their existing always-allowed treatment. This is a scope/provenance measurement only; no browser, model, package, or production claim follows from it.

## Checks run at the head

Harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks passed. Logs are retained under `local/kimi/studio-scope/1b85c53174fb73ebd9c43fef776fb5de3c64fe52/`, with the gate ledger and detailed gate logs under `local/kimi/gates/`. The local scope PASS reads the item at the merge base, so it validates only the three always-allowed changed paths in this PR; the nine new scope entries were inspected directly and still require acceptance-branch revalidation after merge. Fresh review and protected delivery are not inferred from local checks.

## Skills used

Protected scope-decision review, source/diff inspection, and verification before completion. A fresh exact-head Kimi delivery review is required before push/queue.

## Still open

Fresh exact-head review, PR/GitHub checks, protected queue/merge, fresh-main verification, and subsequent acceptance-branch scope revalidation are pending at this edit. This scope PR does not itself approve the acceptance implementation.

## Plain summary

The item now names the narrow native host and evidence paths that its existing acceptance work actually needs. This is a procedural scope alignment only, with no product or model capability change.
