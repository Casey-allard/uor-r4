## Paper 1 handoff bundle v2 (unconditional far-field: \(\Re s \ge 0.6\))

This folder is a self-contained handoff for **Paper 1**: TeX sources, verifier code, and the exact JSON artifacts the manuscript cites.

**Version v2 Update:**
- Includes the updated `verify_attachment_arb.py` (v2) which computes the L1 tail sum diagnostic.
- Includes updated artifacts (`pick_*.json`) generated with `N=16, Ncoeff=128` and high-precision FFTs to demonstrate negligible analytic tail.
- Includes updated `paper1_farfield.pdf` reflecting the Hybrid Strategy + Finite Pick Check logic.

### 1) Full source bundle (TeX + inputs + figures)

- **Main TeX**: `papers/paper1_farfield.tex`
- **Inputs**:
  - `papers/riemann_common_preamble.tex` (shared preamble)
  - `papers/riemann_bibliography.tex` (inline `thebibliography`; no BibTeX step)
- **Figures**: Paper 1 uses **no `\\includegraphics`** figures.

Compile (twice for references):

```bash
cd paper1_handoff_v2_2026-01-22/papers
pdflatex -interaction=nonstopmode paper1_farfield.tex
pdflatex -interaction=nonstopmode paper1_farfield.tex
```

### 2) Code bundle (verifier scripts)

- **Verifier**: `scripts/verify_attachment_arb.py`
  - Imports: `python-flint` (ARB ball arithmetic) and `numpy`.
  - No local helper modules are required (single-file verifier).

### 3) Artifacts bundle (referenced outputs)

These are the JSON artifacts referenced in `paper1_farfield.tex` (Table `tab:artifact-data`, Remark `rem:artifact-repro`, and Appendix `app:audit`):

- `artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`  (rectangle certification)
- `artifacts/pick_sigma0599_raw_zeta_N16.json`  (primary Pick certificate, \(\sigma_0=0.599\))
- `artifacts/pick_sigma06_raw_zeta_N16.json`    (Pick cross-check, \(\sigma_0=0.6\))
- `artifacts/pick_sigma07_raw_zeta_N16.json`    (Pick cross-check, \(\sigma_0=0.7\))

### Run instructions (exact commands)

The canonical audit commands (and the minimum expected JSON fields) are in:

- `papers/paper1_farfield.tex` — Appendix **“Audit manifest (verifier commands and expected fields)”** (label `app:audit`)
- `meta/README.md` — sections **“Quick Start”** and **“Verification Manifest”**

If you want the exact command lines in one place, copy/paste from Appendix `app:audit` in the TeX source or see `AUDIT_CHECKLIST.md`.

### Pinned environment (minimum)

- `requirements.txt` pins the minimum Python packages used by the verifier.
- `ENVIRONMENT.txt` records the OS/Python/package versions used when this bundle was assembled.

### Pass/fail logs (hashes)

- `SHA256SUMS.txt` lists sha256 hashes of all files in this bundle (excluding the hash file itself).
  - On macOS, you can verify with:

```bash
cd paper1_handoff_v2_2026-01-22
shasum -a 256 -c SHA256SUMS.txt
```

### Extra pointers (original repo metadata)

- `meta/Makefile` and `meta/README.md` are copied from the project root for convenience.
- The three-paper split tracking docs are in `meta/THREE_PAPER_SPLIT_PLAN.md` and `meta/THREE_PAPER_SPLIT_PROGRESS.md`.
