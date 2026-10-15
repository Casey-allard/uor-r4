# #1652 — Connect geometric occurrence reads to actual BPE answer loss

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T17:38:13Z
- **Updated:** 2026-10-09T13:17:31Z
- **Closed:** 2026-10-03T21:07:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1652
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/geometric-source-consumer-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Correct retrieved values reach the grounded language input, but the retained contrast still has four distractor-following answers. This adds an integer geometric selected-record occurrence reader and connects its exact BPE copy weights and NoRead to ordinary answer loss, preserving the entire pointer-aware language parent.

At tested source `b3fcbc4a`, one complete teacher-forced answer (four BPE tokens plus EOS) yields finite nonzero context, potential and NoRead gradients in every row, with zero optimizer updates. Exported-payload reload reproduces native traces, disabled mixing preserves parent score bits, loss agrees with the native target NLL, and a changed packed payload rejects. The run takes 55.44s real and maximum RSS 1,020,280,832B. Four focused tests pass; independent source and outcome review approve the bounded result.

This is a connected hybrid construction, not fitted or autonomously generated chat. Typed frame metadata is retained outside scoring; content/age/held-span channels are inactive. The parent/mixture remains floating, Stop is not separately learned, and non-source vocabulary ratios still depend on the parent. Source/native/config/data/executable bindings, the constructor-error attempt and corruption negative remain retained. No shared session, compiler, corpus, CUDA/Metal or peer worktree was edited.

The owner clarified that making geometric chat work precedes beating transformers. The roadmap now advances a bounded geometric learning continuation after complete B8 cost measurement; learned reader controls follow for attribution and improvement, rather than blocking the first working mechanism. Comparative claims keep their information/capacity/cost requirements.

Validation: offline connected-example compilation, two integer occurrence/rejection tests, two training mixture/null/zero-mass/disabled tests, actual source-pinned construction, formatting of changed Rust files, claim-wording and whitespace checks. Test-profile dependency compilation is not a serving-performance benchmark. The five historical required CI statuses are delivery acknowledgements, not executed validation.

References #1552 and #1512.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5971776027 — 2026-10-03T17:42:57Z — Casey-allard

Independent source/outcome review approves tested source b3fcbc4a and independently checked report SHA4729ea02663094468f865884a460fa7fe6d5b187a74663fa9479854672ccf000. Independent documentation review approves exact delivery head09627a0a, including the owner correction: working geometric chat first, comparative wins later. Changes after tested source are only the four listed documentation/evidence paths; numerical source is unchanged. One teacher-forced answer/five prediction rows/zero updates; floating parent and mixture remain explicit. Protected auto-merge enabled, no bypass; queued acknowledgement jobs are not executed tests.

### comment 5971790238 — 2026-10-03T17:44:43Z — Casey-allard

Executed from exact delivery checkout09627a0a: cargo test -p uor-r4-integer -p uor-r4-training --lib geometric_occurrence --offline: integer2 passed/0 failed; training2 passed/0 failed. Complete build1m00s; no changed numerical source after construction b3fcbc4a. Focused binaries and SHA receipts retained at ~/uor-r4-local/workspace/research/geometric-source-consumer-20261003/exact-head-focused-tests.json. Source-pinned construction executable and all source/native/negative artifacts retained. Only this track disposable CARGO_TARGET_DIR was removed after execution; no peer model, checkout or cache cleanup.

### comment 5973492493 — 2026-10-03T21:07:54Z — Casey-allard

Construction delivery is superseded by merged successor #1658 (merge commit bc967c3909d135acc3a7c44be67f45e35ba6a5f7). That successor includes the construction source plus lexical/native realization continuation and preserves construction evidence. Closing this duplicate PR without deleting its branch; wider #1552/#1512 acceptance remains open.


---
_Comment/review ids in this file: 5971776027, 5971790238, 5973492493_

