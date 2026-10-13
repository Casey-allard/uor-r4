#!/usr/bin/env python3
"""Compare poincaré vs ambient correlation results for INC-0153 screen."""
import json

with open("results/analysis/inc0153_event_corr_repar_screen_poincare.json") as f:
    poin = json.load(f)
with open("results/analysis/inc0153_event_corr_repar_screen_ambient.json") as f:
    amb = json.load(f)

print("=== INC-0153 Re-Parameterized Gate Screen ===")
print(f"Gate params: threshold={poin['event_gate_threshold']}, tau={poin['event_gate_tau']}\n")

for ri, route_id in enumerate(["HOPF_BASE_K75", "HOPF_TRANS_K75_L0"]):
    print(f"--- {route_id} ---")
    pr = poin["routes"][ri]
    ar = amb["routes"][ri]

    gate_mean = pr["correlation_error_signal"]["gate_mean"]["mean"]
    gate_std = pr["correlation_error_signal"]["gate_std"]["mean"]
    gate_active = pr["correlation_error_signal"]["gate_active_frac"]["mean"]
    print(f"  Gate: mean={gate_mean:.4f}, std={gate_std:.4f}, active_frac={gate_active:.1%}")

    for signal_name in ["error_signal", "margin_signal"]:
        print(f"  Signal: {signal_name}")
        pc = pr[f"correlation_{signal_name}"]
        ac = ar[f"correlation_{signal_name}"]

        for metric in [
            "roughness_vs_gate_pearson",
            "roughness_vs_gate_spearman",
            "highfreq_vs_gate_pearson",
            "highfreq_vs_gate_spearman",
            "roughness_vs_error_pearson",
            "roughness_vs_error_spearman",
        ]:
            pv = pc[metric]["mean"]
            av = ac[metric]["mean"]
            delta = pv - av
            print(f"    {metric:35s}  poinc={pv:+.4f}  amb={av:+.4f}  delta={delta:+.4f}")
    print()
