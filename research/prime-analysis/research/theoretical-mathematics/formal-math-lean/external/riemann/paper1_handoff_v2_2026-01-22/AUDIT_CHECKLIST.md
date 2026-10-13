## Paper 1 audit checklist (claim \(\Re s \ge 0.6\) zero-free)

### Where the checklist lives in the paper

- `papers/paper1_farfield.tex`:
  - Table `tab:artifact-data` (summary numbers)
  - Appendix `app:audit` (exact commands + minimum expected JSON fields)

### Claim → artifact → regeneration command

- **Main claim (Theorem `thm:farfield`)**: \(\zeta(s)\neq 0\) for \(\Re s\ge 0.6\).
  - **Primary certificate used in the current proof path**:
    - **Artifact**: `artifacts/pick_sigma0599_raw_zeta_N16.json`
    - **What it certifies**: strict Pick gap at \(\sigma_0=0.599\) for \(N=16\), which globalizes to \(\{\Re s>0.599\}\) (then implies the boundary exclusion \(\Re s\ge 0.6\)).
    - **Regeneration command** (from Appendix `app:audit`):

```bash
python scripts/verify_attachment_arb.py \
  --pick-certify \
  --pick-sigma0 0.599 \
  --pick-N 16 \
  --pick-coeff-count 128 \
  --pick-K 512 \
  --pick-rho 0.4 \
  --pick-rho-bound 0.5 \
  --arith-gauge raw_zeta \
  --arith-P-cut 2000 \
  --prec 1024 \
  --pick-out artifacts/pick_sigma0599_raw_zeta_N16.json
```

- **Finite-rectangle checkpoint** (Table `tab:artifact-data`, Lemma `lem:rect-cert` in Paper 1):
  - **Artifact**: `artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`
  - **What it certifies**: a strict bound \(\max_{[\;0.6,0.7\;]\times[\;0,20\;]}|\Theta_{\rm proj}(s)|<1\) by interval subdivision.
  - **Regeneration command** (from Appendix `app:audit`):

```bash
python scripts/verify_attachment_arb.py \
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
  --theta-out artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json \
  --progress
```

- **Pick cross-check at \(\sigma_0=0.6\)** (Table `tab:artifact-data`):
  - **Artifact**: `artifacts/pick_sigma06_raw_zeta_N16.json`
  - **Regeneration command**:

```bash
python scripts/verify_attachment_arb.py \
  --pick-certify \
  --pick-sigma0 0.6 \
  --pick-N 16 \
  --pick-coeff-count 128 \
  --pick-K 512 \
  --pick-rho 0.4 \
  --pick-rho-bound 0.5 \
  --arith-gauge raw_zeta \
  --arith-P-cut 2000 \
  --prec 1024 \
  --pick-out artifacts/pick_sigma06_raw_zeta_N16.json
```

- **Pick cross-check at \(\sigma_0=0.7\)** (Table `tab:artifact-data`):
  - **Artifact**: `artifacts/pick_sigma07_raw_zeta_N16.json`
  - **Regeneration command**:

```bash
python scripts/verify_attachment_arb.py \
  --pick-certify \
  --pick-sigma0 0.7 \
  --pick-N 16 \
  --pick-coeff-count 128 \
  --pick-K 512 \
  --pick-rho 0.4 \
  --pick-rho-bound 0.5 \
  --arith-gauge raw_zeta \
  --arith-P-cut 2000 \
  --outer-mode rigorous \
  --outer-P-cut 20000 \
  --outer-T 10 --outer-n 200 \
  --prec 1024 \
  --pick-out artifacts/pick_sigma07_raw_zeta_N16.json
```

### Fast audit (no regeneration)

Open each JSON file above and check the minimum expected fields listed in Appendix `app:audit` of `papers/paper1_farfield.tex` (also duplicated in `meta/README.md`).
