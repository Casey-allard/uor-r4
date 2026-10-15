# #1895 — Current state: real-text gate, served result, data-vs-size split (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T11:33:58Z
- **Updated:** 2026-10-09T13:19:44Z
- **Closed:** 2026-10-08T11:39:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1895
- **Merged:** 2026-10-08T11:39:24Z
- **Merge commit:** 95ad4e8d8e0697ef4745ce45c2fd357e1e7f6cc0
- **Base / Head:** main / UOR-Foundation:deepseek/current-state-chat
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this integrates

`docs/integration/current-state.md` owns changing results and next actions, and it carried none of the overnight findings. This adds them as the newest section.

**Gate.** The Kneser-Ney 5-gram on `chat-v0-p2.train` → `heldout`: **1.280275 bits per byte** (399,988 positions, 1,088,056 bytes) — the first n-gram gate ever measured on this corpus, which the transition plan sets as its acceptance bar. Capacity sweep at fixed estimator: 2.754200 at 1,000 retained rows per order, 1.856607 at 20,000, 1.280275 at KN's own caps. **The plan's pre-registered stop rule fires** (stop if KN-5 ≥ ~1.25).

**Sealed evaluations converted.** All of them recorded `bits_per_byte: null`; `token-stream-bytes` converts exactly and shows both streams carry a **64-byte UORT header** — the ids previously read as out-of-vocabulary were header bytes. On chat: `geo-8m-l2` **1.119883**, `s2-metal-b` 1.122717, `a` 1.124197, transformer control 1.145440. **All three geometric variants beat the parameter-matched control on both corpora**, ordered `l2 < s2 < a < control`, by 0.8–2.2 %.

**Served artifact:** **below uniform** on chat (4.3885 vs 4.2292, top-1 2.25 %), working in domain (1.8202, top-1 25.81 %). The 3.27 BPB gap decomposes into 2.5683 domain mismatch + 0.5830 in-domain export loss — and **not** a kernel loss, since the sealed D11 record shows d10 ≡ d11 to sixteen digits.

**End-to-end served result:** 0.8770 fused · 0.9138 snapped · **0.9550 through the bit-exact multiplier-free engine** · against **4.3885 served today**.

**Data vs size, settled:** at matched 7,155,396 parameters, chat-only data is worth **0.1592 BPB** and size **0.0837 BPB**.

Next dependencies ordered in the section: common-position re-scoring under the document-clustered protocol (clustered SE 0.003489), then **data coverage before parameters**, then served fidelity; and any mechanism experiment on the discrete scorer belongs **in domain**, where it has 25.81 % top-1.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

