# O3 A3_ref Asymptotic Scaffold

Generated: 2026-02-17T19:09:43.893517+00:00

## Quantified Statement

There exist x0>=3 and constants C_H>=0, A_H>=0 such that for all x>=x0 and W in {30,210,2310,30030}, |H_W^(M_ref)(x)| <= C_H (log x)^A_H.

## Proof-Chain Template

- Bound eta_+(x;W) by C_eta (log x)^A_eta uniformly in W and x>=x0.
- Use deterministic identity |H| <= sqrt((1+eta_+) E2/x).
- Bound E2/x uniformly and combine exponents/constants.

## Current Constant Candidates

- `m_ref = 512`
- `A_eta = 4.0`
- `C_eta_uplifted = 2.0682571024001217`
- `A_H = 1.2`
- `C_H = 3.47471643488075`

## Grid Validation Snapshot

- valid holds: `True`
- valid ratio max: `0.8894251462871531`
- valid max gap h-rhs: `-10.242604505187373`
- deterministic eta holds: `True`
- deterministic eta max gap: `-5.220266329722577`

## Remaining O3 Blockers

- Replace safety-factor calibrated C_eta with theorem-side sign-sensitive offdiag constants.
- Prove eta_+ upper bound uniformly for all x>=x0 and all W in the wheel family.
- Prove asymptotic bound for E2/x independent of sampled grids.
