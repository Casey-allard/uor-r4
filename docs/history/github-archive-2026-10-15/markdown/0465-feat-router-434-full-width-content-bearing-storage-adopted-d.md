# #465 — feat(router): #434 — full-width content-bearing storage (adopted de-banding)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T15:14:48Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-06T15:45:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/465
- **Merged:** 2026-08-06T15:45:12Z
- **Merge commit:** 717c5442a00451a21b819598dea77afd4f37515a
- **Base / Head:** main / UOR-Foundation:i434-debanding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements the de-banding decision adopted on #434 after PR #442's payoff measurement. Follows the approved order: consumer audit, storage change, regression gates.

**What was banded:** `index_sentence_routed` (lib.rs:1749-1753) copied only `active_range` — width 32 of 512 (**1/16**) — into the stored vector. The content vector itself was **never** banded: `content_state_vector` already returned the full 512-d zeta word-sum, and storage discarded 15/16 of it.

**Change:** serialized `banded_storage: bool`, **default false (full width)**, with wasm accessors; the pre-#434 banded copy is retained behind the flag for A/B and cheap rollback. Content-free (#255 arm one) path is byte-identical in both modes. Determinism and canonical ordering preserved.

**Consumer audit** (docs/debanding_audit_434.md): no shipping consumer breaks. Two TEST consumers assumed sparsity and are now pinned to `set_banded_storage(true)` — `zeta_state_retrieval::content_full_and_band` (asserts zeros outside the band) and memory-lift ARM ONE, which was a **silent control invalidation**: its stored side stayed banded while its queries went full width, collapsing arm-one MRR 0.1674 → 0.0006. Pinned, it restores to exactly 0.1674. Size: in-memory unchanged (banding wrote zeros); JSON persistence ~16× on the vector payload (~60 MB → ~992 MB for a 46k-window store) — flagged in the audit as the real cost.

**Regression gates — both PASS:**

| gate | baseline | de-banded |
|---|---|---|
| memory-lift content-derived MRR / top-1 | 0.2348 / 0.208 | **0.8948 / 0.832** |
| memory-lift content-free (control) | 0.1674 | 0.1674 |
| memory-lift shuffled (control) | 0.0002 | 0.0001 |
| reconnection ARM B free top-1 | 7.64 | **8.17** |
| reconnection ARM B anchor accuracy | 9.3% | **11.4%** |
| reconnection ceiling / nulls | 31.5 / 2.14 / 3.09 | 31.5 / 2.14 / 3.07 |

ARM B (production path) now reproduces ARM BF exactly — the harness's BF-vs-B rule is **vacuous**, not regressed, because it compared BF against a banded B. `set_banded_storage(true)` reproduces 7.64 / 9.3.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

