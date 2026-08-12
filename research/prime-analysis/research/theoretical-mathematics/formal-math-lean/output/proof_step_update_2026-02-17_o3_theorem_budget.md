# Proof Obligation Ledger

- Updated (UTC): `2026-02-17T19:12:30.180075+00:00`

## Obligation States

| id | state | reductions |
|---|---|---:|
| O1 | open | 2 |
| O2 | open | 4 |
| O3 | open | 2 |
| O4 | open | 0 |
| O5 | open | 4 |

## Latest Event

- obligation: `O3`
- summary: Added deterministic sign-sensitive theorem-side constant budget for O3 and linked it into O5 integration.
- removed assumption: Removed reliance on only calibrated O3 constants by introducing an explicit conservative theorem-side budget from deterministic chain replacements.
- remaining work: Prove asymptotic sign-sensitive offdiag and E2/x bounds that justify the theorem budget constants for all x>=x0, all W.
- evidence:
  - `/Users/adminamn/Documents/New project/research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json`
  - `/Users/adminamn/Documents/New project/research/output/o5_integrated_implication_draft_2026-02-17.json`
  - `/Users/adminamn/Documents/New project/research/output/proof_canonical_manifest.json`
