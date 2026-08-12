# Lemma C Theorem-Candidate Note (Triangle Form)

Generated: February 17, 2026
Primary artifact: `/Users/adminamn/Documents/New project/research/output/lemma_c_triangle_transfer_none_z128_ref512.json`

## Non-optimizer Transfer Form
For fixed `M < M_ref`, define
\[
R_M(x;W):=\left|E(x)/\sqrt{x}-(a_{ref}H_W^{(M)}(x)+b_{ref})\right|,
\]
with `(a_ref,b_ref)` fit once on `H^{(M_ref)}`.

Then pointwise by triangle inequality:
\[
R_M(x;W)
\le
\underbrace{\left|E(x)/\sqrt{x}-(a_{ref}H_W^{(M_ref)}(x)+b_{ref})\right|}_{R_{ref}(x;W)}
+|a_{ref}|\,\underbrace{\left|H_W^{(M)}(x)-H_W^{(M_ref)}(x)\right|}_{\Delta_M(x;W)}.
\]

Hence uniformly on tested grid:
\[
R_M(x;W)\le C_0+|a_{ref}|\Delta_M(x;W),\quad C_0:=\sup R_{ref}.
\]

## Calibrated Constants (kernel=none, M=128, M_ref=512)
- `a_ref = -0.0047075423063228546`
- `b_ref = -0.07903726672397497`
- `C0 = 0.6126289168356691`

Grid validation (`x<=10^6`, bases `30,210,2310,30030`):
- holds: `True`
- violations: `0`
- max gap `(lhs-rhs)`: `-0.050994542006281174`

## Optional Split Of C0
Using explicit surrogate at `M_ref`:
\[
C_0 \le C_{smooth}+C_{link}
\]
with calibrated values:
- `C_smooth_ref = 0.1508075667403741`
- `C_link_ref = 0.6381808448814672`
- `C_smooth_ref + C_link_ref = 0.7889884116218413` (conservative split)

## Why this matters
This is a theorem-shaped Lemma C candidate because:
1. coefficient in front of truncation is explicit (`|a_ref|`),
2. transfer is obtained by deterministic inequality (no LP fit),
3. only remaining non-theorem object is uniform control of `C0` and asymptotic control of `\Delta_M` (Lemma B).
