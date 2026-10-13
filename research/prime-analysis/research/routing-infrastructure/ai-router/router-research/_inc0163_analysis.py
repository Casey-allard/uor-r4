#!/usr/bin/env python3
"""INC-0163 analysis: Matched-progress compute efficiency.

Compares ORIG vs PERM cumulative routing cost at matched learning progress.
Uses cosine similarity as progress metric (NOT MSE).
"""
import json
import os
import numpy as np
from collections import defaultdict

ROOT = os.path.dirname(os.path.abspath(__file__))
RESULTS_DIR = os.path.join(ROOT, "results", "parsed")

SEEDS = [0, 1, 2, 3, 4]
K_VALUES = [75, 100, 150, 200]
MODES = ["TRANS", "BASE"]


def load_results():
    """Load all INC-0163 results, combining TRANS and BASE files."""
    all_data = {}  # (mode, K, transform, seed) -> result dict
    for seed in SEEDS:
        # TRANS results
        trans_path = os.path.join(RESULTS_DIR, f"inc0163_matched_seed{seed}.json")
        if os.path.exists(trans_path):
            with open(trans_path, "r") as f:
                data = json.load(f)
            for r in data["results"]:
                rid = r["route_id"]
                mode = "TRANS" if rid.startswith("TRANS") else "BASE"
                transform = "ORIG" if rid.endswith("ORIG") else "PERM"
                K = r["args"]["K"]
                all_data[(mode, K, transform, seed)] = r

        # BASE results
        base_path = os.path.join(RESULTS_DIR, f"inc0163_matched_base_seed{seed}.json")
        if os.path.exists(base_path):
            with open(base_path, "r") as f:
                data = json.load(f)
            for r in data["results"]:
                rid = r["route_id"]
                mode = "TRANS" if rid.startswith("TRANS") else "BASE"
                transform = "ORIG" if rid.endswith("ORIG") else "PERM"
                K = r["args"]["K"]
                all_data[(mode, K, transform, seed)] = r

    return all_data


def interpolate_step_for_cosine(eval_curve, target_cos):
    """Find the step at which eval cosine first exceeds target_cos.

    Returns interpolated step or None if never reached.
    """
    for i, pt in enumerate(eval_curve):
        if pt["eval_cosine"] >= target_cos:
            if i == 0:
                return pt["step"]
            prev = eval_curve[i - 1]
            # Linear interpolation
            dc = pt["eval_cosine"] - prev["eval_cosine"]
            if dc <= 0:
                return pt["step"]
            frac = (target_cos - prev["eval_cosine"]) / dc
            return prev["step"] + frac * (pt["step"] - prev["step"])
    return None


def interpolate_cost_at_step(progress_curve, target_step):
    """Interpolate cumulative routing cost at a given step."""
    for i, pt in enumerate(progress_curve):
        if pt["step"] >= target_step:
            if i == 0:
                return pt["cumul_unique_buckets"], pt["cumul_effective_buckets"]
            prev = progress_curve[i - 1]
            frac = (target_step - prev["step"]) / (pt["step"] - prev["step"])
            ub = prev["cumul_unique_buckets"] + frac * (pt["cumul_unique_buckets"] - prev["cumul_unique_buckets"])
            eb = prev["cumul_effective_buckets"] + frac * (pt["cumul_effective_buckets"] - prev["cumul_effective_buckets"])
            return ub, eb
    # Beyond last point - use last value
    last = progress_curve[-1]
    return last["cumul_unique_buckets"], last["cumul_effective_buckets"]


def main():
    all_data = load_results()
    print(f"Loaded {len(all_data)} route results\n")

    # ========== PART 1: Summary metrics at convergence ==========
    print("=" * 80)
    print("PART 1: CONVERGENCE SUMMARY (5-seed mean ± std)")
    print("=" * 80)

    for mode in MODES:
        print(f"\n--- {mode} ---")
        print(f"{'K':>5} | {'eff_ORIG':>12} | {'eff_PERM':>12} | {'ratio':>8} | "
              f"{'cos_ORIG':>10} | {'cos_PERM':>10} | {'cos_diff':>10}")
        print("-" * 85)
        for K in K_VALUES:
            eff_orig = []
            eff_perm = []
            cos_orig = []
            cos_perm = []
            for seed in SEEDS:
                ro = all_data.get((mode, K, "ORIG", seed))
                rp = all_data.get((mode, K, "PERM", seed))
                if ro and rp:
                    eff_orig.append(ro["summary"]["effective_bucket_count"])
                    eff_perm.append(rp["summary"]["effective_bucket_count"])
                    cos_orig.append(ro["summary"]["final_eval_cosine"])
                    cos_perm.append(rp["summary"]["final_eval_cosine"])

            if not eff_orig:
                continue

            eo = np.array(eff_orig)
            ep = np.array(eff_perm)
            co = np.array(cos_orig)
            cp = np.array(cos_perm)
            ratio = ep / eo

            print(f"{K:5d} | {eo.mean():8.2f}±{eo.std():4.2f} | "
                  f"{ep.mean():8.2f}±{ep.std():4.2f} | "
                  f"{ratio.mean():6.3f}x | "
                  f"{co.mean():.6f} | {cp.mean():.6f} | "
                  f"{(co - cp).mean():+.6f}")

    # ========== PART 2: Matched-progress analysis ==========
    print("\n" + "=" * 80)
    print("PART 2: MATCHED-PROGRESS COMPUTE EFFICIENCY")
    print("=" * 80)

    # Determine target progress levels from the data
    # Use fractions of the max eval cosine achieved by PERM (the weaker routing)
    for mode in MODES:
        print(f"\n--- {mode} ---")
        for K in K_VALUES:
            # Collect all PERM final cosines to set target levels
            perm_max_cos = []
            for seed in SEEDS:
                rp = all_data.get((mode, K, "PERM", seed))
                if rp:
                    perm_max_cos.append(rp["summary"]["final_eval_cosine"])

            if not perm_max_cos:
                continue

            min_perm_max = min(perm_max_cos)
            # Target levels: 50%, 70%, 90% of the minimum PERM max cosine
            targets = [0.5 * min_perm_max, 0.7 * min_perm_max, 0.9 * min_perm_max]

            print(f"\n  K={K} (target levels: {[f'{t:.4f}' for t in targets]})")
            print(f"  {'target':>8} | {'step_ORIG':>10} | {'step_PERM':>10} | "
                  f"{'ub_ORIG':>8} | {'ub_PERM':>8} | {'ub_ratio':>8} | "
                  f"{'eb_ORIG':>8} | {'eb_PERM':>8} | {'eb_ratio':>8}")
            print("  " + "-" * 100)

            for target in targets:
                steps_orig = []
                steps_perm = []
                ub_orig = []
                ub_perm = []
                eb_orig = []
                eb_perm = []

                for seed in SEEDS:
                    ro = all_data.get((mode, K, "ORIG", seed))
                    rp = all_data.get((mode, K, "PERM", seed))
                    if not ro or not rp:
                        continue

                    so = interpolate_step_for_cosine(ro["eval_curve"], target)
                    sp = interpolate_step_for_cosine(rp["eval_curve"], target)

                    if so is not None and sp is not None:
                        steps_orig.append(so)
                        steps_perm.append(sp)

                        ubo, ebo = interpolate_cost_at_step(ro["progress_curve"], so)
                        ubp, ebp = interpolate_cost_at_step(rp["progress_curve"], sp)
                        ub_orig.append(ubo)
                        ub_perm.append(ubp)
                        eb_orig.append(ebo)
                        eb_perm.append(ebp)

                if not steps_orig:
                    print(f"  {target:.4f} | {'(not reached)':>60}")
                    continue

                so_arr = np.array(steps_orig)
                sp_arr = np.array(steps_perm)
                ubo_arr = np.array(ub_orig)
                ubp_arr = np.array(ub_perm)
                ebo_arr = np.array(eb_orig)
                ebp_arr = np.array(eb_perm)

                print(f"  {target:.4f} | "
                      f"{so_arr.mean():8.0f}±{so_arr.std():3.0f} | "
                      f"{sp_arr.mean():8.0f}±{sp_arr.std():3.0f} | "
                      f"{ubo_arr.mean():6.1f} | {ubp_arr.mean():6.1f} | "
                      f"{(ubp_arr / ubo_arr).mean():6.3f}x | "
                      f"{ebo_arr.mean():6.1f} | {ebp_arr.mean():6.1f} | "
                      f"{(ebp_arr / ebo_arr).mean():6.3f}x")

    # ========== PART 3: Convergence speed comparison ==========
    print("\n" + "=" * 80)
    print("PART 3: CONVERGENCE SPEED (steps to reach 90% of PERM max cosine)")
    print("=" * 80)

    for mode in MODES:
        print(f"\n--- {mode} ---")
        print(f"{'K':>5} | {'steps_ORIG':>12} | {'steps_PERM':>12} | "
              f"{'speed_ratio':>12} | {'seeds_reached':>14}")
        print("-" * 65)

        for K in K_VALUES:
            perm_max_cos = []
            for seed in SEEDS:
                rp = all_data.get((mode, K, "PERM", seed))
                if rp:
                    perm_max_cos.append(rp["summary"]["final_eval_cosine"])
            if not perm_max_cos:
                continue

            target = 0.9 * min(perm_max_cos)
            steps_o = []
            steps_p = []
            for seed in SEEDS:
                ro = all_data.get((mode, K, "ORIG", seed))
                rp = all_data.get((mode, K, "PERM", seed))
                if not ro or not rp:
                    continue
                so = interpolate_step_for_cosine(ro["eval_curve"], target)
                sp = interpolate_step_for_cosine(rp["eval_curve"], target)
                if so is not None and sp is not None:
                    steps_o.append(so)
                    steps_p.append(sp)

            if not steps_o:
                print(f"{K:5d} | {'(target not reached)':>40}")
                continue

            so_arr = np.array(steps_o)
            sp_arr = np.array(steps_p)
            ratio = sp_arr / so_arr  # >1 means ORIG is faster

            print(f"{K:5d} | {so_arr.mean():8.0f}±{so_arr.std():3.0f} | "
                  f"{sp_arr.mean():8.0f}±{sp_arr.std():3.0f} | "
                  f"{ratio.mean():8.3f}x | {len(steps_o)}/{len(SEEDS)}")

    # ========== PART 4: Success criteria check ==========
    print("\n" + "=" * 80)
    print("PART 4: SUCCESS CRITERIA CHECK")
    print("=" * 80)

    # Criterion 1: At matched progress, ORIG requires lower cumulative routing cost
    # at >= 3 of 4 K values
    k_pass_count = {"TRANS": 0, "BASE": 0}
    for mode in MODES:
        print(f"\n--- {mode} ---")
        for K in K_VALUES:
            perm_max_cos = []
            for seed in SEEDS:
                rp = all_data.get((mode, K, "PERM", seed))
                if rp:
                    perm_max_cos.append(rp["summary"]["final_eval_cosine"])
            if not perm_max_cos:
                continue
            target = 0.7 * min(perm_max_cos)

            ub_ratios = []
            eb_ratios = []
            for seed in SEEDS:
                ro = all_data.get((mode, K, "ORIG", seed))
                rp = all_data.get((mode, K, "PERM", seed))
                if not ro or not rp:
                    continue
                so = interpolate_step_for_cosine(ro["eval_curve"], target)
                sp = interpolate_step_for_cosine(rp["eval_curve"], target)
                if so is not None and sp is not None:
                    ubo, ebo = interpolate_cost_at_step(ro["progress_curve"], so)
                    ubp, ebp = interpolate_cost_at_step(rp["progress_curve"], sp)
                    ub_ratios.append(ubp / ubo)
                    eb_ratios.append(ebp / ebo)

            if ub_ratios:
                ub_mean = np.mean(ub_ratios)
                eb_mean = np.mean(eb_ratios)
                pass_str = "PASS" if eb_mean > 1.0 else "FAIL"
                print(f"  K={K}: ub_ratio={ub_mean:.3f}x  eb_ratio={eb_mean:.3f}x  [{pass_str}]")
                if eb_mean > 1.0:
                    k_pass_count[mode] += 1

    print(f"\n  TRANS: {k_pass_count['TRANS']}/4 K values pass (threshold: >= 3)")
    trans_pass = k_pass_count["TRANS"] >= 3
    print(f"  TRANS overall: {'PASS' if trans_pass else 'FAIL'}")

    print(f"  BASE: {k_pass_count['BASE']}/4 K values pass")
    base_pass = k_pass_count["BASE"] >= 3
    print(f"  BASE overall: {'PASS' if base_pass else 'FAIL'}")

    # Criterion 2: Advantage visible across 5-seed mean (convergence summary)
    print("\n  Convergence eff_ratio > 1.0 at all K (5-seed mean):")
    all_pass = True
    for mode in MODES:
        for K in K_VALUES:
            eff_orig = []
            eff_perm = []
            for seed in SEEDS:
                ro = all_data.get((mode, K, "ORIG", seed))
                rp = all_data.get((mode, K, "PERM", seed))
                if ro and rp:
                    eff_orig.append(ro["summary"]["effective_bucket_count"])
                    eff_perm.append(rp["summary"]["effective_bucket_count"])
            if eff_orig:
                ratio = np.mean(eff_perm) / np.mean(eff_orig)
                status = "PASS" if ratio > 1.0 else "FAIL"
                if ratio <= 1.0:
                    all_pass = False
                print(f"    {mode} K={K}: {ratio:.3f}x [{status}]")
    print(f"  All K/mode > 1.0: {'PASS' if all_pass else 'FAIL'}")

    # Overall
    print("\n" + "=" * 80)
    overall = trans_pass and all_pass
    print(f"OVERALL: {'PASS -- KEEP' if overall else 'FAIL'}")
    print("=" * 80)


if __name__ == "__main__":
    main()
