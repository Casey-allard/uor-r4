# ORDER: correct final evidence citations, kimi-80

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `b9750be653d451691694853b62d4a98b256f48f3`, tree `77487ea04c695bc38526b1acbdd89b469cc22695`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Fresh reviewer agent-63 returned CHANGES limited to record identities, citations and evidence durability. PR #3879 stays unqueued.

## Findings and decision review

`d39953a7d4ebab479673eba50e06481151b740bc` is the earlier source/acceptance head; `988155ae56dc0fdd872b7428018ead566179fe3f` is the source/assertion head for the retained default/GUI counts; `c17105e704e1ee9b02271062a13fc23747649604` is an intermediate record-only head; `b9750be653d451691694853b62d4a98b256f48f3` is the final record-only head under review. `3c7176d5f` is also an intermediate record-only head, not a distinct source head. Correct current-state, notebook, desktop/README and kimi-79's plain summary/citations to say exactly that. Point readers to `kimi-79/REPORT.md` for binding evidence, not only its ORDER or the older report.

Copy c171's local-policy and full-gate logs byte-identically into the main checkout, but cite the durable b975 logs as the final policy/gate evidence. Do not embed a guessed self-hash for the next record-only commit; the PR body and fresh exact-head reviewer will name the final pushed HEAD/tree.

## Files in scope

- `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `docs/products/desktop-client.md`, `web/studio/README.md`, and `docs/loop/tasks/studio-typed-shell-upgrade/kimi-80/**`.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-79/REPORT.md` only to make c171 intermediate and cite the durable b975 final logs.
- Ignored durable copies under the main checkout's `local/kimi/studio-final-acceptance/`.

No production code, tests, model/API/engine/worker/tokenizer or new feature claims. No production publish, signed release, useful-chat or full Cowork parity claim.

## Done when

- Every active record assigns d399, 988, c171 and b975 to the same source/assertion or record-only role and links `kimi-79/REPORT.md`.
- c171 and b975 logs exist byte-identically in the main checkout; final policy/gate citations use b975.
- Local policy and full gates pass at the final record-only head, fresh exact-head review approves, GitHub checks pass, and protected merge/fresh-main verification complete.