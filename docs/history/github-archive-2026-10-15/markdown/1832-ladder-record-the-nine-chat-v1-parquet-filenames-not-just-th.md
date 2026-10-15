# #1832 — ladder: record the nine chat-v1 parquet filenames, not just their count

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T05:37:51Z
- **Updated:** 2026-10-09T13:19:29Z
- **Closed:** 2026-10-07T05:39:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1832
- **Merged:** 2026-10-07T05:39:18Z
- **Merge commit:** f403bef9bfe46349180ddc496f52519738703123
- **Base / Head:** main / UOR-Foundation:deepseek/chatv1-recipe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Documentation only.**

## The gap

`ladder-runbook.md` §2.3 wrote the `prepare-chat-parquet` sources as:

```
source='smoltalk_smol-magpie-ultra.train:Apache-2.0:13900:<6 parquet files joined by |>'
source='ultrachat_200k.default.train_sft:MIT:10200:<3 parquet files joined by |>'
```

**chat-v1 is 0.50 of the 96M rung's mix and 1,388,814,239 tokens, and it was not reproducible from the repository** — the filenames appear nowhere on trunk, and the wrapper that first ran it (`/root/run-100m.sh`) was pod-local and is gone. The file is also absent from the canonical volume in any form, so the 96M rung could not be re-run without re-deriving this.

Same shape as two other gaps found this week: the artifact is cited, the path to it is not on trunk.

## The recovery

From the Hugging Face datasets-server parquet index
(`https://datasets-server.huggingface.co/parquet?dataset=<repo>`), which reports
the same counts the runbook asserts:

| source | files | size |
|---|---|---|
| `HuggingFaceTB/smoltalk` — `smol-magpie-ultra`/train | **`0000`–`0005.parquet`** (6) | ~1.4 GB |
| `HuggingFaceH4/ultrachat_200k` — `default`/`train_sft` | **`0000`–`0002.parquet`** (3) | ~0.73 GB |

Both resolve on each dataset's auto-converted `refs/convert/parquet` revision —
verified (HTTP 206 on a ranged request).

## What the section now says

- the **download step**, since `source=` takes local paths and the tool fetches nothing;
- **every file named in row order**;
- the note that **order plus `SKIP` together select the rows**;
- and a line saying to **record the names, not the count** — that's what made this unreproducible.

References #820.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

