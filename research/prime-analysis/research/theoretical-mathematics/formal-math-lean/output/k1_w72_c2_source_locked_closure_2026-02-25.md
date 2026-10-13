# K1 W72 C2 Source-Locked Closure Package (2026-02-24)

## Objective
Close C2 in theorem-style by source-locking tau-ratio assumptions and proving the `c0=1/2` dichotomy packet.

## Source-Locked Tau Inputs
- source files used: `4`
- `lmfdb`: `https://www.lmfdb.org/L/download_zeros/1-1-1.1-r0-0-0`
- `lmfdb`: `https://www.lmfdb.org/L/download_zeros/1-1-1.1-r0-0-0`
- `odlyzko_100k`: `https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros1`
- `odlyzko_2m_gz`: `https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros6.gz`
- tau1 envelope: `[14.134725141734694631, 14.134725142000000631]` (width `2.653e-10`)
- tau2 envelope: `[21.022039638771556014, 21.022039638999999056]` (width `2.284e-10`)

## Ratio Certificate
- rho interval: `[1.487262003864974425, 1.487262003909051833]`
- rho in (1,2): `true`
- excludes 3/2: `true`
- distance interval to 3/2: `0.012737996091`

## Rational Branch Scan
- q_scan_max: `1000`
- first candidate denominator hit: `None`
- q-lower-bound from scan: `1001`
- implied uniform c0 from scan: `0.999995075057`

## C2(1/2) Dichotomy Closure
- Irrational branch: equidistribution gives positive density for `cos(2*pi*rho*k + beta0) >= 1/2`.
- Rational branch: rho in (1,2) plus rho != 3/2 implies denominator q>=3; periodic geometry gives repeating witnesses with cos >= cos(pi/q) >= 1/2.
- certified C2(1/2) closure under source-lock assumptions: `true`

## Rounding Threshold at c0=1/2
- a1: `0.98`
- X_round upper bound (using tau envelope highs): `35.777750435449`
- Formula: `0.5 + max(tau1_hi/(2*arccos(a1)), tau2_hi)`.

## Combined With W70 Tail Contract
- W70 constants: beta=`0.62`, eta=`0.01`, C_total=`124.268080597763`, x1=`1.0e+21`
- For any A1>0 and 0<q_target<a1, tail threshold remains `X_tail = max(x1, (C_total/(q_target*A1))^(1/eta))`.
- Joint constructive threshold is `X_joint = max(X_round, X_tail)`.

This closes the C2 branch mathematically at the same artifact quality level as W69/W70, with explicit source-locked tau assumptions.
