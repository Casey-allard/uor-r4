# Artifact Verification Guide: Unconditional Far-Field Zero-Freeness (\(\Re s\ge 0.6\))

This repository contains the **machine-checkable verification artifacts** associated with **Paper 1** of the three-paper split:

> **“A certified zero-free region for the Riemann zeta function in the half-plane \(\Re s \ge 0.6\)”**  
> Jonathan Washburn (2026)

The goal of this guide is to make it easy for a skeptical referee to **audit the unconditional far-field claim** (the \(\Re s\ge 0.6\) zero-free region) by running a single, rigorous verifier based on **ARB ball arithmetic** (via `python-flint`).

## Papers in this repository (three-paper split)

- **Paper 1 (Unconditional)**: `paper1_farfield.tex` / `paper1_farfield.pdf`  
  *A certified zero-free region for the Riemann zeta function in the half-plane \(\Re s \ge 0.6\)*
- **Paper 2 (Effective / height-limited)**: `paper2_energy_barrier.tex` / `paper2_energy_barrier.pdf`  
  *Energy barriers and Carleson budgets for off-critical zeros of the Riemann zeta function*
- **Paper 3 (Conditional / hypothesis-driven)**: `paper3_cutoff_conditional.tex` / `paper3_cutoff_conditional.pdf`  
  *A cutoff principle and conditional closure of the Riemann Hypothesis*

To compile (run twice for references):

```bash
pdflatex -interaction=nonstopmode paper1_farfield.tex
pdflatex -interaction=nonstopmode paper2_energy_barrier.tex
pdflatex -interaction=nonstopmode paper3_cutoff_conditional.tex
```

Or, using the included `Makefile`:

```bash
make all
```

## Overview: what is being discharged?

Paper 1’s unconditional far-field step is the hybrid certification summarized in:

- Theorem **“Certified far-field zero-freeness”** (`paper1_farfield.tex`, Theorem `thm:farfield`)
- Table **“Certified far-field artifact data”** (`paper1_farfield.tex`, Table `tab:artifact-data`)
- Remark **“Artifact reproducibility and verification”** (`paper1_farfield.tex`, Remark `rem:artifact-repro`)
- Appendix **“Audit manifest (verifier commands and expected fields)”** (`paper1_farfield.tex`, Appendix `app:audit`)

It establishes:

> **Unconditional far-field statement.** The Riemann zeta function has **no zeros** in the region \(\{\Re s\ge 0.6\}\).

The verification uses **rigorous interval arithmetic** throughout. A printed `PASS` is intended to mean:  
**within the logic of ball arithmetic, the relevant inequality is certified and cannot be violated.**

## Prerequisites

- Python 3.9+
- `python-flint` (bindings to FLINT/ARB)

Install:

```bash
pip install python-flint
```

## Files you should look at

- `verify_attachment_arb.py` — the main verifier (ARB / ball arithmetic).
- `theta_certify_sigma06_07_t0_20_outer_zeta_proj.json` — rectangle certification artifact for \([0.6,0.7]\times[0,20]\) (this matches Table `tab:artifact-data`).
- `pick_sigma0599_raw_zeta_N16.json` — Pick certificate artifact at \(\sigma_0=0.599\) with \(N=16\) (this matches Table `tab:artifact-data`; this is the primary half-plane certificate used to cover \(\Re s\ge 0.6\) without a separate tail lemma).
- `pick_sigma06_raw_zeta_N16.json` — Pick certificate artifact at \(\sigma_0=0.6\) with \(N=16\) (cross-check; matches Table `tab:artifact-data`).
- `pick_sigma07_raw_zeta_N16.json` — Pick certificate artifact at \(\sigma_0=0.7\) with \(N=16\) (cross-check; matches Table `tab:artifact-data`).

## Quick Start (recommended)

There are two sensible audit modes:

1. **Fast audit (recommended for referees):** verify the shipped JSON artifacts match the manuscript tables.
2. **Full regeneration:** rerun the certificate computations to reproduce the JSON artifacts from scratch.

### Fast audit (verify shipped artifacts)

- **Rectangle artifact:** open `theta_certify_sigma06_07_t0_20_outer_zeta_proj.json` and check:
  - `results.ok` is `true`
  - `results.theta_hi < 1`
  - `results.processed_boxes = 380764`

- **Pick artifact (primary):** open `pick_sigma0599_raw_zeta_N16.json` and check:
  - `pick.delta_cert` matches Table `tab:artifact-data`
  - `pick.P_spd_at_0` is `true`
  - `pick.tail_weighted_l2_partial_hi` matches Table `tab:artifact-data`

- **Pick artifacts (cross-checks):** open `pick_sigma06_raw_zeta_N16.json` and `pick_sigma07_raw_zeta_N16.json` and check the same fields.

### Full regeneration (optional)

Run the verifier in the repository root:

```bash
cd /path/to/this/repository
python verify_attachment_arb.py --help
```

The key audit modes corresponding to Table `tab:artifact-data` are:

1. **Rectangle audit** (`theta_certify`) for \([0.6,0.7]\times[0,20]\) in the far-field.
2. **Pick certificate audit** (`pick_certify`) at \(\sigma_0=0.599\), \(N=16\), plus the tail bound check.

## Verification Manifest (how Table `tab:artifact-data` is produced)

### 1) Rectangle Check (interval arithmetic subdivision)

**Goal:** certify \(|\\Theta(s)|<1\) on the finite rectangle
\[
[\sigma_{\min},\sigma_{\max}]\times[t_{\min},t_{\max}] = [0.6,0.7]\times[0,20].
\]

**Artifact file (included):** `theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`

Key fields in that JSON (expected):

- `results.ok = true`
- `results.theta_hi = 0.9999928763...`
- `results.processed_boxes = 380764`
- `results.margin_1_minus_theta_hi ≈ 7.12e-6`

**Optional regeneration command (writes a new artifact):**

```bash
python verify_attachment_arb.py \
  --theta-certify \
  --arith-gauge outer_zeta_proj \
  --arith-P-cut 2000 \
  --rect-sigma-min 0.6 --rect-sigma-max 0.7 \
  --rect-t-min 0.0 --rect-t-max 20.0 \
  --outer-mode midpoint \
  --outer-P-cut 2000 \
  --outer-T 50.0 --outer-n 2001 \
  --theta-init-n-sigma 10 --theta-init-n-t 50 \
  --theta-min-sigma-width 0.0001 --theta-min-t-width 0.001 \
  --theta-max-boxes 500000 \
  --prec 260 \
  --theta-out theta_certify_sigma06_07_t0_20_outer_zeta_proj.json \
  --progress
```

### 2) Pick Matrix Certificate (Schur on \(\{\Re s>0.599\}\))

**Goal:** certify a strict Pick gap at \(\sigma_0=0.599\) for the \(16\times 16\) arithmetic Pick minor \(P_{16}(\sigma_0)\):
\[
P_{16}(0.599) \succeq \delta I,\qquad \delta>0.
\]

**Artifact file (included):** `pick_sigma0599_raw_zeta_N16.json`

Key fields in that JSON (expected):

- `pick.delta_cert = 0.5944375202...`
- `pick.P_spd_at_0 = true`
- `pick.tail_weighted_l2_partial_hi = 0.0137007383...`

**Optional regeneration command (writes a new artifact):**

```bash
python verify_attachment_arb.py \
  --pick-certify \
  --pick-sigma0 0.599 \
  --pick-N 16 \
  --pick-coeff-count 32 \
  --pick-K 256 \
  --pick-rho 0.1 \
  --pick-rho-bound 0.2 \
  --arith-gauge raw_zeta \
  --arith-P-cut 2000 \
  --prec 260 \
  --pick-out pick_sigma0599_raw_zeta_N16.json
```

Optional cross-check regenerations can be done by changing `--pick-sigma0` and `--pick-out` to reproduce `pick_sigma06_raw_zeta_N16.json` and `pick_sigma07_raw_zeta_N16.json` (see Table `tab:artifact-data`).

### 3) Tail Bound Audit (stability of the infinite Pick matrix)

**Goal:** ensure truncating to \(N=16\) does not invalidate the infinite-dimensional Schur conclusion.

This is the paper’s “tail perturbation” check: if \(\varepsilon_N\) is the coefficient tail bound and \(C\) is the perturbation constant, verify \(C\varepsilon_N<\delta\).

In Paper 1 (`paper1_farfield.tex`), it is summarized in Proposition `prop:pick-gap` and Table `tab:artifact-data` (see also Appendix `app:audit`).

## Why this is rigorous (and not just numerics)

The verifier uses **ball arithmetic**:

- Every number is stored as an interval (midpoint + radius).
- Every arithmetic operation propagates errors automatically.
- Certification is done by **boolean checks** like “upper bound < 1” or “Cholesky succeeds with positive pivots”.

So a `PASS` indicates there is no mathematical possibility (within the verified enclosures) that the inequality fails.

## Troubleshooting

- **ImportError: No module named flint**  
  Install `python-flint` and ensure you’re using the right Python (`python --version`).

- **Runtime**  
  The rectangle subdivision is compute-heavy. If you only want to audit the shipped result, inspect `theta_certify_sigma06_07_t0_20_outer_zeta_proj.json` and check `results.ok` / `results.theta_hi`.

## Citation

If using these artifacts, cite the manuscript:

> Washburn, J. (2026). *A certified zero-free region for the Riemann zeta function in the half-plane \(\Re s \ge 0.6\)*. Preprint.


