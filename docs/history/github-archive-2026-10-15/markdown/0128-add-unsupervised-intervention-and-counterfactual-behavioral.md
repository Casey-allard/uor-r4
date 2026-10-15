# #128 — Add unsupervised intervention and counterfactual behavioral probes

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:49Z
- **Updated:** 2026-07-24T15:59:43Z
- **Closed:** 2026-07-24T15:59:43Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/128
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends the anti-degeneracy and evaluation work in #23 and #29.
Source: PDF §§7, 11, and 17.

Build the behavioral-probing layer needed to distinguish reusable predictive structure from surface association. The compiler should be able to compare a baseline context with controlled interventions and record the teacher's changed behavior.

## Scope

- Define intervention records: source observation, intervention descriptor, affected span/bytes, expected invariants, teacher outputs, and content IDs.
- Support at least context ablation, paraphrase/surface-preserving variation, entity/value substitution, temporal change, and action/goal change.
- Record counterfactual outcomes and uncertainty without asserting causal truth beyond the intervention protocol.
- Add split rules preventing near-duplicate baseline/counterfactual leakage into evaluation.
- Feed probe results into overlap induction, future-state objectives, and semantic-coherence certificates.

## Acceptance criteria

- Probe records are deterministic, content-addressed, serializable, and replayable.
- The harness reports sensitivity to intended changes and invariance to declared nuisance changes.
- Every probe declares its expected relation (invariant, sensitive, or unknown) and failures are visible in certificates.
- Tokenizer-neutral span/byte anchors are used where applicable; the deployed runtime remains unaware of probe metadata.
- A small fixture proves that a predictive table that memorizes surface form fails the probe suite.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

