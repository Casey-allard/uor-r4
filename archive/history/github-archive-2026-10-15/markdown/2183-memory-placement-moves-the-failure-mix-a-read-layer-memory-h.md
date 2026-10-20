# #2183 — Memory placement moves the failure mix: a read-layer memory holds the headline with half the wrong-value failures (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T23:06:26Z
- **Updated:** 2026-10-10T23:12:50Z
- **Closed:** 2026-10-10T23:09:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2183
- **Merged:** 2026-10-10T23:09:52Z
- **Merge commit:** 6e3c4553cd494b27d12743b8cb9fcf968656f8d1
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-memory-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Cycle 10 varies what the measured memory configuration never varied — *where* it sits and *how big* it is — and the result is specific: a memory on a **read** layer (layer 8 of `rrarrarrar`) holds the anchor's own headline (**20/40**) while **halving the wrong-value failures (17 → 10)** and gaining **abstention (0 → 4/24)**, at the cost of a new distractor-key pattern; the memory's **size** at a recurrence layer (64 → 256 sub-keys) buys 19 → 22/40, which the **no-memory control** also reads, for **16× the parameters**. Neither probe meets the pre-registered promotion bar (≥ 24/40 with BPB within 0.01), so nothing is promoted and no mechanism is closed (D22 §1).** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory (D22 order 2) · count 0/3 · headline before reply 30/232 + memory 19/40 → after **20/40 (read layer)** and **22/40 (default size)** with wrong-value **17 → 10**

## The probes

| arm | placement | size | steps | params | v5 memory | unknowable | wrong-value | derangement | wall |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| G1 (measured configuration) | layer 4 (an `r` layer) | 64 sub-keys, 16 top-k | 2,000 | 32.6 M | 19/40 | 0/24 | 17 | 1 | 387 s |
| **P256** | layer 4 | **256 sub-keys, 32 top-k, key width 128** | 2,000 | 65.9 M | **22/40** | 1/24 | 14 | 3 | 2,007 s |
| **P8** | **layer 8** — the last **read** layer (`a`) | 256 sub-keys, 32 top-k, key width 128 | 2,000 | 65.9 M | **20/40** | **4/24** | **10** | 1 | 2,000 s |
| C1 (control, **no memory**) | — | — | 1,000 | 30.2 M | 22/40 | 3/24 | 12 | 0 | 63 s |
| anchor D10 (**no memory**) | — | — | 2,000 | 30.2 M | **20/40** | 0/24 | 17 | 0 | — |

Both probes ran with the trainer's **default** memory sizes: the intended 64-sub-key P8 arm took the defaults too, so the pair varies the **placement** and both vary the **size** against the earlier arm. That is stated rather than glossed — the two variations are not orthogonal in this pair.

**No promotion.** The pre-registered bar was **≥ 24/40 and BPB within 0.01 of 1.17425**: P256's 22/40 is what the **no-memory control at the shorter fit** also reads, and P8's 20/40 is the anchor's own. The BPB reports are still running and are named pending, not guessed.

**What it does establish:** this line's failures are value-selection failures, and a memory on a **read** layer is the first thing that halves them without losing the headline. **Next:** iterate the read-layer placement with the two seeds the brief asks for, then D22's ordered pieces — the **identity pointer keyed on a slot/entity address** (order 3, wrong-value primary) and **powered read binding** (order 4).

## Also measured: the bridge's cost scales with the memory

2,000 steps took **387 s** with the 64-sub-key memory and **2,000 s / 2,007 s** with the 256-sub-key one — **5.2× slower** — so the host bridge, not the GPU trunk, is now the cost. A bigger memory is not free even when it is not better, and the kernel remains the honest follow-up.

## Checklist

- [x] Checks at the exact head `23932fc46a4e05fdb0758dcc3fcf2dc78ecdec6e`: `python3 scripts/check_claim_wording.py` **pass** (docs-only: the record, a `current-state` entry, the README's two mechanism sections).
- [x] Research record: `docs/labs/addressed-memory-2026-10-10/README.md` (the probe section appended to this mechanism's record)
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] README **How it works** and **Results so far** updated where the result changed (D22 §6)
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, post the probes' BPB readings

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

