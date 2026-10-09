#!/usr/bin/env python3
"""
verify_d4_s1_s2_evidence.py — Claim-presence, evidence concordance, and manifest identity check.

Scope & Notice:
This script performs a consistency, concordance, and manifest-identity check across committed
evidence JSON documents (d4-geometric-s1-qat-2026-09-29.json, d4-s2-dialogue-qat-2026-09-29.json),
result summaries, and sealed report roots on disk. It does NOT perform an independent recomputation
of model training, straight-through gradients, or greedy generation.

If required evidence roots are missing from disk, this script reports UNAVAILABLE/INCOMPLETE and
fails with exit code 2 (unless --allow-unavailable-roots is explicitly supplied).

Usage:
    python3 scripts/verify_d4_s1_s2_evidence.py [--allow-unavailable-roots]
"""

import argparse
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

def verify_manifest_and_payloads(label: str, root_path: str, expected_manifest_hash: str):
    """Verifies that root_path exists, manifest.json matches expected SHA-256, and payload files exist."""
    if not os.path.exists(root_path):
        print(f"  [UNAVAILABLE/INCOMPLETE] {label}: root directory missing: {root_path}")
        return False, "missing_root"

    manifest_file = os.path.join(root_path, "manifest.json")
    if not os.path.exists(manifest_file):
        print(f"  [FAIL] {label}: manifest.json missing in {root_path}")
        return False, "missing_manifest"

    actual_manifest_hash = sha256_file(manifest_file)
    if actual_manifest_hash != expected_manifest_hash:
        print(f"  [FAIL] {label}: manifest SHA256 mismatch! Expected {expected_manifest_hash}, got {actual_manifest_hash}")
        return False, "manifest_mismatch"

    # Check payload files listed in manifest
    try:
        with open(manifest_file) as f:
            manifest_data = json.load(f)
        for item in manifest_data.get("files", []):
            rel_path = item.get("path")
            expected_bytes = item.get("bytes")
            payload_path = os.path.join(root_path, rel_path)
            if not os.path.exists(payload_path):
                print(f"  [FAIL] {label}: payload file {rel_path} missing in {root_path}")
                return False, "missing_payload"
            if expected_bytes is not None and os.path.getsize(payload_path) != expected_bytes:
                print(f"  [FAIL] {label}: payload file {rel_path} size mismatch in {root_path}")
                return False, "payload_size_mismatch"
    except Exception as e:
        print(f"  [FAIL] {label}: error reading manifest payloads: {e}")
        return False, "manifest_read_error"

    print(f"  [PASS] {label}: {root_path} manifest & payloads verified ({actual_manifest_hash[:16]}...)")
    return True, "ok"

def main():
    parser = argparse.ArgumentParser(description="Verify D4 S1 and S2 QAT evidence concordance and manifest identities.")
    parser.add_argument("--allow-unavailable-roots", action="store_true", help="Report missing disk roots as unavailable without failing exit status.")
    args = parser.parse_args()

    print("=== D4 S1 & S2 Evidence Concordance & Manifest Check ===")
    print("Notice: Claim-presence and evidence check; does not re-execute model training.\n")

    here = os.path.dirname(os.path.abspath(__file__))
    if os.path.exists(os.path.join(here, "docs")):
        repo_root = here
    else:
        repo_root = os.path.dirname(here)
    s1_json_path = os.path.join(repo_root, "docs/evidence/d4-geometric-s1-qat-2026-09-29.json")
    s2_json_path = os.path.join(repo_root, "docs/evidence/d4-s2-dialogue-qat-2026-09-29.json")
    s1_md_path = os.path.join(repo_root, "docs/integration/d4-geometric-s1-qat-result-2026-09-29.md")
    s2_md_path = os.path.join(repo_root, "docs/integration/d4-s2-dialogue-qat-result-2026-09-29.md")

    failures = 0
    passes = 0
    unavailable_roots = 0

    # ---------------------------------------------------------
    # 1. S1 Geometric QAT Verification
    # ---------------------------------------------------------
    print("--- 1. S1 Geometric QAT Manifest & Evidence Check ---")
    if not os.path.exists(s1_json_path):
        print(f"FAIL: Missing S1 evidence JSON at {s1_json_path}")
        sys.exit(1)

    with open(s1_json_path) as f:
        s1_ev = json.load(f)

    s1_roots = {
        "s1.arm1_qat.train": (s1_ev["arm1_qat"]["root"], s1_ev["arm1_qat"]["manifest_sha256"]),
        "s1.arm1_qat.export": (s1_ev["arm1_qat"]["export_root"], s1_ev["arm1_qat"]["export_manifest_sha256"]),
        "s1.arm1_qat.lut_eval": (s1_ev["arm1_qat"]["lut_eval_root"], s1_ev["arm1_qat"]["lut_eval_manifest_sha256"]),
        "s1.arm1_qat.d11_eval": (s1_ev["arm1_qat"]["d11_eval_root"], s1_ev["arm1_qat"]["d11_eval_manifest_sha256"]),
        "s1.arm2_float_continuation.train": (s1_ev["arm2_float_continuation"]["root"], s1_ev["arm2_float_continuation"]["manifest_sha256"]),
    }

    for label, (path, expected_hash) in s1_roots.items():
        ok, reason = verify_manifest_and_payloads(label, path, expected_hash)
        if ok:
            passes += 1
        elif reason == "missing_root":
            unavailable_roots += 1
        else:
            failures += 1

    # S1 Payload Concordance
    s1_train_report = os.path.join(s1_ev["arm1_qat"]["root"], "report.json")
    if os.path.exists(s1_train_report):
        with open(s1_train_report) as f:
            rep = json.load(f)
        float_nll = rep.get("qat", {}).get("final_float", {}).get("nll")
        model_sha = rep.get("model_sha256")
        rec_float_nll = s1_ev["arm1_qat"]["final_float_nll"]
        rec_own_model_sha = s1_ev["arm1_qat"].get("own_float_model_sha256")

        if float_nll is not None and abs(float_nll - rec_float_nll) < 1e-9:
            print(f"  [PASS] S1 own-float NLL matches report.json ({float_nll:.10f})")
            passes += 1
        else:
            print(f"  [FAIL] S1 own-float NLL mismatch: {float_nll} vs {rec_float_nll}")
            failures += 1

        if rec_own_model_sha and model_sha == rec_own_model_sha:
            print(f"  [PASS] S1 own-float model SHA-256 matches ({model_sha[:16]}...)")
            passes += 1
        elif rec_own_model_sha:
            print(f"  [FAIL] S1 model SHA-256 mismatch: {model_sha} vs {rec_own_model_sha}")
            failures += 1

    s1_lut_eval_file = os.path.join(s1_ev["arm1_qat"]["lut_eval_root"], "evaluation.json")
    if os.path.exists(s1_lut_eval_file):
        with open(s1_lut_eval_file) as f:
            lut_data = json.load(f)
        top1 = lut_data.get("top1_agreement", 0.0)
        rec_top1 = s1_ev["arm1_qat"]["lut_eval_top1_agreement"]
        if abs(top1 - rec_top1) < 1e-6:
            print(f"  [PASS] S1 LUT top1 agreement matches ({top1 * 100:.2f}%)")
            passes += 1
        else:
            print(f"  [FAIL] S1 LUT top1 mismatch: {top1} vs {rec_top1}")
            failures += 1

    # ---------------------------------------------------------
    # 2. S2 Dialogue QAT Verification
    # ---------------------------------------------------------
    print("\n--- 2. S2 Dialogue QAT Manifest & Evidence Check ---")
    if not os.path.exists(s2_json_path):
        print(f"FAIL: Missing S2 evidence JSON at {s2_json_path}")
        sys.exit(1)

    with open(s2_json_path) as f:
        s2_ev = json.load(f)

    s2_roots = {}
    for arm_key in ["sealed_roots_arm1_qat", "sealed_roots_arm2_float_control"]:
        if arm_key in s2_ev:
            for phase, info in s2_ev[arm_key].items():
                s2_roots[f"s2.{arm_key}.{phase}"] = (info["path"], info["manifest_sha256"])

    for label, (path, expected_hash) in s2_roots.items():
        ok, reason = verify_manifest_and_payloads(label, path, expected_hash)
        if ok:
            passes += 1
        elif reason == "missing_root":
            unavailable_roots += 1
        else:
            failures += 1

    # S2 Payload Concordance
    arm1_train_path = s2_ev["sealed_roots_arm1_qat"]["train"]["path"]
    report_file = os.path.join(arm1_train_path, "report.json")
    if os.path.exists(report_file):
        with open(report_file) as f:
            rep = json.load(f)
        last_step = rep["curve"][-1]
        dev_float_nll = last_step["dev_float_response_nll"]
        dev_resp_nll = last_step["dev_response_nll"]

        rec_float = s2_ev["measurements"]["161_response_development_nll"]["arm1_qat_float"]
        rec_served = s2_ev["measurements"]["161_response_development_nll"]["arm1_qat_served_quantized"]

        if abs(dev_float_nll - rec_float) < 1e-9 and abs(dev_resp_nll - rec_served) < 1e-9:
            print("  [PASS] Arm 1 NLL values match report.json bit-for-bit")
            passes += 1
        else:
            print("  [FAIL] Arm 1 NLL mismatch between raw report.json and evidence JSON")
            failures += 1

    arm1_d11_path = s2_ev["sealed_roots_arm1_qat"]["d11_eval"]["path"]
    d11_eval_file = os.path.join(arm1_d11_path, "evaluation.json")
    if os.path.exists(d11_eval_file):
        with open(d11_eval_file) as f:
            d11_ev = json.load(f)
        windows = d11_ev["per_window"]
        mean_d11_nll = sum(w["d11_nll"] for w in windows) / len(windows)
        diff_count = d11_ev["positions_with_a_difference"]
        rec_valid = s2_ev["measurements"]["integer_serving_fidelity"]["arm1_valid_window_nll"]

        if diff_count == 0 and abs(mean_d11_nll - rec_valid) < 1e-9:
            print("  [PASS] D11 eval confirms bit-identical integer serving (diff=0) and exact NLL match")
            passes += 1
        else:
            print("  [FAIL] D11 eval mismatch")
            failures += 1

    arm2_d11_path = s2_ev["sealed_roots_arm2_float_control"]["d11_eval"]["path"]
    arm2_d11_file = os.path.join(arm2_d11_path, "evaluation.json")
    if os.path.exists(arm2_d11_file):
        with open(arm2_d11_file) as f:
            arm2_d11_ev = json.load(f)
        windows = arm2_d11_ev["per_window"]
        mean_arm2_d11_nll = sum(w["d11_nll"] for w in windows) / len(windows)
        diff_count = arm2_d11_ev["positions_with_a_difference"]
        rec_arm2_valid = s2_ev["measurements"]["integer_serving_fidelity"]["arm2_valid_window_nll"]

        if diff_count == 0 and abs(mean_arm2_d11_nll - rec_arm2_valid) < 1e-9:
            print("  [PASS] Arm 2 D11 eval confirms bit-identical integer serving (diff=0) and exact NLL match")
            passes += 1
        else:
            print("  [FAIL] Arm 2 D11 eval mismatch")
            failures += 1

    arm1_chat_path = s2_ev["sealed_roots_arm1_qat"]["chat"]["path"]
    chat_file = os.path.join(arm1_chat_path, "chat.json")
    if os.path.exists(chat_file):
        with open(chat_file) as f:
            chat_data = json.load(f)
        turns = [t for r in chat_data["record"]["rows"] for t in r["turns"]]
        if len(turns) == 58:
            print("  [PASS] 58 turns confirmed in chat.json")
            passes += 1
        else:
            print(f"  [FAIL] Expected 58 turns, found {len(turns)}")
            failures += 1

    # ---------------------------------------------------------
    # 3. Markdown Document & Scope Concordance
    # ---------------------------------------------------------
    print("\n--- 3. Markdown Document & Scope Concordance ---")
    if os.path.exists(s2_md_path):
        with open(s2_md_path) as f:
            md_content = f.read()

        s2_required = [
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
            "matched only 5/58 (53/58 diverged)",
        ]
        for req in s2_required:
            if req in md_content:
                print(f"  [PASS] S2 Markdown contains required scope marker: '{req}'")
                passes += 1
            else:
                print(f"  [FAIL] S2 Markdown missing required scope marker: '{req}'")
                failures += 1

    if os.path.exists(s1_md_path):
        with open(s1_md_path) as f:
            s1_md_content = f.read()

        s1_required = [
            "Top-1 Agreement (vs QAT Own Float Model)",
            "f4caf562",
            "1.962490",
            "+0.010637",
        ]
        for req in s1_required:
            if req in s1_md_content:
                print(f"  [PASS] S1 Markdown contains required scope marker: '{req}'")
                passes += 1
            else:
                print(f"  [FAIL] S1 Markdown missing required scope marker: '{req}'")
                failures += 1

    # Final summary and exit code handling
    print(f"\nVerification summary: {passes} passed, {failures} failed, {unavailable_roots} roots unavailable.")
    if unavailable_roots > 0:
        if args.allow_unavailable_roots:
            print(f"Notice: {unavailable_roots} evidence roots unavailable on host (permitted via --allow-unavailable-roots).")
        else:
            print(f"ERROR: {unavailable_roots} required evidence roots are unavailable/missing from disk.")
            sys.exit(2)

    if failures > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
