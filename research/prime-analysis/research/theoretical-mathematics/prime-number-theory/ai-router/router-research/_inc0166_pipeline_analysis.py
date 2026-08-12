#!/usr/bin/env python3
"""Analyze INC-0166 boundary audit results from the standard pipeline."""
import json
import os
from collections import defaultdict
import numpy as np

parsed_dir = "results/parsed"
runs = defaultdict(list)

for fn in sorted(os.listdir(parsed_dir)):
    if not fn.startswith("inc0166_boundary_audit_") or not fn.endswith(".json"):
        continue
    with open(os.path.join(parsed_dir, fn)) as f:
        d = json.load(f)
    if not d.get("parsed"):
        continue
    args = d.get("args", {})
    metrics = d.get("metrics", {})
    parts = fn.replace("inc0166_boundary_audit_", "").split("_seed")
    rid = parts[0]

    runs[rid].append({
        "K": args.get("K"),
        "sector_mode": args.get("sector_mode"),
        "input_transform": args.get("input_transform", "none"),
        "seed": args.get("seed"),
        "eval_shells": metrics.get("eval_shells"),
        "eval_sectors": metrics.get("eval_sectors"),
        "buckets": metrics.get("buckets"),
        "slots_used": metrics.get("slots_used"),
        "shell_pmax": metrics.get("shell_pmax"),
        "sector_pmax": metrics.get("sector_pmax"),
        "shell_entropy": metrics.get("shell_entropy"),
        "sector_entropy": metrics.get("sector_entropy"),
        "pmax_before": metrics.get("pmax_before"),
        "pmax_after": metrics.get("pmax_after"),
        "entropy_before": metrics.get("entropy_before"),
        "entropy_after": metrics.get("entropy_after"),
        "test_unseen_rate": metrics.get("test_unseen_rate"),
        "new_slots": metrics.get("new_slots"),
        "accepted_splits": metrics.get("accepted_splits"),
    })

print("=" * 100)
print("INC-0166 Boundary Audit — Pipeline Results (5-seed means)")
print("=" * 100)

header = f"{'Route':<22} {'K':>3} {'shells':>6} {'sectors':>7} {'buckets':>7} {'sh_ent':>7} {'se_ent':>7} {'pmax':>7} {'unseen':>6}"
print(header)
print("-" * 90)
for rid in sorted(runs.keys()):
    rlist = runs[rid]
    K = rlist[0]["K"]
    shells = np.mean([r["eval_shells"] for r in rlist])
    sectors = np.mean([r["eval_sectors"] for r in rlist])
    buckets = np.mean([r["buckets"] for r in rlist])
    sh_ent = np.mean([r["shell_entropy"] for r in rlist])
    se_ent = np.mean([r["sector_entropy"] for r in rlist])
    pmax = np.mean([r["pmax_after"] for r in rlist])
    unseen = np.mean([r["test_unseen_rate"] for r in rlist])
    print(f"{rid:<22} {K:>3} {shells:>6.1f} {sectors:>7.1f} {buckets:>7.1f} {sh_ent:>7.4f} {se_ent:>7.4f} {pmax:>7.4f} {unseen:>6.4f}")

# Effective bucket ratio analysis (PERM/ORIG)
print("\n" + "=" * 100)
print("Effective Bucket Ratio (PERM/ORIG, 5-seed mean)")
print("=" * 100)

K_vals = sorted(set(r["K"] for rlist in runs.values() for r in rlist))

for mode_prefix in ["TRANS", "BASE"]:
    print(f"\n--- {mode_prefix} ---")
    for K in K_vals:
        orig_key = f"{mode_prefix}_K{K}_ORIG"
        perm_key = f"{mode_prefix}_K{K}_PERM"
        if orig_key in runs and perm_key in runs:
            orig_buckets = np.mean([r["buckets"] for r in runs[orig_key]])
            perm_buckets = np.mean([r["buckets"] for r in runs[perm_key]])
            ratio = perm_buckets / orig_buckets if orig_buckets > 0 else float("nan")
            orig_sectors = np.mean([r["eval_sectors"] for r in runs[orig_key]])
            perm_sectors = np.mean([r["eval_sectors"] for r in runs[perm_key]])
            print(f"  K={K:>3}: ratio={ratio:.4f}  (ORIG buckets={orig_buckets:.1f}, PERM={perm_buckets:.1f})  "
                  f"ORIG sectors={orig_sectors:.1f}, PERM sectors={perm_sectors:.1f}")

# Slot growth analysis
print("\n" + "=" * 100)
print("Slot Growth (new_slots and accepted_splits, 5-seed mean)")
print("=" * 100)

for mode_prefix in ["TRANS", "BASE"]:
    print(f"\n--- {mode_prefix} ---")
    for K in K_vals:
        for suffix in ["ORIG", "PERM"]:
            key = f"{mode_prefix}_K{K}_{suffix}"
            if key in runs:
                new_slots = np.mean([r["new_slots"] for r in runs[key]])
                splits = np.mean([r["accepted_splits"] for r in runs[key]])
                slots = np.mean([r["slots_used"] for r in runs[key]])
                print(f"  {key:<22} slots={slots:.1f}  new={new_slots:.1f}  splits={splits:.1f}")

# Per-seed breakdown for K=100 (the dip)
print("\n" + "=" * 100)
print("Per-seed detail for K=100")
print("=" * 100)

for rid in sorted(runs.keys()):
    if "K100" not in rid:
        continue
    print(f"\n{rid}:")
    for r in sorted(runs[rid], key=lambda x: x["seed"]):
        print(f"  seed={r['seed']}  shells={r['eval_shells']}  sectors={r['eval_sectors']}  "
              f"buckets={r['buckets']}  pmax={r['pmax_after']:.4f}  unseen={r['test_unseen_rate']:.4f}")
