# Ranked Formula Candidates

Generated: 2026-02-16T22:53:59.396272+00:00

## Top Sparse Models (Cross-Family)

1. holdout ~= +0.651083 -0.086104*sep
   score=-4.7124, global_r2=0.0226, transfer_mean=-3.3086, transfer_min=-5.5578

2. holdout ~= +0.640775 -0.006008*inv4
   score=-5.5172, global_r2=0.0107, transfer_mean=-3.8226, transfer_min=-6.7091

3. holdout ~= +0.598214 +0.545292*h2
   score=-6.0685, global_r2=0.0393, transfer_mean=-4.2032, transfer_min=-7.4203

4. holdout ~= +0.620305 -0.049019*sep +0.452347*h2
   score=-6.5118, global_r2=0.0454, transfer_mean=-4.4997, transfer_min=-7.9337

5. holdout ~= +0.581676 +0.019840*inv5
   score=-6.7909, global_r2=0.0183, transfer_mean=-4.7057, transfer_min=-8.2790

6. holdout ~= +0.756637 -0.074679*sep +1.080052*ent
   score=-8.7930, global_r2=0.0436, transfer_mean=-6.0866, transfer_min=-10.7092

7. holdout ~= +0.784076 +1.352538*ent -0.007522*inv4
   score=-8.8123, global_r2=0.0436, transfer_mean=-6.0987, transfer_min=-10.7380

8. holdout ~= +0.741831 +1.214046*ent
   score=-8.9722, global_r2=0.0271, transfer_mean=-6.2127, transfer_min=-10.9851

## Top Dimensionless Invariants

1. ent: mean=-0.101760, cv=0.0024, family_disp=0.0000, sign_consistency=1.000, stability=0.9976
2. wob: mean=+1.506945, cv=0.0044, family_disp=0.0000, sign_consistency=1.000, stability=0.9956
3. inv2: mean=+14.809062, cv=0.0067, family_disp=0.0000, sign_consistency=1.000, stability=0.9934
4. inv5: mean=+1.845389, cv=0.0067, family_disp=0.0000, sign_consistency=1.000, stability=0.9933
5. h1: mean=+0.174088, cv=0.0085, family_disp=0.0000, sign_consistency=1.000, stability=0.9916
6. h3: mean=+0.201483, cv=0.0099, family_disp=0.0000, sign_consistency=1.000, stability=0.9902
7. inv4: mean=+3.742705, cv=0.0084, family_disp=0.0022, sign_consistency=1.000, stability=0.9895
8. inv3: mean=+0.115528, cv=0.0112, family_disp=0.0000, sign_consistency=1.000, stability=0.9889

## Overnight Validation Plan Inputs

- Validate top 3 sparse models across larger N and additional modulus families.
- Track only top 3 invariants with strongest stability scores.
- Require transfer_min R^2 to remain positive across families before promoting conjectures.
