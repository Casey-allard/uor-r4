# #1602 — reencode-dialogue: exact protocol-2 copies of prepared dialogue splits (chat-v0 in /2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T05:29:03Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-02T06:16:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1602
- **Merged:** 2026-10-02T06:16:25Z
- **Merge commit:** e7d7604b808dd6f616cc214fd4b50986208bc081
- **Base / Head:** main / UOR-Foundation:codex/reencode-dialogue
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 (D19), #1591 (protocol 2).

chat-v0's raw-to-token preparation tool is not in the repo (its README records only raw text). Protocol 2 needs chat-v0 encoded in `/2`.

**`reencode-dialogue in=SPLIT out=NEW tokenizer=T`** decodes each protocol 1 document back into messages by the literal-role template. A document is kept **only if re-encoding those messages in protocol 1 reproduces its tokens and response mask exactly**; it is then written in protocol 2. Same documents, same sources, same split.
- The manifest's counts and digests are recomputed.
- `dialogue_protocol` is declared, and the source split is recorded under `reencoded_from`.
- Documents that fail the round trip are dropped and counted.

**Measured on chat-v0** (t4-1433 tokenizer, `d36d3e87`):
- held-out: **6,526/6,526** documents;
- train: **73,625/73,632** documents, 82.2M tokens. The 7 dropped documents contain marker-like text.
- 17 s for the training split.

**`m-world corpus`:** `chat=` is now merged only in the protocol it declares (absent = 1), in place of the blanket refusal under `protocol=2` (#1591).

**Checks:** template-parse unit test; `m-world` example tests (6); the real round trip above. Evidence branch: `codex/ci/reencode-dialogue`.

**Used by:** the running `/2` instruction-recovery arm (emit-3p2 + chat-v0 `/2` + M-world recall, #1552).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5946601532 — 2026-10-02T06:15:31Z — Casey-allard

**Exact-head review: `d979a7805e6963f037590d3edeeec3cf45fa5bb6`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- `reencode-dialogue` keeps a document only if its parsed messages re-encode in protocol 1 to the exact tokens and mask; a wrong parse cannot pass.
- It writes into a new directory only (`create_dir`), refuses a split that already declares a protocol, and recomputes counts and digests.
- `m-world corpus` merges `chat=` only when the declared protocols match (absent = 1); protocol 1 corpora are unchanged.
- Measured: chat-v0 held-out 6,526/6,526 and train 73,625/73,632 documents round-tripped.
- The PR merges cleanly with main and with #1604.

**Evidence at this head:** fmt and the check of all targets succeeded; `uor-r4-training` lib 517 passed, 0 failed; `m-world` examples pass.

Verdict: approve for merge at this head.

### comment 5946702065 — 2026-10-02T06:25:38Z — Casey-allard

**Merged** as `e7d7604b` at head `d979a780`; main's tree equals the PR head's tree.

### comment 5946704225 — 2026-10-02T06:25:51Z — Casey-allard

**Correction to the merge note above:** main's whole tree does not equal the PR head, because #1599 and #1601 merged in between. The files this PR changes (`src/bin/reencode-dialogue.rs`, `examples/m-world.rs`) are byte-identical at `e7d7604b` and at the reviewed head `d979a780`.


---
_Comment/review ids in this file: 5946601532, 5946702065, 5946704225_

