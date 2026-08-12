# Tau14/Tau21 Constructive Gate Multiwindow Summary (2026-02-24)

- Windows: `x10000000, x20000000, x30000000, x40000000, x50000000`
- Gate mode: `nonnegative`
- Base threshold: `a1=0.98`
- Robust search windows (`delta_min>0` across all windows): 100, 200, 500, 1000, 2000, 5000, 10000

| search_W | delta_min | delta_max | q_max | rr_max | robust |
|---:|---:|---:|---:|---:|---:|
| 100 | 0.905328 | 0.957403 | 0.074672 | 0.074672 | true |
| 200 | 0.864531 | 0.957212 | 0.115469 | 0.115469 | true |
| 500 | 0.893434 | 0.970715 | 0.086566 | 0.086566 | true |
| 1000 | 0.568890 | 0.963734 | 0.411110 | 0.411110 | true |
| 2000 | 0.744150 | 0.935318 | 0.235850 | 0.235850 | true |
| 5000 | 0.721129 | 0.958686 | 0.258871 | 0.258871 | true |
| 10000 | 0.716286 | 0.971026 | 0.263714 | 0.263714 | true |

Finite-window report only; not a theorem proof.
