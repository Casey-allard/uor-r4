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
- `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.270s`, warm amortized `2.289s`, savings `-0.018s`
  cold offline `2.218s`, warm offline `2.226s`, offline savings `-0.007s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `-2.170s`
- `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `34.589s`, warm amortized `34.857s`, savings `-0.268s`
  cold offline `32.291s`, warm offline `32.862s`, offline savings `-0.570s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `-25.643s`

