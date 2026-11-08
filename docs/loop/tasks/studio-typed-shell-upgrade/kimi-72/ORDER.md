# ORDER: align typed Studio item scope with native acceptance, kimi-72

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `973bc075050aef59e753159c9ba30a3086b6c7f1`.

## Decision review

Agent-54 blocked `bf161456d` because the item's base scope omits paths needed by the approved native browser/desktop acceptance. `docs/loop/README.md` requires a separate item PR, reviewed by Kimi, before the acceptance ORDER can include them. This is that scope-only PR; it changes no product code, model behavior, milestone bar, release default, or production deployment.

Kimi top-session decision: the existing typed native/shared acceptance needs the narrow host, desktop-product/current-state, adapter, runtime-test, and final-acceptance plan/spec paths added to the item. These are implementation/evidence dependencies of the existing item, not a new feature or Phase B scope. The item stays `ready`; publication and useful-chat/M7/M8 claims remain closed.

## Files in scope

- `docs/roadmap/items/studio-typed-shell-upgrade.md`, only its declared scope list.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-72/**`.

No other file, dependency, generated artifact, model compute, or owner-controlled path.

## Done when

The declared item scope covers the exact native host/product/current-state/adapter/runtime-test/final-plan paths used by the reviewed acceptance work; report/local policy/full gates pass; a fresh Kimi delivery review approves this exact head; and the PR merges through the protected queue and is verified on fresh main. The acceptance branch can then merge this base without scope warnings for those paths.

## Stop and report

Stop before adding product code, broadening to unrelated paths, changing a model/security rule, or treating this scope alignment as production publication or Phase A completion.
