# Translated Cache Compare

## Source
- cold: `inc0093_product_phase_translation_cache_residency_mix_screen_cold`
- warm: `inc0093_product_phase_translation_cache_residency_mix_screen_warm`
- tolerance: `1e-12`

## Route Comparisons
- `DENSE_Q01_T2500`: cold amortized `0.126s`, warm amortized `0.118s`, savings `0.008s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `DENSE_Q01_T40000`: cold amortized `9.525s`, warm amortized `9.213s`, savings `0.312s`
  cold offline `0.000s`, warm offline `0.000s`, offline savings `0.000s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.000s`
- `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.664s`, warm amortized `0.063s`, savings `2.601s`
  cold offline `2.604s`, warm offline `0.007s`, offline savings `2.597s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.055s`
- `FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `35.771s`, warm amortized `2.035s`, savings `33.735s`
  cold offline `33.749s`, warm offline `0.044s`, offline savings `33.705s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.178s`

