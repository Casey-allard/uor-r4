# #2120 — The pointer trace is BUILT and BROKEN, and the v5 "memory" category carried no memory reader (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:19:28Z
- **Updated:** 2026-10-10T03:23:28Z
- **Closed:** 2026-10-10T01:20:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2120
- **Merged:** 2026-10-10T01:20:52Z
- **Merge commit:** 976763808ea119be7aae9045914010e9f6ef2171
- **Base / Head:** main / UOR-Foundation:numeric-pointer-trace
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## 0. The structural finding, first, because it outranks the result

**THE ARTIFACT THAT ANSWERED THE v5 "MEMORY" PANEL CARRIES NO MEMORY OPERATOR AT ALL.**

`chat-29m-B-lr5e-4`'s entire config is `arch`, `vocab_size`, `width`, `heads`, `mlp_hidden`, `context`, `pattern`, `read`, `rotation`, `seed` and a **`pointer`** — a copy operator and **no memory reader**.

- `memory_read_diagnostic` returns `None` **by construction** on every row.
- **The v5 memory rows never exercised a memory reader.** What the panel measured was **the plain dialogue stack with a copy pointer**.
- **The qualifier is now on the v5 acceptance record and the current-state entry:** "10 of 40 on the memory category" is 10 of 40 for a **copy-pointer dialogue stack on requests *named* memory rows**, not for an addressed-memory path. **The category name does not describe the mechanism that answered it.**
- It reframes the standing reading: "the learned read/emit path does not deliver the value" is, on this artifact, "**the pointer** does not deliver the value".

## 1. The trace: built, run, and BROKEN by its own pre-registered positive control

`crates/uor-r4-training/src/bin/numeric-pointer-trace.rs` turns on `set_pointer_copy_trace`, generates each row through the **same** `reply_panel` → `greedy_reply_with_copy_stop` path `chat-grade reply` uses, and records per step the emitted id against the pointer's selected `source_id`, compared **as token ids, never strings**. It ran on all 13 numeric rows in **10.6 s**:

```text
READER: 12, PARTIAL: 1
```

**BY PRE-REGISTERED CONDITION 2, THAT IS NOT A RESULT.** `mem-040` — the positive control, the one numeric row that emits the stored value exactly (*"Your sister's locker number is 84."*) — traces as **`READER` with 0 digit tokens selected**. **The control failed, so the trace is not observing the path that produced the v5 replies, and every classification is void.**

## 2. The evidence it is broken, not merely surprising

**`mem-040` and `mem-006` emit BYTE-IDENTICAL id sequences in the probe:**

| row | sealed v5 reply | probe emitted ids |
|---|---|---|
| `mem-040` | *"Your sister's locker number is 84."* | `[2997, 1700, 386, 1412, 435, 772, 291, 69, 1790, 292, 80, 71, 16, 1]` |
| `mem-006` | *"I'm a helpful assistant that runs on your computer."* | `[2997, 1700, 386, 1412, 435, 772, 291, 69, 1790, 292, 80, 71, 16, 1]` |

Two rows share a 12-id reply, two share a 14-id reply, two share a 9-id reply. **The probe produces history-insensitive replies** — the row's own turns are not reaching the model on this path — so its per-step selection cannot be compared to anything. **The reader-versus-emitter split is still NOT MEASURED, and this PR does not report one.**

## 3. What the next attempt must do

1. **Make the probe reproduce the sealed replies byte for byte** on the same rows and artifact — the missing control — before any classification is trusted.
2. **Diagnose the history construction** (13-row subset versus the whole panel) rather than guess at it.
3. **Do not re-run this probe as-is** — a trace whose positive control fails cannot be repaired by more rows.

## 4. The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The learned read/emit path | **OPEN, with a new structural qualifier rather than an answer** — no memory reader on this artifact, so the question is the pointer's, and the trace is built but not yet trustworthy |

Five pieces into this line, every expensive step has been avoided by a cheap one that ran first — and this piece is that pattern working again: a broken instrument was caught by its own pre-registered control in a 10.6-second run, before any of its 13 classifications reached a conclusion.

## Delivery

Record [`docs/labs/numeric-pointer-trace-2026-10-09/README.md`](docs/labs/numeric-pointer-trace-2026-10-09/README.md); the probe bin; **the v5 record and current-state qualifiers**; newest-first current-state entry ending in **Next:**; bundle on cloud-store before any local deletion: `numeric-pointer-trace-2026-10-09.tar`, 60,416 bytes, md5 `3ccbfce7ba22318ac2bbdb2f30b780f0`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

