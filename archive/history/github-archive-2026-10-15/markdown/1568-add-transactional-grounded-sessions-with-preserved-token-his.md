# #1568 — Add transactional grounded sessions with preserved token history

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T07:11:23Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-10-01T07:22:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1568
- **Merged:** 2026-10-01T07:22:42Z
- **Merge commit:** fd505996bdd433f6c5c9e995411f2c9e31b362af
- **Base / Head:** main / UOR-Foundation:codex/grounded-session-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A saved model/store checkpoint could not continue an ordinary conversation while preserving its exact generated token history or consuming a persisted compiler. This adds the development session boundary for #962: predicted relation/action/source spans feed the existing exact store, and replies use the actual pointer-aware stack emitter. Memory and transcript commit together after successful generation.

The session saves a separate sealed envelope with the model/store checkpoint, opaque compiler artifact, tokenizer/protocol, source turns, generated IDs and explicit context/storage limits. It supports strict history or the largest fitting suffix of complete turns while retaining durable memory. An additive tokenizer method appends user and recall messages without re-encoding earlier generated IDs. Found-but-unrepresentable values, conflict and eviction remain distinct from proven absence.

**Scope:** this is a float development library. The real saved compiler/span adapter and ordinary-text CLI remain an explicit dependency; injected test actions cannot satisfy integrated learned-input acceptance. It does not establish grounded-language quality, D11 serving, D5 selected access or full #962 completion. Full-store staging and retained diagnostic input records remain prototype costs. Current-state records these limits and the separate artifact-bound paraphrase-label diagnosis.

**Validation:** [exact-head CI 36829014641](https://github.com/UOR-Foundation/uor-r4/actions/runs/36829014641) passed formatting/policy checks, library/current-consumer compilation, 8 tokenizer tests and 6 session tests. The child helper is ignored by the parent harness and explicitly passed in a fresh subprocess. [Nonauthor review](https://github.com/UOR-Foundation/uor-r4/pull/1568#issuecomment-5926572797) approves head `2f0449641576df63e5fe96824ea26133c01ff6a5`. Queue compatibility acknowledgements are not test results; the advisory publisher Not Found failure and D19 manual fallback are recorded in the [validation receipt](https://github.com/UOR-Foundation/uor-r4/pull/1568#issuecomment-5926679934). The fresh-process fixture uses an initialized geometric pointer model, not the trained emit-1/compiler pair. Actual trained integration remains NOT_RUN.

Lab: Codex. Class: shared development interface under owner-adopted D19. References #962, #1552, #964; #962 stays open. No local Cargo/model worker or changes to Claude's compiler/data/job were made.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5926572797 — 2026-10-01T07:14:22Z — Casey-allard

# Independent grounded-session source review

Verdict: **APPROVE the scoped source change; no required code fixes remain.** This is a nonauthor source review, not an execution result or authorization to close #962. Actual compile/test results remain the delivery owner's responsibility.

- Reviewer: Codex subagent `/root/adoption_review_evidence`; authored none of the reviewed files or commits.
- Exact reviewed head: `2f0449641576df63e5fe96824ea26133c01ff6a5` (PR #1568).
- Reviewed base: `695ed34b5afddfb4bb639b9a80e35385c30f6237`.
- Worktree: `/Users/casey.allard/uor-r4-worktrees/grounded-checkpoint-snap-20261001`.
- Scope: the new `stack_grounded_session.rs`, tokenizer assistant suffix, module registration, and current-state claims. Inspected their actual store/checkpoint/tokenizer/greedy-emitter contracts. Worktree was clean when the final head was confirmed.
- Exact new module SHA256: `00e311385d3b82c6cf20d87ad1d3d15d6bacc13c357205b8ca389b900fd96647`.

## Material checks and dispositions

1. **The turn transaction has the right boundary.** Original text is compiled once; UTF-8 byte spans are checked against that unchanged text, with lossless isolated-span tokenization and rejection of protocol/OOV tokens. Store writes are staged. Bounds, recall encoding and actual pointer-aware greedy generation must succeed before store/history/turn records are replaced. Assert and Correct remain distinct. See `stack_grounded_session.rs:458-534`, `:550-574`, and `:579-636`.

2. **Persistence is bound to actual records, beyond a matching commit count.** Earlier review found this missing; the final head fixes it. A write receipt must match its saved address/chain position, revision, record ID, commit, source turn, action, conflict, predecessor and exact value bytes. Current-query receipts must match the actual head visible at that turn's pinned commit. The check uses retained immutable value bytes, so a payload evicted by later writes does not invalidate a genuine earlier Found receipt. Base `StackStore::from_bytes` already validates record encoding and the core chain invariants. See `stack_grounded_session.rs:918-1106`, `stack_store.rs:535-550`, and `stack_store.rs:601-623`.

3. **Tokenizer and incremental history agree.** Earlier review found a model/tokenizer extent mismatch that could silently decode generated IDs away. The final constructor requires equal extents and nonempty decoded bytes for every slot, including holes left by sparse added-token IDs (`stack_grounded_session.rs:352-366`). The tokenizer suffix delegates to the existing segmented encoder with no BOS, preserves role/content segmentation and normalization, and inserts only the needed separator (`dialogue.rs:205-234`). Previously generated reply IDs are retained directly, not decoded and retokenized. EOS/cycle/cap metadata is checked against `Reply::stop`, with caller EOS tracked separately (`stack_grounded_session.rs:788-809`).

4. **Model context is separate from durable storage.** WholeCompletedTurns retains the largest fitting suffix of completed turns and the entire current user/recall turn, reserves reply and closure capacity, and reports the first retained turn. StrictFullHistory remains available. Old-window removal does not remove full source/generated history or exact store records. An oversized current turn fails rather than clipping source spans or values. The documented state reset and removed pointer candidates are real behavioral consequences, not full-history equivalence. See `stack_grounded_session.rs:704-785` and `current-state.md:70-79`.

5. **Unsupported recall values are not fabricated absence.** Literal `none` and carriage-return values remain losslessly storable. Their Found read is retained while textual recall is marked Unsupported and omitted. Conflict, unresolved, no-history and eviction do not become `Memory: none.`. NoRead retains the real read receipt while disabling its injection; UpdateDisabled does not mutate the store. See `stack_grounded_session.rs:640-701` and tests beginning at `:1534` and `:1565`.

6. **The fresh-process test is materially a fresh process.** The parent saves before the next actual pointer-aware generation, invokes the same test binary with `--ignored --exact stack_grounded_session::tests::fresh_process_child`, and requires a separately written, parsed child outcome equal to the parent outcome. A misspelled selector running zero tests cannot pass because the required output file would be missing. The child reloads the actual sealed model/store/tokenizer/compiler fixture and checks the snap/pointer configuration. See `stack_grounded_session.rs:1773-1814`. This is an initialized tiny model with a clearly test-only compiler and an injected first response; it is not a trained R1/S4 or learned-compiler witness.

## Test and claim boundaries

Source tests cover span/update/conflict distinctions, storable-but-unsupported recall values, rollback on input/generation/context/store-bound errors, controls, whole-turn windowing, sealed snapshot identity, independently resealed history/write/read receipt corruption, and the separate-process fixture. Tokenizer tests cover segment boundaries, concatenation equivalence, masks, skips and literal specials. These are descriptions of test coverage inspected in source. **I ran no Cargo command, test executable or model worker; no PASS or runtime qualification is asserted by this review.**

The final documentation correctly calls this a floating-point development consumer. The real saved compiler/span adapter and ordinary-text CLI remain missing in the inspected integration source. The trait and synthetic fixture are not that adapter. Actual learned text-input construction, changed-source/NoRead/UpdateDisabled behavior, trained artifact continuation, temporal/scope/conflict acceptance, D11 serving and useful language remain unqualified by this change. #962 stays open; the delivered boundary is partial and reviewable.

No broader proof campaign or new model experiment is required by this review. If execution exposes a failure, repair that concrete failure and re-review the resulting delta at its new exact head.


### comment 5926679934 — 2026-10-01T07:21:02Z — Casey-allard

**Executed exact-head validation:** PR head `2f0449641576df63e5fe96824ea26133c01ff6a5`; [GitHub run 36829014641](https://github.com/UOR-Foundation/uor-r4/actions/runs/36829014641). The workflow's separate CI commit checks out and verifies this exact PR head.

- Claim-wording, policy JSON, whitespace and `cargo fmt --check`: PASS.
- `cargo check -p uor-r4-training --lib --example m-world --locked`: PASS, 1m 00s.
- `cargo test -p uor-r4-tokenizer --lib dialogue::tests --locked -- --test-threads=2`: **8 passed, 0 failed, 0 ignored**, tests 0.01s after 4.82s compilation.
- `cargo test -p uor-r4-training --lib stack_grounded_session::tests --locked -- --test-threads=2`: **6 parent tests passed, 0 failed; 1 child entrypoint ignored by the parent harness and explicitly executed successfully in a fresh subprocess (1 passed, 0 failed)**. Parent tests took 0.09s after 4m 45s compilation; child execution 0.02s. The ignored helper is not an unexecuted capability test.
- [Nonauthor review](https://github.com/UOR-Foundation/uor-r4/pull/1568#issuecomment-5926572797) approves this exact head. No required fixes remain.

These are executed interface/persistence tests. The child loads the actual initialized geometric pointer model with icosian snap, test-only compiler and retained prior generated IDs, then compares the complete continuation result. This is not the trained emit-1/learned-compiler witness. That real adapter/CLI and full #962 acceptance remain open. There is no D11, D5, useful-language or energy qualification.

The five historical queue checks are compatibility acknowledgements. The separate advisory publisher [run 36828959992](https://github.com/UOR-Foundation/uor-r4/actions/runs/36828959992) failed with **Not Found**, and is not represented as PASS. D19's adopted manual fallback permits exact-head review plus actual scoped checks and protected queue delivery while that publisher is unavailable. No ruleset is changed or bypassed. No local Cargo/model worker ran.



---
_Comment/review ids in this file: 5926572797, 5926679934_

