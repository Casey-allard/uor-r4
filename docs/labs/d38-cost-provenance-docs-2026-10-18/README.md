# D38 evidence and injective-draft cost provenance — 18 October 2026

Documentation only, based on main `948744c995a52bdc082bd611a7891d7db0012e7d`.
This correction does not change a result, registered parameter, budget, permission
or running status. No model or resource operation was performed.

## Corrections and primary evidence

- [ROADMAP R3/R5](../../../ROADMAP.md#0-mission-and-hard-runtime-rules)
  now spell out the separate evidence required by
  [D38](../../integration/DECISIONS.md#d38--d5-split-d5-a-residency-and-d5-b-sparsity-owner-2026-10-18):
  D5-a post-warmup hardware-counter cache/DRAM bytes per token and, when the
  model exceeds cache, D5-b measured selected parameter access per token.
  J/token, RSS, tokens/s and D25 remain required; residency does not prove sparsity.
- Only [section 8 of the unregistered injective draft](../../harness/prep/m1-attention-injective-init-DRAFT-2026-10-18.md#8-cost)
  changes. The historical second paired `rows_v1` Q1 attempt (Dose2), source
  `20d158ae` / ELF `e7cb6197`, remains distinct from the original owner-stopped
  904/1,065-batch attempt and the new injective lineage. The
  [historical diagnostic record](../attention-rows-q1-diagnosis-2026-10-18/README.md)
  and [retained timing clarification6253995798](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6253995798)
  establish the provenance: 5,361 seconds was a resource lifetime, not a
  per-batch timing series or a measured upper bound for the changed lineage.
  The 1.31 s/batch, six-hour/$11.4 full-dose and $2.8 Q1 figures are retained
  as assumption-based planning extrapolations, with their scaling and rate
  assumptions stated. Changed-lineage throughput and all-in cost remain
  unmeasured. The draft stays unregistered and grants no compute release.

## Validation and scope

Validation passed: 42 scoped local links/anchors, current Python and existing
Rust claim-wording gates, eight harness state formats, and `git diff --check`.
The Rust claim-wording crate is byte-identical to the source of the retained
tool used here. Direct base comparisons confirm all draft bytes outside
section 8, every ROADMAP line outside R3/R5, peer claims and prior current-state
entries remain unchanged. No model suite was run for this prose-only correction.
Only the two documentation candidates and required delivery bookkeeping change;
the peer E0063/D39 and running-status work is preserved. No model, test, workflow,
diagram, archive or GitHub issue-state change is included.

**Next:** independent review of the exact PR head before protected delivery.
This is a documentation correction, not a model qualification or a measured
efficiency claim.
