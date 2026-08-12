## Riemann handoff bundle (2026-01-29)

This folder is a self-contained handoff for the **three-paper split**:

- **Paper I (far-field)**: `papers/paper1_farfield.tex` (unconditional far-field zero-free region \(\Re s\ge 0.6\))
- **Paper II (near-field barrier)**: `papers/paper2_energy_barrier.tex`
- **Paper III (cutoff / conditional closure)**: `papers/paper3_cutoff_conditional.tex`

It includes the TeX sources + inputs, compiled PDFs, the verifier script, the exact JSON artifacts cited for optional cross-checks, and SHA256 checksums.

### 1) Compile the papers

All papers are designed to compile with `pdflatex` (no BibTeX).

```bash
cd riemann_handoff_2026-01-29/papers
pdflatex -interaction=nonstopmode paper1_farfield.tex
pdflatex -interaction=nonstopmode paper1_farfield.tex
pdflatex -interaction=nonstopmode paper2_energy_barrier.tex
pdflatex -interaction=nonstopmode paper2_energy_barrier.tex
pdflatex -interaction=nonstopmode paper3_cutoff_conditional.tex
pdflatex -interaction=nonstopmode paper3_cutoff_conditional.tex
```

### 2) Verifier + artifacts (optional cross-checks for Paper I)

- **Verifier**: `scripts/verify_attachment_arb.py`
- **Artifacts** (JSON):
  - `artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`
  - `artifacts/pick_sigma0599_raw_zeta_N16.json`
  - `artifacts/pick_sigma06_raw_zeta_N16.json`
  - `artifacts/pick_sigma07_raw_zeta_N16.json`

Paper I includes an appendix **“Optional computational audit manifest”** (label `app:audit`) with exact command lines and minimum expected JSON fields.

### 3) Python environment

- **Pinned packages**: `requirements.txt`
- **Captured build environment**: `ENVIRONMENT.txt`

Install (example):

```bash
python3 -m pip install -r requirements.txt
```

### 4) Integrity / checksums

- **Checksums**: `SHA256SUMS.txt`

Verify on macOS:

```bash
cd riemann_handoff_2026-01-29
shasum -a 256 -c SHA256SUMS.txt
```

### 5) Project metadata

- `meta/README.md` (repo README snapshot)
- `meta/Makefile` (build convenience)
- `meta/THREE_PAPER_SPLIT_PLAN.md`
- `meta/THREE_PAPER_SPLIT_PROGRESS.md`
