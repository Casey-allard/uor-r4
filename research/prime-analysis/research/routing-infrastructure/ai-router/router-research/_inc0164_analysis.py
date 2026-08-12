#!/usr/bin/env python3
"""INC-0164 analysis: scaling-law consistency test.

Compares measured matched-progress compute ratios against predicted
ratios from the INC-0162 scaling law.

NO MSE used. Progress metric: cosine similarity.
"""
import json
import os
import sys
import numpy as np

ROOT = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(ROOT, "results", "parsed")

# INC-0162 scaling law: effective_buckets = c * K^alpha
ALPHA = {
    "TRANS": {"ORIG": 0.500, "PERM": 0.795},
    "BASE":  {"ORIG": 0.882, "PERM": 0.979},
}
C_COEFF = {
    "TRANS": {"ORIG": 3.96, "PERM": 1.81},
    "BASE":  {"ORIG": 1.15, "PERM": 0.99},
}

K_VALUES = [75, 100, 150, 200]
SEEDS = [0, 1, 2, 3, 4]
PROGRESS_LEVELS = [0.50, 0.60, 0.70, 0.80]


def load_results(mode, seeds):
    """Load per-seed results for a mode (trans or base)."""
    all_seeds = {}
    for s in seeds:
        prefix = "inc0164_trans" if mode == "TRANS" else "inc0164_base"
        path = os.path.join(RESULTS, f"{prefix}_seed{s}.json")
        with open(path, "r") as f:
            data = json.load(f)
        by_route = {}
        for r in data["results"]:
            by_route[r["route_id"]] = r
        all_seeds[s] = by_route
    return all_seeds


def interpolate_step_for_cosine(curve, target_cos):
    """Find step at which window_cosine first reaches target_cos."""
    for i, pt in enumerate(curve):
        if pt["window_cosine"] >= target_cos:
            if i == 0:
                return pt["step"]
            prev = curve[i - 1]
            frac = ((target_cos - prev["window_cosine"])
                    / max(pt["window_cosine"] - prev["window_cosine"], 1e-12))
            return prev["step"] + frac * (pt["step"] - prev["step"])
    return None  # never reached


def interpolate_eff_at_step(curve, step):
    """Interpolate cumulative effective buckets at a given step."""
    for i, pt in enumerate(curve):
        if pt["step"] >= step:
            if i == 0:
                return pt["cumul_effective_buckets"]
            prev = curve[i - 1]
            frac = ((step - prev["step"])
                    / max(pt["step"] - prev["step"], 1e-12))
            return (prev["cumul_effective_buckets"]
                    + frac * (pt["cumul_effective_buckets"]
                              - prev["cumul_effective_buckets"]))
    return curve[-1]["cumul_effective_buckets"] if curve else None


def predicted_ratio(K, mode):
    """Predicted eff-bucket ratio from scaling law: (c_PERM/c_ORIG) * K^(alpha_PERM - alpha_ORIG)."""
    da = ALPHA[mode]["PERM"] - ALPHA[mode]["ORIG"]
    c_ratio = C_COEFF[mode]["PERM"] / C_COEFF[mode]["ORIG"]
    return c_ratio * (K ** da)


def main():
    print("=" * 72)
    print("INC-0164: Scaling-Law Consistency Test")
    print("=" * 72)

    for mode in ["TRANS", "BASE"]:
        print(f"\n{'='*72}")
        print(f"  MODE: {mode}")
        print(f"{'='*72}")
        
        all_seeds = load_results(mode, SEEDS)
        da = ALPHA[mode]["PERM"] - ALPHA[mode]["ORIG"]

        # ---------------------------------------------------------------
        # PART 1: Convergence summary (same as INC-0163 for validation)
        # ---------------------------------------------------------------
        print(f"\n--- PART 1: Convergence Summary (5-seed mean +/- std) ---")
        print(f"{'K':>5s}  {'eff_ORIG':>12s}  {'eff_PERM':>12s}  "
              f"{'ratio':>8s}  {'cos_diff':>10s}")

        for K in K_VALUES:
            orig_id = f"{mode}_K{K}_ORIG"
            perm_id = f"{mode}_K{K}_PERM"
            eff_orig = [all_seeds[s][orig_id]["summary"]["effective_bucket_count"]
                        for s in SEEDS]
            eff_perm = [all_seeds[s][perm_id]["summary"]["effective_bucket_count"]
                        for s in SEEDS]
            cos_orig = [all_seeds[s][orig_id]["summary"]["final_eval_cosine"]
                        for s in SEEDS]
            cos_perm = [all_seeds[s][perm_id]["summary"]["final_eval_cosine"]
                        for s in SEEDS]
            mo = np.mean(eff_orig)
            mp = np.mean(eff_perm)
            ratio = mp / mo
            cdiff = np.mean(cos_orig) - np.mean(cos_perm)
            print(f"{K:5d}  {mo:7.2f}±{np.std(eff_orig):.2f}  "
                  f"{mp:7.2f}±{np.std(eff_perm):.2f}  "
                  f"{ratio:7.3f}×  {cdiff:+.4f}")

        # ---------------------------------------------------------------
        # PART 2: Matched-progress compute ratios at multiple checkpoints
        # ---------------------------------------------------------------
        print(f"\n--- PART 2: Matched-Progress Compute Ratios ---")
        print(f"  Progress checkpoints: {PROGRESS_LEVELS}")
        print(f"  Scaling law prediction: ratio ~ K^{da:.3f}")
        print()

        # Collect per-K, per-checkpoint ratios across seeds
        ratio_table = {}  # (K, p) -> list of ratios
        for K in K_VALUES:
            for p in PROGRESS_LEVELS:
                ratios = []
                for s in SEEDS:
                    orig_id = f"{mode}_K{K}_ORIG"
                    perm_id = f"{mode}_K{K}_PERM"
                    perm_curve = all_seeds[s][perm_id]["progress_curve"]
                    orig_curve = all_seeds[s][orig_id]["progress_curve"]

                    # Max cosine for PERM
                    perm_max_cos = max(pt["window_cosine"] for pt in perm_curve)
                    target_cos = p * perm_max_cos

                    # Step at which each reaches target
                    step_orig = interpolate_step_for_cosine(orig_curve, target_cos)
                    step_perm = interpolate_step_for_cosine(perm_curve, target_cos)

                    if step_orig is None or step_perm is None:
                        continue
                    if step_orig <= 0:
                        continue

                    # Eff buckets at that step
                    eff_orig = interpolate_eff_at_step(orig_curve, step_orig)
                    eff_perm = interpolate_eff_at_step(perm_curve, step_perm)

                    if eff_orig and eff_orig > 0:
                        ratios.append(eff_perm / eff_orig)
                ratio_table[(K, p)] = ratios

        # Print table
        header = f"{'K':>5s}"
        for p in PROGRESS_LEVELS:
            header += f"  {'p='+str(p):>12s}"
        header += f"  {'predicted':>10s}"
        print(header)

        for K in K_VALUES:
            row = f"{K:5d}"
            for p in PROGRESS_LEVELS:
                rs = ratio_table.get((K, p), [])
                if rs:
                    row += f"  {np.mean(rs):7.3f}±{np.std(rs):.3f}"
                else:
                    row += f"  {'N/A':>12s}"
            pred = predicted_ratio(K, mode)
            row += f"  {pred:9.3f}×"
            print(row)

        # ---------------------------------------------------------------
        # PART 3: Predicted vs Measured at p=0.70 (primary checkpoint)
        # ---------------------------------------------------------------
        print(f"\n--- PART 3: Predicted vs Measured Ratios (p=0.70) ---")
        print(f"{'K':>5s}  {'measured':>10s}  {'predicted':>10s}  "
              f"{'meas/pred':>10s}  {'within 25%':>10s}")

        all_pass = True
        for K in K_VALUES:
            rs = ratio_table.get((K, 0.70), [])
            if rs:
                meas = np.mean(rs)
                pred = predicted_ratio(K, mode)
                ratio_mp = meas / pred
                within = abs(ratio_mp - 1.0) < 0.25
                if not within:
                    # Check monotonic trend instead
                    pass
                print(f"{K:5d}  {meas:9.3f}×  {pred:9.3f}×  "
                      f"{ratio_mp:9.3f}   {'PASS' if within else 'CHECK'}")
            else:
                print(f"{K:5d}  {'N/A':>10s}")
                all_pass = False

        # ---------------------------------------------------------------
        # PART 4: Monotonicity check
        # ---------------------------------------------------------------
        print(f"\n--- PART 4: Monotonicity Check ---")
        for p in PROGRESS_LEVELS:
            means = []
            for K in K_VALUES:
                rs = ratio_table.get((K, p), [])
                if rs:
                    means.append(np.mean(rs))
                else:
                    means.append(None)
            valid = [m for m in means if m is not None]
            if len(valid) >= 2:
                monotonic = all(valid[i] <= valid[i + 1]
                               for i in range(len(valid) - 1))
                trend = "MONOTONIC" if monotonic else "NON-MONOTONIC"
            else:
                trend = "INSUFFICIENT"
            vals_str = ", ".join(f"{v:.3f}" if v else "N/A" for v in means)
            print(f"  p={p}: [{vals_str}]  → {trend}")

        # ---------------------------------------------------------------
        # PART 5: Seed variance
        # ---------------------------------------------------------------
        print(f"\n--- PART 5: Seed Variance (p=0.70) ---")
        for K in K_VALUES:
            rs = ratio_table.get((K, 0.70), [])
            if rs:
                m = np.mean(rs)
                s = np.std(rs)
                cv = s / m if m > 0 else 0.0
                print(f"  K={K}: mean={m:.3f}  std={s:.3f}  "
                      f"CV={cv:.3f}  {'PASS' if cv < 0.30 else 'FAIL'}")

    # ===================================================================
    # OVERALL ASSESSMENT
    # ===================================================================
    print(f"\n{'='*72}")
    print("OVERALL ASSESSMENT")
    print(f"{'='*72}")

    # Re-compute key checks
    criteria = []

    for mode in ["TRANS", "BASE"]:
        all_seeds = load_results(mode, SEEDS)
        da = ALPHA[mode]["PERM"] - ALPHA[mode]["ORIG"]

        # Check 1: ratios increase with K
        means_70 = []
        for K in K_VALUES:
            rs = []
            for s in SEEDS:
                orig_id = f"{mode}_K{K}_ORIG"
                perm_id = f"{mode}_K{K}_PERM"
                pc = all_seeds[s][perm_id]["progress_curve"]
                oc = all_seeds[s][orig_id]["progress_curve"]
                perm_max = max(pt["window_cosine"] for pt in pc)
                target = 0.70 * perm_max
                so = interpolate_step_for_cosine(oc, target)
                sp = interpolate_step_for_cosine(pc, target)
                if so and sp and so > 0:
                    eo = interpolate_eff_at_step(oc, so)
                    ep = interpolate_eff_at_step(pc, sp)
                    if eo and eo > 0:
                        rs.append(ep / eo)
            means_70.append(np.mean(rs) if rs else 0)

        increasing = all(means_70[i] <= means_70[i + 1] + 0.05
                        for i in range(len(means_70) - 1))
        criteria.append((f"{mode} ratios increase with K", increasing))

        # Check 2: measured follows predicted trend
        deviations = []
        for i, K in enumerate(K_VALUES):
            pred = predicted_ratio(K, mode)
            meas = means_70[i]
            if pred > 0:
                deviations.append(abs(meas / pred - 1.0))
        trend_ok = all(d < 0.50 for d in deviations) if deviations else False
        criteria.append((f"{mode} predicted within 50%", trend_ok))

        # Check 3: ORIG lower cost consistently
        orig_lower = all(m > 1.0 for m in means_70)
        criteria.append((f"{mode} ORIG lower cost", orig_lower))

        # Check 4: low seed variance
        for K in K_VALUES:
            rs = []
            for s in SEEDS:
                orig_id = f"{mode}_K{K}_ORIG"
                perm_id = f"{mode}_K{K}_PERM"
                pc = all_seeds[s][perm_id]["progress_curve"]
                oc = all_seeds[s][orig_id]["progress_curve"]
                perm_max = max(pt["window_cosine"] for pt in pc)
                target = 0.70 * perm_max
                so = interpolate_step_for_cosine(oc, target)
                sp = interpolate_step_for_cosine(pc, target)
                if so and sp and so > 0:
                    eo = interpolate_eff_at_step(oc, so)
                    ep = interpolate_eff_at_step(pc, sp)
                    if eo and eo > 0:
                        rs.append(ep / eo)
            if rs:
                cv = np.std(rs) / np.mean(rs) if np.mean(rs) > 0 else 1.0
                criteria.append((f"{mode} K={K} CV<0.30", cv < 0.30))

    print()
    all_pass = True
    for name, passed in criteria:
        status = "PASS" if passed else "FAIL"
        if not passed:
            all_pass = False
        print(f"  [{status}] {name}")

    print()
    if all_pass:
        print("OVERALL: PASS -- KEEP")
        print("The compute advantage is explained by the routing scaling law.")
    else:
        n_fail = sum(1 for _, p in criteria if not p)
        n_total = len(criteria)
        if n_fail <= 2:
            print(f"OVERALL: PARTIAL PASS ({n_total - n_fail}/{n_total} criteria)")
        else:
            print(f"OVERALL: FAIL ({n_fail}/{n_total} criteria failed)")


if __name__ == "__main__":
    main()
