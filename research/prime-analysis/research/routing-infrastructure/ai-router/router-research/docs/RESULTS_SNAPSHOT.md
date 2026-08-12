# Results Snapshot (pasted logs)

## BASE: kmeans, radial scaling, no time pressure (lambda=0.0), budget=96
seed=0 after_growth test_mse=0.811499 buckets=70
seed=1 after_growth test_mse=0.843119 buckets=85
seed=2 after_growth test_mse=0.843608 buckets=87
seed=3 after_growth test_mse=0.833795 buckets=59

## POLAR: phase2, radial scaling, no time pressure (lambda=0.0), budget=96
seed=0 after_growth test_mse=0.854181 buckets=56
seed=1 after_growth test_mse=0.837800 buckets=100
seed=2 after_growth test_mse=0.858360 buckets=76
seed=3 after_growth test_mse=0.840959 buckets=108

## Time pressure on kmeans (lambda in 0.25..1.2)
Observed: pre-growth test_mse rises; post-growth improves but usually remains worse than lambda=0.0 baseline.

Conclusion: stop doing long manual runs; implement PR-1..PR-2 first.
