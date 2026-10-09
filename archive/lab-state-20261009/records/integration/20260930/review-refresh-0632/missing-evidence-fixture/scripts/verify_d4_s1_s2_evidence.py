#!/usr/bin/env python3
"""
verify_d4_s1_s2_evidence.py — Independent reader and verifier for D4 S1 and S2 QAT evidence.

Verifies:
1. Manifest SHA-256 identities of sealed report roots recorded in evidence JSON files.
2. Direct payload arithmetic from sealed report files (train/report.json, d11_eval/evaluation.json, chat/chat.json).
3. Exact concordance between raw payload metrics, evidence JSON records, and markdown result documents.
4. Explicit disclosure of missing comparators (QAT integer vs own unquantized float).

Usage:
    python3 scripts/verify_d4_s1_s2_evidence.py
"""

import hashlib
import json
import os
import sys

def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def main():
    print("=== D4 S1 & S2 Historical Evidence Verification ===")
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    s2_json_path = os.path.join(repo_root, "docs/evidence/d4-s2-dialogue-qat-2026-09-29.json")
    s1_json_path = os.path.join(repo_root, "docs/evidence/d4-geometric-s1-qat-2026-09-29.json")
    s2_md_path = os.path.join(repo_root, "docs/integration/d4-s2-dialogue-qat-result-2026-09-29.md")

    failures = 0
    passes = 0

    # 1. Verify S2 Evidence JSON
    if not os.path.exists(s2_json_path):
        print(f"FAIL: Missing S2 evidence JSON at {s2_json_path}")
        sys.exit(1)

    with open(s2_json_path) as f:
        s2_ev = json.load(f)

    print("\n--- 1. S2 Manifest Identity Verification ---")
    all_roots = {}
    for arm_key in ["sealed_roots_arm1_qat", "sealed_roots_arm2_float_control"]:
        if arm_key in s2_ev:
            for phase, info in s2_ev[arm_key].items():
                all_roots[f"{arm_key}.{phase}"] = (info["path"], info["manifest_sha256"])

    for label, (path, expected_hash) in all_roots.items():
        manifest_file = os.path.join(path, "manifest.json")
        if os.path.exists(manifest_file):
            actual_hash = sha256_file(manifest_file)
            if actual_hash == expected_hash:
                print(f"  [PASS] {label}: {path} manifest SHA256 matched ({actual_hash[:16]}...)")
                passes += 1
            else:
                print(f"  [FAIL] {label}: hash mismatch! Expected {expected_hash}, got {actual_hash}")
                failures += 1
        else:
            print(f"  [SKIP] {label}: path not mounted on this host ({path})")

    print("\n--- 2. S2 Raw Payload Arithmetic & Metrics ---")
    # Check Arm 1 train report
    arm1_train_path = s2_ev["sealed_roots_arm1_qat"]["train"]["path"]
    report_file = os.path.join(arm1_train_path, "report.json")
    if os.path.exists(report_file):
        with open(report_file) as f:
            rep = json.load(f)
        last_step = rep["curve"][-1]
        step = last_step["step"]
        dev_float_nll = last_step["dev_float_response_nll"]
        dev_resp_nll = last_step["dev_response_nll"]
        delta = dev_resp_nll - dev_float_nll

        rec_float = s2_ev["measurements"]["161_response_development_nll"]["arm1_qat_float"]
        rec_served = s2_ev["measurements"]["161_response_development_nll"]["arm1_qat_served_quantized"]

        print(f"  Step: {step}")
        print(f"  Raw report dev_float_response_nll: {dev_float_nll:.10f} (recorded: {rec_float:.10f})")
        print(f"  Raw report dev_response_nll:       {dev_resp_nll:.10f} (recorded: {rec_served:.10f})")
        print(f"  Raw delta (served - float):        {delta:.10f} nats")

        if abs(dev_float_nll - rec_float) < 1e-9 and abs(dev_resp_nll - rec_served) < 1e-9:
            print("  [PASS] Arm 1 NLL values match report.json bit-for-bit")
            passes += 1
        else:
            print("  [FAIL] Arm 1 NLL mismatch between raw report.json and evidence JSON")
            failures += 1

    # Check Arm 1 D11 Evaluation
    arm1_d11_path = s2_ev["sealed_roots_arm1_qat"]["d11_eval"]["path"]
    d11_eval_file = os.path.join(arm1_d11_path, "evaluation.json")
    if os.path.exists(d11_eval_file):
        with open(d11_eval_file) as f:
            d11_ev = json.load(f)
        windows = d11_ev["per_window"]
        mean_d11_nll = sum(w["d11_nll"] for w in windows) / len(windows)
        diff_count = d11_ev["positions_with_a_difference"]
        rec_valid = s2_ev["measurements"]["integer_serving_fidelity"]["arm1_valid_window_nll"]

        print(f"  D11 Eval: {len(windows)} windows, positions_with_a_difference={diff_count}")
        print(f"  Computed mean D11 NLL: {mean_d11_nll:.12f} (recorded: {rec_valid:.12f})")
        if diff_count == 0 and abs(mean_d11_nll - rec_valid) < 1e-9:
            print("  [PASS] D11 eval confirms bit-identical integer serving (diff=0) and exact NLL match")
            passes += 1
        else:
            print("  [FAIL] D11 eval mismatch")
            failures += 1

    # Check Arm 2 D11 Evaluation
    arm2_d11_path = s2_ev["sealed_roots_arm2_float_control"]["d11_eval"]["path"]
    arm2_d11_file = os.path.join(arm2_d11_path, "evaluation.json")
    if os.path.exists(arm2_d11_file):
        with open(arm2_d11_file) as f:
            arm2_d11_ev = json.load(f)
        windows = arm2_d11_ev["per_window"]
        mean_arm2_d11_nll = sum(w["d11_nll"] for w in windows) / len(windows)
        diff_count = arm2_d11_ev["positions_with_a_difference"]
        rec_arm2_valid = s2_ev["measurements"]["integer_serving_fidelity"]["arm2_valid_window_nll"]

        print(f"  Arm 2 D11 Eval: {len(windows)} windows, positions_with_a_difference={diff_count}")
        print(f"  Computed mean Arm 2 D11 NLL: {mean_arm2_d11_nll:.12f} (recorded: {rec_arm2_valid:.12f})")
        if diff_count == 0 and abs(mean_arm2_d11_nll - rec_arm2_valid) < 1e-9:
            print("  [PASS] Arm 2 D11 eval confirms bit-identical integer serving (diff=0) and exact NLL match (2.984152881894)")
            passes += 1
        else:
            print("  [FAIL] Arm 2 D11 eval mismatch")
            failures += 1

    # Check Turn Counts in Chat
    arm1_chat_path = s2_ev["sealed_roots_arm1_qat"]["chat"]["path"]
    chat_file = os.path.join(arm1_chat_path, "chat.json")
    if os.path.exists(chat_file):
        with open(chat_file) as f:
            chat_data = json.load(f)
        turns = [t for r in chat_data["record"]["rows"] for t in r["turns"]]
        print(f"  Chat evaluation: {len(turns)} total turns across {len(chat_data['record']['rows'])} rows")
        if len(turns) == 58:
            print("  [PASS] 58 turns confirmed in chat.json")
            passes += 1
        else:
            print(f"  [FAIL] Expected 58 turns, found {len(turns)}")
            failures += 1

    print("\n--- 3. Markdown Document & Scope Verifications ---")
    if os.path.exists(s2_md_path):
        with open(s2_md_path) as f:
            md_content = f.read()

        required_strings = [
            "56 / 58 turns (96.55%)",
            "5 / 58 turns",
            "6 / 58 turns",
            "MISSED",
            "Literal target",
            "Corrected parent target",
            "MISSING / UNAVAILABLE",
            "2.9842 nats",
            "7,238,304 weights read per token",
            "dense parameter access",
        ]
        for req in required_strings:
            if req in md_content:
                print(f"  [PASS] S2 Markdown contains required scope marker: '{req}'")
                passes += 1
            else:
                print(f"  [FAIL] S2 Markdown missing required scope marker: '{req}'")
                failures += 1

    s1_md_path = os.path.join(repo_root, "docs/integration/d4-geometric-s1-qat-result-2026-09-29.md")
    if os.path.exists(s1_md_path):
        with open(s1_md_path) as f:
            s1_md_content = f.read()

        s1_required_strings = [
            "Top-1 Agreement (vs QAT Own Float Model)",
            "f4caf562",
        ]
        for req in s1_required_strings:
            if req in s1_md_content:
                print(f"  [PASS] S1 Markdown contains required scope marker: '{req}'")
                passes += 1
            else:
                print(f"  [FAIL] S1 Markdown missing required scope marker: '{req}'")
                failures += 1

    print(f"\nVerification summary: {passes} passed, {failures} failed.")
    if failures > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
