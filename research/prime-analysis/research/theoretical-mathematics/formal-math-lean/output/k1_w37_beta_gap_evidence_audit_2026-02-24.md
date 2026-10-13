# K1 Beta-Gap Evidence Audit (2026-02-24)

## Global Stats
- count: 2455
- min: 0.5
- q05: 0.52
- median: 0.62
- q95: 0.75
- max: 0.94
- fraction_gt_threshold: 0.9531568228105907

## Empirical Probe Summary
- files: 112
- files_all_beta_gt_threshold: 94
- fraction_files_all_beta_gt_threshold: 0.8392857142857143

## Key Interpretation
- No file in this audit provides an unconditional theorem proving uniform beta-gap > threshold.
- Many empirical probes operate with beta significantly above threshold, which is supportive but not a formal proof.

| class | files | all_beta_gt_threshold_files |
|:--|---:|---:|
| constant_propagation_math | 14 | 11 |
| empirical_model_probe | 112 | 94 |
| other | 109 | 107 |
