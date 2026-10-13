# Paper 1 Experimental Setup

This document records the paper-facing setup details for Paper 1 using canonical sources only.

## Overview

Paper 1 uses two experimental settings:
- a semantic proxy based on static structured embeddings,
- a toy trainable language model with fixed-routing and learned-routing comparisons.

The paper’s strongest result is proxy-side. The trainable result is a transfer test at toy scale.

## Datasets

### Semantic proxy

Canonical source files indicate a PPMI-SVD proxy built from WikiText-2-derived data.

Relevant files:
- `router-research/results/analysis/inc0143_ppmi_semantic_finalize.json`
- `router-research/results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`

Canonical dataset inputs referenced in those analyses:
- `data/wikitext2_proxy/ppmi_proxy.npz`

Proxy regime characteristics:
- structured static embeddings,
- no end-to-end LM backpropagation,
- routing evaluated as a geometric/static routing problem rather than a full language-model training problem.

### Trainable language model

Penn Treebank:
- canonical LM transfer result source:
  `router-research/results/analysis/inc0171_lm_integration.json`

WikiText-2:
- canonical replication source:
  `router-research/results/analysis/inc0173_wt2_confirm.json`

## Model sizes

### Trainable LM architecture

From the canonical LM analysis files:
- number of layers: `2`
- hidden dimension: `128`
- number of attention heads: `4`
- vocabulary size: `5000`
- sequence length: `32`
- number of experts: `64`
- expert dimension: `128`

Paper-facing interpretation:
- this is a 2-layer toy transformer-like LM used to test routing transfer,
- not a production-scale MoE system.

## Training steps

Canonical LM training horizon:
- `4000` steps for PTB confirm
- `4000` steps for WT2 confirm

These are the values encoded in:
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`

## Seeds

### Semantic proxy

The proxy evidence chain includes multiple closed increments with different seed standards.
For the paper-facing setup, the important point is:
- the proxy claim is supported by late closed multi-run analyses rather than a single run,
- the canonical scaling and persistence claims come from the closed Stage 7 analysis chain.

### Trainable LM

PTB:
- `2` seeds in the canonical confirm result

WT2:
- `2` seeds in the canonical confirm result

Relevant files:
- `router-research/results/analysis/inc0171_lm_integration.json`
- `router-research/results/analysis/inc0173_wt2_confirm.json`

## Routing budgets

### Semantic proxy

Canonical proxy scaling range used for Paper 1:
- anchor range: `K = 25` to `400`
- large-`K` extension: up to `K = 5000`

Canonical analysis files:
- `router-research/results/analysis/inc0168_norm_geometry.json`
- `router-research/results/analysis/inc0170_large_k.json`

### Trainable LM

Canonical LM routing budget:
- `K = 64`

## Conditions and controls

### Semantic proxy

Main conditions:
- Hopf fixed routing
- permuted fixed routing control
- dense routing baseline

Structural controls in the wider canonical evidence chain:
- structured vs column-permuted embeddings
- Hopf vs adaptive `K`-means
- norm-invariance / shell-activation comparisons

### Trainable LM

Main paper conditions:
- `BASELINE`: learned top-1 routing, no auxiliary load-balancing loss
- `HOPF`: fixed Hopf routing, no learned gate matrix
- `PERMUTED`: fixed routing with permuted sector assignments

Baseline-rationale control:
- `LEARNED_SPARSE` with load-balancing auxiliary loss appears in the canonical control analysis
- used to justify why the main paper comparator remains the no-aux top-1 baseline

Relevant canonical control source:
- `router-research/results/analysis/inc0172_moe_substitution.json`

## Metrics

### Effective routing footprint

Paper-facing definition:
- the effective routing footprint summarizes how many routing sectors or expert paths are effectively active under the empirical routing distribution.

Operationally in the canonical analyses:
- it is reported as `effective_bucket_count` on the proxy
- it is reported as `eff_buckets` in the trainable LM setting
- lower values relative to the full routing budget indicate more concentrated routing

This is the main routing-efficiency metric in Paper 1.

### Proxy-side metrics

Canonical proxy metrics include:
- effective bucket count
- routing gini
- sector entropy
- compression ratios between controls and main route families

### LM-side metrics

Canonical LM metrics include:
- validation perplexity
- effective expert-path count (`eff_buckets`)
- HOPF / BASELINE perplexity ratio
- HOPF vs PERMUTED delta in perplexity
- effective-path ratio versus dense routing

## What Paper 1 should say about setup

- The proxy is the main geometry testbed.
- The LM is a toy transfer test.
- The paper compares fixed routing against learned top-1 routing and a random fixed-routing null.
- The paper does not claim large-scale LM validation.
