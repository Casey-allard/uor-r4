# #1770 — addressed_attention: dispatch harness — the engine fires but never selects the computing action

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T00:28:51Z
- **Updated:** 2026-10-06T00:29:55Z
- **Closed:** 2026-10-06T00:29:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1770
- **Merged:** 2026-10-06T00:29:55Z
- **Merge commit:** c950c03ce2c8a476e39f3755a9547408f634018d
- **Base / Head:** main / UOR-Foundation:deepseek/aa-dispatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Drives the eight-phase addressed-attention runtime end to end on CPU (~2 s, 0.65 ms/step), using the canonical construction copied from the in-crate tests. Production code unchanged; the module's own tests still pass 50/0/7.

**Why it exists:** `addressed_attention` is called from 83 files, all inside `uor-r4-core/src` — zero in `uor-r4-training`. This makes the module drivable without the trainer and answers whether that exclusion is by omission or by mathematics.

**Counters, 1000 predict+observe steps:** `circuit_calls` 7902, `candidate_scores` 446690, `predictions` 1000, `observations` 1000, `scalar_decodes` 1954 — all advance. **`selected_operations` = 0.**

**The cause is demonstrated, not inferred:** the control head chose only Hold 269 / AcquireB 258 / Clear 245 / AcquireA 228 — never `AddAB`/`SubAB`. The arithmetic that would produce an answer is never executed.

**Scored surface** (deterministic CompiledPolicy, frozen dev split, 154 positions): Full 1/154 correct, 0/8 exact; ReadDisabled 0/154; ExactPayloadMasked 0/154; StateTransportDisabled 0/154 — all three destructive controls at least as good as Full. 248 distinct symbols of 257, so no collapse.

Identities reproduce the sealed evidence bit-identically (geometry `49a50365…`, 665216 params, digest `d29bb3f0…`).

**Open, bounded question this enables:** can any objective make the control head select `AddAB`/`SubAB`? The harness runs in two seconds, so the sweep is cheap.

Raw output: `reports/aa-dispatch-count-3/` (`SUMMARY.md`, `dispatch.json`, logs), with failed roots preserved and sealed.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

