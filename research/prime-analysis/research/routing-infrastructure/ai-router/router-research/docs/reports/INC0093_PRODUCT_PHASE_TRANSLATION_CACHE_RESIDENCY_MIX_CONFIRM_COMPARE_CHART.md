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
- `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: cold amortized `2.473s`, warm amortized `0.133s`, savings `2.341s`
  cold offline `2.421s`, warm offline `0.022s`, offline savings `2.399s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `-0.029s`
- `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: cold amortized `34.338s`, warm amortized `2.386s`, savings `31.953s`
  cold offline `32.378s`, warm offline `0.280s`, offline savings `32.099s`
  top1 stable=True, cand_frac stable=True, warm margin vs dense `7.196s`

