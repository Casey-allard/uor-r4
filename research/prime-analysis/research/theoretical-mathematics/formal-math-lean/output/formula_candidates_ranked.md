# Ranked Formula Candidates

Generated: 2026-02-16T22:43:00.713561+00:00

## Top Sparse Models (Cross-Family)

1. holdout ~= +0.999495 -0.025454*inv2
   score=0.7615, global_r2=0.6703, transfer_mean=0.5066, transfer_min=0.4292

2. holdout ~= +0.974653 -0.028540*inv2 +0.018953*inv4
   score=0.7582, global_r2=0.7067, transfer_mean=0.5172, transfer_min=0.4173

3. holdout ~= +0.980235 +0.087880*sep -0.026389*inv2
   score=0.7444, global_r2=0.6794, transfer_mean=0.5050, transfer_min=0.4379

4. holdout ~= +0.928923 +0.283770*h2 -0.021408*inv2
   score=0.7129, global_r2=0.7060, transfer_mean=0.4864, transfer_min=0.3601

5. holdout ~= +0.953883 +0.124270*h2 -0.025509*inv2 +0.011221*inv4
   score=0.6688, global_r2=0.7074, transfer_mean=0.4691, transfer_min=0.3315

6. holdout ~= +0.746727 +0.648554*h2 -0.080911*inv5
   score=0.5346, global_r2=0.5140, transfer_mean=0.3660, transfer_min=0.3203

7. holdout ~= +0.771331 +1.070244*h2 -0.050835*inv4
   score=0.5127, global_r2=0.6569, transfer_mean=0.3536, transfer_min=0.1394

8. holdout ~= +0.377185 +2.757841*h3 +3.238268*inv3 -0.371142*inv5
   score=0.4739, global_r2=0.6920, transfer_mean=0.3174, transfer_min=0.1741

## Top Dimensionless Invariants

1. ent: mean=-0.101023, cv=0.0045, family_disp=0.0000, sign_consistency=1.000, stability=0.9955
2. wob: mean=+1.508276, cv=0.0074, family_disp=0.0000, sign_consistency=1.000, stability=0.9926
3. inv2: mean=+14.930286, cv=0.0080, family_disp=0.0000, sign_consistency=1.000, stability=0.9920
4. inv5: mean=+1.859855, cv=0.0092, family_disp=0.0000, sign_consistency=1.000, stability=0.9909
5. inv4: mean=+3.741054, cv=0.0113, family_disp=0.0011, sign_consistency=1.000, stability=0.9878
6. h3: mean=+0.205964, cv=0.0159, family_disp=0.0000, sign_consistency=1.000, stability=0.9843
7. inv3: mean=+0.112567, cv=0.0175, family_disp=0.0000, sign_consistency=1.000, stability=0.9828
8. inv1: mean=+2.038950, cv=0.0197, family_disp=0.0000, sign_consistency=1.000, stability=0.9807

## Overnight Validation Plan Inputs

- Validate top 3 sparse models across larger N and additional modulus families.
- Track only top 3 invariants with strongest stability scores.
- Require transfer_min R^2 to remain positive across families before promoting conjectures.
