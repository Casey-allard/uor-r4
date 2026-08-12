# Paper 1 Evidence Map

Scope: `Paper 1` only.

Working paper identity:
- Public paper title in curated bundle: `Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization`
- Narrow claim family: angular manifold routing / fixed geometric routing from embedding angular structure
- Explicitly out of scope for Paper 1: `H^4 x H^4` full architecture, spectral operators, sparse-event routing as a standalone contribution, 10x-100x hardware claims, broad “geometry-native compute substrate” framing

## Current paper locations

Most canonical paper bundle:
- `router-research/papers/main.tex`
- `router-research/papers/main.pdf`
- `router-research/papers/references.bib`
- `router-research/papers/figures/fig_scaling_law.pdf`
- `router-research/papers/SUBMISSION_CHECKLIST.md`
- `router-research/docs/research/PUBLICATION_STRATEGY.md`

Likely active but non-canonical rewrite drafts:
- `submit/3/main.tex`
- `submit/3/main.pdf`
- `submit/main2.tex`
- `submit/main2.pdf`
- `submit/main.tex`
- `submit/main.pdf`
- `submit/rewrite_plan.md`

Release/package duplicates, not source-of-truth authoring files:
- `router-research/papers/release_bundle/main.tex`
- `router-research/papers/release_bundle/main.pdf`
- `router-research/papers/angular_manifold_routing_bundle.zip`
- `router-research/papers/arxiv_source_upload.zip`

## Proposed main claim of Paper 1

L2-normalized token embeddings contain exploitable angular structure, and a fixed Hopf-derived routing map can use that structure to produce a sublinear routing footprint on a semantic proxy and partially transfer to a small trainable language model without a learned gate matrix; however, the geometry-specific advantage does not survive end-to-end LM training.

## Recommended canonical evidence chain

Use `router-research/papers/` as the manuscript source of truth, but tie every paper claim back to the late closed increments below.

### Primary results

| Result | Why it belongs | Best supporting artifacts | Recommended placement |
|---|---|---|---|
| 1. Canonical proxy scaling law: `eff_buckets = 2.957 * K^0.572` vs dense `K^1.0` | Strongest headline result; this is the core fixed-routing efficiency claim | `router-research/papers/main.tex`; `router-research/papers/figures/fig_scaling_law.pdf`; `router-research/papers/generate_figures.py`; `router-research/results/analysis/inc0168_norm_geometry.json`; `router-research/docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md`; `router-research/docs/research/ACTIVE_STATE.md` | Main text |
| 2. Large-K persistence through `K=5000` with `2.6x-2.8x` compression vs permuted/dense controls | Shows the effect does not collapse at larger routing budgets; important for real paper scope | `router-research/results/analysis/inc0170_large_k.json`; `router-research/docs/research/increments/INC_0170_large_k_angular_capacity.md`; `router-research/docs/research/ACTIVE_STATE.md` | Main text or short main-text result paragraph; details in appendix |
| 3. Fixed 4D Hopf geometry beats adaptive 100D `K`-means on structured embeddings | Strong argument that the routing signal is geometric, not just generic clustering | `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`; `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`; `router-research/docs/DECISIONS.md` | Main text if space permits; otherwise appendix |
| 4. PTB trainable LM result: fixed routing replaces learned top-1 gating at `1.081` PPL ratio, with no learned gate matrix | Main trainable transfer claim; must be in the paper | `router-research/results/analysis/inc0171_lm_integration.json`; `router-research/docs/research/increments/INC_0171_lm_integration.md`; `router-research/papers/main.tex` | Main text |
| 5. WT2 replication: same `1.081` HOPF/BASELINE ratio, second dataset, same setup | Strongest replication asset; de-risks “PTB-only” criticism | `router-research/results/analysis/inc0173_wt2_confirm.json`; `router-research/results/analysis/inc0173_wt2_replication.json`; `router-research/results/analysis/inc0173_wt2_seed1.json`; `router-research/docs/research/increments/INC_0173_wt2_replication.md` | Main text |

### Strongest controls and falsifications

| Control / falsification | What it checks | Best supporting artifacts | Recommended placement |
|---|---|---|---|
| 1. Structured embedding vs column-permuted control on semantic proxy | Establishes that the effect depends on semantic angular structure rather than arbitrary occupancy | `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`; `router-research/docs/research/increments/INC_0143_TBD.md`; `router-research/docs/research/ACTIVE_STATE.md` | Main text or appendix |
| 2. Hopf vs adaptive `K`-means | Falsifies “any adaptive clustering works equally well” | `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`; `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md` | Main text or appendix |
| 3. HOPF vs PERMUTED in PTB LM | Falsifies geometry-specific quality gain in trainable LM | `router-research/results/analysis/inc0171_lm_integration.json`; `router-research/docs/research/increments/INC_0171_lm_integration.md` | Main text |
| 4. HOPF vs PERMUTED in WT2 LM | Replicates the same null on a second dataset | `router-research/results/analysis/inc0173_wt2_confirm.json`; `router-research/docs/research/increments/INC_0173_wt2_replication.md` | Main text |
| 5. Learned sparse + auxiliary load-balance control collapses toward uniform routing | Supports why `BASELINE` top-1 without aux loss is the correct learned comparator for Paper 1 | `router-research/results/analysis/inc0172_moe_substitution.json`; `router-research/docs/research/increments/INC_0172_moe_substitution_study.md`; `router-research/docs/research/PUBLICATION_STRATEGY.md` | Appendix or supplement |

## One key null result

Null result:
- In trainable LM settings, the specific Hopf geometry is not distinguishable from randomly permuted fixed routing.

Best supporting artifacts:
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`
- `router-research/docs/research/increments/INC_0171_lm_integration.md`
- `router-research/docs/research/increments/INC_0173_wt2_replication.md`

Recommended placement:
- Main text

Reason:
- This is the paper’s most important honesty result and it sharply narrows the claim.

## One key limitation

Limitation:
- The strongest efficiency evidence is on a static PPMI-SVD semantic proxy, while the trainable evidence is limited to a 2-layer toy LM with FFN routing only.

Best supporting artifacts:
- `router-research/docs/research/PUBLICATION_STRATEGY.md`
- `router-research/papers/main.tex`
- `router-research/docs/research/increments/INC_0171_lm_integration.md`
- `router-research/docs/research/increments/INC_0173_wt2_replication.md`

Recommended placement:
- Main text limitations section

## Main-text / appendix / repo-only triage

### Main text

- Proxy scaling law and figure
  Supporting files:
  `router-research/papers/figures/fig_scaling_law.pdf`
  `router-research/papers/generate_figures.py`
  `router-research/results/analysis/inc0168_norm_geometry.json`
  `router-research/results/analysis/inc0170_large_k.json`

- PTB trainable LM replacement result
  Supporting files:
  `router-research/results/analysis/inc0171_lm_integration.json`
  `router-research/docs/research/increments/INC_0171_lm_integration.md`

- WT2 replication
  Supporting files:
  `router-research/results/analysis/inc0173_wt2_confirm.json`
  `router-research/docs/research/increments/INC_0173_wt2_replication.md`

- Geometry-null control in trainable LM: HOPF approximately equals PERMUTED
  Supporting files:
  `router-research/results/analysis/inc0171_lm_integration.json`
  `router-research/results/analysis/inc0173_wt2_confirm.json`

### Appendix / supplement

- Structured-vs-permuted semantic discrimination foundation
  Supporting files:
  `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
  `router-research/docs/research/increments/INC_0143_TBD.md`

- Hopf vs adaptive `K`-means
  Supporting files:
  `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
  `router-research/docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`

- Norm invariance / angular-only mechanism
  Supporting files:
  `router-research/results/analysis/inc0168_norm_geometry.json`
  `router-research/docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md`

- Hardware proxy closure
  Supporting files:
  `router-research/docs/research/increments/INC_0165_hardware_proxy_closure.md`
  `router-research/docs/DECISIONS.md`

- Baseline control showing load-balance auxiliary loss is the wrong comparator
  Supporting files:
  `router-research/results/analysis/inc0172_moe_substitution.json`
  `router-research/docs/research/increments/INC_0172_moe_substitution_study.md`

### Repo-only

- Phase transport as an independent contribution
  Supporting files:
  `router-research/results/analysis/inc0145_phase_transport_stage4_confirm.json`
  `router-research/results/analysis/inc0146_phase_transport_k75_confirm.json`
  `router-research/docs/research/increments/INC_0147_phase_transport_lambda_control.md`

- Spectral/operator work
  Representative files:
  `router-research/results/analysis/inc0150_signal_confirm_ambient.json`
  `router-research/results/analysis/inc0151_signal_finalize_ambient.json`
  `router-research/spectral_emergence_moonshot.md`

- Sparse-event routing line
  Representative files:
  `router-research/results/analysis/inc0158_*`
  `router-research/results/analysis/inc0159_*`
  `router-research/tools/sparse_event_training_efficiency_probe.py`

- Full-program / architecture framing
  Representative files:
  `router-research/CORE_PROJECT_GOALS.md`
  `router-research/GEOMETRIC_COMPUTATION_HYPOTHESIS.md`
  `router-research/THEORY_SKETCH.md`
  `router-research/geometric_routing_architecture_summary.md`
  `router-research/adaptive_field_computer_moonshot.md`

## Historical or internal artifacts that should not drive Paper 1

- All broad preprint PDFs under `preprints/`
- Working rewrite drafts under `submit/`
- Release-bundle duplicates under `router-research/papers/release_bundle/`
- Increment-history language as manuscript structure
- Internal RR / INC process narration except for provenance notes
- Product-manifold, spectral, sparse-event, and moonshot notes that exceed the narrow Paper 1 claim

## Paper-facing recommendation

If the paper is rewritten from scratch, anchor it on this reduced chain:
1. Semantic proxy establishes fixed-routing sparsity and scaling.
2. Large-K extension shows the effect persists.
3. Small trainable LM shows partial transfer.
4. PTB and WT2 both confirm the same narrow transfer ratio.
5. The null result is explicit: trainable LM quality does not distinguish Hopf from permuted fixed routing.

Everything else should justify, qualify, or stay out.
