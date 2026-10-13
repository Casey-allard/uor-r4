# Paper 1 Manuscript Blueprint

Scope: professional ML-paper architecture for Paper 1 only.

Canonical source-of-truth assumptions:
- Manuscript source-of-truth: `router-research/papers/main.tex`
- Evidence triage source-of-truth: `paper1_evidence_map.md`
- Repo asset triage source-of-truth: `paper1_repo_inventory.md`
- `submit/` drafts are exploratory and non-canonical unless explicitly used for comparison

## Paper identity

Paper 1 is a narrow empirical paper about fixed geometric routing from embedding angular structure. Its claim is not that geometry replaces modern MoE routing broadly, but that a fixed Hopf-derived routing map yields a sublinear routing footprint on a semantic proxy and partially transfers to a toy trainable LM, while failing to show a geometry-specific quality advantage over randomly permuted fixed routing in end-to-end training.

## Exact scoped contributions

- A fixed Hopf-derived routing map yields a sublinear routing footprint on a semantic proxy, with a canonical scaling law and persistence to larger routing budgets.
- The same fixed-routing mechanism partially transfers to a 2-layer trainable LM, replacing a learned top-1 gate at modest perplexity cost and without a learned gate matrix.
- The trainable LM result is sharply limited by a null finding: Hopf routing is not distinguishable from randomly permuted fixed routing on quality.

## Section order

1. Abstract
2. Introduction
3. Fixed geometric routing method
4. Semantic proxy experiments
5. Trainable language-model experiments
6. Discussion and limitations
7. Reproducibility statement
8. Conclusion
9. Appendix / supplement

## Section blueprint

### 1. Abstract

Purpose:
- State the problem, method, proxy result, LM transfer result, null result, and limitation in one compact block.

Must include:
- Fixed Hopf-derived routing map
- Proxy scaling result
- PTB and WT2 transfer result
- HOPF approximately equals PERMUTED null result
- Toy-scale limitation

Canonical evidence:
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`
- `router-research/docs/research/PUBLICATION_STRATEGY.md`

### 2. Introduction

Purpose:
- Frame the reader problem in standard ML terms: sparse routing usually needs learned gating; can some routing structure come directly from geometry?

Must include:
- Why fixed routing is interesting
- Why angular structure motivates the method
- Narrow scope statement up front
- Short contribution list, capped at 3 bullets

Must not include:
- Internal increment chain
- “Independent parallel development”
- Broad H^4 x H^4 architecture framing
- “Deeper computational substrate” framing

Canonical evidence:
- `router-research/docs/research/PUBLICATION_STRATEGY.md`
- `router-research/papers/main.tex`

### 3. Fixed geometric routing method

Purpose:
- Make the routing mechanism legible without importing project-history language.

Must include:
- 4D projection
- Hopf coordinate definitions
- Sector assignment logic
- Routing budget `K`
- Precise note on optional transport term
- Effective routing footprint metric definition

Must not include:
- “winning algorithm”
- “survived all falsification attempts”
- Increment references in the main flow

Canonical evidence:
- `router-research/papers/main.tex`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`

Production dependency:
- Needs a clean method schematic generated as a paper asset

### 4. Semantic proxy experiments

Purpose:
- Establish the main Paper 1 result cleanly and early.

Subsections:
- Setup
- Main scaling result
- Structural controls
- Optional short mechanism note

Must include:
- PPMI-SVD proxy setup
- Canonical scaling law `2.957 * K^0.572`
- Large-`K` persistence through `K=5000`
- Dense and permuted controls
- Short statement that the effect depends on angular structure

May include if space allows:
- Hopf vs adaptive `K`-means control

Should not headline in the main body:
- Hardware proxy as a central claim
- Full norm-invariance discussion
- Detailed phase-transport mechanism isolation

Canonical evidence:
- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`
- `router-research/docs/research/increments/INC_0143_TBD.md`
- `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`
- `router-research/docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md`
- `router-research/docs/research/increments/INC_0170_large_k_angular_capacity.md`

Production dependencies:
- JSON-backed scaling figure
- Exported proxy summary table
- Optional controls table or appendix panel

### 5. Trainable language-model experiments

Purpose:
- Show the transfer result, then immediately narrow it with the null result.

Subsections:
- Setup and conditions
- PTB result
- WT2 replication
- Null result and interpretation

Must include:
- 2-layer transformer setup
- BASELINE / HOPF / PERMUTED comparison
- PTB HOPF/BASELINE ratio
- WT2 replication at same standard
- HOPF approximately equals PERMUTED null
- No learned gate matrix

Must mention carefully:
- Baseline is top-1 without auxiliary load-balancing
- The aux-loss control exists to justify that comparator, but belongs in appendix/supplement

Canonical evidence:
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0172_moe_substitution.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`
- `router-research/docs/research/increments/INC_0171_lm_integration.md`
- `router-research/docs/research/increments/INC_0172_moe_substitution_study.md`
- `router-research/docs/research/increments/INC_0173_wt2_replication.md`

Production dependencies:
- Exported PTB/WT2/cross-dataset tables from canonical JSON
- A compact condition summary table

### 6. Discussion and limitations

Purpose:
- Explicitly state what the paper proves and does not prove.

Must include:
- Proxy result is stronger than LM result
- LM setting is toy-scale only
- Hopf does not beat permuted fixed routing in trainable LM
- FFN routing only, not attention routing
- Hardware proxy is simulated if mentioned

Must not include:
- Grand-program manifesto
- “Deeper substrate” speculative framing
- Paper 2/3 roadmap in the main discussion

Canonical evidence:
- `router-research/docs/research/PUBLICATION_STRATEGY.md`
- `router-research/docs/research/increments/INC_0165_hardware_proxy_closure.md`
- `router-research/docs/research/increments/INC_0171_lm_integration.md`
- `router-research/docs/research/increments/INC_0173_wt2_replication.md`

### 7. Reproducibility statement

Purpose:
- Say how the reader can reproduce the core public artifact without turning the paper into a repo memo.

Must include:
- Reproduction bundle reference
- What is directly reproducible now

Should not include:
- Large code listing in main text if space is tight

Canonical evidence:
- `router-research/papers/release_bundle/README.md`
- `router-research/papers/ZENODO_METADATA.md`
- `router-research/papers/release_bundle/hopf_routing_demo.py`

Recommendation:
- Move the long code listing out of the main body unless a venue strongly supports it

### 8. Conclusion

Purpose:
- End on the narrow result, not the broad program.

Must include:
- Fixed routing from angular structure works on the proxy
- Partial transfer to toy LM
- Geometry-specific advantage not established in trainable LM

Must not include:
- Product-manifold architecture
- 10x-100x hardware target
- Broader substrate claim

### 9. Appendix / supplement

Recommended appendix contents:
- Structured-vs-permuted semantic control
- Hopf vs adaptive `K`-means control
- Norm invariance / angular-only result
- Auxiliary-loss baseline control
- Optional hardware-proxy table if retained at all

Canonical evidence:
- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0172_moe_substitution.json`
- `router-research/docs/research/increments/INC_0165_hardware_proxy_closure.md`

## Figure and table architecture

### Main-paper assets

1. Early anchor visual
- Type: method schematic
- Role: make the routing mechanism legible before the scaling result
- Source status: must be newly generated from the algorithm, not currently present as a canonical paper asset

2. Main scaling/result figure
- Type: proxy scaling figure
- Role: headline visual for Paper 1
- Source status: current figure is acceptable only as provisional
- Current asset: `router-research/papers/figures/fig_scaling_law.pdf`
- Requirement: replace or regenerate from canonical JSON-backed source

3. Method/condition summary table
- Type: compact setup table
- Role: summarize proxy and LM conditions clearly
- Source status: new generated table recommended

4. LM result table
- Type: PTB + WT2 result table or paired PTB / WT2 tables
- Role: support the transfer claim
- Source status: current TeX tables are provisional because they are hand-entered

5. Key control/null-result table
- Type: compact table emphasizing HOPF vs PERMUTED and the aux-loss control rationale
- Role: narrow the claim honestly
- Source status: new generated table recommended

### Appendix-only assets

- Structured-vs-permuted proxy control table
- Hopf-vs-kmeans control table
- Norm invariance summary table or panel
- Optional hardware-proxy summary table

## What gets cut entirely

- TurboQuant comparison table as a central framing device
- “Independent parallel development”
- Explicit increment ranges or RR / INC numbering in prose
- “Winning algorithm” and “survived all falsification attempts”
- Long evidence-chain appendix written as project history
- “Broader perspective” section about deeper computational substrate
- “Next steps” section that markets later papers
- Any claim built around `H^4 x H^4`, spectral operators, sparse events, or 10x-100x savings
- Large in-body code listing unless required by venue

## What moves to appendix / supplement

- Structured-vs-permuted proxy evidence
- Hopf-vs-kmeans comparison
- Norm invariance
- Auxiliary-loss comparator control
- Hardware proxy summary if kept
- Minimal provenance table if needed for artifact review

## What stays repo-only

- Phase transport as an independent paper contribution
- Spectral/operator line
- Sparse-event routing line
- Product-manifold / H^4 x H^4 architecture framing
- Moonshot and theory-sketch documents
- Historical preprints and `submit/` rewrites

## What makes the current canonical draft feel like a memo instead of a paper

- It is organized around the internal research program rather than the reader’s question.
- It repeatedly foregrounds project history, increment chronology, and internal validation logic.
- It uses internal language like “winning algorithm,” “survived all falsification attempts,” and “correct baseline.”
- It carries too many adjacent claims at once: proxy scaling, hardware proxy, mechanism isolation, TurboQuant framing, reproduction code, and broad future-program framing.
- It retains an evidence-chain appendix that reads like a lab notebook summary instead of a paper appendix.
- It includes hand-curated tables and figure inputs without clearly exposing production provenance.

## Internal project-history material that must be removed from the manuscript body

- References to specific increment IDs as narrative structure
- “Independent parallel development” framing
- “publication extension,” “killed on design grounds,” and similar internal governance wording
- Evidence-chain chronology in the conclusion or appendix
- Repo-state or release-process language as part of the scientific story

## Explanatory gaps that must be filled for a real rewrite

- A clean explanation of why fixed routing from embedding geometry matters relative to standard sparse routing
- A legible schematic of the 4D projection and sectorization
- A precise definition of the effective routing footprint metric in paper language
- A cleaner explanation of why the semantic proxy is a valid first-stage test
- A cleaner explanation of why HOPF approximately equals PERMUTED is a narrowing result rather than a contradiction
- A compact explanation of why the no-aux learned top-1 baseline is the right main comparator

## Paper-production gaps before a full rewrite

- JSON-backed scaling figure generation
- Exported PTB / WT2 / cross-dataset tables from canonical results
- Method schematic figure
- Consolidated claim-to-artifact mapping
- Appendix-ready controls panel or table

## rewrite_prerequisites

### Assets that must be generated before the full rewrite

- JSON-backed scaling figure source and regenerated figure
- Exported PTB / WT2 / cross-dataset tables
- Method schematic
- Control/null-result summary table
- Optional appendix controls panel

### Assets that can be reused as-is

- `router-research/papers/main.tex` as structural source material only
- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0172_moe_substitution.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`
- `router-research/docs/research/PUBLICATION_STRATEGY.md`

### Assets that should be discarded from the rewrite workflow

- `submit/` drafts as manuscript source
- `preprints/` archive as claim selection input
- `router-research/papers/release_bundle/` as an editing source
- Internal moonshot / broad-program docs as paper framing
- Evidence-chain appendix in its current form
