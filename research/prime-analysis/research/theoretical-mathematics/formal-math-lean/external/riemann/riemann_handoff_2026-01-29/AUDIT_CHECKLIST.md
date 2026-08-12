## Audit checklist (handoff 2026-01-29)

### Scope

- **Paper I** is an *unconditional* far-field theorem (\(\Re s\ge 0.6\) zero-free) proved via the boundary wedge certificate \((\mathrm{P}^+)\).
- The **JSON artifacts** shipped here are **optional cross-checks** only; they are not load-bearing in the analytic proof.

### Where the exact audit commands live

- `papers/paper1_farfield.tex` → Appendix **“Optional computational audit manifest (verifier commands and expected fields)”** (label `app:audit`).

### Fast audit (no regeneration)

Open each JSON file in `artifacts/` and check the minimum expected fields listed in Paper I’s audit appendix:

- `artifacts/theta_certify_sigma06_07_t0_20_outer_zeta_proj.json`
- `artifacts/pick_sigma0599_raw_zeta_N16.json`
- `artifacts/pick_sigma06_raw_zeta_N16.json`
- `artifacts/pick_sigma07_raw_zeta_N16.json`

### Regeneration audit (optional)

From the bundle root:

```bash
python scripts/verify_attachment_arb.py --help
```

Then copy/paste the exact commands from Paper I’s audit appendix.

### Expected outcome

- The verifier uses **ARB ball arithmetic** (`python-flint`), so the checks are rigorous interval inequalities (e.g. “upper bound < 1”, “LDLᵀ pivots positive”).
- If the audit checks pass, the numerical values summarized in Paper I’s Table `tab:artifact-data` are certified within the logic of ball arithmetic.
