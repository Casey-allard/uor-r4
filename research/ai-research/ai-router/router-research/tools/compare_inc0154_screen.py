#!/usr/bin/env python3
"""INC-0154 results analysis — event-gate efficiency interaction screen."""
import json
import sys

def main():
    path = "results/analysis/inc0154_event_gate_efficiency_screen.json"
    with open(path) as f:
        d = json.load(f)
    routes = d["results"]

    order = [
        "BASE_K75_ORIG_GATEOFF", "BASE_K75_ORIG_T070",
        "BASE_K75_PERM_GATEOFF", "BASE_K75_PERM_T070",
        "TRANS_K75_ORIG_GATEOFF", "TRANS_K75_ORIG_T070",
        "TRANS_K75_PERM_GATEOFF", "TRANS_K75_PERM_T070",
    ]

    print(f"{'Route':<30} {'MSE':>10} {'gate_mean':>10} {'gate_active':>12} {'gate_err_mean':>14}")
    print("-" * 78)
    for rid in order:
        m = routes[rid][0]["metrics"]
        mse = m["test_mse_after"]
        gm = m["event_gate_mean"]
        ga = m["event_gate_active_frac"]
        ge = m["event_gate_error_mean"]
        print(f"{rid:<30} {mse:>10.6f} {gm:>10.6f} {ga:>12.4f} {ge:>14.6f}")

    print()
    print("=== KEY COMPARISONS ===")
    print()

    # gate_mean deltas
    for prefix, tag in [("BASE", "BASE_K75"), ("TRANS", "TRANS_K75")]:
        orig_gm = routes[f"{tag}_ORIG_T070"][0]["metrics"]["event_gate_mean"]
        perm_gm = routes[f"{tag}_PERM_T070"][0]["metrics"]["event_gate_mean"]
        delta = orig_gm - perm_gm
        print(f"{prefix} gate_mean: ORIG={orig_gm:.6f}  PERM={perm_gm:.6f}  delta={delta:+.6f} ({delta*100:+.1f}pp)")

    print()

    # MSE degradation from gating
    for prefix, tag in [("BASE", "BASE_K75"), ("TRANS", "TRANS_K75")]:
        mo = routes[f"{tag}_ORIG_GATEOFF"][0]["metrics"]["test_mse_after"]
        mg = routes[f"{tag}_ORIG_T070"][0]["metrics"]["test_mse_after"]
        mp = routes[f"{tag}_PERM_GATEOFF"][0]["metrics"]["test_mse_after"]
        mgp = routes[f"{tag}_PERM_T070"][0]["metrics"]["test_mse_after"]
        orig_ratio = (mg - mo) / mo * 100 if mo > 0 else 0
        perm_ratio = (mgp - mp) / mp * 100 if mp > 0 else 0
        print(f"{prefix} MSE degradation from gating: ORIG={orig_ratio:+.2f}%  PERM={perm_ratio:+.2f}%")

    print()

    # error_mean deltas (the raw error the gate sees)
    for prefix, tag in [("BASE", "BASE_K75"), ("TRANS", "TRANS_K75")]:
        orig_err = routes[f"{tag}_ORIG_T070"][0]["metrics"]["event_gate_error_mean"]
        perm_err = routes[f"{tag}_PERM_T070"][0]["metrics"]["event_gate_error_mean"]
        delta = orig_err - perm_err
        print(f"{prefix} error_mean: ORIG={orig_err:.6f}  PERM={perm_err:.6f}  delta={delta:+.6f}")

    print()

    # Absolute MSE comparison
    for prefix, tag in [("BASE", "BASE_K75"), ("TRANS", "TRANS_K75")]:
        orig_off = routes[f"{tag}_ORIG_GATEOFF"][0]["metrics"]["test_mse_after"]
        orig_on = routes[f"{tag}_ORIG_T070"][0]["metrics"]["test_mse_after"]
        perm_off = routes[f"{tag}_PERM_GATEOFF"][0]["metrics"]["test_mse_after"]
        perm_on = routes[f"{tag}_PERM_T070"][0]["metrics"]["test_mse_after"]
        print(f"{prefix} MSE: ORIG_OFF={orig_off:.6f}  ORIG_ON={orig_on:.6f}  PERM_OFF={perm_off:.6f}  PERM_ON={perm_on:.6f}")

    # Success check
    print()
    print("=== DECISION CRITERIA ===")
    base_orig_gm = routes["BASE_K75_ORIG_T070"][0]["metrics"]["event_gate_mean"]
    base_perm_gm = routes["BASE_K75_PERM_T070"][0]["metrics"]["event_gate_mean"]
    trans_orig_gm = routes["TRANS_K75_ORIG_T070"][0]["metrics"]["event_gate_mean"]
    trans_perm_gm = routes["TRANS_K75_PERM_T070"][0]["metrics"]["event_gate_mean"]

    base_delta = (base_perm_gm - base_orig_gm) * 100
    trans_delta = (trans_perm_gm - trans_orig_gm) * 100

    print(f"BASE: PERM gate_mean - ORIG gate_mean = {base_delta:+.1f}pp (need >10pp for success)")
    print(f"TRANS: PERM gate_mean - ORIG gate_mean = {trans_delta:+.1f}pp (need >10pp for success)")

    if base_delta > 10 and trans_delta > 10:
        print("VERDICT: SUCCESS — geometric routing produces fewer gate firings")
    elif base_delta < 5 and trans_delta < 5:
        print("VERDICT: FALSIFIED — event-gate efficiency is routing-agnostic")
    else:
        print("VERDICT: INTERMEDIATE — partial signal, may need refine or more seeds")

if __name__ == "__main__":
    main()
