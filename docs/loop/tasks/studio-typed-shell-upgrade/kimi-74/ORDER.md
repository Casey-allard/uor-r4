# ORDER: finish preserved Studio acceptance delivery, kimi-74

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `c4d350b8a5af364945471c7d2566c0f1db62bfe6`. This is the delivery repair for `kimi-73`; implementation and acceptance history are preserved.

## Findings and decision review

The recovered workbench and this older acceptance unit cannot be one PR: `loop pr-check --local` found two different task ORDERs. Deliver this unit first from its preserved merge `c4d350b8a`, then merge the resulting `main` into the workbench. At this base the policy gate selects `kimi-73` but refuses because its REPORT is absent. Supply the missing active delivery record and verify the unchanged implementation. The already merged scope PR #3866 is in the current base; the separate notebook scope warning is advisory under D76 and must be reported, not hidden. Do not widen scope or rewrite history.

## Files in scope

- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-74/**` only.
- Generated test/build/evidence output under ignored `local/kimi/**` and `target/kimi-final-acceptance/`; earlier evidence is copied byte-identical into this worktree before any rerun can replace a same-named output.

No application, model/API/engine/worker/tokenizer, roadmap, product-document, dependency, production-publish, credential, real-profile, or other lab changes.

## Done when

- The active REPORT accurately leads with limitations, records previous measured browser/package evidence at its exact code identity, and lists fresh checks and NOT_RUN delivery steps.
- Typed tests/typecheck/build, plain-JS tests, typed-runtime tests, desktop default/GUI tests, and the real isolated loopback workflow run at this head. Full repository gates and local policy/report checks pass. No local N/A is counted as GitHub evidence.
- A fresh exact-head reviewer sees only the item chain, ORDERs and diff, returns APPROVE/CHANGES with evidence, and the counted review/GitHub checks precede protected queue/merge and fresh-main verification. The workbench is delivered separately afterwards.

## Stop and report

Stop before overwriting unarchived evidence, editing implementation to avoid a delivery failure, claiming useful model behavior or a signed/production release, or touching the owner's checkout. Report every failed/refused check and preserve other labs' work.
