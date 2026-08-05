# Deferral record — 2026-08-05 (issue #425 hygiene pass)

Scope: post-negative hygiene after the 2026-08 measurement campaign. Each
entry either defers a shipped-but-unmeasured mechanism with an explicit
activation condition, or records the disposition of a mechanism whose
measurement already concluded. Issue records remain authoritative; this
document only fixes where the pointers aim.

## R4 Spin(4) attention engine — deferred, with activation condition

What ships: `uor-r4-model-source` carries an experimental softmax-free
attention variant ("R4 4D Spin(4) quaternionic alignment": per-head scores
computed as chunked 4D dot products), gated by `cfg.r4_attention` /
`set_r4_attention`. It is selectable end-to-end: `uor-r4-api` accepts
`engine: "r4-attention"` in `InferenceRequest`, and the CLI compile path
accepts `--r4-attention`. Default is off everywhere.

Status: shipped and selectable, never measured. No campaign issue graded it
against the standard Llama attention arm on any pinned fixture, so there is
no evidence for removal (it is not measured-dead) and none for promotion.

Disposition: deferred, not removed. The #410 measured-dead precedent does
not apply — that discipline removes code an experiment showed to be dead;
this engine has simply never been put on the bench.

Activation condition: a pinned teacher-forced A/B on the standard fixture
set (same artifact, tokenizer, and prompt kappas as the parity harness),
`r4-attention` versus the standard attention arm, recording top-1, top-8,
and teacher bits/token. If the A/B records zero-or-negative, the engine
becomes a #410-style removal candidate; until the A/B exists, the flag stays
shipped and off.

## bott_fock context store — tracking re-pointed from #234 to #424

`docs/r4_furey_quantum_geometric_plan.md` records `bott_fock.rs` (the O(1)
Bott-periodic context fold) as implemented, unit-tested, unused by the
runtime, and "tracked there" — pointing at issue #234. Issue #234 has since
closed (its closure names the graph-path slice's 1.23% top-1 as "the number
to improve"), which left the bott_fock activation question tracked by a
closed issue.

Disposition: the activation-and-measurement question for bott_fock now lives
on issue #424. Nothing about the mechanism's status changes: it remains
implemented, unit-tested, and unused by the shipped runtime (the shipped
context is the hard 8-token window), awaiting a measured comparison under
the long-range-context motivation that #290's negative explicitly left
alive.

## FMM far-field family — disposition note (resolves the C1 contradiction)

The contradiction: `docs/fmm_290_novel_context_protocol.md` stated that "the
novel-context accuracy and cost decision for #290 remains open", while the
recorded result (`research/290-fmm/RESULT-52.md`) had already closed it
negative. Both could not be right; the issue record is authoritative.

The recorded negative, in brief: measured against the truncated SVD — by
Eckart–Young the optimal rank-k approximation, so an upper bound on any
practical scheme — the far-field operator carries ~6% relative matvec error
at rank 20 and still 2.5–3.7% at rank 64; and the interaction kernel is an
inner-product Gram block of 288-dimensional bundles, hence exactly rank
less than or equal to 288 by construction with an exact O((n_A + n_B)·D)
factorization, so there is no O(n²) interaction for an FMM to remove. The
far-field/H-matrix/HODLR family over this kernel is killed; the §3a
long-range-context motivation is explicitly not killed.

Disposition taken under #425 (this pass): a #410-style measured-dead
removal of the uncalled deployment surfaces, verified caller-by-caller:

- The FMM section emission is removed from the score pipeline
  (`compile_fmm_section` and the `fmm_section` input in
  `uor-r4-graph-certify::score`, plus its two CLI call sites). Scored
  artifacts no longer carry an FMM section, so artifact bytes and the graph
  kappa change; Gate C never read the section, so Gate C numbers do not.
- The runtime packed-kernel evaluation path is removed
  (`evaluate_fmm_translation_table` in `uor-r4-graph-runtime`, whose only
  caller was its own fixture test, plus the now-unreachable
  `FmmBufferTooSmall` error variant). No serving surface — engine, CLI, or
  `uor-r4-api` — ever called it.
- The score-report schema advances to twenty-two, dropping the
  `fmm_bytes`/`fmm_rank`/`fmm_candidate_count` footprint fields.
- Retained: the format-crate parser (`uor-r4-graph-format::fmm`,
  `SectionId::FMM`, and the stage-2 validation arm), so previously emitted
  artifacts still parse and validate; and the certifier-side
  `FmmCandidateScorer`/`FmmFixedCandidateScorer`, which the BDD S7
  exploratory parity scenario builds on the fly from graph bytes — a
  measurement harness, not a deployment path.

A dated addendum in `docs/fmm_290_novel_context_protocol.md` now points at
RESULT-52 and this record, replacing the stale "remains open" claim.
