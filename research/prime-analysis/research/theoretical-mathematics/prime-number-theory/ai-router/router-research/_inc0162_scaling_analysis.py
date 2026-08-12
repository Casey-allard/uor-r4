#!/usr/bin/env python3
"""INC-0162: Compile scaling law from INC-0161 (K=25-100) + INC-0162 (K=150,200).

Fits: effective_buckets = c * K^alpha
Reports alpha_ORIG vs alpha_PERM for TRANS and BASE.
"""
import json
import numpy as np

K_VALUES = [25, 50, 75, 100, 150, 200]
SEEDS = [0, 1, 2, 3, 4]

# Route naming patterns
ROUTE_TYPES = ["TRANS_ORIG", "TRANS_PERM", "BASE_ORIG", "BASE_PERM"]


def route_id(rtype, k):
    parts = rtype.split("_")
    return f"{parts[0]}_K{k}_{parts[1]}"


# Collect effective_bucket_count for every (route_type, K, seed)
# shape: {route_type: {K: [seed0, seed1, ...]}}
eff_data = {rt: {k: [] for k in K_VALUES} for rt in ROUTE_TYPES}
gini_data = {rt: {k: [] for k in K_VALUES} for rt in ROUTE_TYPES}
top_half_data = {rt: {k: [] for k in K_VALUES} for rt in ROUTE_TYPES}

for seed in SEEDS:
    # K=25,50,75,100 from INC-0161
    with open(f"results/parsed/inc0161_routing_cost_seed{seed}.json") as f:
        d161 = json.load(f)
    for r in d161["results"]:
        rid = r["route_id"]
        k = r["args"]["K"]
        tc = r["training_cost"]
        for rt in ROUTE_TYPES:
            if rid == route_id(rt, k):
                eff_data[rt][k].append(tc["effective_bucket_count"])
                gini_data[rt][k].append(tc["training_gini"])
                top_half_data[rt][k].append(tc["top_half_concentration"])

    # K=150,200 from INC-0162
    with open(f"results/parsed/inc0162_scaling_seed{seed}.json") as f:
        d162 = json.load(f)
    for r in d162["results"]:
        rid = r["route_id"]
        k = r["args"]["K"]
        tc = r["training_cost"]
        for rt in ROUTE_TYPES:
            if rid == route_id(rt, k):
                eff_data[rt][k].append(tc["effective_bucket_count"])
                gini_data[rt][k].append(tc["training_gini"])
                top_half_data[rt][k].append(tc["top_half_concentration"])

# Verify completeness
for rt in ROUTE_TYPES:
    for k in K_VALUES:
        n = len(eff_data[rt][k])
        assert n == 5, f"Missing data: {rt} K={k} has {n} seeds"

# --- Per-route summary table ---
print("=" * 110)
print(f"{'Route':<20} {'K':>4} {'eff_bucket':>16} {'entropy':>16} {'gini':>16} {'top_half':>16}")
print("-" * 110)
for rt in ROUTE_TYPES:
    for k in K_VALUES:
        eff = eff_data[rt][k]
        gin = gini_data[rt][k]
        th = top_half_data[rt][k]
        ent = [np.log(e) for e in eff]
        em, es = np.mean(eff), np.std(eff)
        hm, hs = np.mean(ent), np.std(ent)
        gm, gs = np.mean(gin), np.std(gin)
        tm, ts = np.mean(th), np.std(th)
        print(f"{rt:<20} {k:>4}  {em:7.2f}+/-{es:5.2f}  {hm:7.4f}+/-{hs:.4f}  {gm:7.4f}+/-{gs:.4f}  {tm:7.4f}+/-{ts:.4f}")
    print()

# --- Compression ratios ---
print("=" * 110)
print("COMPRESSION RATIOS (mean +/- std across 5 seeds)")
print("-" * 110)
print(f"{'K':>4} {'TRANS eff_ratio':>20} {'TRANS gini_ratio':>20} {'BASE eff_ratio':>18} {'BASE gini_ratio':>18}")
print("-" * 110)
comp_trans_eff = {}
comp_base_eff = {}
for k in K_VALUES:
    t_eff = [eff_data["TRANS_PERM"][k][s] / eff_data["TRANS_ORIG"][k][s] for s in range(5)]
    t_gini = [gini_data["TRANS_ORIG"][k][s] / gini_data["TRANS_PERM"][k][s] for s in range(5)]
    b_eff = [eff_data["BASE_PERM"][k][s] / eff_data["BASE_ORIG"][k][s] for s in range(5)]
    b_gini = [gini_data["BASE_ORIG"][k][s] / gini_data["BASE_PERM"][k][s] for s in range(5)]

    comp_trans_eff[k] = np.mean(t_eff)
    comp_base_eff[k] = np.mean(b_eff)

    print(f"{k:>4}  {np.mean(t_eff):7.4f}+/-{np.std(t_eff):.4f}  "
          f"{np.mean(t_gini):7.4f}+/-{np.std(t_gini):.4f}  "
          f"{np.mean(b_eff):7.4f}+/-{np.std(b_eff):.4f}  "
          f"{np.mean(b_gini):7.4f}+/-{np.std(b_gini):.4f}")

# --- Scaling law fit: effective_buckets = c * K^alpha ---
# Linear regression in log-log space: log(eff) = log(c) + alpha * log(K)
print()
print("=" * 110)
print("SCALING LAW FIT: effective_buckets = c * K^alpha")
print("-" * 110)

log_K = np.log(np.array(K_VALUES, dtype=float))

for rt in ROUTE_TYPES:
    # Per-seed fits
    alphas = []
    log_cs = []
    for s in range(5):
        log_eff = np.array([np.log(eff_data[rt][k][s]) for k in K_VALUES])
        # Linear regression: log(eff) = alpha * log(K) + log(c)
        A = np.vstack([log_K, np.ones(len(log_K))]).T
        result = np.linalg.lstsq(A, log_eff, rcond=None)
        alpha, log_c = result[0]
        alphas.append(alpha)
        log_cs.append(log_c)

    am, as_ = np.mean(alphas), np.std(alphas)
    cm, cs_ = np.mean(np.exp(log_cs)), np.std(np.exp(log_cs))

    # Also fit on seed-averaged data
    mean_log_eff = np.array([np.log(np.mean(eff_data[rt][k])) for k in K_VALUES])
    A = np.vstack([log_K, np.ones(len(log_K))]).T
    result = np.linalg.lstsq(A, mean_log_eff, rcond=None)
    alpha_mean, log_c_mean = result[0]

    # R^2 on mean fit
    predicted = alpha_mean * log_K + log_c_mean
    ss_res = np.sum((mean_log_eff - predicted) ** 2)
    ss_tot = np.sum((mean_log_eff - np.mean(mean_log_eff)) ** 2)
    r2 = 1 - ss_res / ss_tot

    print(f"{rt:<15}  alpha = {am:.4f} +/- {as_:.4f}  "
          f"(mean-fit: {alpha_mean:.4f}, R^2={r2:.6f})  "
          f"c = {cm:.2f}")

# --- Key comparisons ---
print()
print("=" * 110)
print("SCALING EXPONENT COMPARISON")
print("-" * 110)

for prefix in ["TRANS", "BASE"]:
    orig_key = f"{prefix}_ORIG"
    perm_key = f"{prefix}_PERM"
    
    alphas_orig = []
    alphas_perm = []
    for s in range(5):
        for rt, alist in [(orig_key, alphas_orig), (perm_key, alphas_perm)]:
            log_eff = np.array([np.log(eff_data[rt][k][s]) for k in K_VALUES])
            A = np.vstack([log_K, np.ones(len(log_K))]).T
            result = np.linalg.lstsq(A, log_eff, rcond=None)
            alist.append(result[0][0])

    ao_m, ao_s = np.mean(alphas_orig), np.std(alphas_orig)
    ap_m, ap_s = np.mean(alphas_perm), np.std(alphas_perm)
    diff = ap_m - ao_m
    passes = ao_m < ap_m

    print(f"{prefix}:  alpha_ORIG = {ao_m:.4f} +/- {ao_s:.4f}  "
          f"alpha_PERM = {ap_m:.4f} +/- {ap_s:.4f}  "
          f"delta = {diff:.4f}  "
          f"ORIG < PERM: {'PASS' if passes else 'FAIL'}")

# --- Compression ratio trend ---
print()
print("=" * 110)
print("COMPRESSION RATIO TREND (monotonically increasing?)")
print("-" * 110)
for prefix in ["TRANS", "BASE"]:
    ratios = []
    for k in K_VALUES:
        orig_key = f"{prefix}_ORIG"
        perm_key = f"{prefix}_PERM"
        r = [eff_data[perm_key][k][s] / eff_data[orig_key][k][s] for s in range(5)]
        ratios.append(np.mean(r))
    monotonic = all(ratios[i] <= ratios[i+1] for i in range(len(ratios)-1))
    print(f"{prefix}: K={K_VALUES} -> ratios={[round(r,3) for r in ratios]}  monotonic: {'YES' if monotonic else 'NO'}")

# --- Overall success check ---
print()
print("=" * 110)
print("SUCCESS CRITERIA")
print("-" * 110)

# Recompute exponents for the final check
checks = []
for prefix in ["TRANS", "BASE"]:
    orig_key = f"{prefix}_ORIG"
    perm_key = f"{prefix}_PERM"
    alphas_o = []
    alphas_p = []
    for s in range(5):
        for rt, alist in [(orig_key, alphas_o), (perm_key, alphas_p)]:
            log_eff = np.array([np.log(eff_data[rt][k][s]) for k in K_VALUES])
            A = np.vstack([log_K, np.ones(len(log_K))]).T
            result = np.linalg.lstsq(A, log_eff, rcond=None)
            alist.append(result[0][0])
    c1 = np.mean(alphas_o) < np.mean(alphas_p)
    print(f"1. alpha_ORIG < alpha_PERM ({prefix}): {np.mean(alphas_o):.4f} < {np.mean(alphas_p):.4f} => {'PASS' if c1 else 'FAIL'}")
    checks.append(c1)

# Compression continues increasing at K=150,200
for prefix in ["TRANS", "BASE"]:
    orig_key = f"{prefix}_ORIG"
    perm_key = f"{prefix}_PERM"
    ratios = []
    for k in K_VALUES:
        r = [eff_data[perm_key][k][s] / eff_data[orig_key][k][s] for s in range(5)]
        ratios.append(np.mean(r))
    increasing = ratios[-1] > ratios[-3]  # K=200 > K=75
    print(f"2. Compression continues increasing at K=200 ({prefix}): {ratios[-1]:.4f} > {ratios[-3]:.4f} => {'PASS' if increasing else 'FAIL'}")
    checks.append(increasing)

# ORIG higher concentration at all 6 K values
for prefix in ["TRANS", "BASE"]:
    orig_key = f"{prefix}_ORIG"
    perm_key = f"{prefix}_PERM"
    all_higher = True
    for k in K_VALUES:
        gini_o = np.mean(gini_data[orig_key][k])
        gini_p = np.mean(gini_data[perm_key][k])
        if gini_o <= gini_p:
            all_higher = False
    print(f"3. ORIG higher concentration at all 6 K ({prefix}): {'PASS' if all_higher else 'FAIL'}")
    checks.append(all_higher)

overall = all(checks)
print(f"\nOVERALL: {'PASS -- KEEP' if overall else 'FAIL'}")
