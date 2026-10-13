# Referee package (black-only) — Riemann zero-free region


This package is intended for an internal “referee pass” (proof-reading + reproducibility check).

## Revision note
- **BLACK step 3**: Section 3 (Schur/Herglotz pinch mechanism) rewritten for correctness/clarity; added complex analysis citation (Ahlfors).

## 1) Paper (theory)
- `paper/paper1_farfield.pdf` — compiled PDF for reading
- `paper/paper1_farfield.tex` — main LaTeX source
- `paper/paper1_pplus_proof.tex` — appendix source (P+) proof, input by the main file
- `paper/riemann_common_preamble.tex` — shared preamble
- `paper/riemann_bibliography.tex` — bibliography

### Markup convention
- This bundle is a **black-only** cleanup pass: prior blue/green referee–author markup has been removed from the LaTeX sources.

## 2) Computation / reproducibility
### Core scripts
- `compute/verify_attachment_arb.py` — verifier (python-flint / Arb ball arithmetic)
- `compute/requirements.txt` — Python deps
- `compute/reproduce.sh` — automation script (creates venv + runs the basic zeta rectangle checks)

### Shipped JSON artifacts referenced in the paper’s audit appendix
- `compute/artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`
- `compute/artifacts/pick_sigma0599_raw_zeta_N16.json`
- `compute/artifacts/pick_sigma06_raw_zeta_N16.json`
- `compute/artifacts/pick_sigma07_raw_zeta_N16.json`

### Existing independent audit outputs (Lambda A10)
- `compute/audit_existing/audit_2026-02-02_lambda_a10_bundle.zip` (report + logs + JSON outputs)
- `compute/audit_existing/audit_2026-02-02_lambda_a10/` (same contents, unzipped)

### Theta regeneration attempt artifacts (if present)
- `compute/audit_existing/theta_certify_sigma06_07_t0_20_outer_zeta_proj_REGEN.json`
- `compute/audit_existing/run_out_logs/theta_certify_sigma06_07_t0_20_REGEN.log`

## 3) Quick-start (local)
### Read the paper
Open `paper/paper1_farfield.pdf`.

### Reproduce computations (x86_64 only)
From `compute/`:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
bash reproduce.sh
```

For full regeneration of the *paper-listed* theta/pick artifacts, use the exact commands in `paper/paper1_farfield.tex` Appendix “Supplementary computational audit manifest”.
