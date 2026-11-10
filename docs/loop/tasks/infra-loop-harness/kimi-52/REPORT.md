# REPORT: register the Studio reducer test and crate FILEMAP scope, kimi-52

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Order: [ORDER.md](ORDER.md). Base: `993bfd36cc4ef4660d482d0f65238361d7126e12`. References #3915 and #3917; no acceptance change.

## What did not work

- Task 3's required `web/studio/src/state/agent.test.ts` was absent from the item and suborder scope. Local policy reported it as an advisory because D76 makes scope advisory; that does not make the registration correct.
- The new `crates/uor-r4-studio-agent/` contract crate had no FILEMAP row. The remaining FILEMAP warnings predate this work and are not changed here.
- No implementation or test expectation was changed to evade those gaps.

## What changed

- Added the exact mandated reducer-test path `web/studio/src/state/agent.test.ts` to the Studio item scope; no other glob or acceptance text changed.
- Added one FILEMAP row describing the Studio event schema, canonical JSON, cursor and approval contract crate.
- Total source registration diff: two lines. No product code, dependency, model/API/engine/worker/tokenizer, service, Tauri, `.github`, issue body, generated roadmap body, or test expectation changed.

## Measurements

- `roadmap check`: **143 items OK**. `roadmap render --check`: current, because the edit is frontmatter scope metadata rather than rendered body content.
- `git diff --check`: clean. No model, GPU, paid compute, credential, deployment or useful-model claim.

## Checks run at the head

- `cargo run -q -p xtask -- roadmap check`: PASS (`local/kimi/studio-scope-repair/check.log`).
- `cargo run -q -p xtask -- roadmap render --check`: PASS (`render-check.log`).
- Final local policy, full-gate, GitHub and fresh exact-head review results are bound in the PR and retained local logs after execution; pending results are not PASS.

## Skills used

- Scope review and isolated-worktree delivery, working discipline, solution integrity, subagent-driven task-scope correction, and verification-before-completion. Task 3's independent code review is separate; no subagent decided this scope change.

## Still open

- Local policy/full gates, fresh exact-head approval, required GitHub checks, actual protected merge and fresh-main verification.
- After merge: update #3915 from main and issue a corrected Task 3 suborder including the mandated test path, then confirm the reducer-test and new-crate advisories are gone.
- No useful-model, production, hosted-service or full-parity claim is made.

## Plain summary

The approved reducer test and the new contract crate were missing from registration metadata. This unit adds exactly those two lines and leaves all implementation and acceptance unchanged. The roadmap validates and the render is reproducible; protected delivery remains outstanding.
