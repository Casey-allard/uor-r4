# #1591 — Opt-in literal-role dialogue protocol 2: the space after a role marker belongs to the message

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T01:00:23Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-02T03:00:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1591
- **Merged:** 2026-10-02T03:00:56Z
- **Merge commit:** 55c5dcc5944088abbaad4e864ec76ecf0f22af05
- **Base / Head:** main / UOR-Foundation:codex/dialogue-protocol-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552, #1512.

**Owner decision (1 October): add an opt-in literal-role dialogue protocol 2.**

**The problem.** Copy is 0/33 in every M-world model, whatever the attention mechanism (soft, exact or ranked prime routes; #1512).
- Protocol 1 (`uor-r4-tokenizer/src/dialogue.rs`) encodes the marker `Assistant: ` with its trailing space, and the reply content as its own segment. So a reply's first word ("mamverkkul") never has the leading-space token it has mid-sentence in the user's message (" mamverkkul"), and no pointer can copy it.
- emit-1's copy replies show this exactly: `famverkkul sterkhitbeend cheedtre vutark zenlem lasarkbrim` against `mamverkkul …`. The middle of the span is copied; the first token is not.

**Protocol 2** (`uor-r4.literal-role-dialogue/2`).
- Markers end at the colon: `User:`, `Assistant:`, `System:`.
- Each message's content is encoded as `" " + content`, so every word, a reply's first included, is the token it is mid-sentence.
- Separators, BOS/EOS and masks are protocol 1's. The reply's leading-space token is part of the response.
- `DialogueProtocol` gains `literal_roles_v2`, `literal_roles_version`, `spaced_content`, `assistant_marker` and `reply_text` (drops the one leading space from a decoded reply).
- `bind` accepts both schemas, and the schema is part of the protocol identity.
- **Protocol 1 is byte-for-byte unchanged.** Every existing artifact, session and other lab's path keeps it.

**Opt-in plumbing.**
- `stack_dialogue::episode_contract_for(.., version)`: the assistant marker that locates scored responses is the version's.
- `m-world corpus | evaluate protocol=1|2`, validated before the report root is claimed (world=v2 only, never with `chat=`, whose documents are protocol 1).
  - A version 2 corpus records `dialogue_protocol` in its manifest.
  - `evaluate` scores `reply_text` and records the schema.
- `geometric-stack dialogue-train protocol=1|2` for both contracts. It appears in the run record and lineage only when 2, so existing checkpoints resume unchanged.
- M-world v2's meter gains `Meter::spaced`. Conversations are still drawn under protocol 1's meter, so both versions encode the same conversations (a paired design). A version 2 document is checked against the spaced meter.

**Checks run locally** (head `a71771ce`).
- `uor-r4-tokenizer`: 13 tests, including protocol 1 vs 2 token streams on a fixture with a `" H"` merge. The reply's first word is the user's token only under 2. The test also covers masks, the prompt marker, `reply_text`, incremental-encoding equality, identity, round trip and refusal of an unknown version.
- `m-world` 6 tests (protocol parse included) and `geometric-stack` 9 tests.
- The ignored real-tokenizer test (`real_tokenizer_episodes_fit_the_context`) passes with the t4-1433 tokenizer. Over 3,000 conversations every protocol 2 document matches `Meter::spaced`, and protocol 1 still matches the original meter.
- Paired corpora: 5,600 conversations, 20,212 responses in both versions; protocol 2 uses 5% fewer tokens.
- GitHub evidence: `codex/ci/dialogue-protocol-v2` (workflow v3), for the tokenizer lib, the training lib and both examples.

**Measurement running.** P1 runs on free GitHub runners (`codex/ci/farm/protocol-p1`; emit-1 published there by owner decision). It fine-tunes emit-1 on the identical conversations under protocol 1 vs 2 and evaluates on M-world v2 development in each protocol. Results will be added here. No language result is claimed by this PR.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5943994512 — 2026-10-02T01:40:29Z — Casey-allard

Measured result for this PR (P1): copy 1/33 → 10/33 under protocol 2 (item-paired McNemar p = 0.004), other categories not significantly different. Full table: https://github.com/UOR-Foundation/uor-r4/issues/1552 (latest comment).

### comment 5944145921 — 2026-10-02T01:55:22Z — Casey-allard

**Task A — review of #1591 at exact head `a71771ce42b706f44b8b4f7707595d7da90492f0`. Verdict: PASS. Recommend merge.**

**Exact head reviewed:** `a71771ce42b706f44b8b4f7707595d7da90492f0` (branch `codex/dialogue-protocol-v2`, base `main`, mergeable, 5 files, +311/−28).

**Executed checks — all four the card names, all green at this head.**

| # | Check | Result |
|---|---|---|
| 1 | `cargo test -p uor-r4-tokenizer` | **13 passed, 0 failed** |
| 2 | `cargo test -p uor-r4-training --example m-world --example geometric-stack` | **9 passed + 6 passed, 0 failed** |
| 3 | `UOR_R4_TOKENIZER=…/bundle-learned-1/tokenizer.json cargo test -p uor-r4-training --lib -- --ignored real_tokenizer_episodes_fit_the_context` | **1 passed, 0 failed** (23.6 s) |
| — | total | **29 passed, 0 failed, 0 ignored** |

The tokenizer at `/Volumes/UOR-Workspace/uor-r4-lab/claude-t4-1433-resume/bundle-learned-1/tokenizer.json` was readable (109,457 B) and the ignored test ran rather than skipping.

**The four substantive checks.**

**(a) Protocol 1 is byte-identical — by construction, not by assertion.** The refactor turns `literal_roles_v1` into `Self::literal_roles(tokenizer, SCHEMA)`, and the old body becomes the shared `literal_roles` with only the schema string parameterised. The encoder's new `match (message.role, spaced)` adds three arms; the three `spaced == false` arms carry **the original three literals unchanged** (`"System: "`, `"User: "`, `"Assistant: "`), and `spaced == false` also takes the original `self.append(&mut out, content, …)` path with no space injection. I checked this specifically because a marker-literal change is exactly the kind of edit that silently alters a corpus. It does not. Confirmed by the tokenizer suite passing.

**(b) Protocol 2's streams and masks are correct.** Marker ends at the colon; content becomes `" {trimmed}"`, so the leading space is inside the message. That is the right call: it means the assistant content — and the space with it — carry mask value 1, so the space is scored as part of the response rather than sitting unmasked on the marker side. `reply_text` drops exactly that one space via `strip_prefix(' ')`, which is the correct inverse of `format!(" {content}")`. Empty content is still skipped by the pre-existing `skipped_empty` guard **before** the marker is appended, so `format!(" ")` is never emitted for an empty message — I checked that ordering because a space-only assistant turn would have been a masked-empty-target defect.

**This change fixes a real defect, and I have independent evidence for it.** In an earlier review I traced the `0/33` copy failure to exactly this boundary: under the #1017 tokenizer the reply-initial `m` is id **79**, while the same letter after a space in user text is `Ġm` = id **283**, and token 79 never occurs in user input — so a copy mechanism sourcing from user words could not emit it. Putting the space into the content makes the reply's first word encode as it does mid-sentence. This is the correct fix for the right reason.

**(c) The paired-meter design is sound.** `Meter::spaced` is a **separate constructor** that changes three counted strings (`"User:"`, `"Assistant:"`) and sets `spaced: true`; `Meter::text` then counts `format!(" {}", text.trim())`. The v1 meter is untouched, so the `/2` accounting the card asked me to check is a genuinely paired comparison rather than one meter being adapted in place. The identity it relies on — `count(" ") + count(word)` equals `count(" word")` — holds for byte-BPE because the leading space attaches to the first token of the following word.

**(d) `protocol=` is validated before anything is claimed or loaded.** The value is parsed inside the `DialogueSettings` literal (which returns early on an invalid value) at the point of settings construction; `report_output::claim` and the corpus loads happen later inside `dialogue_train`. The existing comment at that site — *"Validate the A1 options before anything is claimed or loaded"* — is where the new arm sits, so the new option inherits the correct ordering. A dedicated test, `the_protocol_version_is_one_or_two`, covers the refusal.

**Two further things I verified because they are the kind of change that breaks instrumentation.**

- **Existing run records are unchanged.** `protocol` is added to the report and lineage records **only when it differs from 1** (`if self.protocol != 1`), so v1 runs keep byte-identical records. This matters for comparing against S2 and my current `chat-v0-phase-a-1` run.
- **`episode_contract_at` is preserved as a delegation** to `episode_contract_for(…, 1)`, so every existing caller keeps v1 semantics without being touched.

**No blocking findings.** Two non-blocking notes for the record, neither a defect:

1. `literal_roles_version` takes a `u8` and refuses anything but 1 or 2 with a typed `UnsupportedSchema`. Good. If a version 3 arrives, the refusal message names the version — worth keeping that style.
2. The `SCHEMA_V2` doc comment states that v1's separators, BOS/EOS and masks are unchanged, and I verified that against the code rather than accepting it. It is accurate.

**Scope of this review.** Executed at the exact head with the four named commands; I read the full diff of all five files. I did **not** re-run the corpus-scale or training-scale behaviour of protocol 2 — that is a measurement question, not a review question, and the card scoped this as a review. Anything that consumes protocol 2 at scale should still report its own numbers.

Next on my queue: Task B (the learned route gate), for which I now hold the pointer-path file ownership.


### comment 5944531949 — 2026-10-02T02:33:30Z — Casey-allard

**Exact-head review: `370d26804b757ae3e7df69a2c4d64e8e84ae4ea9`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- Protocol 1 is byte-for-byte unchanged: fixture test, existing tests, and the real-tokenizer meter test (`/1` documents still equal `Meter::new`).
- Protocol 2: markers end at the colon; content is encoded as `" " + content`; the reply mask covers the reply's leading-space token and EOS.
- `bind` accepts both schemas; the schema is part of the identity.
- `protocol=` is validated before any root is claimed and is refused with `chat=`.
- `dialogue-train`: `protocol` is in the run record and lineage only when 2.
- Corpora are drawn under the `/1` meter (paired) and checked against `Meter::spaced`, 3,000/3,000 with the real tokenizer.
- The merge of main at `86188622` is clean. The combined tree with #1593 (`ea183faf`) compiles in every target, and its `m-world` tests pass.

**Evidence at this head.** Run https://github.com/UOR-Foundation/uor-r4/actions/runs/36952936679: fmt and the check of all targets succeeded; `uor-r4-tokenizer` 13 passed; `uor-r4-training` lib 504 passed, 0 failed; examples `geometric-stack` and `m-world` pass.

**Measured** (P1, #1552): copy 1/33 → 10/33 under `/2` (McNemar p = 0.004); no other category differs significantly.

Verdict: approve for merge at this head.


---
_Comment/review ids in this file: 5943994512, 5944145921, 5944531949_

