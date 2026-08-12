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
- `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.320s`, warm amortized `0.067s`, savings `2.254s`
  cold offline `2.225s`, warm offline `0.020s`, offline savings `2.205s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `0.052s`
- `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `36.028s`, warm amortized `2.801s`, savings `33.227s`
  cold offline `33.661s`, warm offline `0.260s`, offline savings `33.401s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `6.412s`

