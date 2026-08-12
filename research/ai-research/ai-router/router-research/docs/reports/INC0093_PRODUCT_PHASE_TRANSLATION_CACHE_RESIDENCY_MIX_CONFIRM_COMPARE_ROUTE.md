# Translated Cache Compare

## Source
- cold: `inc0093_product_phase_translation_cache_residency_mix_confirm_cold`
- warm: `inc0093_product_phase_translation_cache_residency_mix_confirm_warm`
- tolerance: `1e-12`

## Route Comparisons
- `DENSE_Q01_T2500`: cold amortized `0.114s`, warm amortized `0.103s`, savings `0.010s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `DENSE_Q01_T40000`: cold amortized `9.187s`, warm amortized `9.581s`, savings `-0.395s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.278s`, warm amortized `2.534s`, savings `-0.256s`
  cold offline `2.226s`, warm offline `2.480s`, offline savings `-0.253s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `-2.431s`
- `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `35.052s`, warm amortized `34.715s`, savings `0.337s`
  cold offline `32.981s`, warm offline `32.751s`, offline savings `0.230s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `-25.133s`

