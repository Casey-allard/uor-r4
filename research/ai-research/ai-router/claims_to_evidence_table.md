# Claims to Evidence Table

Scope: main claims and paper-facing supporting assets for Paper 1 only.

| Main claim | Required supporting figure(s) / table(s) | Required control(s) | Canonical source files / scripts | New asset required from canonical results? | Current paper asset status |
|---|---|---|---|---|---|
| 1. Fixed Hopf-derived routing yields a sublinear routing footprint on a semantic proxy | Main scaling figure; proxy scaling table | Dense routing baseline; permuted-routing control | `router-research/results/analysis/inc0168_norm_geometry.json`; `router-research/results/analysis/inc0170_large_k.json`; `router-research/papers/generate_figures.py`; `router-research/papers/figures/fig_scaling_law.pdf` | Yes. Figure should be regenerated from canonical JSON instead of hard-coded points. Scaling table should be exported from results. | Provisional |
| 2. The proxy advantage persists through large routing budgets up to `K=5000` | Main scaling figure or appendix extension panel; scaling summary table | Permuted-routing control at large `K`; dense baseline | `router-research/results/analysis/inc0170_large_k.json`; `router-research/docs/research/increments/INC_0170_large_k_angular_capacity.md` | Yes. Add generated large-`K` summary table or appendix panel. | Should be replaced |
| 3. The effect depends on structured angular signal, not arbitrary occupancy | Appendix control table | Structured vs column-permuted proxy control | `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`; `router-research/docs/research/increments/INC_0143_TBD.md` | Yes. Needs exported control table or appendix panel. | Missing |
| 4. Fixed 4D Hopf geometry outperforms adaptive 100D `K`-means on structured embeddings | Control table or appendix-only figure/table | Hopf vs adaptive `K`-means comparison | `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`; `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md` | Yes. Needs generated control table. | Missing |
| 5. The strongest proxy scaling result is angular-only rather than radial-shell driven | Appendix norm-invariance table or short supplement panel | Norm-invariance check; shell-activation comparison | `router-research/results/analysis/inc0168_norm_geometry.json`; `router-research/docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md` | Yes. Needs compact exported summary table. | Missing |
| 6. Fixed geometric routing partially transfers to a toy LM, replacing learned top-1 gating at modest perplexity cost | LM result table; method/condition summary table | Learned top-1 baseline | `router-research/results/analysis/inc0171_lm_integration.json`; `router-research/docs/research/increments/INC_0171_lm_integration.md` | Yes. PTB table should be exported from JSON. | Provisional |
| 7. The PTB transfer result replicates on WT2 at the same standard | Cross-dataset table or combined LM table | Same condition set on second dataset | `router-research/results/analysis/inc0173_wt2_confirm.json`; `router-research/results/analysis/inc0173_wt2_replication.json`; `router-research/docs/research/increments/INC_0173_wt2_replication.md` | Yes. WT2 and cross-dataset tables should be exported from JSON. | Provisional |
| 8. In trainable LM settings, Hopf routing is not distinguishable from permuted fixed routing on quality | Key null-result table | HOPF vs PERMUTED on PTB and WT2 | `router-research/results/analysis/inc0171_lm_integration.json`; `router-research/results/analysis/inc0173_wt2_confirm.json` | Yes. Needs dedicated compact null-result table. | Missing |
| 9. The no-aux learned top-1 baseline is the correct main comparator for Paper 1 | Appendix baseline-control table or short supplement paragraph/table | Load-balance auxiliary-loss control | `router-research/results/analysis/inc0172_moe_substitution.json`; `router-research/docs/research/increments/INC_0172_moe_substitution_study.md`; `router-research/docs/research/PUBLICATION_STRATEGY.md` | Yes. Needs appendix-ready table. | Missing |
| 10. The paper’s scope is limited: strongest evidence is proxy-side and the LM result is toy-scale only | Limitations paragraph, optionally no dedicated figure/table | Null result against PERMUTED; architecture scope statement | `router-research/docs/research/PUBLICATION_STRATEGY.md`; `router-research/docs/research/increments/INC_0171_lm_integration.md`; `router-research/docs/research/increments/INC_0173_wt2_replication.md` | No. This is a prose limitation backed by existing canonical docs. | Acceptable |

## Figure and table inventory

### Main-paper figures

#### Figure 1. Early anchor visual
- Type: method schematic
- Purpose: make the fixed routing mechanism understandable before the quantitative results
- Canonical source: algorithm and notation in `router-research/papers/main.tex`
- New asset required: yes
- Current asset status: missing

#### Figure 2. Main scaling/result figure
- Type: proxy scaling figure
- Purpose: primary evidence visual
- Canonical source: `router-research/results/analysis/inc0168_norm_geometry.json`, `router-research/results/analysis/inc0170_large_k.json`
- Current asset: `router-research/papers/figures/fig_scaling_law.pdf`
- New asset required: yes, regenerate from canonical JSON
- Current asset status: provisional

### Main-paper tables

#### Table 1. Method and condition summary
- Purpose: compact paper-facing setup summary for proxy and LM conditions
- Canonical source: `router-research/results/analysis/inc0171_lm_integration.json`, `router-research/results/analysis/inc0173_wt2_confirm.json`, manuscript setup in `router-research/papers/main.tex`
- New asset required: yes
- Current asset status: should be replaced

#### Table 2. LM results
- Purpose: PTB and WT2 transfer results
- Canonical source: `router-research/results/analysis/inc0171_lm_integration.json`, `router-research/results/analysis/inc0173_wt2_confirm.json`
- New asset required: yes, exported from canonical JSON
- Current asset status: provisional

#### Table 3. Key control / null-result table
- Purpose: emphasize HOPF vs PERMUTED null and justify comparator choice
- Canonical source: `router-research/results/analysis/inc0171_lm_integration.json`, `router-research/results/analysis/inc0173_wt2_confirm.json`, `router-research/results/analysis/inc0172_moe_substitution.json`
- New asset required: yes
- Current asset status: missing

### Appendix-only assets

#### Appendix Table A1. Structured-vs-permuted proxy control
- Canonical source: `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- New asset required: yes
- Current asset status: missing

#### Appendix Table A2. Hopf-vs-kmeans control
- Canonical source: `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- New asset required: yes
- Current asset status: missing

#### Appendix Table A3. Norm invariance summary
- Canonical source: `router-research/results/analysis/inc0168_norm_geometry.json`
- New asset required: yes
- Current asset status: missing

#### Appendix Table A4. Auxiliary-loss baseline control
- Canonical source: `router-research/results/analysis/inc0172_moe_substitution.json`
- New asset required: yes
- Current asset status: missing

#### Optional Appendix Table A5. Hardware proxy summary
- Canonical source: `router-research/docs/research/increments/INC_0165_hardware_proxy_closure.md`
- New asset required: maybe, depending on whether hardware proxy is retained as appendix-only support
- Current asset status: missing

## Acceptable vs provisional vs replace guidance

### Acceptable

- `router-research/papers/main.tex` as canonical manuscript source for structure extraction only
- `router-research/results/analysis/*.json` late closed increments as evidence sources
- `router-research/docs/research/PUBLICATION_STRATEGY.md` as scope authority

### Provisional

- `router-research/papers/figures/fig_scaling_law.pdf`
- Current PTB / WT2 / cross-dataset tables embedded in `router-research/papers/main.tex`

### Should be replaced

- Hard-coded plotting logic in `router-research/papers/generate_figures.py`
- Hand-entered result tables in the manuscript
- The current evidence-chain appendix as a paper appendix

### Should not drive the paper

- `submit/`
- `preprints/`
- `router-research/papers/release_bundle/` as a writing source
- Internal project-history and moonshot docs
