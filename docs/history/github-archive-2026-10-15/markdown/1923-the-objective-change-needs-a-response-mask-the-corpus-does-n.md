# #1923 — The objective change needs a response mask the corpus does not have (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:40:28Z
- **Updated:** 2026-10-09T13:21:05Z
- **Closed:** 2026-10-08T15:42:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1923
- **Merged:** 2026-10-08T15:42:15Z
- **Merge commit:** 09a8709f2a403bca5034823c2571c40b8d67e3f9
- **Base / Head:** main / UOR-Foundation:deepseek/mask-absent
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What round 48 implies

The pre-registered form-diversity arm failed on all three measures, so the chat model's failure to bind replies to turns is an **objective or capacity** problem. The indicated change is a loss that distinguishes the **response region** of a conversation from the prompt.

## What the machinery requires — and what's missing

`dialogue-train` accepts exactly that:

```
train_tokens=TRAIN.uort train_mask=TRAIN.mask train_manifest=TRAIN/manifest.json
```

**The mask does not exist for this corpus.** The training data on the volume is `chat-train.u16` (164,496,986 B), `heldout.u16` and `tokenizer.json` — no mask, no manifest, no per-turn sidecar. Searching the whole volume for a mask returns only cargo-registry sources. The corpus was built as a **flat token stream with role templates rendered into the text** (protocol 2), so reply boundaries survive only in the source conversations.

## Consequence — and it flips a recommendation

The objective change **cannot be run on what is on disk**. It requires rebuilding the corpus from source parquet with a mask emitted alongside the tokens — the same ~2.1 GB licence-clean re-fetch and `prepare-chat-parquet` path that rounds 36 and 48 argued *against*.

**But that argument was about volume, and this is not a volume reason.** Stated precisely for the decision:

- **for more tokens of the same register: NO** — the evidence is against it;
- **for a mask that enables the objective change: this appears to be the only route**, and the decision should be made on *that* basis, not on corpus size.

The corpus has already discarded the structure, and no amount of re-reading it locally recovers that.

## The next step is cheap, and it's recorded

Read `prepare-chat-parquet` (on `main`) for a **mask or per-turn sidecar emission option** before assuming new code is required — the runbook's §2.3 documents the source/skip protocol, so the tool may already produce what's needed.

Supporting facts for that step: both sources are licence-clean and already used here (smoltalk `smol-magpie-ultra` Apache-2.0; `ultrachat_200k` MIT), skip counts are recorded, and the tokenizer is local with a known sha256 — so a rebuild is deterministic and verifiable. The held-out stream every measurement tonight used (sha `e5f400b0…`) lives on the laptop and is unaffected.

Scope: this documents an **absence**, verified by listing the data directory and searching the volume, and the decision it changes. **No objective change has been run and no result is claimed for one.** Evidence: `docs/evidence/mask_absent_objective_path_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

