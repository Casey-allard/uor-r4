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
- `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.404s`, warm amortized `0.056s`, savings `2.348s`
  cold offline `2.352s`, warm offline `0.007s`, offline savings `2.345s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.047s`
- `FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `34.014s`, warm amortized `2.019s`, savings `31.995s`
  cold offline `32.060s`, warm offline `0.044s`, offline savings `32.016s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.563s`

