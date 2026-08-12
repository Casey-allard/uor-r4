# A4 Kernel Robustness Note

Generated: February 17, 2026
Secondary artifact: `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_gauss100_z128_ref512.json`

## Result
The unified A1-A4 theorem RHS check also holds under gaussian kernel(scale=100) with:
- zero violations,
- strict negative max gap,
- stable base-uniform behavior.

Key constants (gaussian run):
- `a_ref = -0.0029239859183348943`
- `b_ref = -0.09515364489183957`
- `C0_ref = 0.6110001071680652`
- `C_delta = 9.235747022557273e-04`
- `tau_M = 0.003203703279839466`
- `C_H = 4.929947488048965e-07`

Interpretation: theorem structure appears robust across at least two kernel regimes (`none`, `gaussian(100)`), though constants differ.
