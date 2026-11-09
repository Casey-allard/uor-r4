# ORDER: repair final Studio acceptance review findings, kimi-73

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `bf161456d803a19bd76fdf2e526601c80e47720c`, tree `304f7729fd286901813431eec743e69f88d3bb34`. Review: agent-54 returned CHANGES at that exact head.

## Findings and decision review

The loop requires a separate item-scope PR for host/product/plan paths outside the item's base scope; that scope-only unit is `kimi-72` in `.worktrees/kimi-studio-scope`. Do not widen item scope in this acceptance branch. Merge that reviewed scope PR before this unit's final policy check.

The acceptance repair itself must: assert every session-flow field in the loopback verifier and retain them in JSON; limit loopback attachment claims to the measured current-file action; mark older timeout/435-test records historical; distinguish completed exact-head checks from remaining GitHub delivery; prove runtime-code identity for the earlier packaged executable; and test an approval aged beyond 60 seconds. Preserve the sound process/permission implementations and all negative evidence.

## Files in scope

- `web/studio/verify_studio_loopback.mjs` and its generated local JSON evidence.
- `src-tauri/src/lib.rs` only for a same-module expired-approval test.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/**`, `kimi-73/**`, and directly affected product/current-state/roadmap/README/notebook records.

No new dependency, model/engine/worker/tokenizer/`studio_api.js` change, production publication, real endpoint/credential/browser profile, permission weakening, or Phase B work.

## Done when

The loopback JSON retains complete two-session/exact-42 state, claims match measured actions, the aged grant is refused and consumed without execution, historical/current records do not contradict, runtime-code identity is logged, the separate scope PR is merged, complete checks pass at the repaired exact head, and a fresh delivery reviewer returns APPROVE before protected delivery.

## Stop and report

Stop before weakening a check, hiding a failed workflow, changing a security boundary, claiming final-binary GUI evidence without identity proof, or pushing before scope alignment and fresh approval.
