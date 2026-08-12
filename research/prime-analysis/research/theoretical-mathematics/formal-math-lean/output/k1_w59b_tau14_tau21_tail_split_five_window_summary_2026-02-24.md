# Tau14/Tau21 Tail-Split Multiwindow Summary (2026-02-24)

- Windows: `x10000000, x20000000, x30000000, x40000000, x50000000`
- Base threshold: `a1=0.98`
- Robust mode counts (`delta_split_min>0` across all windows): 2, 4, 32
- Robust mode counts (`delta_split_neg_min>0` across all windows): 2, 4, 8, 16, 24, 32

| M | d_split_min | d_split_neg_min | d_raw_neg_min | q_split_max | q_split_neg_max | q_raw_neg_max | robust_abs | robust_neg |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 0.905328 | 0.905328 | 0.905328 | 0.074672 | 0.074672 | 0.074672 | true | true |
| 4 | 0.169385 | 0.549699 | 0.905328 | 0.810615 | 0.430301 | 0.074672 | true | true |
| 8 | -0.255862 | 0.381508 | 0.905328 | 1.235862 | 0.598492 | 0.074672 | false | true |
| 12 | -1.084099 | -0.089385 | 0.905328 | 2.064099 | 1.069385 | 0.074672 | false | false |
| 16 | -0.065789 | 0.432111 | 0.905328 | 1.045789 | 0.547889 | 0.074672 | false | true |
| 24 | -0.027069 | 0.451472 | 0.905328 | 1.007069 | 0.528528 | 0.074672 | false | true |
| 32 | 0.118578 | 0.524295 | 0.905328 | 0.861422 | 0.455705 | 0.074672 | true | true |

Finite-window report only; not a theorem proof.
