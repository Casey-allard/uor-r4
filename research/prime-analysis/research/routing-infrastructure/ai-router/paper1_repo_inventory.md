# Paper 1 Repo Inventory

Scope: assets relevant to the first angular-routing paper only.

## Canonical paper assets

### Manuscript source

- `router-research/papers/main.tex`
- `router-research/papers/main.pdf`
- `router-research/papers/references.bib`
- `router-research/papers/Makefile`
- `router-research/papers/SUBMISSION_CHECKLIST.md`
- `router-research/papers/ZENODO_METADATA.md`

### Figure assets

- `router-research/papers/figures/fig_scaling_law.pdf`
- `router-research/papers/figures/fig_scaling_law.png`
- `router-research/papers/generate_figures.py`

### Primary result sources

- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0172_moe_substitution.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`
- `router-research/results/analysis/inc0173_wt2_replication.json`
- `router-research/results/analysis/inc0173_wt2_seed1.json`

### Canonical narrative / scoping docs

- `router-research/docs/research/PUBLICATION_STRATEGY.md`
- `router-research/docs/research/ACTIVE_STATE.md`
- `router-research/docs/research/increments/INC_0143_TBD.md`
- `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`
- `router-research/docs/research/increments/INC_0165_hardware_proxy_closure.md`
- `router-research/docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md`
- `router-research/docs/research/increments/INC_0170_large_k_angular_capacity.md`
- `router-research/docs/research/increments/INC_0171_lm_integration.md`
- `router-research/docs/research/increments/INC_0172_moe_substitution_study.md`
- `router-research/docs/research/increments/INC_0173_wt2_replication.md`

## Likely stale or redundant assets

### Historical preprint archive

These look useful for historical context, not for Paper 1 source selection:
- `preprints/`
- `preprints/More Formal/`
- `preprints/angular_manifold_routing_preprint.pdf`
- `preprints/geometric_routing_llm_research_paper.pdf`
- `preprints/geometric_routing_llm_architecture_submission_paper.pdf`
- `preprints/geometric_router_llm_hypothesis_paper.pdf`
- `preprints/geometric_routing_divisor_spectral_hypothesis.pdf`

### Parallel working drafts

These appear to be active rewrite experiments or sandbox drafts, not the clean canonical bundle:
- `submit/main.tex`
- `submit/main2.tex`
- `submit/3/main.tex`
- `submit/rewrite_plan.md`

### Packaging duplicates

Useful for release, but not where the paper should be edited:
- `router-research/papers/release_bundle/main.tex`
- `router-research/papers/release_bundle/main.pdf`
- `router-research/papers/release_bundle/references.bib`
- `router-research/papers/release_bundle/figures/fig_scaling_law.pdf`
- `router-research/papers/release_bundle/generate_figures.py`
- `router-research/papers/angular_manifold_routing_bundle.zip`
- `router-research/papers/arxiv_source_upload.zip`

### Internal / broader-program artifacts that should not be treated as Paper 1 assets

- `router-research/CORE_PROJECT_GOALS.md`
- `router-research/GEOMETRIC_COMPUTATION_HYPOTHESIS.md`
- `router-research/THEORY_SKETCH.md`
- `router-research/SEMANTIC_PACKET_ROUTING_RESEARCH_NOTE.md`
- `router-research/spectral_emergence_moonshot.md`
- `router-research/geometric_routing_architecture_summary.md`
- `router-research/adaptive_field_computer_moonshot.md`

## Missing assets that should be created before a paper rewrite

### 1. Figure provenance cleanup

Needed:
- A figure-generation script that reads directly from canonical analysis JSON instead of hard-coded points.

Why:
- `router-research/papers/generate_figures.py` currently hard-codes the scaling points and fit lines.

Recommended output:
- `router-research/papers/generate_scaling_figure_from_results.py`
- Optional machine-readable source table: `router-research/papers/assets/scaling_law_points.csv`

### 2. Paper-ready table sources

Needed:
- Generated table sources for PTB, WT2, and cross-dataset summaries.

Why:
- The manuscript tables are currently hand-entered in LaTeX rather than produced from result JSON.

Recommended output:
- `router-research/papers/assets/ptb_results.csv`
- `router-research/papers/assets/wt2_results.csv`
- `router-research/papers/assets/cross_dataset_summary.csv`
- Optional helper script: `router-research/papers/export_tables.py`

### 3. Method schematic

Needed:
- One clean schematic showing the fixed 4D projection, Hopf coordinates, and sector assignment.

Why:
- The current paper has a scaling figure but no manuscript-grade method figure.

Recommended output:
- `router-research/papers/figures/fig_hopf_routing_schematic.pdf`

### 4. One consolidated evidence table

Needed:
- A single provenance table mapping every paper claim/table/figure to its supporting increment/result file.

Why:
- The repo has the pieces, but the provenance is spread across `ACTIVE_STATE.md`, increment docs, analysis JSON, and manuscript prose.

Recommended output:
- `router-research/papers/CLAIM_TO_ARTIFACT_MAP.md`

### 5. Cleaner appendix assets

Needed:
- A compact appendix-ready panel or table for controls:
  structured-vs-permuted proxy, Hopf-vs-kmeans, aux-loss baseline control, and norm invariance.

Why:
- Those controls are strong, but they are currently dispersed across increment docs.

Recommended output:
- `router-research/papers/figures/fig_controls_panel.pdf`
- Or `router-research/papers/assets/controls_summary_table.tex`

## Recommended Paper 1 working set

If someone takes over the rewrite, start here and ignore the rest at first:
- `router-research/papers/main.tex`
- `router-research/papers/SUBMISSION_CHECKLIST.md`
- `router-research/docs/research/PUBLICATION_STRATEGY.md`
- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0172_moe_substitution.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`

Everything else is secondary until that reduced set is fully paper-shaped.
