# ORDER: register the Studio reducer test and crate FILEMAP scope, kimi-52

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item scope maintenance). Base: `993bfd36cc4ef4660d482d0f65238361d7126e12`. References #3915 and #3917; no product acceptance change.

## Goal and decision review

Register the reducer test path already required by the approved Task 3 brief and the new Rust contract crate's FILEMAP row. The top session reviewed the item scope, Task 3 file list, local scope advisory and FILEMAP guidance. This is a narrow PM scope repair, not a feature, dependency, model or acceptance expansion.

## Cancel probe (ran first)

`loop pr-check --local` on #3915 reports `web/studio/src/state/agent.test.ts is outside item studio-cursor-hosted's scope` and `crates/uor-r4-studio-agent` missing from FILEMAP. The test is explicitly required in the approved plan and RED/GREEN command; the crate was added in reviewed Task 1. No other open PR edits these two registration files. Repair the registration, not the implementation.

## Files in scope

- `docs/roadmap/items/studio-cursor-hosted.md`: add only `web/studio/src/state/agent.test.ts` to the scope list.
- `docs/start/FILEMAP.md`: add only the `crates/uor-r4-studio-agent/` row, describing the Studio event/canonical contract.
- `docs/loop/tasks/infra-loop-harness/kimi-52/**`

No product code, test expectation, model/API/engine/worker/tokenizer, service, Tauri, `.github`, generated metadata, issue body, or acceptance edit. Roadmap body/render is unchanged because this is frontmatter scope metadata.

## Done when

- The item scope admits the mandated reducer test and the FILEMAP row names the actual crate. `roadmap check` and `render --check` remain current; no unrelated path/status changes.
- ORDER/REPORT, local policy/full gates and required GitHub checks pass; fresh exact-head review, protected merge and fresh-main verification complete.
- After merge, #3915 merges the registration forward, receives a corrected Task 3 suborder including the test path, and shows no reducer-test scope or new-crate FILEMAP advisory.

## Stop and re-scope if

Any additional glob, behavior change, new file, issue state change or acceptance edit is needed. Do not hide the advisory, rename the test to evade scope, or modify Task 3 code in this PR.
