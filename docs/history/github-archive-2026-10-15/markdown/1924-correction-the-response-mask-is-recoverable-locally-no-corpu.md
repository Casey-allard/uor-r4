# #1924 — Correction: the response mask is recoverable locally — no corpus rebuild needed (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:44:36Z
- **Updated:** 2026-10-09T13:21:11Z
- **Closed:** 2026-10-08T15:46:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1924
- **Merged:** 2026-10-08T15:46:20Z
- **Merge commit:** ca352be6bfb2ab94725767f8be1ffb75a039d94d
- **Base / Head:** main / UOR-Foundation:deepseek/mask-recoverable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What round 49 got wrong

Round 49 concluded that the objective change "cannot be run on what is on disk" and that rebuilding the corpus from source parquet "appears to be the only route." **The second half is wrong, and the source says so.**

## What the source shows

1. **The builder already emits a mask.** `prepare-chat-parquet.rs` computes `encoded.response_mask` per conversation (lines 151, 162–163) and writes it — **line 210: `let mask_path = out.join("response_mask.u8");`** — plus a `response_tokens` statistic. The mask is a first-class output, not missing machinery; it's simply absent beside the corpus copy on this volume.

2. **The corpus is literal-role, which is what makes recovery possible.** The builder prepares a *"literal-role dialogue split"* via `DialogueProtocol::literal_roles_version(&tokenizer, version)`. The role structure is **rendered into the token stream as literal marker tokens**, not discarded.

3. **The marker is derivable from the tokenizer and protocol version alone.** `stack_dialogue.rs:333-352` builds an `EpisodeContract` from `literal_roles_version(...)` + `encoder.encode_assistant_prefix(&[])` → **`assistant_marker_ids`**, documented as *"the assistant marker that opens each scored response."* **Nothing in that path reads the source conversations.**

**So the response mask can be reconstructed from the existing `chat-train.u16`**: scan for the assistant marker ids, mark what follows each marker as response until the next role marker or BOS, write `response_mask.u8`. Required: the local tokenizer (sha known), the protocol version (2, in the runbook), and the marker logic above. **No parquet, no re-fetch, no new licence decision.**

## Corrected recommendation — and I'm withdrawing my own question

**Round 49's owner-facing question is withdrawn.** The objective change does **not** require the ~2.1 GB rebuild. The rebuild stays justified only if the project wants *more or different conversations* — and every piece of evidence this session produced says it does not need them for this purpose. What it needs is a **mask extracted from the corpus it already has**.

The question for you, if any, is now just whether to spend the effort on the objective change itself.

## Three checks before trusting the extraction

1. **Scan `chat-train.u16` for the assistant markers** — counts and spacing; also yields the response fraction, checkable against the builder's `response_tokens` statistic if any original manifest survives on the volume.
2. Whether `dialogue-train`'s episode framing needs sidecars beyond mask + manifest (it takes `train_tokens`, `train_mask`, `train_manifest` together).
3. **Which protocol version the corpus used** — the runbook says 2; a marker scan for both candidates settles it empirically.

Scope: all of the above is read from source and the tool's own output paths. **No scan has been run and no mask produced yet** — this corrects a *recommendation*, not a measurement. Evidence: `docs/evidence/mask_recoverable_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

