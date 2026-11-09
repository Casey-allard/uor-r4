# REPORT: reconcile publication status records and exact-head evidence, kimi-15

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `dfee63a0449863ce12a6549aaf8beaccddf3c4af`, tree `66270e868a68c9c4ee81192db4700be5cf776d27`. Agent-76 returned CHANGES on PR #3899 at `eb806fdded1a228d54fc22c94bcabb93bc9211af`; its verdict is posted. Current main was merged normally as `2c5d2a407f54ef43169507ffec598999dbb241fd`, producing merge commit `155f811f25ce942326d9a5d08415c6ba0945727d`, tree `0ae1f11f53783f2aabc45a4ec0a79bfde3eba072`. The actual live publication remains `6540e2717d3a606330dbd141e3642f3f13b87ecf`; rollback remains `ab752d59586c595f9409d8c9ba59026a51d4a613`.

## What did not work

- Current product records still described typed static publication as pending or placed mobile/accessibility/cutover checks before an already-completed publication. Agent-77 returned CHANGES at `808b5e03091af0be716645d5785f378fb23907a2`: a second historical current-state entry still used a present-tense legacy-live claim, the roadmap-check sentence could be read as six problems instead of four total, and typed-production wording needed explicit static scope. All three are corrected without changing application behavior.
- kimi-14's ORDER confused the status-record base with the merged publisher's tree. Both records now use the actual status-record base and distinguish the publisher merge identity.
- Prior local gate rows belonged to the old commit with subsequent uncommitted edits. They were not exact clean-head receipts for the repaired records. The first policy run at the new merge commit also failed because this REPORT did not yet exist; that failure is retained, not relabeled as a pass.
- The PR-body candidate omitted the native-window/exit limitation and exact unrelated roadmap IDs. It is corrected before posting the final body.
- PR #3899 conflicted with current main and its old comparison included already-merged publisher implementation history. A blanket ours/theirs merge was not used: each seven-file conflict was inspected and resolved between stale pre-publication wording and the verified live identity.
- The first app window disappeared before `exit` completed; cause is unknown. The exact test app/sidecar PIDs were terminated. A clean instance passed second-open dialogs/cancellation and normal quit. Fresh native exit code 0 is not claimed; earlier c663 evidence retains exit 0.

## What changed

- Product, current-state, desktop, parity, README and roadmap records now identify completed static publication while retaining open hosted/native/full-runtime work and dated history. Mobile/accessibility/cutover checks remain open after this static publication.
- The unrelated `float-kernel-fusion.md` missing `## What is missing`, `infra-desktop-client-tauri.md` unsupported `in_progress`, and stale `issues.json` IDs `float-kernel-fusion` and `infra-desktop-client-tauri` remain disclosed, not edited or presented as passing.
- Current main's CPU model and notebook changes are preserved byte-for-byte. The merge introduces no application/model/API/engine/worker/tokenizer/publisher change, dependency, `.github/**` edit or second publish.
- Final size: **13 documentation/task-record files, +163/-23** (0 code, 0 tests, 0 generated application assets). The previous estimate covered the same named status-record files; this REPORT and its generated receipts are the only added rows beyond that evidence.

## Measurements

- Merged publisher PR #3897 is `a8e51c0f4d0ed6e999b6d9e13f8289215efbd684`, tree `0f41d59fb5fe10291f5b5c4bef0a190081c50484`. Pages run `42668187497` completed successfully at `6540e2717d3a606330dbd141e3642f3f13b87ecf`.
- Existing valid publication evidence is unchanged: fresh-main explicit staged Playwright **45/45**, direct staged WASM **7/7**, 14 requests with zero external/raw-TypeScript/runtime errors, and exact arithmetic `42`. The staged bundle is 806,759 bytes with SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`.
- Live/local `models.json` are each 4,365 bytes with SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`, including both M1 and research FBS seeds. Public desktop and restored mobile primary show the neutral workbench; split/secondary close works, and Terminal explicitly reports its static-browser PTY limitation.
- Evidence and screenshots are under `local/kimi/typed-publish-main/a8e51c0f4d0ed6e999b6d9e13f8289215efbd684/`. Store revisions `a1814d1e3fe0c9b109aab3de7e77ce395d929700` and `b2581dda7782c759e1ae106f74d3cc95732ce5d6` were downloaded and recursively compared: **61 files** and **120 files** matched respectively.

## Checks run at the head

- At clean merge commit `155f811f25ce942326d9a5d08415c6ba0945727d`, `git status --porcelain` was empty before and after `uor-harness loop gates --full`; all **11/11** gates are GREEN. Logs and ledger excerpts are under `local/kimi/typed-publish/155f811f25ce942326d9a5d08415c6ba0945727d/`. Ledger rows 89–99 name that commit and working-tree hash `15842a4c22f187577d9d47e64b14ce3745daa2c2`. That ledger hash is the harness's working-tree identifier, not Git's `HEAD^{tree}` `0ae1f11f53783f2aabc45a4ec0a79bfde3eba072`; the empty-before/after status and commit binding are the clean-head proof, not an assertion that these different identifiers should match.
- `uor-harness loop pr-check --local` at that commit confirmed 0 code/test/generated files and passed order, scope, banned-presence, owner-paths, rule-sources, rendered-lock and read-path. It failed only the then-missing REPORT; commit-order and existing FILEMAP gaps are disclosed advisories. Final policy and **11/11** gates are rerun after this REPORT is committed, with the exact final head and clean ledger/log retained.
- The final GitHub diff must remain records-only. Fresh exact-head review, required checks, protected queue acceptance, actual merge and fresh-main verification remain outstanding at this report's writing. No gate or review result is inferred from the earlier live deployment.

## Skills used

Adversarial review reception, solution-integrity and scope guards, explicit merge-conflict inspection, working-discipline evidence verification, and verification-before-completion.

## Still open

Final clean-head checks, fresh exact-head review, GitHub required checks, protected queue acceptance and actual merge, fresh-main identity verification, final evidence preservation, then this lab's merged worktree/reproducible-output cleanup. Full agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, connectors, hosted native workflows, live R4 API, signed release and useful-model behavior remain unbuilt. The next implementation draft is `local/kimi/studio-agent-order-draft.md`; it must wait for Claude's actual streaming/tool API contract before the live adapter is claimed.

## Plain summary

The live neutral workbench was already published. This repair makes the records agree with that deployment and proves the final documentation-only head on a clean tree. It does not claim full Cowork parity or a working hosted R4 API.
