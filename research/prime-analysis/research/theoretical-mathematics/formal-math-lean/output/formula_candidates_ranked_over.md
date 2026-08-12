# Ranked Formula Candidates

Generated: 2026-02-16T22:50:57.432318+00:00

## Top Sparse Models (Cross-Family)

1. holdout ~= +0.507220 -1.112790*ent
   score=-0.0782, global_r2=0.0246, transfer_mean=-0.0391, transfer_min=-0.1012

2. holdout ~= +0.547212 -1.012700*ent -0.008025*inv4
   score=-0.2362, global_r2=0.0385, transfer_mean=-0.1237, transfer_min=-0.3285

3. holdout ~= +0.517244 -0.079599*sep -1.309952*ent
   score=-0.2365, global_r2=0.0385, transfer_mean=-0.1239, transfer_min=-0.3288

4. holdout ~= +0.649395 +0.269814*sep -0.035249*inv4
   score=-0.2386, global_r2=0.0383, transfer_mean=-0.1259, transfer_min=-0.3292

5. holdout ~= +0.639140 -0.051849*sep
   score=-0.2530, global_r2=0.0062, transfer_mean=-0.1370, transfer_min=-0.3904

6. holdout ~= +0.461104 -0.228526*sep -1.866272*ent +0.015029*inv4
   score=-0.2611, global_r2=0.0386, transfer_mean=-0.1286, transfer_min=-0.3288

7. holdout ~= +0.653778 -0.009169*inv4
   score=-0.2792, global_r2=0.0184, transfer_mean=-0.1430, transfer_min=-0.4832

8. holdout ~= +0.608197 +0.005611*inv1
   score=-0.3659, global_r2=0.0049, transfer_mean=-0.2269, transfer_min=-0.4808

## Top Dimensionless Invariants

1. ent: mean=-0.101023, cv=0.0045, family_disp=0.0000, sign_consistency=1.000, stability=0.9955
2. wob: mean=+1.508276, cv=0.0074, family_disp=0.0000, sign_consistency=1.000, stability=0.9926
3. inv2: mean=+14.930286, cv=0.0080, family_disp=0.0000, sign_consistency=1.000, stability=0.9920
4. inv5: mean=+1.859855, cv=0.0092, family_disp=0.0000, sign_consistency=1.000, stability=0.9909
5. h3: mean=+0.205964, cv=0.0159, family_disp=0.0000, sign_consistency=1.000, stability=0.9843
6. inv3: mean=+0.112567, cv=0.0175, family_disp=0.0000, sign_consistency=1.000, stability=0.9828
7. inv4: mean=+3.723414, cv=0.0128, family_disp=0.0068, sign_consistency=1.000, stability=0.9808
8. inv1: mean=+2.038950, cv=0.0197, family_disp=0.0000, sign_consistency=1.000, stability=0.9807

## Overnight Validation Plan Inputs

- Validate top 3 sparse models across larger N and additional modulus families.
- Track only top 3 invariants with strongest stability scores.
- Require transfer_min R^2 to remain positive across families before promoting conjectures.
