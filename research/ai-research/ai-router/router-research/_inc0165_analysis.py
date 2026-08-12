#!/usr/bin/env python3
"""INC-0165 analysis: hardware proxy closure.

Compares ORIG vs PERM across three hardware proxy models at matched progress.
NO MSE. Cosine used only as matched-progress checkpoint variable.
"""
import json
import os
import numpy as np

ROOT = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(ROOT, "results", "parsed")

K_VALUES = [75, 100, 150, 200]
SEEDS = [0, 1, 2, 3, 4]
PROGRESS_LEVELS = [0.50, 0.60, 0.70, 0.80]
CACHE_LINE_GRANS = [1, 2, 4]
LRU_CAPS = [8, 16, 32]


def load_results(mode, seeds):
    all_seeds = {}
    for s in seeds:
        prefix = "inc0165_trans" if mode == "TRANS" else "inc0165_base"
        path = os.path.join(RESULTS, f"{prefix}_seed{s}.json")
        with open(path, "r") as f:
            data = json.load(f)
        by_route = {}
        for r in data["results"]:
            by_route[r["route_id"]] = r
        all_seeds[s] = by_route
    return all_seeds


def interpolate_step_for_cosine(curve, target_cos):
    for i, pt in enumerate(curve):
        if pt["window_cosine"] >= target_cos:
            if i == 0:
                return pt["step"]
            prev = curve[i - 1]
            frac = ((target_cos - prev["window_cosine"])
                    / max(pt["window_cosine"] - prev["window_cosine"], 1e-12))
            return prev["step"] + frac * (pt["step"] - prev["step"])
    return None


def interpolate_metric_at_step(curve, step, metric_key):
    for i, pt in enumerate(curve):
        if pt["step"] >= step:
            if i == 0:
                return pt[metric_key]
            prev = curve[i - 1]
            frac = (step - prev["step"]) / max(pt["step"] - prev["step"], 1e-12)
            return prev[metric_key] + frac * (pt[metric_key] - prev[metric_key])
    return curve[-1][metric_key] if curve else None


def interpolate_nested_at_step(curve, step, outer_key, inner_key):
    """Interpolate a nested metric like lru_misses["16"] at a given step."""
    for i, pt in enumerate(curve):
        if pt["step"] >= step:
            val = pt[outer_key][inner_key]
            if i == 0:
                return val
            prev = curve[i - 1]
            prev_val = prev[outer_key][inner_key]
            frac = (step - prev["step"]) / max(pt["step"] - prev["step"], 1e-12)
            return prev_val + frac * (val - prev_val)
    return curve[-1][outer_key][inner_key] if curve else None


def main():
    print("=" * 76)
    print("INC-0165: Hardware Proxy Closure — Memory Traffic & Cache Locality")
    print("=" * 76)

    for mode in ["TRANS", "BASE"]:
        print(f"\n{'='*76}")
        print(f"  MODE: {mode}")
        print(f"{'='*76}")

        all_seeds = load_results(mode, SEEDS)

        # ------------------------------------------------------------------
        # PART 1: Final hardware summary (convergence)
        # ------------------------------------------------------------------
        print(f"\n--- PART 1: Hardware Summary at Convergence (5-seed mean) ---")
        print(f"{'K':>5s}  {'eff_ratio':>10s}  {'cost_ratio':>10s}  "
              f"{'lru8_ratio':>10s}  {'lru16_ratio':>11s}  {'lru32_ratio':>11s}")

        for K in K_VALUES:
            orig_id = f"{mode}_K{K}_ORIG"
            perm_id = f"{mode}_K{K}_PERM"

            eff_rs, cost_rs = [], []
            lru_rs = {c: [] for c in LRU_CAPS}

            for s in SEEDS:
                so = all_seeds[s][orig_id]["summary"]
                sp = all_seeds[s][perm_id]["summary"]
                eff_rs.append(sp["effective_bucket_count"] / max(so["effective_bucket_count"], 1))
                cost_rs.append(sp["cumul_eff_cost"] / max(so["cumul_eff_cost"], 1))
                for c in LRU_CAPS:
                    lru_rs[c].append(sp["lru_misses"][str(c)] / max(so["lru_misses"][str(c)], 1))

            print(f"{K:5d}  {np.mean(eff_rs):9.3f}×  {np.mean(cost_rs):9.3f}×  "
                  f"{np.mean(lru_rs[8]):9.3f}×  {np.mean(lru_rs[16]):10.3f}×  "
                  f"{np.mean(lru_rs[32]):10.3f}×")

        # ------------------------------------------------------------------
        # PART 2: Matched-progress hardware costs at p=0.70
        # ------------------------------------------------------------------
        print(f"\n--- PART 2: Matched-Progress Hardware Ratios (p=0.70) ---")
        print(f"{'K':>5s}  {'eff_cost':>10s}  {'cl_G1':>8s}  {'cl_G2':>8s}  "
              f"{'cl_G4':>8s}  {'lru8':>8s}  {'lru16':>8s}  {'lru32':>8s}  "
              f"{'bytes_16':>10s}")

        matched_ratios = {}  # (K, metric) -> list of ratios

        for K in K_VALUES:
            orig_id = f"{mode}_K{K}_ORIG"
            perm_id = f"{mode}_K{K}_PERM"

            ratios = {m: [] for m in ["eff_cost", "cl_1", "cl_2", "cl_4",
                                       "lru_8", "lru_16", "lru_32", "bytes_16"]}

            for s in SEEDS:
                oc = all_seeds[s][orig_id]["progress_curve"]
                pc = all_seeds[s][perm_id]["progress_curve"]
                perm_max = max(pt["window_cosine"] for pt in pc)
                target = 0.70 * perm_max

                so = interpolate_step_for_cosine(oc, target)
                sp = interpolate_step_for_cosine(pc, target)
                if so is None or sp is None or so <= 0:
                    continue

                # Model A: eff cost
                eco = interpolate_metric_at_step(oc, so, "cumul_eff_cost")
                ecp = interpolate_metric_at_step(pc, sp, "cumul_eff_cost")
                if eco and eco > 0:
                    ratios["eff_cost"].append(ecp / eco)

                # Model B: cache-line touches
                for g in CACHE_LINE_GRANS:
                    clo = interpolate_nested_at_step(oc, so, "cache_line_touches", str(g))
                    clp = interpolate_nested_at_step(pc, sp, "cache_line_touches", str(g))
                    if clo and clo > 0:
                        ratios[f"cl_{g}"].append(clp / clo)

                # Model C: LRU misses
                for c in LRU_CAPS:
                    lo = interpolate_nested_at_step(oc, so, "lru_misses", str(c))
                    lp = interpolate_nested_at_step(pc, sp, "lru_misses", str(c))
                    if lo and lo > 0:
                        ratios[f"lru_{c}"].append(lp / lo)

                # Bytes moved (LRU-16)
                bo = interpolate_nested_at_step(oc, so, "bytes_moved_lru16", None) if False else None
                # bytes_moved_lru16 is a flat key, not nested
                bo = interpolate_metric_at_step(oc, so, "bytes_moved_lru16")
                bp = interpolate_metric_at_step(pc, sp, "bytes_moved_lru16")
                if bo and bo > 0:
                    ratios["bytes_16"].append(bp / bo)

            row = f"{K:5d}"
            for m in ["eff_cost", "cl_1", "cl_2", "cl_4",
                       "lru_8", "lru_16", "lru_32", "bytes_16"]:
                rs = ratios[m]
                if rs:
                    row += f"  {np.mean(rs):7.3f}×"
                    matched_ratios[(K, m)] = rs
                else:
                    row += f"  {'N/A':>8s}"
            print(row)

        # ------------------------------------------------------------------
        # PART 3: All progress checkpoints (eff_cost and lru_16 only)
        # ------------------------------------------------------------------
        print(f"\n--- PART 3: Hardware Ratios Across Progress Checkpoints ---")
        print(f"{'K':>5s}  {'metric':>10s}", end="")
        for p in PROGRESS_LEVELS:
            print(f"  {'p='+str(p):>12s}", end="")
        print()

        for K in K_VALUES:
            orig_id = f"{mode}_K{K}_ORIG"
            perm_id = f"{mode}_K{K}_PERM"

            for metric_label, metric_key, is_nested, nest_key in [
                ("eff_cost", "cumul_eff_cost", False, None),
                ("lru_16", "lru_misses", True, "16"),
            ]:
                row = f"{K:5d}  {metric_label:>10s}"
                for p in PROGRESS_LEVELS:
                    rs = []
                    for s in SEEDS:
                        oc = all_seeds[s][orig_id]["progress_curve"]
                        pc = all_seeds[s][perm_id]["progress_curve"]
                        perm_max = max(pt["window_cosine"] for pt in pc)
                        target = p * perm_max

                        so = interpolate_step_for_cosine(oc, target)
                        sp = interpolate_step_for_cosine(pc, target)
                        if so is None or sp is None or so <= 0:
                            continue

                        if is_nested:
                            vo = interpolate_nested_at_step(oc, so, metric_key, nest_key)
                            vp = interpolate_nested_at_step(pc, sp, metric_key, nest_key)
                        else:
                            vo = interpolate_metric_at_step(oc, so, metric_key)
                            vp = interpolate_metric_at_step(pc, sp, metric_key)
                        if vo and vo > 0:
                            rs.append(vp / vo)

                    if rs:
                        row += f"  {np.mean(rs):7.3f}±{np.std(rs):.3f}"
                    else:
                        row += f"  {'N/A':>12s}"
                print(row)

        # ------------------------------------------------------------------
        # PART 4: Seed variance (p=0.70, eff_cost and lru_16)
        # ------------------------------------------------------------------
        print(f"\n--- PART 4: Seed Variance (p=0.70) ---")
        for K in K_VALUES:
            for m in ["eff_cost", "lru_16"]:
                rs = matched_ratios.get((K, m), [])
                if rs:
                    mn = np.mean(rs)
                    sd = np.std(rs)
                    cv = sd / mn if mn > 0 else 0
                    print(f"  K={K} {m:>10s}: mean={mn:.3f}  "
                          f"std={sd:.3f}  CV={cv:.3f}  "
                          f"{'PASS' if cv < 0.30 else 'FAIL'}")

    # ===================================================================
    # OVERALL ASSESSMENT
    # ===================================================================
    print(f"\n{'='*76}")
    print("OVERALL ASSESSMENT")
    print(f"{'='*76}")

    criteria = []

    for mode in ["TRANS", "BASE"]:
        all_seeds = load_results(mode, SEEDS)

        for model_name, metric_key, is_nested, nest_key in [
            ("Model A (eff_cost)", "cumul_eff_cost", False, None),
            ("Model B (cl_G1)", "cache_line_touches", True, "1"),
            ("Model C (lru_16)", "lru_misses", True, "16"),
        ]:
            ratios_by_K = []
            for K in K_VALUES:
                orig_id = f"{mode}_K{K}_ORIG"
                perm_id = f"{mode}_K{K}_PERM"
                rs = []
                for s in SEEDS:
                    oc = all_seeds[s][orig_id]["progress_curve"]
                    pc = all_seeds[s][perm_id]["progress_curve"]
                    perm_max = max(pt["window_cosine"] for pt in pc)
                    target = 0.70 * perm_max
                    so = interpolate_step_for_cosine(oc, target)
                    sp = interpolate_step_for_cosine(pc, target)
                    if so is None or sp is None or so <= 0:
                        continue
                    if is_nested:
                        vo = interpolate_nested_at_step(oc, so, metric_key, nest_key)
                        vp = interpolate_nested_at_step(pc, sp, metric_key, nest_key)
                    else:
                        vo = interpolate_metric_at_step(oc, so, metric_key)
                        vp = interpolate_metric_at_step(pc, sp, metric_key)
                    if vo and vo > 0:
                        rs.append(vp / vo)
                ratios_by_K.append(np.mean(rs) if rs else 0)

            # Criterion 1: ORIG lower cost (all ratios > 1)
            c1 = all(r > 1.0 for r in ratios_by_K)
            criteria.append((f"{mode} {model_name}: ORIG lower cost", c1))

            # Criterion 3: advantage grows or stays with K
            if len(ratios_by_K) >= 2:
                # Check overall trend (first vs last)
                trend = ratios_by_K[-1] >= ratios_by_K[0] - 0.05
                criteria.append((f"{mode} {model_name}: trend with K", trend))

        # Criterion 2: low variance
        for K in K_VALUES:
            orig_id = f"{mode}_K{K}_ORIG"
            perm_id = f"{mode}_K{K}_PERM"
            rs = []
            for s in SEEDS:
                so_s = all_seeds[s][orig_id]["summary"]
                sp_s = all_seeds[s][perm_id]["summary"]
                rs.append(sp_s["cumul_eff_cost"] / max(so_s["cumul_eff_cost"], 1))
            cv = np.std(rs) / np.mean(rs) if np.mean(rs) > 0 else 1
            criteria.append((f"{mode} K={K} CV<0.30", cv < 0.30))

    print()
    n_pass = 0
    for name, passed in criteria:
        status = "PASS" if passed else "FAIL"
        if passed:
            n_pass += 1
        print(f"  [{status}] {name}")

    n_total = len(criteria)
    print(f"\n  {n_pass}/{n_total} criteria pass")

    if n_pass == n_total:
        print("\nOVERALL: PASS -- KEEP")
    elif n_pass >= n_total - 2:
        print(f"\nOVERALL: PARTIAL PASS ({n_pass}/{n_total}) -- KEEP")
    else:
        print(f"\nOVERALL: FAIL ({n_total - n_pass} failures)")


if __name__ == "__main__":
    main()
