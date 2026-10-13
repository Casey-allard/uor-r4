#!/usr/bin/env python3
"""INC-0166 Part B analysis: K=100 boundary audit.

Analyzes shell/sector distributions across K=[75,90,100,110,125,150]
to diagnose the reproducible K≈100 dip in BASE effective-bucket ratios.
"""
import json
import os
import numpy as np

ROOT = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(ROOT, "results", "parsed")

K_VALUES = [75, 90, 100, 110, 125, 150]
SEEDS = [0, 1, 2, 3, 4]


def load_results(mode, seeds):
    all_seeds = {}
    for s in seeds:
        prefix = f"inc0166_{mode.lower()}"
        path = os.path.join(RESULTS, f"{prefix}_seed{s}.json")
        with open(path, "r") as f:
            data = json.load(f)
        by_route = {}
        for r in data["results"]:
            by_route[r["route_id"]] = r
        all_seeds[s] = by_route
    return all_seeds


def main():
    print("=" * 80)
    print("INC-0166 Part B: K=100 Boundary Audit — Sector Discretization Analysis")
    print("=" * 80)

    for mode in ["BASE", "TRANS"]:
        print(f"\n{'='*80}")
        print(f"  MODE: {mode}")
        print(f"{'='*80}")

        all_seeds = load_results(mode, SEEDS)

        # ---- Table 1: Static routing structure ----
        print(f"\n--- Table 1: Static Routing Structure (5-seed mean) ---")
        print(f"{'K':>5s} {'type':>6s}  {'shells':>6s} {'sectors':>7s} "
              f"{'buckets':>8s} {'eff':>8s} {'gini':>8s} "
              f"{'top_half':>9s} {'sect_ent':>9s}")

        for K in K_VALUES:
            for typ in ["ORIG", "PERM"]:
                rid = f"{mode}_K{K}_{typ}"
                shells, sectors, buckets = [], [], []
                effs, ginis = [], []
                top_halfs, sect_ents = [], []

                for s in SEEDS:
                    r = all_seeds[s][rid]
                    st = r["static"]
                    shells.append(st["n_active_shells"])
                    sectors.append(st["n_active_sectors"])
                    buckets.append(st["n_active_buckets"])
                    effs.append(st["bucket_eff"])
                    ginis.append(st["bucket_gini"])
                    top_halfs.append(st["top_half_concentration"])
                    sect_ents.append(st["sector_entropy"])

                print(f"{K:5d} {typ:>6s}  {np.mean(shells):6.1f} "
                      f"{np.mean(sectors):7.1f} {np.mean(buckets):8.1f} "
                      f"{np.mean(effs):8.1f} {np.mean(ginis):8.4f} "
                      f"{np.mean(top_halfs):9.4f} {np.mean(sect_ents):9.4f}")

        # ---- Table 2: Effective bucket ratio PERM/ORIG ----
        print(f"\n--- Table 2: Effective Bucket Ratio (PERM/ORIG, 5-seed) ---")
        print(f"{'K':>5s}  {'mean':>8s}  {'std':>8s}  {'min':>8s}  {'max':>8s}  "
              f"{'trend':>6s}")

        prev_ratio = None
        for K in K_VALUES:
            ratios = []
            for s in SEEDS:
                eff_o = all_seeds[s][f"{mode}_K{K}_ORIG"]["static"]["bucket_eff"]
                eff_p = all_seeds[s][f"{mode}_K{K}_PERM"]["static"]["bucket_eff"]
                ratios.append(eff_p / eff_o)
            mn = np.mean(ratios)
            trend = ""
            if prev_ratio is not None:
                trend = "UP" if mn > prev_ratio + 0.005 else (
                    "DOWN" if mn < prev_ratio - 0.005 else "FLAT")
            prev_ratio = mn
            print(f"{K:5d}  {mn:8.4f}  {np.std(ratios):8.4f}  "
                  f"{np.min(ratios):8.4f}  {np.max(ratios):8.4f}  {trend:>6s}")

        # ---- Table 3: Gini ratio ORIG/PERM ----
        print(f"\n--- Table 3: Gini Ratio (ORIG/PERM, 5-seed) ---")
        print(f"{'K':>5s}  {'mean':>8s}  {'std':>8s}")

        for K in K_VALUES:
            ratios = []
            for s in SEEDS:
                g_o = all_seeds[s][f"{mode}_K{K}_ORIG"]["static"]["bucket_gini"]
                g_p = all_seeds[s][f"{mode}_K{K}_PERM"]["static"]["bucket_gini"]
                ratios.append(g_o / g_p if g_p > 0 else 0)
            print(f"{K:5d}  {np.mean(ratios):8.4f}  {np.std(ratios):8.4f}")

        # ---- Table 4: Active sector count (ORIG vs PERM) ----
        print(f"\n--- Table 4: Active Sector Count (5-seed mean) ---")
        print(f"{'K':>5s}  {'ORIG_sect':>10s}  {'PERM_sect':>10s}  "
              f"{'ratio':>8s}  {'ORIG/K':>8s}  {'PERM/K':>8s}")

        for K in K_VALUES:
            o_sects, p_sects = [], []
            for s in SEEDS:
                o_sects.append(all_seeds[s][f"{mode}_K{K}_ORIG"]["static"]["n_active_sectors"])
                p_sects.append(all_seeds[s][f"{mode}_K{K}_PERM"]["static"]["n_active_sectors"])
            om, pm = np.mean(o_sects), np.mean(p_sects)
            print(f"{K:5d}  {om:10.1f}  {pm:10.1f}  "
                  f"{pm/om:8.4f}  {om/K:8.4f}  {pm/K:8.4f}")

        # ---- Table 5: Sector entropy gap ----
        print(f"\n--- Table 5: Sector Entropy (5-seed mean) ---")
        print(f"{'K':>5s}  {'ORIG_ent':>10s}  {'PERM_ent':>10s}  "
              f"{'delta':>8s}  {'max_ent':>8s}  {'ORIG/max':>9s}  {'PERM/max':>9s}")

        for K in K_VALUES:
            o_ents, p_ents = [], []
            n_o_sects, n_p_sects = [], []
            for s in SEEDS:
                o_ents.append(all_seeds[s][f"{mode}_K{K}_ORIG"]["static"]["sector_entropy"])
                p_ents.append(all_seeds[s][f"{mode}_K{K}_PERM"]["static"]["sector_entropy"])
                n_o_sects.append(all_seeds[s][f"{mode}_K{K}_ORIG"]["static"]["n_active_sectors"])
                n_p_sects.append(all_seeds[s][f"{mode}_K{K}_PERM"]["static"]["n_active_sectors"])
            oe, pe = np.mean(o_ents), np.mean(p_ents)
            max_ent = np.log(K)
            print(f"{K:5d}  {oe:10.4f}  {pe:10.4f}  "
                  f"{pe-oe:8.4f}  {max_ent:8.4f}  {oe/max_ent:9.4f}  {pe/max_ent:9.4f}")

        # ---- Table 6: Training-weighted distributions ----
        print(f"\n--- Table 6: Training-Weighted Effective Buckets (5-seed mean) ---")
        print(f"{'K':>5s}  {'ORIG_eff':>10s}  {'PERM_eff':>10s}  "
              f"{'ratio':>8s}  {'ORIG_gini':>10s}  {'PERM_gini':>10s}")

        for K in K_VALUES:
            o_effs, p_effs = [], []
            o_ginis, p_ginis = [], []
            for s in SEEDS:
                o_effs.append(all_seeds[s][f"{mode}_K{K}_ORIG"]["training"]["effective_bucket_count"])
                p_effs.append(all_seeds[s][f"{mode}_K{K}_PERM"]["training"]["effective_bucket_count"])
                o_ginis.append(all_seeds[s][f"{mode}_K{K}_ORIG"]["training"]["gini"])
                p_ginis.append(all_seeds[s][f"{mode}_K{K}_PERM"]["training"]["gini"])
            oe, pe = np.mean(o_effs), np.mean(p_effs)
            print(f"{K:5d}  {oe:10.1f}  {pe:10.1f}  "
                  f"{pe/oe:8.4f}  {np.mean(o_ginis):10.4f}  {np.mean(p_ginis):10.4f}")

    # ---- Overall diagnosis ----
    print(f"\n{'='*80}")
    print("DIAGNOSIS")
    print(f"{'='*80}")

    # Check shell counts
    all_shells_one = True
    for mode in ["BASE", "TRANS"]:
        all_seeds_data = load_results(mode, SEEDS)
        for K in K_VALUES:
            for typ in ["ORIG", "PERM"]:
                for s in SEEDS:
                    n_sh = all_seeds_data[s][f"{mode}_K{K}_{typ}"]["static"]["n_active_shells"]
                    if n_sh != 1:
                        all_shells_one = False

    if all_shells_one:
        print("\n[FINDING 1] ALL runs have exactly 1 active shell across K=75-150.")
        print("  The K=100 dip is NOT a shell-transition effect.")
        print("  Shell partitioning does not engage in this K range.")
        print("  Diagnosis: SECTOR-DISCRETIZATION driven.\n")
    else:
        print("\n[FINDING 1] Shell counts vary across K — shell transition detected.\n")

    # Check where sector saturation happens
    print("[FINDING 2] Sector occupancy analysis:")
    for mode in ["BASE", "TRANS"]:
        all_seeds_data = load_results(mode, SEEDS)
        print(f"\n  {mode}:")
        for K in K_VALUES:
            o_sects = [all_seeds_data[s][f"{mode}_K{K}_ORIG"]["static"]["n_active_sectors"]
                       for s in SEEDS]
            p_sects = [all_seeds_data[s][f"{mode}_K{K}_PERM"]["static"]["n_active_sectors"]
                       for s in SEEDS]
            o_effs = [all_seeds_data[s][f"{mode}_K{K}_ORIG"]["static"]["bucket_eff"]
                      for s in SEEDS]
            p_effs = [all_seeds_data[s][f"{mode}_K{K}_PERM"]["static"]["bucket_eff"]
                      for s in SEEDS]

            om_s = np.mean(o_sects)
            pm_s = np.mean(p_sects)
            ratio = np.mean(p_effs) / np.mean(o_effs)

            saturated_o = "SATURATED" if abs(om_s - K) < 1 else f"{om_s/K:.2%} of K"
            saturated_p = "SATURATED" if abs(pm_s - K) < 1 else f"{pm_s/K:.2%} of K"

            print(f"    K={K:3d}: ORIG {om_s:5.1f} sectors ({saturated_o}), "
                  f"PERM {pm_s:5.1f} sectors ({saturated_p}), "
                  f"eff_ratio={ratio:.4f}")

    # Compute eff ratio transitions
    print(f"\n[FINDING 3] Eff ratio transition detail:")
    for mode in ["BASE", "TRANS"]:
        all_seeds_data = load_results(mode, SEEDS)
        print(f"\n  {mode}:")
        ratios_by_k = {}
        for K in K_VALUES:
            rs = []
            for s in SEEDS:
                eo = all_seeds_data[s][f"{mode}_K{K}_ORIG"]["static"]["bucket_eff"]
                ep = all_seeds_data[s][f"{mode}_K{K}_PERM"]["static"]["bucket_eff"]
                rs.append(ep / eo)
            ratios_by_k[K] = np.mean(rs)

        for i, K in enumerate(K_VALUES):
            mark = ""
            if i > 0 and ratios_by_k[K] < ratios_by_k[K_VALUES[i-1]] - 0.005:
                mark = " ← DIP"
            elif i > 0 and ratios_by_k[K] > ratios_by_k[K_VALUES[i-1]] + 0.005:
                mark = " ← RISE"
            print(f"    K={K:3d}: ratio={ratios_by_k[K]:.4f}{mark}")

        # Check if dip is present
        # Look for local min between K=75 and K=150
        ks = list(ratios_by_k.keys())
        vals = [ratios_by_k[k] for k in ks]
        local_mins = []
        for i in range(1, len(vals) - 1):
            if vals[i] < vals[i-1] and vals[i] < vals[i+1]:
                local_mins.append(ks[i])
        if local_mins:
            print(f"    Local minimum at K={local_mins}")
        else:
            print(f"    No local minimum found (monotonic or endpoint dip)")


if __name__ == "__main__":
    main()
