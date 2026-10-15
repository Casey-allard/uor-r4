# #258 — Graded store bit-prefix keys are a candidate PRISM realization — document the ADDR/PRISM correspondence

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T11:50:55Z
- **Updated:** 2026-07-29T16:22:05Z
- **Closed:** 2026-07-29T16:22:05Z
- **Labels:** documentation, enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/258
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The transformerless graded store's bit-prefix keys are **structural addresses with meaningful order and distance** — shorter prefix = coarser region containing the finer, Hamming neighborhood = semantic neighborhood. That is coordinate-space work: the PRISM slot of the UOR architecture (the relational/value layer), grown independently inside r4 without knowing it is a PRISM-shaped object. Meanwhile κ-labels are deliberately razor-edged identity — same-or-not, nothing else (by design; see uor-addr ARCHITECTURE.md).

The honest architectural framing is therefore not "UOR is provenance plumbing around inference" but: **r4 uses UOR's identity layer (ADDR) and has grown its own relational layer that is a candidate PRISM realization.** The discrete-abstains / continuous-binds seam identified in the lineage audit is the identity-vs-structure two-layer question, in vivo.

### Scope (either/or, decide in the issue)
1. **Document the correspondence**: a short doc mapping graded-store key semantics onto PRISM's layer contract — what matches, what is missing, what would have to be true for the store's key scheme to be a conformant realization.
2. **Evaluate as a PRISM realization** (more ambitious): assess the graded store's key scheme against the PRISM model contract (`uor_standards/prism`, `PrismModel` trait) and prototype the mapping.

Interacts with #243 (the quantization scheme IS this address space — its review discipline should cite this correspondence) and #247 (session-signature lane extends the same coordinate space to conversation state).

### DoD
A committed doc (docs/) stating the correspondence with code references, plus a dated decision: document-only, or proceed to a realization evaluation as a follow-up.

*From Maura's ADDR/PRISM seam analysis.*


## Comments (1)

### comment 5120580717 — 2026-07-29T16:22:04Z — Casey-allard

Closing as delivered: the correspondence document merged in #262 (docs/addr_prism_correspondence.md) with the decision box recommending (a) — defer PRISM realization until #243's address-space outcome. The deferred decision is now tracked by #243 Phase A itself (signed off 2026-07-29); nothing else remains in this issue's scope.


---
_Comment/review ids in this file: 5120580717_

