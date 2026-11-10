# ORDER: repair the two records blocking official roadmap sync, kimi-51

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8 (notes/roadmap migration). Base: `4265dc3d6c94015ba97bcb1505ed8d57e339a39a`. References #3915, #2826 and #3678.

## Goal and decision review

Restore a valid roadmap corpus so the official sync can create the already merged `studio-cursor-hosted` issue. The top session reviewed CHARTER §5.1–5.3, the current synthetic-desktop record, #2826's closed state, the done→in_progress change in merged #3675, `float-kernel-fusion`'s current remaining-work text, and Agent-100's source/route review. These are direct preconditions of the corpus-global sync, not new dependencies of the Studio item.

Decision: restore `infra-desktop-client-tauri` to `done` at its existing synthetic stand-in scope, matching its prior accepted status and closed issue, with no trained-product claim. Rename `float-kernel-fusion`'s `## State and remaining work` to the required `## What is missing`, preserving its body and open status. No acceptance, scope, technical rule, bar or model status changes.

## Cancel probe (ran first)

`cargo run -q -p xtask -- roadmap check` reports exactly four errors: the missing float section, invalid desktop status, and their two issue-id consequences. Both files match origin/main; no open PR edits them. #2826 is CLOSED and was previously done; #3678 is OPEN. Agent-100 confirmed there is no supported per-item validation bypass and recommended this separate existing-item repair. Continue; no manual issue or generated-metadata edit.

## Files in scope

- `docs/roadmap/items/infra-desktop-client-tauri.md` (status only)
- `docs/roadmap/items/float-kernel-fusion.md` (heading only)
- `docs/roadmap/README.md` (official render only)
- `docs/loop/tasks/infra-loop-harness/kimi-51/**`

No product code, tests, model/API/engine/worker/tokenizer, `.github`, issue-body, `issues.json` hand edit, or Studio source change. Official `roadmap sync` is a later fresh-main unit, not this repair PR.

## Done when

- `cargo run -q -p xtask -- roadmap check` passes with zero errors; `roadmap render` and `roadmap render --check` pass, including the already merged Cursor item without inventing an issue number.
- Only the two intended input lines and generated roadmap output change; #2826 remains closed, #3678 remains open, and no trained/useful-model claim is added.
- ORDER/REPORT, local policy, full gates, fresh exact-head review, required GitHub checks, protected merge and fresh-main verification complete. Then return to #3915 for the official sync dry run/apply and write-side verification.

## Stop and re-scope if

Evidence does not support the synthetic desktop item's done acceptance, a different status/scope/acceptance edit is needed, the render changes unrelated inputs, or the official validator still reports an error. Do not weaken the validator, manually create an issue, or patch generated metadata.
