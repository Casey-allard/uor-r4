# Translated Cache Compare

## Source
- cold: `inc0083_product_phase_translation_persistent_route_cache_confirm_cold`
- warm: `inc0083_product_phase_translation_persistent_route_cache_confirm_warm`
- tolerance: `1e-12`

## Route Comparisons
- `DENSE_Q04_T40000`: cold amortized `9.347s`, warm amortized `9.246s`, savings `0.101s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `H4XH4_FIELD_A150_CPX8_Q04_T40000`: cold amortized `10.389s`, warm amortized `1.972s`, savings `8.416s`
  cold offline `8.397s`, warm offline `0.011s`, offline savings `8.386s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.274s`
- `DENSE_Q08_T40000`: cold amortized `9.506s`, warm amortized `9.113s`, savings `0.394s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `H4XH4_FIELD_A150_CPX8_Q08_T40000`: cold amortized `6.165s`, warm amortized `1.891s`, savings `4.274s`
  cold offline `4.251s`, warm offline `0.006s`, offline savings `4.246s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.222s`

