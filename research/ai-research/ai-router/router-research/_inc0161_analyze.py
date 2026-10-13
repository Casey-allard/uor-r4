#!/usr/bin/env python3
"""Compile INC-0161 multi-seed results."""
import json
import numpy as np

data = {}
for s in range(5):
    with open(f"results/parsed/inc0161_routing_cost_seed{s}.json") as f:
        data[s] = json.load(f)

route_ids = [r["route_id"] for r in data[0]["results"]]
metrics = {rid: {"eff": [], "gini": [], "top_half": [], "unique": []}
           for rid in route_ids}

for s in range(5):
    for r in data[s]["results"]:
        rid = r["route_id"]
        tc = r["training_cost"]
        metrics[rid]["eff"].append(tc["effective_bucket_count"])
        metrics[rid]["gini"].append(tc["training_gini"])
        metrics[rid]["top_half"].append(tc["top_half_concentration"])
        metrics[rid]["unique"].append(tc["unique_buckets"])

print("=" * 100)
print(f"{'Route':<20} {'eff_bucket':>14} {'gini':>14} {'top_half':>14} {'unique':>14}")
print("-" * 100)
for rid in route_ids:
    m = metrics[rid]
    em, es = np.mean(m["eff"]), np.std(m["eff"])
    gm, gs = np.mean(m["gini"]), np.std(m["gini"])
    tm, ts = np.mean(m["top_half"]), np.std(m["top_half"])
    um, us = np.mean(m["unique"]), np.std(m["unique"])
    print(f"{rid:<20} {em:7.2f}+/-{es:4.2f} {gm:7.4f}+/-{gs:.4f} {tm:7.4f}+/-{ts:.4f} {um:7.1f}+/-{us:.1f}")

pairs = [
    ("TRANS_K25_ORIG", "TRANS_K25_PERM"),
    ("BASE_K25_ORIG", "BASE_K25_PERM"),
    ("TRANS_K50_ORIG", "TRANS_K50_PERM"),
    ("BASE_K50_ORIG", "BASE_K50_PERM"),
    ("TRANS_K75_ORIG", "TRANS_K75_PERM"),
    ("BASE_K75_ORIG", "BASE_K75_PERM"),
    ("TRANS_K100_ORIG", "TRANS_K100_PERM"),
    ("BASE_K100_ORIG", "BASE_K100_PERM"),
]

print()
print("=" * 100)
print("COMPARISON RATIOS (mean +/- std across 5 seeds)")
print("-" * 100)

for orig, perm in pairs:
    eff_r = [metrics[perm]["eff"][s] / metrics[orig]["eff"][s] for s in range(5)]
    gini_r = [metrics[orig]["gini"][s] / metrics[perm]["gini"][s] for s in range(5)]
    th_d = [metrics[orig]["top_half"][s] - metrics[perm]["top_half"][s] for s in range(5)]

    all_above = all(r > 1.0 for r in eff_r)
    tag = "ALL>1.0" if all_above else "FAIL"

    print(f"\n{orig} vs {perm}:")
    print(f"  eff_ratio(PERM/ORIG): {np.mean(eff_r):.4f} +/- {np.std(eff_r):.4f}  [{tag}]  per-seed: {[round(r,4) for r in eff_r]}")
    print(f"  gini_ratio(ORIG/PERM): {np.mean(gini_r):.4f} +/- {np.std(gini_r):.4f}  per-seed: {[round(r,4) for r in gini_r]}")
    print(f"  top_half_diff(ORIG-PERM): {np.mean(th_d):.4f} +/- {np.std(th_d):.4f}")

print()
print("=" * 100)
print("SUCCESS CRITERIA CHECK")
print("-" * 100)

# 1. TRANS K=75 eff_ratio > 1.2x mean
tk75_eff = [metrics["TRANS_K75_PERM"]["eff"][s] / metrics["TRANS_K75_ORIG"]["eff"][s] for s in range(5)]
m1 = np.mean(tk75_eff)
print(f"1. TRANS K=75 eff_ratio mean = {m1:.4f} (threshold: > 1.2)  => {'PASS' if m1 > 1.2 else 'FAIL'}")

# 2. Gini ratio > 1.3x at TRANS K=75
tk75_gini = [metrics["TRANS_K75_ORIG"]["gini"][s] / metrics["TRANS_K75_PERM"]["gini"][s] for s in range(5)]
m2 = np.mean(tk75_gini)
print(f"2. TRANS K=75 gini_ratio mean = {m2:.4f} (threshold: > 1.3)  => {'PASS' if m2 > 1.3 else 'FAIL'}")

# 3. Compression at >= 3 of 4 K values
kp = 0
for k in [25, 50, 75, 100]:
    orig_k = f"TRANS_K{k}_ORIG"
    perm_k = f"TRANS_K{k}_PERM"
    ratios = [metrics[perm_k]["eff"][s] / metrics[orig_k]["eff"][s] for s in range(5)]
    m = np.mean(ratios)
    passed = m > 1.2
    if passed:
        kp += 1
    print(f"   K={k}: TRANS eff_ratio mean = {m:.4f}  => {'PASS' if passed else 'FAIL'}")
print(f"3. Compression at >= 3 K values: {kp}/4  => {'PASS' if kp >= 3 else 'FAIL'}")

# 4. Every seed > 1.0x at TRANS K=75
all_s = all(r > 1.0 for r in tk75_eff)
print(f"4. Every seed > 1.0x at TRANS K=75: {all_s}  => {'PASS' if all_s else 'FAIL'}")

overall = m1 > 1.2 and m2 > 1.3 and kp >= 3 and all_s
print(f"\nOVERALL: {'PASS -- KEEP' if overall else 'FAIL'}")
