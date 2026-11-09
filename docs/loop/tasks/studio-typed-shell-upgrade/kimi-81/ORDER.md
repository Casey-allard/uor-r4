# ORDER: resolve final identity and evidence-chain findings, kimi-81

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `3e33a175bcf928e1e42e7bbe8c7a149709095da3`, tree `fc3df19e7e556d5f3c43b385a0525d6184f96255`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Fresh reviewer agent-64 returned CHANGES. PR #3879 remains draft and unqueued.

## Findings and decision review

The implementation and measured development behavior were accepted. The remaining blocker is contradictory evidence provenance:

- `kimi-79/REPORT.md` calls d399 source/acceptance and then says d399 through 3c7176d5f was record-only; it also calls b975 final in its measurements and intermediate in its summary.
- `kimi-80/REPORT.md` and four active records claim that report binds the final exact identity, but it names only its base. A committed report cannot embed its own commit hash without changing the hash. Do not manufacture another self-referential record. The final identity belongs in the PR body and durable `identity.json`; the report binds the historical roles, checks and negative evidence.
- The final identity.json, loopback JSON and screenshot are retained in the main checkout, not both copies. Make that location exact rather than claiming both.
- The mixed-case RED observation is retained as a report observation, not a separate raw red log. Do not claim a durable red log exists. The executable regression and complete green logs are the retained test evidence.

`d39953a7d4ebab479673eba50e06481151b740bc` is earlier source/acceptance; `988155ae56dc0fdd872b7428018ead566179fe3f`, tree `e6542fb375518c1b38639cff825b9ac65f7b3594`, is the source/assertion head for default/GUI suites. `3c7176d5f`, `c17105e704e1ee9b02271062a13fc23747649604`, `b9750be653d451691694853b62d4a98b256f48f3`, and `3e33a175bcf928e1e42e7bbe8c7a149709095da3` are intermediate record-only heads, not the final delivery identity. The new candidate's exact HEAD/tree will be computed from Git after committing and written into the PR body and durable identity evidence, with fresh local policy/gate and loopback results.

## Files in scope

- This task folder and `kimi-79/REPORT.md`, `kimi-80/REPORT.md`, plus the limited RED-evidence clarification in `kimi-78/REPORT.md` if needed.
- `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `docs/products/desktop-client.md`, and `web/studio/README.md` only to correct the identity/evidence references.
- Ignored evidence under both worktrees' `local/kimi/studio-final-acceptance/`; the main checkout is the authoritative durable copy.

No production code, tests, model/API/engine/worker/tokenizer, or new feature claims. Do not rewrite or delete historical evidence. No production publish, signed release, useful-chat or full Cowork parity claim.

## Done when

- Every active source/record role is consistent, b975 and 3e33 are intermediate, and no report claims to bind an unnamed final self-hash.
- Final identity is explicit in the PR body and main-checkout durable identity.json, with no circular citation; missing raw RED evidence is disclosed.
- The actual candidate receives fresh local policy/full gates and clean loopback, a new fresh-context exact-head approval, required GitHub checks, protected queue/merge and fresh-main verification.
