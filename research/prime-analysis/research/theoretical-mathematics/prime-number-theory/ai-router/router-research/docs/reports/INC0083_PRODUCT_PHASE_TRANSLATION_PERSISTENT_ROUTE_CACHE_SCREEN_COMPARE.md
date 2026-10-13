# Translated Cache Compare

## Source
- cold: `inc0083_product_phase_translation_persistent_route_cache_screen_cold`
- warm: `inc0083_product_phase_translation_persistent_route_cache_screen_warm`
- tolerance: `1e-12`

## Route Comparisons
- `DENSE_Q04_T40000`: cold amortized `9.773s`, warm amortized `9.883s`, savings `-0.110s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `H4XH4_FIELD_A150_CPX8_Q04_T40000`: cold amortized `10.641s`, warm amortized `2.021s`, savings `8.620s`
  cold offline `8.519s`, warm offline `0.011s`, offline savings `8.507s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.863s`
- `DENSE_Q08_T40000`: cold amortized `9.558s`, warm amortized `9.822s`, savings `-0.264s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `H4XH4_FIELD_A150_CPX8_Q08_T40000`: cold amortized `6.250s`, warm amortized `2.080s`, savings `4.170s`
  cold offline `4.255s`, warm offline `0.006s`, offline savings `4.250s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.742s`

