# ORDER: register LexLean Studio repairs, kimi-7

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: `d416181c1` from fresh `origin/main`.

## Goal

Register `m7-lexlean-studio-tools` as the separate roadmap item for the already tested exact-arithmetic and Studio identity repairs. This item/scope PR is required so the implementation PR has that item at its merge base. It delivers no implementation, model change, publish or completion claim.

## Scope

- New `docs/roadmap/items/m7-lexlean-studio-tools.md`
- `docs/loop/tasks/m7-studio-live-model-sources/kimi-7/**`

No generated roadmap files, source code, tests, model weights/catalogs, Pages, desktop, `.github/**`, owner records or other labs' files. Existing issue #3704 is referenced, not duplicated or closed.

## Cancel probe

The new item is absent from protected `main`; both dependencies exist. Issue #3704 is open and describes this exact gap. The implementation candidate's local policy failed only because the new item was absent at its merge base, confirming this registration is necessary.

## Done when

- The item is `ready`, depends on `m7-studio-live-model-sources` and `infra-desktop-client-tauri`, and records the measured LexLean rejection, bounded implementation and honest non-capabilities.
- Roadmap validation and repository gates run; any pre-existing generated-roadmap mismatch is reported without changing unrelated records.
- Local policy and fresh exact-head review pass at this PR's head, then the protected PR merges with `References #3440` and `References #3704`.

## Stop and report

Stop after registration merges and rebase the preserved implementation commit onto that base. Do not add implementation to this PR, close either issue, publish Pages or claim useful conversation, visual understanding or milestone completion.
