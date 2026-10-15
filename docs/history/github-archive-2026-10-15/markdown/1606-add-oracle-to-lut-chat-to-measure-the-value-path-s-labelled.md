# #1606 — Add oracle= to lut-chat to measure the value path's labelled ceiling

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T06:35:12Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-10-02T13:59:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1606
- **Merged:** 2026-10-02T13:59:58Z
- **Merge commit:** c64825e54abeeed97546631d529cb712b28e91e4
- **Base / Head:** main / UOR-Foundation:codex/lut-chat-oracle-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

A memory row that answers wrongly has **three separable causes** — the value is not retrievable, it is retrievable but not selected, or it is selected but not emitted. **Reply scoring alone cannot tell them apart**, and this lane needed to distinguish them before spending more on memory mechanisms.

`oracle=` supplies the value as an **explicit statement of the relation being asked about**, so a failure with the value present cannot be blamed on the store. It measures the **labelled ceiling** of the value path.

## Change

`oracle=<file.json>` — a JSON object of `request id -> statement`. Each matched request gets **one extra user turn inserted at the front**, encoded through the armoury's own role markers, separators and masking **unchanged**, with the row's real turns following in their original order. A turn is inserted rather than raw token ids so the protocol is not bypassed; requests absent from the file are untouched (so a panel can be mixed). Opt-in: no `oracle=` means the command line and behaviour are exactly as before.

## Exact-head evidence

**Reviewed head:** `8e73a28ac112c81d8a5e0a7fcdd9a72362731a8f` (one commit on `origin/main`).

- **Compile:** `cargo check --features cpu-accelerate -p uor-r4-training --example geometric-stack` → Finished. `cargo fmt` applied.
- **Exercised, not merely compiled** — the argument is accepted and validated:
  - malformed file → `Error: Json(Error("EOF while parsing a value", line: 1, column: 0))` (the file is parsed and rejected precisely);
  - `oracle=` is a **recognised key**, not an unknown-argument error, and execution proceeds to artifact loading (`Invalid("artifact format: missing UORLUT01 magic")` with `/dev/null`).
- **Used for its purpose.** Measured against the sealed `chat-prose-init-phase-a-1` parent over the ten `dev-mem` rows, with a **matched control** that inserts a value-free turn of the same shape so the extra turn is charged separately:

| Arm | Context | Value present |
|---|---|---:|
| control — filler turn, no value | filler, fact, distractor, question | 2/10 |
| parent — as measured | fact, distractor, question | 4/10 |
| **oracle — correct value supplied** | oracle statement, fact, distractor, question | **5/10** |

**The ceiling is 5/10, and the inserted turn alone costs 2.** Result reported on #1512.

## Scope

Adds an **instrument**. It changes no existing behaviour and no default. The ten-row panel, one seed, greedy and 32 tokens make this a **diagnostic, not a capability result**; the `5 vs 4` swing is a five-row effect and should be read as a **bound**, not an effect size. The fixture `research/value-faith-oracle-20261002.json` and its note document where the statements come from: each is the fact the row states in its **own first turn**, restated as a standalone statement of the same relation.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

